struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) coord: vec2<f32>,
};

struct Uniforms {
    rect: vec4<f32>,
    mouse: vec2<f32>,
    time: f32,
    opacity: f32,
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
    camera_position: vec4<f32>,
    camera_forward: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
};

struct Hit {
    t: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    albedo: vec3<f32>,
    emission: vec3<f32>,
    kind: i32,
};

const SURFACE_BIAS: f32 = 0.001;
const MAX_REFLECTION_BOUNCES: i32 = 3;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn miss_hit() -> Hit {
    return Hit(1e9, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0), 0);
}

fn intersect_sphere(ray_origin: vec3<f32>, ray_direction: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    let oc = ray_origin - center;
    let b = dot(oc, ray_direction);
    let c = dot(oc, oc) - radius * radius;
    let h = b * b - c;

    if h < 0.0 {
        return -1.0;
    }

    let s = sqrt(h);
    let t_near = -b - s;
    if t_near > SURFACE_BIAS {
        return t_near;
    }

    let t_far = -b + s;
    if t_far > SURFACE_BIAS {
        return t_far;
    }

    return -1.0;
}

fn intersect_plane(ray_origin: vec3<f32>, ray_direction: vec3<f32>, normal: vec3<f32>, distance: f32) -> f32 {
    let denom = dot(ray_direction, normal);
    if abs(denom) < 1e-4 {
        return -1.0;
    }

    let t = -(dot(ray_origin, normal) + distance) / denom;
    if t > SURFACE_BIAS {
        return t;
    }

    return -1.0;
}

fn sign_nonzero(value: f32) -> f32 {
    return select(-1.0, 1.0, value >= 0.0);
}

fn intersect_box(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    center: vec3<f32>,
    half_size: vec3<f32>,
) -> f32 {
    let box_min = center - half_size;
    let box_max = center + half_size;

    var t_min = -1e30;
    var t_max = 1e30;

    if abs(ray_direction.x) < 1e-5 {
        if ray_origin.x < box_min.x || ray_origin.x > box_max.x {
            return -1.0;
        }
    } else {
        var t1 = (box_min.x - ray_origin.x) / ray_direction.x;
        var t2 = (box_max.x - ray_origin.x) / ray_direction.x;
        if t1 > t2 {
            let tmp = t1;
            t1 = t2;
            t2 = tmp;
        }
        t_min = max(t_min, t1);
        t_max = min(t_max, t2);
    }

    if abs(ray_direction.y) < 1e-5 {
        if ray_origin.y < box_min.y || ray_origin.y > box_max.y {
            return -1.0;
        }
    } else {
        var t1 = (box_min.y - ray_origin.y) / ray_direction.y;
        var t2 = (box_max.y - ray_origin.y) / ray_direction.y;
        if t1 > t2 {
            let tmp = t1;
            t1 = t2;
            t2 = tmp;
        }
        t_min = max(t_min, t1);
        t_max = min(t_max, t2);
    }

    if abs(ray_direction.z) < 1e-5 {
        if ray_origin.z < box_min.z || ray_origin.z > box_max.z {
            return -1.0;
        }
    } else {
        var t1 = (box_min.z - ray_origin.z) / ray_direction.z;
        var t2 = (box_max.z - ray_origin.z) / ray_direction.z;
        if t1 > t2 {
            let tmp = t1;
            t1 = t2;
            t2 = tmp;
        }
        t_min = max(t_min, t1);
        t_max = min(t_max, t2);
    }

    if t_max < max(t_min, SURFACE_BIAS) {
        return -1.0;
    }

    if t_min > SURFACE_BIAS {
        return t_min;
    }

    return t_max;
}

fn box_normal(point: vec3<f32>, center: vec3<f32>, half_size: vec3<f32>) -> vec3<f32> {
    let local = (point - center) / half_size;
    let abs_local = abs(local);

    if abs_local.x > abs_local.y && abs_local.x > abs_local.z {
        return vec3<f32>(sign_nonzero(local.x), 0.0, 0.0);
    }

    if abs_local.y > abs_local.z {
        return vec3<f32>(0.0, sign_nonzero(local.y), 0.0);
    }

    return vec3<f32>(0.0, 0.0, sign_nonzero(local.z));
}

fn intersect_cone(
    ray_origin: vec3<f32>,
    ray_direction: vec3<f32>,
    apex: vec3<f32>,
    height: f32,
    radius: f32,
) -> f32 {
    let slope = radius / height;
    let slope2 = slope * slope;

    let ro = vec3<f32>(ray_origin.x - apex.x, apex.y - ray_origin.y, ray_origin.z - apex.z);
    let rd = vec3<f32>(ray_direction.x, -ray_direction.y, ray_direction.z);

    let a = rd.x * rd.x + rd.z * rd.z - slope2 * rd.y * rd.y;
    let b = 2.0 * (ro.x * rd.x + ro.z * rd.z - slope2 * ro.y * rd.y);
    let c = ro.x * ro.x + ro.z * ro.z - slope2 * ro.y * ro.y;

    var best_t = -1.0;

    let disc = b * b - 4.0 * a * c;
    if disc >= 0.0 && abs(a) > 1e-6 {
        let s = sqrt(disc);
        let inv_2a = 0.5 / a;
        let t0 = (-b - s) * inv_2a;
        let t1 = (-b + s) * inv_2a;

        if t0 > SURFACE_BIAS {
            let y0 = ro.y + t0 * rd.y;
            if y0 >= 0.0 && y0 <= height {
                best_t = t0;
            }
        }

        if t1 > SURFACE_BIAS {
            let y1 = ro.y + t1 * rd.y;
            if y1 >= 0.0 && y1 <= height {
                if best_t < 0.0 || t1 < best_t {
                    best_t = t1;
                }
            }
        }
    }

    if abs(rd.y) > 1e-6 {
        let t_base = (height - ro.y) / rd.y;
        if t_base > SURFACE_BIAS {
            let p = ro + rd * t_base;
            if p.x * p.x + p.z * p.z <= radius * radius {
                if best_t < 0.0 || t_base < best_t {
                    best_t = t_base;
                }
            }
        }
    }

    return best_t;
}

fn cone_normal(point: vec3<f32>, apex: vec3<f32>, height: f32, radius: f32) -> vec3<f32> {
    let slope = radius / height;
    let slope2 = slope * slope;
    let local = vec3<f32>(point.x - apex.x, apex.y - point.y, point.z - apex.z);

    if abs(local.y - height) < 0.01 {
        return vec3<f32>(0.0, -1.0, 0.0);
    }

    let grad_local = vec3<f32>(local.x, -slope2 * local.y, local.z);
    let grad_world = vec3<f32>(grad_local.x, -grad_local.y, grad_local.z);
    return normalize(grad_world);
}

fn sky_color(direction: vec3<f32>) -> vec3<f32> {
    let t = clamp(direction.y * 0.5 + 0.5, 0.0, 1.0);
    let horizon = vec3<f32>(0.82, 0.90, 1.0);
    let zenith = vec3<f32>(0.16, 0.21, 0.31);
    return mix(horizon, zenith, t);
}

fn light_position() -> vec3<f32> {
    return vec3<f32>(
        sin(uniforms.time * 0.60) * 1.6,
        5.5 + sin(uniforms.time * 0.37) * 0.35,
        5.8 + cos(uniforms.time * 0.22) * 0.8,
    );
}

fn light_radiance() -> vec3<f32> {
    return vec3<f32>(13.0, 12.6, 11.8);
}

fn reflectivity_for_kind(kind: i32) -> f32 {
    switch kind {
        case 1: {
            return 0.18;
        }
        case 2: {
            return 0.08;
        }
        case 3: {
            return 0.34;
        }
        case 5: {
            return 0.06;
        }
        default: {
            return 0.0;
        }
    }
}

fn scene_intersect(ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> Hit {
    var hit = miss_hit();

    // Front sphere
    let center_a = vec3<f32>(0.0, 1.0, 5.85);
    let t_a = intersect_sphere(ray_origin, ray_direction, center_a, 1.0);
    if t_a > 0.0 && t_a < hit.t {
        let p = ray_origin + ray_direction * t_a;
        hit = Hit(
            t_a,
            p,
            normalize(p - center_a),
            vec3<f32>(0.85, 0.30, 0.26),
            vec3<f32>(0.0),
            1,
        );
    }

    // Back-left cone
    let cone_apex = vec3<f32>(-1.70, 2.05, 7.60);
    let cone_height = 2.00;
    let cone_radius = 0.95;
    let t_cone = intersect_cone(ray_origin, ray_direction, cone_apex, cone_height, cone_radius);
    if t_cone > 0.0 && t_cone < hit.t {
        let p = ray_origin + ray_direction * t_cone;
        hit = Hit(
            t_cone,
            p,
            cone_normal(p, cone_apex, cone_height, cone_radius),
            vec3<f32>(0.95, 0.67, 0.22),
            vec3<f32>(0.0),
            2,
        );
    }

    // Back-right cube
    let cube_center = vec3<f32>(1.80, 0.95, 7.55);
    let cube_half = vec3<f32>(0.90, 0.90, 0.90);
    let t_cube = intersect_box(ray_origin, ray_direction, cube_center, cube_half);
    if t_cube > 0.0 && t_cube < hit.t {
        let p = ray_origin + ray_direction * t_cube;
        hit = Hit(
            t_cube,
            p,
            box_normal(p, cube_center, cube_half),
            vec3<f32>(0.34, 0.62, 0.90),
            vec3<f32>(0.0),
            3,
        );
    }

    let center_light = light_position();
    let t_light = intersect_sphere(ray_origin, ray_direction, center_light, 0.75);
    if t_light > 0.0 && t_light < hit.t {
        let p = ray_origin + ray_direction * t_light;
        hit = Hit(
            t_light,
            p,
            normalize(p - center_light),
            vec3<f32>(1.0),
            light_radiance(),
            4,
        );
    }

    let t_plane = intersect_plane(
        ray_origin,
        ray_direction,
        vec3<f32>(0.0, 1.0, 0.0),
        0.0,
    );
    if t_plane > 0.0 && t_plane < hit.t {
        let p = ray_origin + ray_direction * t_plane;
        hit = Hit(
            t_plane,
            p,
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(0.90, 0.90, 0.90),
            vec3<f32>(0.0),
            5,
        );
    }

    return hit;
}

fn is_occluded(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32) -> bool {
    let blocker = scene_intersect(origin, direction);
    return blocker.kind != 0 && blocker.t < max_distance;
}

fn direct_lighting(hit: Hit, view_direction: vec3<f32>) -> vec3<f32> {
    let light_pos = light_position();
    let to_light = light_pos - hit.position;
    let light_distance = length(to_light);
    if light_distance <= 0.001 {
        return vec3<f32>(0.0);
    }

    let light_direction = to_light / light_distance;
    let n_dot_l = max(dot(hit.normal, light_direction), 0.0);
    if n_dot_l <= 0.0 {
        return hit.albedo * 0.035;
    }

    let shadow_origin = hit.position + hit.normal * (SURFACE_BIAS * 2.0);
    let in_shadow = is_occluded(shadow_origin, light_direction, light_distance - SURFACE_BIAS * 4.0);
    let shadow_factor = select(1.0, 0.14, in_shadow);
    let attenuation = 1.0 / (1.0 + light_distance * light_distance * 0.06);

    let diffuse = hit.albedo * n_dot_l * shadow_factor * attenuation * light_radiance();

    var specular_power = 24.0;
    var specular_level = 0.08;
    if hit.kind == 1 {
        specular_power = 34.0;
        specular_level = 0.16;
    }
    if hit.kind == 2 {
        specular_power = 18.0;
        specular_level = 0.05;
    }
    if hit.kind == 3 {
        specular_power = 72.0;
        specular_level = 0.34;
    }
    let half_vector = normalize(light_direction + view_direction);
    let specular = vec3<f32>(specular_level) * pow(max(dot(hit.normal, half_vector), 0.0), specular_power)
        * shadow_factor * attenuation * light_radiance();

    let ambient = hit.albedo * 0.035;
    return ambient + diffuse + specular;
}

fn trace_ray(ray_origin_input: vec3<f32>, ray_direction_input: vec3<f32>) -> vec3<f32> {
    var ray_origin = ray_origin_input;
    var ray_direction = ray_direction_input;
    var throughput = vec3<f32>(1.0, 1.0, 1.0);
    var radiance = vec3<f32>(0.0, 0.0, 0.0);

    for (var bounce = 0; bounce < MAX_REFLECTION_BOUNCES; bounce += 1) {
        let hit = scene_intersect(ray_origin, ray_direction);
        if hit.kind == 0 {
            radiance += throughput * sky_color(ray_direction);
            break;
        }

        radiance += throughput * hit.emission;
        radiance += throughput * direct_lighting(hit, normalize(-ray_direction));

        let reflectivity = reflectivity_for_kind(hit.kind);
        if reflectivity <= 0.0 || bounce == MAX_REFLECTION_BOUNCES - 1 {
            break;
        }

        throughput *= reflectivity;
        if max(max(throughput.x, throughput.y), throughput.z) < 0.02 {
            break;
        }

        ray_origin = hit.position + hit.normal * (SURFACE_BIAS * 2.0);
        ray_direction = normalize(reflect(ray_direction, hit.normal));
    }

    return radiance;
}

fn tonemap_aces(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let vertices = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    let local = vertices[in.vertex_index];
    let pixel_pos = uniforms.rect.xy + local * uniforms.rect.zw;

    let clip = vec2<f32>(
        (pixel_pos.x / uniforms.screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / uniforms.screen_size.y) * -2.0 + 1.0,
    );

    var out: VertexOutput;
    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.coord = vec2<f32>(local.x * 2.0 - 1.0, 1.0 - local.y * 2.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let width = max(uniforms.rect.z, 1.0);
    let height = max(uniforms.rect.w, 1.0);
    let aspect = width / height;

    let ray_direction = normalize(
        uniforms.camera_forward.xyz
            + in.coord.x * aspect * uniforms.camera_right.xyz
            + in.coord.y * uniforms.camera_up.xyz
    );

    var color = trace_ray(uniforms.camera_position.xyz, ray_direction);
    color = tonemap_aces(color);
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, uniforms.opacity);
}
