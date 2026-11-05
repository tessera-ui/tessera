fn sdf_g2_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>, k: vec4<f32>) -> f32 {
    let top_radii = select(r.x, r.y, p.x > 0.0);
    let bottom_radii = select(r.w, r.z, p.x > 0.0);
    let r_for_quadrant = select(top_radii, bottom_radii, p.y > 0.0);
    let top_k = select(k.x, k.y, p.x > 0.0);
    let bottom_k = select(k.w, k.z, p.x > 0.0);
    let k_for_quadrant = select(top_k, bottom_k, p.y > 0.0);

    let q = abs(p) - b + r_for_quadrant;
    let v = max(q, vec2<f32>(0.0));

    if abs(k_for_quadrant - 2.0) < 0.001 {
        return length(v) + min(max(q.x, q.y), 0.0) - r_for_quadrant;
    }

    let dist_corner_shape = pow(pow(v.x, k_for_quadrant) + pow(v.y, k_for_quadrant), 1.0 / k_for_quadrant);

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

fn grad_sd_g2_rounded_box(coord: vec2<f32>, half_size: vec2<f32>, r: vec4<f32>, k: vec4<f32>) -> vec2<f32> {
    let top_radii = select(r.x, r.y, coord.x > 0.0);
    let bottom_radii = select(r.w, r.z, coord.x > 0.0);
    let r_for_quadrant = select(top_radii, bottom_radii, coord.y > 0.0);
    let top_k = select(k.x, k.y, coord.x > 0.0);
    let bottom_k = select(k.w, k.z, coord.x > 0.0);
    let k_for_quadrant = select(top_k, bottom_k, coord.y > 0.0);
    let inner_half_size = half_size - r_for_quadrant;
    let corner_coord = abs(coord) - inner_half_size;

    if corner_coord.x >= 0.0 && corner_coord.y >= 0.0 {
        let grad_dir = vec2<f32>(
            pow(corner_coord.x + 0.0001, k_for_quadrant - 1.0),
            pow(corner_coord.y + 0.0001, k_for_quadrant - 1.0)
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

fn signed_one(value: f32) -> f32 {
    return select(-1.0, 1.0, value >= 0.0);
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

fn shape_sd_normal(shape_type: f32, coord: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>, k: vec4<f32>) -> vec3<f32> {
    if shape_type == 1.0 {
        return vec3<f32>(sdf_ellipse(coord, half_size), grad_sd_ellipse(coord, half_size));
    }
    if shape_type == 2.0 {
        return vec3<f32>(sdf_axis_aligned_box(coord, half_size), grad_sd_axis_aligned_box(coord, half_size));
    }
    return vec3<f32>(sdf_g2_rounded_box(coord, half_size, radii, k), grad_sd_g2_rounded_box(coord, half_size, radii, k));
}
