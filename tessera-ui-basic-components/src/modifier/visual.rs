//! Visual modifiers for opacity, clipping, backgrounds, and borders.
//!
//! ## Usage
//!
//! Add drawing effects like alpha, clipping, and shape-based backgrounds or
//! borders.

use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, PxPosition, PxSize, tessera,
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
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_alpha expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        input.place_child(child_id, PxPosition::ZERO);
        input.multiply_opacity(alpha);

        Ok(child_measurement)
    }));

    child();
}

#[tessera]
pub(crate) fn modifier_clip_to_bounds<F>(child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        input.enable_clipping();
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_clip_to_bounds expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        input.place_child(child_id, PxPosition::ZERO);
        Ok(child_measurement)
    }));

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
            shadow: None,
        },
        ResolvedShape::Ellipse => ShapeCommand::Ellipse {
            color,
            shadow: None,
        },
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
            shadow: None,
            border_width,
        },
        ResolvedShape::Ellipse => ShapeCommand::OutlinedEllipse {
            color,
            shadow: None,
            border_width,
        },
    }
}

#[tessera]
pub(crate) fn modifier_background<F>(color: Color, shape: Shape, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_background expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        input
            .metadata_mut()
            .push_draw_command(shape_background_command(
                color,
                shape,
                child_measurement.into(),
            ));
        input.place_child(child_id, PxPosition::ZERO);
        Ok(child_measurement)
    }));

    child();
}

#[tessera]
fn modifier_border_overlay(width: Dp, color: Color, shape: Shape) {
    measure(Box::new(move |input| {
        let size = ComputedData {
            width: input.parent_constraint.width().resolve(),
            height: input.parent_constraint.height().resolve(),
        };

        input.metadata_mut().push_draw_command(shape_border_command(
            color,
            width,
            shape,
            size.into(),
        ));

        Ok(size)
    }));
}

#[tessera]
pub(crate) fn modifier_border<F>(width: Dp, color: Color, shape: Shape, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let content_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_border expects exactly two children");
        let overlay_id = input
            .children_ids
            .get(1)
            .copied()
            .expect("modifier_border expects exactly two children");
        let child_measurement = input.measure_child_in_parent_constraint(content_id)?;
        input.place_child(content_id, PxPosition::ZERO);
        let overlay_constraint = Constraint::new(
            DimensionValue::Fixed(child_measurement.width),
            DimensionValue::Fixed(child_measurement.height),
        );
        let _ = input.measure_child(overlay_id, &overlay_constraint)?;
        input.place_child(overlay_id, PxPosition::ZERO);
        Ok(child_measurement)
    }));

    child();
    modifier_border_overlay(width, color, shape);
}
