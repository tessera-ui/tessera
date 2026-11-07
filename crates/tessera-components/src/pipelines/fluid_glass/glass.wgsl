struct GlassUniforms {
    tint_color: vec4<f32>,
    rect_uv_bounds: vec4<f32>,
    corner_radii: vec4<f32>,
    corner_g2: vec4<f32>,
    material_tone: vec4<f32>,
    blur_sigma_px: f32,
    pixels_per_dp: f32,
    opacity: f32,
    material_mode: u32,
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

fn to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(linear, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, linear * 12.92, linear <= vec3(0.0031308));
}

fn from_srgb(encoded: vec3<f32>) -> vec3<f32> {
    return select(pow(max((encoded + 0.055) / 1.055, vec3(0.0)), vec3(2.4)), encoded / 12.92, encoded <= vec3(0.04045));
}

fn blurred_material_backdrop(uv: vec2<f32>, sigma_uv: vec2<f32>) -> vec3<f32> {
    if all(sigma_uv == vec2(0.0)) {
        return to_srgb(textureSampleLevel(t_diffuse, s_diffuse, uv, 0.0).rgb);
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
            let point = clamp(vec2<i32>(floor((uv + offset) * size_f)), vec2(0), size - 1);
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

    let shape_eval = shape_sd_normal(
        instance.shape_type,
        centered_coord,
        half_size,
        instance.corner_radii,
        instance.corner_g2,
    );
    let sd = shape_eval.x;
    let aa = max(fwidth(sd), 0.0001);
    let shape_alpha = 1.0 - smoothstep(-0.5 * aa, 0.5 * aa, sd);
    if shape_alpha == 0.0 || instance.opacity == 0.0 {
        discard;
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
        let edge_depth = min(44.0 * pixels_per_dp, 0.6 * min_dimension);
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
        color = blurred_material_backdrop(sample_uv, instance.blur_sigma_px * px_to_uv);
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
            let dark = instance.material_mode == 2u;
            let neutral_shift = select(-0.10, 0.095, dark) + 0.24 * (tint_luminance - 0.5);
            color += (neutral_shift + 0.045 * chroma) * tint_weight;
            color = mix(color, tint, smoothstep(0.35, 1.0, tint_weight));
        }
        color = from_srgb(clamp(color, vec3(0.0), vec3(1.0)));
    } else {
        color = textureSampleLevel(t_diffuse, s_diffuse, sample_uv, 0.0).rgb;
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
