struct Uniforms {
    color: vec4<f32>,
    time: f32, // Kept for background, but not used for logo
    size: vec2<f32>,
    seed: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // The logo is now static. No more floating animation.
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.normal = model.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Simple, high-contrast lighting model
    let light_dir = normalize(vec3(0.5, 0.5, 1.0));
    let diffuse_light = pow(max(dot(in.normal, light_dir), 0.0), 2.0) * 0.6 + 0.4;

    // 2. Subtle Fresnel effect for edge highlighting
    let view_dir = normalize(vec3(0.0, 0.0, 1.0));
    let fresnel_dot = 1.0 - abs(dot(view_dir, in.normal));
    let fresnel = smoothstep(0.7, 1.0, fresnel_dot) * 0.5;

    var final_color = u.color;
    
    // Combine base color with lighting and add fresnel highlight
    let lit_rgb = final_color.rgb * diffuse_light + vec3<f32>(fresnel);
    final_color.r = lit_rgb.r;
    final_color.g = lit_rgb.g;
    final_color.b = lit_rgb.b;

    return vec4<f32>(final_color.rgb, u.color.a);
}