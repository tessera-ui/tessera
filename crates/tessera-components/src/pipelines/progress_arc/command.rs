use tessera_ui::{Color, DrawCommand};

/// Stroke cap used for arc ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressArcCap {
    /// Rounded stroke ends.
    Round,
    /// Flat stroke ends.
    Butt,
}

/// Draw command for a circular arc stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressArcCommand {
    /// Stroke color.
    pub color: Color,
    /// Stroke width in physical pixels.
    pub stroke_width_px: f32,
    /// Start angle in degrees, where 0° is at 3 o'clock.
    pub start_angle_degrees: f32,
    /// Sweep angle in degrees, in the clockwise direction.
    pub sweep_angle_degrees: f32,
    /// Stroke cap applied to arc ends.
    pub cap: ProgressArcCap,
}

impl DrawCommand for ProgressArcCommand {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}
