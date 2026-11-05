use tessera_ui::{Color, DrawCommand};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LoadingShapeCommand {
    pub sequence: usize,
    pub progress: f32,
    pub rotation: f32,
    pub color: Color,
}

impl DrawCommand for LoadingShapeCommand {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}
