//! Layout modifiers for sizing, padding, and constraints.
//!
//! ## Usage
//!
//! Apply padding, sizing, or minimum touch target adjustments to component
//! subtrees.

use std::{any::TypeId, sync::Arc};

use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutModifierChild, LayoutModifierInput,
    LayoutModifierNode, LayoutModifierOutput, LayoutOutput, MeasurementError, ParentDataMap,
    ParentDataModifierNode, Px, PxPosition,
};

use crate::alignment::Alignment;

/// Controls whether minimum interactive size wrappers are enforced.
#[derive(Clone, PartialEq, Copy, Debug)]
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

pub(crate) fn shrink_dimension(dimension: AxisConstraint, before: Px, after: Px) -> AxisConstraint {
    let subtract = before + after;
    let min = (dimension.min - subtract).max(Px::ZERO);
    let max = dimension.max.map(|value| (value - subtract).max(Px::ZERO));
    AxisConstraint::new(min, max)
}

fn resolve_axis_constraint(
    parent: AxisConstraint,
    override_axis: Option<AxisConstraint>,
    fill_parent_max: bool,
) -> AxisConstraint {
    if fill_parent_max {
        return match parent.max {
            Some(max) => AxisConstraint::exact(max),
            None => parent,
        };
    }

    match override_axis {
        None => parent,
        Some(axis) => AxisConstraint::new(
            axis.min,
            match (axis.max, parent.max) {
                (Some(lhs), Some(rhs)) => Some(lhs.min(rhs)),
                (Some(lhs), None) => Some(lhs),
                (None, Some(rhs)) => Some(rhs),
                (None, None) => None,
            },
        ),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PaddingModifierNode {
    pub padding: Padding,
}

impl LayoutModifierNode for PaddingModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
        output: &mut LayoutOutput<'_>,
    ) -> Result<LayoutModifierOutput, MeasurementError> {
        let left_px: Px = self.padding.left.into();
        let top_px: Px = self.padding.top.into();
        let right_px: Px = self.padding.right.into();
        let bottom_px: Px = self.padding.bottom.into();

        let parent_constraint = Constraint::new(
            input.layout_input.parent_constraint().width(),
            input.layout_input.parent_constraint().height(),
        );
        let constraint = Constraint::new(
            shrink_dimension(parent_constraint.width, left_px, right_px),
            shrink_dimension(parent_constraint.height, top_px, bottom_px),
        );
        let child_size = child.measure(&constraint)?;
        child.place(PxPosition::new(left_px, top_px), output);
        Ok(LayoutModifierOutput {
            size: ComputedData {
                width: child_size.width + left_px + right_px,
                height: child_size.height + top_px + bottom_px,
            },
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct OffsetModifierNode {
    pub x: Dp,
    pub y: Dp,
}

impl LayoutModifierNode for OffsetModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
        output: &mut LayoutOutput<'_>,
    ) -> Result<LayoutModifierOutput, MeasurementError> {
        let parent_constraint = Constraint::new(
            input.layout_input.parent_constraint().width(),
            input.layout_input.parent_constraint().height(),
        );
        let child_size = child.measure(&parent_constraint)?;
        child.place(PxPosition::new(self.x.into(), self.y.into()), output);
        Ok(LayoutModifierOutput { size: child_size })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ConstraintModifierNode {
    pub width_override: Option<AxisConstraint>,
    pub height_override: Option<AxisConstraint>,
    pub fill_width: bool,
    pub fill_height: bool,
}

impl LayoutModifierNode for ConstraintModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
        output: &mut LayoutOutput<'_>,
    ) -> Result<LayoutModifierOutput, MeasurementError> {
        let parent_width = input.layout_input.parent_constraint().width();
        let parent_height = input.layout_input.parent_constraint().height();
        let constraint = Constraint::new(
            resolve_axis_constraint(parent_width, self.width_override, self.fill_width),
            resolve_axis_constraint(parent_height, self.height_override, self.fill_height),
        );
        let child_size = child.measure(&constraint)?;
        child.place(PxPosition::ZERO, output);
        Ok(LayoutModifierOutput { size: child_size })
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct MinimumInteractiveModifierNode;

impl LayoutModifierNode for MinimumInteractiveModifierNode {
    fn measure(
        &self,
        _input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
        output: &mut LayoutOutput<'_>,
    ) -> Result<LayoutModifierOutput, MeasurementError> {
        const MIN_SIZE: Dp = Dp(48.0);
        let child_size = child.measure(&Constraint::NONE)?;
        let min_px: Px = MIN_SIZE.into();
        let width = child_size.width.max(min_px);
        let height = child_size.height.max(min_px);
        let x = ((width - child_size.width) / 2).max(Px(0));
        let y = ((height - child_size.height) / 2).max(Px(0));
        child.place(PxPosition::new(x, y), output);
        Ok(LayoutModifierOutput {
            size: ComputedData { width, height },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Parent data carrying relative layout weight for row and column containers.
pub struct WeightParentData {
    /// Relative weight used by weighted parent layouts.
    pub weight: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct WeightParentDataModifierNode {
    pub weight: f32,
}

impl ParentDataModifierNode for WeightParentDataModifierNode {
    fn apply_parent_data(&self, map: &mut ParentDataMap) {
        map.insert(
            TypeId::of::<WeightParentData>(),
            Arc::new(WeightParentData {
                weight: self.weight,
            }),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Parent data carrying boxed-layout alignment overrides.
pub struct AlignmentParentData {
    /// Alignment requested by the child for layered boxed layouts.
    pub alignment: Alignment,
}

#[derive(Clone, Copy)]
pub(crate) struct AlignmentParentDataModifierNode {
    pub alignment: Alignment,
}

impl ParentDataModifierNode for AlignmentParentDataModifierNode {
    fn apply_parent_data(&self, map: &mut ParentDataMap) {
        map.insert(
            TypeId::of::<AlignmentParentData>(),
            Arc::new(AlignmentParentData {
                alignment: self.alignment,
            }),
        );
    }
}
