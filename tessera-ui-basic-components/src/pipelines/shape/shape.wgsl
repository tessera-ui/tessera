struct ShapeUniforms {
    corner_radii: vec4f,       // x:tl, y:tr, z:br, w:bl
    corner_g2: vec4f,          // x:tl, y:tr, z:br, w:bl
    primary_color: vec4f,
    border_color: vec4f,
    shadow_ambient_color: vec4f,
    shadow_ambient_params: vec3f,
    shadow_spot_color: vec4f,
    shadow_spot_params: vec3f,
    render_mode: f32,
    ripple_params: vec4f,
    ripple_color: vec4f,
    border_width: f32,
    position: vec4f,           // x, y, width, height
    screen_size: vec2f,
};

const MODE_FILL: f32 = 0.0;
const MODE_OUTLINE: f32 = 1.0;
const MODE_SHADOW: f32 = 2.0;
const MODE_RIPPLE_FILL: f32 = 3.0;
const MODE_RIPPLE_OUTLINE: f32 = 4.0;
const MODE_RIPPLE_FILLED_OUTLINE: f32 = 5.0;

const EPS_DISCARD: f32 = 0.001;
const SHADOW_AA_MARGIN_PX: f32 = 1.0;

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
    @location(0) local_pos: vec2f, // Normalized object space (scaled in fragment)
    @location(1) @interpolate(flat) instance_index: u32,
};

fn shadow_layer_pad(color: vec4f, params: vec3f) -> vec2f {
    if color.a <= 0.0 {
        return vec2f(0.0, 0.0);
    }

    let offset = params.xy;
    let smoothness = params.z;
    return vec2f(abs(offset.x), abs(offset.y)) + vec2f(smoothness, smoothness);
}

fn shadow_pad(instance: ShapeUniforms) -> vec2f {
    var pad = vec2f(0.0, 0.0);
    pad = max(pad, shadow_layer_pad(instance.shadow_ambient_color, instance.shadow_ambient_params));
    pad = max(pad, shadow_layer_pad(instance.shadow_spot_color, instance.shadow_spot_params));
    return pad + vec2f(SHADOW_AA_MARGIN_PX, SHADOW_AA_MARGIN_PX);
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let instance = uniforms.instances[in.instance_index];
    let screen_dimensions = instance.screen_size;

    // `in.position` is the unit quad vertex (0,0 to 1,1).
    // `instance.position` is the rect (x, y, width, height) in pixels.
    let instance_pos = instance.position.xy;
    let instance_size = instance.position.zw;

    var pad = vec2f(0.0, 0.0);
    if instance.render_mode == MODE_SHADOW {
        pad = shadow_pad(instance);
    }

    let expanded_size = instance_size + pad * 2.0;

    // Calculate the vertex's final pixel position.
    let pixel_pos = instance_pos - pad + in.position * expanded_size;

    // Convert final pixel position to NDC for the rasterizer.
    let clip_pos = vec2<f32>(
        (pixel_pos.x / screen_dimensions.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_dimensions.y) * -2.0 + 1.0
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);

    // local_pos is normalized object space (so `local_pos * size` yields pixels).
    // For shadows, expand the normalized range so the SDF can cover pixels
    // outside the layout box.
    if instance.render_mode == MODE_SHADOW {
        let safe_size = max(instance_size, vec2f(1.0, 1.0));
        out.local_pos = (in.position * expanded_size - pad) / safe_size - 0.5;
    } else {
        out.local_pos = in.position - 0.5; // Center around 0 for SDF
    }
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

fn is_ellipse(corner_radii: vec4f) -> bool {
    return corner_radii.x < 0.0;
}

fn sdf_shape(p: vec2f, half_size: vec2f, corner_radii: vec4f, corner_g2: vec4f) -> f32 {
    if is_ellipse(corner_radii) {
        return sdf_ellipse(p, half_size);
    }
    return sdf_g2_rounded_box(p, half_size, corner_radii, corner_g2);
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

fn aa_mask(dist: f32) -> f32 {
    let aa = fwidth(dist);
    return 1.0 - smoothstep(-aa, aa, dist);
}

fn outline_mask(dist: f32, border_width: f32) -> f32 {
    let aa = fwidth(dist);
    let outer = 1.0 - smoothstep(-aa, aa, dist);
    let inner = 1.0 - smoothstep(-aa, aa, dist + border_width);
    return max(0.0, outer - inner);
}

fn shadow_layer_alpha(
    p_object: vec2f,
    half_size: vec2f,
    corner_radii: vec4f,
    corner_g2: vec4f,
    offset: vec2f,
    smoothness: f32,
) -> f32 {
    let dist = sdf_shape(p_object - offset, half_size, corner_radii, corner_g2);
    let mask = aa_mask(dist);
    let soft = smoothstep(smoothness, 0.0, dist);
    return mask * soft;
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
    let mode = instance.render_mode;

    let size = instance.position.zw;
    let half_size = size * 0.5;

    let p_normalized = in.local_pos;
    let p_object = p_normalized * size;

    var final_color: vec4f;

    if mode == MODE_SHADOW {
        var ambient_alpha: f32 = 0.0;
        if instance.shadow_ambient_color.a > 0.0 {
            ambient_alpha = shadow_layer_alpha(
                p_object,
                half_size,
                instance.corner_radii,
                instance.corner_g2,
                instance.shadow_ambient_params.xy,
                instance.shadow_ambient_params.z,
            );
        }

        var spot_alpha: f32 = 0.0;
        if instance.shadow_spot_color.a > 0.0 {
            spot_alpha = shadow_layer_alpha(
                p_object,
                half_size,
                instance.corner_radii,
                instance.corner_g2,
                instance.shadow_spot_params.xy,
                instance.shadow_spot_params.z,
            );
        }

        let ambient_a = ambient_alpha * instance.shadow_ambient_color.a;
        let spot_a = spot_alpha * instance.shadow_spot_color.a;
        let out_a = ambient_a + spot_a * (1.0 - ambient_a);
        if out_a <= EPS_DISCARD {
            discard;
        }

        let out_pm_rgb = instance.shadow_ambient_color.rgb * ambient_a
            + instance.shadow_spot_color.rgb * spot_a * (1.0 - ambient_a);
        final_color = vec4f(out_pm_rgb / out_a, out_a);
    } else {
        let dist = sdf_shape(p_object, half_size, instance.corner_radii, instance.corner_g2);
        let shape_mask = aa_mask(dist);

        if mode == MODE_FILL {
            if shape_mask <= EPS_DISCARD {
                discard;
            }
            final_color = vec4f(instance.primary_color.rgb, instance.primary_color.a * shape_mask);
        } else if mode == MODE_OUTLINE {
            if instance.border_width <= 0.0 {
                discard;
            }
            let mask = outline_mask(dist, instance.border_width);
            if mask <= EPS_DISCARD {
                discard;
            }
            final_color = vec4f(instance.primary_color.rgb, instance.primary_color.a * mask);
        } else if mode == MODE_RIPPLE_FILL || mode == MODE_RIPPLE_OUTLINE || mode == MODE_RIPPLE_FILLED_OUTLINE {
            var base_rgb: vec3f;
            var base_a: f32;

            if mode == MODE_RIPPLE_FILL {
                base_rgb = instance.primary_color.rgb;
                base_a = instance.primary_color.a * shape_mask;
            } else if mode == MODE_RIPPLE_OUTLINE {
                if instance.border_width <= 0.0 {
                    discard;
                }
                let mask = outline_mask(dist, instance.border_width);
                base_rgb = instance.primary_color.rgb;
                base_a = instance.primary_color.a * mask;
            } else {
                if instance.border_width <= 0.0 {
                    if shape_mask <= EPS_DISCARD {
                        discard;
                    }
                    base_rgb = instance.primary_color.rgb;
                    base_a = instance.primary_color.a * shape_mask;
                } else {
                    let dist_inner_edge = dist + instance.border_width;
                    let aa = fwidth(dist);
                    let t = smoothstep(-aa, aa, dist_inner_edge);
                    base_rgb = mix(instance.primary_color.rgb, instance.border_color.rgb, t);
                    base_a = mix(instance.primary_color.a, instance.border_color.a, t) * shape_mask;
                }
            }

            let ripple_center = instance.ripple_params.xy;
            let ripple_radius = instance.ripple_params.z;
            let ripple_alpha = instance.ripple_params.w;
            let ripple_bounded = instance.ripple_color.a > 0.5;

            let p_pixel = p_normalized * size;
            let center_pixel = ripple_center * size;
            let dist_to_center_pixel = distance(p_pixel, center_pixel);
            let min_dimension = min(size.x, size.y);
            let dist_norm = dist_to_center_pixel / max(min_dimension, 1.0);
            let aa_ripple = max(fwidth(dist_norm), EPS_DISCARD);
            let ripple_mask = calculate_ripple_mask(dist_norm, ripple_radius, aa_ripple);
            let bounded_mask = select(1.0, shape_mask, ripple_bounded);
            let overlay_a = clamp(ripple_mask * ripple_alpha * bounded_mask, 0.0, 1.0);

            let out_a = overlay_a + base_a * (1.0 - overlay_a);
            if out_a <= EPS_DISCARD {
                discard;
            }

            let out_pm_rgb = instance.ripple_color.rgb * overlay_a + base_rgb * base_a * (1.0 - overlay_a);
            final_color = vec4f(out_pm_rgb / out_a, out_a);
        } else {
            discard;
        }
    }

    return vec4f(final_color.rgb * final_color.a, final_color.a);
}
