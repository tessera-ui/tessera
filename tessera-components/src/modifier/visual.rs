//! Visual modifiers for opacity, clipping, backgrounds, and borders.
//!
//! ## Usage
//!
//! Apply basic visual effects like alpha, clipping, and shape borders.

use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, PxPosition, PxSize, RenderInput, tessera,
};

use crate::{
    pipelines::shape::command::ShapeCommand,
    shape_def::{ResolvedShape, Shape},
};

#[tessera]
pub(crate) fn modifier_alpha<F>(alpha: f32, child: F)
where
    F: FnOnce(),
{
    layout(AlphaLayout { alpha });

    child();
}

#[tessera]
pub(crate) fn modifier_clip_to_bounds<F>(child: F)
where
    F: FnOnce(),
{
    layout(ClipLayout);

    child();
}

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

#[tessera]
pub(crate) fn modifier_background<F>(color: Color, shape: Shape, child: F)
where
    F: FnOnce(),
{
    layout(BackgroundLayout { color, shape });

    child();
}

#[tessera]
fn modifier_border_overlay(width: Dp, color: Color, shape: Shape) {
    layout(BorderOverlayLayout {
        width,
        color,
        shape,
    });
}

#[tessera]
pub(crate) fn modifier_border<F>(width: Dp, color: Color, shape: Shape, child: F)
where
    F: FnOnce(),
{
    layout(BorderLayout);

    child();
    modifier_border_overlay(width, color, shape);
}

#[derive(Clone, Copy, PartialEq)]
struct AlphaLayout {
    alpha: f32,
}

impl LayoutSpec for AlphaLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_alpha expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        output.place_child(child_id, PxPosition::ZERO);

        Ok(child_measurement)
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        metadata.opacity *= self.alpha;
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct ClipLayout;

impl LayoutSpec for ClipLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_clip_to_bounds expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        output.place_child(child_id, PxPosition::ZERO);
        Ok(child_measurement)
    }

    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().clips_children = true;
    }
}

#[derive(Clone, PartialEq)]
struct BackgroundLayout {
    color: Color,
    shape: Shape,
}

impl LayoutSpec for BackgroundLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_background expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        output.place_child(child_id, PxPosition::ZERO);
        Ok(child_measurement)
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        let size = metadata
            .computed_data
            .expect("modifier_background must have computed size before record");
        metadata
            .fragment_mut()
            .push_draw_command(shape_background_command(
                self.color,
                self.shape,
                size.into(),
            ));
    }
}

#[derive(Clone, PartialEq)]
struct BorderOverlayLayout {
    width: Dp,
    color: Color,
    shape: Shape,
}

impl LayoutSpec for BorderOverlayLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let size = ComputedData {
            width: input.parent_constraint().width().resolve(),
            height: input.parent_constraint().height().resolve(),
        };
        Ok(size)
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        let size = metadata
            .computed_data
            .expect("modifier_border_overlay must have computed size before record");
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

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct BorderLayout;

impl LayoutSpec for BorderLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let content_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_border expects exactly two children");
        let overlay_id = input
            .children_ids()
            .get(1)
            .copied()
            .expect("modifier_border expects exactly two children");
        let child_measurement = input.measure_child_in_parent_constraint(content_id)?;
        output.place_child(content_id, PxPosition::ZERO);
        let overlay_constraint = Constraint::new(
            DimensionValue::Fixed(child_measurement.width),
            DimensionValue::Fixed(child_measurement.height),
        );
        let _ = input.measure_child(overlay_id, &overlay_constraint)?;
        output.place_child(overlay_id, PxPosition::ZERO);
        Ok(child_measurement)
    }
}
