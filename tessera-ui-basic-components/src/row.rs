//! # Row Module
//!
//! Provides the [`row`] component and related utilities for horizontal layout in Tessera UI.
//!
//! This module defines a flexible and composable row layout component, allowing child components to be arranged horizontally with customizable alignment, sizing, and weighted space distribution. It is a fundamental building block for constructing responsive UI layouts, such as toolbars, navigation bars, forms, and any scenario requiring horizontal stacking of elements.
//!
//! ## Features
//! - Horizontal arrangement of child components
//! - Support for main/cross axis alignment and flexible sizing
//! - Weighted children for proportional space allocation
//! - Declarative macro [`row_ui!`] for ergonomic usage
//!
//! ## Typical Usage
//! Use the [`row`] component to build horizontal layouts, optionally combining with [`column`](crate::column) for complex grid or responsive designs.
//!
//! See the documentation and examples for details on arguments and usage patterns.
//!
//! ---
//!
//! This module is part of the `tessera-ui-basic-components` crate.
//!
//! ## Example
//! ```
//! use tessera_ui_basic_components::{row::{row_ui, RowArgs}, text::text};
//! row_ui!(RowArgs::default(),
//!     || text("A".to_string()),
//!     || text("B".to_string()),
//! );
//! ```
use derive_builder::Builder;
use tessera_ui::{ComputedData, Constraint, DimensionValue, Px, PxPosition, place_node};
use tessera_ui_macros::tessera;

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

pub use crate::row_ui;

/// Arguments for the `row` component.
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct RowArgs {
    /// Width behavior for the row.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub width: DimensionValue,
    /// Height behavior for the row.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
    /// Main axis alignment (horizontal alignment).
    #[builder(default = "MainAxisAlignment::Start")]
    pub main_axis_alignment: MainAxisAlignment,
    /// Cross axis alignment (vertical alignment).
    #[builder(default = "CrossAxisAlignment::Start")]
    pub cross_axis_alignment: CrossAxisAlignment,
}

impl Default for RowArgs {
    fn default() -> Self {
        RowArgsBuilder::default().build().unwrap()
    }
}

/// Represents a child item within a row layout.
pub struct RowItem {
    /// Optional weight for flexible space distribution
    pub weight: Option<f32>,
    /// The actual child component
    pub child: Box<dyn FnOnce() + Send + Sync>,
}

impl RowItem {
    /// Creates a new `RowItem` with optional weight.
    pub fn new(child: Box<dyn FnOnce() + Send + Sync>, weight: Option<f32>) -> Self {
        RowItem { weight, child }
    }

    /// Creates a weighted row item
    pub fn weighted(child: Box<dyn FnOnce() + Send + Sync>, weight: f32) -> Self {
        RowItem {
            weight: Some(weight),
            child,
        }
    }
}

/// Trait to allow various types to be converted into a `RowItem`.
pub trait AsRowItem {
    fn into_row_item(self) -> RowItem;
}

impl AsRowItem for RowItem {
    fn into_row_item(self) -> RowItem {
        self
    }
}

/// Default conversion: a simple function closure becomes a `RowItem` without weight.
impl<F: FnOnce() + Send + Sync + 'static> AsRowItem for F {
    fn into_row_item(self) -> RowItem {
        RowItem {
            weight: None,
            child: Box::new(self),
        }
    }
}

/// Allow (FnOnce, weight) to be a RowItem
impl<F: FnOnce() + Send + Sync + 'static> AsRowItem for (F, f32) {
    fn into_row_item(self) -> RowItem {
        RowItem {
            weight: Some(self.1),
            child: Box::new(self.0),
        }
    }
}

/// A row component that arranges its children horizontally.
/// A layout component that arranges its children horizontally.
///
/// The `row` component is a fundamental building block for creating horizontal layouts.
/// It takes a set of child components and arranges them one after another in a single
/// row. The layout behavior can be extensively customized through the `RowArgs` struct.
///
/// # Arguments
///
/// * `args`: A `RowArgs` struct that configures the layout properties of the row.
///   - `width` and `height`: Control the dimensions of the row container. They can be
///     set to `DimensionValue::Wrap` to fit the content, `DimensionValue::Fixed` for a
///     specific size, or `DimensionValue::Fill` to occupy available space.
///   - `main_axis_alignment`: Determines how children are distributed along the horizontal
///     axis (e.g., `Start`, `Center`, `End`, `SpaceBetween`).
///   - `cross_axis_alignment`: Determines how children are aligned along the vertical
///     axis (e.g., `Start`, `Center`, `End`, `Stretch`).
///
/// * `children_items_input`: An array of child components to be displayed in the row.
///   Children can be simple closures, or they can be wrapped in `RowItem` to provide
///   a `weight` for flexible space distribution. Weighted children will expand to fill
///   any remaining space in the row according to their weight proportion.
///
/// # Example
///
/// A simple row with three text components, centered horizontally and vertically.
///
/// ```
/// use tessera_ui_basic_components::{row::{row_ui, RowArgs}, text::text};
/// use tessera_ui_basic_components::alignment::{MainAxisAlignment, CrossAxisAlignment};
/// use tessera_ui::{DimensionValue, Dp};
///
/// let args = RowArgs {
///     main_axis_alignment: MainAxisAlignment::Center,
///     cross_axis_alignment: CrossAxisAlignment::Center,
///     width: DimensionValue::Fill { min: None, max: None },
///     height: DimensionValue::Fixed(Dp(50.0).into()),
/// };
///
/// row_ui!(args,
///     || text("First".to_string()),
///     || text("Second".to_string()),
///     || text("Third".to_string()),
/// );
/// ```
#[tessera]
pub fn row<const N: usize>(args: RowArgs, children_items_input: [impl AsRowItem; N]) {
    let children_items: [RowItem; N] =
        children_items_input.map(|item_input| item_input.into_row_item());

    let mut child_closures = Vec::with_capacity(N);
    let mut child_weights = Vec::with_capacity(N);

    for child_item in children_items {
        child_closures.push(child_item.child);
        child_weights.push(child_item.weight);
    }

    measure(Box::new(move |input| {
        let row_intrinsic_constraint = Constraint::new(args.width, args.height);
        // This is the effective constraint for the row itself
        let row_effective_constraint = row_intrinsic_constraint.merge(input.parent_constraint);

        let mut children_sizes = vec![None; N];
        let mut max_child_height = Px(0);

        // For row, main axis is horizontal, so check width for weight distribution
        let should_use_weight_for_width = match row_effective_constraint.width {
            DimensionValue::Fixed(_) => true,
            DimensionValue::Fill { max: Some(_), .. } => true,
            DimensionValue::Wrap { max: Some(_), .. } => true,
            _ => false,
        };

        if should_use_weight_for_width {
            let available_width_for_children = row_effective_constraint.width.get_max().unwrap();

            let mut weighted_children_indices = Vec::new();
            let mut unweighted_children_indices = Vec::new();
            let mut total_weight_sum = 0.0f32;

            for (i, weight_opt) in child_weights.iter().enumerate() {
                if let Some(w) = weight_opt {
                    if *w > 0.0 {
                        weighted_children_indices.push(i);
                        total_weight_sum += w;
                    } else {
                        unweighted_children_indices.push(i);
                    }
                } else {
                    unweighted_children_indices.push(i);
                }
            }

            let mut total_width_of_unweighted_children = Px(0);
            for &child_idx in &unweighted_children_indices {
                let child_id = input.children_ids[child_idx];

                // Parent (row) offers Wrap for width and its own effective height constraint to unweighted children
                let parent_offered_constraint_for_child = Constraint::new(
                    DimensionValue::Wrap {
                        min: None,
                        max: row_effective_constraint.width.get_max(),
                    },
                    row_effective_constraint.height,
                );

                // measure_node will fetch the child's intrinsic constraint and merge it
                let child_result =
                    input.measure_child(child_id, &parent_offered_constraint_for_child)?;

                children_sizes[child_idx] = Some(child_result);
                total_width_of_unweighted_children += child_result.width;
                max_child_height = max_child_height.max(child_result.height);
            }

            let remaining_width_for_weighted_children =
                (available_width_for_children - total_width_of_unweighted_children).max(Px(0));
            if total_weight_sum > 0.0 {
                for &child_idx in &weighted_children_indices {
                    let child_weight = child_weights[child_idx].unwrap_or(0.0);
                    let allocated_width_for_child =
                        Px((remaining_width_for_weighted_children.0 as f32
                            * (child_weight / total_weight_sum)) as i32);
                    let child_id = input.children_ids[child_idx];

                    // Parent (row) offers Fixed allocated width and its own effective height constraint to weighted children
                    let parent_offered_constraint_for_child = Constraint::new(
                        DimensionValue::Fixed(allocated_width_for_child),
                        row_effective_constraint.height,
                    );

                    // measure_node will fetch the child's intrinsic constraint and merge it
                    let child_result =
                        input.measure_child(child_id, &parent_offered_constraint_for_child)?;

                    children_sizes[child_idx] = Some(child_result);
                    max_child_height = max_child_height.max(child_result.height);
                }
            }

            let final_row_width = available_width_for_children;
            // row's height is determined by its own effective constraint, or by wrapping content if no explicit max.
            let final_row_height = match row_effective_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Fill { max: Some(h), .. } => h,
                DimensionValue::Wrap { min, max } => {
                    let mut h = max_child_height;
                    if let Some(min_h) = min {
                        h = h.max(min_h);
                    }
                    if let Some(max_h) = max {
                        h = h.min(max_h);
                    }
                    h
                }
                _ => max_child_height, // Fill { max: None } or Wrap { max: None } -> wraps content
            };

            let total_measured_children_width: Px = children_sizes
                .iter()
                .filter_map(|size_opt| size_opt.as_ref().map(|s| s.width))
                .fold(Px(0), |acc, width| acc + width);

            place_children_with_alignment(
                &children_sizes,
                input.children_ids,
                input.metadatas,
                final_row_width,
                final_row_height,
                total_measured_children_width,
                args.main_axis_alignment,
                args.cross_axis_alignment,
                N,
            );

            Ok(ComputedData {
                width: final_row_width,
                height: final_row_height,
            })
        } else {
            // Not using weight logic for width (row width is Wrap or Fill without max)
            let mut total_children_measured_width = Px(0);

            for i in 0..N {
                let child_id = input.children_ids[i];

                // Parent (row) offers Wrap for width and its effective height
                let parent_offered_constraint_for_child = Constraint::new(
                    match row_effective_constraint.width {
                        DimensionValue::Fixed(v) => DimensionValue::Wrap {
                            min: None,
                            max: Some(v),
                        },
                        DimensionValue::Fill { max, .. } => DimensionValue::Wrap { min: None, max },
                        DimensionValue::Wrap { max, .. } => DimensionValue::Wrap { min: None, max },
                    },
                    row_effective_constraint.height,
                );

                // measure_node will fetch the child's intrinsic constraint and merge it
                let child_result =
                    input.measure_child(child_id, &parent_offered_constraint_for_child)?;

                children_sizes[i] = Some(child_result);
                total_children_measured_width += child_result.width;
                max_child_height = max_child_height.max(child_result.height);
            }

            // Determine row's final size based on its own constraints and content
            let final_row_width = match row_effective_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Fill { min, .. } => {
                    // Max is None if here. In this case, Fill should take up all available space
                    // from the parent, not wrap the content.
                    let mut w = input
                        .parent_constraint
                        .width
                        .get_max()
                        .unwrap_or(total_children_measured_width);
                    if let Some(min_w) = min {
                        w = w.max(min_w);
                    }
                    w
                }
                DimensionValue::Wrap { min, max } => {
                    let mut w = total_children_measured_width;
                    if let Some(min_w) = min {
                        w = w.max(min_w);
                    }
                    if let Some(max_w) = max {
                        w = w.min(max_w);
                    }
                    w
                }
            };

            let final_row_height = match row_effective_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Fill { min, max } => {
                    let mut h = max_child_height;
                    if let Some(min_h) = min {
                        h = h.max(min_h);
                    }
                    if let Some(max_h) = max {
                        h = h.min(max_h);
                    } else {
                        h = max_child_height;
                    }
                    h
                }
                DimensionValue::Wrap { min, max } => {
                    let mut h = max_child_height;
                    if let Some(min_h) = min {
                        h = h.max(min_h);
                    }
                    if let Some(max_h) = max {
                        h = h.min(max_h);
                    }
                    h
                }
            };

            place_children_with_alignment(
                &children_sizes,
                input.children_ids,
                input.metadatas,
                final_row_width,
                final_row_height,
                total_children_measured_width,
                args.main_axis_alignment,
                args.cross_axis_alignment,
                N,
            );

            Ok(ComputedData {
                width: final_row_width,
                height: final_row_height,
            })
        }
    }));

    for child_closure in child_closures {
        child_closure();
    }
}

/// A helper function to place children with alignment (horizontal layout).
fn place_children_with_alignment(
    children_sizes: &[Option<ComputedData>],
    children_ids: &[tessera_ui::NodeId],
    metadatas: &tessera_ui::ComponentNodeMetaDatas,
    final_row_width: Px,
    final_row_height: Px,
    total_children_width: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
) {
    let available_space = (final_row_width - total_children_width).max(Px(0));

    // Calculate start position and spacing on the main axis (horizontal for row)
    let (mut current_x, spacing_between_children) = match main_axis_alignment {
        MainAxisAlignment::Start => (Px(0), Px(0)),
        MainAxisAlignment::Center => (available_space / 2, Px(0)),
        MainAxisAlignment::End => (available_space, Px(0)),
        MainAxisAlignment::SpaceEvenly => {
            if child_count > 0 {
                let s = available_space / (child_count as i32 + 1);
                (s, s)
            } else {
                (Px(0), Px(0))
            }
        }
        MainAxisAlignment::SpaceBetween => {
            if child_count > 1 {
                (Px(0), available_space / (child_count as i32 - 1))
            } else if child_count == 1 {
                (available_space / 2, Px(0))
            } else {
                (Px(0), Px(0))
            }
        }
        MainAxisAlignment::SpaceAround => {
            if child_count > 0 {
                let s = available_space / (child_count as i32);
                (s / 2, s)
            } else {
                (Px(0), Px(0))
            }
        }
    };

    for (i, child_size_opt) in children_sizes.iter().enumerate() {
        if let Some(child_actual_size) = child_size_opt {
            let child_id = children_ids[i];

            // Calculate position on the cross axis (vertical for row)
            let y_offset = match cross_axis_alignment {
                CrossAxisAlignment::Start => Px(0),
                CrossAxisAlignment::Center => {
                    (final_row_height - child_actual_size.height).max(Px(0)) / 2
                }
                CrossAxisAlignment::End => (final_row_height - child_actual_size.height).max(Px(0)),
                CrossAxisAlignment::Stretch => Px(0),
            };

            place_node(child_id, PxPosition::new(current_x, y_offset), metadatas);
            current_x += child_actual_size.width;
            if i < child_count - 1 {
                current_x += spacing_between_children;
            }
        }
    }
}

/// A declarative macro to simplify the creation of a [`row`](crate::row::row) component.
///
/// The first argument is the `RowArgs` struct, followed by a variable number of
/// child components. Each child expression will be converted to a `RowItem`
/// using the `AsRowItem` trait. This allows passing closures, `RowItem` instances,
/// or `(FnOnce, weight)` tuples.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::{row::{row_ui, RowArgs, RowItem}, text::text};
///
/// row_ui![
///     RowArgs::default(),
///     || text("Hello".to_string()), // Closure
///     (|| text("Weighted".to_string()), 0.5), // Weighted closure
///     RowItem::new(Box::new(|| text("Item".to_string())), None) // RowItem instance
/// ];
/// ```
#[macro_export]
macro_rules! row_ui {
    ($args:expr $(, $child:expr)* $(,)?) => {
        {
            use $crate::row::AsRowItem;
            $crate::row::row($args, [
                $(
                    $child.into_row_item()
                ),*
            ])
        }
    };
}
