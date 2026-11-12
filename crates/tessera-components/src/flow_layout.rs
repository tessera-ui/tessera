use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, LayoutResult, MeasurementError, Px, PxPosition,
    layout::{LayoutChild, MeasureScope},
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    layout_flex::{alignment_offsets, cross_axis_child_constraint, cross_axis_offset, px_mul},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FlowAxis {
    Horizontal,
    Vertical,
}

impl FlowAxis {
    fn main(self, size: ComputedData) -> Px {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    fn cross(self, size: ComputedData) -> Px {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    fn constraint(self, main: AxisConstraint, cross: AxisConstraint) -> Constraint {
        match self {
            Self::Horizontal => Constraint::new(main, cross),
            Self::Vertical => Constraint::new(cross, main),
        }
    }

    fn size(self, main: Px, cross: Px) -> ComputedData {
        match self {
            Self::Horizontal => ComputedData {
                width: main,
                height: cross,
            },
            Self::Vertical => ComputedData {
                width: cross,
                height: main,
            },
        }
    }

    fn position(self, main: f32, cross: f32) -> PxPosition {
        match self {
            Self::Horizontal => PxPosition::from_f32_xy(main, cross),
            Self::Vertical => PxPosition::from_f32_xy(cross, main),
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct FlowLayout {
    pub(crate) axis: FlowAxis,
    pub(crate) main_axis_alignment: MainAxisAlignment,
    pub(crate) cross_axis_alignment: CrossAxisAlignment,
    pub(crate) line_alignment: MainAxisAlignment,
    pub(crate) item_spacing: Px,
    pub(crate) line_spacing: Px,
    pub(crate) max_items_per_line: usize,
    pub(crate) max_lines: usize,
}

impl tessera_ui::LayoutPolicy for FlowLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let weights: Vec<_> = children
            .iter()
            .map(|child| {
                child
                    .parent_data::<tessera_foundation::modifier::WeightParentData>(input)
                    .map(|data| data.weight)
            })
            .collect();
        let parent = *input.parent_constraint().as_ref();
        let (main_constraint, cross_constraint) = match self.axis {
            FlowAxis::Horizontal => (parent.width, parent.height),
            FlowAxis::Vertical => (parent.height, parent.width),
        };
        let max_main = main_constraint.resolve_max();
        let child_constraint = self.axis.constraint(
            main_constraint.without_min(),
            cross_axis_child_constraint(cross_constraint, self.cross_axis_alignment),
        );
        let mut sizes = Vec::with_capacity(children.len());
        for (child, weight) in children.iter().zip(&weights) {
            let size = if max_main.is_some() && weight.unwrap_or(0.0) > 0.0 {
                child.probe(input, &child_constraint)?.size()
            } else {
                child.measure(input, &child_constraint)?.size()
            };
            sizes.push(Some(size));
        }

        let lines = build_lines(
            self.axis,
            &sizes,
            max_main,
            self.item_spacing,
            self.max_items_per_line,
            self.max_lines,
        );
        if let Some(max_main) = max_main {
            self.measure_weighted(
                input,
                &children,
                &lines,
                &mut sizes,
                &weights,
                max_main,
                cross_constraint,
            )?;
        }

        let metrics = compute_metrics(self.axis, &lines, &sizes, self.item_spacing);
        if self.cross_axis_alignment == CrossAxisAlignment::Stretch {
            for (line, metric) in lines.iter().zip(&metrics) {
                for &index in line {
                    let size = sizes[index].expect("measured flow child");
                    sizes[index] = Some(
                        children[index]
                            .measure(
                                input,
                                &self.axis.constraint(
                                    AxisConstraint::exact(self.axis.main(size)),
                                    AxisConstraint::exact(metric.cross),
                                ),
                            )?
                            .size(),
                    );
                }
            }
        }

        let content_main = metrics
            .iter()
            .fold(Px::ZERO, |max, metric| max.max(metric.main));
        let mut content_cross = metrics
            .iter()
            .fold(Px::ZERO, |sum, metric| sum + metric.cross);
        if !metrics.is_empty() {
            content_cross += px_mul(self.line_spacing, metrics.len().saturating_sub(1));
        }
        let size = self.axis.size(
            main_constraint.clamp(content_main),
            cross_constraint.clamp(content_cross),
        );
        let mut result = LayoutResult::default();
        self.place(&mut result, &children, &lines, &sizes, &metrics, size);
        Ok(result.with_size(size))
    }
}

#[derive(Clone, Copy)]
struct LineMetric {
    main: Px,
    cross: Px,
}

fn build_lines(
    axis: FlowAxis,
    sizes: &[Option<ComputedData>],
    max_main: Option<Px>,
    item_spacing: Px,
    max_items_per_line: usize,
    max_lines: usize,
) -> Vec<Vec<usize>> {
    if max_items_per_line == 0 || max_lines == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut main = Px::ZERO;
    for (index, size) in sizes.iter().enumerate() {
        let Some(size) = size else { continue };
        let child_main = axis.main(*size);
        let wraps = !line.is_empty()
            && (line.len() >= max_items_per_line
                || max_main.is_some_and(|limit| main + item_spacing + child_main > limit));
        if wraps {
            lines.push(line);
            if lines.len() >= max_lines {
                return lines;
            }
            line = Vec::new();
            main = Px::ZERO;
        }
        main = if line.is_empty() {
            child_main
        } else {
            main + item_spacing + child_main
        };
        line.push(index);
    }
    if !line.is_empty() && lines.len() < max_lines {
        lines.push(line);
    }
    lines
}

fn compute_metrics(
    axis: FlowAxis,
    lines: &[Vec<usize>],
    sizes: &[Option<ComputedData>],
    spacing: Px,
) -> Vec<LineMetric> {
    lines
        .iter()
        .map(|line| {
            let mut main = Px::ZERO;
            let mut cross = Px::ZERO;
            for (pos, &index) in line.iter().enumerate() {
                if let Some(size) = sizes[index] {
                    main += axis.main(size);
                    if pos + 1 < line.len() {
                        main += spacing;
                    }
                    cross = cross.max(axis.cross(size));
                }
            }
            LineMetric { main, cross }
        })
        .collect()
}

impl FlowLayout {
    #[allow(clippy::too_many_arguments)]
    fn measure_weighted(
        &self,
        scope: &mut MeasureScope<'_>,
        children: &[LayoutChild],
        lines: &[Vec<usize>],
        sizes: &mut [Option<ComputedData>],
        weights: &[Option<f32>],
        max_main: Px,
        cross_constraint: AxisConstraint,
    ) -> Result<(), MeasurementError> {
        let mut allocations = Vec::new();
        for line in lines {
            let mut total_weight = 0.0;
            let mut fixed_main = Px::ZERO;
            let mut weighted = Vec::new();
            for &index in line {
                let weight = weights[index].unwrap_or(0.0);
                if weight > 0.0 {
                    total_weight += weight;
                    weighted.push(index);
                } else if let Some(size) = sizes[index] {
                    fixed_main += self.axis.main(size);
                }
            }
            if total_weight <= 0.0 {
                continue;
            }
            let spacing = if line.len() > 1 {
                px_mul(self.item_spacing, line.len().saturating_sub(1))
            } else {
                Px::ZERO
            };
            let remaining = (max_main - fixed_main - spacing).max(Px::ZERO);
            let mut allocated_total = Px::ZERO;
            for (pos, &index) in weighted.iter().enumerate() {
                let allocated = if pos + 1 == weighted.len() {
                    (remaining - allocated_total).max(Px::ZERO)
                } else {
                    Px(
                        (remaining.0 as f32 * (weights[index].unwrap_or(0.0) / total_weight))
                            as i32,
                    )
                };
                allocated_total += allocated;
                allocations.push((index, allocated));
            }
        }
        for (index, allocated) in allocations {
            sizes[index] = Some(
                children[index]
                    .measure(
                        scope,
                        &self.axis.constraint(
                            AxisConstraint::exact(allocated),
                            cross_axis_child_constraint(
                                cross_constraint,
                                self.cross_axis_alignment,
                            ),
                        ),
                    )?
                    .size(),
            );
        }
        Ok(())
    }

    fn place(
        &self,
        result: &mut LayoutResult,
        children: &[LayoutChild],
        lines: &[Vec<usize>],
        sizes: &[Option<ComputedData>],
        metrics: &[LineMetric],
        size: ComputedData,
    ) {
        if lines.is_empty() {
            return;
        }
        let line_cross = metrics
            .iter()
            .fold(Px::ZERO, |sum, metric| sum + metric.cross);
        let total_cross = line_cross + px_mul(self.line_spacing, lines.len().saturating_sub(1));
        let available_cross = (self.axis.cross(size).0 - total_cross.0).max(0) as f32;
        let (start_cross, extra_line_spacing) =
            alignment_offsets(available_cross, lines.len(), self.line_alignment);
        let line_gap = self.line_spacing.0 as f32 + extra_line_spacing;
        let mut current_cross = start_cross;
        for (line_index, line) in lines.iter().enumerate() {
            let metric = metrics[line_index];
            let available_main = (self.axis.main(size).0 - metric.main.0).max(0) as f32;
            let (start_main, extra_item_spacing) =
                alignment_offsets(available_main, line.len(), self.main_axis_alignment);
            let item_gap = self.item_spacing.0 as f32 + extra_item_spacing;
            let mut current_main = start_main;
            for (pos, &index) in line.iter().enumerate() {
                if let Some(child_size) = sizes[index] {
                    let cross_offset = cross_axis_offset(
                        self.axis.cross(child_size),
                        metric.cross,
                        self.cross_axis_alignment,
                    );
                    result.place_child(
                        children[index],
                        self.axis
                            .position(current_main, current_cross + cross_offset),
                    );
                    current_main += self.axis.main(child_size).0 as f32;
                    if pos + 1 < line.len() {
                        current_main += item_gap;
                    }
                }
            }
            current_cross += metric.cross.0 as f32;
            if line_index + 1 < lines.len() {
                current_cross += line_gap;
            }
        }
    }
}
