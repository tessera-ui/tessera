struct Instance {
    bounds: vec4f,
    radii: vec4f,
    exponents: vec4f,
    viewport: vec2f,
    ellipse: u32,
    padding: u32,
}

@group(0) @binding(0) var<storage, read> masks: array<Instance>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) @interpolate(flat) index: u32,
    @location(1) @interpolate(linear, sample) local: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> VertexOutput {
    let corners = array<vec2f, 6>(vec2f(0, 0), vec2f(1, 0), vec2f(0, 1), vec2f(0, 1), vec2f(1, 0), vec2f(1, 1));
    let mask = masks[index];
    let pixel = mask.bounds.xy + corners[vertex] * mask.bounds.zw;
    var result: VertexOutput;
    result.position = vec4f(pixel / mask.viewport * vec2f(2, -2) + vec2f(-1, 1), 0, 1);
    result.index = index;
    result.local = corners[vertex] * mask.bounds.zw;
    return result;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let mask = masks[in.index];
    let half_size = mask.bounds.zw * 0.5;
    let point = in.local - half_size;
    if mask.ellipse != 0u {
        let p = point / max(half_size, vec2f(0.0001));
        if dot(p, p) > 1.0 { discard; }
    } else {
        var corner = select(2u, 1u, point.y < 0.0);
        if point.x < 0.0 { corner = select(3u, 0u, point.y < 0.0); }
        let radius = mask.radii[corner];
        if radius > 0.0 {
            let q = max(abs(point) - half_size + radius, vec2f(0)) / radius;
            let exponent = mask.exponents[corner];
            if pow(q.x, exponent) + pow(q.y, exponent) > 1.0 { discard; }
        }
    }
    return vec4f(0);
}
