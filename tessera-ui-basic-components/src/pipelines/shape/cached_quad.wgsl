struct RectUniform {
    position: vec4<f32>,
    screen_size: vec2<f32>,
    padding: vec2<f32>,
}

struct RectUniforms {
    rects: array<RectUniform>,
}

@group(0) @binding(0)
var cache_sampler: sampler;

@group(0) @binding(1)
var cache_texture: texture_2d<f32>;

@group(1) @binding(0)
var<storage, read> rect_data: RectUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) instance_idx: u32) -> VertexOutput {
    let rect = rect_data.rects[instance_idx];
    let pixel_pos = rect.position.xy + in.position * rect.position.zw;
    let clip = vec2<f32>(
        (pixel_pos.x / rect.screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / rect.screen_size.y) * -2.0 + 1.0,
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip, 0.0, 1.0);
    out.uv = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(cache_texture, cache_sampler, in.uv);
}
