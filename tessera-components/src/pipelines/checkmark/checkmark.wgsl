struct CheckmarkInstance {
    position: vec4f,
    color: vec4f,
    parameters: vec4f,
    padding_screen: vec4f,
};

@group(0) @binding(0)
var<storage, read> instances: array<CheckmarkInstance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) @interpolate(flat) instance: u32,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> VertexOutput {
    let quad = array<vec2f, 6>(vec2f(0.0, 1.0), vec2f(1.0, 1.0), vec2f(1.0, 0.0), vec2f(0.0, 1.0), vec2f(1.0, 0.0), vec2f(0.0, 0.0));
    let data = instances[index];
    let uv = quad[vertex];
    let pixel = data.position.xy + uv * data.position.zw;
    var out: VertexOutput;
    out.clip_position = vec4f(pixel.x / data.padding_screen.z * 2.0 - 1.0, 1.0 - pixel.y / data.padding_screen.w * 2.0, 0.0, 1.0);
    out.uv = uv;
    out.instance = index;
    return out;
}

// Distance from point to line segment
fn distance_to_line_segment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn square_stroke_distance(p: vec2f, a: vec2f, b: vec2f, width: f32) -> f32 {
    let segment = b - a;
    let extent = length(segment);
    var axis = vec2f(1.0, 0.0);
    if (extent > 0.0) {
        axis = segment / extent;
    }
    let delta = p - (a + b) * 0.5;
    let q = abs(vec2f(dot(delta, axis), dot(delta, vec2f(-axis.y, axis.x))))
        - vec2f(extent * 0.5 + width * 0.5, width * 0.5);
    return length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0);
}

// Calculate the total length along the checkmark path
fn get_path_length_at_point(p: vec2f, line1_start: vec2f, line1_end: vec2f, line2_start: vec2f, line2_end: vec2f) -> f32 {
    let line1_length = length(line1_end - line1_start);
    let line2_length = length(line2_end - line2_start);
    let total_length = line1_length + line2_length;
    
    // Check which line segment the point is closest to
    let dist1 = distance_to_line_segment(p, line1_start, line1_end);
    let dist2 = distance_to_line_segment(p, line2_start, line2_end);
    
    if (dist1 < dist2) {
        // Point is on first line segment
        let pa = p - line1_start;
        let ba = line1_end - line1_start;
        let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        return h * line1_length;
    } else {
        // Point is on second line segment
        let pa = p - line2_start;
        let ba = line2_end - line2_start;
        let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        return line1_length + h * line2_length;
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let data = instances[in.instance];
    let size = data.position.zw;
    let color = data.color;
    let stroke_width = data.parameters.x;
    let progress = data.parameters.y;
    let padding = data.padding_screen.xy;

    if (data.parameters.w > 0.5) {
        if (progress <= 0.0) {
            return vec4f(0.0);
        }
        let morph = data.parameters.z;
        let p = in.uv * size;
        let start = vec2f(0.25, 0.5) * size;
        let joint = mix(vec2f(0.4, 0.65), vec2f(0.5, 0.5), morph) * size;
        let end = mix(vec2f(0.75, 0.3), vec2f(0.75, 0.5), morph) * size;
        let first_length = length(joint - start);
        let second_length = length(end - joint);
        let drawn_length = progress * (first_length + second_length);
        let first_end = mix(start, joint, min(drawn_length / first_length, 1.0));
        var distance = square_stroke_distance(p, start, first_end, stroke_width);
        if (drawn_length > first_length) {
            let second_end = mix(joint, end, (drawn_length - first_length) / second_length);
            distance = min(distance, square_stroke_distance(p, joint, second_end, stroke_width));
        }
        return vec4f(color.rgb, color.a * (1.0 - smoothstep(-0.5, 0.5, distance)));
    }
    
    // Convert UV coordinates to pixel coordinates within the padded area
    let padded_size = size - 2.0 * padding;
    let pixel_pos = (vec2f(in.uv.x, 1.0 - in.uv.y) - 0.5) * size;
    let local_pos = pixel_pos;
    
    // Define checkmark path points (normalized to padded area)
    let scale = min(padded_size.x, padded_size.y);
    let center_offset = (padded_size - scale) * 0.5;
    
    // Checkmark path: two line segments forming a check
    // First line: from bottom-left to middle-bottom
    let morph = data.parameters.z;
    let line1_start = mix(vec2f(-0.35, -0.1), vec2f(-0.375, 0.0), morph) * scale + center_offset;
    let line1_end = mix(vec2f(-0.05, -0.35), vec2f(0.0, 0.0), morph) * scale + center_offset;
    
    // Second line: from middle-bottom to top-right  
    let line2_start = line1_end;
    let line2_end = mix(vec2f(0.4, 0.25), vec2f(0.375, 0.0), morph) * scale + center_offset;
    
    // Calculate total path length
    let line1_length = length(line1_end - line1_start);
    let line2_length = length(line2_end - line2_start);
    let total_length = line1_length + line2_length;
    let current_length = progress * total_length;
    
    // Calculate distance to the checkmark path
    let dist1 = distance_to_line_segment(local_pos, line1_start, line1_end);
    let dist2 = distance_to_line_segment(local_pos, line2_start, line2_end);
    let min_dist = min(dist1, dist2);
    
    // Calculate position along the path
    let path_pos = get_path_length_at_point(local_pos, line1_start, line1_end, line2_start, line2_end);
    
    // Only draw if we're within the current progress
    let should_draw = path_pos <= current_length;
    
    // Anti-aliased stroke
    let half_stroke = stroke_width * 0.5;
    let alpha = 1.0 - smoothstep(half_stroke - 1.0, half_stroke + 1.0, min_dist);
    
    // Apply progress masking
    let final_alpha = select(0.0, alpha, should_draw) * color.a;
    
    return vec4f(color.rgb, final_alpha);
}
