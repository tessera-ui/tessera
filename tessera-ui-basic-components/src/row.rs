//! # Row Component
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
use tessera_ui::{
    ComponentNodeMetaDatas, ComputedData, Constraint, DimensionValue, MeasureInput,
    MeasurementError, NodeId, Px, PxPosition, place_node, tessera,
};

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

struct PlaceChildrenArgs<'a> {
    children_sizes: &'a [Option<ComputedData>],
    children_ids: &'a [NodeId],
    metadatas: &'a ComponentNodeMetaDatas,
    final_row_width: Px,
    final_row_height: Px,
    total_children_width: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
}

struct MeasureWeightedChildrenArgs<'a> {
    input: &'a MeasureInput<'a>,
    weighted_indices: &'a [usize],
    children_sizes: &'a mut [Option<ComputedData>],
    max_child_height: &'a mut Px,
    remaining_width: Px,
    total_weight: f32,
    row_effective_constraint: &'a Constraint,
    child_weights: &'a [Option<f32>],
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

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let row_intrinsic_constraint = Constraint::new(args.width, args.height);
            let row_effective_constraint = row_intrinsic_constraint.merge(input.parent_constraint);

            let should_use_weight_for_width = match row_effective_constraint.width {
                DimensionValue::Fixed(_) => true,
                DimensionValue::Fill { max: Some(_), .. } => true,
                DimensionValue::Wrap { max: Some(_), .. } => true,
                _ => false,
            };

            if should_use_weight_for_width {
                measure_weighted_row(input, &args, &child_weights, &row_effective_constraint)
            } else {
                measure_unweighted_row(input, &args, &row_effective_constraint)
            }
        },
    ));

    for child_closure in child_closures {
        child_closure();
    }
}

fn measure_weighted_row(
    input: &MeasureInput,
    args: &RowArgs,
    child_weights: &[Option<f32>],
    row_effective_constraint: &Constraint,
) -> Result<ComputedData, MeasurementError> {
    // Prepare buffers and metadata for measurement:
    // - `children_sizes` stores each child's measurement result (width, height).
    // - `max_child_height` tracks the maximum height among children to compute the row's final height.
    // - `available_width_for_children` is the total width available to allocate to children under the current constraint (present only for Fill/Fixed/Wrap(max)).
    let n = input.children_ids.len();
    let mut children_sizes = vec![None; n];
    let mut max_child_height = Px(0);
    let available_width_for_children = row_effective_constraint.width.get_max().unwrap();

    // Classify children into weighted and unweighted and compute the total weight.
    let (weighted_indices, unweighted_indices, total_weight) = classify_children(child_weights);

    let total_width_of_unweighted_children = measure_unweighted_children(
        input,
        &unweighted_indices,
        &mut children_sizes,
        &mut max_child_height,
        row_effective_constraint,
    )?;

    measure_weighted_children(&mut MeasureWeightedChildrenArgs {
        input,
        weighted_indices: &weighted_indices,
        children_sizes: &mut children_sizes,
        max_child_height: &mut max_child_height,
        remaining_width: available_width_for_children - total_width_of_unweighted_children,
        total_weight,
        row_effective_constraint,
        child_weights,
    })?;

    let final_row_width = available_width_for_children;
    let final_row_height = calculate_final_row_height(row_effective_constraint, max_child_height);

    let total_measured_children_width: Px = children_sizes
        .iter()
        .filter_map(|s| s.map(|s| s.width))
        .fold(Px(0), |acc, w| acc + w);

    place_children_with_alignment(&PlaceChildrenArgs {
        children_sizes: &children_sizes,
        children_ids: input.children_ids,
        metadatas: input.metadatas,
        final_row_width,
        final_row_height,
        total_children_width: total_measured_children_width,
        main_axis_alignment: args.main_axis_alignment,
        cross_axis_alignment: args.cross_axis_alignment,
        child_count: n,
    });

    Ok(ComputedData {
        width: final_row_width,
        height: final_row_height,
    })
}

fn measure_unweighted_row(
    input: &MeasureInput,
    args: &RowArgs,
    row_effective_constraint: &Constraint,
) -> Result<ComputedData, MeasurementError> {
    // Measure an unweighted row:
    // For each child, create a 'wrap' constraint based on the row's effective constraint,
    // use `input.measure_child` to obtain its actual size, and accumulate total width and max height.
    let n = input.children_ids.len();
    let mut children_sizes = vec![None; n];
    let mut total_children_measured_width = Px(0);
    let mut max_child_height = Px(0);

    for i in 0..n {
        let child_id = input.children_ids[i];
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
        let child_result = input.measure_child(child_id, &parent_offered_constraint_for_child)?;
        children_sizes[i] = Some(child_result);
        total_children_measured_width += child_result.width;
        max_child_height = max_child_height.max(child_result.height);
    }

    let final_row_width =
        calculate_final_row_width(row_effective_constraint, total_children_measured_width);
    let final_row_height = calculate_final_row_height(row_effective_constraint, max_child_height);

    place_children_with_alignment(&PlaceChildrenArgs {
        children_sizes: &children_sizes,
        children_ids: input.children_ids,
        metadatas: input.metadatas,
        final_row_width,
        final_row_height,
        total_children_width: total_children_measured_width,
        main_axis_alignment: args.main_axis_alignment,
        cross_axis_alignment: args.cross_axis_alignment,
        child_count: n,
    });

    Ok(ComputedData {
        width: final_row_width,
        height: final_row_height,
    })
}

fn classify_children(child_weights: &[Option<f32>]) -> (Vec<usize>, Vec<usize>, f32) {
    // Split children into weighted and unweighted categories and compute the total weight of weighted children.
    // Returns: (weighted_indices, unweighted_indices, total_weight)
    let mut weighted_indices = Vec::new();
    let mut unweighted_indices = Vec::new();
    let mut total_weight = 0.0;

    for (i, weight) in child_weights.iter().enumerate() {
        if let Some(w) = weight {
            if *w > 0.0 {
                weighted_indices.push(i);
                total_weight += w;
            } else {
                // weight == 0.0 is treated as an unweighted item (it won't participate in remaining-space allocation)
                unweighted_indices.push(i);
            }
        } else {
            unweighted_indices.push(i);
        }
    }
    (weighted_indices, unweighted_indices, total_weight)
}

fn measure_unweighted_children(
    input: &MeasureInput,
    unweighted_indices: &[usize],
    children_sizes: &mut [Option<ComputedData>],
    max_child_height: &mut Px,
    row_effective_constraint: &Constraint,
) -> Result<Px, MeasurementError> {
    // Measure all unweighted children and return their total width.
    // For each unweighted child, pass a Wrap-type constraint (max is the row's available maximum),
    // and update children_sizes and max_child_height.
    let mut total_width = Px(0);
    for &child_idx in unweighted_indices {
        let child_id = input.children_ids[child_idx];
        let parent_offered_constraint_for_child = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: row_effective_constraint.width.get_max(),
            },
            row_effective_constraint.height,
        );
        let child_result = input.measure_child(child_id, &parent_offered_constraint_for_child)?;
        children_sizes[child_idx] = Some(child_result);
        total_width += child_result.width;
        *max_child_height = (*max_child_height).max(child_result.height);
    }
    Ok(total_width)
}

fn measure_weighted_children(
    args: &mut MeasureWeightedChildrenArgs,
) -> Result<(), MeasurementError> {
    // Allocate remaining width proportionally for each weighted child and measure them:
    // - allocated_width = remaining_width * (child_weight / total_weight)
    // - measure the child using a Fixed(allocated_width) constraint
    // - update children_sizes and max_child_height
    if args.total_weight > 0.0 {
        for &child_idx in args.weighted_indices {
            let child_weight = args.child_weights[child_idx].unwrap_or(0.0);
            let allocated_width =
                Px((args.remaining_width.0 as f32 * (child_weight / args.total_weight)) as i32);
            let child_id = args.input.children_ids[child_idx];
            let parent_offered_constraint_for_child = Constraint::new(
                DimensionValue::Fixed(allocated_width),
                args.row_effective_constraint.height,
            );
            let child_result = args
                .input
                .measure_child(child_id, &parent_offered_constraint_for_child)?;
            args.children_sizes[child_idx] = Some(child_result);
            *args.max_child_height = (*args.max_child_height).max(child_result.height);
        }
    }
    Ok(())
}

fn calculate_final_row_width(
    row_effective_constraint: &Constraint,
    total_children_measured_width: Px,
) -> Px {
    // Decide the final width based on the row's width constraint type:
    // - Fixed: use the fixed width
    // - Fill: try to occupy the parent's available maximum width (limited by min)
    // - Wrap: use the total width of children, limited by min/max constraints
    match row_effective_constraint.width {
        DimensionValue::Fixed(w) => w,
        DimensionValue::Fill { min, max } => {
            if let Some(max) = max {
                let w = max;
                if let Some(min) = min { w.max(min) } else { w }
            } else {
                panic!(
                    "Seem that you are using Fill without max constraint, which is not supported in Row width."
                );
            }
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
    }
}

fn calculate_final_row_height(row_effective_constraint: &Constraint, max_child_height: Px) -> Px {
    // Calculate the final height based on the height constraint type:
    // - Fixed: use the fixed height
    // - Fill: use the maximum height available from the parent (limited by min)
    // - Wrap: use the maximum child height, limited by min/max
    match row_effective_constraint.height {
        DimensionValue::Fixed(h) => h,
        DimensionValue::Fill { min, max } => {
            if let Some(max_h) = max {
                let h = max_h;
                if let Some(min_h) = min {
                    h.max(min_h)
                } else {
                    h
                }
            } else {
                panic!(
                    "Seem that you are using Fill without max constraint, which is not supported in Row height."
                );
            }
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
    }
}

fn place_children_with_alignment(args: &PlaceChildrenArgs) {
    // Compute the initial x and spacing between children according to the main axis (horizontal),
    // then iterate measured children:
    // - use calculate_cross_axis_offset to compute each child's offset on the cross axis (vertical)
    // - place each child with place_node at the computed coordinates
    let (mut current_x, spacing) = calculate_main_axis_layout(args);

    for (i, child_size_opt) in args.children_sizes.iter().enumerate() {
        if let Some(child_actual_size) = child_size_opt {
            let child_id = args.children_ids[i];
            let y_offset = calculate_cross_axis_offset(
                child_actual_size,
                args.final_row_height,
                args.cross_axis_alignment,
            );

            place_node(
                child_id,
                PxPosition::new(current_x, y_offset),
                args.metadatas,
            );
            current_x += child_actual_size.width;
            if i < args.child_count - 1 {
                current_x += spacing;
            }
        }
    }
}

fn calculate_main_axis_layout(args: &PlaceChildrenArgs) -> (Px, Px) {
    // Calculate the start position on the main axis and the spacing between children:
    // Returns (start_x, spacing_between_children)
    let available_space = (args.final_row_width - args.total_children_width).max(Px(0));
    match args.main_axis_alignment {
        MainAxisAlignment::Start => (Px(0), Px(0)),
        MainAxisAlignment::Center => (available_space / 2, Px(0)),
        MainAxisAlignment::End => (available_space, Px(0)),
        MainAxisAlignment::SpaceEvenly => calculate_space_evenly(available_space, args.child_count),
        MainAxisAlignment::SpaceBetween => {
            calculate_space_between(available_space, args.child_count)
        }
        MainAxisAlignment::SpaceAround => calculate_space_around(available_space, args.child_count),
    }
}

fn calculate_space_evenly(available_space: Px, child_count: usize) -> (Px, Px) {
    if child_count > 0 {
        let s = available_space / (child_count as i32 + 1);
        (s, s)
    } else {
        (Px(0), Px(0))
    }
}

fn calculate_space_between(available_space: Px, child_count: usize) -> (Px, Px) {
    if child_count > 1 {
        (Px(0), available_space / (child_count as i32 - 1))
    } else if child_count == 1 {
        (available_space / 2, Px(0))
    } else {
        (Px(0), Px(0))
    }
}

fn calculate_space_around(available_space: Px, child_count: usize) -> (Px, Px) {
    if child_count > 0 {
        let s = available_space / (child_count as i32);
        (s / 2, s)
    } else {
        (Px(0), Px(0))
    }
}

fn calculate_cross_axis_offset(
    child_actual_size: &ComputedData,
    final_row_height: Px,
    cross_axis_alignment: CrossAxisAlignment,
) -> Px {
    // Compute child's offset on the cross axis (vertical):
    // - Start: align to top (0)
    // - Center: center (remaining_height / 2)
    // - End: align to bottom (remaining_height)
    // - Stretch: no offset (the child will be stretched to fill height; stretching handled in measurement)
    match cross_axis_alignment {
        CrossAxisAlignment::Start => Px(0),
        CrossAxisAlignment::Center => (final_row_height - child_actual_size.height).max(Px(0)) / 2,
        CrossAxisAlignment::End => (final_row_height - child_actual_size.height).max(Px(0)),
        CrossAxisAlignment::Stretch => Px(0),
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
