struct VsOutput {
	float4 pos: SV_Position;
	float2 uv: TEXCOORD;
	float4 color: COLOR;
};

struct VsOutput2 {
	float4 pos: SV_Position;
	float2 uv: TEXCOORD;
};

VsOutput VS(float2 pos: a_Pos, float4 color: a_Color) {
	VsOutput p = {
		float4(pos, 0.0, 1.0),
		float2(0,0),
		color,
	};
	return p;
}

VsOutput2 VS_Display(float2 pos: a_Pos, float2 uv: a_TexCoord) {
	VsOutput2 p = {
		float4(pos, 0.0, 1.0),
		uv,
	};
	return p;
}

Texture2D<float> t_Src;
SamplerState t_Src_;

#define PARTICLE_RADIUS 0.05

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
	return float4((alpha*alpha).xxx, 1.0);
}

float4 PS_Display(VsOutput2 v): SV_Target {
	float alpha = t_Src.Sample(t_Src_, v.uv).x;
	float4 black = float4(0.0.xxx, 1.0);
	float4 white = 1.0.xxxx;
	float4 blue = float4(0.0, 0.5, 1.0, 1.0);
	if (alpha > 0.3) {
		return lerp(white, blue, (alpha-0.3)/(1.0-0.3));
	} else if (alpha > 0.2) {
		return lerp(black, white, (alpha-0.2)*10.0);
	} else {
		return black;
	}
}
