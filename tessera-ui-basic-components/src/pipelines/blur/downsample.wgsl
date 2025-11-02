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
    let output_size = textureDimensions(dest_texture);
    let coord = global_id.xy;

    if coord.x >= output_size.x || coord.y >= output_size.y {
        return;
    }

    let scale = max(uniforms.scale, 1u);
    let area_origin = vec2<i32>(i32(uniforms.area_x), i32(uniforms.area_y));
    let area_size = vec2<i32>(i32(uniforms.area_width), i32(uniforms.area_height));
    let max_sample = area_origin + max(area_size - vec2<i32>(1, 1), vec2<i32>(0, 0));

    let coord_i = vec2<i32>(coord);
    let scale_i = i32(scale);
    let base = area_origin + coord_i * scale_i;

    var total = vec4<f32>(0.0);
    var sample_count = 0.0;

    for (var y = 0; y < scale_i; y = y + 1) {
        for (var x = 0; x < scale_i; x = x + 1) {
            let sample_coord = clamp(base + vec2<i32>(x, y), area_origin, max_sample);
            let sample_color = textureLoad(source_texture, vec2<u32>(sample_coord), 0);
            total = total + sample_color;
            sample_count = sample_count + 1.0;
        }
    }

    let average = total / sample_count;
    textureStore(dest_texture, coord, average);
}
