struct SdfUniforms {
    size: vec2<f32>,
    corner_radii: vec4<f32>,
    shape_type: f32,
    g2_k_value: f32,
}

@group(0) @binding(0) var<uniform> uniforms: SdfUniforms;
@group(0) @binding(1) var sdf_texture: texture_storage_2d<rgba16float, write>;

fn sdf_g2_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>, k: f32) -> f32 {
    let top_radii = select(r.x, r.y, p.x > 0.0);
    let bottom_radii = select(r.w, r.z, p.x > 0.0);
    let r_for_quadrant = select(top_radii, bottom_radii, p.y > 0.0);

    let q = abs(p) - b + r_for_quadrant;
    let v = max(q, vec2<f32>(0.0));

    if abs(k - 2.0) < 0.001 {
        return length(v) + min(max(q.x, q.y), 0.0) - r_for_quadrant;
    }

    let dist_corner_shape = pow(pow(v.x, k) + pow(v.y, k), 1.0 / k);
    return dist_corner_shape + min(max(q.x, q.y), 0.0) - r_for_quadrant;
}

fn sdf_ellipse(p: vec2<f32>, r: vec2<f32>) -> f32 {
    if r.x <= 0.0 || r.y <= 0.0 {
        return 1.0e6;
    }
    return (length(p / r) - 1.0) * min(r.x, r.y);
}

fn sdf_axis_aligned_box(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let dist = abs(p) - half_size;
    let outside = max(dist, vec2<f32>(0.0, 0.0));
    return length(outside) + min(max(dist.x, dist.y), 0.0);
}

fn signed_one(value: f32) -> f32 {
    return select(-1.0, 1.0, value >= 0.0);
}

fn grad_sd_g2_rounded_box(coord: vec2<f32>, half_size: vec2<f32>, r: vec4<f32>, k: f32) -> vec2<f32> {
    let top_radii = select(r.x, r.y, coord.x > 0.0);
    let bottom_radii = select(r.w, r.z, coord.x > 0.0);
    let r_for_quadrant = select(top_radii, bottom_radii, coord.y > 0.0);
    let inner_half_size = half_size - r_for_quadrant;
    let corner_coord = abs(coord) - inner_half_size;

    if corner_coord.x >= 0.0 && corner_coord.y >= 0.0 {
        let grad_dir = vec2<f32>(
            pow(corner_coord.x + 0.0001, k - 1.0),
            pow(corner_coord.y + 0.0001, k - 1.0)
        );
        return sign(coord) * normalize(grad_dir);
    } else {
        if corner_coord.x > corner_coord.y {
            return sign(coord) * vec2<f32>(1.0, 0.0);
        } else {
            return sign(coord) * vec2<f32>(0.0, 1.0);
        }
    }
}

fn grad_sd_ellipse(coord: vec2<f32>, r: vec2<f32>) -> vec2<f32> {
    return normalize(coord / (r * r));
}

fn grad_sd_axis_aligned_box(coord: vec2<f32>, half_size: vec2<f32>) -> vec2<f32> {
    let dist = abs(coord) - half_size;
    if dist.x > 0.0 || dist.y > 0.0 {
        let outside = max(dist, vec2<f32>(0.0, 0.0));
        return sign(coord) * normalize(outside);
    }

    let dist_to_edge = half_size - abs(coord);
    if dist_to_edge.x < dist_to_edge.y {
        return vec2<f32>(signed_one(coord.x), 0.0);
    } else {
        return vec2<f32>(0.0, signed_one(coord.y));
    }
}

fn compute_sd_and_normal(coord: vec2<f32>, half_size: vec2<f32>, uniforms: SdfUniforms) -> vec3<f32> {
    if uniforms.shape_type == 1.0 {
        let sd = sdf_ellipse(coord, half_size);
        let normal = grad_sd_ellipse(coord, half_size);
        return vec3<f32>(sd, normal);
    }
    if uniforms.shape_type == 2.0 {
        let sd = sdf_axis_aligned_box(coord, half_size);
        let normal = grad_sd_axis_aligned_box(coord, half_size);
        return vec3<f32>(sd, normal);
    }
    let sd = sdf_g2_rounded_box(coord, half_size, uniforms.corner_radii, uniforms.g2_k_value);
    let normal = grad_sd_g2_rounded_box(coord, half_size, uniforms.corner_radii, uniforms.g2_k_value);
    return vec3<f32>(sd, normal);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(sdf_texture);
    if global_id.x >= dims.x || global_id.y >= dims.y {
        return;
    }

    let size = uniforms.size;
    let coord = vec2<f32>(vec2<u32>(global_id.xy)) + vec2<f32>(0.5, 0.5);
    let half_size = size * 0.5;
    let centered = coord - half_size;

    let result = compute_sd_and_normal(centered, half_size, uniforms);
    let normal = normalize(result.yz);
    textureStore(sdf_texture, global_id.xy, vec4<f32>(result.x, normal.x, normal.y, 0.0));
}
