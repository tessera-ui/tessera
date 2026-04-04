//! # Layout Constraint System
//!
//! This module defines Tessera's interval-based layout constraints.
//!
//! A parent layout can only bound a child's size. Exact size is represented as
//! a tight interval where `min == max`.

use std::ops::Sub;

use crate::{Dp, Px};

/// A single-axis layout constraint expressed as an allowed interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AxisConstraint {
    /// The minimum allowed size on this axis.
    pub min: Px,
    /// The maximum allowed size on this axis.
    ///
    /// When `None`, the axis is unbounded above.
    pub max: Option<Px>,
}

impl AxisConstraint {
    /// An unconstrained axis with minimum `0`.
    pub const NONE: Self = Self {
        min: Px::ZERO,
        max: None,
    };

    /// Creates a new interval constraint.
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

    /// Creates a tight axis constraint.
    pub const fn exact(size: Px) -> Self {
        Self {
            min: size,
            max: Some(size),
        }
    }

    /// Creates an axis with only a lower bound.
    pub const fn at_least(min: Px) -> Self {
        Self { min, max: None }
    }

    /// Creates an axis with only an upper bound.
    pub const fn at_most(max: Px) -> Self {
        Self {
            min: Px::ZERO,
            max: Some(max),
        }
    }

    /// Returns the preferred minimum size for this axis.
    pub const fn resolve_min(self) -> Px {
        self.min
    }

    /// Returns the upper bound for this axis, if present.
    pub const fn resolve_max(self) -> Option<Px> {
        self.max
    }

    /// Returns the intersection of two axis constraints.
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

    /// Returns a version of this axis with the lower bound cleared.
    pub const fn without_min(self) -> Self {
        Self {
            min: Px::ZERO,
            max: self.max,
        }
    }

    /// Clamps a measured size into this interval.
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

/// The constraint inherited from a parent node during measurement.
#[derive(Clone, Copy, Debug)]
pub struct ParentConstraint<'a>(&'a Constraint);

impl<'a> ParentConstraint<'a> {
    pub(crate) fn new(constraint: &'a Constraint) -> Self {
        Self(constraint)
    }

    /// Returns the inherited width constraint.
    pub const fn width(self) -> AxisConstraint {
        self.0.width
    }

    /// Returns the inherited height constraint.
    pub const fn height(self) -> AxisConstraint {
        self.0.height
    }

    /// Returns a reference to the underlying constraint.
    pub const fn as_ref(self) -> &'a Constraint {
        self.0
    }
}

/// A two-dimensional interval constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraint {
    /// The width interval.
    pub width: AxisConstraint,
    /// The height interval.
    pub height: AxisConstraint,
}

impl Constraint {
    /// An unconstrained width/height interval.
    pub const NONE: Self = Self {
        width: AxisConstraint::NONE,
        height: AxisConstraint::NONE,
    };

    /// Creates a new 2D constraint from width and height intervals.
    pub fn new(width: impl Into<AxisConstraint>, height: impl Into<AxisConstraint>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }

    /// Creates a tight 2D constraint.
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
    fn axis_exact_is_tight_interval() {
        let axis = AxisConstraint::exact(Px(100));
        assert_eq!(axis.min, Px(100));
        assert_eq!(axis.max, Some(Px(100)));
    }

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
