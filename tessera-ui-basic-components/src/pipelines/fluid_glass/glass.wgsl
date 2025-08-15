struct GlassUniforms {
    // Grouped by alignment to match Rust struct and std140 layout.
    // vec4s
    tint_color: vec4<f32>,
    rect_uv_bounds: vec4<f32>,

    // vec2s
    rect_size_px: vec2<f32>,
    ripple_center: vec2<f32>,

    // f32s
    corner_radius: f32,
    shape_type: f32,
    g2_k_value: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
    ripple_radius: f32,
    ripple_alpha: f32,
    ripple_strength: f32,
    border_width: f32,
    screen_size: vec2<f32>, // Screen dimensions
    light_source: vec2<f32>, // Light source position in world coordinates
    light_scale: f32, // Light intensity scale factor
};

struct GlassInstances {
    length: u32,
    instances: array<GlassUniforms>,
};

@group(0) @binding(0) var<storage, read> uniforms: GlassInstances;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // Local UV [0, 1]
    @location(1) @interpolate(flat) instance_index: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let instance = uniforms.instances[instance_index];
    let rect_uv_min = instance.rect_uv_bounds.xy;
    let rect_uv_max = instance.rect_uv_bounds.zw;

    // Define a unit quad (from 0,0 to 1,1). These are the local UVs.
    let local_uvs = array<vec2<f32>, 4>(
        vec2(0.0, 0.0), // Top-left
        vec2(0.0, 1.0), // Bottom-left
        vec2(1.0, 1.0), // Bottom-right
        vec2(1.0, 0.0)  // Top-right
    );

    let indices = array<u32, 6>(0, 1, 2, 0, 2, 3);
    let local_uv = local_uvs[indices[vertex_index]];

    // Map local UV to the instance's global UV space
    let global_uv = rect_uv_min + local_uv * (rect_uv_max - rect_uv_min);

    // Convert global UV coordinates [0, 1] to clip space coordinates [-1, 1].
    // Y is flipped in clip space (positive is up).
    let clip_pos = vec2<f32>(
        global_uv.x * 2.0 - 1.0,
        -(global_uv.y * 2.0 - 1.0)
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);
    out.uv = local_uv; // Pass the LOCAL UV to the fragment shader
    out.instance_index = instance_index;
    return out;
}

fn circle_map(x: f32) -> f32 {
    return 1.0 - sqrt(1.0 - x * x);
}

fn normal_to_tangent(normal: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(normal.y, -normal.x);
}

fn sdf_g2_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32, k: f32) -> f32 {
    let q = abs(p) - b + r;
    let v = max(q, vec2<f32>(0.0));

    if abs(k - 2.0) < 0.001 {
        return length(v) + min(max(q.x, q.y), 0.0) - r;
    }

    let dist_corner_shape = pow(pow(v.x, k) + pow(v.y, k), 1.0 / k);

    return dist_corner_shape + min(max(q.x, q.y), 0.0) - r;
}

fn sdf_ellipse(p: vec2<f32>, r: vec2<f32>) -> f32 {
    if r.x <= 0.0 || r.y <= 0.0 {
        return 1.0e6;
    }
    return (length(p / r) - 1.0) * min(r.x, r.y);
}

fn grad_sd_g2_rounded_box(coord: vec2<f32>, half_size: vec2<f32>, r: f32, k: f32) -> vec2<f32> {
    let inner_half_size = half_size - r;
    let corner_coord = abs(coord) - inner_half_size;

    if corner_coord.x >= 0.0 && corner_coord.y >= 0.0 {
        let grad_dir = vec2<f32>(
            pow(corner_coord.x + 0.0001, k - 1.0),
            pow(corner_coord.y + 0.0001, k - 1.0)
        );
        return sign(coord) * normalize(grad_dir);
    } else {
        if corner_coord.x > corner_coord.y {
            return sign(coord) * vec2<f32>(1.0, 0.0);
        } else {
            return sign(coord) * vec2<f32>(0.0, 1.0);
        }
    }
}

fn grad_sd_ellipse(coord: vec2<f32>, r: vec2<f32>) -> vec2<f32> {
    return normalize(coord / (r * r));
}

fn to_linear_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(0.04045);
    let lower = srgb / vec3<f32>(12.92);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return select(higher, lower, srgb <= cutoff);
}

fn from_linear_srgb(linear: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(0.0031308);
    let lower = linear * vec3<f32>(12.92);
    let higher = vec3<f32>(1.055) * pow(linear, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(higher, lower, linear <= cutoff);
}

fn saturate_color(color: vec4<f32>, amount: f32) -> vec4<f32> {
    let linear_srgb = to_linear_srgb(color.rgb);
    let rgb_to_y = vec3<f32>(0.2126, 0.7152, 0.0722);
    let y = dot(linear_srgb, rgb_to_y);
    let gray = vec3<f32>(y);
    let adjusted_linear_srgb = mix(gray, linear_srgb, amount);
    let adjusted_srgb = from_linear_srgb(adjusted_linear_srgb);
    return vec4<f32>(adjusted_srgb, color.a);
}

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn refraction_color(instance: GlassUniforms, local_coord: vec2<f32>, size: vec2<f32>, k: f32) -> vec4<f32> {
    let half_size = size * 0.5;
    let centered_coord = local_coord - half_size;

    var sd: f32;
    if instance.shape_type == 1.0 {
        sd = sdf_ellipse(centered_coord, half_size);
    } else {
        sd = sdf_g2_rounded_box(centered_coord, half_size, instance.corner_radius, k);
    }

    var refracted_coord = local_coord;
    if sd < 0.0 && -sd < instance.refraction_height {
        var normal: vec2<f32>;
        if instance.shape_type == 1.0 {
            normal = grad_sd_ellipse(centered_coord, half_size);
        } else {
            let max_grad_radius = max(min(half_size.x, half_size.y), instance.corner_radius);
            let grad_radius = min(instance.corner_radius * 1.5, max_grad_radius);
            normal = grad_sd_g2_rounded_box(centered_coord, half_size, grad_radius, k);
        }

        let refracted_distance = circle_map(1.0 - (-sd / instance.refraction_height)) * instance.refraction_amount;
        let refracted_direction = normalize(normal + instance.eccentric_factor * normalize(centered_coord));
        refracted_coord = local_coord + refracted_distance * refracted_direction;
    }

    let rect_uv_start = instance.rect_uv_bounds.xy;
    let px_to_uv_ratio = (instance.rect_uv_bounds.zw - rect_uv_start) / instance.rect_size_px;
    let sample_uv = rect_uv_start + refracted_coord * px_to_uv_ratio;
    return textureSample(t_diffuse, s_diffuse, sample_uv);
}

fn dispersion_color_on_refracted(instance: GlassUniforms, local_coord: vec2<f32>, size: vec2<f32>, k: f32) -> vec4<f32> {
    let half_size = size * 0.5;
    let centered_coord = local_coord - half_size;

    var sd: f32;
    if instance.shape_type == 1.0 {
        sd = sdf_ellipse(centered_coord, half_size);
    } else {
        sd = sdf_g2_rounded_box(centered_coord, half_size, instance.corner_radius, k);
    }

    let base_refracted = refraction_color(instance, local_coord, size, k);

    if sd < 0.0 && -sd < instance.dispersion_height && instance.dispersion_height > 0.0 {
        var normal: vec2<f32>;
        if instance.shape_type == 1.0 {
            normal = grad_sd_ellipse(centered_coord, half_size);
        } else {
            normal = grad_sd_g2_rounded_box(centered_coord, half_size, instance.corner_radius, k);
        }
        let tangent = normal_to_tangent(normal);

        let dispersion_fraction = 1.0 - (-sd / instance.dispersion_height);
        let dispersion_width = instance.dispersion_height * 2.0 * pow(circle_map(dispersion_fraction), 2.0);

        if dispersion_width < 2.0 {
            return base_refracted;
        }

        let sample_count = 12;
        var red_color = 0.0;
        var green_color = 0.0;
        var blue_color = 0.0;
        var red_weight = 0.0;
        var green_weight = 0.0;
        var blue_weight = 0.0;

        for (var i = 0; i < sample_count; i = i + 1) {
            let t = f32(i) / f32(sample_count - 1);
            let sample_coord = local_coord + tangent * (t - 0.5) * dispersion_width;
            let refracted_c = refraction_color(instance, sample_coord, size, k);

            if t >= 0.0 && t <= 0.5 {
                blue_color += refracted_c.b;
                blue_weight += 1.0;
            }
            if t >= 0.25 && t <= 0.75 {
                green_color += refracted_c.g;
                green_weight += 1.0;
            }
            if t >= 0.5 && t <= 1.0 {
                red_color += refracted_c.r;
                red_weight += 1.0;
            }
        }

        red_color = red_color / max(red_weight, 1.0);
        green_color = green_color / max(green_weight, 1.0);
        blue_color = blue_color / max(blue_weight, 1.0);

        return vec4<f32>(red_color, green_color, blue_color, base_refracted.a);
    } else {
        return base_refracted;
    }
}

fn blend_overlay(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let a = base * 2.0;
    let b = (vec3<f32>(1.0) - (vec3<f32>(1.0) - base) * 2.0) * (blend - 0.5) + base;
    return select(b, a, blend < vec3<f32>(0.5));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let instance = uniforms.instances[in.instance_index];

    let local_coord = in.uv * instance.rect_size_px;
    let half_size = instance.rect_size_px * 0.5;
    let centered_coord = local_coord - half_size;
    let k = instance.g2_k_value;

    var sd: f32;
    if instance.shape_type == 1.0 {
        sd = sdf_ellipse(centered_coord, half_size);
    } else {
        sd = sdf_g2_rounded_box(centered_coord, half_size, instance.corner_radius, k);
    }

    var base_color: vec4<f32>;
    if instance.dispersion_height > 0.0 {
        base_color = dispersion_color_on_refracted(instance, local_coord, instance.rect_size_px, k);
    } else {
        base_color = refraction_color(
            instance,
            local_coord,
            instance.rect_size_px,
            k
        );
    }

    var color = base_color.rgb;

    let p_pixel = in.uv * instance.rect_size_px;
    let center_pixel = instance.ripple_center * instance.rect_size_px;
    let dist_pixels = distance(p_pixel, center_pixel);

    let min_dimension = min(instance.rect_size_px.x, instance.rect_size_px.y);
    let radius_pixels = instance.ripple_radius * min_dimension;

    if dist_pixels < radius_pixels {
        let ripple_factor = 1.0 - dist_pixels / radius_pixels;
        color += vec3<f32>(1.0, 1.0, 1.0) * ripple_factor * instance.ripple_alpha;
    }

    let tint_weight = instance.tint_color.a;
    if tint_weight > 0.0 {
        color = mix(color, instance.tint_color.rgb, tint_weight);
    }

    color = saturate_color(vec4(color, base_color.a), instance.chroma_multiplier).rgb;

    if instance.noise_amount > 0.0 {
        let grain = (rand(local_coord * instance.noise_scale + instance.time) - 0.5) * instance.noise_amount;
        color += grain;
    }

    var final_color = vec4(color, base_color.a);
    let width = fwidth(sd);
    let shape_alpha = smoothstep(width, -width, sd);

    if instance.border_width > 0.0 {
        // 1. Define the border region.
        // sd is the distance to the shape edge, negative inside, positive outside.
        // This expression creates a "band" region where sd is between 0 and -border_width.
        let border_width_aa = fwidth(sd); // Add anti-aliasing for the border
        let outer = smoothstep(0.0 + border_width_aa, -border_width_aa, sd);
        let inner = smoothstep(-instance.border_width + border_width_aa, -instance.border_width - border_width_aa, sd);
        let border_mask = outer - inner;
        // Only compute highlight within the border region.
        if border_mask > 0.0 {
            // 2. Compute highlight normal (same logic as AGSL, using new function).
            var normal: vec2<f32>;
            if instance.shape_type == 1.0 {
                normal = grad_sd_ellipse(centered_coord, half_size);
            } else {
                normal = grad_sd_g2_rounded_box(centered_coord, half_size, instance.corner_radius, k);
            }

            // 3. Compute highlight distribution.
            let highlight_dir = normalize(vec2<f32>(cos(radians(136.0)), sin(radians(136.0)))); // Light direction at 136 degrees.
            let top_light_fraction = dot(highlight_dir, normal);
            let bottom_light_fraction = -top_light_fraction;
            let highlight_decay = 1.5;
            let highlight_fraction = pow(max(top_light_fraction, bottom_light_fraction), highlight_decay);
        
            // 4. Blend highlight color with border mask and add to final color.
            let border_color = vec3<f32>(1.0); // Base border color (white).
            let highlight_intensity = highlight_fraction * border_mask;

            // Create highlight layer color (white highlight times its intensity).
            let highlight_layer_color = border_color * highlight_intensity;

            // Use overlay blend mode to mix highlight layer into final color.
            let final_rgb_with_highlight = blend_overlay(final_color.rgb, highlight_layer_color);

            // Only apply blended result in border region.
            let highlight_rgb = mix(final_color.rgb, final_rgb_with_highlight, border_mask);

            final_color.r = highlight_rgb.r;
            final_color.g = highlight_rgb.g;
            final_color.b = highlight_rgb.b;
        }
    }

    final_color.a = shape_alpha;

    return final_color;
}
