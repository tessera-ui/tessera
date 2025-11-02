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

// Quality knob: larger values blur at lower resolution; set to 1u to disable downscaling.
const DOWNSCALE_FACTOR: u32 = 2u;

fn gaussian_blur(coord: vec2<u32>, direction: vec2<f32>, texture_size: vec2<u32>) -> vec4<f32> {
    var total = vec4<f32>(0.0);
    var total_weight = 0.0;

    let scale = max(DOWNSCALE_FACTOR, 1u);
    let scale_vec = vec2<u32>(scale, scale);
    let scale_i = vec2<i32>(i32(scale), i32(scale));
    let half_scale = vec2<i32>(i32(scale) / 2, i32(scale) / 2);
    let down_coord = vec2<i32>(coord / scale_vec);
    let scale_f = f32(scale);

    let base_radius = uniforms.radius;
    // Decide sample count in the downscaled space.
    let down_radius = base_radius / scale_f;
    let radius = i32(ceil(down_radius));

    let clamped_radius = clamp(radius, 1, 15); // Minimum radius of 1 to avoid division by zero
    let sigma = max(base_radius / 3.0, 0.5); // Minimum sigma to avoid division by zero
    let two_sigma_squared = 2.0 * sigma * sigma;

    let down_texture_size = max(
        vec2<u32>(1u, 1u),
        (texture_size + scale_vec - vec2<u32>(1u, 1u)) / scale_vec,
    );
    let down_texture_size_i = vec2<i32>(down_texture_size);
    let texture_size_i = vec2<i32>(texture_size);
    let dir = vec2<i32>(i32(round(direction.x)), i32(round(direction.y)));

    for (var i = -clamped_radius; i <= clamped_radius; i = i + 1) {
        let sample_down = down_coord + dir * i;
        let sample_down_clamped = clamp(
            sample_down,
            vec2<i32>(0, 0),
            down_texture_size_i - vec2<i32>(1, 1),
        );

        let sample_full = sample_down_clamped * scale_i + half_scale;
        let sample_full_clamped = clamp(
            sample_full,
            vec2<i32>(0, 0),
            texture_size_i - vec2<i32>(1, 1),
        );

        // Calculate Gaussian weight
        let distance = f32(i) * scale_f;
        let distance_squared = distance * distance;
        let weight = exp(-distance_squared / two_sigma_squared);

        let sample_color = textureLoad(source_texture, vec2<u32>(sample_full_clamped), 0);
        total = total + sample_color * weight;
        total_weight = total_weight + weight;
    }

    return total / total_weight;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(dest_texture);
    let coord = global_id.xy;

    if coord.x >= output_size.x || coord.y >= output_size.y {
        return;
    }

    let inside_area = coord.x >= uniforms.area_x
        && coord.x < uniforms.area_x + uniforms.area_width
        && coord.y >= uniforms.area_y
        && coord.y < uniforms.area_y + uniforms.area_height;

    if !inside_area {
        return;
    }

    let texture_size = vec2<u32>(textureDimensions(source_texture));
    let direction = vec2<f32>(uniforms.direction_x, uniforms.direction_y);
    let blurred_color = gaussian_blur(coord, direction, texture_size);
    textureStore(dest_texture, coord, blurred_color);
}
