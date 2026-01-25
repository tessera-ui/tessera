//! Layout modifiers for sizing, padding, and constraints.
//!
//! ## Usage
//!
//! Apply padding, sizing, or minimum touch target adjustments to component
//! subtrees.

use tessera_ui::{
    ComputedData, Constraint, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, Px, PxPosition, tessera,
};

/// Controls whether minimum interactive size wrappers are enforced.
#[derive(Clone, Copy, Debug)]
pub struct MinimumInteractiveComponentEnforcement {
    /// When true, `minimum_interactive_component_size` expands to the minimum
    /// size.
    pub enabled: bool,
}

impl Default for MinimumInteractiveComponentEnforcement {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Padding values in density-independent pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Padding {
    /// Left padding.
    pub left: Dp,
    /// Top padding.
    pub top: Dp,
    /// Right padding.
    pub right: Dp,
    /// Bottom padding.
    pub bottom: Dp,
}

impl Padding {
    /// Creates padding with explicit edges.
    pub const fn new(left: Dp, top: Dp, right: Dp, bottom: Dp) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Creates symmetric padding on all edges.
    pub const fn all(value: Dp) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// Creates symmetric padding for horizontal and vertical edges.
    pub const fn symmetric(horizontal: Dp, vertical: Dp) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    /// Creates padding with only horizontal edges.
    pub const fn horizontal(value: Dp) -> Self {
        Self::symmetric(value, Dp(0.0))
    }

    /// Creates padding with only vertical edges.
    pub const fn vertical(value: Dp) -> Self {
        Self::symmetric(Dp(0.0), value)
    }

    /// Creates padding with only the left edge.
    pub const fn left(value: Dp) -> Self {
        Self {
            left: value,
            top: Dp(0.0),
            right: Dp(0.0),
            bottom: Dp(0.0),
        }
    }

    /// Creates padding with only the top edge.
    pub const fn top(value: Dp) -> Self {
        Self {
            left: Dp(0.0),
            top: value,
            right: Dp(0.0),
            bottom: Dp(0.0),
        }
    }

    /// Creates padding with only the right edge.
    pub const fn right(value: Dp) -> Self {
        Self {
            left: Dp(0.0),
            top: Dp(0.0),
            right: value,
            bottom: Dp(0.0),
        }
    }

    /// Creates padding with only the bottom edge.
    pub const fn bottom(value: Dp) -> Self {
        Self {
            left: Dp(0.0),
            top: Dp(0.0),
            right: Dp(0.0),
            bottom: value,
        }
    }
}

fn subtract_opt_px(value: Option<Px>, subtract: Px) -> Option<Px> {
    value.map(|v| (v - subtract).max(Px(0)))
}

pub(crate) fn shrink_dimension(dimension: DimensionValue, before: Px, after: Px) -> DimensionValue {
    let subtract = before + after;
    match dimension {
        DimensionValue::Fixed(value) => DimensionValue::Fixed((value - subtract).max(Px(0))),
        DimensionValue::Wrap { min, max } => DimensionValue::Wrap {
            min: subtract_opt_px(min, subtract),
            max: subtract_opt_px(max, subtract),
        },
        DimensionValue::Fill { min, max } => DimensionValue::Fill {
            min: subtract_opt_px(min, subtract),
            max: subtract_opt_px(max, subtract),
        },
    }
}

#[tessera]
pub(crate) fn modifier_padding<F>(padding: Padding, child: F)
where
    F: FnOnce(),
{
    layout(PaddingLayout { padding });

    child();
}

#[tessera]
pub(crate) fn modifier_offset<F>(x: Dp, y: Dp, child: F)
where
    F: FnOnce(),
{
    layout(OffsetLayout { x, y });

    child();
}

#[tessera]
pub(crate) fn modifier_constraints<F>(
    width_override: Option<DimensionValue>,
    height_override: Option<DimensionValue>,
    child: F,
) where
    F: FnOnce(),
{
    layout(ConstraintLayout {
        width_override,
        height_override,
    });

    child();
}

#[tessera]
pub(crate) fn modifier_minimum_interactive_size<F>(child: F)
where
    F: FnOnce(),
{
    layout(MinimumInteractiveLayout);

    child();
}

#[derive(Clone, Copy, PartialEq)]
struct PaddingLayout {
    padding: Padding,
}

impl LayoutSpec for PaddingLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_padding expects exactly one child");

        let left_px: Px = self.padding.left.into();
        let top_px: Px = self.padding.top.into();
        let right_px: Px = self.padding.right.into();
        let bottom_px: Px = self.padding.bottom.into();

        let parent_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let constraint = Constraint::new(
            shrink_dimension(parent_constraint.width, left_px, right_px),
            shrink_dimension(parent_constraint.height, top_px, bottom_px),
        );

        let child_measurement = input.measure_child(child_id, &constraint)?;
        let content_width = child_measurement.width + left_px + right_px;
        let content_height = child_measurement.height + top_px + bottom_px;
        output.place_child(child_id, PxPosition::new(left_px, top_px));

        Ok(ComputedData {
            width: content_width,
            height: content_height,
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
struct OffsetLayout {
    x: Dp,
    y: Dp,
}

impl LayoutSpec for OffsetLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_offset expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        output.place_child(child_id, PxPosition::new(self.x.into(), self.y.into()));

        Ok(child_measurement)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ConstraintLayout {
    width_override: Option<DimensionValue>,
    height_override: Option<DimensionValue>,
}

impl LayoutSpec for ConstraintLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_constraints expects exactly one child");

        let parent_width = input.parent_constraint().width();
        let parent_height = input.parent_constraint().height();
        let constraint = Constraint::new(
            self.width_override.unwrap_or(parent_width),
            self.height_override.unwrap_or(parent_height),
        )
        .merge(input.parent_constraint());

        let child_measurement = input.measure_child(child_id, &constraint)?;
        output.place_child(child_id, PxPosition::ZERO);

        Ok(child_measurement)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct MinimumInteractiveLayout;

impl LayoutSpec for MinimumInteractiveLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        const MIN_SIZE: Dp = Dp(48.0);
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_minimum_interactive_size expects exactly one child");

        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;

        let min_px: Px = MIN_SIZE.into();
        let content_width = child_measurement.width.max(min_px);
        let content_height = child_measurement.height.max(min_px);

        let x = ((content_width - child_measurement.width) / 2).max(Px(0));
        let y = ((content_height - child_measurement.height) / 2).max(Px(0));
        output.place_child(child_id, PxPosition::new(x, y));

        Ok(ComputedData {
            width: content_width,
            height: content_height,
        })
    }
}
