struct GlassUniforms {
    rect_size_px: vec2<f32>,
    corner_radius: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
};

@group(0) @binding(0) var<uniform> uniforms: GlassUniforms;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

fn circle_map(x: f32) -> f32 {
    return 1.0 - sqrt(1.0 - x * x);
}

fn normal_to_tangent(normal: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(normal.y, -normal.x);
}

fn sd_rectangle(coord: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let d = abs(coord) - half_size;
    let outside = length(max(d, vec2<f32>(0.0)));
    let inside = min(max(d.x, d.y), 0.0);
    return outside + inside;
}

fn sd_rounded_rectangle(coord: vec2<f32>, half_size: vec2<f32>, corner_radius: f32) -> f32 {
    let inner_half_size = half_size - vec2<f32>(corner_radius);
    return sd_rectangle(coord, inner_half_size) - corner_radius;
}

fn grad_sd_rounded_rectangle(coord: vec2<f32>, half_size: vec2<f32>, corner_radius: f32) -> vec2<f32> {
    let inner_half_size = half_size - vec2<f32>(corner_radius);
    let corner_coord = abs(coord) - inner_half_size;
    
    if (corner_coord.x >= 0.0 && corner_coord.y >= 0.0) {
        return sign(coord) * normalize(corner_coord);
    } else {
        if (-corner_coord.x < -corner_coord.y) {
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

fn refraction_color(coord: vec2<f32>, size: vec2<f32>, corner_radius: f32, eccentric_factor: f32, height: f32, amount: f32) -> vec4<f32> {
    let half_size = size * 0.5;
    let centered_coord = coord - half_size;
    let sd = sd_rounded_rectangle(centered_coord, half_size, corner_radius);
    
    if (sd < 0.0 && -sd < height) {
        let max_grad_radius = max(min(half_size.x, half_size.y), corner_radius);
        let grad_radius = min(corner_radius * 1.5, max_grad_radius);
        let normal = grad_sd_rounded_rectangle(centered_coord, half_size, grad_radius);
        
        let refracted_distance = circle_map(1.0 - (-sd / height)) * amount;
        let refracted_direction = normalize(normal + eccentric_factor * normalize(centered_coord));
        let refracted_coord = coord + refracted_distance * refracted_direction;
        
        // 边界夹紧而不是返回黑色
        let clamped_coord = clamp(refracted_coord, vec2<f32>(0.0), size - vec2<f32>(1.0));
        let refracted_uv = clamped_coord / size;
        return textureSample(t_diffuse, s_diffuse, refracted_uv);
    } else {
        let uv = coord / size;
        return textureSample(t_diffuse, s_diffuse, uv);
    }
}

fn dispersion_color_on_refracted(coord: vec2<f32>, size: vec2<f32>, corner_radius: f32, dispersion_height: f32) -> vec4<f32> {
    let half_size = size * 0.5;
    let centered_coord = coord - half_size;
    let sd = sd_rounded_rectangle(centered_coord, half_size, corner_radius);
    
    if (sd < 0.0 && -sd < dispersion_height && dispersion_height > 0.0) {
        let normal = grad_sd_rounded_rectangle(centered_coord, half_size, corner_radius);
        let tangent = normal_to_tangent(normal);
        
        let dispersion_fraction = 1.0 - (-sd / dispersion_height);
        let dispersion_width = dispersion_height * 2.0 * pow(circle_map(dispersion_fraction), 2.0);
        
        if (dispersion_width < 2.0) {
            return refraction_color(coord, size, corner_radius, uniforms.eccentric_factor, uniforms.refraction_height, uniforms.refraction_amount);
        }
        
        let sample_count = 30;
        var red_color = 0.0;
        var green_color = 0.0;
        var blue_color = 0.0;
        var red_weight = 0.0;
        var green_weight = 0.0;
        var blue_weight = 0.0;
        
        for (var i = 0; i < sample_count; i++) {
            let t = f32(i) / f32(sample_count - 1);
            let sample_coord = coord + tangent * (t - 0.5) * dispersion_width;
            let refracted_color = refraction_color(sample_coord, size, corner_radius, uniforms.eccentric_factor, uniforms.refraction_height, uniforms.refraction_amount);
            
            if (t >= 0.0 && t <= 0.5) {
                blue_color += refracted_color.b;
                blue_weight += 1.0;
            }
            if (t >= 0.25 && t <= 0.75) {
                green_color += refracted_color.g;
                green_weight += 1.0;
            }
            // 红色通道 (0.5 - 1.0)
            if (t >= 0.5 && t <= 1.0) {
                red_color += refracted_color.r;
                red_weight += 1.0;
            }
        }
        
        // 归一化
        red_color = red_color / max(red_weight, 1.0);
        green_color = green_color / max(green_weight, 1.0);
        blue_color = blue_color / max(blue_weight, 1.0);
        
        // 保持原始alpha
        let original_refracted = refraction_color(coord, size, corner_radius, uniforms.eccentric_factor, uniforms.refraction_height, uniforms.refraction_amount);
        
        return vec4<f32>(red_color, green_color, blue_color, original_refracted.a);
    } else {
        // 在玻璃区域外，直接应用折射
        return refraction_color(coord, size, corner_radius, uniforms.eccentric_factor, uniforms.refraction_height, uniforms.refraction_amount);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coord = in.uv * uniforms.rect_size_px;
    
    var color: vec4<f32>;
    if (uniforms.dispersion_height > 0.0) {
        color = dispersion_color_on_refracted(coord, uniforms.rect_size_px, uniforms.corner_radius, uniforms.dispersion_height);
    } else {
        color = refraction_color(
            coord, 
            uniforms.rect_size_px, 
            uniforms.corner_radius, 
            uniforms.eccentric_factor, 
            uniforms.refraction_height, 
            uniforms.refraction_amount
        );
    }
    
    color = saturate_color(color, uniforms.chroma_multiplier);
    
    let half_size = uniforms.rect_size_px * 0.5;
    let centered_coord = coord - half_size;
    let sd = sd_rounded_rectangle(centered_coord, half_size, uniforms.corner_radius);
    
    if (sd > 0.0) {
        color.a = 0.0;
    }
    
    return color;
}
