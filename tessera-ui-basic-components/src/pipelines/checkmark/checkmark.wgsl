struct CheckmarkUniforms {
    size: vec2f,           // width, height of the checkmark area
    color: vec4f,          // RGBA color of the checkmark
    stroke_width: f32,     // thickness of the checkmark lines
    progress: f32,         // animation progress (0.0 to 1.0)
    padding: vec2f,        // padding around the checkmark
};

@group(0) @binding(0)
var<uniform> checkmark_params: CheckmarkUniforms;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4f(model.position.xy, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

// Distance from point to line segment
fn distance_to_line_segment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
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
    let size = checkmark_params.size;
    let color = checkmark_params.color;
    let stroke_width = checkmark_params.stroke_width;
    let progress = checkmark_params.progress;
    let padding = checkmark_params.padding;
    
    // Convert UV coordinates to pixel coordinates within the padded area
    let padded_size = size - 2.0 * padding;
    let pixel_pos = (vec2f(in.uv.x, 1.0 - in.uv.y) - 0.5) * size;
    let local_pos = pixel_pos;
    
    // Define checkmark path points (normalized to padded area)
    let scale = min(padded_size.x, padded_size.y);
    let center_offset = (padded_size - scale) * 0.5;
    
    // Checkmark path: two line segments forming a check
    // First line: from bottom-left to middle-bottom
    let line1_start = vec2f(-0.35, -0.1) * scale + center_offset;
    let line1_end = vec2f(-0.05, -0.35) * scale + center_offset;
    
    // Second line: from middle-bottom to top-right  
    let line2_start = line1_end;
    let line2_end = vec2f(0.4, 0.25) * scale + center_offset;
    
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
