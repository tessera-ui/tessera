struct Uniforms {
    rect: vec4<f32>,
    viewport_scene: vec4<f32>,
    color: vec4<f32>,
    corner: vec4<f32>,
    stroke_color: vec4<f32>,
    stroke_start_joint: vec4<f32>,
    stroke_end_width: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var backdrop: texture_2d<f32>;

fn encode(value: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(value, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, value * 12.92, value <= vec3(0.0031308));
}
fn decode(value: vec3<f32>) -> vec3<f32> {
    return select(pow((value + 0.055) / 1.055, vec3(2.4)), value / 12.92, value <= vec3(0.04045));
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0));
    let window = uniforms.rect.xy + corners[index] * uniforms.rect.zw;
    let ndc = window / uniforms.viewport_scene.xy * 2.0 - 1.0;
    return vec4(ndc.x, -ndc.y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let half_size = uniforms.rect.zw * 0.5;
    let q = abs(position.xy - uniforms.rect.xy - half_size) - half_size + uniforms.corner.x;
    let distance = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - uniforms.corner.x;
    let coverage = clamp(0.5 - distance / uniforms.corner.y, 0.0, 1.0);
    let alpha = uniforms.color.a * coverage;
    let local = position.xy - uniforms.rect.xy;
    let start = uniforms.stroke_start_joint.xy * uniforms.rect.zw;
    let joint = uniforms.stroke_start_joint.zw * uniforms.rect.zw;
    let end = uniforms.stroke_end_width.xy * uniforms.rect.zw;
    let stroke_distance = min(segment_distance(local, start, joint), segment_distance(local, joint, end));
    let stroke_alpha = uniforms.stroke_color.a * clamp(uniforms.stroke_end_width.z * 0.5 + 0.5 - stroke_distance, 0.0, 1.0) * coverage;
    if alpha == 0.0 && stroke_alpha == 0.0 { discard; }
    let background = textureLoad(backdrop, vec2<i32>(floor(position.xy - uniforms.viewport_scene.zw)), 0);
    let straight = encode(background.rgb / max(background.a, 0.000001));
    let fill_alpha = background.a + alpha * (1.0 - background.a);
    let fill = encode(uniforms.color.rgb) * alpha + straight * background.a * (1.0 - alpha);
    let output_alpha = fill_alpha + stroke_alpha * (1.0 - fill_alpha);
    let color = (encode(uniforms.stroke_color.rgb) * stroke_alpha + fill * (1.0 - stroke_alpha)) / output_alpha;
    return vec4(decode(color) * output_alpha, output_alpha);
}

fn segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let line = end - start;
    let fraction = clamp(dot(point - start, line) / max(dot(line, line), 0.000001), 0.0, 1.0);
    return length(point - start - fraction * line);
}
