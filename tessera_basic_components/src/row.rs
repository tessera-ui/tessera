use tessera::{
    ComputedData,
    Constraint,
    DimensionValue, // Removed ComponentNodeMetaDatas
    measure_node,
    place_node,
};
use tessera_macros::tessera;

/// Represents a child item within a Row layout.
pub struct RowItem {
    /// Determines how much space the child should take if its width_behavior is Fill,
    /// relative to other Fill children with weights.
    pub weight: Option<f32>,
    /// Defines the width behavior of this child.
    pub width_behavior: DimensionValue,
    /// The actual child component. Must be Send + Sync.
    pub child: Box<dyn FnOnce() + Send + Sync>,
}

impl RowItem {
    /// Creates a new `RowItem`.
    pub fn new(
        child: Box<dyn FnOnce() + Send + Sync>,
        width_behavior: DimensionValue,
        weight: Option<f32>,
    ) -> Self {
        RowItem {
            weight,
            width_behavior,
            child,
        }
    }

    /// Helper to create a RowItem that wraps its content.
    pub fn wrap(child: Box<dyn FnOnce() + Send + Sync>) -> Self {
        Self::new(child, DimensionValue::Wrap, None)
    }

    /// Helper to create a RowItem that is fixed width.
    pub fn fixed(child: Box<dyn FnOnce() + Send + Sync>, width: u32) -> Self {
        Self::new(child, DimensionValue::Fixed(width), None)
    }

    /// Helper to create a RowItem that fills available space, optionally with a weight and max.
    pub fn fill(
        child: Box<dyn FnOnce() + Send + Sync>,
        weight: Option<f32>,
        max_width: Option<u32>,
    ) -> Self {
        Self::new(child, DimensionValue::Fill { max: max_width }, weight)
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

/// Default conversion: a simple function closure becomes a `RowItem` that wraps its content.
impl<F: FnOnce() + Send + Sync + 'static> AsRowItem for F {
    fn into_row_item(self) -> RowItem {
        RowItem {
            weight: None,
            width_behavior: DimensionValue::Wrap, // Default to Wrap
            child: Box::new(self),
        }
    }
}

// Allow (FnOnce, DimensionValue) to be a RowItem
impl<F: FnOnce() + Send + Sync + 'static> AsRowItem for (F, DimensionValue) {
    fn into_row_item(self) -> RowItem {
        RowItem {
            weight: None, // No weight specified
            width_behavior: self.1,
            child: Box::new(self.0),
        }
    }
}

// Allow (FnOnce, DimensionValue, f32_weight) to be a RowItem
impl<F: FnOnce() + Send + Sync + 'static> AsRowItem for (F, DimensionValue, f32) {
    fn into_row_item(self) -> RowItem {
        RowItem {
            weight: Some(self.2),
            width_behavior: self.1,
            child: Box::new(self.0),
        }
    }
}

/// A row component that arranges its children horizontally.
/// Children can have fixed sizes, wrap their content, or fill available space (optionally with weights).
#[tessera]
pub fn row<const N: usize>(children_items_input: [impl AsRowItem; N]) {
    let children_items: [RowItem; N] =
        children_items_input.map(|item_input| item_input.into_row_item());
    let children_items_for_measure: Vec<_> = children_items
        .iter()
        .map(|child| (child.weight, child.width_behavior))
        .collect(); // For the measure closure

    measure(Box::new(
        move |node_id, tree, row_parent_constraint, children_node_ids, metadatas| {
            // Use children_items_for_measure inside this closure
            let effective_row_constraint = metadatas
                .get_mut(&node_id)
                .unwrap()
                .constraint
                .merge(row_parent_constraint);

            let mut measured_children_sizes: Vec<Option<ComputedData>> = vec![None; N];
            let mut total_width_for_fixed_wrap: u32 = 0;
            let mut computed_max_row_height: u32 = 0;

            for i in 0..N {
                let item = &children_items_for_measure[i]; // Use cloned version
                let child_node_id = children_node_ids[i];

                match item.1 {
                    DimensionValue::Fixed(fixed_width) => {
                        let child_constraint_for_measure = Constraint::new(
                            DimensionValue::Fixed(fixed_width),
                            effective_row_constraint.height,
                        );
                        let final_child_constraint = metadatas
                            .get_mut(&child_node_id)
                            .unwrap()
                            .constraint
                            .merge(&child_constraint_for_measure);
                        let size =
                            measure_node(child_node_id, &final_child_constraint, tree, metadatas);
                        measured_children_sizes[i] = Some(size);
                        total_width_for_fixed_wrap += size.width;
                        computed_max_row_height = computed_max_row_height.max(size.height);
                    }
                    DimensionValue::Wrap => {
                        let child_constraint_for_measure =
                            Constraint::new(DimensionValue::Wrap, effective_row_constraint.height);
                        let final_child_constraint = metadatas
                            .get_mut(&child_node_id)
                            .unwrap()
                            .constraint
                            .merge(&child_constraint_for_measure);
                        let size =
                            measure_node(child_node_id, &final_child_constraint, tree, metadatas);
                        measured_children_sizes[i] = Some(size);
                        total_width_for_fixed_wrap += size.width;
                        computed_max_row_height = computed_max_row_height.max(size.height);
                    }
                    DimensionValue::Fill { .. } => {}
                }
            }

            let mut remaining_width_for_fill: u32 = 0;
            let mut is_row_effectively_wrap_for_children = false;

            match effective_row_constraint.width {
                DimensionValue::Fixed(row_fixed_width) => {
                    remaining_width_for_fill =
                        row_fixed_width.saturating_sub(total_width_for_fixed_wrap);
                }
                DimensionValue::Wrap => {
                    is_row_effectively_wrap_for_children = true;
                }
                DimensionValue::Fill {
                    max: Some(row_max_budget),
                } => {
                    remaining_width_for_fill =
                        row_max_budget.saturating_sub(total_width_for_fixed_wrap);
                }
                DimensionValue::Fill { max: None } => {
                    is_row_effectively_wrap_for_children = true;
                }
            }

            let mut total_fill_weight: f32 = 0.0;
            let mut fill_children_indices: Vec<usize> = Vec::new();
            let mut fill_children_without_weight_indices: Vec<usize> = Vec::new();

            for i in 0..N {
                let item = &children_items_for_measure[i]; // Use cloned version
                if let DimensionValue::Fill { .. } = item.1 {
                    fill_children_indices.push(i);
                    if let Some(w) = item.0 {
                        if w > 0.0 {
                            total_fill_weight += w;
                        } else {
                            fill_children_without_weight_indices.push(i);
                        }
                    } else {
                        fill_children_without_weight_indices.push(i);
                    }
                }
            }

            let mut actual_width_taken_by_fill_children: u32 = 0;

            if is_row_effectively_wrap_for_children {
                for &index in &fill_children_indices {
                    let child_node_id = children_node_ids[index];
                    let child_constraint_for_measure =
                        Constraint::new(DimensionValue::Wrap, effective_row_constraint.height);
                    let final_child_constraint = metadatas
                        .get_mut(&child_node_id)
                        .unwrap()
                        .constraint
                        .merge(&child_constraint_for_measure);
                    let size =
                        measure_node(child_node_id, &final_child_constraint, tree, metadatas);
                    measured_children_sizes[index] = Some(size);
                    actual_width_taken_by_fill_children += size.width;
                    computed_max_row_height = computed_max_row_height.max(size.height);
                }
            } else if !fill_children_indices.is_empty() && remaining_width_for_fill > 0 {
                let mut temp_remaining_width = remaining_width_for_fill;

                if total_fill_weight > 0.0 {
                    for &index in &fill_children_indices {
                        let item = &children_items_for_measure[index]; // Use cloned version
                        if let Some(weight) = item.0
                            && weight > 0.0
                        {
                            let child_node_id = children_node_ids[index];
                            let proportional_width = ((weight / total_fill_weight)
                                * remaining_width_for_fill as f32)
                                as u32;

                            if let DimensionValue::Fill {
                                max: child_max_fill,
                            } = item.1
                            {
                                let alloc_width = child_max_fill
                                    .map_or(proportional_width, |m| proportional_width.min(m));
                                let final_alloc_width = alloc_width.min(temp_remaining_width);

                                let child_constraint_for_measure = Constraint::new(
                                    DimensionValue::Fixed(final_alloc_width),
                                    effective_row_constraint.height,
                                );
                                let final_child_constraint = metadatas
                                    .get_mut(&child_node_id)
                                    .unwrap()
                                    .constraint
                                    .merge(&child_constraint_for_measure);
                                let size = measure_node(
                                    child_node_id,
                                    &final_child_constraint,
                                    tree,
                                    metadatas,
                                );
                                measured_children_sizes[index] = Some(size);
                                actual_width_taken_by_fill_children += size.width;
                                temp_remaining_width =
                                    temp_remaining_width.saturating_sub(size.width);
                                computed_max_row_height = computed_max_row_height.max(size.height);
                            }
                        }
                    }
                }

                if !fill_children_without_weight_indices.is_empty() && temp_remaining_width > 0 {
                    let num_unweighted_fill = fill_children_without_weight_indices.len();
                    let width_per_unweighted_child =
                        temp_remaining_width / num_unweighted_fill as u32;

                    for &index in &fill_children_without_weight_indices {
                        let item = &children_items_for_measure[index]; // Use cloned version
                        let child_node_id = children_node_ids[index];
                        if let DimensionValue::Fill {
                            max: child_max_fill,
                        } = item.1
                        {
                            let alloc_width = child_max_fill
                                .map_or(width_per_unweighted_child, |m| {
                                    width_per_unweighted_child.min(m)
                                });
                            let final_alloc_width = alloc_width.min(temp_remaining_width);

                            let child_constraint_for_measure = Constraint::new(
                                DimensionValue::Fixed(final_alloc_width),
                                effective_row_constraint.height,
                            );
                            let final_child_constraint = metadatas
                                .get_mut(&child_node_id)
                                .unwrap()
                                .constraint
                                .merge(&child_constraint_for_measure);
                            let size = measure_node(
                                child_node_id,
                                &final_child_constraint,
                                tree,
                                metadatas,
                            );
                            measured_children_sizes[index] = Some(size);
                            actual_width_taken_by_fill_children += size.width;
                            temp_remaining_width = temp_remaining_width.saturating_sub(size.width);
                            computed_max_row_height = computed_max_row_height.max(size.height);
                        }
                    }
                }
            }

            let total_children_width =
                total_width_for_fixed_wrap + actual_width_taken_by_fill_children;

            let final_row_width = match effective_row_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Wrap => total_children_width,
                DimensionValue::Fill { max } => {
                    let resolved_fill_width = max.unwrap_or(total_children_width);
                    if max.is_some() {
                        total_children_width.min(resolved_fill_width)
                    } else {
                        total_children_width
                    }
                }
            };
            let final_row_height = match effective_row_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Wrap => computed_max_row_height,
                DimensionValue::Fill { max } => {
                    max.map_or(computed_max_row_height, |m| computed_max_row_height.min(m))
                }
            };

            let mut current_x_offset: u32 = 0;
            for i in 0..N {
                if let Some(size) = measured_children_sizes[i] {
                    place_node(children_node_ids[i], [current_x_offset, 0], metadatas);
                    current_x_offset += size.width;
                } else {
                    metadatas
                        .entry(children_node_ids[i])
                        .or_default()
                        .computed_data = Some(ComputedData {
                        width: 0,
                        height: 0,
                    });
                    place_node(children_node_ids[i], [current_x_offset, 0], metadatas);
                }
            }

            ComputedData {
                width: final_row_width,
                height: final_row_height,
            }
        },
    ));

    // Use the original children_items here, iterating by reference
    for item in children_items {
        (item.child)();
    }
}
