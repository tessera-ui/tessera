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
    let uv = pos.xy / vec2(u.width, u.height);
    let t = u.time * 0.3;

    let centers = array<vec2<f32>, 4>(
        vec2<f32>(0.3 + 0.1 * sin(t), 0.4 + 0.1 * cos(t)),
        vec2<f32>(0.7 + 0.1 * cos(t * 1.2), 0.5 + 0.1 * sin(t * 1.3)),
        vec2<f32>(0.5 + 0.15 * sin(t * 0.7), 0.7 + 0.1 * cos(t * 0.8)),
        vec2<f32>(0.2 + 0.1 * cos(t * 1.5), 0.8 + 0.1 * sin(t * 1.7))
    );
    let colors = array<vec3<f32>, 4>(
        vec3<f32>(1.0, 0.5, 0.3),
        vec3<f32>(0.3, 0.8, 1.0),
        vec3<f32>(0.9, 0.9, 0.2),
        vec3<f32>(0.7, 0.2, 0.9)
    );
    let radii = array<f32, 4>(0.18, 0.15, 0.13, 0.11);

    var final_color = vec3<f32>(0.45, 0.38, 0.60); // Color of the background
    for (var i = 0; i < 4; i = i + 1) {
        let center = centers[i];
        let color = colors[i];
        let radius = radii[i];

        // calculate distance from the center
        let d = distance(uv, center);
        let edge = smoothstep(radius, radius - 0.03, d);

        // blend the color with the background
        final_color = mix(final_color, color, edge);
    }

    return vec4<f32>(final_color, 1.0);
}