struct ShapeUniforms {
    corner_radii: vec4f,       // x:tl, y:tr, z:br, w:bl
    corner_g2: vec4f,          // x:tl, y:tr, z:br, w:bl
    primary_color: vec4f,
    border_color: vec4f,
    shadow_color: vec4f,
    render_params: vec4f,
    ripple_params: vec4f,
    ripple_color: vec4f,
    border_width: f32,
    position: vec4f,           // x, y, width, height
    screen_size: vec2f,
};

struct ShapeInstances {
    instances: array<ShapeUniforms>,
};

@group(0) @binding(0)
var<storage, read> uniforms: ShapeInstances;

struct VertexInput {
    @location(0) position: vec2f,
    @builtin(instance_index) instance_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) local_pos: vec2f, // Local UV [0, 1]
    @location(1) @interpolate(flat) instance_index: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let instance = uniforms.instances[in.instance_index];
    let screen_dimensions = instance.screen_size;

    // `in.position` is the unit quad vertex (0,0 to 1,1).
    // `instance.position` is the rect (x, y, width, height) in pixels.
    let instance_pos = instance.position.xy;
    let instance_size = instance.position.zw;

    // Calculate the vertex's final pixel position.
    let pixel_pos = instance_pos + in.position * instance_size;

    // Convert final pixel position to NDC for the rasterizer.
    let clip_pos = vec2<f32>(
        (pixel_pos.x / screen_dimensions.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_dimensions.y) * -2.0 + 1.0
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);
    // The local UV is simply the incoming unit quad vertex position.
    out.local_pos = in.position - 0.5; // Center around 0 for SDF
    out.instance_index = in.instance_index;
    return out;
}
// p: point to sample (in object space, centered at 0,0)
// b: half-size of the box
// r: corner radii (tl, tr, br, bl) -> (x, y, z, w)
// k: exponent for p-norm per corner
fn sdf_g2_rounded_box(p: vec2f, b: vec2f, r: vec4f, k: vec4f) -> f32 {
    // Select radius based on the quadrant p is in.
    // In our local coordinates, p.y is negative for the top half and positive for the bottom half.
    var radius: f32;
    var corner_k: f32;
    if (p.y < 0.0) { // Top half
        if (p.x < 0.0) { // Top-Left
            radius = r.x;
            corner_k = k.x;
        } else { // Top-Right
            radius = r.y;
            corner_k = k.y;
        }
    } else { // Bottom half
        if (p.x < 0.0) { // Bottom-Left
            radius = r.w;
            corner_k = k.w;
        } else { // Bottom-Right
            radius = r.z;
            corner_k = k.z;
        }
    }

    let q = abs(p) - b + radius;

    let v_x = max(q.x, 0.0);
    let v_y = max(q.y, 0.0);

    var dist_corner_shape: f32;
    if abs(corner_k - 2.0) < 0.001 {
        dist_corner_shape = length(vec2f(v_x, v_y));
    } else {
        if v_x == 0.0 && v_y == 0.0 {
            dist_corner_shape = 0.0;
        } else {
            dist_corner_shape = pow(pow(v_x, corner_k) + pow(v_y, corner_k), 1.0 / corner_k);
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
fn calculate_ripple_mask(dist_to_center: f32, ripple_radius: f32, aa: f32) -> f32 {
    if ripple_radius <= 0.0 {
        return 0.0;
    }

    // Filled circle that expands over time, with a soft edge.
    return 1.0 - smoothstep(ripple_radius - aa, ripple_radius + aa, dist_to_center);
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
    let ripple_bounded = instance.ripple_color.a > 0.5;

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
            dist_shadow = sdf_g2_rounded_box(
                p_scaled_shadow_space,
                rect_half_size,
                corner_radii,
                instance.corner_g2
            );
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
            dist_object = sdf_g2_rounded_box(
                p_scaled_object_space,
                rect_half_size,
                corner_radii,
                instance.corner_g2
            );
        }
        let aa_width_object = fwidth(dist_object);

        let shape_mask = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);

        if render_mode == 0.0 { // --- Draw Fill ---
            if shape_mask <= 0.001 {
                discard;
            }
            final_color = vec4f(primary_color_uniform.rgb, primary_color_uniform.a * shape_mask);

        } else if render_mode == 1.0 { // --- Draw Outline ---
            if border_width <= 0.0 {
                discard;
            }
            // Alpha for the outer edge of the border
            let alpha_outer_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);
            // Alpha for the inner edge of the border (shape shrunk by border_width)
            let alpha_inner_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object + border_width);
            let outline_mask = max(0.0, alpha_outer_edge - alpha_inner_edge);

            if outline_mask <= 0.001 {
                discard;
            }
            final_color = vec4f(primary_color_uniform.rgb, primary_color_uniform.a * outline_mask);

        } else if render_mode == 3.0 || render_mode == 4.0 || render_mode == 5.0 { // --- Fill / Outline / Filled+Outline with Ripple ---
            // Base color (can be transparent for outlined-only, allowing ripple to draw over background).
            var base_rgb: vec3f;
            var base_a: f32;

            if render_mode == 3.0 { // fill base
                base_rgb = primary_color_uniform.rgb;
                base_a = primary_color_uniform.a * shape_mask;
            } else if render_mode == 4.0 { // outline base
                if border_width <= 0.0 {
                    discard;
                }
                let alpha_outer_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object);
                let alpha_inner_edge = 1.0 - smoothstep(-aa_width_object, aa_width_object, dist_object + border_width);
                let outline_mask = max(0.0, alpha_outer_edge - alpha_inner_edge);
                base_rgb = primary_color_uniform.rgb;
                base_a = primary_color_uniform.a * outline_mask;
            } else { // render_mode == 5.0 (filled + outline base)
                if border_width <= 0.0 {
                    if shape_mask <= 0.001 {
                        discard;
                    }
                    base_rgb = primary_color_uniform.rgb;
                    base_a = primary_color_uniform.a * shape_mask;
                } else {
                    let dist_inner_edge = dist_object + border_width;
                    let aa = fwidth(dist_object);
                    let t = smoothstep(-aa, aa, dist_inner_edge);
                    base_rgb = mix(primary_color_uniform.rgb, instance.border_color.rgb, t);
                    base_a = mix(primary_color_uniform.a, instance.border_color.a, t) * shape_mask;
                }
            }

            // Ripple overlay.
            let p_pixel = p_normalized * size;
            let center_pixel = ripple_center * size;
            let dist_to_ripple_center_pixel = distance(p_pixel, center_pixel);
            let min_dimension = min(size.x, size.y);
            let dist_norm = dist_to_ripple_center_pixel / max(min_dimension, 1.0);
            let aa_ripple = max(fwidth(dist_norm), 0.001);
            let ripple_mask = calculate_ripple_mask(dist_norm, ripple_radius, aa_ripple);
            let bounded_mask = select(1.0, shape_mask, ripple_bounded);
            let overlay_a = clamp(ripple_mask * ripple_alpha * bounded_mask, 0.0, 1.0);

            // Composite overlay on top of base in unpremultiplied space.
            let out_a = overlay_a + base_a * (1.0 - overlay_a);
            if out_a <= 0.001 {
                discard;
            }

            let out_pm_rgb = ripple_color_rgb * overlay_a + base_rgb * base_a * (1.0 - overlay_a);
            final_color = vec4f(out_pm_rgb / out_a, out_a);
        } else {
            discard;
        }
    }

    final_color = vec4f(final_color.rgb * final_color.a, final_color.a);
    return final_color;
}
