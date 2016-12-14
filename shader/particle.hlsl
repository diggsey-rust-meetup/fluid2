struct VsOutput {
	float4 pos: SV_Position;
	float2 uv: TEXCOORD;
	float4 color: COLOR;
};

VsOutput VS(float2 pos: a_Pos, float4 color: a_Color) {
	VsOutput p = {
		float4(pos, 0.0, 1.0),
		float2(0,0),
		color,
	};
	return p;
}

#define PARTICLE_RADIUS 0.01

[maxvertexcount(4)]
void GS(point VsOutput p[1], inout TriangleStream<VsOutput> vs) {
	VsOutput v;
	v.color = p[0].color;
	v.pos = p[0].pos + float4(-PARTICLE_RADIUS, -PARTICLE_RADIUS, 0, 0);
	v.uv = float2(-1, -1);
	vs.Append(v);
	v.pos = p[0].pos + float4(PARTICLE_RADIUS, -PARTICLE_RADIUS, 0, 0);
	v.uv = float2(1, -1);
	vs.Append(v);
	v.pos = p[0].pos + float4(-PARTICLE_RADIUS, PARTICLE_RADIUS, 0, 0);
	v.uv = float2(-1, 1);
	vs.Append(v);
	v.pos = p[0].pos + float4(PARTICLE_RADIUS, PARTICLE_RADIUS, 0, 0);
	v.uv = float2(1, 1);
	vs.Append(v);
}

float4 PS(VsOutput v): SV_Target {
	float alpha = max(1-dot(v.uv, v.uv), 0);
	return float4(v.color.xyz, v.color.w*alpha);
}
