use derive_builder::Builder;
use tessera::{ComputedData, Constraint, DimensionValue, Px, PxPosition, place_node};
use tessera_macros::tessera;

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

/// A column component that arranges its children vertically.
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

    measure(Box::new(move |input| {
        let column_intrinsic_constraint = Constraint::new(args.width, args.height);
        // This is the effective constraint for the column itself
        let column_effective_constraint =
            column_intrinsic_constraint.merge(input.parent_constraint);

        let mut children_sizes = vec![None; N];
        let mut max_child_width = Px(0);

        let should_use_weight_for_height = match column_effective_constraint.height {
            DimensionValue::Fixed(_) => true,
            DimensionValue::Fill { max: Some(_), .. } => true,
            DimensionValue::Wrap { max: Some(_), .. } => true,
            _ => false,
        };

        if should_use_weight_for_height {
            let available_height_for_children =
                column_effective_constraint.height.get_max().unwrap();

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

            let mut total_height_of_unweighted_children = Px(0);
            for &child_idx in &unweighted_children_indices {
                let child_id = input.children_ids[child_idx];

                // Parent (column) offers its own effective width constraint.
                // Height is Wrap for unweighted children in this path.
                let parent_offered_constraint_for_child = Constraint::new(
                    column_effective_constraint.width,
                    DimensionValue::Wrap {
                        min: None,
                        max: column_effective_constraint.height.get_max(),
                    },
                );

                // measure_node will fetch the child's intrinsic constraint and merge it with parent_offered_constraint_for_child
                let child_result = tessera::measure_node(
                    child_id,
                    &parent_offered_constraint_for_child,
                    input.tree,
                    input.metadatas,
                    input.compute_resource_manager.clone(),
                    input.gpu,
                )?;

                children_sizes[child_idx] = Some(child_result);
                total_height_of_unweighted_children += child_result.height;
                max_child_width = max_child_width.max(child_result.width);
            }

            let remaining_height_for_weighted_children =
                (available_height_for_children - total_height_of_unweighted_children).max(Px(0));
            if total_weight_sum > 0.0 {
                for &child_idx in &weighted_children_indices {
                    let child_weight = child_weights[child_idx].unwrap_or(0.0);
                    let allocated_height_for_child =
                        Px((remaining_height_for_weighted_children.0 as f32
                            * (child_weight / total_weight_sum)) as i32);
                    let child_id = input.children_ids[child_idx];

                    // Parent (column) offers its own effective width constraint.
                    // Height is Fixed for weighted children.
                    let parent_offered_constraint_for_child = Constraint::new(
                        column_effective_constraint.width,
                        DimensionValue::Fixed(allocated_height_for_child),
                    );

                    // measure_node will fetch the child's intrinsic constraint and merge it
                    let child_result = tessera::measure_node(
                        child_id,
                        &parent_offered_constraint_for_child,
                        input.tree,
                        input.metadatas,
                        input.compute_resource_manager.clone(),
                        input.gpu,
                    )?;

                    children_sizes[child_idx] = Some(child_result);
                    max_child_width = max_child_width.max(child_result.width);
                }
            }

            let final_column_height = available_height_for_children;
            // column's width is determined by its own effective constraint, or by wrapping content if no explicit max.
            let final_column_width = match column_effective_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Fill { max: Some(w), .. } => w,
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
                _ => max_child_width, // Fill { max: None } or Wrap { max: None } -> wraps content
            };

            let total_measured_children_height: Px = children_sizes
                .iter()
                .filter_map(|size_opt| size_opt.as_ref().map(|s| s.height))
                .fold(Px(0), |acc, height| acc + height);

            place_children_with_alignment(
                &children_sizes,
                input.children_ids,
                input.metadatas,
                final_column_width,
                final_column_height,
                total_measured_children_height,
                args.main_axis_alignment,
                args.cross_axis_alignment,
                N,
            );

            Ok(ComputedData {
                width: final_column_width,
                height: final_column_height,
            })
        } else {
            // Not using weight logic for height (column height is Wrap or Fill without max)
            let mut total_children_measured_height = Px(0);

            for i in 0..N {
                let child_id = input.children_ids[i];

                // Parent (column) offers its effective width and Wrap for height
                let parent_offered_constraint_for_child = Constraint::new(
                    column_effective_constraint.width,
                    DimensionValue::Wrap {
                        min: None,
                        max: column_effective_constraint.height.get_max(),
                    },
                );

                // measure_node will fetch the child's intrinsic constraint and merge it
                let child_result = tessera::measure_node(
                    child_id,
                    &parent_offered_constraint_for_child,
                    input.tree,
                    input.metadatas,
                    input.compute_resource_manager.clone(),
                    input.gpu,
                )?;

                children_sizes[i] = Some(child_result);
                total_children_measured_height += child_result.height;
                max_child_width = max_child_width.max(child_result.width);
            }

            // Determine column's final size based on its own constraints and content
            let final_column_height = match column_effective_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Fill { min, .. } => {
                    // Max is None if here
                    let mut h = total_children_measured_height;
                    if let Some(min_h) = min {
                        h = h.max(min_h);
                    }
                    h
                }
                DimensionValue::Wrap { min, max } => {
                    let mut h = total_children_measured_height;
                    if let Some(min_h) = min {
                        h = h.max(min_h);
                    }
                    if let Some(max_h) = max {
                        h = h.min(max_h);
                    }
                    h
                }
            };

            let final_column_width = match column_effective_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Fill { min, max } => {
                    let mut w = max_child_width;
                    if let Some(min_w) = min {
                        w = w.max(min_w);
                    }
                    if let Some(max_w) = max {
                        w = w.min(max_w);
                    }
                    // column's own max for Fill
                    // If Fill has no max, it behaves like Wrap for width determination
                    else {
                        w = max_child_width;
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
            };

            place_children_with_alignment(
                &children_sizes,
                input.children_ids,
                input.metadatas,
                final_column_width,
                final_column_height,
                total_children_measured_height,
                args.main_axis_alignment,
                args.cross_axis_alignment,
                N,
            );

            Ok(ComputedData {
                width: final_column_width,
                height: final_column_height,
            })
        }
    }));

    for child_closure in child_closures {
        child_closure();
    }
}

fn place_children_with_alignment(
    children_sizes: &[Option<ComputedData>],
    children_ids: &[tessera::NodeId],
    metadatas: &tessera::ComponentNodeMetaDatas,
    final_column_width: Px,
    final_column_height: Px,
    total_children_height: Px,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    child_count: usize,
) {
    let available_space = (final_column_height - total_children_height).max(Px(0));

    let (mut current_y, spacing_between_children) = match main_axis_alignment {
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

            let x_offset = match cross_axis_alignment {
                CrossAxisAlignment::Start => Px(0),
                CrossAxisAlignment::Center => {
                    (final_column_width - child_actual_size.width).max(Px(0)) / 2
                }
                CrossAxisAlignment::End => {
                    (final_column_width - child_actual_size.width).max(Px(0))
                }
                CrossAxisAlignment::Stretch => Px(0),
            };

            place_node(child_id, PxPosition::new(x_offset, current_y), metadatas);
            current_y += child_actual_size.height;
            if i < child_count - 1 {
                current_y += spacing_between_children;
            }
        }
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
/// use tessera_basic_components::{column::{column_ui, ColumnArgs, ColumnItem}, text::text};
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
