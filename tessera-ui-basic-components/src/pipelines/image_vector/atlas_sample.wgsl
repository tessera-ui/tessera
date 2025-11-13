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
}

@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@group(0) @binding(2)
var<uniform> uniforms: SampleUniforms;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(lo, hi, c > vec3<f32>(0.04045));
}

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
    out.clip_position = vec4<f32>(uniforms.origin + quad_pos * uniforms.scale, 0.0, 1.0);
    out.uv = uniforms.uv_origin + quad_pos * uniforms.uv_scale;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas_texture, atlas_sampler, in.uv);
    let tinted = sampled * uniforms.tint;
    let rgb_lin = srgb_to_linear(tinted.rgb);
    let a = tinted.a;
    return vec4<f32>(rgb_lin * a, a);
}
