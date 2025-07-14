struct GlassUniforms {
    // Vector values
    tint_color: vec4<f32>,
    highlight_color: vec4<f32>,
    inner_shadow_color: vec4<f32>,
    rect_uv_bounds: vec4<f32>, // x_min, y_min, x_max, y_max

    // vec2 types
    rect_size_px: vec2<f32>,

    // f32 types
    corner_radius: f32,
    g2_k_value: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    highlight_size: f32,
    highlight_smoothing: f32,
    inner_shadow_radius: f32,
    inner_shadow_smoothing: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
};

@group(0) @binding(0) var<uniform> uniforms: GlassUniforms;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),

        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );
    let uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),

        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0)
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[idx], 0.0, 1.0);
    out.uv = uvs[idx];
    return out;
}

fn circle_map(x: f32) -> f32 {
    return 1.0 - sqrt(1.0 - x * x);
}

fn normal_to_tangent(normal: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(normal.y, -normal.x);
}

// G2-like SDF for rounded corners using p-norm
// p: point to sample
// b: half-size of the box
// r: corner radius
// k: exponent for p-norm (k=2.0 for G1 circle, k>2.0 for G2-like superellipse)
fn sdf_g2_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32, k: f32) -> f32 {
    let q = abs(p) - b + r;
    let v = max(q, vec2<f32>(0.0));

    // Fall back to standard circular SDF for k=2
    if (abs(k - 2.0) < 0.001) {
        return length(v) + min(max(q.x, q.y), 0.0) - r;
    }

    // P-norm for the corner distance
    let dist_corner_shape = pow(pow(v.x, k) + pow(v.y, k), 1.0/k);
    
    return dist_corner_shape + min(max(q.x, q.y), 0.0) - r;
}

// Gradient (for normal vector) of the G2 rounded box SDF
fn grad_sd_g2_rounded_box(coord: vec2<f32>, half_size: vec2<f32>, r: f32, k: f32) -> vec2<f32> {
    let inner_half_size = half_size - r;
    let corner_coord = abs(coord) - inner_half_size;

    if (corner_coord.x >= 0.0 && corner_coord.y >= 0.0) { // In a corner region
        // The gradient direction of the p-norm is (x^(k-1), y^(k-1))
        // Add a small epsilon to avoid pow(0, negative) which is NaN
        let grad_dir = vec2<f32>(
            pow(corner_coord.x + 0.0001, k - 1.0), 
            pow(corner_coord.y + 0.0001, k - 1.0)
        );

        // Return the normalized gradient, preserving the original signs
        return sign(coord) * normalize(grad_dir);

    } else { // On a straight edge
        // Determine if it's a vertical or horizontal edge based on which is closer to the center
        if (corner_coord.x > corner_coord.y) {
            return sign(coord) * vec2<f32>(1.0, 0.0);
        } else {
            return sign(coord) * vec2<f32>(0.0, 1.0);
        }
    }
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

fn refraction_color(local_coord: vec2<f32>, size: vec2<f32>, corner_radius: f32, k: f32, eccentric_factor: f32, height: f32, amount: f32, rect_uv_start: vec2<f32>, px_to_uv_ratio: vec2<f32>) -> vec4<f32> {
    let half_size = size * 0.5;
    let centered_coord = local_coord - half_size;
    let sd = sdf_g2_rounded_box(centered_coord, half_size, corner_radius, k);
    
    var refracted_coord = local_coord;
    if (sd < 0.0 && -sd < height) {
        let max_grad_radius = max(min(half_size.x, half_size.y), corner_radius);
        let grad_radius = min(corner_radius * 1.5, max_grad_radius);
        let normal = grad_sd_g2_rounded_box(centered_coord, half_size, grad_radius, k);
        
        let refracted_distance = circle_map(1.0 - (-sd / height)) * amount;
        let refracted_direction = normalize(normal + eccentric_factor * normalize(centered_coord));
        refracted_coord = local_coord + refracted_distance * refracted_direction;
    }
    
    let sample_uv = rect_uv_start + refracted_coord * px_to_uv_ratio;
    return textureSample(t_diffuse, s_diffuse, sample_uv);
}

fn dispersion_color_on_refracted(local_coord: vec2<f32>, size: vec2<f32>, corner_radius: f32, k: f32, dispersion_height: f32, rect_uv_start: vec2<f32>, px_to_uv_ratio: vec2<f32>) -> vec4<f32> {
    let half_size = size * 0.5;
    let centered_coord = local_coord - half_size;
    let sd = sdf_g2_rounded_box(centered_coord, half_size, corner_radius, k);
    
    let base_refracted = refraction_color(local_coord, size, corner_radius, k, uniforms.eccentric_factor, uniforms.refraction_height, uniforms.refraction_amount, rect_uv_start, px_to_uv_ratio);

    if (sd < 0.0 && -sd < dispersion_height && dispersion_height > 0.0) {
        let normal = grad_sd_g2_rounded_box(centered_coord, half_size, corner_radius, k);
        let tangent = normal_to_tangent(normal);
        
        let dispersion_fraction = 1.0 - (-sd / dispersion_height);
        let dispersion_width = dispersion_height * 2.0 * pow(circle_map(dispersion_fraction), 2.0);
        
        if (dispersion_width < 2.0) {
            return base_refracted;
        }
        
        let sample_count = 30;
        var red_color = 0.0;
        var green_color = 0.0;
        var blue_color = 0.0;
        var red_weight = 0.0;
        var green_weight = 0.0;
        var blue_weight = 0.0;
        
        for (var i = 0; i < sample_count; i = i + 1) {
            let t = f32(i) / f32(sample_count - 1);
            let sample_coord = local_coord + tangent * (t - 0.5) * dispersion_width;
            let refracted_c = refraction_color(sample_coord, size, corner_radius, k, uniforms.eccentric_factor, uniforms.refraction_height, uniforms.refraction_amount, rect_uv_start, px_to_uv_ratio);
            
            if (t >= 0.0 && t <= 0.5) {
                blue_color += refracted_c.b;
                blue_weight += 1.0;
            }
            if (t >= 0.25 && t <= 0.75) {
                green_color += refracted_c.g;
                green_weight += 1.0;
            }
            if (t >= 0.5 && t <= 1.0) {
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rect_uv_min = uniforms.rect_uv_bounds.xy;
    let rect_uv_max = uniforms.rect_uv_bounds.zw;

    if (in.uv.x < rect_uv_min.x || in.uv.x > rect_uv_max.x ||
        in.uv.y < rect_uv_min.y || in.uv.y > rect_uv_max.y) {
        discard;
    }

    let rect_uv_size = rect_uv_max - rect_uv_min;
    if (rect_uv_size.x <= 0.0 || rect_uv_size.y <= 0.0) {
        discard;
    }
    let px_to_uv_ratio = rect_uv_size / uniforms.rect_size_px;
    
    let local_uv = (in.uv - rect_uv_min) / rect_uv_size;
    let local_coord = local_uv * uniforms.rect_size_px;
    let half_size = uniforms.rect_size_px * 0.5;
    let centered_coord = local_coord - half_size;
    let k = uniforms.g2_k_value;
    let sd = sdf_g2_rounded_box(centered_coord, half_size, uniforms.corner_radius, k);

    // 1. Get base refracted/dispersed color
    var base_color: vec4<f32>;
    if (uniforms.dispersion_height > 0.0) {
        base_color = dispersion_color_on_refracted(local_coord, uniforms.rect_size_px, uniforms.corner_radius, k, uniforms.dispersion_height, rect_uv_min, px_to_uv_ratio);
    } else {
        base_color = refraction_color(
            local_coord, 
            uniforms.rect_size_px, 
            uniforms.corner_radius, 
            k,
            uniforms.eccentric_factor, 
            uniforms.refraction_height, 
            uniforms.refraction_amount,
            rect_uv_min,
            px_to_uv_ratio
        );
    }

    var color = base_color.rgb;

    // 2. Apply Tint
    let tint_weight = uniforms.tint_color.a;
    if (tint_weight > 0.0) {
        color = mix(color, uniforms.tint_color.rgb, tint_weight);
    }

    // 3. Apply Inner Shadow
    if (uniforms.inner_shadow_color.a > 0.0) {
        let shadow_dist = -sd;
        let shadow_factor = pow(
            1.0 - smoothstep(0.0, uniforms.inner_shadow_radius, shadow_dist), 
            uniforms.inner_shadow_smoothing
        );
        let shadow_alpha = shadow_factor * uniforms.inner_shadow_color.a;
        color = mix(color, uniforms.inner_shadow_color.rgb, shadow_alpha);
    }
    
    // 4. Apply Highlight/Shine
    if (uniforms.highlight_color.a > 0.0) {
        let shine_pos_uv = vec2(0.25, 0.25);
        let dist_to_shine = distance(local_uv, shine_pos_uv);
        let highlight_factor = 1.0 - smoothstep(0.0, uniforms.highlight_size, dist_to_shine);
        let highlight_alpha = pow(highlight_factor, uniforms.highlight_smoothing) * uniforms.highlight_color.a;
        color += uniforms.highlight_color.rgb * highlight_alpha;
    }

    // 5. Apply saturation
    color = saturate_color(vec4(color, base_color.a), uniforms.chroma_multiplier).rgb;
    
    // 6. Apply Noise/Grain
    if (uniforms.noise_amount > 0.0) {
        let grain = (rand(local_coord * uniforms.noise_scale + uniforms.time) - 0.5) * uniforms.noise_amount;
        color += grain;
    }

    // 7. Final Alpha calculation and anti-aliasing
    var final_color = vec4(color, base_color.a);
    if (sd > 0.0) {
        final_color.a = 0.0;
    } else {
        // Anti-alias the edge
        final_color.a *= smoothstep(1.0, -1.0, sd);
    }
    
    return final_color;
}