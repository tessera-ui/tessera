//! A flowing horizontal layout component.
//!
//! ## Usage
//!
//! Wrap chips, tags, or button groups across multiple rows.
use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot,
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, layout_primitive},
    tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

/// # flow_row
///
/// A layout component that wraps children into multiple horizontal rows.
///
/// ## Usage
///
/// Wrap chips, tags, or button groups to fit variable-width screens.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the flowing row container.
/// - `main_axis_alignment` — alignment within each wrapped line.
/// - `cross_axis_alignment` — cross-axis alignment for items inside a line.
/// - `line_alignment` — alignment between wrapped lines.
/// - `item_spacing` — spacing between items in the same line.
/// - `line_spacing` — spacing between wrapped lines.
/// - `max_items_per_line` — optional cap for items per line.
/// - `max_lines` — optional cap for total wrapped lines.
/// - `children` — child slot rendered inside the layout.
///
/// ## Examples
///
/// ```
/// use tessera_components::{flow_row::flow_row, text::text};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     flow_row().children(move || {
///         rendered.with_mut(|count| *count += 1);
///         text().content("First");
///         rendered.with_mut(|count| *count += 1);
///         text().content("Second");
///     });
///     assert_eq!(rendered.get(), 2);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn flow_row(
    modifier: Option<Modifier>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    line_alignment: MainAxisAlignment,
    item_spacing: Dp,
    line_spacing: Dp,
    max_items_per_line: Option<usize>,
    max_lines: Option<usize>,
    children: RenderSlot,
) {
    let modifier = modifier.unwrap_or_default();
    let item_spacing = sanitize_spacing(Px::from(item_spacing));
    let line_spacing = sanitize_spacing(Px::from(line_spacing));
    let max_items_per_line = max_items_per_line.unwrap_or(usize::MAX);
    let max_lines = max_lines.unwrap_or(usize::MAX);
    layout_primitive()
        .modifier(modifier)
        .layout_policy(FlowRowLayout {
            main_axis_alignment,
            cross_axis_alignment,
            line_alignment,
            item_spacing,
            line_spacing,
            max_items_per_line,
            max_lines,
        })
        .child(move || {
            children.render();
        });
}

#[derive(Clone, PartialEq)]
struct FlowRowLayout {
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    line_alignment: MainAxisAlignment,
    item_spacing: Px,
    line_spacing: Px,
    max_items_per_line: usize,
    max_lines: usize,
}

impl LayoutPolicy for FlowRowLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_weights = collect_child_weights(input);
        let n = child_weights.len();
        let children_ids = input.children_ids();
        assert_eq!(
            children_ids.len(),
            n,
            "Mismatch between children defined in scope and runtime children count"
        );

        let flow_constraint = *input.parent_constraint().as_ref();
        let max_width = flow_constraint.width.resolve_max();

        let child_constraint =
            Constraint::new(flow_constraint.width.without_min(), flow_constraint.height);

        let use_weighted_remeasure = max_width.is_some();
        let mut unweighted_nodes = Vec::new();
        let mut weighted_nodes = Vec::new();
        for (idx, &child_id) in children_ids.iter().enumerate().take(n) {
            let weight = child_weights.get(idx).copied().flatten().unwrap_or(0.0);
            if weight > 0.0 && use_weighted_remeasure {
                weighted_nodes.push((child_id, child_constraint));
            } else {
                unweighted_nodes.push((child_id, child_constraint));
            }
        }

        let unweighted_results = if unweighted_nodes.is_empty() {
            None
        } else {
            Some(input.measure_children(unweighted_nodes)?)
        };
        let weighted_results = if weighted_nodes.is_empty() {
            None
        } else {
            Some(input.measure_children_untracked(weighted_nodes)?)
        };

        let mut children_sizes = vec![None; n];
        for (i, &child_id) in children_ids.iter().enumerate().take(n) {
            if let Some(results) = &unweighted_results
                && let Some(child_size) = results.get(&child_id)
            {
                children_sizes[i] = Some(*child_size);
                continue;
            }
            if let Some(results) = &weighted_results
                && let Some(child_size) = results.get(&child_id)
            {
                children_sizes[i] = Some(*child_size);
            }
        }

        let lines = build_row_lines(
            &children_sizes,
            max_width,
            self.item_spacing,
            self.max_items_per_line,
            self.max_lines,
        );

        if let Some(max_width) = max_width {
            apply_weighted_children_row(WeightedRowMeasureInput {
                input,
                children_ids,
                flow_constraint: &flow_constraint,
                lines: &lines,
                children_sizes: &mut children_sizes,
                child_weights: &child_weights,
                item_spacing: self.item_spacing,
                max_width,
            })?;
        }

        let line_metrics = compute_row_line_metrics(&lines, &children_sizes, self.item_spacing);
        let (content_width, content_height) =
            compute_row_content_size(&line_metrics, self.line_spacing);

        let final_width = resolve_dimension(flow_constraint.width, content_width, "FlowRow width");
        let final_height =
            resolve_dimension(flow_constraint.height, content_height, "FlowRow height");

        place_flow_row(
            output,
            children_ids,
            &lines,
            &children_sizes,
            &line_metrics,
            self.main_axis_alignment,
            self.cross_axis_alignment,
            self.line_alignment,
            self.item_spacing,
            self.line_spacing,
            final_width,
            final_height,
        );

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}

fn collect_child_weights(input: &LayoutInput<'_>) -> Vec<Option<f32>> {
    input
        .children_ids()
        .iter()
        .map(|&child_id| {
            input
                .child_parent_data::<crate::modifier::WeightParentData>(child_id)
                .map(|data| data.weight)
        })
        .collect()
}

#[derive(Clone, PartialEq, Copy)]
struct LineMetric {
    main: Px,
    cross: Px,
}

fn build_row_lines(
    children_sizes: &[Option<ComputedData>],
    max_width: Option<Px>,
    item_spacing: Px,
    max_items_per_line: usize,
    max_lines: usize,
) -> Vec<Vec<usize>> {
    if max_items_per_line == 0 || max_lines == 0 {
        return Vec::new();
    }

    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut current_line: Vec<usize> = Vec::new();
    let mut current_width = Px::ZERO;

    for (index, child_size_opt) in children_sizes.iter().enumerate() {
        let Some(child_size) = child_size_opt else {
            continue;
        };

        let spacing = if current_line.is_empty() {
            Px::ZERO
        } else {
            item_spacing
        };
        let proposed_width = current_width + spacing + child_size.width;
        let exceeds_width = match max_width {
            Some(limit) => proposed_width > limit && !current_line.is_empty(),
            None => false,
        };
        let exceeds_items = current_line.len() >= max_items_per_line;

        if (exceeds_width || exceeds_items) && !current_line.is_empty() {
            lines.push(current_line);
            if lines.len() >= max_lines {
                return lines;
            }
            current_line = Vec::new();
            current_width = Px::ZERO;
        }

        current_width = if current_line.is_empty() {
            child_size.width
        } else {
            current_width + item_spacing + child_size.width
        };
        current_line.push(index);
    }

    if !current_line.is_empty() && lines.len() < max_lines {
        lines.push(current_line);
    }

    lines
}

struct WeightedRowMeasureInput<'a, 'b> {
    input: &'a LayoutInput<'b>,
    children_ids: &'a [tessera_ui::NodeId],
    flow_constraint: &'a Constraint,
    lines: &'a [Vec<usize>],
    children_sizes: &'a mut [Option<ComputedData>],
    child_weights: &'a [Option<f32>],
    item_spacing: Px,
    max_width: Px,
}

fn apply_weighted_children_row(
    args: WeightedRowMeasureInput<'_, '_>,
) -> Result<(), MeasurementError> {
    let WeightedRowMeasureInput {
        input,
        children_ids,
        flow_constraint,
        lines,
        children_sizes,
        child_weights,
        item_spacing,
        max_width,
    } = args;
    let mut allocations: Vec<(usize, Px)> = Vec::new();

    for line in lines {
        let mut total_weight = 0.0;
        let mut fixed_width = Px::ZERO;
        let mut weighted_indices: Vec<usize> = Vec::new();

        for &idx in line {
            let weight = child_weights[idx].unwrap_or(0.0);
            if weight > 0.0 {
                total_weight += weight;
                weighted_indices.push(idx);
            } else if let Some(size) = children_sizes[idx] {
                fixed_width += size.width;
            }
        }

        if total_weight <= 0.0 {
            continue;
        }

        let spacing_total = if line.len() > 1 {
            px_mul(item_spacing, line.len().saturating_sub(1))
        } else {
            Px::ZERO
        };
        let remaining = (max_width - fixed_width - spacing_total).max(Px::ZERO);

        let mut allocated_total = Px::ZERO;
        for (pos, idx) in weighted_indices.iter().enumerate() {
            let weight = child_weights[*idx].unwrap_or(0.0);
            let allocated = if pos + 1 == weighted_indices.len() {
                (remaining - allocated_total).max(Px::ZERO)
            } else {
                Px((remaining.0 as f32 * (weight / total_weight)) as i32)
            };
            allocated_total += allocated;
            allocations.push((*idx, allocated));
        }
    }

    if allocations.is_empty() {
        return Ok(());
    }

    let weighted_constraints: Vec<_> = allocations
        .iter()
        .map(|(idx, allocated)| {
            (
                children_ids[*idx],
                Constraint::new(AxisConstraint::exact(*allocated), flow_constraint.height),
            )
        })
        .collect();
    let weighted_results = input.measure_children(weighted_constraints)?;

    for (idx, _) in allocations {
        let child_id = children_ids[idx];
        if let Some(child_size) = weighted_results.get(&child_id) {
            children_sizes[idx] = Some(*child_size);
        }
    }

    Ok(())
}

fn compute_row_line_metrics(
    lines: &[Vec<usize>],
    children_sizes: &[Option<ComputedData>],
    item_spacing: Px,
) -> Vec<LineMetric> {
    let mut metrics = Vec::with_capacity(lines.len());

    for line in lines {
        let mut main = Px::ZERO;
        let mut cross = Px::ZERO;

        for (pos, idx) in line.iter().enumerate() {
            if let Some(child_size) = children_sizes[*idx] {
                main += child_size.width;
                if pos + 1 < line.len() {
                    main += item_spacing;
                }
                cross = cross.max(child_size.height);
            }
        }

        metrics.push(LineMetric { main, cross });
    }

    metrics
}

fn compute_row_content_size(line_metrics: &[LineMetric], line_spacing: Px) -> (Px, Px) {
    let mut width = Px::ZERO;
    let mut height = Px::ZERO;

    for metric in line_metrics {
        width = width.max(metric.main);
        height += metric.cross;
    }

    if !line_metrics.is_empty() {
        height += px_mul(line_spacing, line_metrics.len().saturating_sub(1));
    }

    (width, height)
}

#[allow(clippy::too_many_arguments)]
fn place_flow_row(
    output: &mut LayoutOutput<'_>,
    children_ids: &[tessera_ui::NodeId],
    lines: &[Vec<usize>],
    children_sizes: &[Option<ComputedData>],
    line_metrics: &[LineMetric],
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    line_alignment: MainAxisAlignment,
    item_spacing: Px,
    line_spacing: Px,
    final_width: Px,
    final_height: Px,
) {
    if lines.is_empty() {
        return;
    }

    let mut total_line_cross = Px::ZERO;
    for metric in line_metrics {
        total_line_cross += metric.cross;
    }
    let total_line_spacing = px_mul(line_spacing, lines.len().saturating_sub(1));
    let total_cross = total_line_cross + total_line_spacing;
    let available_cross = (final_height - total_cross).max(Px::ZERO);

    let (start_cross, extra_line_spacing) =
        calculate_alignment_offsets(available_cross, lines.len(), line_alignment);
    let line_gap = line_spacing + extra_line_spacing;

    let mut current_y = start_cross;

    for (line_index, line) in lines.iter().enumerate() {
        let line_metric = line_metrics[line_index];
        let available_main = (final_width - line_metric.main).max(Px::ZERO);
        let (start_main, extra_item_spacing) =
            calculate_alignment_offsets(available_main, line.len(), main_axis_alignment);
        let item_gap = item_spacing + extra_item_spacing;

        let mut current_x = start_main;
        for (pos, idx) in line.iter().enumerate() {
            if let Some(child_size) = children_sizes[*idx] {
                let child_id = children_ids[*idx];
                let y_offset = calculate_cross_axis_offset(
                    child_size.height,
                    line_metric.cross,
                    cross_axis_alignment,
                );
                output.place_child(child_id, PxPosition::new(current_x, current_y + y_offset));
                current_x += child_size.width;
                if pos + 1 < line.len() {
                    current_x += item_gap;
                }
            }
        }

        current_y += line_metric.cross;
        if line_index + 1 < lines.len() {
            current_y += line_gap;
        }
    }
}

fn calculate_alignment_offsets(
    available_space: Px,
    count: usize,
    alignment: MainAxisAlignment,
) -> (Px, Px) {
    match alignment {
        MainAxisAlignment::Start => (Px::ZERO, Px::ZERO),
        MainAxisAlignment::Center => (available_space / 2, Px::ZERO),
        MainAxisAlignment::End => (available_space, Px::ZERO),
        MainAxisAlignment::SpaceEvenly => {
            if count > 0 {
                let s = available_space / (count as i32 + 1);
                (s, s)
            } else {
                (Px::ZERO, Px::ZERO)
            }
        }
        MainAxisAlignment::SpaceBetween => {
            if count > 1 {
                (Px::ZERO, available_space / (count as i32 - 1))
            } else if count == 1 {
                (available_space / 2, Px::ZERO)
            } else {
                (Px::ZERO, Px::ZERO)
            }
        }
        MainAxisAlignment::SpaceAround => {
            if count > 0 {
                let s = available_space / (count as i32);
                (s / 2, s)
            } else {
                (Px::ZERO, Px::ZERO)
            }
        }
    }
}

fn calculate_cross_axis_offset(
    child_cross: Px,
    line_cross: Px,
    alignment: CrossAxisAlignment,
) -> Px {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Px::ZERO,
        CrossAxisAlignment::Center => (line_cross - child_cross).max(Px::ZERO) / 2,
        CrossAxisAlignment::End => (line_cross - child_cross).max(Px::ZERO),
    }
}

fn resolve_dimension(axis: AxisConstraint, content: Px, _context: &str) -> Px {
    axis.clamp(content)
}

fn sanitize_spacing(px: Px) -> Px {
    if px < Px::ZERO { Px::ZERO } else { px }
}

fn px_mul(px: Px, times: usize) -> Px {
    if times == 0 {
        return Px::ZERO;
    }
    px_from_i64(px.0 as i64 * times as i64)
}

fn px_from_i64(value: i64) -> Px {
    if value > i64::from(i32::MAX) {
        Px(i32::MAX)
    } else if value < i64::from(i32::MIN) {
        Px(i32::MIN)
    } else {
        Px(value as i32)
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        AxisConstraint, ComputedData, LayoutInput, LayoutOutput, LayoutPolicy, MeasurementError,
        Modifier, NoopRenderPolicy, Px, layout::layout_primitive, tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        modifier::{ModifierExt as _, SemanticsArgs},
    };

    use super::flow_row;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(
            &self,
            _input: &LayoutInput<'_>,
            _output: &mut LayoutOutput<'_>,
        ) -> Result<ComputedData, MeasurementError> {
            Ok(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            })
        }
    }

    #[tessera]
    fn fixed_test_box(tag: String, width: i32, height: i32) {
        layout_primitive()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn flow_row_layout_case() {
        flow_row()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::new(Px::ZERO, Some(Px::new(100)))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .line_alignment(MainAxisAlignment::Start)
            .item_spacing(Px::new(5).into())
            .line_spacing(Px::new(4).into())
            .children(|| {
                fixed_test_box()
                    .tag("flow_row_first".to_string())
                    .width(30)
                    .height(10);
                fixed_test_box()
                    .tag("flow_row_second".to_string())
                    .width(25)
                    .height(10);
                fixed_test_box()
                    .tag("flow_row_third".to_string())
                    .width(20)
                    .height(12);
            });
    }

    #[test]
    fn flow_row_wraps_children_across_lines() {
        tessera_ui::assert_layout! {
            viewport: (100, 100),
            content: {
                flow_row_layout_case();
            },
            expect: {
                node("flow_row_first").position(0, 0).size(30, 10);
                node("flow_row_second").position(35, 0).size(25, 10);
                node("flow_row_third").position(0, 14).size(20, 12);
            }
        }
    }
}
