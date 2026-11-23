use std::ops::Sub;

use crate::{Dp, Px};

/// Minimum and optional maximum size for one layout axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AxisConstraint {
    /// Smallest allowed size.
    pub min: Px,
    /// Largest allowed size, if bounded.
    pub max: Option<Px>,
}

impl AxisConstraint {
    /// A zero minimum with no upper bound.
    pub const NONE: Self = Self {
        min: Px::ZERO,
        max: None,
    };

    /// Creates a range, raising `max` to `min` when the values conflict.
    pub fn new(min: Px, max: Option<Px>) -> Self {
        let normalized_max = match max {
            Some(value) if value < min => Some(min),
            other => other,
        };
        Self {
            min,
            max: normalized_max,
        }
    }

    /// Requires exactly `size`.
    pub const fn exact(size: Px) -> Self {
        Self {
            min: size,
            max: Some(size),
        }
    }

    /// Sets a lower bound without an upper bound.
    pub const fn at_least(min: Px) -> Self {
        Self { min, max: None }
    }

    /// Sets an upper bound with a zero minimum.
    pub const fn at_most(max: Px) -> Self {
        Self {
            min: Px::ZERO,
            max: Some(max),
        }
    }

    /// Returns the lower bound.
    pub const fn resolve_min(self) -> Px {
        self.min
    }

    /// Returns the upper bound, if present.
    pub const fn resolve_max(self) -> Option<Px> {
        self.max
    }

    /// Combines the ranges, raising the maximum to the larger minimum if they do not overlap.
    pub fn intersect(self, parent: Self) -> Self {
        let min = self.min.max(parent.min);
        let max = match (self.max, parent.max) {
            (Some(lhs), Some(rhs)) => Some(lhs.min(rhs)),
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            (None, None) => None,
        };
        Self::new(min, max)
    }

    /// Resets the minimum to zero while preserving the maximum.
    pub const fn without_min(self) -> Self {
        Self {
            min: Px::ZERO,
            max: self.max,
        }
    }

    /// Clamps `value` to this range.
    pub fn clamp(self, value: Px) -> Px {
        let mut value = value.max(self.min);
        if let Some(max) = self.max {
            value = value.min(max);
        }
        value
    }
}

impl Default for AxisConstraint {
    fn default() -> Self {
        Self::NONE
    }
}

impl From<Px> for AxisConstraint {
    fn from(value: Px) -> Self {
        Self::exact(value)
    }
}

impl From<Dp> for AxisConstraint {
    fn from(value: Dp) -> Self {
        Self::exact(value.into())
    }
}

impl Sub<Px> for AxisConstraint {
    type Output = AxisConstraint;

    fn sub(self, rhs: Px) -> Self::Output {
        let min = (self.min - rhs).max(Px::ZERO);
        let max = self.max.map(|value| (value - rhs).max(Px::ZERO));
        Self::new(min, max)
    }
}

impl std::ops::Add<Px> for AxisConstraint {
    type Output = AxisConstraint;

    fn add(self, rhs: Px) -> Self::Output {
        let min = self.min + rhs;
        let max = self.max.map(|value| value + rhs);
        Self::new(min, max)
    }
}

impl std::ops::AddAssign<Px> for AxisConstraint {
    fn add_assign(&mut self, rhs: Px) {
        *self = *self + rhs;
    }
}

impl std::ops::SubAssign<Px> for AxisConstraint {
    fn sub_assign(&mut self, rhs: Px) {
        *self = *self - rhs;
    }
}

/// Borrowed view of the constraints passed by a parent layout.
#[derive(Clone, Copy, Debug)]
pub struct ParentConstraint<'a>(&'a Constraint);

impl<'a> ParentConstraint<'a> {
    pub(crate) fn new(constraint: &'a Constraint) -> Self {
        Self(constraint)
    }

    /// Returns the width constraint.
    pub const fn width(self) -> AxisConstraint {
        self.0.width
    }

    /// Returns the height constraint.
    pub const fn height(self) -> AxisConstraint {
        self.0.height
    }

    /// Returns the underlying constraints.
    pub const fn as_ref(self) -> &'a Constraint {
        self.0
    }

    /// Returns constraints with both minimums set to zero.
    pub const fn without_min(self) -> Constraint {
        Constraint {
            width: self.0.width.without_min(),
            height: self.0.height.without_min(),
        }
    }
}

/// Width and height constraints supplied to layout measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraint {
    /// Allowed width range.
    pub width: AxisConstraint,
    /// Allowed height range.
    pub height: AxisConstraint,
}

impl Constraint {
    /// Unbounded width and height.
    pub const NONE: Self = Self {
        width: AxisConstraint::NONE,
        height: AxisConstraint::NONE,
    };

    /// Creates constraints from ranges or exact `Dp`/`Px` values.
    pub fn new(width: impl Into<AxisConstraint>, height: impl Into<AxisConstraint>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }

    /// Requires exactly the given width and height.
    pub fn exact(width: Px, height: Px) -> Self {
        Self {
            width: AxisConstraint::exact(width),
            height: AxisConstraint::exact(height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_new_clamps_max_to_min() {
        let axis = AxisConstraint::new(Px(40), Some(Px(20)));
        assert_eq!(axis.min, Px(40));
        assert_eq!(axis.max, Some(Px(40)));
    }

    #[test]
    fn axis_intersect_returns_interval_overlap() {
        let parent = Constraint::new(
            AxisConstraint::new(Px(20), Some(Px(100))),
            AxisConstraint::new(Px(10), Some(Px(80))),
        );
        let child = Constraint::new(
            AxisConstraint::new(Px(30), Some(Px(120))),
            AxisConstraint::new(Px(0), Some(Px(40))),
        );

        let width = child.width.intersect(parent.width);
        let height = child.height.intersect(parent.height);

        assert_eq!(width, AxisConstraint::new(Px(30), Some(Px(100))));
        assert_eq!(height, AxisConstraint::new(Px(10), Some(Px(40))));
    }

    #[test]
    fn axis_intersect_keeps_unbounded_max_when_parent_is_unbounded() {
        let parent = Constraint::new(AxisConstraint::at_least(Px(50)), AxisConstraint::NONE);
        let child = Constraint::new(
            AxisConstraint::new(Px(20), None),
            AxisConstraint::new(Px(10), Some(Px(30))),
        );

        let width = child.width.intersect(parent.width);
        let height = child.height.intersect(parent.height);

        assert_eq!(width, AxisConstraint::at_least(Px(50)));
        assert_eq!(height, AxisConstraint::new(Px(10), Some(Px(30))));
    }

    #[test]
    fn arithmetic_adjusts_intervals() {
        let mut axis = AxisConstraint::new(Px(20), Some(Px(60)));
        axis -= Px(5);
        assert_eq!(axis, AxisConstraint::new(Px(15), Some(Px(55))));
        axis += Px(10);
        assert_eq!(axis, AxisConstraint::new(Px(25), Some(Px(65))));
    }
}
