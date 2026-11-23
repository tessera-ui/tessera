pub mod pipeline;

use tessera_ui::{
    Color, Px,
    render::{DrawCommand, SampleRegion},
};

use crate::shape_def::Shape;

#[derive(Clone, PartialEq)]
pub(crate) struct FluidGlassRenderArgs {
    pub(crate) tint_color: Color,
    pub(crate) shape: Shape,
    pub(crate) noise_amount: f32,
    pub(crate) noise_scale: f32,
    pub(crate) time: f32,
    pub(crate) ripple_center: Option<[f32; 2]>,
    pub(crate) ripple_radius: Option<f32>,
    pub(crate) ripple_alpha: Option<f32>,
    pub(crate) ripple_strength: Option<f32>,
    pub(crate) border_width: Px,
    pub(crate) material_tone: Option<[f32; 4]>,
    pub(crate) blur_sigma_px: f32,
    pub(crate) pixels_per_dp: f32,
    pub(crate) opacity: f32,
    pub(crate) contrast: Option<f32>,
    pub(crate) dark: bool,
    pub(crate) feedback: f32,
}

#[derive(Clone, PartialEq)]
pub(crate) struct FluidGlassCommand {
    pub(crate) render: FluidGlassRenderArgs,
}

impl DrawCommand for FluidGlassCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        let ripple_padding =
            self.render.ripple_strength.unwrap_or_default().abs() * 3.0 * self.render.pixels_per_dp;
        let blur_padding = 3.0 * self.render.blur_sigma_px;
        let padding = Px((blur_padding + ripple_padding + 2.0).ceil() as i32);
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
