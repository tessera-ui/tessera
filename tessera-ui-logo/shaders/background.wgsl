struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0)
    );
    let p = positions[in_vertex_index];
    return vec4<f32>(p.x, p.y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // A simple, robust, and proven procedural shader from Shadertoy.
    // This will definitively prove if the pipeline is working.
    let uv = pos.xy / vec2(u.width, u.height);
    let t = u.time;

    let final_color = 0.5 + 0.5 * cos(t + uv.xyx + vec3(0.0, 2.0, 4.0));

    return vec4(final_color, 1.0);
}