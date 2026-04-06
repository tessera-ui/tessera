//! A horizontal layout component.
//!
//! ## Usage
//!
//! Use to stack children horizontally.
use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, LayoutPolicy, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, RenderSlot,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

struct PlaceChildrenArgs<'a> {
    children_sizes: &'a [Option<ComputedData>],
    children: &'a [tessera_ui::layout::LayoutChild<'a>],
    final_row_width: Px,
    final_row_height: Px,
    total_children_width: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
}

struct MeasureWeightedChildrenArgs<'a> {
    input: &'a MeasureScope<'a>,
    weighted_indices: &'a [usize],
    children_sizes: &'a mut [Option<ComputedData>],
    max_child_height: &'a mut Px,
    remaining_width: Px,
    total_weight: f32,
    row_effective_constraint: &'a Constraint,
    child_weights: &'a [f32],
}

#[derive(Clone, PartialEq)]
struct RowLayout {
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
}

impl LayoutPolicy for RowLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_weights = collect_child_weights(input);
        let n = child_weights.len();
        assert_eq!(
            children.len(),
            n,
            "Mismatch between children defined in scope and runtime children count"
        );

        let row_effective_constraint = *input.parent_constraint().as_ref();

        let has_weighted_children = child_weights.iter().any(|&weight| weight > 0.0);
        let should_use_weight_for_width =
            has_weighted_children && row_effective_constraint.width.resolve_max().is_some();

        if should_use_weight_for_width {
            measure_weighted_row(
                input,
                &mut result,
                self.main_axis_alignment,
                self.cross_axis_alignment,
                &child_weights,
                &row_effective_constraint,
            )
        } else {
            measure_unweighted_row(
                input,
                &mut result,
                self.main_axis_alignment,
                self.cross_axis_alignment,
                &row_effective_constraint,
            )
        }
    }
}

/// # row
///
/// A layout component that arranges its children in a horizontal row.
///
/// ## Usage
///
/// Stack components horizontally, with options for alignment and flexible
/// spacing.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the row container.
/// - `main_axis_alignment` — alignment along the horizontal axis.
/// - `cross_axis_alignment` — alignment along the vertical axis.
/// - `children` — child slot rendered inside the row.
///
/// ## Examples
///
/// ```
/// use tessera_components::{modifier::ModifierExt as _, row::row, spacer::spacer, text::text};
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// row().children(|| {
///     text().content("First");
///     spacer().modifier(Modifier::new().weight(1.0));
///     text().content("Last");
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn row(
    modifier: Option<Modifier>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    children: RenderSlot,
) {
    let modifier = modifier.unwrap_or_default();
    layout()
        .modifier(modifier)
        .layout_policy(RowLayout {
            main_axis_alignment,
            cross_axis_alignment,
        })
        .child(move || {
            children.render();
        });
}

fn measure_weighted_row(
    input: &MeasureScope<'_>,
    result: &mut LayoutResult,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_weights: &[f32],
    row_effective_constraint: &Constraint,
) -> Result<LayoutResult, MeasurementError> {
    let children = input.children();
    // Prepare buffers and metadata for measurement:
    // - `children_sizes` stores each child's measurement result (width, height).
    // - `max_child_height` tracks the maximum height among children to compute the
    //   row's final height.
    // - `available_width_for_children` is the total width available to allocate to
    //   children under the current constraint (present only for
    //   Fill/Fixed/Wrap(max)).
    let mut children_sizes = vec![None; child_weights.len()];
    let mut max_child_height = Px(0);
    let available_width_for_children = row_effective_constraint
        .width
        .resolve_max()
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

    place_children_with_alignment(
        &PlaceChildrenArgs {
            children_sizes: &children_sizes,
            children: &children,
            final_row_width,
            final_row_height,
            total_children_width: total_measured_children_width,
            main_axis_alignment,
            cross_axis_alignment,
            child_count: child_weights.len(),
        },
        result,
    );

    result.size = ComputedData {
        width: final_row_width,
        height: final_row_height,
    };
    Ok(result.clone())
}

fn measure_unweighted_row(
    input: &MeasureScope<'_>,
    result: &mut LayoutResult,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    row_effective_constraint: &Constraint,
) -> Result<LayoutResult, MeasurementError> {
    let children = input.children();
    let mut children_sizes = vec![None; children.len()];
    let mut total_children_measured_width = Px(0);
    let mut max_child_height = Px(0);

    let parent_offered_constraint_for_child = Constraint::new(
        row_effective_constraint.width.without_min(),
        row_effective_constraint.height,
    );

    let children_to_measure: Vec<_> = input
        .children()
        .iter()
        .map(|child| (*child, parent_offered_constraint_for_child))
        .collect();

    let children_results = input.measure_children(children_to_measure)?;

    for (i, child_id) in children.iter().enumerate() {
        if let Some(child_result) = children_results.get(child_id) {
            children_sizes[i] = Some(child_result.size());
            total_children_measured_width += child_result.width;
            max_child_height = max_child_height.max(child_result.height);
        }
    }

    let final_row_width =
        calculate_final_row_width(row_effective_constraint, total_children_measured_width);
    let final_row_height = calculate_final_row_height(row_effective_constraint, max_child_height);

    place_children_with_alignment(
        &PlaceChildrenArgs {
            children_sizes: &children_sizes,
            children: &children,
            final_row_width,
            final_row_height,
            total_children_width: total_children_measured_width,
            main_axis_alignment,
            cross_axis_alignment,
            child_count: children_sizes.len(),
        },
        result,
    );

    result.size = ComputedData {
        width: final_row_width,
        height: final_row_height,
    };
    Ok(result.clone())
}

fn classify_children(child_weights: &[f32]) -> (Vec<usize>, Vec<usize>, f32) {
    // Split children into weighted and unweighted categories and compute the total
    // weight of weighted children. Returns: (weighted_indices,
    // unweighted_indices, total_weight)
    let mut weighted_indices = Vec::new();
    let mut unweighted_indices = Vec::new();
    let mut total_weight = 0.0;

    for (i, &weight) in child_weights.iter().enumerate() {
        if weight > 0.0 {
            weighted_indices.push(i);
            total_weight += weight;
        } else {
            unweighted_indices.push(i);
        }
    }
    (weighted_indices, unweighted_indices, total_weight)
}

fn measure_unweighted_children(
    input: &MeasureScope<'_>,
    unweighted_indices: &[usize],
    children_sizes: &mut [Option<ComputedData>],
    max_child_height: &mut Px,
    row_effective_constraint: &Constraint,
) -> Result<Px, MeasurementError> {
    let mut total_width = Px(0);

    let parent_offered_constraint_for_child = Constraint::new(
        row_effective_constraint.width.without_min(),
        row_effective_constraint.height,
    );

    let children_to_measure: Vec<_> = unweighted_indices
        .iter()
        .map(|&child_idx| {
            (
                input.children()[child_idx],
                parent_offered_constraint_for_child,
            )
        })
        .collect();

    let children_results = input.measure_children(children_to_measure)?;

    for &child_idx in unweighted_indices {
        let child_id = input.children()[child_idx];
        if let Some(child_result) = children_results.get(&child_id) {
            children_sizes[child_idx] = Some(child_result.size());
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
            let child_weight = args.child_weights[child_idx];
            let allocated_width =
                Px((args.remaining_width.0 as f32 * (child_weight / args.total_weight)) as i32);
            let child_id = args.input.children()[child_idx];
            let parent_offered_constraint_for_child = Constraint::new(
                AxisConstraint::exact(allocated_width),
                args.row_effective_constraint.height,
            );
            (child_id, parent_offered_constraint_for_child)
        })
        .collect();

    let children_results = args.input.measure_children(children_to_measure)?;

    for &child_idx in args.weighted_indices {
        let child_id = args.input.children()[child_idx];
        if let Some(child_result) = children_results.get(&child_id) {
            args.children_sizes[child_idx] = Some(child_result.size());
            *args.max_child_height = (*args.max_child_height).max(child_result.height);
        }
    }

    Ok(())
}

fn collect_child_weights(input: &MeasureScope<'_>) -> Vec<f32> {
    input
        .children()
        .iter()
        .map(|child_id| {
            child_id
                .parent_data::<crate::modifier::WeightParentData>()
                .map(|data| data.weight)
                .unwrap_or(0.0)
        })
        .collect()
}

fn calculate_final_row_width(
    row_effective_constraint: &Constraint,
    total_children_measured_width: Px,
) -> Px {
    row_effective_constraint
        .width
        .clamp(total_children_measured_width)
}

fn calculate_final_row_height(row_effective_constraint: &Constraint, max_child_height: Px) -> Px {
    row_effective_constraint.height.clamp(max_child_height)
}

fn place_children_with_alignment(args: &PlaceChildrenArgs, result: &mut LayoutResult) {
    // Compute the initial x and spacing between children according to the main axis
    // (horizontal), then iterate measured children:
    // - use calculate_cross_axis_offset to compute each child's offset on the cross
    //   axis (vertical)
    // - place each child with place_node at the computed coordinates
    let (mut current_x, spacing) = calculate_main_axis_layout(args);

    for (i, child_size_opt) in args.children_sizes.iter().enumerate() {
        if let Some(child_actual_size) = child_size_opt {
            let child_id = args.children[i];
            let y_offset = calculate_cross_axis_offset(
                child_actual_size,
                args.final_row_height,
                args.cross_axis_alignment,
            );

            result.place_child(child_id, PxPosition::new(current_x, y_offset));
            current_x += child_actual_size.width;
            if i < args.child_count - 1 {
                current_x += spacing;
            }
        }
    }
}

fn calculate_main_axis_layout(args: &PlaceChildrenArgs) -> (Px, Px) {
    // Calculate the start position on the main axis and the spacing between
    // children: Returns (start_x, spacing_between_children)
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
    // - Stretch: no offset (the child will be stretched to fill height; stretching
    //   handled in measurement)
    match cross_axis_alignment {
        CrossAxisAlignment::Start => Px(0),
        CrossAxisAlignment::Center => (final_row_height - child_actual_size.height).max(Px(0)) / 2,
        CrossAxisAlignment::End => (final_row_height - child_actual_size.height).max(Px(0)),
        CrossAxisAlignment::Stretch => Px(0),
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        AxisConstraint, ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier,
        NoopRenderPolicy, Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        modifier::{ModifierExt as _, SemanticsArgs},
    };

    use super::row;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    #[derive(Clone, PartialEq)]
    struct FillWidthTestLayout {
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    impl LayoutPolicy for FillWidthTestLayout {
        fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let width = input
                .parent_constraint()
                .width()
                .resolve_max()
                .expect("FillWidthTestLayout requires a bounded width constraint");

            Ok(LayoutResult::new(ComputedData {
                width,
                height: Px::new(self.height),
            }))
        }
    }

    #[tessera]
    fn fixed_test_box(tag: String, width: i32, height: i32) {
        layout()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn fill_width_test_box(tag: String, height: i32) {
        layout()
            .layout_policy(FillWidthTestLayout { height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn row_layout_case() {
        row()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(100))),
                Some(AxisConstraint::exact(Px::new(30))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(|| {
                row_fixed_box();
                layout().modifier(Modifier::new().weight(1.0)).child(|| {
                    row_weighted_content_box();
                });
            });
    }

    #[tessera]
    fn row_fixed_box() {
        fixed_test_box()
            .tag("row_fixed".to_string())
            .width(20)
            .height(10);
    }

    #[tessera]
    fn row_weighted_content_box() {
        fill_width_test_box()
            .tag("row_weighted_content".to_string())
            .height(10);
    }

    #[tessera]
    fn row_alignment_case() {
        row()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(100))),
                Some(AxisConstraint::exact(Px::new(30))),
            ))
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::End)
            .children(|| {
                fixed_test_box()
                    .tag("row_center_first".to_string())
                    .width(20)
                    .height(10);
                fixed_test_box()
                    .tag("row_center_second".to_string())
                    .width(10)
                    .height(12);
            });
    }

    #[test]
    fn row_allocates_remaining_width_to_weighted_child() {
        tessera_ui::assert_layout! {
            viewport: (120, 60),
            content: {
                row_layout_case();
            },
            expect: {
                node("row_fixed").position(0, 0).size(20, 10);
                node("row_weighted_content").position(20, 0).size(80, 10);
            }
        }
    }

    #[test]
    fn row_honors_main_and_cross_axis_alignment() {
        tessera_ui::assert_layout! {
            viewport: (120, 60),
            content: {
                row_alignment_case();
            },
            expect: {
                node("row_center_first").position(35, 20).size(20, 10);
                node("row_center_second").position(55, 18).size(10, 12);
            }
        }
    }
}
