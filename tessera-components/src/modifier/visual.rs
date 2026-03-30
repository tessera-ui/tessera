//! Visual modifiers for opacity, clipping, backgrounds, and borders.
//!
//! ## Usage
//!
//! Apply basic visual effects like alpha, clipping, and shape borders.

use tessera_ui::{Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, PxSize};

use crate::{
    pipelines::shape::command::ShapeCommand,
    shape_def::{ResolvedShape, Shape},
};

fn shape_background_command(color: Color, shape: Shape, size: PxSize) -> ShapeCommand {
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::Rect {
            color,
            corner_radii,
            corner_g2,
        },
        ResolvedShape::Ellipse => ShapeCommand::Ellipse { color },
    }
}

fn shape_border_command(color: Color, width: Dp, shape: Shape, size: PxSize) -> ShapeCommand {
    let border_width = width.to_pixels_f32();
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::OutlinedRect {
            color,
            corner_radii,
            corner_g2,
            border_width,
        },
        ResolvedShape::Ellipse => ShapeCommand::OutlinedEllipse {
            color,
            border_width,
        },
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AlphaModifierNode {
    pub alpha: f32,
}

impl DrawModifierNode for AlphaModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_>, content: &mut dyn DrawModifierContent) {
        let mut metadata = ctx.render_input.metadata_mut();
        metadata.multiply_opacity(self.alpha);
        drop(metadata);
        content.draw(ctx.render_input);
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ClipModifierNode;

impl DrawModifierNode for ClipModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_>, content: &mut dyn DrawModifierContent) {
        ctx.render_input.metadata_mut().set_clips_children(true);
        content.draw(ctx.render_input);
    }
}

#[derive(Clone)]
pub(crate) struct BackgroundModifierNode {
    pub color: Color,
    pub shape: Shape,
}

impl DrawModifierNode for BackgroundModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_>, content: &mut dyn DrawModifierContent) {
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("background modifier must have computed size before record");
        metadata
            .fragment_mut()
            .push_draw_command(shape_background_command(
                self.color,
                self.shape,
                size.into(),
            ));
        drop(metadata);
        content.draw(ctx.render_input);
    }
}

#[derive(Clone)]
pub(crate) struct BorderModifierNode {
    pub width: Dp,
    pub color: Color,
    pub shape: Shape,
}

impl DrawModifierNode for BorderModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_>, content: &mut dyn DrawModifierContent) {
        content.draw(ctx.render_input);
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("border modifier must have computed size before record");
        metadata
            .fragment_mut()
            .push_draw_command(shape_border_command(
                self.color,
                self.width,
                self.shape,
                size.into(),
            ));
    }
}
