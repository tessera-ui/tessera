// A simple shader to copy a texture region.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate a full-screen triangle using vertex indices.
    // This is a common trick to avoid needing a vertex buffer for a simple blit.
    let x = f32(i32(in_vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(in_vertex_index) % 2) * 4.0 - 1.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    // Convert clip-space coordinates to UV coordinates [0, 1].
    // The Y-axis is flipped to match wgpu/Vulkan's coordinate system.
    out.uv = vec2<f32>(x * 0.5 + 0.5, y * -0.5 + 0.5);
    return out;
}

// Bindings for the source texture and sampler.
@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var s_source: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the source texture at the given UV coordinate and return its color.
    return textureSample(t_source, s_source, in.uv);
}
