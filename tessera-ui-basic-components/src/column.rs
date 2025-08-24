//! Provides a flexible vertical layout component for arranging child widgets in a single column.
//!
//! This module defines the `column` component, which stacks its children vertically and offers fine-grained control
//! over alignment, sizing, and flexible space distribution via weights. It is suitable for building user interfaces
//! where elements need to be organized from top to bottom, such as forms, lists, or grouped controls.
//!
//! Key features include:
//! - Main/cross axis alignment options
//! - Support for fixed, fill, or wrap sizing
//! - Weighted children for proportional space allocation
//! - Ergonomic macro for declarative usage
//!
//! Typical usage involves composing UI elements that should be laid out in a vertical sequence, with customizable
//! alignment and spacing behaviors.
use derive_builder::Builder;
use tessera_ui::{
    ComponentNodeMetaDatas, ComputedData, Constraint, DimensionValue, MeasureInput,
    MeasurementError, NodeId, Px, PxPosition, place_node, tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

pub use crate::column_ui;

/// Arguments for the `column` component.
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct ColumnArgs {
    /// Width behavior for the column.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub width: DimensionValue,
    /// Height behavior for the column.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
    /// Main axis alignment (vertical alignment).
    #[builder(default = "MainAxisAlignment::Start")]
    pub main_axis_alignment: MainAxisAlignment,
    /// Cross axis alignment (horizontal alignment).
    #[builder(default = "CrossAxisAlignment::Start")]
    pub cross_axis_alignment: CrossAxisAlignment,
}

impl Default for ColumnArgs {
    fn default() -> Self {
        ColumnArgsBuilder::default().build().unwrap()
    }
}

/// Represents a child item within a column layout.
pub struct ColumnItem {
    /// Optional weight for flexible space distribution
    pub weight: Option<f32>,
    /// The actual child component
    pub child: Box<dyn FnOnce() + Send + Sync>,
}

impl ColumnItem {
    /// Creates a new `ColumnItem` with optional weight.
    pub fn new(child: Box<dyn FnOnce() + Send + Sync>, weight: Option<f32>) -> Self {
        ColumnItem { weight, child }
    }

    /// Creates a weighted column item
    pub fn weighted(child: Box<dyn FnOnce() + Send + Sync>, weight: f32) -> Self {
        ColumnItem {
            weight: Some(weight),
            child,
        }
    }
}

/// Trait to allow various types to be converted into a `ColumnItem`.
pub trait AsColumnItem {
    fn into_column_item(self) -> ColumnItem;
}

impl AsColumnItem for ColumnItem {
    fn into_column_item(self) -> ColumnItem {
        self
    }
}

/// Default conversion: a simple function closure becomes a `ColumnItem` without weight.
impl<F: FnOnce() + Send + Sync + 'static> AsColumnItem for F {
    fn into_column_item(self) -> ColumnItem {
        ColumnItem {
            weight: None,
            child: Box::new(self),
        }
    }
}

/// Allow (FnOnce, weight) to be a ColumnItem
impl<F: FnOnce() + Send + Sync + 'static> AsColumnItem for (F, f32) {
    fn into_column_item(self) -> ColumnItem {
        ColumnItem {
            weight: Some(self.1),
            child: Box::new(self.0),
        }
    }
}

/// Helper struct used to place children with alignment. Local to this module.
struct PlaceChildrenArgs<'a> {
    children_sizes: &'a [Option<ComputedData>],
    children_ids: &'a [NodeId],
    metadatas: &'a ComponentNodeMetaDatas,
    final_column_width: Px,
    final_column_height: Px,
    total_children_height: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
}

/// Helper: classify children into weighted / unweighted and compute total weight.
fn classify_children(child_weights: &[Option<f32>]) -> (Vec<usize>, Vec<usize>, f32) {
    let mut weighted_indices = Vec::new();
    let mut unweighted_indices = Vec::new();
    let mut total_weight = 0.0;
    for (i, weight_opt) in child_weights.iter().enumerate() {
        if let Some(w) = weight_opt {
            if *w > 0.0 {
                weighted_indices.push(i);
                total_weight += w;
            } else {
                unweighted_indices.push(i);
            }
        } else {
            unweighted_indices.push(i);
        }
    }
    (weighted_indices, unweighted_indices, total_weight)
}

/// Measure all non-weighted children (vertical variant).
/// Returns the accumulated total height of those children.
fn measure_unweighted_children_for_column(
    input: &MeasureInput,
    indices: &[usize],
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
    column_effective_constraint: &Constraint,
) -> Result<Px, MeasurementError> {
    let mut total = Px(0);
    for &child_idx in indices {
        let child_id = input.children_ids[child_idx];
        let parent_offered_constraint_for_child = Constraint::new(
            column_effective_constraint.width,
            DimensionValue::Wrap {
                min: None,
                max: column_effective_constraint.height.get_max(),
            },
        );
        let child_result = input.measure_child(child_id, &parent_offered_constraint_for_child)?;
        children_sizes[child_idx] = Some(child_result);
        total += child_result.height;
        *max_child_width = (*max_child_width).max(child_result.width);
    }
    Ok(total)
}

/// Measure weighted children by distributing the remaining height proportionally.
fn measure_weighted_children_for_column(
    input: &MeasureInput,
    weighted_indices: &[usize],
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
    remaining_height: Px,
    total_weight: f32,
    column_effective_constraint: &Constraint,
    child_weights: &[Option<f32>],
) -> Result<(), MeasurementError> {
    if total_weight <= 0.0 {
        return Ok(());
    }
    for &child_idx in weighted_indices {
        let child_weight = child_weights[child_idx].unwrap_or(0.0);
        let allocated_height =
            Px((remaining_height.0 as f32 * (child_weight / total_weight)) as i32);
        let child_id = input.children_ids[child_idx];
        let parent_offered_constraint_for_child = Constraint::new(
            column_effective_constraint.width,
            DimensionValue::Fixed(allocated_height),
        );
        let child_result = input.measure_child(child_id, &parent_offered_constraint_for_child)?;
        children_sizes[child_idx] = Some(child_result);
        *max_child_width = (*max_child_width).max(child_result.width);
    }
    Ok(())
}

fn calculate_final_column_height(
    column_effective_constraint: &Constraint,
    measured_children_height: Px,
) -> Px {
    match column_effective_constraint.height {
        DimensionValue::Fixed(h) => h,
        DimensionValue::Fill { min, max } => {
            let mut h = measured_children_height;
            if let Some(min_h) = min {
                h = h.max(min_h);
            }
            if let Some(max_h) = max {
                h = h.min(max_h);
            } else {
                h = measured_children_height;
            }
            h
        }
        DimensionValue::Wrap { min, max } => {
            let mut h = measured_children_height;
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

fn calculate_final_column_width(
    column_effective_constraint: &Constraint,
    max_child_width: Px,
    parent_constraint: &Constraint,
) -> Px {
    match column_effective_constraint.width {
        DimensionValue::Fixed(w) => w,
        DimensionValue::Fill { min, .. } => {
            let mut w = parent_constraint.width.get_max().unwrap_or(max_child_width);
            if let Some(min_w) = min {
                w = w.max(min_w);
            }
            w
        }
        DimensionValue::Wrap { min, max } => {
            let mut w = max_child_width;
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

/// Measure column when height uses weighted allocation.
/// Returns (final_width, final_height, total_measured_children_height)
fn measure_weighted_column(
    input: &MeasureInput,
    _args: &ColumnArgs,
    child_weights: &[Option<f32>],
    column_effective_constraint: &Constraint,
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
) -> Result<(Px, Px, Px), MeasurementError> {
    let available_height_for_children = column_effective_constraint.height.get_max().unwrap();

    let (weighted_children_indices, unweighted_children_indices, total_weight_sum) =
        classify_children(child_weights);

    let total_height_of_unweighted_children = measure_unweighted_children_for_column(
        input,
        &unweighted_children_indices,
        children_sizes,
        max_child_width,
        column_effective_constraint,
    )?;

    let remaining_height_for_weighted_children =
        (available_height_for_children - total_height_of_unweighted_children).max(Px(0));

    measure_weighted_children_for_column(
        input,
        &weighted_children_indices,
        children_sizes,
        max_child_width,
        remaining_height_for_weighted_children,
        total_weight_sum,
        column_effective_constraint,
        child_weights,
    )?;

    let total_measured_children_height: Px = children_sizes
        .iter()
        .filter_map(|s| s.as_ref().map(|s| s.height))
        .fold(Px(0), |acc, h| acc + h);

    let final_column_height =
        calculate_final_column_height(column_effective_constraint, total_measured_children_height);
    let final_column_width = calculate_final_column_width(
        column_effective_constraint,
        *max_child_width,
        input.parent_constraint,
    );

    Ok((
        final_column_width,
        final_column_height,
        total_measured_children_height,
    ))
}

fn measure_unweighted_column(
    input: &MeasureInput,
    _args: &ColumnArgs,
    column_effective_constraint: &Constraint,
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
) -> Result<(Px, Px, Px), MeasurementError> {
    let n = children_sizes.len();
    let mut total_children_measured_height = Px(0);
    for i in 0..n {
        let child_id = input.children_ids[i];
        let parent_offered_constraint_for_child = Constraint::new(
            column_effective_constraint.width,
            DimensionValue::Wrap {
                min: None,
                max: column_effective_constraint.height.get_max(),
            },
        );
        let child_result = input.measure_child(child_id, &parent_offered_constraint_for_child)?;
        children_sizes[i] = Some(child_result);
        total_children_measured_height += child_result.height;
        *max_child_width = (*max_child_width).max(child_result.width);
    }
    let final_column_height =
        calculate_final_column_height(column_effective_constraint, total_children_measured_height);
    let final_column_width = calculate_final_column_width(
        column_effective_constraint,
        *max_child_width,
        input.parent_constraint,
    );
    Ok((
        final_column_width,
        final_column_height,
        total_children_measured_height,
    ))
}

/// A column component that arranges its children vertically.
///
/// The `column` stacks its children from top to bottom and provides control over sizing,
/// alignment and proportional space distribution via weights. Use `ColumnArgs` to configure
/// the column's width/height behavior and alignment policies.
///
/// Children may be provided as:
/// - closures (converted to `ColumnItem` with no weight),
/// - `ColumnItem` instances, or
/// - `(closure, weight)` tuples to allocate proportional space.
///
/// # Parameters
/// - `args`: configuration for width/height and alignment (`ColumnArgs`).
/// - `children_items_input`: array of items convertible to `ColumnItem` via `AsColumnItem`.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::column::{column_ui, ColumnArgsBuilder};
/// use tessera_ui_basic_components::text::text;
///
/// column_ui!(
///     ColumnArgsBuilder::default()
///         .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
///         .cross_axis_alignment(CrossAxisAlignment::Center)
///         .build()
///         .unwrap(),
///     || text("First item".to_string()),
///     (|| text("Weighted".to_string()), 0.5),
///     ColumnItem::new(Box::new(|| text("Last".to_string())), None)
/// );
/// ```
///
/// Note: This function registers a measurement closure that measures children according
/// to column constraints and then places them using `place_node`.
#[tessera]
pub fn column<const N: usize>(args: ColumnArgs, children_items_input: [impl AsColumnItem; N]) {
    let children_items: [ColumnItem; N] =
        children_items_input.map(|item_input| item_input.into_column_item());

    let mut child_closures = Vec::with_capacity(N);
    let mut child_weights = Vec::with_capacity(N);

    for child_item in children_items {
        child_closures.push(child_item.child);
        child_weights.push(child_item.weight);
    }

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let column_intrinsic_constraint = Constraint::new(args.width, args.height);
            let column_effective_constraint =
                column_intrinsic_constraint.merge(input.parent_constraint);

            let mut children_sizes = vec![None; N];
            let mut max_child_width = Px(0);

            let should_use_weight_for_height = matches!(
                column_effective_constraint.height,
                DimensionValue::Fixed(_)
                    | DimensionValue::Fill { max: Some(_), .. }
                    | DimensionValue::Wrap { max: Some(_), .. }
            );

            let (final_column_width, final_column_height, total_measured_children_height) =
                if should_use_weight_for_height {
                    measure_weighted_column(
                        input,
                        &args,
                        &child_weights,
                        &column_effective_constraint,
                        &mut children_sizes,
                        &mut max_child_width,
                    )?
                } else {
                    measure_unweighted_column(
                        input,
                        &args,
                        &column_effective_constraint,
                        &mut children_sizes,
                        &mut max_child_width,
                    )?
                };

            place_children_with_alignment(&PlaceChildrenArgs {
                children_sizes: &children_sizes,
                children_ids: input.children_ids,
                metadatas: input.metadatas,
                final_column_width,
                final_column_height,
                total_children_height: total_measured_children_height,
                main_axis_alignment: args.main_axis_alignment,
                cross_axis_alignment: args.cross_axis_alignment,
                child_count: N,
            });

            Ok(ComputedData {
                width: final_column_width,
                height: final_column_height,
            })
        },
    ));

    for child_closure in child_closures {
        child_closure();
    }
}

/// Place measured children into the column according to main and cross axis alignment.
///
/// This helper computes the starting y position and spacing between children based on
/// `MainAxisAlignment` variants (Start, Center, End, SpaceEvenly, SpaceBetween, SpaceAround)
/// and aligns each child horizontally using `CrossAxisAlignment`. It calls `place_node` to
/// record each child's layout position.
///
/// `args` contains measured child sizes, node ids, component metadata and final column size.
fn place_children_with_alignment(args: &PlaceChildrenArgs) {
    let (mut current_y, spacing_between_children) = calculate_main_axis_layout_for_column(
        args.final_column_height,
        args.total_children_height,
        args.main_axis_alignment,
        args.child_count,
    );

    for (i, child_size_opt) in args.children_sizes.iter().enumerate() {
        if let Some(child_actual_size) = child_size_opt {
            let child_id = args.children_ids[i];
            let x_offset = calculate_cross_axis_offset_for_column(
                child_actual_size,
                args.final_column_width,
                args.cross_axis_alignment,
            );
            place_node(
                child_id,
                PxPosition::new(x_offset, current_y),
                args.metadatas,
            );
            current_y += child_actual_size.height;
            if i < args.child_count - 1 {
                current_y += spacing_between_children;
            }
        }
    }
}

fn calculate_main_axis_layout_for_column(
    final_column_height: Px,
    total_children_height: Px,
    main_axis_alignment: MainAxisAlignment,
    child_count: usize,
) -> (Px, Px) {
    let available_space = (final_column_height - total_children_height).max(Px(0));
    match main_axis_alignment {
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
    }
}

fn calculate_cross_axis_offset_for_column(
    child_actual_size: &ComputedData,
    final_column_width: Px,
    cross_axis_alignment: CrossAxisAlignment,
) -> Px {
    match cross_axis_alignment {
        CrossAxisAlignment::Start => Px(0),
        CrossAxisAlignment::Center => (final_column_width - child_actual_size.width).max(Px(0)) / 2,
        CrossAxisAlignment::End => (final_column_width - child_actual_size.width).max(Px(0)),
        CrossAxisAlignment::Stretch => Px(0),
    }
}

/// A declarative macro to simplify the creation of a [`column`](crate::column::column) component.
///
/// The first argument is the `ColumnArgs` struct, followed by a variable number of
/// child components. Each child expression will be converted to a `ColumnItem`
/// using the `AsColumnItem` trait. This allows passing closures, `ColumnItem` instances,
/// or `(FnOnce, weight)` tuples.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::{column::{column_ui, ColumnArgs, ColumnItem}, text::text};
///
/// column_ui!(
///     ColumnArgs::default(),
///     || text("Hello".to_string()), // Closure
///     (|| text("Weighted".to_string()), 0.5), // Weighted closure
///     ColumnItem::new(Box::new(|| text("Item".to_string())), None) // ColumnItem instance
/// );
/// ```
#[macro_export]
macro_rules! column_ui {
    ($args:expr $(, $child:expr)* $(,)?) => {
        {
            use $crate::column::AsColumnItem;
            $crate::column::column($args, [
                $(
                    $child.into_column_item()
                ),*
            ])
        }
    };
}
