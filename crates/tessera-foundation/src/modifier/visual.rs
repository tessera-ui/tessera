use tessera_ui::{DrawModifierContent, DrawModifierContext, DrawModifierNode, render::ClipShape};

use crate::shape_def::{ResolvedShape, Shape};

#[derive(Clone, Copy)]
pub(crate) struct AlphaModifierNode {
    pub alpha: f32,
}

impl DrawModifierNode for AlphaModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        {
            let mut metadata = ctx.render_input.metadata_mut();
            metadata.multiply_opacity(self.alpha);
        }
        content.draw(ctx.render_input);
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ClipModifierNode;

impl DrawModifierNode for ClipModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        ctx.render_input.metadata_mut().set_clips_children(true);
        content.draw(ctx.render_input);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ShapeClipModifierNode {
    pub shape: Shape,
}

impl DrawModifierNode for ShapeClipModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("shape clip must have a measured size");
        let shape = match self.shape.resolve_for_size(size.into()) {
            ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } => ClipShape::RoundedRectangle {
                radii: corner_radii,
                exponents: corner_g2,
            },
            ResolvedShape::Ellipse => ClipShape::Ellipse,
        };
        metadata.set_clip_shape(shape);
        content.draw(ctx.render_input);
    }
}
