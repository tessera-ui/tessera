pub mod lens;
mod params;
pub mod pipeline;
mod shared;

use glam::Vec4;
use tessera_ui::{
    Px,
    render::{DrawCommand, DrawRegion, PaddingRect, SampleRegion},
};

/// A colour matrix described by contrast endpoints, saturation and a fill blend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToneMatrix {
    pub white: f32,
    pub black: f32,
    pub saturation: f32,
    pub fill: [f32; 4],
}

impl ToneMatrix {
    /// Rows of a 3x4 matrix: saturation compensated for the black/white contrast, the contrast
    /// remap, then a blend toward the fill colour by its alpha.
    fn rows(self) -> [Vec4; 3] {
        let contrast = self.white - self.black;
        let saturation = if contrast != 0.0 {
            self.saturation / contrast
        } else {
            self.saturation
        };
        let keep = 1.0 - self.fill[3];
        std::array::from_fn(|row| {
            let mut out = [0.0; 4];
            for (column, value) in out.iter_mut().take(3).enumerate() {
                let identity = if row == column { saturation } else { 0.0 };
                *value = keep * contrast * ((1.0 - saturation) * LUMA[column] + identity);
            }
            out[3] = keep * self.black + self.fill[3] * self.fill[row];
            Vec4::from_array(out)
        })
    }
}

const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// Optical inputs of a liquid glass material. Lengths are in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LiquidGlassMaterial {
    /// Extra backdrop captured around the group.
    pub margin: f32,
    /// How far the glass field extends outside each element.
    pub field_extent: f32,
    pub inner_refraction: [f32; 2],
    pub outer_refraction: [f32; 2],
    pub refraction_distance: [f32; 2],
    pub refraction_opacity: f32,
    pub blur_radius: f32,
    pub blur_opacity: [f32; 4],
    pub blur_distance: [f32; 4],
    pub blur_fill_radius: f32,
    /// Darken, lighten and normal opacities of the blur fill.
    pub blur_fill: [f32; 3],
    pub face: ToneMatrix,
    pub face_max_luma: f32,
    pub face_opacity: f32,
    pub shadow: ToneMatrix,
    pub shadow_opacity: f32,
    pub shadow_radius: f32,
    pub shadow_offset: f32,
    /// Offset, stroke width, blur radius, opacity and mask of the ring shadow.
    pub ring: [f32; 5],
    /// Angle, height, spread, amount, effect offset and colour bias of the edge highlight.
    pub highlight: [f32; 6],
    pub clamp: f32,
    /// Rows of the colour matrix applied under the diffuse highlight (RGBA in, bias last).
    pub vibrant: [[f32; 5]; 4],
}

/// One fused element: a continuous-corner rectangle in group-local points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LiquidGlassElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

impl LiquidGlassElement {
    /// Blend toward a circular arc per axis when the continuous corner does not fit the side.
    fn corner(self) -> Vec4 {
        let radius = self.radius.abs();
        let length = 1.528_665 * radius;
        let weight = |half: f32| {
            if length == radius {
                0.0
            } else {
                ((length - half) / (length - radius)).clamp(0.0, 1.0)
            }
        };
        Vec4::new(
            radius,
            weight(self.width * 0.5),
            weight(self.height * 0.5),
            0.0,
        )
    }
}

/// Draw command for one fused glass group.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LiquidGlassCommand {
    pub material: LiquidGlassMaterial,
    pub elements: Vec<LiquidGlassElement>,
    /// Fusion distance between elements, in points.
    pub smoothness: f32,
    pub pixels_per_point: f32,
    pub opacity: f32,
}

impl LiquidGlassCommand {
    fn field_padding(&self) -> f32 {
        self.material.field_extent + self.smoothness
    }
}

impl DrawCommand for LiquidGlassCommand {
    fn draw_region(&self) -> DrawRegion {
        let padding = self.field_padding() + self.material.shadow_offset.abs();
        DrawRegion::PaddedLocal(PaddingRect::uniform(Px(
            (padding * self.pixels_per_point).ceil() as i32,
        )))
    }

    fn sample_region(&self) -> Option<SampleRegion> {
        let padding = (self.material.margin + 2.0)
            .max(self.field_padding() + self.material.shadow_offset.abs());
        Some(SampleRegion::uniform_padding_local(Px(
            (padding * self.pixels_per_point).ceil() as i32 + 2,
        )))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.opacity *= opacity.clamp(0.0, 1.0);
    }
}
