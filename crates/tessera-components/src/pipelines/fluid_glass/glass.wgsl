struct GlassUniforms {
    tint_color: vec4<f32>,
    rect_uv_bounds: vec4<f32>,
    corner_radii: vec4<f32>,
    corner_g2: vec4<f32>,
    material_tone: vec4<f32>,
    lens: vec4<f32>,
    lens_shadow: vec4<f32>,
    lens_contour_a: vec4<f32>,
    lens_contour_b: vec4<f32>,
    lens_contour_c: vec4<f32>,
    raster_origin: vec2<f32>,
    raster_size: vec2<f32>,
    scene_origin_px: vec2<f32>,
    raster_step: vec2<f32>,
    contour_iterations: u32,
    refraction_width: f32,
    refraction_corner_g2: f32,
    inner_refraction_width: f32,
    inner_refraction: f32,
    dispersion_angle: f32,
    spectrum_radius: u32,
    blur_sigma_px: f32,
    pixels_per_dp: f32,
    opacity: f32,
    material_mode: u32,
    // Maps window UVs into the scene texture: scale in xy, offset in zw.
    scene_uv: vec4<f32>,
    scene_size_px: vec2<f32>,
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
    contrast: f32,
    feedback: f32,
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
const LUMA: vec3<f32> = vec3(0.2126, 0.7152, 0.0722);
const GAUSSIAN_QUANTILES: array<f32, 49> = array(
    -2.31876, -1.87187, -1.63504, -1.46523, -1.32953, -1.21468, -1.11394,
    -1.02339, -0.94054, -0.86370, -0.79164, -0.72348, -0.65852, -0.59623,
    -0.53618, -0.47800, -0.42139, -0.36611, -0.31192, -0.25863, -0.20607,
    -0.15407, -0.10249, -0.05118,  0.00000,  0.05118,  0.10249,  0.15407,
     0.20607,  0.25863,  0.31192,  0.36611,  0.42139,  0.47800,  0.53618,
     0.59623,  0.65852,  0.72348,  0.79164,  0.86370,  0.94054,  1.02339,
     1.11394,  1.21468,  1.32953,  1.46523,  1.63504,  1.87187,  2.31876,
);
const STRATA_ORDER: array<u32, 7> = array(5u, 0u, 2u, 3u, 4u, 6u, 1u);

struct GlassInstances {
    instances: array<GlassUniforms>,
}

@group(0) @binding(0) var<storage, read> uniforms: GlassInstances;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;
@group(0) @binding(3) var t_lens_primary: texture_2d<f32>;
@group(0) @binding(4) var t_lens_background: texture_2d<f32>;

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
    let lens_padding = max(1.5, abs(instance.lens_shadow.x) + 3.0 * instance.lens_shadow.z);
    let padding = select(vec2(0.0), vec2(lens_padding * instance.pixels_per_dp) / instance.rect_size_px, instance.lens.x > 0.0);
    let local_uv = local_uvs[indices[vertex_index]] * (1.0 + 2.0 * padding) - padding;
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

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn blend_overlay(base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let multiply = 2.0 * base * blend;
    let screen = 1.0 - 2.0 * (1.0 - base) * (1.0 - blend);
    return select(screen, multiply, base < vec3<f32>(0.5));
}

fn custom_lens_coord(
    instance: GlassUniforms,
    centered_coord: vec2<f32>,
    half_size: vec2<f32>,
    shape_eval: vec3<f32>
) -> vec2<f32> {
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

    return distorted_coord + half_size;
}

fn scene_sample_uv(instance: GlassUniforms, window_uv: vec2<f32>) -> vec2<f32> {
    let size = vec2<f32>(textureDimensions(t_diffuse));
    return clamp(
        window_uv * instance.scene_uv.xy + instance.scene_uv.zw,
        vec2(0.5) / size,
        (instance.scene_size_px - vec2(0.5)) / size,
    );
}

fn to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(linear, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, linear * 12.92, linear <= vec3(0.0031308));
}

fn from_srgb(encoded: vec3<f32>) -> vec3<f32> {
    return select(pow(max((encoded + 0.055) / 1.055, vec3(0.0)), vec3(2.4)), encoded / 12.92, encoded <= vec3(0.04045));
}

fn encoded_texels(instance: GlassUniforms, position: vec2<i32>, fraction: vec2<f32>) -> vec3<f32> {
    let upper = vec2<i32>(instance.scene_size_px) - 1;
    let low = clamp(position, vec2(0), upper);
    let high = clamp(position + 1, vec2(0), upper);
    let top = mix(to_srgb(textureLoad(t_diffuse, low, 0).rgb), to_srgb(textureLoad(t_diffuse, vec2(high.x, low.y), 0).rgb), fraction.x);
    let bottom = mix(to_srgb(textureLoad(t_diffuse, vec2(low.x, high.y), 0).rgb), to_srgb(textureLoad(t_diffuse, high, 0).rgb), fraction.x);
    return mix(top, bottom, fraction.y);
}

fn lens_scene_sample(instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    // Keep the interpolation phase independent of the scene snapshot's integer origin.
    let position = fract(instance.scene_origin_px) + coord / instance.raster_step - 0.5;
    let low = vec2<i32>(floor(instance.scene_origin_px)) + vec2<i32>(floor(position));
    return encoded_texels(instance, low, fract(position));
}

fn lens_profile(distance: f32, height: f32) -> f32 {
    let t = clamp(1.0 + distance / height, 0.0, 1.0);
    return 1.0 - sqrt(max(1.0 - t * t, 0.0));
}

fn gaussian_cdf(x: f32) -> f32 {
    let t = 1.0 / (1.0 + 0.2316419 * abs(x));
    let tail = 0.39894228 * exp(-0.5 * x * x) * t *
        (0.31938153 + t * (-0.35656378 + t * (1.78147794 + t * (-1.821256 + t * 1.3302744))));
    return select(tail, 1.0 - tail, x >= 0.0);
}

fn cubic_nearest(point: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, axis: u32, iterations: u32) -> vec4<f32> {
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 3.0 * p0 - 6.0 * p1 + 3.0 * p2;
    let c = 3.0 * (p1 - p0);
    var t = clamp((point[axis] - p0[axis]) / (p3[axis] - p0[axis]), 0.0, 1.0);
    for (var i = 0u; i < iterations; i++) {
        let position = ((a * t + b) * t + c) * t + p0;
        let tangent = (3.0 * a * t + 2.0 * b) * t + c;
        let second = 6.0 * a * t + 2.0 * b;
        let delta = position - point;
        let denominator = dot(tangent, tangent) + dot(delta, second);
        if abs(denominator) > 0.000001 {
            t = clamp(t - clamp(dot(delta, tangent) / denominator, -0.25, 0.25), 0.0, 1.0);
        }
    }
    var position = ((a * t + b) * t + c) * t + p0;
    var tangent = (3.0 * a * t + 2.0 * b) * t + c;
    if dot(point - p0, point - p0) < dot(point - position, point - position) {
        position = p0;
        tangent = c;
    }
    if dot(point - p3, point - p3) < dot(point - position, point - position) {
        position = p3;
        tangent = 3.0 * a + 2.0 * b + c;
    }
    return vec4(position, tangent);
}

fn lens_shape(instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    let half_size = instance.rect_size_px * 0.5;
    if instance.lens_contour_c.z == 0.0 {
        return shape_sd_normal(instance.shape_type, coord - half_size, half_size,
            instance.corner_radii, instance.corner_g2);
    }
    let centered = coord - half_size;
    let point = half_size - abs(centered);
    let join = instance.lens_contour_b.xy * half_size;
    let first = cubic_nearest(point, vec2(0.0, half_size.y),
        instance.lens_contour_a.xy * half_size, instance.lens_contour_a.zw * half_size,
        join, 1u, instance.contour_iterations);
    let second = cubic_nearest(point, join, instance.lens_contour_b.zw * half_size,
        instance.lens_contour_c.xy * half_size, vec2(half_size.x, 0.0), 0u, instance.contour_iterations);
    let a = point - first.xy;
    let b = point - second.xy;
    let nearest = select(first, second, dot(b, b) < dot(a, a));
    let delta = point - nearest.xy;
    let tangent = normalize(nearest.zw);
    let distance = delta.x * tangent.y - delta.y * tangent.x;
    return vec3(distance, abs(vec2(tangent.y, tangent.x)) * sign(centered));
}

fn lens_coverage(instance: GlassUniforms, shape: vec3<f32>) -> f32 {
    let step = lens_raster_step(instance);
    let footprint = max(dot(abs(shape.yz), step), 0.0001);
    if instance.lens_contour_c.z > 0.0 {
        return clamp(0.5 - shape.x / (instance.lens_contour_c.z * footprint), 0.0, 1.0);
    }
    return 1.0 - smoothstep(-0.5 * footprint, 0.5 * footprint, shape.x);
}

fn lens_primary_pixel(instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    let half_size = instance.rect_size_px * 0.5;
    let shape = lens_shape(instance, coord);
    let coverage = lens_coverage(instance, shape);
    let original = lens_scene_sample(instance, coord);
    var color = original;
    if coverage > 0.0 {
        let flow = shape_sd_normal(instance.shape_type, coord - half_size, half_size,
            instance.corner_radii, vec4(instance.refraction_corner_g2));
        let displacement = instance.lens.y * instance.pixels_per_dp *
            lens_profile(flow.x, instance.refraction_width * instance.pixels_per_dp);
        let refracted = lens_scene_sample(instance, coord + flow.yz * displacement);
        color = mix(original, refracted, coverage);
    }
    return color;
}

fn lens_raster_step(instance: GlassUniforms) -> vec2<f32> {
    return instance.raster_step;
}

fn lens_raster_sample(image: texture_2d<f32>, instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    let point = clamp(instance.raster_origin + coord / lens_raster_step(instance) - 0.5,
        vec2(0.0), instance.raster_size - 1.0);
    let low = vec2<i32>(floor(point));
    let high = min(low + 1, vec2<i32>(instance.raster_size) - 1);
    let fraction = fract(point);
    let top = mix(textureLoad(image, low, 0).rgb, textureLoad(image, vec2(high.x, low.y), 0).rgb, fraction.x);
    let bottom = mix(textureLoad(image, vec2(low.x, high.y), 0).rgb, textureLoad(image, high, 0).rgb, fraction.x);
    return mix(top, bottom, fraction.y);
}

fn lens_primary_sample(instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    return lens_raster_sample(t_lens_primary, instance, coord);
}

fn lens_refracted_pixel(instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    let half_size = instance.rect_size_px * 0.5;
    let shape = lens_shape(instance, coord);
    let coverage = lens_coverage(instance, shape);
    var sample_coord = coord;
    if coverage > 0.0 && instance.inner_refraction != 0.0 {
        let flow = shape_sd_normal(instance.shape_type, coord - half_size, half_size,
            instance.corner_radii, vec4(instance.refraction_corner_g2));
        let displacement = instance.inner_refraction * instance.pixels_per_dp *
            lens_profile(flow.x, instance.inner_refraction_width * instance.pixels_per_dp);
        sample_coord += flow.yz * displacement;
    }
    var color = lens_primary_sample(instance, sample_coord);
    if coverage < 1.0 {
        color = mix(lens_primary_sample(instance, coord), color, coverage);
    }
    if instance.lens_shadow.w > 0.0 {
        let shadow_shape = lens_shape(instance, coord - vec2(0.0, instance.lens_shadow.x * instance.pixels_per_dp));
        let distance = shadow_shape.x / instance.pixels_per_dp;
        let alpha = instance.lens_shadow.w * (gaussian_cdf((distance + instance.lens_shadow.y) / instance.lens_shadow.z) -
            gaussian_cdf(distance / instance.lens_shadow.z));
        color *= 1.0 - alpha;
    }
    if coverage < 1.0 {
        let rim = (1.0 - smoothstep(0.75, 1.25, shape.x / instance.pixels_per_dp)) * (1.0 - coverage);
        let side = abs(shape.y);
        let rim_color = pow(max(color, vec3(0.0)), vec3(1.25 + 0.4 * side)) * (0.782 + 0.03 * side);
        color = mix(color, rim_color, rim);
    }
    return color;
}

fn lens_refracted_sample(instance: GlassUniforms, coord: vec2<f32>) -> vec3<f32> {
    return lens_raster_sample(t_lens_background, instance, coord);
}

fn blurred_material_backdrop(instance: GlassUniforms, uv: vec2<f32>, sigma_uv: vec2<f32>) -> vec3<f32> {
    if all(sigma_uv == vec2(0.0)) {
        return to_srgb(textureSampleLevel(t_diffuse, s_diffuse, scene_sample_uv(instance, uv), 0.0).rgb);
    }
    let size = vec2<i32>(textureDimensions(t_diffuse));
    let size_f = vec2<f32>(size);
    var color = vec3(0.0);
    for (var y = 0u; y < 7u; y++) {
        for (var x = 0u; x < 7u; x++) {
            // Interleave quantiles across both axes so stripes do not resolve into seven bands.
            let ox = GAUSSIAN_QUANTILES[7u * x + STRATA_ORDER[y]];
            let oy = GAUSSIAN_QUANTILES[7u * y + 6u - x];
            let offset = vec2(ox, oy) * sigma_uv;
            let point = vec2<i32>(floor(scene_sample_uv(instance, uv + offset) * size_f));
            color += to_srgb(textureLoad(t_diffuse, point, 0).rgb);
        }
    }
    return color / 49.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let instance = uniforms.instances[in.instance_index];

    let local_coord = in.uv * instance.rect_size_px;
    let half_size = instance.rect_size_px * 0.5;
    let centered_coord = local_coord - half_size;

    var shape_eval = shape_sd_normal(
        instance.shape_type,
        centered_coord,
        half_size,
        instance.corner_radii,
        instance.corner_g2,
    );
    if instance.lens.x > 0.0 {
        shape_eval = lens_shape(instance, local_coord);
    }
    let sd = shape_eval.x;
    let aa = max(fwidth(sd), 0.0001);
    var shape_alpha = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, sd);
    if instance.lens.x > 0.0 {
        shape_alpha = lens_coverage(instance, shape_eval);
    }
    if (shape_alpha == 0.0 && instance.lens.x == 0.0) || instance.opacity == 0.0 {
        discard;
    }

    if instance.lens.x > 0.0 {
        let background = lens_refracted_sample(instance, local_coord);
        if shape_alpha == 0.0 {
            return vec4(from_srgb(background), instance.opacity);
        }
        let edge = clamp(1.0 + sd / (instance.lens.x * instance.pixels_per_dp), 0.0, 1.0);
        let normal = shape_eval.yz;
        let optical_normal = shape_sd_normal(instance.shape_type, centered_coord, half_size,
            instance.corner_radii, vec4(instance.refraction_corner_g2)).yz;
        let s = sin(instance.dispersion_angle);
        let c = cos(instance.dispersion_angle);
        let chromatic_direction = vec2(-s * optical_normal.x - c * optical_normal.y, -c * optical_normal.x + s * optical_normal.y);
        let dispersion = chromatic_direction * instance.lens.z * instance.pixels_per_dp;
        let red_weights = array(0.05452, 0.11949, 0.19587, 0.49712, 0.13300);
        let green_weights = array(0.31562, 0.20857, 0.09664, 0.03698);
        var spectrum = vec3(0.0);
        // Keep the shared RGB sampling loop dynamic to avoid duplicating the nested raster stages in driver compilation.
        for (var i = -i32(instance.spectrum_radius); i <= i32(instance.spectrum_radius); i++) {
            let value = lens_refracted_sample(instance, local_coord + dispersion * f32(i));
            let index = u32(abs(i));
            if i >= 0 {
                spectrum.r += value.r * red_weights[index];
            }
            if i <= 0 {
                spectrum.b += value.b * red_weights[index];
            }
            if index < 4u {
                spectrum.g += value.g * green_weights[index];
            }
        }
        var color = mix(background, spectrum, min(1.0, 1.0856 * edge));
        let light_direction = abs(normal.y);
        let key_angular = pow(clamp((light_direction - 0.173648) / 0.826352, 0.0, 1.0), 1.25258);
        let diffuse_angular = pow(clamp((light_direction - 0.615661) / 0.384339, 0.0, 1.0), 3.03428);
        let distance = -sd / instance.pixels_per_dp;
        let glint = 0.796314 * exp(-pow((distance - 0.498737) / 0.445711, 2.0)) * key_angular;
        let diffuse = 0.553605 * pow(lens_profile(sd, 8.24117 * instance.pixels_per_dp), 0.945338) * diffuse_angular;
        let highlighted = vec3(
            dot(color, vec3(1.1201999, -0.1894, -0.019)),
            dot(color, vec3(-0.0563, 0.98709995, -0.0191)),
            dot(color, vec3(-0.0563, -0.1893, 1.1574)),
        ) + vec3(0.1471);
        color = clamp(color + (highlighted - color) * (glint + diffuse), vec3(0.0), vec3(1.0));
        color = mix(vec3(1.0), color, instance.lens.w);
        return vec4(from_srgb(mix(background, color, shape_alpha)), instance.opacity);
    }

    let center_pixel = instance.ripple_center * instance.rect_size_px;
    let ripple_vector = local_coord - center_pixel;
    let dist_pixels = length(ripple_vector);
    let min_dimension = min(instance.rect_size_px.x, instance.rect_size_px.y);
    let radius_pixels = instance.ripple_radius * min_dimension;
    let pixels_per_dp = instance.pixels_per_dp;
    let material = instance.material_mode != 0u;
    var sample_coord = local_coord;
    if material {
        let edge_width = min(27.0 * pixels_per_dp, 0.6 * min_dimension);
        let edge_refraction = select(1.0, 0.4, (instance.material_mode & 4u) != 0u);
        let edge_depth = min(44.0 * pixels_per_dp * edge_refraction, 0.6 * min_dimension);
        let edge = clamp(1.0 + sd / edge_width, 0.0, 1.0);
        sample_coord -= shape_eval.yz * edge_depth * pow(edge, 3.9);
    } else {
        sample_coord = custom_lens_coord(instance, centered_coord, half_size, shape_eval);
    }
    if dist_pixels > 0.0 && dist_pixels < radius_pixels {
        let wave = sin(3.14159265 * dist_pixels / radius_pixels);
        sample_coord -= ripple_vector / dist_pixels * wave * instance.ripple_strength * 3.0 * pixels_per_dp;
    }

    let rect_uv_start = instance.rect_uv_bounds.xy;
    let px_to_uv = (instance.rect_uv_bounds.zw - rect_uv_start) / instance.rect_size_px;
    let sample_uv = rect_uv_start + sample_coord * px_to_uv;
    var color: vec3<f32>;
    let tint_weight = instance.tint_color.a;
    if material {
        color = blurred_material_backdrop(instance, sample_uv, instance.blur_sigma_px * px_to_uv);
        if instance.contrast != 1.0 {
            color = clamp((color - 0.5) * instance.contrast + 0.5, vec3(0.0), vec3(1.0));
        }
        let tone = instance.material_tone;
        let luminance = dot(color, LUMA);
        color = tone.x + tone.y * pow(luminance, tone.w) + tone.z * (color - luminance);
        if tint_weight > 0.0 {
            let tint = to_srgb(instance.tint_color.rgb);
            let tint_luminance = dot(tint, LUMA);
            let chroma = tint - tint_luminance;
            let dark = (instance.material_mode & 2u) != 0u;
            let neutral_shift = select(-0.10, 0.095, dark) + 0.24 * (tint_luminance - 0.5);
            color += (neutral_shift + 0.045 * chroma) * tint_weight;
            color = mix(color, tint, smoothstep(0.35, 1.0, tint_weight));
        }
        color = from_srgb(clamp(color, vec3(0.0), vec3(1.0)));
    } else {
        color = textureSampleLevel(t_diffuse, s_diffuse, scene_sample_uv(instance, sample_uv), 0.0).rgb;
        if tint_weight > 0.0 {
            color = mix(color, instance.tint_color.rgb, tint_weight);
        }
    }
    if dist_pixels < radius_pixels {
        let ripple_factor = 1.0 - dist_pixels / radius_pixels;
        color += vec3<f32>(1.0, 1.0, 1.0) * ripple_factor * instance.ripple_alpha;
    }

    if instance.feedback > 0.0 {
        let light_falloff = 1.0 - smoothstep(0.0, 0.8 * min_dimension, dist_pixels);
        let spread = mix(0.65, 1.0, light_falloff);
        color = mix(color, vec3(1.0), instance.feedback * spread);
    }

    if instance.noise_amount > 0.0 {
        let grain = (rand(local_coord * instance.noise_scale + instance.time) - 0.5) * instance.noise_amount;
        color += grain;
    }

    if instance.border_width > 0.0 && sd > -instance.border_width - aa {
        let rim = smoothstep(-instance.border_width - aa, -instance.border_width + aa, sd);
        let alignment = abs(dot(vec2(-0.71934, 0.69466), shape_eval.yz));
        if material {
            color = mix(color, vec3(1.0), rim * (0.004 + 0.065 * pow(alignment, 6.0)));
        } else {
            let highlight = pow(alignment, 1.5);
            color = mix(color, blend_overlay(color, vec3(rim * highlight)), rim);
        }
    }

    return vec4(color, shape_alpha * instance.opacity);
}
