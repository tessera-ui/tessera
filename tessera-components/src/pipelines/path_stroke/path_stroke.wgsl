struct Tile {
    position: vec4<f32>,
    bounds: vec4<f32>,
    color: vec4<f32>,
    screen: vec4<f32>,
    geometry: vec4<u32>,
}

struct Segment {
    points: vec4<f32>,
    caps: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> tiles: array<Tile>;
@group(0) @binding(1) var<storage, read> segments: array<Segment>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) tile: u32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let tile = tiles[index];
    let point = tile.position.xy + mix(tile.bounds.xy, tile.bounds.zw, corners[vertex]) * tile.position.w;
    var output: VertexOutput;
    output.position = vec4(point / tile.screen.xy * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    output.tile = index;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tile = tiles[input.tile];
    let point = (input.position.xy - tile.position.xy) / tile.position.w;
    var distance = 1e20;
    var cap_distance = -1e20;
    for (var i = 0u; i < tile.geometry.y; i++) {
        let segment = segments[tile.geometry.x + i];
        let a = segment.points.xy;
        let direction = segment.points.zw - a;
        let length = dot(direction, direction);
        let t = dot(point - a, direction) / max(length, 1e-20);
        let d = distanceToSegment(point, a, direction, t);
        if d < distance {
            distance = d;
            cap_distance = -1e20;
            if segment.caps.x != 0.0 && t < 0.0 {
                cap_distance = -t * sqrt(length);
            }
            if segment.caps.y != 0.0 && t > 1.0 {
                cap_distance = (t - 1.0) * sqrt(length);
            }
        }
    }
    distance = max(distance - tile.position.z, cap_distance);
    let coverage = clamp(0.5 - distance * tile.position.w, 0.0, 1.0);
    return vec4(tile.color.rgb, tile.color.a * coverage);
}

fn distanceToSegment(point: vec2<f32>, a: vec2<f32>, direction: vec2<f32>, t: f32) -> f32 {
    return length(point - a - direction * clamp(t, 0.0, 1.0));
}
