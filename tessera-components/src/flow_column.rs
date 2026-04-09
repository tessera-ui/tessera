//! A flowing vertical layout component.
//!
//! ## Usage
//!
//! Wrap tall lists or cards into multiple columns.
use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlot,
    layout::{LayoutChild, LayoutPolicy, MeasureScope, layout},
    tessera,
};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

/// # flow_column
///
/// A layout component that wraps children into multiple vertical columns.
///
/// ## Usage
///
/// Wrap tall item stacks into columns for responsive dashboards or menus.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the flowing column container.
/// - `main_axis_alignment` — alignment within each wrapped column.
/// - `cross_axis_alignment` — cross-axis alignment for items inside a column.
/// - `line_alignment` — alignment between wrapped columns.
/// - `item_spacing` — spacing between items in the same column.
/// - `line_spacing` — spacing between wrapped columns.
/// - `max_items_per_line` — optional cap for items per column.
/// - `max_lines` — optional cap for total wrapped columns.
/// - `children` — child slot rendered inside the layout.
///
/// ## Examples
///
/// ```
/// use tessera_components::{flow_column::flow_column, text::text};
/// use tessera_ui::{LayoutResult, remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     flow_column().children(move || {
///         rendered.with_mut(|count| *count += 1);
///         text().content("Alpha");
///         rendered.with_mut(|count| *count += 1);
///         text().content("Beta");
///     });
///     assert_eq!(rendered.get(), 2);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn flow_column(
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
    layout()
        .modifier(modifier)
        .layout_policy(FlowColumnLayout {
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
struct FlowColumnLayout {
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    line_alignment: MainAxisAlignment,
    item_spacing: Px,
    line_spacing: Px,
    max_items_per_line: usize,
    max_lines: usize,
}

impl LayoutPolicy for FlowColumnLayout {
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

        let flow_constraint = *input.parent_constraint().as_ref();
        let max_height = flow_constraint.height.resolve_max();

        let child_constraint =
            Constraint::new(flow_constraint.width, flow_constraint.height.without_min());

        let use_weighted_remeasure = max_height.is_some();
        let mut unweighted_nodes = Vec::new();
        let mut weighted_nodes = Vec::new();
        for (idx, child_id) in children.iter().enumerate().take(n) {
            let weight = child_weights.get(idx).copied().flatten().unwrap_or(0.0);
            if weight > 0.0 && use_weighted_remeasure {
                weighted_nodes.push((*child_id, child_constraint));
            } else {
                unweighted_nodes.push((*child_id, child_constraint));
            }
        }

        let mut children_sizes = vec![None; n];
        for (i, child_id) in children.iter().enumerate().take(n) {
            if let Some((_, constraint)) = unweighted_nodes
                .iter()
                .find(|(candidate, _)| candidate == child_id)
            {
                children_sizes[i] = Some(child_id.measure(constraint)?.size());
                continue;
            }
            if let Some((_, constraint)) = weighted_nodes
                .iter()
                .find(|(candidate, _)| candidate == child_id)
            {
                children_sizes[i] = Some(child_id.measure_untracked(constraint)?.size());
            }
        }

        let lines = build_column_lines(
            &children_sizes,
            max_height,
            self.item_spacing,
            self.max_items_per_line,
            self.max_lines,
        );

        if let Some(max_height) = max_height {
            apply_weighted_children_column(WeightedColumnMeasureInput {
                input,
                children: &children,
                flow_constraint: &flow_constraint,
                lines: &lines,
                children_sizes: &mut children_sizes,
                child_weights: &child_weights,
                item_spacing: self.item_spacing,
                max_height,
            })?;
        }

        let line_metrics = compute_column_line_metrics(&lines, &children_sizes, self.item_spacing);
        let (content_width, content_height) =
            compute_column_content_size(&line_metrics, self.line_spacing);

        let final_width =
            resolve_dimension(flow_constraint.width, content_width, "FlowColumn width");
        let final_height =
            resolve_dimension(flow_constraint.height, content_height, "FlowColumn height");

        place_flow_column(
            &mut result,
            &children,
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

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
    }
}

fn collect_child_weights(input: &MeasureScope<'_>) -> Vec<Option<f32>> {
    input
        .children()
        .iter()
        .map(|child_id| {
            child_id
                .parent_data::<crate::modifier::WeightParentData>()
                .map(|data| data.weight)
        })
        .collect()
}

#[derive(Clone, PartialEq, Copy)]
struct LineMetric {
    main: Px,
    cross: Px,
}

fn build_column_lines(
    children_sizes: &[Option<ComputedData>],
    max_height: Option<Px>,
    item_spacing: Px,
    max_items_per_line: usize,
    max_lines: usize,
) -> Vec<Vec<usize>> {
    if max_items_per_line == 0 || max_lines == 0 {
        return Vec::new();
    }

    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut current_line: Vec<usize> = Vec::new();
    let mut current_height = Px::ZERO;

    for (index, child_size_opt) in children_sizes.iter().enumerate() {
        let Some(child_size) = child_size_opt else {
            continue;
        };

        let spacing = if current_line.is_empty() {
            Px::ZERO
        } else {
            item_spacing
        };
        let proposed_height = current_height + spacing + child_size.height;
        let exceeds_height = match max_height {
            Some(limit) => proposed_height > limit && !current_line.is_empty(),
            None => false,
        };
        let exceeds_items = current_line.len() >= max_items_per_line;

        if (exceeds_height || exceeds_items) && !current_line.is_empty() {
            lines.push(current_line);
            if lines.len() >= max_lines {
                return lines;
            }
            current_line = Vec::new();
            current_height = Px::ZERO;
        }

        current_height = if current_line.is_empty() {
            child_size.height
        } else {
            current_height + item_spacing + child_size.height
        };
        current_line.push(index);
    }

    if !current_line.is_empty() && lines.len() < max_lines {
        lines.push(current_line);
    }

    lines
}

struct WeightedColumnMeasureInput<'a, 'b> {
    input: &'a MeasureScope<'b>,
    children: &'a [LayoutChild<'b>],
    flow_constraint: &'a Constraint,
    lines: &'a [Vec<usize>],
    children_sizes: &'a mut [Option<ComputedData>],
    child_weights: &'a [Option<f32>],
    item_spacing: Px,
    max_height: Px,
}

fn apply_weighted_children_column(
    args: WeightedColumnMeasureInput<'_, '_>,
) -> Result<(), MeasurementError> {
    let WeightedColumnMeasureInput {
        input: _input,
        children,
        flow_constraint,
        lines,
        children_sizes,
        child_weights,
        item_spacing,
        max_height,
    } = args;
    let mut allocations: Vec<(usize, Px)> = Vec::new();

    for line in lines {
        let mut total_weight = 0.0;
        let mut fixed_height = Px::ZERO;
        let mut weighted_indices: Vec<usize> = Vec::new();

        for &idx in line {
            let weight = child_weights[idx].unwrap_or(0.0);
            if weight > 0.0 {
                total_weight += weight;
                weighted_indices.push(idx);
            } else if let Some(size) = children_sizes[idx] {
                fixed_height += size.height;
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
        let remaining = (max_height - fixed_height - spacing_total).max(Px::ZERO);

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

    for &(idx, allocated) in &allocations {
        let child_id = children[idx];
        let child_size = child_id.measure(&Constraint::new(
            flow_constraint.width,
            AxisConstraint::exact(allocated),
        ))?;
        children_sizes[idx] = Some(child_size.size());
    }

    Ok(())
}

fn compute_column_line_metrics(
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
                main += child_size.height;
                if pos + 1 < line.len() {
                    main += item_spacing;
                }
                cross = cross.max(child_size.width);
            }
        }

        metrics.push(LineMetric { main, cross });
    }

    metrics
}

fn compute_column_content_size(line_metrics: &[LineMetric], line_spacing: Px) -> (Px, Px) {
    let mut width = Px::ZERO;
    let mut height = Px::ZERO;

    for metric in line_metrics {
        height = height.max(metric.main);
        width += metric.cross;
    }

    if !line_metrics.is_empty() {
        width += px_mul(line_spacing, line_metrics.len().saturating_sub(1));
    }

    (width, height)
}

#[allow(clippy::too_many_arguments)]
fn place_flow_column(
    result: &mut LayoutResult,
    children: &[LayoutChild<'_>],
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
    let available_cross = (final_width - total_cross).max(Px::ZERO);

    let (start_cross, extra_line_spacing) =
        calculate_alignment_offsets(available_cross, lines.len(), line_alignment);
    let line_gap = line_spacing + extra_line_spacing;

    let mut current_x = start_cross;

    for (line_index, line) in lines.iter().enumerate() {
        let line_metric = line_metrics[line_index];
        let available_main = (final_height - line_metric.main).max(Px::ZERO);
        let (start_main, extra_item_spacing) =
            calculate_alignment_offsets(available_main, line.len(), main_axis_alignment);
        let item_gap = item_spacing + extra_item_spacing;

        let mut current_y = start_main;
        for (pos, idx) in line.iter().enumerate() {
            if let Some(child_size) = children_sizes[*idx] {
                let child_id = children[*idx];
                let x_offset = calculate_cross_axis_offset(
                    child_size.width,
                    line_metric.cross,
                    cross_axis_alignment,
                );
                result.place_child(child_id, PxPosition::new(current_x + x_offset, current_y));
                current_y += child_size.height;
                if pos + 1 < line.len() {
                    current_y += item_gap;
                }
            }
        }

        current_x += line_metric.cross;
        if line_index + 1 < lines.len() {
            current_x += line_gap;
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
        AxisConstraint, ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier,
        NoopRenderPolicy, Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        modifier::{ModifierExt as _, SemanticsArgs},
    };

    use super::flow_column;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
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
    fn flow_column_layout_case() {
        flow_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::new(Px::ZERO, Some(Px::new(100)))),
                Some(AxisConstraint::exact(Px::new(40))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .line_alignment(MainAxisAlignment::Start)
            .item_spacing(Px::new(3).into())
            .line_spacing(Px::new(4).into())
            .children(|| {
                fixed_test_box()
                    .tag("flow_column_first".to_string())
                    .width(20)
                    .height(12);
                fixed_test_box()
                    .tag("flow_column_second".to_string())
                    .width(25)
                    .height(15);
                fixed_test_box()
                    .tag("flow_column_third".to_string())
                    .width(18)
                    .height(10);
            });
    }

    #[test]
    fn flow_column_wraps_children_across_columns() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                flow_column_layout_case();
            },
            expect: {
                node("flow_column_first").position(0, 0).size(20, 12);
                node("flow_column_second").position(0, 15).size(25, 15);
                node("flow_column_third").position(29, 0).size(18, 10);
            }
        }
    }
}
