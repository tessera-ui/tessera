struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Uniforms {
    origin: vec2<f32>,
    scale: vec2<f32>,
    tint: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(lo, hi, c > vec3<f32>(0.04045));
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let clip = uniforms.origin + in.position * uniforms.scale;
    out.clip_position = vec4<f32>(clip, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c_srgb = (in.color * uniforms.tint);
    let rgb_lin = srgb_to_linear(c_srgb.rgb);
    let a = c_srgb.a;
    return vec4<f32>(rgb_lin * a, a);
}
