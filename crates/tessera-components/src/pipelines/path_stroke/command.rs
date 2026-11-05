use std::sync::Arc;

use glam::Vec2;
use tessera_ui::{Color, DrawCommand};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PathStrokeCommand {
    pub points: Arc<[Vec2]>,
    pub color: Color,
    pub stroke: f32,
    pub round: bool,
    pub closed: bool,
}

impl DrawCommand for PathStrokeCommand {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}
