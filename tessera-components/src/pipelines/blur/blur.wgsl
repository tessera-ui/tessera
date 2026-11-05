// With bilinear optimization, 16 samples can effectively cover a blur radius of ~30 pixels
const MAX_SAMPLES: u32 = 16u;

struct Uniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    sample_count: u32,
};

// Padded to vec4 for 16-byte alignment required by WGSL uniform buffers
struct WeightsAndOffsets {
    weights: array<vec4<f32>, 16>,
    offsets: array<vec4<f32>, 16>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var dest_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var<uniform> weights_and_offsets: WeightsAndOffsets;

fn gaussian_blur(coord: vec2<u32>, direction: vec2<f32>, texture_size: vec2<u32>) -> vec4<f32> {
    let sample_count = min(uniforms.sample_count, MAX_SAMPLES);

    if sample_count == 0u {
        return textureLoad(source_texture, coord, 0);
    }

    let coord_uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / vec2<f32>(texture_size);
    let tex_size_inv = vec2<f32>(1.0) / vec2<f32>(texture_size);

    var total = textureSampleLevel(source_texture, linear_sampler, coord_uv, 0.0) * weights_and_offsets.weights[0].x;

    // Each textureSample with bilinear filtering effectively samples 2 adjacent pixels
    for (var i = 1u; i < sample_count; i = i + 1u) {
        let offset_pixels = weights_and_offsets.offsets[i].x;
        let offset = direction * offset_pixels * tex_size_inv;
        let weight = weights_and_offsets.weights[i].x;

        total = total + textureSampleLevel(source_texture, linear_sampler, coord_uv + offset, 0.0) * weight;
        total = total + textureSampleLevel(source_texture, linear_sampler, coord_uv - offset, 0.0) * weight;
    }

    return total;
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
    textureStore(dest_texture, vec2<i32>(coord), blurred_color);
}
