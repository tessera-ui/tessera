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

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_coord = global_id.xy;

    if thread_coord.x >= uniforms.area_width || thread_coord.y >= uniforms.area_height {
        return;
    }

    let coord = vec2<u32>(uniforms.area_x, uniforms.area_y) + thread_coord;
    let output_size = textureDimensions(dest_texture);

    if coord.x >= output_size.x || coord.y >= output_size.y {
        return;
    }

    let scale = max(uniforms.scale, 1u);
    let scale_f = f32(scale);
    let area_origin = vec2<f32>(f32(uniforms.area_x), f32(uniforms.area_y));

    let local = vec2<f32>(vec2<f32>(thread_coord)) / scale_f;
    let base = floor(local);
    let frac = local - base;

    let texture_size = vec2<i32>(vec2<u32>(textureDimensions(source_texture)));
    let max_coord = max(texture_size - vec2<i32>(1, 1), vec2<i32>(0, 0));

    let base_i = vec2<i32>(base);
    let base_clamped = clamp(base_i, vec2<i32>(0, 0), max_coord);
    let next_x = clamp(base_clamped + vec2<i32>(1, 0), vec2<i32>(0, 0), max_coord);
    let next_y = clamp(base_clamped + vec2<i32>(0, 1), vec2<i32>(0, 0), max_coord);
    let next_xy = clamp(base_clamped + vec2<i32>(1, 1), vec2<i32>(0, 0), max_coord);

    let c00 = textureLoad(source_texture, vec2<u32>(base_clamped), 0);
    let c10 = textureLoad(source_texture, vec2<u32>(next_x), 0);
    let c01 = textureLoad(source_texture, vec2<u32>(next_y), 0);
    let c11 = textureLoad(source_texture, vec2<u32>(next_xy), 0);

    let row0 = mix(c00, c10, clamp(frac.x, 0.0, 1.0));
    let row1 = mix(c01, c11, clamp(frac.x, 0.0, 1.0));
    let color = mix(row0, row1, clamp(frac.y, 0.0, 1.0));

    textureStore(dest_texture, coord, color);
}
