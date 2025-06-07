use tessera::{
    ComputedData,
    Constraint,
    DimensionValue, // Removed ComponentNodeMetaDatas
    measure_node,
    place_node,
};
use tessera_macros::tessera;

/// Represents a child item within a Column layout.
pub struct ColumnItem {
    /// Determines how much space the child should take if its height_behavior is Fill,
    /// relative to other Fill children with weights.
    pub weight: Option<f32>,
    /// Defines the height behavior of this child.
    pub height_behavior: DimensionValue,
    /// The actual child component. Must be Send + Sync.
    pub child: Box<dyn FnOnce() + Send + Sync>,
}

impl ColumnItem {
    /// Creates a new `ColumnItem`.
    pub fn new(
        child: Box<dyn FnOnce() + Send + Sync>,
        height_behavior: DimensionValue,
        weight: Option<f32>,
    ) -> Self {
        ColumnItem {
            weight,
            height_behavior,
            child,
        }
    }

    /// Helper to create a ColumnItem that wraps its content (height).
    pub fn wrap(child: Box<dyn FnOnce() + Send + Sync>) -> Self {
        Self::new(child, DimensionValue::Wrap, None)
    }

    /// Helper to create a ColumnItem that is fixed height.
    pub fn fixed(child: Box<dyn FnOnce() + Send + Sync>, height: u32) -> Self {
        Self::new(child, DimensionValue::Fixed(height), None)
    }

    /// Helper to create a ColumnItem that fills available space (height),
    /// optionally with a weight and max_height.
    pub fn fill(
        child: Box<dyn FnOnce() + Send + Sync>,
        weight: Option<f32>,
        max_height: Option<u32>,
    ) -> Self {
        Self::new(child, DimensionValue::Fill { max: max_height }, weight)
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

/// Default conversion: a simple function closure becomes a `ColumnItem` that wraps its content (height).
impl<F: Fn() + Send + Sync + 'static> AsColumnItem for F {
    fn into_column_item(self) -> ColumnItem {
        ColumnItem {
            weight: None,
            height_behavior: DimensionValue::Wrap, // Default to Wrap for height
            child: Box::new(self),
        }
    }
}

// Allow (Fn, DimensionValue_for_height) to be a ColumnItem
impl<F: Fn() + Send + Sync + 'static> AsColumnItem for (F, DimensionValue) {
    fn into_column_item(self) -> ColumnItem {
        ColumnItem {
            weight: None,
            height_behavior: self.1,
            child: Box::new(self.0),
        }
    }
}

// Allow (Fn, DimensionValue_for_height, f32_weight) to be a ColumnItem
impl<F: Fn() + Send + Sync + 'static> AsColumnItem for (F, DimensionValue, f32) {
    fn into_column_item(self) -> ColumnItem {
        ColumnItem {
            weight: Some(self.2),
            height_behavior: self.1,
            child: Box::new(self.0),
        }
    }
}

/// A column component that arranges its children vertically.
/// Children can have fixed sizes, wrap their content, or fill available space (optionally with weights).
#[tessera]
pub fn column<const N: usize>(children_items_input: [impl AsColumnItem; N]) {
    let children_items: [ColumnItem; N] =
        children_items_input.map(|item_input| item_input.into_column_item());
    let children_items_for_measure: Vec<_> = children_items
        .iter()
        .map(|child| (child.weight, child.height_behavior))
        .collect(); // For the measure closure

    measure(Box::new(
        move |node_id, tree, column_parent_constraint, children_node_ids, metadatas| {
            // Use children_items_for_measure inside this closure
            let effective_column_constraint = metadatas
                .get_mut(&node_id)
                .unwrap()
                .constraint
                .merge(column_parent_constraint);

            let mut measured_children_sizes: Vec<Option<ComputedData>> = vec![None; N];
            let mut total_height_for_fixed_wrap: u32 = 0;
            let mut computed_max_column_width: u32 = 0;

            for i in 0..N {
                let item = &children_items_for_measure[i]; // Use cloned version
                let child_node_id = children_node_ids[i];

                match item.1 {
                    DimensionValue::Fixed(fixed_height) => {
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Fixed(fixed_height),
                        );
                        let final_child_constraint = metadatas
                            .get_mut(&child_node_id)
                            .unwrap()
                            .constraint
                            .merge(&child_constraint_for_measure);
                        let size =
                            measure_node(child_node_id, &final_child_constraint, tree, metadatas);
                        measured_children_sizes[i] = Some(size);
                        total_height_for_fixed_wrap += size.height;
                        computed_max_column_width = computed_max_column_width.max(size.width);
                    }
                    DimensionValue::Wrap => {
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Wrap,
                        );
                        let final_child_constraint = metadatas
                            .get_mut(&child_node_id)
                            .unwrap()
                            .constraint
                            .merge(&child_constraint_for_measure);
                        let size =
                            measure_node(child_node_id, &final_child_constraint, tree, metadatas);
                        measured_children_sizes[i] = Some(size);
                        total_height_for_fixed_wrap += size.height;
                        computed_max_column_width = computed_max_column_width.max(size.width);
                    }
                    DimensionValue::Fill { .. } => {}
                }
            }

            let mut remaining_height_for_fill: u32 = 0;
            let mut is_column_effectively_wrap_for_children = false;

            match effective_column_constraint.height {
                DimensionValue::Fixed(column_fixed_height) => {
                    remaining_height_for_fill =
                        column_fixed_height.saturating_sub(total_height_for_fixed_wrap);
                }
                DimensionValue::Wrap => {
                    is_column_effectively_wrap_for_children = true;
                }
                DimensionValue::Fill {
                    max: Some(column_max_budget),
                } => {
                    remaining_height_for_fill =
                        column_max_budget.saturating_sub(total_height_for_fixed_wrap);
                }
                DimensionValue::Fill { max: None } => {
                    is_column_effectively_wrap_for_children = true;
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

            let mut actual_height_taken_by_fill_children: u32 = 0;

            if is_column_effectively_wrap_for_children {
                for &index in &fill_children_indices {
                    let child_node_id = children_node_ids[index];
                    let child_constraint_for_measure =
                        Constraint::new(effective_column_constraint.width, DimensionValue::Wrap);
                    let final_child_constraint = metadatas
                        .get_mut(&child_node_id)
                        .unwrap()
                        .constraint
                        .merge(&child_constraint_for_measure);
                    let size =
                        measure_node(child_node_id, &final_child_constraint, tree, metadatas);
                    measured_children_sizes[index] = Some(size);
                    actual_height_taken_by_fill_children += size.height;
                    computed_max_column_width = computed_max_column_width.max(size.width);
                }
            } else if !fill_children_indices.is_empty() && remaining_height_for_fill > 0 {
                let mut temp_remaining_height = remaining_height_for_fill;

                if total_fill_weight > 0.0 {
                    for &index in &fill_children_indices {
                        let item = &children_items_for_measure[index]; // Use cloned version
                        if let Some(weight) = item.0
                            && weight > 0.0
                        {
                            let child_node_id = children_node_ids[index];
                            let proportional_height = ((weight / total_fill_weight)
                                * remaining_height_for_fill as f32)
                                as u32;

                            if let DimensionValue::Fill {
                                max: child_max_fill,
                            } = item.1
                            {
                                let alloc_height = child_max_fill
                                    .map_or(proportional_height, |m| proportional_height.min(m));
                                let final_alloc_height = alloc_height.min(temp_remaining_height);

                                let child_constraint_for_measure = Constraint::new(
                                    effective_column_constraint.width,
                                    DimensionValue::Fixed(final_alloc_height),
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
                                actual_height_taken_by_fill_children += size.height;
                                temp_remaining_height =
                                    temp_remaining_height.saturating_sub(size.height);
                                computed_max_column_width =
                                    computed_max_column_width.max(size.width);
                            }
                        }
                    }
                }

                if !fill_children_without_weight_indices.is_empty() && temp_remaining_height > 0 {
                    let num_unweighted_fill = fill_children_without_weight_indices.len();
                    let height_per_unweighted_child =
                        temp_remaining_height / num_unweighted_fill as u32;

                    for &index in &fill_children_without_weight_indices {
                        let item = &children_items_for_measure[index]; // Use cloned version
                        let child_node_id = children_node_ids[index];
                        if let DimensionValue::Fill {
                            max: child_max_fill,
                        } = item.1
                        {
                            let alloc_height = child_max_fill
                                .map_or(height_per_unweighted_child, |m| {
                                    height_per_unweighted_child.min(m)
                                });
                            let final_alloc_height = alloc_height.min(temp_remaining_height);

                            let child_constraint_for_measure = Constraint::new(
                                effective_column_constraint.width,
                                DimensionValue::Fixed(final_alloc_height),
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
                            actual_height_taken_by_fill_children += size.height;
                            temp_remaining_height =
                                temp_remaining_height.saturating_sub(size.height);
                            computed_max_column_width = computed_max_column_width.max(size.width);
                        }
                    }
                }
            }

            let total_children_height =
                total_height_for_fixed_wrap + actual_height_taken_by_fill_children;

            let final_column_height = match effective_column_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Wrap => total_children_height,
                DimensionValue::Fill { max } => {
                    let resolved_fill_height = max.unwrap_or(total_children_height);
                    if max.is_some() {
                        total_children_height.min(resolved_fill_height)
                    } else {
                        total_children_height
                    }
                }
            };
            let final_column_width = match effective_column_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Wrap => computed_max_column_width,
                DimensionValue::Fill { max } => max.map_or(computed_max_column_width, |m| {
                    computed_max_column_width.min(m)
                }),
            };

            let mut current_y_offset: u32 = 0;
            for i in 0..N {
                if let Some(size) = measured_children_sizes[i] {
                    place_node(children_node_ids[i], [0, current_y_offset], metadatas);
                    current_y_offset += size.height;
                } else {
                    metadatas
                        .entry(children_node_ids[i])
                        .or_default()
                        .computed_data = Some(ComputedData {
                        width: 0,
                        height: 0,
                    });
                    place_node(children_node_ids[i], [0, current_y_offset], metadatas);
                }
            }

            ComputedData {
                width: final_column_width,
                height: final_column_height,
            }
        },
    ));

    // Use the original children_items here, iterating by reference
    for item in children_items {
        (item.child)();
    }
}
