//! A vertical layout component.
//!
//! ## Usage
//!
//! Use to stack children vertically.
use derive_builder::Builder;
use tessera_ui::{
    ComponentNodeMetaDatas, ComputedData, Constraint, DimensionValue, MeasureInput,
    MeasurementError, NodeId, Px, PxPosition, place_node, tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

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
        ColumnArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

/// A scope for declaratively adding children to a `column` component.
pub struct ColumnScope<'a> {
    child_closures: &'a mut Vec<Box<dyn FnOnce() + Send + Sync>>,
    child_weights: &'a mut Vec<Option<f32>>,
}

impl<'a> ColumnScope<'a> {
    /// Adds a child component to the column.
    pub fn child<F>(&mut self, child_closure: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
        self.child_weights.push(None);
    }

    /// Adds a child component to the column with a specified weight for flexible space distribution.
    pub fn child_weighted<F>(&mut self, child_closure: F, weight: f32)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
        self.child_weights.push(Some(weight));
    }
}

/// # column
///
/// A layout component that arranges its children in a vertical column.
///
/// ## Usage
///
/// Stack components vertically, with options for alignment and flexible spacing.
///
/// ## Parameters
///
/// - `args` — configures the column's dimensions and alignment; see [`ColumnArgs`].
/// - `scope_config` — a closure that receives a [`ColumnScope`] for adding children.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::column::{column, ColumnArgs};
/// use tessera_ui_basic_components::text::{text, TextArgsBuilder};
/// use tessera_ui_basic_components::spacer::{spacer, SpacerArgs};
///
/// column(ColumnArgs::default(), |scope| {
///     scope.child(|| text(TextArgsBuilder::default().text("First item".to_string()).build().expect("builder construction failed")));
///     scope.child_weighted(|| spacer(SpacerArgs::default()), 1.0); // This spacer will be flexible
///     scope.child(|| text(TextArgsBuilder::default().text("Last item".to_string()).build().expect("builder construction failed")));
/// });
/// ```
#[tessera]
pub fn column<F>(args: ColumnArgs, scope_config: F)
where
    F: FnOnce(&mut ColumnScope),
{
    let mut child_closures: Vec<Box<dyn FnOnce() + Send + Sync>> = Vec::new();
    let mut child_weights: Vec<Option<f32>> = Vec::new();

    {
        let mut scope = ColumnScope {
            child_closures: &mut child_closures,
            child_weights: &mut child_weights,
        };
        scope_config(&mut scope);
    }

    let n = child_closures.len();

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            assert_eq!(
                input.children_ids.len(),
                n,
                "Mismatch between children defined in scope and runtime children count"
            );

            let column_intrinsic_constraint = Constraint::new(args.width, args.height);
            let column_effective_constraint =
                column_intrinsic_constraint.merge(input.parent_constraint);

            let mut children_sizes = vec![None; n];
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
                child_count: n,
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

    let parent_offered_constraint_for_child = Constraint::new(
        column_effective_constraint.width,
        DimensionValue::Wrap {
            min: None,
            max: column_effective_constraint.height.get_max(),
        },
    );

    let children_to_measure: Vec<_> = indices
        .iter()
        .map(|&child_idx| {
            (
                input.children_ids[child_idx],
                parent_offered_constraint_for_child,
            )
        })
        .collect();

    let children_results = input.measure_children(children_to_measure)?;

    for &child_idx in indices {
        let child_id = input.children_ids[child_idx];
        if let Some(child_result) = children_results.get(&child_id) {
            children_sizes[child_idx] = Some(*child_result);
            total += child_result.height;
            *max_child_width = (*max_child_width).max(child_result.width);
        }
    }

    Ok(total)
}

/// Measure weighted children by distributing the remaining height proportionally.
struct WeightedColumnMeasureContext<'a> {
    input: &'a MeasureInput<'a>,
    children_sizes: &'a mut [Option<ComputedData>],
    max_child_width: &'a mut Px,
    column_effective_constraint: &'a Constraint,
    child_weights: &'a [Option<f32>],
}

fn measure_weighted_children_for_column(
    ctx: WeightedColumnMeasureContext<'_>,
    weighted_indices: &[usize],
    remaining_height: Px,
    total_weight: f32,
) -> Result<(), MeasurementError> {
    if total_weight <= 0.0 {
        return Ok(());
    }

    let children_to_measure: Vec<_> = weighted_indices
        .iter()
        .map(|&child_idx| {
            let child_weight = ctx.child_weights[child_idx].unwrap_or(0.0);
            let allocated_height =
                Px((remaining_height.0 as f32 * (child_weight / total_weight)) as i32);
            let child_id = ctx.input.children_ids[child_idx];
            let parent_offered_constraint_for_child = Constraint::new(
                ctx.column_effective_constraint.width,
                DimensionValue::Fixed(allocated_height),
            );
            (child_id, parent_offered_constraint_for_child)
        })
        .collect();

    let children_results = ctx.input.measure_children(children_to_measure)?;

    for &child_idx in weighted_indices {
        let child_id = ctx.input.children_ids[child_idx];
        if let Some(child_result) = children_results.get(&child_id) {
            ctx.children_sizes[child_idx] = Some(*child_result);
            *ctx.max_child_width = (*ctx.max_child_width).max(child_result.width);
        }
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
            if let Some(max) = max {
                if let Some(min) = min {
                    max.max(min)
                } else {
                    max
                }
            } else {
                panic!(
                    "Seems that you are trying to use Fill without max in a non-infinite parent constraint. This is not supported. Parent constraint: {column_effective_constraint:?}"
                );
            }
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
        DimensionValue::Fill { min, max } => {
            if let Some(max) = max {
                if let Some(min) = min {
                    max.max(min)
                } else {
                    max
                }
            } else {
                panic!(
                    "Seems that you are trying to use Fill without max in a non-infinite parent constraint. This is not supported. Parent constraint: {parent_constraint:?}"
                );
            }
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
    let available_height_for_children = column_effective_constraint
        .height
        .get_max()
        .expect("Column height Fill expected with finite max constraint");

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
        WeightedColumnMeasureContext {
            input,
            children_sizes,
            max_child_width,
            column_effective_constraint,
            child_weights,
        },
        &weighted_children_indices,
        remaining_height_for_weighted_children,
        total_weight_sum,
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

    let parent_offered_constraint_for_child = Constraint::new(
        column_effective_constraint.width,
        DimensionValue::Wrap {
            min: None,
            max: column_effective_constraint.height.get_max(),
        },
    );

    let children_to_measure: Vec<_> = input
        .children_ids
        .iter()
        .map(|&child_id| (child_id, parent_offered_constraint_for_child))
        .collect();

    let children_results = input.measure_children(children_to_measure)?;

    for (i, &child_id) in input.children_ids.iter().enumerate().take(n) {
        if let Some(child_result) = children_results.get(&child_id) {
            children_sizes[i] = Some(*child_result);
            total_children_measured_height += child_result.height;
            *max_child_width = (*max_child_width).max(child_result.width);
        }
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
