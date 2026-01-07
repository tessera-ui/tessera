struct Uniforms {
    contrast: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
};

struct MeanResult {
    total_luminance: u32,
    total_pixels: u32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var dest_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<storage, read> mean_result: MeanResult;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(dest_texture);
    let coord = global_id.xy;

    if global_id.x >= uniforms.area_x && global_id.x < uniforms.area_x + uniforms.area_width && global_id.y >= uniforms.area_y && global_id.y < uniforms.area_y + uniforms.area_height && global_id.x < output_size.x && global_id.y < output_size.y {

        // Calculate mean luminance on the fly
        var mean_luminance: f32 = 0.5;
        if mean_result.total_pixels > 0u {
            mean_luminance = (f32(mean_result.total_luminance) / f32(mean_result.total_pixels)) / 255.0;
        }

        let original_color = textureLoad(source_texture, coord, 0);
        let mean_vec = vec3<f32>(mean_luminance);

        // Apply contrast adjustment using the mean luminance
        let adjusted_color_rgb = (original_color.rgb - mean_vec) * uniforms.contrast + mean_vec;
        let clamped_color_rgb = clamp(adjusted_color_rgb, vec3<f32>(0.0), vec3<f32>(1.0));
        let final_color = vec4<f32>(clamped_color_rgb, original_color.a);

        textureStore(dest_texture, coord, final_color);
    } else if global_id.x < output_size.x && global_id.y < output_size.y {
        let color = textureLoad(source_texture, coord, 0);
        textureStore(dest_texture, coord, color);
    }
}
