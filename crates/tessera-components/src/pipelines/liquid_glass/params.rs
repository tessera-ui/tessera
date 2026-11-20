use encase::ShaderType;
use glam::Vec4;

use super::{LiquidGlassCommand, LiquidGlassMaterial};

pub(super) const MAX_LEVELS: usize = 6;

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ElementData {
    pub rect: Vec4,
    pub corner: Vec4,
}

#[derive(ShaderType, Clone, Debug, PartialEq)]
pub(super) struct Elements {
    #[shader(size(runtime))]
    pub elements: Vec<ElementData>,
}

/// Uniforms of the glass draw; layout matches `Params` in glass.wgsl.
#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct GlassParams {
    pub origin_scale: Vec4,
    pub target_scene: Vec4,
    pub draw_rect: Vec4,
    pub surface: Vec4,
    pub levels: [Vec4; MAX_LEVELS],
    pub field: Vec4,
    pub refraction: Vec4,
    pub mix_opacity: Vec4,
    pub blur_alpha: Vec4,
    pub blur_dist: Vec4,
    pub blur: Vec4,
    pub blur_fill: Vec4,
    pub face_cm0: Vec4,
    pub face_cm1: Vec4,
    pub face_cm2: Vec4,
    pub shadow_colour: Vec4,
    pub shadow: Vec4,
    pub ring: Vec4,
    pub ring_clamp: Vec4,
    pub highlight: Vec4,
    pub highlight_tone: Vec4,
    pub diffuse: Vec4,
    pub diffuse_height: Vec4,
    pub diffuse_spread: Vec4,
    pub diffuse_amount: Vec4,
    pub diffuse_tone: Vec4,
    pub vibrant0: Vec4,
    pub vibrant1: Vec4,
    pub vibrant2: Vec4,
    pub vibrant3: Vec4,
    pub vibrant_bias: Vec4,
}

/// Inverse of a length where zero disables the dependent term.
fn inverse(value: f32) -> f32 {
    if value == 0.0 { f32::MAX } else { 1.0 / value }
}

/// Explicit level of detail used for a blur radius in points.
pub(super) fn blur_lod(blur: f32) -> f32 {
    let arg = if blur < 2.0 { blur * 0.5 + 1.0 } else { blur };
    arg.max(f32::MIN_POSITIVE).log2().max(0.0)
}

impl LiquidGlassMaterial {
    fn blur_radius_px(self) -> f32 {
        self.blur_radius * 1.6
    }

    fn blur_fill_radius_px(self) -> f32 {
        self.blur_fill_radius * 1.6
    }

    fn blur_fill_enabled(self) -> bool {
        self.blur_fill.iter().any(|opacity| *opacity > 0.0)
    }

    /// Number of chain levels needed by the deepest blur this material samples.
    pub(super) fn levels(self) -> usize {
        let body = blur_lod(self.blur_radius_px() * self.blur_opacity[0].max(1.0));
        let fill = if self.blur_fill_enabled() {
            blur_lod(self.blur_fill_radius_px())
        } else {
            0.0
        };
        (body.max(fill).ceil() as usize + 1).min(MAX_LEVELS)
    }
}

// Diffuse key/fill highlight: key light from the top, fill light from the bottom.
const DIFFUSE_SPREAD: f32 = 1.396_263_4;
const DIFFUSE_HEIGHT_SCALE: f32 = 8.0;
const DIFFUSE_SPREAD_SCALE: f32 = 0.65;
const DIFFUSE_AMOUNT_SCALE: f32 = 0.15;
const DIFFUSE_AMOUNT: f32 = 0.5;
const DIFFUSE_CURVATURE: f32 = 0.75;

/// Highlight compression parameter for a highlight amount.
fn compression(amount: f32) -> f32 {
    if amount > 0.0 {
        1.0 / amount - 2.0
    } else {
        0.0
    }
}

pub(super) fn material_params(command: &LiquidGlassCommand) -> GlassParams {
    let m = command.material;
    let o = m.blur_opacity;
    let face = m.face.rows();
    let shadow = m.shadow.rows();
    let [
        ring_offset,
        ring_width,
        ring_radius,
        ring_opacity,
        ring_mask,
    ] = m.ring;
    let [angle, height, spread, amount, effect_offset, bias] = m.highlight;
    let diffuse_amount = compression(DIFFUSE_AMOUNT * DIFFUSE_AMOUNT_SCALE);
    let spread_cos = DIFFUSE_SPREAD.cos();
    let diffuse_cos = (DIFFUSE_SPREAD * DIFFUSE_SPREAD_SCALE).cos();
    let vibrant = |row: usize| {
        Vec4::new(
            m.vibrant[row][0],
            m.vibrant[row][1],
            m.vibrant[row][2],
            m.vibrant[row][3],
        )
    };
    GlassParams {
        field: Vec4::new(
            command.smoothness,
            command.field_padding(),
            1.0 + command.smoothness,
            command.elements.len() as f32,
        ),
        refraction: Vec4::new(
            m.inner_refraction[0],
            inverse(m.inner_refraction[1]),
            m.outer_refraction[0],
            inverse(m.outer_refraction[1]),
        ),
        mix_opacity: Vec4::new(
            m.refraction_distance[0],
            m.refraction_distance[1],
            m.refraction_opacity,
            m.face_opacity,
        ),
        blur_alpha: Vec4::new(o[0], o[0] - o[1], o[1] - o[2], o[2] - o[3]),
        blur_dist: Vec4::from_array(m.blur_distance),
        blur: Vec4::new(
            m.blur_radius_px(),
            m.blur_fill_radius_px(),
            f32::from(u8::from(m.blur_fill_enabled())),
            m.shadow_opacity,
        ),
        blur_fill: Vec4::new(
            m.blur_fill[0],
            m.blur_fill[1],
            m.blur_fill[2],
            1.0 - m.face_max_luma,
        ),
        face_cm0: face[0],
        face_cm1: face[1],
        face_cm2: face[2],
        shadow_colour: Vec4::new(shadow[0].w, shadow[1].w, shadow[2].w, m.shadow.fill[3]),
        shadow: Vec4::new(0.0, m.shadow_offset, inverse(m.shadow_radius), 0.0),
        ring: Vec4::new(0.0, ring_offset, ring_width, ring_radius),
        ring_clamp: Vec4::new(
            ring_opacity,
            ring_mask,
            if m.clamp > 0.0 {
                m.clamp.powf(1.0 / 2.2)
            } else {
                0.0
            },
            0.0,
        ),
        highlight: Vec4::new(angle.sin(), angle.cos(), height, spread.cos()),
        highlight_tone: Vec4::new(compression(amount), effect_offset, bias, 0.0),
        diffuse: Vec4::new(0.0, -1.0, DIFFUSE_CURVATURE, 0.0),
        diffuse_height: Vec4::new(1.0, 1.0, DIFFUSE_HEIGHT_SCALE, DIFFUSE_HEIGHT_SCALE),
        diffuse_spread: Vec4::new(spread_cos, spread_cos, diffuse_cos, diffuse_cos),
        diffuse_amount: Vec4::new(
            compression(DIFFUSE_AMOUNT),
            compression(DIFFUSE_AMOUNT),
            diffuse_amount,
            diffuse_amount,
        ),
        diffuse_tone: Vec4::new(1.0, 1.0, 1.0, 0.0),
        vibrant0: vibrant(0),
        vibrant1: vibrant(1),
        vibrant2: vibrant(2),
        vibrant3: vibrant(3),
        vibrant_bias: Vec4::new(
            m.vibrant[0][4],
            m.vibrant[1][4],
            m.vibrant[2][4],
            m.vibrant[3][4],
        ),
        ..Default::default()
    }
}

pub(super) fn element_data(command: &LiquidGlassCommand) -> Elements {
    Elements {
        elements: command
            .elements
            .iter()
            .map(|element| ElementData {
                rect: Vec4::new(element.x, element.y, element.width, element.height),
                corner: element.corner(),
            })
            .collect(),
    }
}
