struct Instance {
    position: vec4<f32>,
    color: vec4<f32>,
    curve: vec4<f32>,
    wave: vec4<f32>,
    screen: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> instances: array<Instance>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) instance: u32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let instance = instances[index];
    let radius = instance.curve.z * 0.5;
    let vertical = abs(instance.wave.x) * 0.5 + radius + 1.0;
    let low = max(vec2(0.0), vec2(instance.curve.x - radius - 1.0, instance.position.w * 0.5 - vertical));
    let high = min(instance.position.zw, vec2(instance.curve.y + radius + 1.0, instance.position.w * 0.5 + vertical));
    var local = mix(low, high, corners[vertex]);
    if instance.wave.w != 0.0 {
        local = instance.position.zw - local;
    }
    let point = instance.position.xy + local;
    var output: VertexOutput;
    output.position = vec4(point / instance.screen.xy * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    output.instance = index;
    return output;
}

fn curve_point(t: f32, length: f32, control: f32) -> vec2<f32> {
    return vec2(length * t, 2.0 * control * t * (1.0 - t));
}

fn squared_distance(t: f32, point: vec2<f32>, length: f32, control: f32) -> f32 {
    let delta = curve_point(t, length, control) - point;
    return dot(delta, delta);
}

fn cube_root(value: f32) -> f32 {
    return sign(value) * pow(abs(value), 1.0 / 3.0);
}

fn quadratic_distance(point: vec2<f32>, length: f32, control: f32, low: f32, high: f32) -> f32 {
    let a = vec2(0.0, -2.0 * control);
    let b = vec2(length, 2.0 * control);
    let denominator = 2.0 * dot(a, a);
    let c2 = 3.0 * dot(a, b) / denominator;
    let c1 = (dot(b, b) - 2.0 * dot(a, point)) / denominator;
    let c0 = -dot(b, point) / denominator;
    let p = c1 - c2 * c2 / 3.0;
    let q = 2.0 * c2 * c2 * c2 / 27.0 - c2 * c1 / 3.0 + c0;
    let discriminant = q * q / 4.0 + p * p * p / 27.0;
    var distance = min(squared_distance(low, point, length, control), squared_distance(high, point, length, control));
    if discriminant >= 0.0 {
        let root = sqrt(discriminant);
        let t = cube_root(-q * 0.5 + root) + cube_root(-q * 0.5 - root) - c2 / 3.0;
        distance = min(distance, squared_distance(clamp(t, low, high), point, length, control));
    } else {
        let radius = 2.0 * sqrt(-p / 3.0);
        let theta = acos(clamp(3.0 * q / (2.0 * p) * sqrt(-3.0 / p), -1.0, 1.0)) / 3.0;
        for (var i = 0u; i < 3u; i += 1u) {
            let t = radius * cos(theta - f32(i) * 2.09439510239) - c2 / 3.0;
            distance = min(distance, squared_distance(clamp(t, low, high), point, length, control));
        }
    }
    return sqrt(max(distance, 0.0));
}

fn endpoint(instance: Instance, x: f32) -> vec4<f32> {
    let half_wave = instance.curve.w * 0.5;
    let cell = floor((x + instance.wave.y) / half_wave);
    let control = select(instance.wave.x, -instance.wave.x, i32(cell) % 2 != 0);
    let t = (x + instance.wave.y - cell * half_wave) / half_wave;
    return vec4(x, 2.0 * control * t * (1.0 - t), normalize(vec2(half_wave, 2.0 * control * (1.0 - 2.0 * t))));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let instance = instances[input.instance];
    var point = input.position.xy - instance.position.xy;
    if instance.wave.w != 0.0 {
        point = instance.position.zw - point;
    }
    point.y -= instance.position.w * 0.5;
    let start = instance.curve.x;
    let end = instance.curve.y;
    let radius = instance.curve.z * 0.5;
    var distance = 1000000.0;
    if abs(instance.wave.x) < 0.001 || instance.curve.w <= 0.000001 {
        if instance.wave.z != 0.0 {
            distance = length(point - vec2(clamp(point.x, start, end), 0.0)) - radius;
        } else {
            let delta = abs(point - vec2((start + end) * 0.5, 0.0)) - vec2((end - start) * 0.5, radius);
            distance = length(max(delta, vec2(0.0))) + min(max(delta.x, delta.y), 0.0);
        }
    } else {
        let half_wave = instance.curve.w * 0.5;
        let cell = i32(floor((clamp(point.x, start, end) + instance.wave.y) / half_wave));
        for (var offset = -1; offset <= 1; offset += 1) {
            let index = cell + offset;
            let origin = f32(index) * half_wave - instance.wave.y;
            let low = max(0.0, (start - origin) / half_wave);
            let high = min(1.0, (end - origin) / half_wave);
            if high >= low {
                let control = select(instance.wave.x, -instance.wave.x, index % 2 != 0);
                distance = min(distance, quadratic_distance(point - vec2(origin, 0.0), half_wave, control, low, high) - radius);
            }
        }
        if instance.wave.z == 0.0 {
            let first = endpoint(instance, start);
            let last = endpoint(instance, end);
            distance = max(distance, max(-dot(point - first.xy, first.zw), dot(point - last.xy, last.zw)));
        }
    }
    return vec4(instance.color.rgb, instance.color.a * clamp(0.5 - distance, 0.0, 1.0));
}
