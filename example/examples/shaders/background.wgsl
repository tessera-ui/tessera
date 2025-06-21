// A struct to pass data from the vertex to the fragment shader.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// The texture and sampler for our background image.
@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Use a switch statement to select the vertex position and UV based on the index.
    // This is required because dynamic indexing into arrays of constants is not allowed.
    switch (in_vertex_index) {
        case 0u: { // Top-left
            out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0);
            out.uv = vec2<f32>(0.0, 0.0);
        }
        case 1u: { // Top-right
            out.clip_position = vec4<f32>(1.0, 1.0, 0.0, 1.0);
            out.uv = vec2<f32>(1.0, 0.0);
        }
        case 2u: { // Bottom-left
            out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0);
            out.uv = vec2<f32>(0.0, 1.0);
        }
        default: { // Bottom-right (also covers case 3u)
            out.clip_position = vec4<f32>(1.0, -1.0, 0.0, 1.0);
            out.uv = vec2<f32>(1.0, 1.0);
        }
    }

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture using the UVs received from the vertex shader.
    return textureSample(t_diffuse, s_diffuse, in.uv);
}