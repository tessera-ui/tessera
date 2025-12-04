struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct SampleUniforms {
    origin: vec2<f32>,
    scale: vec2<f32>,
    uv_origin: vec2<f32>,
    uv_scale: vec2<f32>,
    tint: vec4<f32>,
    tint_mode: u32,
    rotation: f32,
}

@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@group(0) @binding(2)
var<uniform> uniforms: SampleUniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );

    var out: VertexOutput;
    let quad_pos = positions[vertex_index];

    let centered = quad_pos - vec2<f32>(0.5, 0.5);
    let c = cos(uniforms.rotation);
    let s = sin(uniforms.rotation);
    let rotated = vec2<f32>(
        centered.x * c - centered.y * s,
        centered.x * s + centered.y * c
    );
    let pos = rotated + vec2<f32>(0.5, 0.5);

    out.clip_position = vec4<f32>(uniforms.origin + pos * uniforms.scale, 0.0, 1.0);
    out.uv = uniforms.uv_origin + quad_pos * uniforms.uv_scale;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas_texture, atlas_sampler, in.uv);
    
    // tint_mode == 1u is Solid, 0u is Multiply (default)
    if (uniforms.tint_mode == 1u) {
        let alpha = sampled.a * uniforms.tint.a;
        return vec4<f32>(uniforms.tint.rgb * alpha, alpha);
    } else {
        let tinted = sampled * uniforms.tint;
        return vec4<f32>(tinted.rgb * tinted.a, tinted.a);
    }
}
