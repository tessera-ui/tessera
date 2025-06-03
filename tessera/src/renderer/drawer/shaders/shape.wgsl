struct ShapeUniforms {
    size_cr_is_shadow: vec4f, // size.xy, corner_radius, is_shadow (0.0 or 1.0)
    object_color: vec4f,
    shadow_color: vec4f,
    shadow_params: vec4f, // offset.xy, smoothness, unused
};

@group(0) @binding(0)
var<uniform> shape_params: ShapeUniforms;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f,
    @location(2) local_pos_in: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
    @location(1) local_pos_out: vec2f,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    // Assuming model.position.xy are already the final screen/clip coordinates for the vertex
    out.clip_position = vec4f(model.position.xy, 0.0, 1.0);
    out.local_pos_out = model.local_pos_in; // Pass through local_pos
    return out;
}

// Helper function for SDF
fn sdf_rounded_box(p: vec2f, b: vec2f, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let size = shape_params.size_cr_is_shadow.xy;
    let corner_radius = shape_params.size_cr_is_shadow.z;
    let is_shadow_flag = shape_params.size_cr_is_shadow.w;

    let object_color_uniform = shape_params.object_color;
    let shadow_color_uniform = shape_params.shadow_color;
    let shadow_offset = shape_params.shadow_params.xy;
    let shadow_smoothness = shape_params.shadow_params.z;

    // in.local_pos_out is expected to be in normalized range, e.g., [-0.5, 0.5] for x and y
    let p_normalized = in.local_pos_out;
    // Scale to actual rectangle dimensions, centered at (0,0) for SDF calculation
    let p_scaled_object_space = p_normalized * size;
    let rect_half_size = size * 0.5;

    var final_color: vec4f;

    if (is_shadow_flag == 1.0) { // --- Draw Shadow ---
        let p_scaled_shadow_space = p_scaled_object_space - shadow_offset;
        let dist_shadow = sdf_rounded_box(p_scaled_shadow_space, rect_half_size, corner_radius);

        // Anti-aliasing for shadow edge (pixel-size dependent)
        // Using fwidth for a more robust anti-aliasing width
        let aa_width = fwidth(dist_shadow); // Estimate of how much dist_shadow changes per pixel
        let shadow_alpha = 1.0 - smoothstep(-aa_width, aa_width, dist_shadow);

        // Softness/blur for the shadow (independent of AA)
        // This makes the shadow fade out over 'shadow_smoothness' distance
        let shadow_soft_alpha = smoothstep(shadow_smoothness, 0.0, dist_shadow);
        
        let combined_shadow_alpha = shadow_alpha * shadow_soft_alpha; // Multiply alphas

        if (combined_shadow_alpha <= 0.001) { // Use a small threshold for discarding
            discard;
        }
        final_color = vec4f(shadow_color_uniform.rgb, shadow_color_uniform.a * combined_shadow_alpha);

    } else { // --- Draw Object ---
        let dist_object = sdf_rounded_box(p_scaled_object_space, rect_half_size, corner_radius);

        let aa_width = fwidth(dist_object);
        let object_alpha = 1.0 - smoothstep(-aa_width, aa_width, dist_object);

        if (object_alpha <= 0.001) {
            discard;
        }

        // Option 2: Use color from Uniform (as defined in current ShapeUniforms)
        final_color = vec4f(object_color_uniform.rgb, object_color_uniform.a * object_alpha);
    }

    return final_color;
}