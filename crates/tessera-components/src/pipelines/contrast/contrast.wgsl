struct Uniforms {
    contrast: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
};

struct MeanResult {
    total_luminance: atomic<u32>,
    total_pixels: atomic<u32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var dest_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<storage, read_write> mean_result: MeanResult;

fn luminance(color: vec4<f32>) -> f32 {
    return dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(1, 1, 1)
fn reset() {
    atomicStore(&mean_result.total_luminance, 0u);
    atomicStore(&mean_result.total_pixels, 0u);
}

// Both passes are dispatched over the area only.
fn area_coord(global_id: vec3<u32>) -> vec2<u32> {
    return global_id.xy + vec2<u32>(uniforms.area_x, uniforms.area_y);
}

fn outside_area(global_id: vec3<u32>) -> bool {
    return global_id.x >= uniforms.area_width || global_id.y >= uniforms.area_height
        || any(area_coord(global_id) >= textureDimensions(source_texture));
}

@compute @workgroup_size(8, 8, 1)
fn accumulate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if outside_area(global_id) {
        return;
    }
    let color = textureLoad(source_texture, area_coord(global_id), 0);
    atomicAdd(&mean_result.total_luminance, u32(luminance(color) * 255.0));
    atomicAdd(&mean_result.total_pixels, 1u);
}

@compute @workgroup_size(8, 8, 1)
fn apply(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if outside_area(global_id) {
        return;
    }
    let coord = area_coord(global_id);
    let color = textureLoad(source_texture, coord, 0);
    var mean = 0.5;
    let pixels = atomicLoad(&mean_result.total_pixels);
    if pixels > 0u {
        mean = f32(atomicLoad(&mean_result.total_luminance)) / f32(pixels) / 255.0;
    }
    let adjusted = clamp((color.rgb - vec3<f32>(mean)) * uniforms.contrast + vec3<f32>(mean), vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(dest_texture, coord, vec4<f32>(adjusted, color.a));
}
