struct Uniforms {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    scale: u32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var source_texture: texture_2d<f32>;
@group(0) @binding(2) var dest_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var linear_sampler: sampler;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(dest_texture);
    let coord = global_id.xy;

    if coord.x >= output_size.x || coord.y >= output_size.y {
        return;
    }

    let scale = max(uniforms.scale, 1u);
    let scale_f = f32(scale);
    let area_origin = vec2<f32>(f32(uniforms.area_x), f32(uniforms.area_y));
    let tex_size = vec2<f32>(textureDimensions(source_texture));

    let coord_f = vec2<f32>(coord);
    let src_center = area_origin + (coord_f + vec2<f32>(0.5)) * scale_f;
    let uv = src_center / tex_size;
    let color = textureSampleLevel(source_texture, linear_sampler, uv, 0.0);
    textureStore(dest_texture, coord, color);
}
