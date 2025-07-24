//! # Layout Constraint System
//!
//! This module provides the core constraint system for Tessera's layout engine.
//! It defines how components specify their sizing requirements and how these
//! constraints are resolved in a component hierarchy.
//!
//! ## Overview
//!
//! The constraint system is built around two main concepts:
//!
//! - **[`DimensionValue`]**: Specifies how a single dimension (width or height) should be calculated
//! - **[`Constraint`]**: Combines width and height dimension values for complete layout specification
//!
//! ## Dimension Types
//!
//! There are three fundamental ways a component can specify its size:
//!
//! ### Fixed
//! The component has a specific, unchanging size:
//! ```
//! # use tessera_ui::Px;
//! # use tessera_ui::DimensionValue;
//! let fixed_width = DimensionValue::Fixed(Px(100));
//! ```
//!
//! ### Wrap
//! The component sizes itself to fit its content, with optional bounds:
//! ```
//! # use tessera_ui::Px;
//! # use tessera_ui::DimensionValue;
//! // Wrap content with no limits
//! let wrap_content = DimensionValue::Wrap { min: None, max: None };
//!
//! // Wrap content but ensure at least 50px wide
//! let wrap_with_min = DimensionValue::Wrap { min: Some(Px(50)), max: None };
//!
//! // Wrap content but never exceed 200px
//! let wrap_with_max = DimensionValue::Wrap { min: None, max: Some(Px(200)) };
//!
//! // Wrap content within bounds
//! let wrap_bounded = DimensionValue::Wrap {
//!     min: Some(Px(50)),
//!     max: Some(Px(200))
//! };
//! ```
//!
//! ### Fill
//! The component expands to fill available space, with optional bounds:
//! ```
//! # use tessera_ui::Px;
//! # use tessera_ui::DimensionValue;
//! // Fill all available space
//! let fill_all = DimensionValue::Fill { min: None, max: None };
//!
//! // Fill space but ensure at least 100px
//! let fill_with_min = DimensionValue::Fill { min: Some(Px(100)), max: None };
//!
//! // Fill space but never exceed 300px
//! let fill_with_max = DimensionValue::Fill { min: None, max: Some(Px(300)) };
//! ```
//!
//! ## Constraint Merging
//!
//! When components are nested, their constraints must be merged to resolve conflicts
//! and ensure consistent layout. The [`Constraint::merge`] method implements this
//! logic with the following rules:
//!
//! - **Fixed always wins**: A fixed constraint cannot be overridden by its parent
//! - **Wrap preserves content sizing**: Wrap constraints maintain their intrinsic sizing behavior
//! - **Fill adapts to available space**: Fill constraints expand within parent bounds
//!
//! ### Merge Examples
//!
//! ```
//! # use tessera_ui::Px;
//! # use tessera_ui::{Constraint, DimensionValue};
//! // Parent provides 200px of space
//! let parent = Constraint::new(
//!     DimensionValue::Fixed(Px(200)),
//!     DimensionValue::Fixed(Px(200))
//! );
//!
//! // Child wants to fill with minimum 50px
//! let child = Constraint::new(
//!     DimensionValue::Fill { min: Some(Px(50)), max: None },
//!     DimensionValue::Fill { min: Some(Px(50)), max: None }
//! );
//!
//! // Result: Child fills parent's 200px space, respecting its 50px minimum
//! let merged = child.merge(&parent);
//! assert_eq!(merged.width, DimensionValue::Fill {
//!     min: Some(Px(50)),
//!     max: Some(Px(200))
//! });
//! ```
//!
//! ## Usage in Components
//!
//! Components typically specify their constraints during the measurement phase:
//!
//! ```rust,ignore
//! #[tessera]
//! fn my_component() {
//!     measure(|constraints| {
//!         // This component wants to be exactly 100x50 pixels
//!         let my_constraint = Constraint::new(
//!             DimensionValue::Fixed(Px(100)),
//!             DimensionValue::Fixed(Px(50))
//!         );
//!         
//!         // Measure children with merged constraints
//!         let child_constraint = my_constraint.merge(&constraints);
//!         // ... measure children ...
//!         
//!         ComputedData::new(Size::new(Px(100), Px(50)))
//!     });
//! }
//! ```

use crate::Px;

/// Defines how a dimension (width or height) should be calculated.
///
/// This enum represents the three fundamental sizing strategies available
/// in Tessera's layout system. Each variant provides different behavior
/// for how a component determines its size in a given dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionValue {
    /// The dimension is a fixed value in logical pixels.
    ///
    /// This variant represents a component that has a specific, unchanging size.
    /// Fixed dimensions cannot be overridden by parent constraints and will
    /// always maintain their specified size regardless of available space.
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::DimensionValue;
    /// let button_width = DimensionValue::Fixed(Px(120));
    /// ```
    Fixed(Px),

    /// The dimension should wrap its content, optionally bounded by min and/or max logical pixels.
    ///
    /// This variant represents a component that sizes itself based on its content.
    /// The component will be as small as possible while still containing all its content,
    /// but can be constrained by optional minimum and maximum bounds.
    ///
    /// # Parameters
    /// - `min`: Optional minimum size - the component will never be smaller than this
    /// - `max`: Optional maximum size - the component will never be larger than this
    ///
    /// # Examples
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::DimensionValue;
    /// // Text that wraps to its content size
    /// let text_width = DimensionValue::Wrap { min: None, max: None };
    ///
    /// // Text with minimum width to prevent being too narrow
    /// let min_text_width = DimensionValue::Wrap { min: Some(Px(100)), max: None };
    ///
    /// // Text that wraps but never exceeds container width
    /// let bounded_text = DimensionValue::Wrap { min: Some(Px(50)), max: Some(Px(300)) };
    /// ```
    Wrap { min: Option<Px>, max: Option<Px> },

    /// The dimension should fill the available space, optionally bounded by min and/or max logical pixels.
    ///
    /// This variant represents a component that expands to use all available space
    /// provided by its parent. The expansion can be constrained by optional minimum
    /// and maximum bounds.
    ///
    /// # Parameters
    /// - `min`: Optional minimum size - the component will never be smaller than this
    /// - `max`: Optional maximum size - the component will never be larger than this
    ///
    /// # Examples
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::DimensionValue;
    /// // Fill all available space
    /// let flexible_width = DimensionValue::Fill { min: None, max: None };
    ///
    /// // Fill space but ensure minimum usability
    /// let min_fill_width = DimensionValue::Fill { min: Some(Px(200)), max: None };
    ///
    /// // Fill space but cap maximum size for readability
    /// let capped_fill = DimensionValue::Fill { min: Some(Px(100)), max: Some(Px(800)) };
    /// ```
    Fill { min: Option<Px>, max: Option<Px> },
}

impl DimensionValue {
    /// Zero-sized dimension, equivalent to `Fixed(Px(0))`.
    pub const ZERO: Self = DimensionValue::Fixed(Px(0));

    /// Fill with no constraints.
    pub const FILLED: Self = DimensionValue::Fill {
        min: None,
        max: None,
    };

    /// Wrap with no constraints.
    pub const WRAP: Self = DimensionValue::Wrap {
        min: None,
        max: None,
    };
}

impl Default for DimensionValue {
    /// Returns the default dimension value: `Wrap { min: None, max: None }`.
    ///
    /// This default represents a component that sizes itself to its content
    /// without any constraints, which is the most flexible and commonly used
    /// sizing behavior.
    fn default() -> Self {
        DimensionValue::Wrap {
            min: None,
            max: None,
        }
    }
}

impl DimensionValue {
    /// Converts this dimension value to a maximum pixel value.
    ///
    /// This method is useful during layout calculation when you need to determine
    /// the maximum space a component might occupy.
    ///
    /// # Parameters
    /// - `default`: The value to use when no maximum is specified
    ///
    /// # Returns
    /// - For `Fixed`: Returns the fixed value
    /// - For `Wrap` and `Fill`: Returns the `max` value if specified, otherwise the `default`
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::DimensionValue;
    /// let fixed = DimensionValue::Fixed(Px(100));
    /// assert_eq!(fixed.to_max_px(Px(200)), Px(100));
    ///
    /// let wrap_unbounded = DimensionValue::Wrap { min: None, max: None };
    /// assert_eq!(wrap_unbounded.to_max_px(Px(200)), Px(200));
    ///
    /// let wrap_bounded = DimensionValue::Wrap { min: None, max: Some(Px(150)) };
    /// assert_eq!(wrap_bounded.to_max_px(Px(200)), Px(150));
    /// ```
    pub fn to_max_px(&self, default: Px) -> Px {
        match self {
            DimensionValue::Fixed(value) => *value,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(default),
            DimensionValue::Fill { max, .. } => max.unwrap_or(default),
        }
    }

    /// Returns the maximum value of this dimension, if defined.
    ///
    /// This method extracts the maximum constraint from a dimension value,
    /// which is useful for layout calculations and constraint validation.
    ///
    /// # Returns
    /// - For `Fixed`: Returns `Some(fixed_value)` since fixed dimensions have an implicit maximum
    /// - For `Wrap` and `Fill`: Returns the `max` value if specified, otherwise `None`
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::DimensionValue;
    /// let fixed = DimensionValue::Fixed(Px(100));
    /// assert_eq!(fixed.get_max(), Some(Px(100)));
    ///
    /// let wrap_bounded = DimensionValue::Wrap { min: Some(Px(50)), max: Some(Px(200)) };
    /// assert_eq!(wrap_bounded.get_max(), Some(Px(200)));
    ///
    /// let wrap_unbounded = DimensionValue::Wrap { min: None, max: None };
    /// assert_eq!(wrap_unbounded.get_max(), None);
    /// ```
    pub fn get_max(&self) -> Option<Px> {
        match self {
            DimensionValue::Fixed(value) => Some(*value),
            DimensionValue::Wrap { max, .. } => *max,
            DimensionValue::Fill { max, .. } => *max,
        }
    }

    /// Returns the minimum value of this dimension, if defined.
    ///
    /// This method extracts the minimum constraint from a dimension value,
    /// which is useful for layout calculations and ensuring components
    /// maintain their minimum required size.
    ///
    /// # Returns
    /// - For `Fixed`: Returns `Some(fixed_value)` since fixed dimensions have an implicit minimum
    /// - For `Wrap` and `Fill`: Returns the `min` value if specified, otherwise `None`
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::DimensionValue;
    /// let fixed = DimensionValue::Fixed(Px(100));
    /// assert_eq!(fixed.get_min(), Some(Px(100)));
    ///
    /// let fill_bounded = DimensionValue::Fill { min: Some(Px(50)), max: Some(Px(200)) };
    /// assert_eq!(fill_bounded.get_min(), Some(Px(50)));
    ///
    /// let fill_unbounded = DimensionValue::Fill { min: None, max: None };
    /// assert_eq!(fill_unbounded.get_min(), None);
    /// ```
    pub fn get_min(&self) -> Option<Px> {
        match self {
            DimensionValue::Fixed(value) => Some(*value),
            DimensionValue::Wrap { min, .. } => *min,
            DimensionValue::Fill { min, .. } => *min,
        }
    }
}

/// Represents layout constraints for a component node.
///
/// A `Constraint` combines width and height dimension values to provide
/// complete layout specification for a component. It defines how a component
/// should size itself in both dimensions and provides methods for merging
/// constraints in a component hierarchy.
///
/// # Examples
///
/// ```
/// # use tessera_ui::Px;
/// # use tessera_ui::{Constraint, DimensionValue};
/// // A button with fixed size
/// let button_constraint = Constraint::new(
///     DimensionValue::Fixed(Px(120)),
///     DimensionValue::Fixed(Px(40))
/// );
///
/// // A flexible container that fills width but wraps height
/// let container_constraint = Constraint::new(
///     DimensionValue::Fill { min: Some(Px(200)), max: None },
///     DimensionValue::Wrap { min: None, max: None }
/// );
///
/// // A text component with bounded wrapping
/// let text_constraint = Constraint::new(
///     DimensionValue::Wrap { min: Some(Px(100)), max: Some(Px(400)) },
///     DimensionValue::Wrap { min: None, max: None }
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraint {
    /// The width dimension constraint
    pub width: DimensionValue,
    /// The height dimension constraint
    pub height: DimensionValue,
}

impl Constraint {
    /// A constraint that specifies no preference (Wrap { None, None } for both width and height).
    ///
    /// This constant represents the most flexible constraint possible, where a component
    /// will size itself to its content without any bounds. It's equivalent to the default
    /// constraint and is useful as a starting point for constraint calculations.
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::{Constraint, DimensionValue};
    /// let flexible = Constraint::NONE;
    /// assert_eq!(flexible.width, DimensionValue::Wrap { min: None, max: None });
    /// assert_eq!(flexible.height, DimensionValue::Wrap { min: None, max: None });
    /// ```
    pub const NONE: Self = Self {
        width: DimensionValue::Wrap {
            min: None,
            max: None,
        },
        height: DimensionValue::Wrap {
            min: None,
            max: None,
        },
    };

    /// Creates a new constraint with the specified width and height dimensions.
    ///
    /// This is the primary constructor for creating constraint instances.
    ///
    /// # Parameters
    /// - `width`: The dimension value for the width constraint
    /// - `height`: The dimension value for the height constraint
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::{Constraint, DimensionValue};
    /// let constraint = Constraint::new(
    ///     DimensionValue::Fixed(Px(100)),
    ///     DimensionValue::Fill { min: Some(Px(50)), max: None }
    /// );
    /// ```
    pub fn new(width: DimensionValue, height: DimensionValue) -> Self {
        Self { width, height }
    }

    /// Merges this constraint with a parent constraint to resolve layout conflicts.
    ///
    /// This method implements the core constraint resolution algorithm used throughout
    /// Tessera's layout system. When components are nested, their constraints must be
    /// merged to ensure consistent and predictable layout behavior.
    ///
    /// # Merge Rules
    ///
    /// The merging follows a priority system designed to respect component intentions
    /// while ensuring layout consistency:
    ///
    /// ## Fixed Constraints (Highest Priority)
    /// - **Fixed always wins**: A fixed constraint cannot be overridden by its parent
    /// - Fixed dimensions maintain their exact size regardless of available space
    ///
    /// ## Wrap Constraints (Content-Based)
    /// - **Preserves content sizing**: Wrap constraints maintain their intrinsic sizing behavior
    /// - When parent is Fixed: Child wraps within parent's fixed bounds
    /// - When parent is Wrap: Child combines min/max constraints with parent
    /// - When parent is Fill: Child wraps within parent's fill bounds
    ///
    /// ## Fill Constraints (Space-Filling)
    /// - **Adapts to available space**: Fill constraints expand within parent bounds
    /// - When parent is Fixed: Child fills parent's fixed space (respecting own min/max)
    /// - When parent is Wrap: Child fills available space within parent's wrap bounds
    /// - When parent is Fill: Child combines fill constraints with parent
    ///
    /// # Parameters
    /// - `parent_constraint`: The constraint from the parent component
    ///
    /// # Returns
    /// A new constraint that represents the resolved layout requirements
    ///
    /// # Examples
    ///
    /// ```
    /// # use tessera_ui::Px;
    /// # use tessera_ui::{Constraint, DimensionValue};
    /// // Fixed child in fixed parent - child wins
    /// let parent = Constraint::new(
    ///     DimensionValue::Fixed(Px(200)),
    ///     DimensionValue::Fixed(Px(200))
    /// );
    /// let child = Constraint::new(
    ///     DimensionValue::Fixed(Px(100)),
    ///     DimensionValue::Fixed(Px(100))
    /// );
    /// let merged = child.merge(&parent);
    /// assert_eq!(merged.width, DimensionValue::Fixed(Px(100)));
    ///
    /// // Fill child in fixed parent - child fills parent's space
    /// let child_fill = Constraint::new(
    ///     DimensionValue::Fill { min: Some(Px(50)), max: None },
    ///     DimensionValue::Fill { min: Some(Px(50)), max: None }
    /// );
    /// let merged_fill = child_fill.merge(&parent);
    /// assert_eq!(merged_fill.width, DimensionValue::Fill {
    ///     min: Some(Px(50)),
    ///     max: Some(Px(200))
    /// });
    /// ```
    pub fn merge(&self, parent_constraint: &Constraint) -> Self {
        let new_width = Self::merge_dimension(self.width, parent_constraint.width);
        let new_height = Self::merge_dimension(self.height, parent_constraint.height);
        Constraint::new(new_width, new_height)
    }

    /// Internal helper method that merges two dimension values according to the constraint rules.
    ///
    /// This method implements the detailed logic for merging individual dimension constraints.
    /// It's called by the public `merge` method to handle width and height dimensions separately.
    ///
    /// # Parameters
    /// - `child_dim`: The dimension constraint from the child component
    /// - `parent_dim`: The dimension constraint from the parent component
    ///
    /// # Returns
    /// The merged dimension value that respects both constraints appropriately
    fn merge_dimension(child_dim: DimensionValue, parent_dim: DimensionValue) -> DimensionValue {
        match child_dim {
            DimensionValue::Fixed(cv) => DimensionValue::Fixed(cv), // Child's Fixed overrides
            DimensionValue::Wrap {
                min: c_min,
                max: c_max,
            } => match parent_dim {
                DimensionValue::Fixed(pv) => DimensionValue::Wrap {
                    // Wrap stays as Wrap, but constrained by parent's fixed size
                    min: c_min, // Keep child's own min
                    max: match c_max {
                        Some(c) => Some(c.min(pv)), // Child's max capped by parent's fixed size
                        None => Some(pv),           // Parent's fixed size becomes the max
                    },
                },
                DimensionValue::Wrap {
                    min: _p_min,
                    max: p_max,
                } => DimensionValue::Wrap {
                    // Combine min/max from parent and child for Wrap
                    min: c_min, // Wrap always keeps its own min, never inherits from parent
                    max: match (c_max, p_max) {
                        (Some(c), Some(p)) => Some(c.min(p)), // Take the more restrictive max
                        (Some(c), None) => Some(c),
                        (None, Some(p)) => Some(p),
                        (None, None) => None,
                    },
                },
                DimensionValue::Fill {
                    min: _p_fill_min,
                    max: p_fill_max,
                } => DimensionValue::Wrap {
                    // Child wants to wrap, so it stays as Wrap
                    min: c_min, // Keep child's own min, don't inherit from parent's Fill
                    max: match (c_max, p_fill_max) {
                        (Some(c), Some(p)) => Some(c.min(p)), // Child's max should cap parent's fill max
                        (Some(c), None) => Some(c),
                        (None, Some(p)) => Some(p),
                        (None, None) => None,
                    },
                },
            },
            DimensionValue::Fill {
                min: c_fill_min,
                max: c_fill_max,
            } => match parent_dim {
                DimensionValue::Fixed(pv) => {
                    // Child wants to fill, parent is fixed. Result is Fill with parent's fixed size as max.
                    DimensionValue::Fill {
                        min: c_fill_min, // Keep child's own min
                        max: match c_fill_max {
                            Some(c) => Some(c.min(pv)), // Child's max capped by parent's fixed size
                            None => Some(pv),           // Parent's fixed size becomes the max
                        },
                    }
                }
                DimensionValue::Wrap {
                    min: p_wrap_min,
                    max: p_wrap_max,
                } => DimensionValue::Fill {
                    // Fill remains Fill, parent Wrap offers no concrete size unless it has max
                    min: c_fill_min.or(p_wrap_min), // Child's fill min, or parent's wrap min
                    max: match (c_fill_max, p_wrap_max) {
                        // Child's fill max, potentially capped by parent's wrap max
                        (Some(cf), Some(pw)) => Some(cf.min(pw)),
                        (Some(cf), None) => Some(cf),
                        (None, Some(pw)) => Some(pw),
                        (None, None) => None,
                    },
                },
                DimensionValue::Fill {
                    min: p_fill_min,
                    max: p_fill_max,
                } => {
                    // Both are Fill. Combine min and max.
                    // New min is the greater of the two mins (or the existing one).
                    // New max is the smaller of the two maxes (or the existing one).
                    let new_min = match (c_fill_min, p_fill_min) {
                        (Some(cm), Some(pm)) => Some(cm.max(pm)),
                        (Some(cm), None) => Some(cm),
                        (None, Some(pm)) => Some(pm),
                        (None, None) => None,
                    };
                    let new_max = match (c_fill_max, p_fill_max) {
                        (Some(cm), Some(pm)) => Some(cm.min(pm)),
                        (Some(cm), None) => Some(cm),
                        (None, Some(pm)) => Some(pm),
                        (None, None) => None,
                    };
                    // Ensure min <= max if both are Some
                    let (final_min, final_max) = match (new_min, new_max) {
                        (Some(n_min), Some(n_max)) if n_min > n_max => (Some(n_max), Some(n_max)), // Or handle error/warning
                        _ => (new_min, new_max),
                    };
                    DimensionValue::Fill {
                        min: final_min,
                        max: final_max,
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_parent_wrap_child_wrap_grandchild() {
        // Test three-level hierarchy: Fixed(100) -> Wrap{20-80} -> Wrap{10-50}
        // This tests constraint propagation through multiple levels

        // Parent component with fixed 100x100 size
        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        // Child component that wraps content with bounds 20-80
        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80)),
            },
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80)),
            },
        );

        // Grandchild component that wraps content with bounds 10-50
        let grandchild = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50)),
            },
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50)),
            },
        );

        // First level merge: child merges with fixed parent
        let merged_child = child.merge(&parent);

        // Child is Wrap, parent is Fixed - result should be Wrap with child's constraints
        // Since child's max (80) is less than parent's fixed size (100), child keeps its bounds
        assert_eq!(
            merged_child.width,
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80))
            }
        );
        assert_eq!(
            merged_child.height,
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80))
            }
        );

        // Second level merge: grandchild merges with merged child
        let final_result = grandchild.merge(&merged_child);

        // Both are Wrap - result should be Wrap with the more restrictive constraints
        // Grandchild's max (50) is smaller than merged child's max (80), so grandchild wins
        assert_eq!(
            final_result.width,
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50))
            }
        );
        assert_eq!(
            final_result.height,
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50))
            }
        );
    }

    #[test]
    fn test_fill_parent_wrap_child() {
        // Test Fill parent with Wrap child: Fill{50-200} -> Wrap{30-150}
        // Child should remain Wrap and keep its own constraints

        let parent = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
        );

        let result = child.merge(&parent);

        // Child is Wrap, parent is Fill - result should be Wrap
        // Child keeps its own min (30px) and max (150px) since both are within parent's bounds
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
    }

    #[test]
    fn test_fill_parent_wrap_child_no_child_min() {
        // Test Fill parent with Wrap child that has no minimum: Fill{50-200} -> Wrap{None-150}
        // Child should keep its own constraints and not inherit parent's minimum

        let parent = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150)),
            },
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150)),
            },
        );

        let result = child.merge(&parent);

        // Child is Wrap and should keep its own min (None), not inherit from parent's Fill min
        // This preserves the wrap behavior of sizing to content without artificial minimums
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150))
            }
        );
    }

    #[test]
    fn test_fill_parent_wrap_child_no_parent_max() {
        // Test Fill parent with no maximum and Wrap child: Fill{50-None} -> Wrap{30-150}
        // Child should keep its own constraints since parent has no upper bound

        let parent = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: None,
            },
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: None,
            },
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
        );

        let result = child.merge(&parent);

        // Child should keep its own constraints since parent Fill has no max to constrain it
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
    }

    #[test]
    fn test_fixed_parent_wrap_child() {
        // Test Fixed parent with Wrap child: Fixed(100) -> Wrap{30-120}
        // Child's max should be capped by parent's fixed size

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
        );

        let result = child.merge(&parent);

        // Child remains Wrap but max is limited by parent's fixed size
        // min keeps child's own value (30px)
        // max becomes the smaller of child's max (120px) and parent's fixed size (100px)
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_wrap_child_no_child_max() {
        // Test Fixed parent with Wrap child that has no maximum: Fixed(100) -> Wrap{30-None}
        // Parent's fixed size should become the child's maximum

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: None,
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: None,
            },
        );

        let result = child.merge(&parent);

        // Child remains Wrap, parent's fixed size becomes the maximum constraint
        // This prevents the child from growing beyond the parent's available space
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_fill_child() {
        // Test Fixed parent with Fill child: Fixed(100) -> Fill{30-120}
        // Child should fill parent's space but be capped by parent's fixed size

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
        );

        let result = child.merge(&parent);

        // Child remains Fill but max is limited by parent's fixed size
        // min keeps child's own value (30px)
        // max becomes the smaller of child's max (120px) and parent's fixed size (100px)
        assert_eq!(
            result.width,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_fill_child_no_child_max() {
        // Test Fixed parent with Fill child that has no maximum: Fixed(100) -> Fill{30-None}
        // Parent's fixed size should become the child's maximum

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: None,
            },
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: None,
            },
        );

        let result = child.merge(&parent);

        // Child remains Fill, parent's fixed size becomes the maximum constraint
        // This ensures the child fills exactly the parent's available space
        assert_eq!(
            result.width,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_fill_child_no_child_min() {
        // Test Fixed parent with Fill child that has no minimum: Fixed(100) -> Fill{None-120}
        // Child should fill parent's space with no minimum constraint

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Fill {
                min: None,
                max: Some(Px(120)),
            },
            DimensionValue::Fill {
                min: None,
                max: Some(Px(120)),
            },
        );

        let result = child.merge(&parent);

        // Child remains Fill, keeps its own min (None), max is limited by parent's fixed size
        // This allows the child to fill the parent's space without any minimum size requirement
        assert_eq!(
            result.width,
            DimensionValue::Fill {
                min: None,
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Fill {
                min: None,
                max: Some(Px(100))
            }
        );
    }
}
