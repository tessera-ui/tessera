struct GlassUniforms {
    rect_size_px: vec2<f32>,
    corner_radius: f32,
    
    // Refraction parameters
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    
    // Dispersion parameters  
    dispersion_height: f32,
    
    // Chroma boost
    chroma_multiplier: f32,
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

fn circle_map(x: f32) -> f32 {
    return 1.0 - sqrt(1.0 - x * x);
}

fn linearized_rgb(rgb: vec3<f32>) -> vec3<f32> {
    let linear_threshold = vec3<f32>(0.04045);
    let low = rgb / 12.92;
    let high = pow((rgb + 0.055) / 1.055, vec3<f32>(2.4));
    return mix(low, high, step(linear_threshold, rgb));
}

fn luma(color: vec4<f32>) -> f32 {
    let rgb_to_y = vec3<f32>(0.2126, 0.7152, 0.0722);
    return dot(linearized_rgb(color.rgb), rgb_to_y);
}

fn refraction(coord: vec2<f32>, height: f32, amount: f32) -> vec4<f32> {
    let half_size = uniforms.rect_size_px * 0.5;
    let centered_coord = coord - half_size;
    let sd = sd_rounded_rectangle(centered_coord, half_size, uniforms.corner_radius);
    
    if (sd < 0.0 && -sd < height) {
        let max_grad_radius = max(min(half_size.x, half_size.y), uniforms.corner_radius);
        let grad_radius = min(uniforms.corner_radius * 1.5, max_grad_radius);
        let normal = grad_sd_rounded_rectangle(centered_coord, half_size, grad_radius);
        
        let refracted_distance = circle_map(1.0 - (-sd) / height) * amount;
        let refracted_direction = normalize(normal + uniforms.eccentric_factor * normalize(centered_coord));
        let refracted_coord = coord + refracted_distance * refracted_direction;
        
        if (refracted_coord.x < 0.0 || refracted_coord.x >= uniforms.rect_size_px.x ||
            refracted_coord.y < 0.0 || refracted_coord.y >= uniforms.rect_size_px.y) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
        
        let refracted_uv = refracted_coord / uniforms.rect_size_px;
        return textureSample(t_diffuse, s_diffuse, refracted_uv);
    } else {
        let uv = coord / uniforms.rect_size_px;
        return textureSample(t_diffuse, s_diffuse, uv);
    }
}

fn normal_to_tangent(normal: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(normal.y, -normal.x);
}

fn dispersion_effect(coord: vec2<f32>) -> vec4<f32> {
    let half_size = uniforms.rect_size_px * 0.5;
    let centered_coord = coord - half_size;
    let sd = sd_rounded_rectangle(centered_coord, half_size, uniforms.corner_radius);
    
    if (sd < 0.0 && -sd < uniforms.dispersion_height) {
        let normal = grad_sd_rounded_rectangle(centered_coord, half_size, uniforms.corner_radius);
        let tangent = normal_to_tangent(normal);
        
        var dispersed_color = vec4<f32>(0.0);
        var weight = vec4<f32>(0.0);
        
        let dispersion_fraction = 1.0 - (-sd) / uniforms.dispersion_height;
        let dispersion_width = uniforms.dispersion_height * 2.0 * pow(circle_map(dispersion_fraction), 2.0);
        
        if (dispersion_width < 2.0) {
            let uv = coord / uniforms.rect_size_px;
            return textureSample(t_diffuse, s_diffuse, uv);
        }
        
        let max_samples = min(dispersion_width, 100.0);
        for (var i = 0.0; i < 100.0; i += 1.0) {
            let t = i / max_samples;
            if (t > 1.0) { break; }
            
            let sample_coord = coord + tangent * (t - 0.5) * dispersion_width;
            let sample_uv = sample_coord / uniforms.rect_size_px;
            let color = textureSample(t_diffuse, s_diffuse, sample_uv);
            
            if (t >= 0.0 && t < 0.5) {
                dispersed_color.b += color.b;
                weight.b += 1.0;
            }
            if (t > 0.25 && t < 0.75) {
                dispersed_color.g += color.g;
                weight.g += 1.0;
            }
            if (t > 0.5 && t <= 1.0) {
                dispersed_color.r += color.r;
                weight.r += 1.0;
            }
        }
        
        dispersed_color /= max(weight, vec4<f32>(1.0));
        let uv = coord / uniforms.rect_size_px;
        let original_color = textureSample(t_diffuse, s_diffuse, uv);
        dispersed_color.a = original_color.a;
        
        return dispersed_color;
    } else {
        let uv = coord / uniforms.rect_size_px;
        return textureSample(t_diffuse, s_diffuse, uv);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coord = in.uv * uniforms.rect_size_px;
    
    let half_size = uniforms.rect_size_px * 0.5;
    let centered_coord = coord - half_size;
    let sd = sd_rounded_rectangle(centered_coord, half_size, uniforms.corner_radius);
    
    if (sd > 0.0) {
        discard;
    }
    
    var final_color: vec4<f32>;
    
    if (uniforms.dispersion_height > 0.0) {
        final_color = dispersion_effect(coord);
    } else {
        final_color = refraction(coord, uniforms.refraction_height, uniforms.refraction_amount);
    }
    
    return final_color;
}