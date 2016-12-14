use vecmath::*;
use rand;
use std::mem;

#[derive(Debug, Clone, Default)]
pub struct ParticleKinematics {
    pub pos: Vector2<f32>,
    pub vel: Vector2<f32>,
    pub next: Option<usize>,
    pub evel: Vector2<f32>
}

impl ParticleKinematics {
    fn advance(&mut self, acc: &ParticleAcceleration, step: f32) {
        // One step up from most basic Euler integration. Assumes constant acceleration for the time-step.
        let displacement = vec2_add(vec2_scale(self.vel, step), vec2_scale(acc.0, step*step*0.5));
        self.pos = vec2_add(self.pos, displacement);
        self.vel = vec2_add(self.vel, vec2_scale(acc.0, step));
        self.evel = vec2_scale(vec2_add(self.vel, self.evel), 0.5);
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParticleDensityPressure {
    pub density: f32,
    pub pressure: f32,
}
const PARTICLE_MASS: f32 = 0.02;
const PARTICLE_H: f32 = 0.04;
const PARTICLE_REST_DENSITY: f32 = 1200.0;
const PARTICLE_STIFFNESS: f32 = 1000.0;
const PARTICLE_VISCOSITY: f32 = 8.0;
const PARTICLE_G: f32 = 1.8;

fn compute_kernel(r: f32) -> (f32, f32, f32) {
    use std::f32::consts;
    (
        315.0/(64.0 * consts::PI * PARTICLE_H.powi(9)) * (PARTICLE_H*PARTICLE_H-r*r).powi(3),
        -45.0/(consts::PI * PARTICLE_H.powi(6)) * (PARTICLE_H-r).powi(2),
        45.0/(consts::PI * PARTICLE_H.powi(6)) * (PARTICLE_H-r)
    )
}

fn grid_cell(p: Vector2<f32>) -> Vector2<i32> {
    [(p[0]/PARTICLE_H).floor() as i32, (p[1]/PARTICLE_H).floor() as i32]
}

impl ParticleDensityPressure {
    fn recalculate(&mut self, self_k: &ParticleKinematics, other_ks: &[ParticleKinematics], grid: &ParticleGrid) {
        let next_fn = |i| {
            let k: &ParticleKinematics = &other_ks[i];
            (k.pos, k.next)
        };

        self.density = PARTICLE_MASS * (grid.iter_neighbours(self_k.pos, &next_fn).map(|(_, _, r2)| {
            compute_kernel(r2.sqrt()).0
        }).sum::<f32>() + compute_kernel(0.0).0);
        self.pressure = ((self.density / PARTICLE_REST_DENSITY).powi(7) - 1.0)*PARTICLE_STIFFNESS;
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParticleAcceleration(pub Vector2<f32>);

impl ParticleAcceleration {
    fn recalculate(&mut self, self_k: &ParticleKinematics, self_d: &ParticleDensityPressure, other_ks: &[ParticleKinematics], other_ds: &[ParticleDensityPressure], grid: &ParticleGrid) {
        let next_fn = |i| {
            let k: &ParticleKinematics = &other_ks[i];
            (k.pos, k.next)
        };

        let v = PARTICLE_MASS / self_d.density;
        self.0 = vec2_scale(grid.iter_neighbours(self_k.pos, &next_fn).map(|(i, o, r2)| {
            let r = r2.sqrt();
            let k = &other_ks[i];
            let d = &other_ds[i];
            let pacc = vec2_scale(o, v*(self_d.pressure + d.pressure)*compute_kernel(r).1/r);
            let vacc = vec2_scale(vec2_sub(k.evel, self_k.evel), v*PARTICLE_VISCOSITY*compute_kernel(r).2);

            vec2_sub(vacc, pacc)
        }).fold([0.0f32, -PARTICLE_G*self_d.density], vec2_add), 1.0/self_d.density);
    }
}

pub struct ParticleGrid(Vec<Option<usize>>);

impl ParticleGrid {
    pub fn new(count: usize) -> Self {
        ParticleGrid(vec![None; count])
    }
    fn cell_hash(&self, cell: Vector2<i32>) -> usize {
        (cell[0] as usize).wrapping_add((cell[1] as usize).wrapping_mul(87119)) % self.0.len()
    }
    pub fn add(&mut self, k: &mut ParticleKinematics, i: usize) {
        let index = self.cell_hash(grid_cell(k.pos));
        k.next = mem::replace(&mut self.0[index], Some(i));
    }
    pub fn clear(&mut self) {
        for cell in self.0.iter_mut() {
            *cell = None;
        }
    }
    pub fn iter_neighbours<'a, F: Fn(usize) -> (Vector2<f32>, Option<usize>)>(&'a self, pos: Vector2<f32>, next_fn: &'a F) -> ParticleGridNeighbours<'a, F> {
        ParticleGridNeighbours {
            grid: self,
            state: 0,
            next_fn: next_fn,
            current: None,
            origin: pos,
            center: grid_cell(pos)
        }
    }
}

pub struct ParticleGridNeighbours<'a, F: Fn(usize) -> (Vector2<f32>, Option<usize>) + 'a> {
    grid: &'a ParticleGrid,
    state: i32,
    next_fn: &'a F,
    current: Option<usize>,
    origin: Vector2<f32>,
    center: Vector2<i32>
}

impl<'a, F: Fn(usize) -> (Vector2<f32>, Option<usize>) + 'a> Iterator for ParticleGridNeighbours<'a, F> {
    type Item = (usize, Vector2<f32>, f32);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(i) = self.current {
                let (pos, current) = (self.next_fn)(i);
                self.current = current;
                let offset = vec2_sub(pos, self.origin);
                let r2 = vec2_square_len(offset);
                if r2 < PARTICLE_H*PARTICLE_H && r2 > 1e-12 {
                    return Some((i, offset, r2))
                }
            } else if self.state < 9 {
                let offset = [(self.state % 3)-1, (self.state / 3)-1];
                let cell = vec2_add(self.center, offset);
                let index = self.grid.cell_hash(cell);
                self.current = self.grid.0[index];
                self.state += 1;
            } else {
                return None
            }
        }
    }
}

pub struct ParticleSystem {
    kinematics: Vec<ParticleKinematics>,
    density: Vec<ParticleDensityPressure>,
    acceleration: Vec<ParticleAcceleration>,
    grid: ParticleGrid
}

impl ParticleSystem {
    pub fn new(count: usize) -> Self {
        let mut result = ParticleSystem {
            kinematics: vec![Default::default(); count],
            density: vec![Default::default(); count],
            acceleration: vec![Default::default(); count],
            grid: ParticleGrid::new(count)
        };
        result.constrain(|k| {
            k.pos = [rand::random::<f32>()*1.0-1.0, rand::random::<f32>()*2.0-1.0];
        });
        result
    }
    pub fn len(&self) -> usize {
        self.kinematics.len()
    }
    pub fn advance(&mut self, step: f32) {
        for (d, k) in self.density.iter_mut().zip(&self.kinematics) {
            d.recalculate(k, &self.kinematics, &self.grid);
        }
        for ((a, k), d) in self.acceleration.iter_mut().zip(&self.kinematics).zip(&self.density) {
            a.recalculate(k, d, &self.kinematics, &self.density, &self.grid);
        }
        self.grid.clear();
        for (i, (k, a)) in self.kinematics.iter_mut().zip(&self.acceleration).enumerate() {
            k.advance(a, step);
            self.grid.add(k, i);
        }
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(&'a ParticleKinematics, &'a ParticleDensityPressure, &'a ParticleAcceleration)> {
        self.kinematics.iter().zip(&self.density).zip(&self.acceleration).map(|((k, d), a)| (k, d, a))
    }
    pub fn constrain<F: Fn(&mut ParticleKinematics)>(&mut self, f: F) {
        for k in self.kinematics.iter_mut() {
            f(k);
        }
    }
}
