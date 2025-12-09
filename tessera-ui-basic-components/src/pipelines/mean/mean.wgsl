struct AreaUniform {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
};
struct MeanResult {
    total_luminance: atomic<u32>,
    total_pixels: atomic<u32>,
};

@group(0) @binding(0) var<uniform> area: AreaUniform;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> result: MeanResult;
@group(0) @binding(3) var dest_texture: texture_storage_2d<rgba8unorm, write>;

// A simple function to convert RGBA to luminance
fn luminance(color: vec4<f32>) -> f32 {
    return dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_size = textureDimensions(source_texture);

    if global_id.x >= area.area_x && global_id.x < area.area_x + area.area_width && global_id.y >= area.area_y && global_id.y < area.area_y + area.area_height && global_id.x < texture_size.x && global_id.y < texture_size.y {


        let color = textureLoad(source_texture, global_id.xy, 0);
        let lum_u32 = u32(luminance(color) * 255.0);

        atomicAdd(&result.total_luminance, lum_u32);
        atomicAdd(&result.total_pixels, 1u);

        textureStore(dest_texture, global_id.xy, color);
    } else if global_id.x < texture_size.x && global_id.y < texture_size.y {
        let color = textureLoad(source_texture, global_id.xy, 0);
        textureStore(dest_texture, global_id.xy, color);
    }
}
