struct RectUniform {
    position: vec4<f32>,
    color: vec4<f32>,
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

struct RectInstances {
    instances: array<RectUniform>,
}

@group(0) @binding(0)
var<storage, read> uniforms: RectInstances;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let uniform = uniforms.instances[in.instance_index];
    let pixel_pos = uniform.position.xy + in.position * uniform.position.zw;
    let clip = vec2<f32>(
        (pixel_pos.x / uniform.screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / uniform.screen_size.y) * -2.0 + 1.0,
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip, 0.0, 1.0);
    out.color = uniform.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
