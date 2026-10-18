use tessera_ui::{Color, DrawCommand};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LinearWaveCommand {
    pub color: Color,
    pub start: f32,
    pub end: f32,
    pub stroke: f32,
    pub wavelength: f32,
    pub control_height: f32,
    pub shift: f32,
    pub round: bool,
    pub reverse: bool,
}

impl DrawCommand for LinearWaveCommand {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}
