// Backdrop capture and blur chain for liquid glass.
//
// All coordinates use y pointing up from the bottom edge of the render target. Each pass writes
// one level into a scratch texture that is then copied into the chain atlas.

struct BackdropParams {
    // Region origin in pixels from the bottom-left corner, region size in capture texels.
    region: vec4<i32>,
    // Offset of the capture inside level 0, target height in pixels, unused.
    capture: vec4<i32>,
    // Scene texture origin in target pixels, unused.
    scene: vec4<i32>,
    // Source level origin in the atlas, unused.
    levels: vec4<i32>,
    // Source level size, destination level size.
    sizes: vec4<i32>,
};

@group(0) @binding(0) var<uniform> backdrop: BackdropParams;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var destination: texture_storage_2d<rgba16float, write>;

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3(0.0031308));
}

fn mirror(i: i32, size: i32) -> i32 {
    var j = i;
    if j < 0 {
        j = -j;
    }
    if j > size - 1 {
        j = 2 * (size - 1) - j;
    }
    return clamp(j, 0, size - 1);
}

fn scene_pixel(from_bottom: vec2<i32>) -> vec4<f32> {
    let height = backdrop.capture.z;
    let pixel = vec2(from_bottom.x, height - 1 - from_bottom.y) - backdrop.scene.xy;
    let size = vec2<i32>(textureDimensions(source));
    let stored = textureLoad(source, clamp(pixel, vec2(0), size - 1), 0);
    return vec4(srgb_encode(stored.rgb), stored.a);
}

// Level 0: the capture with mirror addressing over the region.
@compute @workgroup_size(8, 8)
fn cs_base(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = vec2<i32>(id.xy);
    if any(texel >= backdrop.sizes.zw) {
        return;
    }
    let capture = vec2(
        mirror(texel.x - backdrop.capture.x, backdrop.region.z),
        mirror(texel.y - backdrop.capture.y, backdrop.region.w),
    );
    let block = backdrop.region.xy + 2 * capture;
    let colour = (scene_pixel(block) + scene_pixel(block + vec2(1, 0)) + scene_pixel(block + vec2(0, 1))
        + scene_pixel(block + vec2(1, 1))) * 0.25;
    textureStore(destination, texel, colour);
}

const TAPS: array<vec3<f32>, 13> = array<vec3<f32>, 13>(
    vec3(0.0, 0.0, 0.10547),
    vec3(1.0, 0.0, 0.09021), vec3(-1.0, 0.0, 0.09021), vec3(0.0, 1.0, 0.09021), vec3(0.0, -1.0, 0.09021),
    vec3(1.0, 1.0, 0.07709), vec3(1.0, -1.0, 0.07709), vec3(-1.0, 1.0, 0.07709), vec3(-1.0, -1.0, 0.07709),
    vec3(2.0, 0.0, 0.05634), vec3(-2.0, 0.0, 0.05634), vec3(0.0, 2.0, 0.05634), vec3(0.0, -2.0, 0.05634),
);

fn level_texel(texel: vec2<i32>, mirrored: bool) -> vec4<f32> {
    var t = texel;
    if mirrored {
        t = backdrop.capture.xy + vec2(
            mirror(texel.x - backdrop.capture.x, backdrop.region.z),
            mirror(texel.y - backdrop.capture.y, backdrop.region.w),
        );
    }
    t = clamp(t, vec2(0), backdrop.sizes.xy - 1);
    return textureLoad(source, backdrop.levels.xy + t, 0);
}

// Level n > 0: the 13-tap kernel over 2x2 boxes of level n - 1. Level 1 reads level 0 with
// mirror addressing over the capture region.
@compute @workgroup_size(8, 8)
fn cs_downsample(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = vec2<i32>(id.xy);
    if any(texel >= backdrop.sizes.zw) {
        return;
    }
    let mirrored = backdrop.capture.w != 0;
    var colour = vec4(0.0);
    for (var i = 0; i < 13; i++) {
        let tap = TAPS[i];
        let corner = 2 * (texel + vec2<i32>(tap.xy));
        let box = (level_texel(corner, mirrored) + level_texel(corner + vec2(1, 0), mirrored)
            + level_texel(corner + vec2(0, 1), mirrored) + level_texel(corner + vec2(1, 1), mirrored)) * 0.25;
        colour += tap.z * box;
    }
    textureStore(destination, texel, colour);
}

// Full-resolution copy of a region given by its top-left target pixel (`region.xy`) and size
// (`sizes.zw`), stored with rows from the bottom up.
@compute @workgroup_size(8, 8)
fn cs_copy(@builtin(global_invocation_id) id: vec3<u32>) {
    let texel = vec2<i32>(id.xy);
    if any(texel >= backdrop.sizes.zw) {
        return;
    }
    let pixel = vec2(backdrop.region.x + texel.x, backdrop.region.y + backdrop.sizes.w - 1 - texel.y)
        - backdrop.scene.xy;
    let size = vec2<i32>(textureDimensions(source));
    let stored = textureLoad(source, clamp(pixel, vec2(0), size - 1), 0);
    textureStore(destination, texel, vec4(srgb_encode(stored.rgb), stored.a));
}
