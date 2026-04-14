//! A vertical layout component.
//!
//! ## Usage
//!
//! Use to stack children vertically.
use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, LayoutPolicy, LayoutResult, MeasurementError,
    Modifier, ParentConstraint, Px, PxPosition, RenderSlot,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

/// # column
///
/// A layout component that arranges its children in a vertical column.
///
/// ## Usage
///
/// Stack components vertically, with options for alignment and flexible
/// spacing.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the column container.
/// - `main_axis_alignment` — alignment along the vertical axis.
/// - `cross_axis_alignment` — alignment along the horizontal axis.
/// - `children` — child slot rendered inside the column.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     column::column, modifier::ModifierExt as _, spacer::spacer, text::text,
/// };
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// column().children(|| {
///     text().content("First item");
///     spacer().modifier(Modifier::new().weight(1.0));
///     text().content("Last item");
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn column(
    modifier: Option<Modifier>,
    main_axis_alignment: Option<MainAxisAlignment>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    children: Option<RenderSlot>,
) {
    let modifier = modifier.unwrap_or_default();
    let main_axis_alignment = main_axis_alignment.unwrap_or_default();
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let children = children.unwrap_or_else(RenderSlot::empty);
    layout()
        .modifier(modifier)
        .layout_policy(ColumnLayout {
            main_axis_alignment,
            cross_axis_alignment,
        })
        .child(move || {
            children.render();
        });
}

#[derive(Clone, PartialEq)]
struct ColumnLayout {
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
}

impl LayoutPolicy for ColumnLayout {
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

        let column_parent_constraint = *input.parent_constraint().as_ref();

        let mut children_sizes = vec![None; n];
        let mut max_child_width = Px(0);

        let has_weighted_children = child_weights.iter().any(|&weight| weight > 0.0);
        let should_use_weight_for_height =
            has_weighted_children && column_parent_constraint.height.resolve_max().is_some();

        let (final_column_width, final_column_height, total_measured_children_height) =
            if should_use_weight_for_height {
                measure_weighted_column(
                    input,
                    &child_weights,
                    &column_parent_constraint,
                    &mut children_sizes,
                    &mut max_child_width,
                )?
            } else {
                measure_unweighted_column(
                    input,
                    &column_parent_constraint,
                    &mut children_sizes,
                    &mut max_child_width,
                )?
            };

        place_children_with_alignment(
            &PlaceChildrenArgs {
                children_sizes: &children_sizes,
                children: &children,
                final_column_width,
                final_column_height,
                total_children_height: total_measured_children_height,
                main_axis_alignment: self.main_axis_alignment,
                cross_axis_alignment: self.cross_axis_alignment,
                child_count: n,
            },
            &mut result,
        );

        Ok(result.with_size(ComputedData {
            width: final_column_width,
            height: final_column_height,
        }))
    }
}

/// Helper struct used to place children with alignment. Local to this module.
struct PlaceChildrenArgs<'a> {
    children_sizes: &'a [Option<ComputedData>],
    children: &'a [tessera_ui::layout::LayoutChild<'a>],
    final_column_width: Px,
    final_column_height: Px,
    total_children_height: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
}

/// Helper: classify children into weighted / unweighted and compute total
/// weight.
fn classify_children(child_weights: &[f32]) -> (Vec<usize>, Vec<usize>, f32) {
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

/// Measure all non-weighted children (vertical variant).
/// Returns the accumulated total height of those children.
fn measure_unweighted_children_for_column(
    input: &MeasureScope<'_>,
    indices: &[usize],
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
    column_parent_constraint: &Constraint,
) -> Result<Px, MeasurementError> {
    let mut total = Px(0);

    let parent_offered_constraint_for_child = Constraint::new(
        column_parent_constraint.width,
        column_parent_constraint.height.without_min(),
    );

    for &child_idx in indices {
        let child_id = input.children()[child_idx];
        let child_result = child_id.measure(&parent_offered_constraint_for_child)?;
        children_sizes[child_idx] = Some(child_result.size());
        total += child_result.height;
        *max_child_width = (*max_child_width).max(child_result.width);
    }

    Ok(total)
}

/// Measure weighted children by distributing the remaining height
/// proportionally.
struct WeightedColumnMeasureContext<'a> {
    input: &'a MeasureScope<'a>,
    children_sizes: &'a mut [Option<ComputedData>],
    max_child_width: &'a mut Px,
    column_parent_constraint: &'a Constraint,
    child_weights: &'a [f32],
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

    for &child_idx in weighted_indices {
        let child_weight = ctx.child_weights[child_idx];
        let allocated_height =
            Px((remaining_height.0 as f32 * (child_weight / total_weight)) as i32);
        let child_id = ctx.input.children()[child_idx];
        let parent_offered_constraint_for_child = Constraint::new(
            ctx.column_parent_constraint.width,
            AxisConstraint::exact(allocated_height),
        );
        let child_result = child_id.measure(&parent_offered_constraint_for_child)?;
        ctx.children_sizes[child_idx] = Some(child_result.size());
        *ctx.max_child_width = (*ctx.max_child_width).max(child_result.width);
    }

    Ok(())
}

fn calculate_final_column_height(
    column_parent_constraint: &Constraint,
    measured_children_height: Px,
) -> Px {
    column_parent_constraint
        .height
        .clamp(measured_children_height)
}

fn calculate_final_column_width(
    column_parent_constraint: &Constraint,
    max_child_width: Px,
    parent_constraint: ParentConstraint<'_>,
) -> Px {
    let _ = parent_constraint;
    column_parent_constraint.width.clamp(max_child_width)
}

/// Measure column when height uses weighted allocation.
/// Returns (final_width, final_height, total_measured_children_height)
fn measure_weighted_column(
    input: &MeasureScope<'_>,
    child_weights: &[f32],
    column_parent_constraint: &Constraint,
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
) -> Result<(Px, Px, Px), MeasurementError> {
    let available_height_for_children = column_parent_constraint
        .height
        .resolve_max()
        .expect("Column height Fill expected with finite max constraint");

    let (weighted_children_indices, unweighted_children_indices, total_weight_sum) =
        classify_children(child_weights);

    let total_height_of_unweighted_children = measure_unweighted_children_for_column(
        input,
        &unweighted_children_indices,
        children_sizes,
        max_child_width,
        column_parent_constraint,
    )?;

    let remaining_height_for_weighted_children =
        (available_height_for_children - total_height_of_unweighted_children).max(Px(0));

    measure_weighted_children_for_column(
        WeightedColumnMeasureContext {
            input,
            children_sizes,
            max_child_width,
            column_parent_constraint,
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
        calculate_final_column_height(column_parent_constraint, total_measured_children_height);
    let final_column_width = calculate_final_column_width(
        column_parent_constraint,
        *max_child_width,
        input.parent_constraint(),
    );

    Ok((
        final_column_width,
        final_column_height,
        total_measured_children_height,
    ))
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

fn measure_unweighted_column(
    input: &MeasureScope<'_>,
    column_parent_constraint: &Constraint,
    children_sizes: &mut [Option<ComputedData>],
    max_child_width: &mut Px,
) -> Result<(Px, Px, Px), MeasurementError> {
    let n = children_sizes.len();
    let mut total_children_measured_height = Px(0);

    let parent_offered_constraint_for_child = Constraint::new(
        column_parent_constraint.width,
        column_parent_constraint.height.without_min(),
    );

    for (i, child_id) in input.children().iter().enumerate().take(n) {
        let child_result = child_id.measure(&parent_offered_constraint_for_child)?;
        children_sizes[i] = Some(child_result.size());
        total_children_measured_height += child_result.height;
        *max_child_width = (*max_child_width).max(child_result.width);
    }

    let final_column_height =
        calculate_final_column_height(column_parent_constraint, total_children_measured_height);
    let final_column_width = calculate_final_column_width(
        column_parent_constraint,
        *max_child_width,
        input.parent_constraint(),
    );
    Ok((
        final_column_width,
        final_column_height,
        total_children_measured_height,
    ))
}

/// Place measured children into the column according to main and cross axis
/// alignment.
///
/// This helper computes the starting y position and spacing between children
/// based on `MainAxisAlignment` variants (Start, Center, End, SpaceEvenly,
/// SpaceBetween, SpaceAround) and aligns each child horizontally using
/// `CrossAxisAlignment`. It calls `place_node` to record each child's layout
/// position.
///
/// `args` contains measured child sizes, node ids, component metadata and final
/// column size.
fn place_children_with_alignment(args: &PlaceChildrenArgs, result: &mut LayoutResult) {
    let (mut current_y, spacing_between_children) = calculate_main_axis_layout_for_column(
        args.final_column_height,
        args.total_children_height,
        args.main_axis_alignment,
        args.child_count,
    );

    for (i, child_size_opt) in args.children_sizes.iter().enumerate() {
        if let Some(child_actual_size) = child_size_opt {
            let child_id = args.children[i];
            let x_offset = calculate_cross_axis_offset_for_column(
                child_actual_size,
                args.final_column_width,
                args.cross_axis_alignment,
            );
            result.place_child(child_id, PxPosition::new(x_offset, current_y));
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

    use super::column;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    #[derive(Clone, PartialEq)]
    struct FillHeightTestLayout {
        width: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    impl LayoutPolicy for FillHeightTestLayout {
        fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let height = input
                .parent_constraint()
                .height()
                .resolve_max()
                .expect("FillHeightTestLayout requires a bounded height constraint");

            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height,
            }))
        }
    }

    #[tessera]
    fn fixed_test_box(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();

        layout()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn fill_height_test_box(tag: Option<String>, width: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();

        layout()
            .layout_policy(FillHeightTestLayout { width })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn column_layout_case() {
        column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(100))),
                Some(AxisConstraint::exact(Px::new(60))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                column_first_box();
                column_second_box();
            });
    }

    #[tessera]
    fn column_first_box() {
        fixed_test_box()
            .tag("column_first".to_string())
            .width(20)
            .height(10);
    }

    #[tessera]
    fn column_second_box() {
        fixed_test_box()
            .tag("column_second".to_string())
            .width(40)
            .height(15);
    }

    #[tessera]
    fn column_weighted_case() {
        column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(90))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(|| {
                fixed_test_box()
                    .tag("column_weighted_fixed".to_string())
                    .width(20)
                    .height(10);
                layout().modifier(Modifier::new().weight(1.0)).child(|| {
                    fill_height_test_box()
                        .tag("column_weighted_fill".to_string())
                        .width(15);
                });
            });
    }

    #[test]
    fn column_centers_children_on_cross_axis() {
        tessera_ui::assert_layout! {
            viewport: (120, 80),
            content: {
                column_layout_case();
            },
            expect: {
                node("column_first").position(40, 0).size(20, 10);
                node("column_second").position(30, 10).size(40, 15);
            }
        }
    }

    #[test]
    fn column_allocates_remaining_height_to_weighted_child() {
        tessera_ui::assert_layout! {
            viewport: (100, 100),
            content: {
                column_weighted_case();
            },
            expect: {
                node("column_weighted_fixed").position(0, 0).size(20, 10);
                node("column_weighted_fill").position(0, 10).size(15, 80);
            }
        }
    }
}
