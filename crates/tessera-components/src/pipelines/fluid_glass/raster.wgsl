@group(0) @binding(5) var lens_output: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(8, 8)
fn cs_primary(@builtin(global_invocation_id) id: vec3<u32>) {
    let instance = uniforms.instances[0];
    if any(id.xy >= vec2<u32>(instance.raster_size)) {
        return;
    }
    let coord = (vec2<f32>(id.xy) + 0.5 - instance.raster_origin) * lens_raster_step(instance);
    textureStore(lens_output, vec2<i32>(id.xy), vec4(lens_primary_pixel(instance, coord), 1.0));
}

@compute @workgroup_size(8, 8)
fn cs_background(@builtin(global_invocation_id) id: vec3<u32>) {
    let instance = uniforms.instances[0];
    if any(id.xy >= vec2<u32>(instance.raster_size)) {
        return;
    }
    let coord = (vec2<f32>(id.xy) + 0.5 - instance.raster_origin) * lens_raster_step(instance);
    textureStore(lens_output, vec2<i32>(id.xy), vec4(lens_refracted_pixel(instance, coord), 1.0));
}
