pub mod pipeline;

use tessera_ui::{
    Color, Px,
    render::{DrawCommand, DrawRegion, PaddingRect, SampleRegion},
};

use crate::shape_def::Shape;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LensShadow {
    pub offset: f32,
    pub width: f32,
    pub blur: f32,
    pub opacity: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LensContour {
    pub controls: [[f32; 2]; 5],
    pub coverage_width: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LensOptics {
    pub contour: Option<LensContour>,
    pub edge_width: f32,
    pub refraction_width: f32,
    pub refraction_corner_g2: f32,
    pub inner_refraction_width: f32,
    pub inner_refraction: f32,
    pub dispersion_angle: f32,
    pub refraction: f32,
    pub dispersion: f32,
    pub transmission: f32,
    pub shadow: LensShadow,
}

/// Render-only arguments consumed by the fluid glass draw pipeline.
#[derive(Clone, PartialEq)]
pub(crate) struct FluidGlassRenderArgs {
    /// The tint color of the glass.
    pub(crate) tint_color: Color,
    /// The resolved glass shape.
    pub(crate) shape: Shape,
    /// The amount of noise to apply over the surface.
    pub(crate) noise_amount: f32,
    /// The scale of the noise pattern.
    pub(crate) noise_scale: f32,
    /// A time value used to animate the shader.
    pub(crate) time: f32,
    /// Optional normalized center (x, y) for the ripple animation.
    pub(crate) ripple_center: Option<[f32; 2]>,
    /// Optional ripple radius, expressed in normalized coordinates.
    pub(crate) ripple_radius: Option<f32>,
    /// Optional ripple tint alpha.
    pub(crate) ripple_alpha: Option<f32>,
    /// Strength multiplier for the ripple distortion.
    pub(crate) ripple_strength: Option<f32>,
    /// Outline thickness of the glass.
    pub(crate) border_width: Px,
    pub(crate) material_tone: Option<[f32; 4]>,
    pub(crate) blur_sigma_px: f32,
    pub(crate) pixels_per_dp: f32,
    pub(crate) opacity: f32,
    pub(crate) contrast: Option<f32>,
    pub(crate) dark: bool,
    pub(crate) regular: bool,
    pub(crate) feedback: f32,
    pub(crate) lens: Option<LensOptics>,
}

/// Draw command wrapping the arguments for the fluid glass surface.
#[derive(Clone, PartialEq)]
pub(crate) struct FluidGlassCommand {
    /// Render-only configuration used by the draw pipeline.
    pub(crate) render: FluidGlassRenderArgs,
}

impl DrawCommand for FluidGlassCommand {
    fn draw_region(&self) -> DrawRegion {
        if let Some(lens) = self.render.lens {
            DrawRegion::PaddedLocal(PaddingRect::uniform(Px((self.render.pixels_per_dp
                * (lens.shadow.offset.abs() + 3.0 * lens.shadow.blur).max(1.5))
            .ceil() as i32)))
        } else {
            DrawRegion::PaddedLocal(PaddingRect::uniform(Px::ZERO))
        }
    }

    fn sample_region(&self) -> Option<SampleRegion> {
        let ripple_padding =
            self.render.ripple_strength.unwrap_or_default().abs() * 3.0 * self.render.pixels_per_dp;
        let blur_padding = 3.0 * self.render.blur_sigma_px;
        let lens_padding = self.render.lens.map_or(0.0, |lens| {
            (lens.refraction.abs() + lens.inner_refraction.abs() + 6.0 * lens.dispersion.abs())
                .max(lens.shadow.offset.abs() + 3.0 * lens.shadow.blur)
                * self.render.pixels_per_dp
        });
        let padding = Px((blur_padding + ripple_padding + lens_padding + 2.0).ceil() as i32);
        if self.render.material_tone.is_some() {
            Some(SampleRegion::uniform_padding_local(padding))
        } else {
            Some(SampleRegion::uniform_padding_local(padding + Px(10)))
        }
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.render.opacity *= opacity.clamp(0.0, 1.0);
    }
}
