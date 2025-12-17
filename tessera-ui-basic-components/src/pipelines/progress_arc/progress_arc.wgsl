struct ArcUniform {
    position: vec4<f32>,
    color: vec4<f32>,
    screen_size: vec2<f32>,
    stroke_width: f32,
    start_angle_degrees: f32,
    sweep_angle_degrees: f32,
    cap: u32,
    _pad: u32,
}

struct ArcInstances {
    instances: array<ArcUniform>,
}

@group(0) @binding(0)
var<storage, read> uniforms: ArcInstances;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) instance_index: u32,
}

fn normalize_angle_degrees(angle: f32) -> f32 {
    var a = angle % 360.0;
    if (a < 0.0) {
        a = a + 360.0;
    }
    return a;
}

fn in_sweep(angle: f32, start: f32, sweep: f32) -> bool {
    if (sweep >= 360.0) {
        return true;
    }
    let end = start + sweep;
    if (end <= 360.0) {
        return angle >= start && angle <= end;
    }
    return angle >= start || angle <= (end - 360.0);
}

fn smooth_mask(distance_to_edge: f32, aa: f32) -> f32 {
    return 1.0 - smoothstep(0.0, aa, distance_to_edge);
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
    out.uv = in.position;
    out.instance_index = in.instance_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uniform = uniforms.instances[in.instance_index];

    let size = uniform.position.zw;
    if (size.x <= 0.0 || size.y <= 0.0) {
        return vec4<f32>(0.0);
    }

    let pixel = uniform.position.xy + in.uv * size;
    let center = uniform.position.xy + size * 0.5;
    let p = pixel - center;

    let stroke = max(uniform.stroke_width, 0.0);
    let radius = (min(size.x, size.y) * 0.5) - (stroke * 0.5);
    if (radius <= 0.0 || stroke <= 0.0) {
        return vec4<f32>(0.0);
    }

    let d = length(p);
    let ring_edge = abs(d - radius) - (stroke * 0.5);

    var angle = degrees(atan2(p.y, p.x));
    if (angle < 0.0) {
        angle = angle + 360.0;
    }

    let start = normalize_angle_degrees(uniform.start_angle_degrees);
    let sweep = clamp(uniform.sweep_angle_degrees, 0.0, 360.0);
    if (sweep <= 0.0) {
        return vec4<f32>(0.0);
    }

    let aa = 1.0;

    var mask = 0.0;
    if (in_sweep(angle, start, sweep)) {
        mask = smooth_mask(ring_edge, aa);
    }

    if (uniform.cap == 1u && sweep < 360.0) {
        let end = start + sweep;
        let start_rad = radians(start);
        let end_rad = radians(end);

        let start_pos = vec2<f32>(cos(start_rad), sin(start_rad)) * radius;
        let end_pos = vec2<f32>(cos(end_rad), sin(end_rad)) * radius;

        let cap_radius = stroke * 0.5;
        let start_cap = length(p - start_pos) - cap_radius;
        let end_cap = length(p - end_pos) - cap_radius;

        mask = max(mask, smooth_mask(start_cap, aa));
        mask = max(mask, smooth_mask(end_cap, aa));
    }

    if (mask <= 0.0) {
        return vec4<f32>(0.0);
    }

    return vec4<f32>(uniform.color.rgb, uniform.color.a * mask);
}

