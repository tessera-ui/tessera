//! Layout modifiers for sizing, padding, and constraints.
//!
//! ## Usage
//!
//! Apply padding, sizing, or minimum touch target adjustments to component
//! subtrees.

use std::any::TypeId;

use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutModifier, LayoutModifierChild,
    LayoutModifierInput, LayoutResult, MeasurementError, ParentDataMap, ParentDataModifierNode,
    PlacementModifierNode, Px, PxPosition,
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

pub use tessera_ui::Insets as Padding;

fn resolve_axis_constraint(
    parent: AxisConstraint,
    override_axis: Option<AxisConstraint>,
    fill_parent_max: bool,
    enforce_incoming: bool,
) -> AxisConstraint {
    if fill_parent_max {
        return match parent.max {
            Some(max) => AxisConstraint::exact(max),
            None => parent,
        };
    }

    match override_axis {
        None => parent,
        Some(axis) if enforce_incoming => constrain_axis_to_parent(axis, parent),
        Some(axis) => axis.intersect(AxisConstraint::new(axis.min, parent.max)),
    }
}

fn constrain_axis_to_parent(axis: AxisConstraint, parent: AxisConstraint) -> AxisConstraint {
    AxisConstraint::new(
        parent.clamp(axis.min),
        axis.max.map(|max| parent.clamp(max)).or(parent.max),
    )
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct PaddingModifierNode {
    pub padding: Padding,
}

impl LayoutModifier for PaddingModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let left_px: Px = self.padding.left.into();
        let top_px: Px = self.padding.top.into();
        let right_px: Px = self.padding.right.into();
        let bottom_px: Px = self.padding.bottom.into();

        let parent_constraint = input.layout_input.parent_constraint();
        let constraint = Constraint::new(
            parent_constraint.width() - (left_px + right_px),
            parent_constraint.height() - (top_px + bottom_px),
        );
        let child_size = child.measure(&constraint)?;
        child.place(PxPosition::new(left_px, top_px));
        Ok(LayoutResult::new(ComputedData {
            width: child_size.width + left_px + right_px,
            height: child_size.height + top_px + bottom_px,
        }))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct OffsetModifierNode {
    pub x: Dp,
    pub y: Dp,
}

impl PlacementModifierNode for OffsetModifierNode {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position.offset(self.x.into(), self.y.into())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ConstraintModifierNode {
    pub width_override: Option<AxisConstraint>,
    pub height_override: Option<AxisConstraint>,
    pub fill_width: bool,
    pub fill_height: bool,
    pub enforce_incoming: bool,
}

impl LayoutModifier for ConstraintModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let parent_width = input.layout_input.parent_constraint().width();
        let parent_height = input.layout_input.parent_constraint().height();
        let constraint = Constraint::new(
            resolve_axis_constraint(
                parent_width,
                self.width_override,
                self.fill_width,
                self.enforce_incoming,
            ),
            resolve_axis_constraint(
                parent_height,
                self.height_override,
                self.fill_height,
                self.enforce_incoming,
            ),
        );
        let child_size = child.measure(&constraint)?;
        child.place(PxPosition::ZERO);
        Ok(LayoutResult::new(child_size))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct RequiredSizeModifierNode {
    pub width: Option<AxisConstraint>,
    pub height: Option<AxisConstraint>,
}

impl LayoutModifier for RequiredSizeModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        // Required size bypasses the incoming bounds: the declared size is
        // measured as-is, unlike `size` which clamps into the parent interval.
        let parent = input.layout_input.parent_constraint();
        let constraint = Constraint::new(
            self.width.unwrap_or_else(|| parent.width()),
            self.height.unwrap_or_else(|| parent.height()),
        );
        let child_size = child.measure(&constraint)?;
        child.place(PxPosition::ZERO);
        Ok(LayoutResult::new(child_size))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct DefaultMinSizeModifierNode {
    pub min_width: Dp,
    pub min_height: Dp,
}

impl LayoutModifier for DefaultMinSizeModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let parent = input.layout_input.parent_constraint();
        let width = default_min_axis(parent.width(), self.min_width);
        let height = default_min_axis(parent.height(), self.min_height);
        let child_size = child.measure(&Constraint::new(width, height))?;
        child.place(PxPosition::ZERO);
        Ok(LayoutResult::new(child_size))
    }
}

/// Applies a default minimum only when the incoming axis leaves the minimum
/// open, so an explicit size declared further out always wins.
fn default_min_axis(parent: AxisConstraint, default_min: Dp) -> AxisConstraint {
    if parent.min != Px::ZERO {
        return parent;
    }

    let default_min: Px = default_min.into();
    let min = parent.max.map_or(default_min, |max| default_min.min(max));
    AxisConstraint::new(min, parent.max)
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct MinimumInteractiveModifierNode;

impl LayoutModifier for MinimumInteractiveModifierNode {
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        const MIN_SIZE: Dp = Dp(48.0);
        let parent_constraint = Constraint::new(
            input.layout_input.parent_constraint().width(),
            input.layout_input.parent_constraint().height(),
        );
        let child_size = child.measure(&parent_constraint)?;
        if input.layout_input.is_intrinsic() {
            child.place(PxPosition::ZERO);
            return Ok(LayoutResult::new(child_size));
        }
        let min_px: Px = MIN_SIZE.into();
        let width = parent_constraint.width.clamp(child_size.width.max(min_px));
        let height = parent_constraint
            .height
            .clamp(child_size.height.max(min_px));
        let x = ((width - child_size.width) / 2).max(Px(0));
        let y = ((height - child_size.height) / 2).max(Px(0));
        child.place(PxPosition::new(x, y));
        Ok(LayoutResult::new(ComputedData { width, height }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Parent data carrying relative layout weight for row and column containers.
pub struct WeightParentData {
    /// Relative weight used by weighted parent layouts.
    pub weight: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct WeightParentDataModifierNode {
    pub weight: f32,
}

impl ParentDataModifierNode for WeightParentDataModifierNode {
    fn equivalent(&self, other: &dyn ParentDataModifierNode) -> bool {
        (other as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    fn apply_parent_data(&self, map: &mut ParentDataMap) {
        map.insert(
            TypeId::of::<WeightParentData>(),
            Box::new(WeightParentData {
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

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct AlignmentParentDataModifierNode {
    pub alignment: Alignment,
}

impl ParentDataModifierNode for AlignmentParentDataModifierNode {
    fn equivalent(&self, other: &dyn ParentDataModifierNode) -> bool {
        (other as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    fn apply_parent_data(&self, map: &mut ParentDataMap) {
        map.insert(
            TypeId::of::<AlignmentParentData>(),
            Box::new(AlignmentParentData {
                alignment: self.alignment,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{AxisConstraint, Px};

    use super::constrain_axis_to_parent;

    #[test]
    fn constrain_clamps_target_into_tight_parent() {
        let parent = AxisConstraint::exact(Px::new(100));

        assert_eq!(
            constrain_axis_to_parent(AxisConstraint::exact(Px::new(50)), parent),
            AxisConstraint::exact(Px::new(100))
        );
        assert_eq!(
            constrain_axis_to_parent(AxisConstraint::exact(Px::new(150)), parent),
            AxisConstraint::exact(Px::new(100))
        );
    }

    #[test]
    fn constrain_keeps_target_inside_loose_parent() {
        let parent = AxisConstraint::new(Px::ZERO, Some(Px::new(200)));

        assert_eq!(
            constrain_axis_to_parent(AxisConstraint::exact(Px::new(50)), parent),
            AxisConstraint::exact(Px::new(50))
        );
    }
}
