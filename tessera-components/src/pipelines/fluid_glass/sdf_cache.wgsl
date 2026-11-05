struct SdfUniforms {
    size: vec2<f32>,
    corner_radii: vec4<f32>,
    corner_g2: vec4<f32>,
    shape_type: f32,
}

@group(0) @binding(0) var<uniform> uniforms: SdfUniforms;
@group(0) @binding(1) var sdf_texture: texture_storage_2d<rgba16float, write>;

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

    let result = shape_sd_normal(uniforms.shape_type, centered, half_size, uniforms.corner_radii, uniforms.corner_g2);
    let normal = normalize(result.yz);
    textureStore(sdf_texture, global_id.xy, vec4<f32>(result.x, normal.x, normal.y, 0.0));
}
