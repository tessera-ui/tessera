struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    rect: vec4<f32>,
    opacity: f32,
};
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    // Shared unit quad is (0,0)-(1,1); map it into the target rect so a single
    // indexed draw covers the image bounds.
    let centered = position * 2.0 - 1.0;
    let clip = uniforms.rect.xy + centered * uniforms.rect.zw;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip, 0.0, 1.0);
    out.tex_coords = vec2<f32>(position.x, 1.0 - position.y);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let alpha = color.a * uniforms.opacity;
    let rgb = color.rgb * uniforms.opacity;
    return vec4<f32>(rgb, alpha);
}
