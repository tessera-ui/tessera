struct Uniforms {
    raster_rect: vec4<f32>,
    rect: vec4<f32>,
    viewport_scene: vec4<f32>,
    color: vec4<f32>,
    corner: vec4<f32>,
    stroke_color: vec4<f32>,
    stroke_start_joint: vec4<f32>,
    stroke_end_width: vec4<f32>,
    outline: vec4<f32>,
    path_points: array<vec4<f32>, 7>,
    path_params: vec4<f32>,
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
    let window = uniforms.raster_rect.xy + corners[index] * uniforms.raster_rect.zw;
    let ndc = window / uniforms.viewport_scene.xy * 2.0 - 1.0;
    return vec4(ndc.x, -ndc.y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let half_size = uniforms.rect.zw * 0.5;
    let radii = uniforms.corner.xy;
    let point = abs(position.xy - uniforms.rect.xy - half_size) - half_size;
    let q = point + radii;
    var distance = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radii.x;
    if radii.x != radii.y {
        distance = max(point.x, point.y);
        if all(q > vec2(0.0)) {
            let normalized = q / max(radii, vec2(0.000001));
            let gradient = q / max(radii * radii, vec2(0.000001));
            distance = length(normalized) * (length(normalized) - 1.0) / max(length(gradient), 0.000001);
        }
    }
    var coverage = clamp(0.5 - distance / uniforms.corner.z, 0.0, 1.0);
    if uniforms.outline.x > 0.0 {
        let inner_half = max(half_size - uniforms.outline.x, vec2(0.0));
        let inner_radius = max(radii.x - uniforms.outline.x, 0.0);
        let inner_q = abs(position.xy - uniforms.rect.xy - half_size) - inner_half + inner_radius;
        let inner_distance = length(max(inner_q, vec2(0.0))) + min(max(inner_q.x, inner_q.y), 0.0) - inner_radius;
        let inner_coverage = clamp(0.5 - inner_distance / uniforms.corner.z, 0.0, 1.0);
        coverage = max(coverage - inner_coverage, 0.0);
    }
    if uniforms.corner.w >= 0.0 {
        let clip_half_size = uniforms.raster_rect.zw * 0.5;
        let clip_q = abs(position.xy - uniforms.raster_rect.xy - clip_half_size) - clip_half_size + uniforms.corner.w;
        let clip_distance = length(max(clip_q, vec2(0.0))) + min(max(clip_q.x, clip_q.y), 0.0) - uniforms.corner.w;
        coverage = min(coverage, clamp(0.5 - clip_distance / uniforms.corner.z, 0.0, 1.0));
    }
    let alpha = uniforms.color.a * coverage;
    let local = position.xy - uniforms.rect.xy;
    let start = uniforms.stroke_start_joint.xy * uniforms.rect.zw;
    let joint = uniforms.stroke_start_joint.zw * uniforms.rect.zw;
    let end = uniforms.stroke_end_width.xy * uniforms.rect.zw;
    let stroke_distance = min(segment_distance(local, start, joint), segment_distance(local, joint, end));
    var stroke_coverage = clamp(uniforms.stroke_end_width.z * 0.5 + 0.5 - stroke_distance, 0.0, 1.0);
    if uniforms.stroke_end_width.w > 0.5 {
        let grid = floor(local - vec2(0.5)) + vec2(0.5);
        let fraction = fract(local - vec2(0.5));
        stroke_coverage = mix(
            mix(mask_coverage(grid, start, joint, end), mask_coverage(grid + vec2(1.0, 0.0), start, joint, end), fraction.x),
            mix(mask_coverage(grid + vec2(0.0, 1.0), start, joint, end), mask_coverage(grid + vec2(1.0), start, joint, end), fraction.x),
            fraction.y,
        );
    }
    if uniforms.path_params.z > 0.5 {
        let grid = floor(local - vec2(0.5)) + vec2(0.5);
        let fraction = fract(local - vec2(0.5));
        stroke_coverage = mix(
            mix(path_coverage(grid), path_coverage(grid + vec2(1.0, 0.0)), fraction.x),
            mix(path_coverage(grid + vec2(0.0, 1.0)), path_coverage(grid + vec2(1.0)), fraction.x),
            fraction.y,
        );
    }
    let stroke_alpha = uniforms.stroke_color.a * stroke_coverage * coverage;
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

fn capsule_coverage(point: vec2<f32>, radius: f32, stretch: vec2<f32>) -> f32 {
    let distance = length(point);
    // Project a screen pixel onto the edge normal after stretching the cap.
    // A fixed ramp in capsule space loses coverage at oblique edges.
    let gradient = point / stretch;
    let footprint = (gradient.x + gradient.y) / max(distance, 0.000001);
    return clamp(0.5 - (distance - radius) / max(footprint, 0.000001), 0.0, 1.0);
}

@fragment
fn fs_sweep(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let local = position.xy - uniforms.rect.xy;
    let profile = uniforms.corner;
    let motion = uniforms.stroke_start_joint;
    let opacity = uniforms.stroke_end_width.x;
    let radius = profile.x * 0.5;
    let extent = uniforms.rect.z;
    let y = abs(local.y - uniforms.rect.w * 0.5);
    let cap = max(abs(local.x - extent * 0.5) - max(extent * 0.5 - radius, 0.0), 0.0);
    let coverage = capsule_coverage(vec2(cap, y), radius, vec2(1.0));
    if coverage <= 0.0 { discard; }

    let x = select(local.x, extent - local.x, motion.w > 0.5) - profile.y;
    let tip = max((abs(x - profile.z * 0.5) - profile.z * 0.5) / motion.x + radius, 0.0);
    let highlight = min(coverage, capsule_coverage(vec2(tip, y / motion.y), radius, motion.xy));
    let gradient = mix(motion.z, 1.0, clamp(x / max(0.6 * profile.z, 0.000001), 0.0, 1.0));
    let bottom = clamp(local.y - uniforms.rect.w * 0.5 - radius + 1.5, 0.0, 1.0);
    let track_alpha = (uniforms.color.a + profile.w * bottom) * coverage;
    let accent_alpha = uniforms.stroke_color.a * highlight * gradient;

    let background = textureLoad(backdrop, vec2<i32>(floor(position.xy - uniforms.viewport_scene.zw)), 0);
    let premultiplied = encode(background.rgb / max(background.a, 0.000001)) * background.a;
    let track = mix(premultiplied, encode(uniforms.color.rgb), track_alpha);
    let track_coverage = mix(background.a, 1.0, track_alpha);
    let foreground = mix(track, encode(uniforms.stroke_color.rgb), accent_alpha);
    let output_alpha = mix(background.a, mix(track_coverage, 1.0, accent_alpha), opacity);
    let straight = mix(premultiplied, foreground, opacity) / max(output_alpha, 0.000001);
    return vec4(decode(straight) * output_alpha, output_alpha);
}

@fragment
fn fs_spokes(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let diagonal = inverseSqrt(2.0);
    let directions = array<vec2<f32>, 8>(
        vec2(0.0, -1.0), vec2(diagonal, -diagonal), vec2(1.0, 0.0), vec2(diagonal, diagonal),
        vec2(0.0, 1.0), vec2(-diagonal, diagonal), vec2(-1.0, 0.0), vec2(-diagonal, -diagonal),
    );
    let point = position.xy - uniforms.rect.xy - uniforms.rect.zw * 0.5;
    let profile = uniforms.corner;
    let weights = array<vec4<f32>, 2>(uniforms.stroke_color, uniforms.stroke_start_joint);
    var transmission = 1.0;
    for (var i = 0u; i < 8u; i++) {
        let axis = directions[i];
        let distance = segment_distance(point, axis * profile.x, axis * profile.y) - profile.z;
        let coverage = clamp(0.5 - distance / profile.w, 0.0, 1.0);
        transmission *= 1.0 - coverage * weights[i / 4u][i % 4u];
    }
    let alpha = uniforms.color.a * (1.0 - transmission);
    if alpha <= 0.0 { discard; }
    let background = textureLoad(backdrop, vec2<i32>(floor(position.xy - uniforms.viewport_scene.zw)), 0);
    let straight = encode(background.rgb / max(background.a, 0.000001));
    let remaining = background.a * (1.0 - alpha);
    let output_alpha = remaining + alpha;
    let color = (encode(uniforms.color.rgb) * alpha + straight * remaining) / output_alpha;
    return vec4(decode(color) * output_alpha, output_alpha);
}

fn dial_coverage(point: vec2<f32>) -> f32 {
    let size = uniforms.rect.zw;
    let shape = uniforms.corner;
    var distance = 0.0;
    if shape.w > 1.5 {
        let half = size * 0.5;
        let radius = min(half.x, half.y);
        let q = abs(point) - half + vec2(radius);
        distance = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
    } else {
        let stroke = shape.x;
        let radius = min(size.x, size.y) * 0.5 - stroke * 0.5;
        distance = abs(length(point) - radius) - stroke * 0.5;
        if shape.w < 0.5 {
            var angle = degrees(atan2(point.y, point.x)) - shape.y;
            angle = angle - floor(angle / 360.0) * 360.0;
            if angle > shape.z {
                let start = radians(shape.y);
                let end = radians(shape.y + shape.z);
                let start_cap = length(point - vec2(cos(start), sin(start)) * radius) - stroke * 0.5;
                let end_cap = length(point - vec2(cos(end), sin(end)) * radius) - stroke * 0.5;
                distance = min(start_cap, end_cap);
            }
        }
    }
    var coverage = clamp(0.5 - distance, 0.0, 1.0);
    let marker = uniforms.stroke_color;
    if marker.z > 0.0 {
        let from_marker = length(point - marker.xy);
        coverage = coverage * clamp(from_marker - marker.z + 0.5, 0.0, 1.0);
        if marker.w > 0.0 {
            coverage = max(coverage, clamp(0.5 - (from_marker - marker.w), 0.0, 1.0));
        }
    }
    return coverage;
}

@fragment
fn fs_dial(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let point = position.xy - uniforms.rect.xy - uniforms.rect.zw * 0.5;
    let alpha = uniforms.color.a * dial_coverage(point);
    if alpha <= 0.0 { discard; }
    let background = textureLoad(backdrop, vec2<i32>(floor(position.xy - uniforms.viewport_scene.zw)), 0);
    let straight = encode(background.rgb / max(background.a, 0.000001));
    let remaining = background.a * (1.0 - alpha);
    let output_alpha = remaining + alpha;
    let color = (encode(uniforms.color.rgb) * alpha + straight * remaining) / output_alpha;
    return vec4(decode(color) * output_alpha, output_alpha);
}

fn mask_coverage(point: vec2<f32>, start: vec2<f32>, joint: vec2<f32>, end: vec2<f32>) -> f32 {
    let distance = min(segment_distance(point, start, joint), segment_distance(point, joint, end));
    return clamp(uniforms.stroke_end_width.z * 0.5 + 0.5 - distance, 0.0, 1.0);
}

fn path_point(index: u32) -> vec2<f32> {
    let pair = uniforms.path_points[index / 2u];
    return select(pair.xy, pair.zw, index % 2u == 1u) * uniforms.rect.zw;
}

fn path_coverage(point: vec2<f32>) -> f32 {
    var distance = length(point - uniforms.path_points[6].zw * uniforms.rect.zw) - uniforms.path_params.y;
    for (var curve = 0u; curve < 4u; curve++) {
        let a = path_point(curve * 3u);
        let b = path_point(curve * 3u + 1u);
        let c = path_point(curve * 3u + 2u);
        let d = path_point(curve * 3u + 3u);
        var previous = a;
        for (var step = 1u; step <= 24u; step++) {
            let t = f32(step) / 24.0;
            let u = 1.0 - t;
            let next = u*u*u*a + 3.0*u*u*t*b + 3.0*u*t*t*c + t*t*t*d;
            distance = min(distance, segment_distance(point, previous, next) - uniforms.path_params.x * 0.5);
            previous = next;
        }
    }
    return clamp(0.5 - distance, 0.0, 1.0);
}
