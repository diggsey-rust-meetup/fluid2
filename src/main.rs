#![feature(conservative_impl_trait)]

extern crate env_logger;
extern crate getopts;
extern crate time;
extern crate glutin;
extern crate rand;
extern crate vecmath;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
#[cfg(target_os = "windows")]
extern crate gfx_device_dx11;
extern crate gfx_window_glutin;
//extern crate gfx_window_glfw;
#[cfg(target_os = "windows")]
extern crate gfx_window_dxgi;

pub use app::ColorFormat;
use gfx::{Bundle, Primitive, ShaderSet, buffer, Bind, Slice};
use gfx::state::Rasterizer;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use fluid::ParticleSystem;

pub mod app;
pub mod shade;
pub mod fluid;

pub type TexFormat = [f32; 4];

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 4] = "a_Color",
    }

    pipeline particles {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

impl Vertex {
    fn new(p: [f32; 2], c: [f32; 4]) -> Vertex {
        Vertex {
            pos: p,
            color: c,
        }
    }
}

struct App<R: gfx::Resources> {
    particles: Bundle<R, particles::Data<R>>,
    system: ParticleSystem,
    vertex_data: Vec<Vertex>,
    time_start: Instant,
}

fn create_shader_set<R: gfx::Resources, F: gfx::Factory<R>>(factory: &mut F, vs_code: &[u8], gs_code: &[u8], ps_code: &[u8]) -> ShaderSet<R> {
    let vs = factory.create_shader_vertex(vs_code).expect("Failed to compile vertex shader");
    let gs = factory.create_shader_geometry(gs_code).expect("Failed to compile geometry shader");
    let ps = factory.create_shader_pixel(ps_code).expect("Failed to compile pixel shader");
    ShaderSet::Geometry(vs, gs, ps)
}

impl<R: gfx::Resources> app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(mut factory: F, init: app::Init<R>) -> Self {
        use gfx::traits::FactoryExt;

        let (_width, _height, _, _) = init.color.get_dimensions();

        let vs = shade::Source {
            hlsl_40:  include_bytes!("../data/vs_particle.fx"),
            .. shade::Source::empty()
        };
        let gs = shade::Source {
            hlsl_40:  include_bytes!("../data/gs_particle.fx"),
            .. shade::Source::empty()
        };
        let ps = shade::Source {
            hlsl_40:  include_bytes!("../data/ps_particle.fx"),
            .. shade::Source::empty()
        };

        let system = ParticleSystem::new(300);
        let vertex_data = vec![Vertex::new([0.0, 0.0], [1.0, 0.0, 0.0, 1.0]); system.len()];

        let vbuf = factory.create_buffer_dynamic(
            system.len(), buffer::Role::Vertex, Bind::empty()
        ).expect("Failed to create vertex buffer");
        let slice = Slice::new_match_vertex_buffer(&vbuf);

        let shader_set = create_shader_set(
            &mut factory,
            vs.select(init.backend).unwrap(),
            gs.select(init.backend).unwrap(),
            ps.select(init.backend).unwrap(),
        );

        println!("Backend: {:?}", init.backend);

        let pso = factory.create_pipeline_state(
            &shader_set,
            Primitive::PointList,
            Rasterizer::new_fill(),
            particles::new()
        ).unwrap();

        let data = particles::Data {
            vbuf: vbuf.clone(),
            out: init.color,
        };

        App {
            particles: Bundle::new(slice.clone(), pso, data),
            system: system,
            vertex_data: vertex_data,
            time_start: Instant::now(),
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        let delta = self.time_start.elapsed();
        self.time_start = Instant::now();
        let delta = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1000_000_000.0;

        for _ in 0..5 {
            self.system.advance(0.002);
            self.system.constrain(|k| {
                if k.pos[0] <= -1.0 {
                    k.pos[0] = -1.0;
                    k.vel[0] = -0.5*k.vel[0];
                } else if k.pos[0] >= 1.0 {
                    k.pos[0] = 1.0;
                    k.vel[0] = -0.5*k.vel[0];
                }
                if k.pos[1] <= -1.0 {
                    k.pos[1] = -1.0;
                    k.vel[1] = -0.5*k.vel[1];
                } else if k.pos[1] >= 1.0 {
                    k.pos[1] = 1.0;
                    k.vel[1] = -0.5*k.vel[1];
                }
            });
        }
        for (v, (k, d, _)) in self.vertex_data.iter_mut().zip(self.system.iter()) {
            v.pos = k.pos;
            v.color[1] = d.density*0.01;
        }

        encoder.clear(&self.particles.data.out, [0.1, 0.2, 0.3, 1.0]);
        encoder.update_buffer(&self.particles.data.vbuf, &self.vertex_data, 0).unwrap();

        self.particles.encode(encoder);
        thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    use app::Application;
    App::launch_default("Fluid simulation with gfx-rs");
}
