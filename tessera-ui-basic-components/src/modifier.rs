//! Modifier extensions for basic components.
//!
//! ## Usage
//!
//! Attach layout behavior like padding and sizing to any subtree.

use tessera_ui::{
    ComputedData, Constraint, DimensionValue, Dp, Modifier, Px, PxPosition, tessera, use_context,
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
#[derive(Clone, Copy, Debug, Default)]
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

    /// Creates padding with explicit edges.
    pub const fn only(left: Dp, top: Dp, right: Dp, bottom: Dp) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

/// Extensions for composing reusable wrapper behavior around component
/// subtrees.
pub trait ModifierExt {
    /// Adds padding around the content.
    fn padding(self, padding: Padding) -> Modifier;

    /// Adds symmetric padding on all edges.
    fn padding_all(self, padding: Dp) -> Modifier;

    /// Adds symmetric padding for horizontal and vertical edges.
    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier;

    /// Offsets the content without affecting layout size.
    fn offset(self, x: Dp, y: Dp) -> Modifier;

    /// Constrains the content to an exact size when possible.
    fn size(self, width: Dp, height: Dp) -> Modifier;

    /// Constrains the content to an exact width when possible.
    fn width(self, width: Dp) -> Modifier;

    /// Constrains the content to an exact height when possible.
    fn height(self, height: Dp) -> Modifier;

    /// Constrains the content size within optional min/max bounds.
    fn size_in(
        self,
        min_width: Option<Dp>,
        max_width: Option<Dp>,
        min_height: Option<Dp>,
        max_height: Option<Dp>,
    ) -> Modifier;

    /// Applies explicit width/height `DimensionValue` constraints.
    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier;

    /// Fills the available width within parent bounds.
    fn fill_max_width(self) -> Modifier;

    /// Fills the available height within parent bounds.
    fn fill_max_height(self) -> Modifier;

    /// Fills the available size within parent bounds.
    fn fill_max_size(self) -> Modifier;

    /// Enforces a minimum interactive size by expanding and centering content.
    fn minimum_interactive_component_size(self) -> Modifier;
}

impl ModifierExt for Modifier {
    fn padding(self, padding: Padding) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_padding(padding, || {
                    child();
                });
            }
        })
    }

    fn padding_all(self, padding: Dp) -> Modifier {
        self.padding(Padding::all(padding))
    }

    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier {
        self.padding(Padding::symmetric(horizontal, vertical))
    }

    fn offset(self, x: Dp, y: Dp) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_offset(x, y, || {
                    child();
                });
            }
        })
    }

    fn size(self, width: Dp, height: Dp) -> Modifier {
        let width_px: Px = width.into();
        let height_px: Px = height.into();
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    Some(DimensionValue::Wrap {
                        min: Some(width_px),
                        max: Some(width_px),
                    }),
                    Some(DimensionValue::Wrap {
                        min: Some(height_px),
                        max: Some(height_px),
                    }),
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn width(self, width: Dp) -> Modifier {
        let width_px: Px = width.into();
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    Some(DimensionValue::Wrap {
                        min: Some(width_px),
                        max: Some(width_px),
                    }),
                    None,
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn height(self, height: Dp) -> Modifier {
        let height_px: Px = height.into();
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    None,
                    Some(DimensionValue::Wrap {
                        min: Some(height_px),
                        max: Some(height_px),
                    }),
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn size_in(
        self,
        min_width: Option<Dp>,
        max_width: Option<Dp>,
        min_height: Option<Dp>,
        max_height: Option<Dp>,
    ) -> Modifier {
        let width = DimensionValue::Wrap {
            min: min_width.map(Into::into),
            max: max_width.map(Into::into),
        };
        let height = DimensionValue::Wrap {
            min: min_height.map(Into::into),
            max: max_height.map(Into::into),
        };
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(Some(width), Some(height), || {
                    child();
                });
            }
        })
    }

    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(width, height, || {
                    child();
                });
            }
        })
    }

    fn fill_max_width(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(Some(DimensionValue::FILLED), None, || {
                    child();
                });
            }
        })
    }

    fn fill_max_height(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(None, Some(DimensionValue::FILLED), || {
                    child();
                });
            }
        })
    }

    fn fill_max_size(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    Some(DimensionValue::FILLED),
                    Some(DimensionValue::FILLED),
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn minimum_interactive_component_size(self) -> Modifier {
        if !use_context::<MinimumInteractiveComponentEnforcement>()
            .get()
            .enabled
        {
            return self;
        }

        self.push_wrapper(move |child| {
            move || {
                modifier_minimum_interactive_size(|| {
                    child();
                });
            }
        })
    }
}

fn subtract_opt_px(value: Option<Px>, subtract: Px) -> Option<Px> {
    value.map(|v| (v - subtract).max(Px(0)))
}

fn shrink_dimension(dimension: DimensionValue, before: Px, after: Px) -> DimensionValue {
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

fn resolve_dimension(dimension: DimensionValue, content: Px, axis: &'static str) -> Px {
    match dimension {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Wrap { min, max } => {
            let mut value = content;
            if let Some(min_value) = min {
                value = value.max(min_value);
            }
            if let Some(max_value) = max {
                value = value.min(max_value);
            }
            value
        }
        DimensionValue::Fill { min, max } => {
            let Some(max_value) = max else {
                panic!(
                    "Seems that you are trying to fill an infinite dimension, which is not allowed\naxis = {axis}\nconstraint = {dimension:?}"
                );
            };
            let mut value = max_value;
            if let Some(min_value) = min {
                value = value.max(min_value);
            }
            value
        }
    }
}

#[tessera]
fn modifier_padding<F>(padding: Padding, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_padding expects exactly one child");

        let left_px: Px = padding.left.into();
        let top_px: Px = padding.top.into();
        let right_px: Px = padding.right.into();
        let bottom_px: Px = padding.bottom.into();

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_constraint = Constraint::new(
            shrink_dimension(parent_constraint.width, left_px, right_px),
            shrink_dimension(parent_constraint.height, top_px, bottom_px),
        );

        let child_measurements = input.measure_children(vec![(child_id, child_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let content_width = child_measurement.width + left_px + right_px;
        let content_height = child_measurement.height + top_px + bottom_px;

        let final_width = resolve_dimension(parent_constraint.width, content_width, "width");
        let final_height = resolve_dimension(parent_constraint.height, content_height, "height");

        input.place_child(child_id, PxPosition::new(left_px, top_px));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_offset<F>(x: Dp, y: Dp, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_offset expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        input.place_child(child_id, PxPosition::new(x.into(), y.into()));

        Ok(child_measurement)
    }));

    child();
}

#[tessera]
fn modifier_constraints<F>(
    width_override: Option<DimensionValue>,
    height_override: Option<DimensionValue>,
    child: F,
) where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_constraints expects exactly one child");

        let parent_width = input.parent_constraint.width();
        let parent_height = input.parent_constraint.height();
        let constraint = Constraint::new(
            width_override.unwrap_or(parent_width),
            height_override.unwrap_or(parent_height),
        )
        .merge(input.parent_constraint);

        let child_measurements = input.measure_children(vec![(child_id, constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width = resolve_dimension(constraint.width, child_measurement.width, "width");
        let final_height = resolve_dimension(constraint.height, child_measurement.height, "height");

        input.place_child(child_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_minimum_interactive_size<F>(child: F)
where
    F: FnOnce(),
{
    const MIN_SIZE: Dp = Dp(48.0);

    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_minimum_interactive_size expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let min_px: Px = MIN_SIZE.into();
        let content_width = child_measurement.width.max(min_px);
        let content_height = child_measurement.height.max(min_px);

        let final_width = resolve_dimension(parent_constraint.width, content_width, "width");
        let final_height = resolve_dimension(parent_constraint.height, content_height, "height");

        let x = ((final_width - child_measurement.width) / 2).max(Px(0));
        let y = ((final_height - child_measurement.height) / 2).max(Px(0));
        input.place_child(child_id, PxPosition::new(x, y));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}
