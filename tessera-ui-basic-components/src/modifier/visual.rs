//! Visual modifiers for opacity, clipping, backgrounds, and borders.
//!
//! ## Usage
//!
//! Add drawing effects like alpha, clipping, and shape-based backgrounds or
//! borders.

use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, Px, PxPosition, PxSize, tessera,
};

use crate::{
    pipelines::shape::command::ShapeCommand,
    shape_def::{ResolvedShape, Shape},
};

use super::layout::resolve_dimension;

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

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");

        input.place_child(child_id, PxPosition::ZERO);
        input.multiply_opacity(alpha);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
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

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");

        input.place_child(child_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
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

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");
        let size = PxSize::new(final_width, final_height);

        input
            .metadata_mut()
            .push_draw_command(shape_background_command(color, shape, size));

        input.place_child(child_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_border_overlay(width: Dp, color: Color, shape: Shape) {
    measure(Box::new(move |input| {
        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let final_width = resolve_dimension(parent_constraint.width, Px(0), "width");
        let final_height = resolve_dimension(parent_constraint.height, Px(0), "height");
        let size = PxSize::new(final_width, final_height);

        input
            .metadata_mut()
            .push_draw_command(shape_border_command(color, width, shape, size));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
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

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(content_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&content_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");

        input.place_child(content_id, PxPosition::ZERO);

        let overlay_constraint = Constraint::new(
            DimensionValue::Fixed(final_width),
            DimensionValue::Fixed(final_height),
        );
        let overlay_measurements =
            input.measure_children(vec![(overlay_id, overlay_constraint)])?;
        overlay_measurements
            .get(&overlay_id)
            .expect("Overlay measurement missing");

        input.place_child(overlay_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
    modifier_border_overlay(width, color, shape);
}
