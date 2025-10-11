struct ShapeUniforms {
    corner_radii: vec4f,       // x:tl, y:tr, z:br, w:bl
    primary_color: vec4f,
    border_color: vec4f,
    shadow_color: vec4f,
    render_params: vec4f,
    ripple_params: vec4f,
    ripple_color: vec4f,
    g2_k_value: f32,
    border_width: f32,
    position: vec4f,           // x, y, width, height
    screen_size: vec2f,
};

struct ShapeInstances {
    instances: array<ShapeUniforms>,
};

@group(0) @binding(0)
var<storage, read> uniforms: ShapeInstances;

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) local_pos: vec2f, // Local UV [0, 1]
    @location(1) @interpolate(flat) instance_index: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let instance = uniforms.instances[instance_index];
    
    let global_pos_pixels = instance.position.xy;
    let size_pixels = instance.position.zw;
    let screen_dimensions = instance.screen_size;

    let rect_uv_min = global_pos_pixels / screen_dimensions;
    let rect_uv_max = (global_pos_pixels + size_pixels) / screen_dimensions;

    let local_uvs = array<vec2<f32>, 4>(
        vec2(0.0, 0.0), // Top-left
        vec2(0.0, 1.0), // Bottom-left
        vec2(1.0, 1.0), // Bottom-right
        vec2(1.0, 0.0)  // Top-right
    );

    let indices = array<u32, 6>(0, 1, 2, 0, 2, 3);
    let local_uv = local_uvs[indices[vertex_index]];

    let global_uv = rect_uv_min + local_uv * (rect_uv_max - rect_uv_min);

    let clip_pos = vec2<f32>(
        global_uv.x * 2.0 - 1.0,
        -(global_uv.y * 2.0 - 1.0)
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);
    out.local_pos = local_uv - 0.5; // Center the local coordinates around 0
    out.instance_index = instance_index;
    return out;
}

// p: point to sample (in object space, centered at 0,0)
// b: half-size of the box
// r: corner radii (tl, tr, br, bl) -> (x, y, z, w)
// k: exponent for p-norm
fn sdf_g2_rounded_box(p: vec2f, b: vec2f, r: vec4f, k: f32) -> f32 {
    // Select radius based on the quadrant p is in.
    // In our local coordinates, p.y is negative for the top half and positive for the bottom half.
    var radius: f32;
    if (p.y < 0.0) { // Top half
        if (p.x < 0.0) { // Top-Left
            radius = r.x;
        } else { // Top-Right
            radius = r.y;
        }
    } else { // Bottom half
        if (p.x < 0.0) { // Bottom-Left
            radius = r.w;
        } else { // Bottom-Right
            radius = r.z;
        }
    }

    let q = abs(p) - b + radius;

    let v_x = max(q.x, 0.0);
    let v_y = max(q.y, 0.0);

    var dist_corner_shape: f32;
    if abs(k - 2.0) < 0.001 {
        dist_corner_shape = length(vec2f(v_x, v_y));
    } else {
        if v_x == 0.0 && v_y == 0.0 {
            dist_corner_shape = 0.0;
        } else {
            dist_corner_shape = pow(pow(v_x, k) + pow(v_y, k), 1.0 / k);
        }
    }

    return dist_corner_shape + min(max(q.x, q.y), 0.0) - radius;
}


// SDF for an ellipse
// p: point to sample
// r: radii of the ellipse
fn sdf_ellipse(p: vec2f, r: vec2f) -> f32 {
    if r.x <= 0.0 || r.y <= 0.0 {
        // Return a large value to prevent rendering if radii are invalid
        return 1.0e6;
    }
    // Scales the distance to be in pixel units, which is important for anti-aliasing.
    return (length(p / r) - 1.0) * min(r.x, r.y);
}


// Calculate ripple effect based on distance from ripple center
fn calculate_ripple_effect(dist_to_center: f32, ripple_radius: f32) -> f32 {
    if ripple_radius <= 0.0 {
        return 0.0;
    }
    
    // Create a smooth ripple wave
    let normalized_dist = dist_to_center / max(ripple_radius, 0.001);
    
    // Simple ripple: fade out as we get further from center, with a peak at the edge
    let ripple_wave = 1.0 - abs(normalized_dist - 1.0);
    
    // Smooth falloff to avoid harsh edges
    let smooth_falloff = smoothstep(0.0, 0.3, ripple_wave) * smoothstep(1.5, 0.8, normalized_dist);

    return clamp(smooth_falloff, 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let instance = uniforms.instances[in.instance_index];
    let size = instance.position.zw;
    let corner_radii = instance.corner_radii;
    let border_width = instance.border_width;

    let primary_color_uniform = instance.primary_color;
    let shadow_color_uniform = instance.shadow_color;

    let shadow_offset = instance.render_params.xy;
    let shadow_smoothness = instance.render_params.z;
    let render_mode = instance.render_params.w; // 0.0: fill, 1.0: outline, 2.0: shadow, 3.0: ripple_fill, 4.0: ripple_outline
    
    // Ripple parameters
    let ripple_center = instance.ripple_params.xy;
    let ripple_radius = instance.ripple_params.z;
    let ripple_alpha = instance.ripple_params.w;
    let ripple_color_rgb = instance.ripple_color.rgb;

    // G2 exponent for rounded corners.
    let G2_K_VALUE: f32 = instance.g2_k_value;

    // in.local_pos is expected to be in normalized range, e.g., [-0.5, 0.5] for x and y
    let p_normalized = in.local_pos;
    // Scale to actual rectangle dimensions, centered at (0,0) for SDF calculation
    let p_scaled_object_space = p_normalized * size;
    let rect_half_size = size * 0.5;

    var final_color: vec4f;

    if render_mode == 2.0 { // --- Draw Shadow ---
        let p_scaled_shadow_space = p_scaled_object_space - shadow_offset;
        var dist_shadow: f32;
        if corner_radii.x < 0.0 {
            dist_shadow = sdf_ellipse(p_scaled_shadow_space, rect_half_size);
        } else {
            dist_shadow = sdf_g2_rounded_box(p_scaled_shadow_space, rect_half_size, corner_radii, G2_K_VALUE);
        }

        // Anti-aliasing for shadow edge
        let aa_width_shadow = fwidth(dist_shadow);
        let shadow_alpha = 1.0 - smoothstep(-aa_width_shadow, aa_width_shadow, dist_shadow);

        // Softness/blur for the shadow
        let shadow_soft_alpha = smoothstep(shadow_smoothness, 0.0, dist_shadow);

        let combined_shadow_alpha = shadow_alpha * shadow_soft_alpha;

        if combined_shadow_alpha <= 0.001 {
            discard;
        }
        final_color = vec4f(shadow_color_uniform.rgb, shadow_color_uniform.a * combined_shadow_alpha);

    } else { // --- Draw Object (Fill or Outline) ---
        var dist_object: f32;
        if corner_radii.x < 0.0 {
            dist_object = sdf_ellipse(p_scaled_object_space, rect_half_size);
        } else {
            dist_object = sdf_g2_rounded_box(p_scaled_object_space, rect_half_size, corner_radii, G2_K_VALUE);
        }
        let aa_width_object = fwidth(dist_object);

        if render_mode == 0.0 { // --- Draw Fill ---
            let object_alpha = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);

            if object_alpha <= 0.001 {
                discard;
            }
            final_color = vec4f(primary_color_uniform.rgb, primary_color_uniform.a * object_alpha);

        } else if render_mode == 1.0 { // --- Draw Outline ---
            if border_width <= 0.0 {
                discard;
            }
            // Alpha for the outer edge of the border
            let alpha_outer_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);
            // Alpha for the inner edge of the border (shape shrunk by border_width)
            let alpha_inner_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object + border_width);
            
            // The outline alpha is the difference
            let outline_alpha = alpha_outer_edge - alpha_inner_edge;

            if outline_alpha <= 0.001 {
                discard;
            }
            final_color = vec4f(primary_color_uniform.rgb, primary_color_uniform.a * max(0.0, outline_alpha));

        } else if render_mode == 3.0 { // --- Draw Ripple Fill ---
            let object_alpha = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);

            if object_alpha <= 0.001 {
                discard;
            }

            // Calculate ripple effect
            let p_pixel = p_normalized * size;
            let center_pixel = ripple_center * size;
            let dist_to_ripple_center_pixel = distance(p_pixel, center_pixel);
            
            let min_dimension = min(size.x, size.y);
            let normalized_dist = dist_to_ripple_center_pixel / min_dimension;
            let ripple_effect = calculate_ripple_effect(normalized_dist, ripple_radius);
            let ripple_final_alpha = ripple_effect * ripple_alpha;

            // Blend primary color with ripple effect
            let base_color = vec3f(primary_color_uniform.rgb);
            let blended_color = mix(base_color, ripple_color_rgb, ripple_final_alpha);

            final_color = vec4f(blended_color, primary_color_uniform.a * object_alpha);

        } else if render_mode == 4.0 { // --- Draw Ripple Outline ---
            if border_width <= 0.0 {
                discard;
            }
            let alpha_outer_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);
            let alpha_inner_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object + border_width);
            
            let outline_alpha = alpha_outer_edge - alpha_inner_edge;

            if outline_alpha <= 0.001 {
                discard;
            }

            // Calculate ripple effect
            let p_pixel = p_normalized * size;
            let center_pixel = ripple_center * size;
            let dist_to_ripple_center_pixel = distance(p_pixel, center_pixel);
            
            let min_dimension = min(size.x, size.y);
            let normalized_dist = dist_to_ripple_center_pixel / min_dimension;
            let ripple_effect = calculate_ripple_effect(normalized_dist, ripple_radius);
            let ripple_final_alpha = ripple_effect * ripple_alpha;

            // Blend primary color with ripple effect
            let base_color = vec3f(primary_color_uniform.rgb);
            let blended_color = mix(base_color, ripple_color_rgb, ripple_final_alpha);

            final_color = vec4f(blended_color, primary_color_uniform.a * max(0.0, outline_alpha));

        } else if render_mode == 5.0 { // --- Draw Fill with Outline ---
            // Calculate ripple effect
            let p_pixel = p_normalized * size;
            let center_pixel = ripple_center * size;
            let dist_to_ripple_center_pixel = distance(p_pixel, center_pixel);
            
            let min_dimension = min(size.x, size.y);
            let normalized_dist = dist_to_ripple_center_pixel / min_dimension;
            let ripple_effect = calculate_ripple_effect(normalized_dist, ripple_radius);
            let ripple_final_alpha = ripple_effect * ripple_alpha;

            // Blend primary color with ripple effect
            let ripple_mixed_fill_color = mix(primary_color_uniform.rgb, ripple_color_rgb, ripple_final_alpha);

            if border_width <= 0.0 { // If no border, just do a normal fill
                let object_alpha = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);
                if object_alpha <= 0.001 {
                    discard;
                }
                final_color = vec4f(ripple_mixed_fill_color, primary_color_uniform.a * object_alpha);
            } else {
                let dist_inner_edge = dist_object + border_width;
                let aa = fwidth(dist_object);

                // Smoothly transition from border color to fill color
                // t is 0 for fill, 1 for border
                let t = smoothstep(-aa, aa, dist_inner_edge);
                let blended_color = mix(ripple_mixed_fill_color, instance.border_color.rgb, t);

                // Handle transparency of the whole shape
                let object_alpha = 1.0 - smoothstep(-aa, aa, dist_object);

                if (object_alpha <= 0.001) {
                    discard;
                }
                // Use the alpha from the primary color for the whole object
                final_color = vec4f(blended_color, primary_color_uniform.a * object_alpha);
            }
        } else {
            // Should not happen with valid render_mode
            discard;
        }
    }

    return final_color;
}
