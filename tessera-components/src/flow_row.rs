//! A flowing horizontal layout component.
//!
//! ## Usage
//!
//! Wrap chips, tags, or button groups across multiple rows.
use derive_setters::Setters;
use tessera_ui::{
    ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    tessera,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    modifier::ModifierExt as _,
};

/// Arguments for the `flow_row` component.
#[derive(PartialEq, Clone, Debug, Setters)]
pub struct FlowRowArgs {
    /// Modifier chain applied to the flow row subtree.
    pub modifier: Modifier,
    /// Alignment of items along the main axis within each row.
    pub main_axis_alignment: MainAxisAlignment,
    /// Alignment of items along the cross axis within each row.
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Alignment of rows along the cross axis inside the container.
    pub line_alignment: MainAxisAlignment,
    /// Spacing between items within a row.
    pub item_spacing: Dp,
    /// Spacing between rows.
    pub line_spacing: Dp,
    /// Maximum number of items per row.
    pub max_items_per_line: usize,
    /// Maximum number of rows.
    pub max_lines: usize,
}

impl Default for FlowRowArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new()
                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP)),
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            line_alignment: MainAxisAlignment::Start,
            item_spacing: Dp::ZERO,
            line_spacing: Dp::ZERO,
            max_items_per_line: usize::MAX,
            max_lines: usize::MAX,
        }
    }
}

/// A scope for declaratively adding children to a `flow_row` component.
pub struct FlowRowScope<'a> {
    child_closures: &'a mut Vec<RenderSlot>,
    child_weights: &'a mut Vec<Option<f32>>,
}

impl<'a> FlowRowScope<'a> {
    /// Adds a child component to the flow row.
    pub fn child<F>(&mut self, child_closure: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.child_closures.push(RenderSlot::new(child_closure));
        self.child_weights.push(None);
    }

    /// Adds a child component with a weight for main-axis distribution.
    pub fn child_weighted<F>(&mut self, child_closure: F, weight: f32)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.child_closures.push(RenderSlot::new(child_closure));
        self.child_weights.push(Some(weight));
    }
}

#[derive(Clone, PartialEq)]
struct FlowRowRenderArgs {
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    line_alignment: MainAxisAlignment,
    item_spacing: Dp,
    line_spacing: Dp,
    max_items_per_line: usize,
    max_lines: usize,
    child_closures: Vec<RenderSlot>,
    child_weights: Vec<Option<f32>>,
}

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
/// - `args` — configures alignment, spacing, and wrapping; see [`FlowRowArgs`].
/// - `scope_config` — a closure that receives a [`FlowRowScope`] for adding
///   children.
///
/// ## Examples
///
/// ```
/// use tessera_components::flow_row::{FlowRowArgs, flow_row};
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     flow_row(FlowRowArgs::default(), |scope| {
///         scope.child(move || {
///             rendered.with_mut(|count| *count += 1);
///             text(&tessera_components::text::TextArgs::default().text("First"));
///         });
///         scope.child(move || {
///             rendered.with_mut(|count| *count += 1);
///             text(&tessera_components::text::TextArgs::default().text("Second"));
///         });
///     });
///     assert_eq!(rendered.get(), 2);
/// }
///
/// demo();
/// ```
pub fn flow_row<F>(args: FlowRowArgs, scope_config: F)
where
    F: FnOnce(&mut FlowRowScope),
{
    let modifier = args.modifier;

    let mut child_closures: Vec<RenderSlot> = Vec::new();
    let mut child_weights: Vec<Option<f32>> = Vec::new();

    {
        let mut scope = FlowRowScope {
            child_closures: &mut child_closures,
            child_weights: &mut child_weights,
        };
        scope_config(&mut scope);
    }

    let render_args = FlowRowRenderArgs {
        main_axis_alignment: args.main_axis_alignment,
        cross_axis_alignment: args.cross_axis_alignment,
        line_alignment: args.line_alignment,
        item_spacing: args.item_spacing,
        line_spacing: args.line_spacing,
        max_items_per_line: args.max_items_per_line,
        max_lines: args.max_lines,
        child_closures,
        child_weights,
    };

    modifier.run(move || flow_row_inner(&render_args));
}

#[tessera]
fn flow_row_inner(args: &FlowRowRenderArgs) {
    let item_spacing = sanitize_spacing(Px::from(args.item_spacing));
    let line_spacing = sanitize_spacing(Px::from(args.line_spacing));
    layout(FlowRowLayout {
        main_axis_alignment: args.main_axis_alignment,
        cross_axis_alignment: args.cross_axis_alignment,
        line_alignment: args.line_alignment,
        item_spacing,
        line_spacing,
        max_items_per_line: args.max_items_per_line,
        max_lines: args.max_lines,
        child_weights: args.child_weights.clone(),
    });

    for child_closure in &args.child_closures {
        child_closure.render();
    }
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
    child_weights: Vec<Option<f32>>,
}

impl LayoutSpec for FlowRowLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let n = self.child_weights.len();
        let children_ids = input.children_ids();
        assert_eq!(
            children_ids.len(),
            n,
            "Mismatch between children defined in scope and runtime children count"
        );

        let flow_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let max_width = flow_constraint.width.get_max();

        let child_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: max_width,
            },
            flow_constraint.height,
        );

        let use_weighted_remeasure = max_width.is_some();
        let mut unweighted_nodes = Vec::new();
        let mut weighted_nodes = Vec::new();
        for (idx, &child_id) in children_ids.iter().enumerate().take(n) {
            let weight = self
                .child_weights
                .get(idx)
                .copied()
                .flatten()
                .unwrap_or(0.0);
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
                child_weights: &self.child_weights,
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
                Constraint::new(DimensionValue::Fixed(*allocated), flow_constraint.height),
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

fn resolve_dimension(dim: DimensionValue, content: Px, context: &str) -> Px {
    match dim {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Fill { min, max } => {
            let Some(max) = max else {
                panic!(
                    "Seems that you are using Fill without max constraint, which is not supported in {context}."
                );
            };
            let mut value = max;
            if let Some(min) = min {
                value = value.max(min);
            }
            value
        }
        DimensionValue::Wrap { min, max } => {
            let mut value = content;
            if let Some(min) = min {
                value = value.max(min);
            }
            if let Some(max) = max {
                value = value.min(max);
            }
            value
        }
    }
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
