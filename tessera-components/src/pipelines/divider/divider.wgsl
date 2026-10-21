struct Instance {
    bounds: vec4<f32>,
    color: vec4<f32>,
    screen: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> instances: array<Instance>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) instance: u32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(0.0, 0.0));
    let instance = instances[index];
    // Whole-pixel geometry prevents MSAA from applying edge coverage twice.
    let point = mix(floor(instance.bounds.xy), ceil(instance.bounds.zw), corners[vertex]);
    var output: VertexOutput;
    output.position = vec4(point / instance.screen.xy * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    output.instance = index;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let instance = instances[input.instance];
    let overlap = clamp(min(input.position.xy + vec2(0.5), instance.bounds.zw) - max(input.position.xy - vec2(0.5), instance.bounds.xy), vec2(0.0), vec2(1.0));
    return vec4(instance.color.rgb, instance.color.a * overlap.x * overlap.y);
}
