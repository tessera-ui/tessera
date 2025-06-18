use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};
use derive_builder::Builder;
use tessera::{ComputedData, Constraint, DimensionValue, Px, PxPosition, place_node};
use tessera_macros::tessera;

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

/// Represents a child item within a Row layout.
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

/// A row component that arranges its children horizontally.
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

    measure(Box::new(move |input| {
        let row_intrinsic_constraint = Constraint::new(args.width, args.height);
        // This is the effective constraint for the row itself
        let row_effective_constraint = row_intrinsic_constraint.merge(input.parent_constraint);

        let mut children_sizes = vec![None; N];
        let mut max_child_height = Px(0);

        // For Row, main axis is horizontal, so check width for weight distribution
        let should_use_weight_for_width = match row_effective_constraint.width {
            DimensionValue::Fixed(_) => true,
            DimensionValue::Fill { max: Some(_), .. } => true,
            DimensionValue::Wrap { max: Some(_), .. } => true,
            _ => false,
        };

        if should_use_weight_for_width {
            let available_width_for_children = match row_effective_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Fill { max: Some(w), .. } => w,
                DimensionValue::Wrap { max: Some(w), .. } => w,
                _ => unreachable!(
                    "Width should be constrained if should_use_weight_for_width is true"
                ),
            };

            let mut weighted_children_indices = Vec::new();
            let mut unweighted_children_indices = Vec::new();
            let mut total_weight_sum = 0.0f32;

            for (i, weight_opt) in child_weights.iter().enumerate() {
                if let Some(w) = weight_opt {
                    if *w > 0.0 {
                        weighted_children_indices.push(i);
                        total_weight_sum += w;
                    } else {
                        unweighted_children_indices.push(i);
                    }
                } else {
                    unweighted_children_indices.push(i);
                }
            }

            let mut total_width_of_unweighted_children = Px(0);
            for &child_idx in &unweighted_children_indices {
                let child_id = input.children_ids[child_idx];

                // Parent (row) offers Wrap for width and its own effective height constraint to unweighted children
                let parent_offered_constraint_for_child = Constraint::new(
                    DimensionValue::Wrap {
                        min: None,
                        max: None,
                    },
                    row_effective_constraint.height,
                );

                // measure_node will fetch the child's intrinsic constraint and merge it
                let child_result = tessera::measure_node(
                    child_id,
                    &parent_offered_constraint_for_child,
                    input.tree,
                    input.metadatas,
                )?;

                children_sizes[child_idx] = Some(child_result);
                total_width_of_unweighted_children += child_result.width;
                max_child_height = max_child_height.max(child_result.height);
            }

            let remaining_width_for_weighted_children =
                (available_width_for_children - total_width_of_unweighted_children).max(Px(0));
            if total_weight_sum > 0.0 {
                for &child_idx in &weighted_children_indices {
                    let child_weight = child_weights[child_idx].unwrap_or(0.0);
                    let allocated_width_for_child =
                        Px((remaining_width_for_weighted_children.0 as f32
                            * (child_weight / total_weight_sum)) as i32);
                    let child_id = input.children_ids[child_idx];

                    // Parent (row) offers Fixed allocated width and its own effective height constraint to weighted children
                    let parent_offered_constraint_for_child = Constraint::new(
                        DimensionValue::Fixed(allocated_width_for_child),
                        row_effective_constraint.height,
                    );

                    // measure_node will fetch the child's intrinsic constraint and merge it
                    let child_result = tessera::measure_node(
                        child_id,
                        &parent_offered_constraint_for_child,
                        input.tree,
                        input.metadatas,
                    )?;

                    children_sizes[child_idx] = Some(child_result);
                    max_child_height = max_child_height.max(child_result.height);
                }
            }

            let final_row_width = available_width_for_children;
            // Row's height is determined by its own effective constraint, or by wrapping content if no explicit max.
            let final_row_height = match row_effective_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Fill { max: Some(h), .. } => h,
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
                _ => max_child_height, // Fill { max: None } or Wrap { max: None } -> wraps content
            };

            let total_measured_children_width: Px = children_sizes
                .iter()
                .filter_map(|size_opt| size_opt.as_ref().map(|s| s.width))
                .fold(Px(0), |acc, width| acc + width);

            place_children_with_alignment(
                &children_sizes,
                input.children_ids,
                input.metadatas,
                final_row_width,
                final_row_height,
                total_measured_children_width,
                args.main_axis_alignment,
                args.cross_axis_alignment,
                N,
            );

            Ok(ComputedData {
                width: final_row_width,
                height: final_row_height,
            })
        } else {
            // Not using weight logic for width (row width is Wrap or Fill without max)
            let mut total_children_measured_width = Px(0);

            for i in 0..N {
                let child_id = input.children_ids[i];

                // Parent (row) offers Wrap for width and its effective height
                let parent_offered_constraint_for_child = Constraint::new(
                    DimensionValue::Wrap {
                        min: None,
                        max: None,
                    },
                    row_effective_constraint.height,
                );

                // measure_node will fetch the child's intrinsic constraint and merge it
                let child_result = tessera::measure_node(
                    child_id,
                    &parent_offered_constraint_for_child,
                    input.tree,
                    input.metadatas,
                )?;

                children_sizes[i] = Some(child_result);
                total_children_measured_width += child_result.width;
                max_child_height = max_child_height.max(child_result.height);
            }

            // Determine row's final size based on its own constraints and content
            let final_row_width = match row_effective_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Fill { min, .. } => {
                    // Max is None if here
                    let mut w = total_children_measured_width;
                    if let Some(min_w) = min {
                        w = w.max(min_w);
                    }
                    w
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
            };

            let final_row_height = match row_effective_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Fill { min, max } => {
                    let mut h = max_child_height;
                    if let Some(min_h) = min {
                        h = h.max(min_h);
                    }
                    if let Some(max_h) = max {
                        h = h.min(max_h);
                    } else {
                        h = max_child_height;
                    }
                    h
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
            };

            place_children_with_alignment(
                &children_sizes,
                input.children_ids,
                input.metadatas,
                final_row_width,
                final_row_height,
                total_children_measured_width,
                args.main_axis_alignment,
                args.cross_axis_alignment,
                N,
            );

            Ok(ComputedData {
                width: final_row_width,
                height: final_row_height,
            })
        }
    }));

    for child_closure in child_closures {
        child_closure();
    }
}

/// 根据对齐方式放置子元素的辅助函数 (水平布局)
fn place_children_with_alignment(
    children_sizes: &[Option<ComputedData>],
    children_ids: &[tessera::NodeId],
    metadatas: &tessera::ComponentNodeMetaDatas,
    final_row_width: Px,
    final_row_height: Px,
    total_children_width: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
) {
    let available_space = (final_row_width - total_children_width).max(Px(0));

    // 计算主轴起始位置和间距 (对于 Row，主轴是水平方向)
    let (mut current_x, spacing_between_children) = match main_axis_alignment {
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
    };

    for (i, child_size_opt) in children_sizes.iter().enumerate() {
        if let Some(child_actual_size) = child_size_opt {
            let child_id = children_ids[i];

            // 计算交叉轴位置 (对于 Row，交叉轴是垂直方向)
            let y_offset = match cross_axis_alignment {
                CrossAxisAlignment::Start => Px(0),
                CrossAxisAlignment::Center => {
                    (final_row_height - child_actual_size.height).max(Px(0)) / 2
                }
                CrossAxisAlignment::End => (final_row_height - child_actual_size.height).max(Px(0)),
                CrossAxisAlignment::Stretch => Px(0),
            };

            place_node(child_id, PxPosition::new(current_x, y_offset), metadatas);
            current_x += child_actual_size.width;
            if i < child_count - 1 {
                current_x += spacing_between_children;
            }
        }
    }
}
