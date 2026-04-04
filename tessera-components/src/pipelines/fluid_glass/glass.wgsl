struct GlassUniforms {
    tint_color: vec4<f32>,
    rect_uv_bounds: vec4<f32>,
    corner_radii: vec4<f32>,
    corner_g2: vec4<f32>,
    clip_rect_uv: vec4<f32>,
    rect_size_px: vec2<f32>,
    ripple_center: vec2<f32>,
    shape_type: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
    ripple_radius: f32,
    ripple_alpha: f32,
    ripple_strength: f32,
    border_width: f32,
    sdf_cache_enabled: f32,
};

const LENS_SOURCE_OVERSCAN_SCALE: f32 = 1.06;
const LENS_CENTER_SHRINK_SCALE: f32 = 1.10;
const LENS_EDGE_WIDTH_FACTOR: f32 = 0.24;
const LENS_EDGE_WIDTH_MIN_PX: f32 = 14.0;
const LENS_EDGE_WIDTH_MAX_PX: f32 = 28.0;
const LENS_EDGE_ENLARGE_STRENGTH_FACTOR: f32 = 0.52;
const LENS_EDGE_ENLARGE_STRENGTH_MIN_PX: f32 = 12.0;
const LENS_EDGE_ENLARGE_STRENGTH_MAX_PX: f32 = 28.0;
const LENS_EDGE_BLEND_EXPONENT: f32 = 1.9;

struct GlassInstances {
    instances: array<GlassUniforms>,
}

@group(0) @binding(0) var<storage, read> uniforms: GlassInstances;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;
@group(0) @binding(3) var sdf_texture: texture_2d<f32>;
@group(0) @binding(4) var sdf_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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

    let local_uvs = array<vec2<f32>, 4>(
        vec2(0.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
        vec2(1.0, 0.0)
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
    out.uv = local_uv;
    out.instance_index = instance_index;
    return out;
}

fn sdf_g2_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>, k: vec4<f32>) -> f32 {
    let top_radii = select(r.x, r.y, p.x > 0.0);
    let bottom_radii = select(r.w, r.z, p.x > 0.0);
    let r_for_quadrant = select(top_radii, bottom_radii, p.y > 0.0);
    let top_k = select(k.x, k.y, p.x > 0.0);
    let bottom_k = select(k.w, k.z, p.x > 0.0);
    let k_for_quadrant = select(top_k, bottom_k, p.y > 0.0);

    let q = abs(p) - b + r_for_quadrant;
    let v = max(q, vec2<f32>(0.0));

    if abs(k_for_quadrant - 2.0) < 0.001 {
        return length(v) + min(max(q.x, q.y), 0.0) - r_for_quadrant;
    }

    let dist_corner_shape = pow(pow(v.x, k_for_quadrant) + pow(v.y, k_for_quadrant), 1.0 / k_for_quadrant);

    return dist_corner_shape + min(max(q.x, q.y), 0.0) - r_for_quadrant;
}

fn sdf_ellipse(p: vec2<f32>, r: vec2<f32>) -> f32 {
    if r.x <= 0.0 || r.y <= 0.0 {
        return 1.0e6;
    }
    return (length(p / r) - 1.0) * min(r.x, r.y);
}

fn sdf_axis_aligned_box(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let dist = abs(p) - half_size;
    let outside = max(dist, vec2<f32>(0.0, 0.0));
    return length(outside) + min(max(dist.x, dist.y), 0.0);
}

fn grad_sd_g2_rounded_box(coord: vec2<f32>, half_size: vec2<f32>, r: vec4<f32>, k: vec4<f32>) -> vec2<f32> {
    let top_radii = select(r.x, r.y, coord.x > 0.0);
    let bottom_radii = select(r.w, r.z, coord.x > 0.0);
    let r_for_quadrant = select(top_radii, bottom_radii, coord.y > 0.0);
    let top_k = select(k.x, k.y, coord.x > 0.0);
    let bottom_k = select(k.w, k.z, coord.x > 0.0);
    let k_for_quadrant = select(top_k, bottom_k, coord.y > 0.0);
    let inner_half_size = half_size - r_for_quadrant;
    let corner_coord = abs(coord) - inner_half_size;

    if corner_coord.x >= 0.0 && corner_coord.y >= 0.0 {
        let grad_dir = vec2<f32>(
            pow(corner_coord.x + 0.0001, k_for_quadrant - 1.0),
            pow(corner_coord.y + 0.0001, k_for_quadrant - 1.0)
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

fn signed_one(value: f32) -> f32 {
    return select(-1.0, 1.0, value >= 0.0);
}

fn grad_sd_axis_aligned_box(coord: vec2<f32>, half_size: vec2<f32>) -> vec2<f32> {
    let dist = abs(coord) - half_size;
    if dist.x > 0.0 || dist.y > 0.0 {
        let outside = max(dist, vec2<f32>(0.0, 0.0));
        return sign(coord) * normalize(outside);
    }

    let dist_to_edge = half_size - abs(coord);
    if dist_to_edge.x < dist_to_edge.y {
        return vec2<f32>(signed_one(coord.x), 0.0);
    } else {
        return vec2<f32>(0.0, signed_one(coord.y));
    }
}

fn shape_sd(instance: GlassUniforms, coord: vec2<f32>, half_size: vec2<f32>, k: vec4<f32>) -> f32 {
    if instance.shape_type == 1.0 {
        return sdf_ellipse(coord, half_size);
    }
    if instance.shape_type == 2.0 {
        return sdf_axis_aligned_box(coord, half_size);
    }
    return sdf_g2_rounded_box(coord, half_size, instance.corner_radii, k);
}

fn shape_normal(instance: GlassUniforms, coord: vec2<f32>, half_size: vec2<f32>, k: vec4<f32>) -> vec2<f32> {
    if instance.shape_type == 1.0 {
        return grad_sd_ellipse(coord, half_size);
    }
    if instance.shape_type == 2.0 {
        return grad_sd_axis_aligned_box(coord, half_size);
    }
    return grad_sd_g2_rounded_box(coord, half_size, instance.corner_radii, k);
}

fn evaluate_shape(
    instance: GlassUniforms,
    coord: vec2<f32>,
    half_size: vec2<f32>,
    k: vec4<f32>
) -> vec3<f32> {
    if instance.sdf_cache_enabled > 0.5 {
        let uv = clamp((coord + half_size) / (half_size * 2.0), vec2<f32>(0.0), vec2<f32>(1.0));
        let sample = textureSampleLevel(sdf_texture, sdf_sampler, uv, 0.0);
        let normal = normalize(sample.yz);
        return vec3<f32>(sample.x, normal);
    }

    let sd = shape_sd(instance, coord, half_size, k);
    let normal = shape_normal(instance, coord, half_size, k);
    return vec3<f32>(sd, normal);
}

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn blend_overlay(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let multiply = 2.0 * base * blend;
    let screen = 1.0 - 2.0 * (1.0 - base) * (1.0 - blend);
    return select(screen, multiply, base < vec3<f32>(0.5));
}

fn lens_sample_uv(
    instance: GlassUniforms,
    centered_coord: vec2<f32>,
    half_size: vec2<f32>,
    shape_eval: vec3<f32>
) -> vec2<f32> {
    let rect_uv_start = instance.rect_uv_bounds.xy;
    let px_to_uv_ratio = (instance.rect_uv_bounds.zw - rect_uv_start) / instance.rect_size_px;
    let min_uv = instance.clip_rect_uv.xy;
    let max_uv = instance.clip_rect_uv.zw;

    let sd = shape_eval.x;
    let normal = shape_eval.yz;

    let min_dimension = min(instance.rect_size_px.x, instance.rect_size_px.y);
    let edge_width_px = clamp(
        min_dimension * LENS_EDGE_WIDTH_FACTOR,
        LENS_EDGE_WIDTH_MIN_PX,
        LENS_EDGE_WIDTH_MAX_PX
    );
    let edge_enlarge_strength_px = clamp(
        min_dimension * LENS_EDGE_ENLARGE_STRENGTH_FACTOR,
        LENS_EDGE_ENLARGE_STRENGTH_MIN_PX,
        LENS_EDGE_ENLARGE_STRENGTH_MAX_PX
    );
    let edge_exponent = LENS_EDGE_BLEND_EXPONENT;

    let overscanned_coord = centered_coord / LENS_SOURCE_OVERSCAN_SCALE;
    let center_shrunk_coord = overscanned_coord * LENS_CENTER_SHRINK_SCALE;
    let interior_distance = max(-sd, 0.0);
    let edge_t = clamp(1.0 - interior_distance / edge_width_px, 0.0, 1.0);
    let edge_factor = pow(edge_t, edge_exponent);
    let edge_enlarged_coord =
        overscanned_coord - normal * edge_enlarge_strength_px * edge_factor;
    let distorted_coord = mix(center_shrunk_coord, edge_enlarged_coord, edge_factor);

    let sample_coord = distorted_coord + half_size;
    let sample_uv = rect_uv_start + sample_coord * px_to_uv_ratio;
    return clamp(sample_uv, min_uv, max_uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let instance = uniforms.instances[in.instance_index];

    let local_coord = in.uv * instance.rect_size_px;
    let half_size = instance.rect_size_px * 0.5;
    let centered_coord = local_coord - half_size;

    let shape_eval = evaluate_shape(instance, centered_coord, half_size, instance.corner_g2);
    let sd = shape_eval.x;
    let sample_uv = lens_sample_uv(instance, centered_coord, half_size, shape_eval);

    let base_color = textureSampleLevel(t_diffuse, s_diffuse, sample_uv, 0.0);
    var color = base_color.rgb;

    let center_pixel = instance.ripple_center * instance.rect_size_px;
    let dist_pixels = distance(local_coord, center_pixel);
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

    if instance.noise_amount > 0.0 {
        let grain = (rand(local_coord * instance.noise_scale + instance.time) - 0.5) * instance.noise_amount;
        color += grain;
    }

    var final_color = vec4(color, base_color.a);
    let width = fwidth(sd);
    let shape_alpha = smoothstep(width, -width, sd);

    if instance.border_width > 0.0 {
        let border_width_aa = width;
        if instance.border_width <= border_width_aa {
            final_color.a = shape_alpha;
            return final_color;
        }

        let outer = 1.0 - smoothstep(-border_width_aa, border_width_aa, sd);
        let inner = 1.0 - smoothstep(
            -instance.border_width - border_width_aa,
            -instance.border_width + border_width_aa,
            sd
        );
        let border_mask = clamp(outer - inner, 0.0, 1.0);
        if border_mask > 0.0 {
            let normal = shape_eval.yz;
            let highlight_dir = normalize(vec2<f32>(cos(radians(136.0)), sin(radians(136.0))));
            let top_light_fraction = dot(highlight_dir, normal);
            let bottom_light_fraction = -top_light_fraction;
            let highlight_decay = 1.5;
            let highlight_fraction = pow(max(top_light_fraction, bottom_light_fraction), highlight_decay);

            let border_color = vec3<f32>(1.0);
            let highlight_intensity = highlight_fraction * border_mask;
            let highlight_layer_color = border_color * highlight_intensity;
            let final_rgb_with_highlight = blend_overlay(final_color.rgb, highlight_layer_color);
            let highlight_rgb = mix(final_color.rgb, final_rgb_with_highlight, border_mask);

            final_color.r = highlight_rgb.r;
            final_color.g = highlight_rgb.g;
            final_color.b = highlight_rgb.b;
        }
    }

    final_color.a = shape_alpha;
    return final_color;
}
