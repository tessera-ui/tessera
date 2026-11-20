// Fused liquid glass: one draw per glass group.
//
// Distances are in points with y pointing down, relative to the group origin. Backdrop surface
// texels use y pointing up from the bottom edge of the render target.

struct Element {
    // x, y, width, height in points.
    rect: vec4<f32>,
    // radius, horizontal corner weight, vertical corner weight, unused.
    corner: vec4<f32>,
};

struct Params {
    // Group origin in target pixels, pixels per point, opacity.
    origin_scale: vec4<f32>,
    // Target size in pixels, scene texture origin in target pixels.
    target_scene: vec4<f32>,
    // Draw rectangle in target pixels: x0, y0, x1, y1.
    draw_rect: vec4<f32>,
    // Level 0 origin in pixels from the bottom-left corner, chain level count, unused.
    surface: vec4<f32>,
    // Atlas origin and size of each chain level, in texels.
    levels: array<vec4<f32>, 6>,
    // Smoothness, glass padding, highlight padding, element count.
    field: vec4<f32>,
    refraction: vec4<f32>,
    // refraction threshold 0, threshold 1, refraction opacity, face opacity.
    mix_opacity: vec4<f32>,
    blur_alpha: vec4<f32>,
    blur_dist: vec4<f32>,
    // blur radius, blur fill radius, blur fill enabled, shadow opacity.
    blur: vec4<f32>,
    // blur fill darken, lighten, normal, max luma complement.
    blur_fill: vec4<f32>,
    face_cm0: vec4<f32>,
    face_cm1: vec4<f32>,
    face_cm2: vec4<f32>,
    // shadow colour (rgb from the shadow matrix bias), shadow face opacity.
    shadow_colour: vec4<f32>,
    // shadow offset (points), shadow inverse radius, unused.
    shadow: vec4<f32>,
    // ring offset (points), stroke width, radius.
    ring: vec4<f32>,
    // ring opacity, ring mask, clamp limit, preserve hue.
    ring_clamp: vec4<f32>,
    // highlight direction, height, spread.
    highlight: vec4<f32>,
    // highlight amount, effect offset, colour bias, unused.
    highlight_tone: vec4<f32>,
    // diffuse highlight direction, curvature, offset.
    diffuse: vec4<f32>,
    // per band: height.
    diffuse_height: vec4<f32>,
    // per band: cos(spread).
    diffuse_spread: vec4<f32>,
    // per band: amount.
    diffuse_amount: vec4<f32>,
    // key colour alpha, fill colour alpha, matrix clamp, unused.
    diffuse_tone: vec4<f32>,
    vibrant0: vec4<f32>,
    vibrant1: vec4<f32>,
    vibrant2: vec4<f32>,
    vibrant3: vec4<f32>,
    // Bias column of the vibrant matrix.
    vibrant_bias: vec4<f32>,
    // Backdrop texels per point, unused.
    sampling: vec4<f32>,
    // Lens displacement: height, curvature, amount in texels, unused.
    lens_displacement: vec4<f32>,
    // Lens chromatic aberration: amount, inverse height, offset, unused.
    lens_aberration: vec4<f32>,
    // Lens aberration direction, refraction direction.
    lens_directions: vec4<f32>,
    // Lens edge fade: start, end, opacity at start, opacity at end.
    lens_edge: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> elements: array<Element>;
@group(0) @binding(2) var surface_texture: texture_2d<f32>;
@group(0) @binding(3) var surface_sampler: sampler;
@group(0) @binding(4) var scene_texture: texture_2d<f32>;
@group(0) @binding(5) var original_texture: texture_2d<f32>;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const C0: f32 = 1.52866495;
const C1: f32 = -0.528664947;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
        vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    );
    let rect = params.draw_rect;
    let pixel = mix(rect.xy, rect.zw, corners[index]);
    let ndc = pixel / params.target_scene.xy * 2.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4(ndc.x, -ndc.y, 0.0, 1.0);
    return out;
}

fn saturate_nan(x: f32) -> f32 {
    // GPU saturate maps NaN to zero; select keeps that behaviour explicit.
    return select(0.0, clamp(x, 0.0, 1.0), x == x);
}

fn element_field(p: vec2<f32>, element: Element) -> vec3<f32> {
    let half_size = element.rect.zw * 0.5;
    let local = p - (element.rect.xy + half_size);
    let ap = abs(local);
    let r = max(abs(element.corner.x), 1e-4);
    let weights = element.corner.yz;
    let len = r * C0;
    let extent = len + (r - len) * max(weights.x, weights.y);
    let qe = ap - half_size + extent;
    let u = max(vec2(0.0), (ap - half_size + len) / len);
    let norm = length(u);
    let hi = max(u.x, u.y);
    let lo = min(u.x, u.y);
    let t = select(clamp(lo / hi, 0.0, 1.0), 0.0, hi == 0.0);
    let poly = (((-0.926054 * t + 3.15601) * t - 3.64122) * t + 1.26803) * t + 0.268531;
    let denom = 1.0 - t * t * clamp(norm, 0.0, 1.0) * poly;
    let continuous = norm + 1.0 - 1.0 / denom;
    let circle = length(max(vec2(0.0), u * C0 + C1)) * 0.654166 + 0.345834;
    let fx = continuous + (circle - continuous) * weights.x;
    let fy = continuous + (circle - continuous) * weights.y;
    let s = select(-1.0, 1.0, u.y > u.x);
    let w = clamp(0.5 - s + s * t, 0.0, 1.0);
    let f = fx + (fy - fx) * w - 1.0;
    let distance = min(max(qe.x, qe.y), 0.0) + len * f;
    let outside = max(qe, vec2(0.0));
    var g = select(vec2(0.0, 1.0), vec2(1.0, 0.0), qe.x > qe.y);
    if outside.x + outside.y > 0.0 {
        g = outside / length(outside);
    }
    g = g * select(vec2(-1.0), vec2(1.0), local >= vec2(0.0));
    return vec3(distance, g);
}

// Field of the group at p: distance, gradient, coverage flag (0 outside every element quad).
fn group_field(p: vec2<f32>, padding: f32) -> vec4<f32> {
    let smoothness = params.field.x;
    var field = vec4(10000.0, 0.0, 0.0, 0.0);
    let count = u32(params.field.w);
    for (var i = 0u; i < count; i++) {
        let element = elements[i];
        let half_size = element.rect.zw * 0.5;
        let local = p - (element.rect.xy + half_size);
        if any(abs(local) > half_size + padding) {
            continue;
        }
        let a = element_field(p, element);
        let k = clamp(0.5 - 0.5 * dot(a.yz, field.yz), 0.0, 1.0) * smoothness;
        var h = clamp((field.x - a.x) * 0.5 / k + 0.5, 0.0, 1.0);
        if h != h {
            h = 0.5;
        }
        let d = field.x + (a.x - field.x) * h - k * h * (1.0 - h);
        let g = field.yz + (a.yz - field.yz) * h;
        field = vec4(d, g, 1.0);
    }
    return field;
}

fn gauss_tail(x: f32) -> f32 {
    let y = saturate_nan(x * 0.70710677 * 0.25 + 0.5) * 4.0 - 2.0;
    let y2 = y * y;
    let p = ((0.00295448 * y2 - 0.0344543) * y2 + 0.168213) * y2 - 0.560547;
    return p * y + 0.5;
}

fn refraction_offset(inv_height: f32, amount: f32, d: f32) -> f32 {
    let t = clamp(inv_height * -d, 0.0, 1.0);
    return amount - clamp(sqrt((2.0 - t) * t), 0.0, 1.0) * amount;
}

fn blur_amount(depth: f32) -> f32 {
    let lo = params.blur_dist.xyz;
    let hi = params.blur_dist.yzw;
    var total = params.blur_alpha.x;
    let coef = params.blur_alpha.yzw;
    for (var i = 0; i < 3; i++) {
        let span = hi[i] - lo[i];
        var ramp = 0.0;
        if span != 0.0 {
            ramp = clamp((depth - lo[i]) / span, 0.0, 1.0);
        } else {
            ramp = select(0.0, 1.0, depth >= lo[i]);
        }
        total -= ramp * coef[i];
    }
    return total * params.blur.x;
}

fn lod_for(blur: f32) -> f32 {
    let arg = select(blur, blur * 0.5 + 1.0, blur < 2.0);
    return max(0.0, log2(max(arg, 1e-30)));
}

fn level_sample(level: i32, uv: vec2<f32>) -> vec4<f32> {
    let entry = params.levels[level];
    let t = clamp(uv / exp2(f32(level)), vec2(0.5), entry.zw - 0.5);
    let atlas = vec2<f32>(textureDimensions(surface_texture));
    return textureSampleLevel(surface_texture, surface_sampler, (entry.xy + t) / atlas, 0.0);
}

// Trilinear sample of the chain at level-0 texel coordinates with an explicit level of detail.
fn backdrop(uv: vec2<f32>, lod: f32) -> vec4<f32> {
    let top = i32(params.surface.z) - 1;
    let level = clamp(lod, 0.0, f32(top));
    let l0 = i32(floor(level));
    let l1 = min(l0 + 1, top);
    return mix(level_sample(l0, uv), level_sample(l1, uv), level - f32(l0));
}

fn unpremultiply(c: vec4<f32>) -> vec3<f32> {
    let rgb = c.rgb / max(c.a, 1e-6);
    return select(rgb, vec3(0.0), abs(rgb) < vec3(1e-6));
}

fn face_transform(colour: vec3<f32>) -> vec3<f32> {
    var x = colour;
    let complement = params.blur_fill.w;
    if complement > 0.0 {
        let l = dot(x, LUMA);
        let k = clamp(1.0 - l * complement, 0.0, 1.0);
        let gray = k * l;
        x = gray + (k * x - gray) * (1.0 + (1.0 - k) * 0.3);
    }
    return vec3(
        dot(x, params.face_cm0.xyz) + params.face_cm0.w,
        dot(x, params.face_cm1.xyz) + params.face_cm1.w,
        dot(x, params.face_cm2.xyz) + params.face_cm2.w,
    );
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3(0.0031308));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
    return select(pow((max(c, vec3(0.0)) + 0.055) / 1.055, vec3(2.4)), c / 12.92, c <= vec3(0.04045));
}

// Glass colour (premultiplied) over the destination, before the diffuse highlight.
fn glass_colour(px: vec2<f32>, p: vec2<f32>, uv0: vec2<f32>) -> vec4<f32> {
    let field = group_field(p, params.field.y);
    let d = field.x;
    let grad = field.yz;
    let alpha = field.w;
    let aa = max(fwidthFine(d), 1e-4);
    let coverage = clamp(0.5 - d / aa, 0.0, 1.0) * alpha;
    let shadow_field = group_field(p - params.shadow.xy, params.field.y);
    let ring_field = group_field(p - params.ring.xy, params.field.y);

    var highlight = 0.0;
    let hl_height = params.highlight.z;
    if hl_height > 0.0 {
        let off = params.highlight_tone.y + d;
        let nd = -off;
        let half_aa = aa * 0.5;
        let skip = off >= half_aa || coverage >= 1.0 || alpha == 0.0;
        let mask = alpha * (1.0 - coverage) * clamp(nd / aa + 0.5, 0.0, 1.0);
        let spread = params.highlight.w;
        let inv = 1.0 / max(1.0 - spread, 1e-5);
        let k = dot(grad, params.highlight.xy);
        let v = clamp((vec2(k, -k) - spread) * inv, vec2(0.0), vec2(1.0)) * mask;
        let r = v / max(params.highlight_tone.x * (1.0 - v) + 1.0, vec2(1e-5));
        highlight = select(r.x + r.y, 0.0, skip);
    }

    var ring = 0.0;
    let ring_opacity = params.ring_clamp.x;
    let ring_mask = params.ring_clamp.y;
    if ring_opacity > 0.0 && (coverage > 0.0 || ring_mask < 1.0) {
        let inv_r = 1.0 / max(params.ring.w, 1e-5);
        let a = inv_r * ring_field.x;
        let b = a + params.ring.z * inv_r;
        let band = clamp(gauss_tail(a) - gauss_tail(b), 0.0, 1.0);
        ring = ring_opacity * ring_field.w * band * (1.0 + (coverage - 1.0) * ring_mask);
    }

    var shadow_amount = 0.0;
    if coverage < 1.0 {
        shadow_amount = params.blur.w * shadow_field.w * gauss_tail(shadow_field.x * params.shadow.z);
    }
    let shadow = shadow_amount * params.shadow_colour;
    if coverage == 0.0 && shadow_amount < 1e-6 && highlight < 1e-6 && ring < 1e-6 {
        return vec4(0.0);
    }

    var body = vec4(0.0);
    if coverage > 0.0 {
        let disp = refraction_offset(params.refraction.y, params.refraction.x, d);
        let shift = vec2(grad.x, -grad.y) * params.sampling.x;
        let base = backdrop(uv0 + disp * shift, lod_for(blur_amount(disp + d)));
        let a_rgb = unpremultiply(base);
        var colour = vec4(a_rgb, 1.0);
        if params.blur.z > 0.0 {
            let lod2 = lod_for(params.blur.y);
            let o = vec2(lod2 * 0.25);
            let fill = unpremultiply((backdrop(uv0 + o, lod2) + backdrop(uv0 - o, lod2)) * 0.5);
            let darken = params.blur_fill.x;
            let lighten = params.blur_fill.y;
            let mixed = darken * min(a_rgb, fill) + lighten * max(a_rgb, fill) + (1.0 - darken - lighten) * a_rgb;
            colour = vec4(mixed + (fill - mixed) * params.blur_fill.z, 1.0);
        }
        let opacity = params.mix_opacity.z;
        if opacity > 0.0 {
            let disp_o = refraction_offset(params.refraction.w, params.refraction.z, d);
            let so = backdrop(uv0 + disp_o * shift, lod_for(blur_amount(disp_o + d)));
            let outer = vec4(select(so.rgb, vec3(0.0), abs(so.rgb) < vec3(1e-6)), so.a);
            let t0 = params.mix_opacity.x;
            let t1 = params.mix_opacity.y;
            let weight = clamp((d - t0) / (t1 - t0), 0.0, 1.0) * opacity;
            colour = colour + (outer - colour) * weight;
        }
        let face = params.mix_opacity.w;
        if face > 0.0 {
            colour = vec4(colour.rgb + (face_transform(colour.rgb) - colour.rgb) * face, colour.a);
        }
        body = colour;
    }

    var out = shadow + (body - shadow) * coverage;
    out = out * (1.0 - ring) + vec4(0.0, 0.0, 0.0, ring);

    if highlight > 1e-6 {
        let a = out.a;
        let ub = unpremultiply(backdrop(uv0, 0.0));
        var tinted = ub;
        let face = params.mix_opacity.w;
        if face > 0.0 {
            tinted = face_transform(ub);
            tinted = select(max(tinted, ub), min(tinted, ub), params.highlight_tone.z < 0.0);
            tinted = ub + (tinted - ub) * (face * highlight);
        }
        let full = a >= 1.0;
        let rgb = select(tinted * (1.0 - a) + out.rgb, out.rgb, full);
        let fill = select(ub, vec3(0.0), full);
        let bias = params.highlight_tone.z * highlight;
        let rgb2 = (bias * (3.0 - 2.0 * rgb) + 1.0) * rgb;
        out = vec4(rgb2 - (1.0 - a) * fill, a);
    }

    let limit = params.ring_clamp.z;
    if limit > 0.0 {
        let a = out.a;
        var rgb = out.rgb / max(a, 1e-6);
        if params.ring_clamp.w > 0.0 {
            let m = max(rgb.r, max(rgb.g, rgb.b));
            rgb = select(rgb, rgb * (limit / m), m > limit);
        } else {
            rgb = clamp(rgb, vec3(-0.75), vec3(limit));
        }
        out = vec4(rgb * a, a);
    }
    return out;
}

fn diffuse_highlight(p: vec2<f32>, dst: vec4<f32>) -> vec4<f32> {
    let field = group_field(p, params.field.z);
    let inv_len = 1.0 / length(field.yz);
    let n = field.yz * inv_len;
    let depth = (-field.x - params.diffuse.w) * inv_len;
    let fw = max(fwidthFine(depth), 1e-4);
    let k = dot(n, params.diffuse.xy);
    let inner = saturate_nan(depth / fw + 0.5);
    var bands = vec4(0.0);
    for (var i = 0; i < 4; i++) {
        let height = params.diffuse_height[i];
        let t = saturate_nan(depth / height);
        let step = select(0.0, 1.0, t < 1.0);
        var a = 1.0 - t;
        if i < 2 {
            a = step + ((1.0 - t) - step) * params.diffuse.z;
        }
        let edge = inner * saturate_nan((height - depth) / fw + 0.5);
        let spread = params.diffuse_spread[i];
        let dir = select(k, -k, i == 1 || i == 3);
        let ang = saturate_nan((dir - spread) / max(1.0 - spread, 1e-4));
        bands[i] = a * edge * ang;
    }
    let total = bands.x + bands.y + bands.z + bands.w;
    if !(total >= 1e-4) || depth < -5.0 {
        return dst;
    }
    let r = bands / max(1.0 + params.diffuse_amount * (1.0 - bands), vec4(1e-4));
    let strength = (r.x + r.z) * params.diffuse_tone.x + (r.y + r.w) * params.diffuse_tone.y;
    let rgb = dst.rgb / max(dst.a, 1e-4);
    let colour = vec3(
        dot(rgb, params.vibrant0.xyz),
        dot(rgb, params.vibrant1.xyz),
        dot(rgb, params.vibrant2.xyz),
    ) + dst.a * vec3(params.vibrant0.w, params.vibrant1.w, params.vibrant2.w) + params.vibrant_bias.xyz;
    let alpha = saturate_nan(dot(rgb, params.vibrant3.xyz) + dst.a * params.vibrant3.w + params.vibrant_bias.w);
    var src = vec4(colour * alpha, alpha) * strength;
    let limit = params.diffuse_tone.z;
    if limit > 0.0 {
        let un = clamp(src.rgb / max(src.a, 1e-4), vec3(-0.75), vec3(limit));
        src = vec4(un * src.a, src.a);
    }
    let out = dst * (1.0 - src.a) + src;
    return vec4(out.rgb, clamp(out.a, 0.0, 1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = in.position.xy;
    let scale = params.origin_scale.z;
    let p = (px - params.origin_scale.xy) / scale;
    let from_bottom = vec2(px.x, params.target_scene.y - px.y);
    let uv0 = (from_bottom - params.surface.xy) * 0.5;

    let scene_pixel = vec2<i32>(floor(px - params.target_scene.zw));
    let stored = textureLoad(scene_texture, scene_pixel, 0);
    let dst0 = vec4(srgb_encode(stored.rgb), stored.a);

    let glass = glass_colour(px, p, uv0);
    let dst1 = glass + dst0 * (1.0 - glass.a);
    let dst2 = diffuse_highlight(p, dst1);
    let result = mix(dst0, dst2, params.origin_scale.w);
    return vec4(srgb_decode(result.rgb), result.a);
}

// --- Pressed lens -----------------------------------------------------------------------------
//
// The lens passes render into half-float textures that cover the lens region with rows stored
// from the bottom up and values kept sRGB encoded. `target_scene.zw` holds the region origin in
// target pixels (top-down).

@vertex
fn vs_region(@builtin(vertex_index) index: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
        vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
    );
    var out: VertexOutput;
    out.position = vec4(corners[index], 0.0, 1.0);
    return out;
}

struct RegionPixel {
    // Target pixel centre, top-down.
    px: vec2<f32>,
    // Region texel holding this pixel, bottom-up.
    texel: vec2<i32>,
};

// Maps a fragment of a region texture to its target pixel; texture row r holds the r-th pixel
// row counted from the bottom of the region.
fn region_pixel(position: vec2<f32>) -> RegionPixel {
    let size = vec2<f32>(textureDimensions(scene_texture));
    var out: RegionPixel;
    out.px = vec2(params.target_scene.z + position.x, params.target_scene.w + size.y - position.y);
    out.texel = vec2<i32>(floor(position));
    return out;
}

// Bottom-up texel coordinates of a target pixel inside the lens region.
fn lens_uv(px: vec2<f32>) -> vec2<f32> {
    let size = vec2<f32>(textureDimensions(scene_texture));
    return vec2(px.x - params.target_scene.z, params.target_scene.w + size.y - px.y);
}

fn lens_profile(depth: f32, height: f32, curvature: f32) -> f32 {
    let t = clamp(depth / height, 0.0, 1.0);
    let step = select(1.0, 1.0 - 0.29289323, t < 1.0);
    let circle = sqrt(max(1.0 - (1.0 - t) * (1.0 - t), 0.0));
    return step + (circle - step) * curvature;
}

@fragment
fn fs_lens_displace(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = region_pixel(in.position.xy);
    let p = (pixel.px - params.origin_scale.xy) / params.origin_scale.z;
    let field = group_field(p, params.field.y);
    let depth = -field.x;
    let fw = max(fwidthFine(depth), 1e-4);
    let base = textureLoad(scene_texture, pixel.texel, 0);
    var mask = clamp(depth / fw + 0.5, 0.0, 1.0) * field.w;
    if depth < -5.0 {
        mask = 0.0;
    }
    if mask < 1.0 / 255.0 {
        return base;
    }
    let n = field.yz / max(length(field.yz), 1e-6);
    let height = params.lens_displacement.x;
    let s = lens_profile(depth, height, params.lens_displacement.y);
    let inner = clamp((height - depth) / fw + 0.5, 0.0, 1.0);
    let v = (1.0 - s) * n * inner;
    let size = vec2<f32>(textureDimensions(surface_texture));
    let uv = vec2<f32>(pixel.texel) + 0.5 + params.lens_displacement.z * vec2(v.x, -v.y);
    let displaced = textureSampleLevel(surface_texture, surface_sampler, uv / size, 0.0);
    return displaced * mask + base * (1.0 - mask);
}

@fragment
fn fs_lens_background(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = region_pixel(in.position.xy);
    let p = (pixel.px - params.origin_scale.xy) / params.origin_scale.z;
    let dst = textureLoad(scene_texture, pixel.texel, 0);
    let glass = glass_colour(pixel.px, p, vec2<f32>(pixel.texel) + 0.5);
    return glass + dst * (1.0 - glass.a);
}

// Seven-tap colour separation along `offset`, unpremultiplied per tap.
fn aberrate(uv: vec2<f32>, offset: vec2<f32>) -> vec4<f32> {
    let size = vec2<f32>(textureDimensions(surface_texture));
    var acc = vec4(0.0);
    for (var i = 0; i < 3; i++) {
        let w = 1.0 - f32(i) / 3.0;
        let s = textureSampleLevel(surface_texture, surface_sampler, (uv + w * offset) / size, 0.0);
        let a = max(s.a, 1e-4);
        acc.r += w * s.r / a;
        acc.g += (1.0 - w) * s.g / a;
        acc.a += s.a;
    }
    for (var i = 0; i < 4; i++) {
        let w = f32(i) / 3.0;
        let s = textureSampleLevel(surface_texture, surface_sampler, (uv - w * offset) / size, 0.0);
        let a = max(s.a, 1e-4);
        acc.g += (1.0 - w) * s.g / a;
        acc.b += w * s.b / a;
        acc.a += s.a;
    }
    return vec4(acc.rgb * vec3(0.5, 1.0 / 3.0, 0.5), acc.a / 7.0);
}

fn lens_foreground(px: vec2<f32>, p: vec2<f32>) -> vec4<f32> {
    let field = group_field(p, params.field.y);
    let d = field.x;
    let aa = max(fwidthFine(d), 1e-4);
    if field.w < 0.001 {
        return vec4(0.0);
    }
    let coverage = clamp(0.5 - d / aa, 0.0, 1.0) * field.w;
    let g = field.yz;
    let scale = params.sampling.x;
    let a = params.lens_directions.xy;
    let profile = params.lens_aberration.x
        - clamp(sqrt(max((2.0 - clamp((-d - params.lens_aberration.z) * params.lens_aberration.y, 0.0, 1.0))
            * clamp((-d - params.lens_aberration.z) * params.lens_aberration.y, 0.0, 1.0), 0.0)), 0.0, 1.0)
            * params.lens_aberration.x;
    let offset = profile * scale * vec2(-(a.y * g.x + a.x * g.y), a.x * g.x - a.y * g.y);
    let sample = aberrate(lens_uv(px), offset);
    let alpha = coverage * sample.a;
    let edge = params.lens_edge;
    let t = clamp((d - edge.x) / (edge.y - edge.x), 0.0, 1.0);
    let fade = 1.0 - mix(edge.z, edge.w, t);
    return vec4(sample.rgb * alpha, alpha) * fade;
}

@fragment
fn fs_lens(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = in.position.xy;
    let p = (px - params.origin_scale.xy) / params.origin_scale.z;
    let texel = vec2<i32>(floor(lens_uv(px)));
    let dst = textureLoad(scene_texture, texel, 0);
    let foreground = lens_foreground(px, p);
    let composed = diffuse_highlight(p, foreground + dst * (1.0 - foreground.a));
    let stored = textureLoad(original_texture, texel, 0);
    let result = mix(stored, composed, params.origin_scale.w);
    return vec4(srgb_decode(result.rgb), result.a);
}
