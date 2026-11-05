struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    rect: vec4<f32>,
    color: vec4<f32>,
    uv_rect: vec4<f32>,
};

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    let tex_coords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );

    var out: VertexOutput;
    let pos = vertices[in_vertex_index] * uniforms.rect.zw + uniforms.rect.xy;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = tex_coords[in_vertex_index];
    return out;
}

@group(0) @binding(0)
var t_mask: texture_2d<f32>;
@group(0) @binding(1)
var s_mask: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = uniforms.uv_rect.xy + in.tex_coords * uniforms.uv_rect.zw;
    let mask = textureSample(t_mask, s_mask, uv);
    let alpha = mask.a * uniforms.color.a;
    let rgb = uniforms.color.rgb * alpha;
    return vec4<f32>(rgb, alpha);
}
