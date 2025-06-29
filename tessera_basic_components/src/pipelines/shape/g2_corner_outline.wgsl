// Struct for shader inputs
struct Vertex {
    @location(0) position: vec2<f32>,
};

// Uniforms
@group(0) @binding(0) var<uniform> size: vec2<f32>;
@group(0) @binding(1) var<uniform> corner_radius: f32;
@group(0) @binding(2) var<uniform> border_width: f32;
@group(0) @binding(3) var<uniform> segments_per_corner: u32;

// Output buffer
@group(0) @binding(4) var<storage, read_write> output: array<Vertex>;

// Function to evaluate a point on a G2 continuous curve for a corner
fn g2_arc(t: f32) -> vec2<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;

    let a = 1.0; 
    let b = 0.8;

    let x = a * (1.0 - t5);
    let y = b * t - (b - 1.0) * t5;
    return vec2<f32>(x, y);
}

// Function to calculate the derivative (tangent) of the G2 curve
fn g2_arc_derivative(t: f32) -> vec2<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let a = 1.0;
    let b = 0.8;

    let dx = -5.0 * a * t4;
    let dy = b - 5.0 * (b - 1.0) * t4;
    return normalize(vec2<f32>(dx, dy));
}


@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let half_size = size / 2.0;
    let clamped_radius = min(corner_radius, min(half_size.x, half_size.y));
    let half_border = border_width / 2.0;

    let total_segments = segments_per_corner * 4u;
    if (global_id.x >= total_segments) {
        return;
    }

    let segment_index = global_id.x;
    let corner_index = segment_index / segments_per_corner;
    let t_raw = f32(segment_index % segments_per_corner) / f32(segments_per_corner - 1u);
    
    // Evaluate curve and tangent
    let p = g2_arc(t_raw);
    let tangent = g2_arc_derivative(t_raw);
    let normal = vec2<f32>(-tangent.y, tangent.x);

    // Calculate inner and outer points for the strip
    let arc_point = (1.0 - p) * clamped_radius;
    let outer_v = arc_point + normal * half_border;
    let inner_v = arc_point - normal * half_border;
    
    // Determine corner center and rotation
    var center: vec2<f32>;
    var rotation: mat2x2<f32>;

    if (corner_index == 0u) { // Top-right
        center = vec2<f32>(half_size.x - clamped_radius, half_size.y - clamped_radius);
        rotation = mat2x2<f32>(0.0, -1.0, 1.0, 0.0);
    } else if (corner_index == 1u) { // Top-left
        center = vec2<f32>(-half_size.x + clamped_radius, half_size.y - clamped_radius);
        rotation = mat2x2<f32>(-1.0, 0.0, 0.0, -1.0);
    } else if (corner_index == 2u) { // Bottom-left
        center = vec2<f32>(-half_size.x + clamped_radius, -half_size.y + clamped_radius);
        rotation = mat2x2<f32>(0.0, 1.0, -1.0, 0.0);
    } else { // Bottom-right
        center = vec2<f32>(half_size.x - clamped_radius, -half_size.y + clamped_radius);
        rotation = mat2x2<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let final_outer_pos = center + rotation * outer_v;
    let final_inner_pos = center + rotation * inner_v;

    // Write vertices to output buffer as a triangle strip
    // Each invocation writes two vertices
    let output_index = segment_index * 2u;
    output[output_index] = Vertex(final_outer_pos);
    output[output_index + 1u] = Vertex(final_inner_pos);
}