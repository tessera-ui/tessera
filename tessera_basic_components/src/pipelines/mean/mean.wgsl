struct MeanResult {
    total_luminance: atomic<u32>,
    total_pixels: atomic<u32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> result: MeanResult;
@group(0) @binding(2) var dest_texture: texture_storage_2d<rgba8unorm, write>;

// A simple function to convert RGBA to luminance
fn luminance(color: vec4<f32>) -> f32 {
    return dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_size = textureDimensions(source_texture);
    if global_id.x >= texture_size.x || global_id.y >= texture_size.y {
        return;
    }

    // On the first invocation of the entire dispatch, reset the atomic counters.
    if global_id.x == 0u && global_id.y == 0u && global_id.z == 0u {
        atomicStore(&result.total_luminance, 0u);
        atomicStore(&result.total_pixels, 0u);
    }
    // A memory barrier to ensure all threads see the reset before proceeding.
    workgroupBarrier();

    let color = textureLoad(source_texture, global_id.xy, 0);
    // Convert luminance from [0.0, 1.0] to [0, 255] and then to u32 for atomic addition
    let lum_u32 = u32(luminance(color) * 255.0);

    atomicAdd(&result.total_luminance, lum_u32);
    atomicAdd(&result.total_pixels, 1u);

    // Pass through the input texture to the output
    textureStore(dest_texture, global_id.xy, color);
}