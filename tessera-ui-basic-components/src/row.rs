//! A horizontal layout component.
//!
//! ## Usage
//!
//! Use to stack children horizontally.
use derive_builder::Builder;
use tessera_ui::{
    ComponentNodeMetaDatas, ComputedData, Constraint, DimensionValue, MeasureInput,
    MeasurementError, NodeId, Px, PxPosition, place_node, tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

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
        RowArgsBuilder::default().build().expect("builder construction failed")
    }
}

/// A scope for declaratively adding children to a `row` component.
pub struct RowScope<'a> {
    child_closures: &'a mut Vec<Box<dyn FnOnce() + Send + Sync>>,
    child_weights: &'a mut Vec<Option<f32>>,
}

impl<'a> RowScope<'a> {
    /// Adds a child component to the row.
    pub fn child<F>(&mut self, child_closure: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
        self.child_weights.push(None);
    }

    /// Adds a child component to the row with a specified weight for flexible space distribution.
    pub fn child_weighted<F>(&mut self, child_closure: F, weight: f32)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
        self.child_weights.push(Some(weight));
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

/// # row
///
/// A layout component that arranges its children in a horizontal row.
///
/// ## Usage
///
/// Stack components horizontally, with options for alignment and flexible spacing.
///
/// ## Parameters
///
/// - `args` — configures the row's dimensions and alignment; see [`RowArgs`].
/// - `scope_config` — a closure that receives a [`RowScope`] for adding children.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::{
///     row::{row, RowArgs},
///     text::{text, TextArgsBuilder},
///     spacer::{spacer, SpacerArgs},
/// };
///
/// row(RowArgs::default(), |scope| {
///     scope.child(|| text(TextArgsBuilder::default().text("First".to_string()).build().expect("builder construction failed")));
///     scope.child_weighted(|| spacer(SpacerArgs::default()), 1.0); // Flexible space
///     scope.child(|| text(TextArgsBuilder::default().text("Last".to_string()).build().expect("builder construction failed")));
/// });
/// ```
#[tessera]
pub fn row<F>(args: RowArgs, scope_config: F)
where
    F: FnOnce(&mut RowScope),
{
    let mut child_closures: Vec<Box<dyn FnOnce() + Send + Sync>> = Vec::new();
    let mut child_weights: Vec<Option<f32>> = Vec::new();

    {
        let mut scope = RowScope {
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

            let row_intrinsic_constraint = Constraint::new(args.width, args.height);
            let row_effective_constraint = row_intrinsic_constraint.merge(input.parent_constraint);

            let should_use_weight_for_width = matches!(
                row_effective_constraint.width,
                DimensionValue::Fixed(_)
                    | DimensionValue::Fill { max: Some(_), .. }
                    | DimensionValue::Wrap { max: Some(_), .. }
            );

            if should_use_weight_for_width {
                measure_weighted_row(input, &args, &child_weights, &row_effective_constraint, n)
            } else {
                measure_unweighted_row(input, &args, &row_effective_constraint, n)
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
    n: usize,
) -> Result<ComputedData, MeasurementError> {
    // Prepare buffers and metadata for measurement:
    // - `children_sizes` stores each child's measurement result (width, height).
    // - `max_child_height` tracks the maximum height among children to compute the row's final height.
    // - `available_width_for_children` is the total width available to allocate to children under the current constraint (present only for Fill/Fixed/Wrap(max)).
    let mut children_sizes = vec![None; n];
    let mut max_child_height = Px(0);
    let available_width_for_children = row_effective_constraint
        .width
        .get_max()
        .expect("Row width Fill expected with finite max constraint");

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
    n: usize,
) -> Result<ComputedData, MeasurementError> {
    let mut children_sizes = vec![None; n];
    let mut total_children_measured_width = Px(0);
    let mut max_child_height = Px(0);

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

    let children_to_measure: Vec<_> = input
        .children_ids
        .iter()
        .map(|&child_id| (child_id, parent_offered_constraint_for_child))
        .collect();

    let children_results = input.measure_children(children_to_measure)?;

    for (i, &child_id) in input.children_ids.iter().enumerate().take(n) {
        if let Some(child_result) = children_results.get(&child_id) {
            children_sizes[i] = Some(*child_result);
            total_children_measured_width += child_result.width;
            max_child_height = max_child_height.max(child_result.height);
        }
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
    let mut total_width = Px(0);

    let parent_offered_constraint_for_child = Constraint::new(
        DimensionValue::Wrap {
            min: None,
            max: row_effective_constraint.width.get_max(),
        },
        row_effective_constraint.height,
    );

    let children_to_measure: Vec<_> = unweighted_indices
        .iter()
        .map(|&child_idx| {
            (
                input.children_ids[child_idx],
                parent_offered_constraint_for_child,
            )
        })
        .collect();

    let children_results = input.measure_children(children_to_measure)?;

    for &child_idx in unweighted_indices {
        let child_id = input.children_ids[child_idx];
        if let Some(child_result) = children_results.get(&child_id) {
            children_sizes[child_idx] = Some(*child_result);
            total_width += child_result.width;
            *max_child_height = (*max_child_height).max(child_result.height);
        }
    }

    Ok(total_width)
}

fn measure_weighted_children(
    args: &mut MeasureWeightedChildrenArgs,
) -> Result<(), MeasurementError> {
    if args.total_weight <= 0.0 {
        return Ok(());
    }

    let children_to_measure: Vec<_> = args
        .weighted_indices
        .iter()
        .map(|&child_idx| {
            let child_weight = args.child_weights[child_idx].unwrap_or(0.0);
            let allocated_width =
                Px((args.remaining_width.0 as f32 * (child_weight / args.total_weight)) as i32);
            let child_id = args.input.children_ids[child_idx];
            let parent_offered_constraint_for_child = Constraint::new(
                DimensionValue::Fixed(allocated_width),
                args.row_effective_constraint.height,
            );
            (child_id, parent_offered_constraint_for_child)
        })
        .collect();

    let children_results = args.input.measure_children(children_to_measure)?;

    for &child_idx in args.weighted_indices {
        let child_id = args.input.children_ids[child_idx];
        if let Some(child_result) = children_results.get(&child_id) {
            args.children_sizes[child_idx] = Some(*child_result);
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

