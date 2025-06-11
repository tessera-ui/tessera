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
    out.clip_position = vec4f(model.position.xy, 0.0, 1.0);
    out.local_pos_out = model.local_pos_in;
    return out;
}

// Helper function for G2-like SDF using p-norm for rounded corners
// p: point to sample
// b: half-size of the box
// r: corner radius
// k: exponent for p-norm (k=2.0 for G1 circle, k>2.0 for G2-like superellipse)
fn sdf_g2_rounded_box(p: vec2f, b: vec2f, r: f32, k: f32) -> f32 {
    let q = abs(p) - b + r;

    let v_x = max(q.x, 0.0);
    let v_y = max(q.y, 0.0);

    var dist_corner_shape: f32;
    // Use a small epsilon for comparing k to 2.0 to handle potential float inaccuracies
    if (abs(k - 2.0) < 0.001) { // G1 behavior (standard circle)
        dist_corner_shape = length(vec2f(v_x, v_y));
    } else { // G2-like behavior with exponent k
        if (v_x == 0.0 && v_y == 0.0) {
            dist_corner_shape = 0.0;
        } else {
            dist_corner_shape = pow(pow(v_x, k) + pow(v_y, k), 1.0/k);
        }
    }
    
    return dist_corner_shape + min(max(q.x, q.y), 0.0) - r;
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

    // G2 exponent for rounded corners.
    // k=2.0 results in standard G1 circular corners.
    // k > 2.0 (e.g., 2.5, 3.0, 4.0) gives G2-like superelliptical corners.
    let G2_K_VALUE: f32 = 3.0;

    // in.local_pos_out is expected to be in normalized range, e.g., [-0.5, 0.5] for x and y
    let p_normalized = in.local_pos_out;
    // Scale to actual rectangle dimensions, centered at (0,0) for SDF calculation
    let p_scaled_object_space = p_normalized * size;
    let rect_half_size = size * 0.5;

    var final_color: vec4f;

    if (is_shadow_flag == 1.0) { // --- Draw Shadow ---
        let p_scaled_shadow_space = p_scaled_object_space - shadow_offset;
        let dist_shadow = sdf_g2_rounded_box(p_scaled_shadow_space, rect_half_size, corner_radius, G2_K_VALUE);

        // Anti-aliasing for shadow edge
        let aa_width = fwidth(dist_shadow);
        let shadow_alpha = 1.0 - smoothstep(-aa_width, aa_width, dist_shadow);

        // Softness/blur for the shadow
        let shadow_soft_alpha = smoothstep(shadow_smoothness, 0.0, dist_shadow);
        
        let combined_shadow_alpha = shadow_alpha * shadow_soft_alpha;

        if (combined_shadow_alpha <= 0.001) {
            discard;
        }
        final_color = vec4f(shadow_color_uniform.rgb, shadow_color_uniform.a * combined_shadow_alpha);

    } else { // --- Draw Object ---
        let dist_object = sdf_g2_rounded_box(p_scaled_object_space, rect_half_size, corner_radius, G2_K_VALUE);

        let aa_width = fwidth(dist_object);
        let object_alpha = 1.0 - smoothstep(-aa_width, aa_width, dist_object);

        if (object_alpha <= 0.001) {
            discard;
        }
        
        final_color = vec4f(object_color_uniform.rgb, object_color_uniform.a * object_alpha);
    }

    return final_color;
}