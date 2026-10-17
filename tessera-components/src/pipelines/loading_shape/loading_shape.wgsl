struct CubicPair {
    start: array<vec2<f32>, 4>,
    end: array<vec2<f32>, 4>,
}

struct Instance {
    position: vec4<f32>,
    color: vec4<f32>,
    center_rotation: vec4<f32>,
    screen_scale_progress: vec4<f32>,
    geometry: vec4<u32>,
}

@group(0) @binding(0) var<storage, read> curves: array<CubicPair>;
@group(1) @binding(0) var<storage, read> instances: array<Instance>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) instance: u32,
    @location(1) @interpolate(flat) edge: vec4<f32>,
}

fn cubic_point(curve: CubicPair, t: f32, progress: f32) -> vec2<f32> {
    let u = 1.0 - t;
    return mix(curve.start[0], curve.end[0], progress) * (u * u * u)
        + mix(curve.start[1], curve.end[1], progress) * (3.0 * u * u * t)
        + mix(curve.start[2], curve.end[2], progress) * (3.0 * u * t * t)
        + mix(curve.start[3], curve.end[3], progress) * (t * t * t);
}

fn rotate(instance: Instance, point: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(point.x * instance.center_rotation.z - point.y * instance.center_rotation.w,
        point.x * instance.center_rotation.w + point.y * instance.center_rotation.z);
}

fn pixel_point(instance: Instance, point: vec2<f32>) -> vec2<f32> {
    let local = (point - instance.center_rotation.xy) * instance.position.z * instance.screen_scale_progress.z;
    return instance.position.xy + instance.position.zw * 0.5 + rotate(instance, local);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> VertexOutput {
    let instance = instances[index];
    let triangle = vertex / 3u;
    let curve_index = triangle / instance.geometry.z;
    var pixel = pixel_point(instance, instance.center_rotation.xy);
    var edge = vec4<f32>(pixel, 0.0, 0.0);
    if (curve_index < instance.geometry.y) {
        let curve = curves[instance.geometry.x + curve_index];
        let step = f32(triangle % instance.geometry.z);
        let p = instance.screen_scale_progress.w;
        let start = pixel_point(instance, cubic_point(curve, step / f32(instance.geometry.z), p));
        let end = pixel_point(instance, cubic_point(curve, (step + 1.0) / f32(instance.geometry.z), p));
        let direction = end - start;
        let normal = vec2<f32>(-direction.y, direction.x) / max(length(direction), 0.000001);
        edge = vec4<f32>(start, normal);
        if (vertex % 3u != 0u) {
            let t = (step + f32(vertex % 3u - 1u)) / f32(instance.geometry.z);
            pixel = pixel_point(instance, cubic_point(curve, t, p));
            if (instance.geometry.w != 0u) {
                // Shared radial expansion keeps fan boundaries watertight at zero tangents.
                let center = instance.position.xy + instance.position.zw * 0.5;
                pixel += (pixel - center) * (6.0 / max(instance.position.z, 1.0));
            }
        }
    }
    var out: VertexOutput;
    out.position = vec4<f32>(pixel / instance.screen_scale_progress.xy * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    out.instance = index;
    out.edge = edge;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let instance = instances[input.instance];
    var color = instance.color;
    if (instance.geometry.w != 0u) {
        let distance = dot(input.position.xy - input.edge.xy, input.edge.zw);
        color.a *= clamp(distance + 0.5, 0.0, 1.0);
    }
    return color;
}
