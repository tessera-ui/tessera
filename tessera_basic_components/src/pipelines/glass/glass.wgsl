// Uniform buffer, provided by the pipeline at group 0
struct Uniforms {
    // The position of the top-left corner of the quad in NDC.
    pos: vec2<f32>,
    // The size of the quad in NDC.
    size: vec2<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// Background texture, provided by the renderer at group 1.
// Note: We use textureLoad for now which doesn't require a separate sampler.
@group(1) @binding(0) var t_background: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // Pass UV coordinates to fragment shader
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // Generate the 6 vertices for the two triangles that make up our quad.
    // The vertex positions are in the [0, 1] range.
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), // Triangle 1
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), // Triangle 2
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let V = positions[in_vertex_index];

    // The incoming position is already in NDC. We just need to scale it.
    let ndc_pos = uniforms.pos + V * uniforms.size;

    var out: VertexOutput;
    // In NDC, the Y-axis is often flipped compared to pixel coordinates.
    // We also need to flip the y-coordinate of the position.
    out.clip_position = vec4<f32>(ndc_pos.x, -ndc_pos.y, 0.0, 1.0);
    
    // Calculate UV coordinates for texture sampling.
    // The origin of texture coordinates (UVs) is usually top-left.
    out.uv = V;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the screen dimensions from the texture size.
    let screen_dims = vec2<f32>(textureDimensions(t_background));

    // Convert UV to pixel coordinates for textureLoad.
    // The quad's starting position in screen space needs to be calculated.
    // We can get it by reversing the NDC calculation for the position.
    let screen_pos = (uniforms.pos + vec2<f32>(1.0, 1.0)) * 0.5 * screen_dims;
    let screen_size = uniforms.size * 0.5 * screen_dims;
    let pixel_coords = screen_pos + in.uv * screen_size;

    // Use textureLoad which takes integer pixel coordinates.
    let background_color = textureLoad(
        t_background,
        vec2<i32>(floor(pixel_coords)),
        0 // Mip-map level 0
    );

    // Apply a simple "frosted glass" effect: mix the background with a semi-transparent white.
    let final_color = mix(background_color.rgb, vec3<f32>(0.95), 0.3);

    // Return the final color with some transparency.
    return vec4<f32>(final_color, 0.75);
}