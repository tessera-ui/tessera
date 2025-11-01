struct Uniforms {
    radius: f32,
    direction_x: f32, // 1.0 for horizontal, 0.0 for vertical
    direction_y: f32, // 0.0 for horizontal, 1.0 for vertical
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var dest_texture: texture_storage_2d<rgba8unorm, write>;

fn gaussian_blur(coord: vec2<u32>, direction: vec2<f32>, texture_size: vec2<u32>) -> vec4<f32> {
    var total = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Use the original float radius for better precision
    let float_radius = uniforms.radius;
    let radius = i32(ceil(float_radius));

    let clamped_radius = clamp(radius, 1, 15); // Minimum radius of 1 to avoid division by zero
    let sigma = max(float_radius / 3.0, 0.5); // Minimum sigma to avoid division by zero
    let two_sigma_squared = 2.0 * sigma * sigma;

    for (var i = -clamped_radius; i <= clamped_radius; i = i + 1) {
        let offset = direction * f32(i);
        let sample_coord = clamp(
            vec2<i32>(coord) + vec2<i32>(offset),
            vec2<i32>(0),
            vec2<i32>(texture_size) - vec2<i32>(1)
        );
        
        // Calculate Gaussian weight
        let distance_squared = f32(i * i);
        let weight = exp(-distance_squared / two_sigma_squared);

        let sample_color = textureLoad(source_texture, vec2<u32>(sample_coord), 0);
        total = total + sample_color * weight;
        total_weight = total_weight + weight;
    }

    return total / total_weight;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(dest_texture);
    let local = global_id.xy;

    if local.x >= uniforms.area_width || local.y >= uniforms.area_height {
        return;
    }

    let coord = vec2<u32>(uniforms.area_x, uniforms.area_y) + local;

    if coord.x >= output_size.x || coord.y >= output_size.y {
        return;
    }

    let texture_size = vec2<u32>(textureDimensions(source_texture));
    let direction = vec2<f32>(uniforms.direction_x, uniforms.direction_y);
    let blurred_color = gaussian_blur(coord, direction, texture_size);
    textureStore(dest_texture, coord, blurred_color);
}
