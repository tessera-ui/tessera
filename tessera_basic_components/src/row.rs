use tessera::Dp; // Added Dp import
use tessera::{
    ComputedData, Constraint, DimensionValue, MeasurementError, Px, PxPosition, measure_nodes,
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
        Self::new(
            child,
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
            None,
        )
    }

    /// Helper to create a RowItem that is fixed width.
    pub fn fixed(child: Box<dyn FnOnce() + Send + Sync>, width: Dp) -> Self {
        Self::new(child, DimensionValue::Fixed(width.into()), None)
    }

    /// Helper to create a RowItem that fills available space, optionally with a weight and max.
    pub fn fill(
        child: Box<dyn FnOnce() + Send + Sync>,
        weight: Option<f32>,
        max_width: Option<Dp>,
    ) -> Self {
        Self::new(
            child,
            DimensionValue::Fill {
                min: None, // Add min field
                max: max_width.map(|dp| dp.into()),
            },
            weight,
        )
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
            width_behavior: DimensionValue::Wrap {
                min: None,
                max: None,
            }, // Default to Wrap
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
    let children_items_for_measure: Vec<(_, _)> = children_items
        .iter()
        .map(|child| (child.weight, child.width_behavior))
        .collect(); // For the measure closure

    measure(Box::new(move |input| {
        let row_intrinsic_constraint = input
            .metadatas
            .get(&input.current_node_id)
            .ok_or(MeasurementError::NodeNotFoundInMeta)?
            .constraint;
        let effective_row_constraint = row_intrinsic_constraint.merge(input.effective_constraint);

        let mut measured_children_sizes: Vec<Option<ComputedData>> = vec![None; N];
        let mut total_width_for_fixed_wrap: Px = Px(0);
        let mut computed_max_row_height: Px = Px(0);

        // --- Stage 1: Measure Fixed and Wrap children ---
        let mut fixed_wrap_nodes_to_measure = Vec::new();
        for i in 0..N {
            let item_behavior = children_items_for_measure[i].1;
            let child_node_id = input.children_ids[i];
            match item_behavior {
                DimensionValue::Fixed(fixed_width) => {
                    let child_constraint_for_measure = Constraint::new(
                        DimensionValue::Fixed(fixed_width),
                        effective_row_constraint.height,
                    );
                    let child_intrinsic_constraint = input
                        .metadatas
                        .get(&child_node_id)
                        .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                        .constraint;
                    let final_child_constraint =
                        child_intrinsic_constraint.merge(&child_constraint_for_measure);
                    fixed_wrap_nodes_to_measure.push((child_node_id, final_child_constraint, i));
                }
                DimensionValue::Wrap { .. } => {
                    // Updated match for Wrap
                    let child_constraint_for_measure = Constraint::new(
                        DimensionValue::Wrap {
                            min: None,
                            max: None,
                        }, // Pass basic Wrap
                        effective_row_constraint.height,
                    );
                    let child_intrinsic_constraint = input
                        .metadatas
                        .get(&child_node_id)
                        .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                        .constraint;
                    let final_child_constraint =
                        child_intrinsic_constraint.merge(&child_constraint_for_measure);
                    fixed_wrap_nodes_to_measure.push((child_node_id, final_child_constraint, i));
                }
                DimensionValue::Fill { .. } => {} // Fill children measured later
            }
        }

        if !fixed_wrap_nodes_to_measure.is_empty() {
            let nodes_for_api: Vec<(_, _)> = fixed_wrap_nodes_to_measure
                .iter()
                .map(|(id, constraint, _idx)| (*id, *constraint))
                .collect();
            let results_map = measure_nodes(nodes_for_api, input.tree, input.metadatas);
            for (child_node_id, _constraint, original_idx) in fixed_wrap_nodes_to_measure {
                let size = results_map
                    .get(&child_node_id)
                    .ok_or_else(|| {
                        MeasurementError::MeasureFnFailed(format!(
                            "Result missing for fixed/wrap child {child_node_id:?}"
                        ))
                    })?
                    .clone()?;
                measured_children_sizes[original_idx] = Some(size);
                total_width_for_fixed_wrap += size.width;
                computed_max_row_height = computed_max_row_height.max(size.height);
            }
        }
        // --- End Stage 1 ---

        let mut remaining_width_for_fill: Px = Px(0);
        let mut is_row_effectively_wrap_for_children = false;

        match effective_row_constraint.width {
            DimensionValue::Fixed(row_fixed_width) => {
                remaining_width_for_fill =
                    (row_fixed_width - total_width_for_fixed_wrap).max(Px(0));
            }
            DimensionValue::Wrap {
                max: row_wrap_max, ..
            } => {
                // Consider row's own wrap max
                if let Some(max_w) = row_wrap_max {
                    remaining_width_for_fill = (max_w - total_width_for_fixed_wrap).max(Px(0));
                } else {
                    is_row_effectively_wrap_for_children = true; // No max, so fill children wrap
                }
            }
            DimensionValue::Fill {
                max: Some(row_max_budget),
                ..
            } => {
                remaining_width_for_fill = (row_max_budget - total_width_for_fixed_wrap).max(Px(0));
            }
            DimensionValue::Fill { max: None, .. } => {
                is_row_effectively_wrap_for_children = true;
            }
        }

        let mut total_fill_weight: f32 = 0.0;
        let mut fill_children_indices_with_weight: Vec<usize> = Vec::new();
        let mut fill_children_indices_without_weight: Vec<usize> = Vec::new();

        for (i, _item) in children_items_for_measure.iter().enumerate().take(N) {
            let item_weight = children_items_for_measure[i].0;
            let item_behavior = children_items_for_measure[i].1;
            if let DimensionValue::Fill { .. } = item_behavior {
                if let Some(w) = item_weight {
                    if w > 0.0 {
                        fill_children_indices_with_weight.push(i);
                        total_fill_weight += w;
                    } else {
                        fill_children_indices_without_weight.push(i);
                    }
                } else {
                    fill_children_indices_without_weight.push(i);
                }
            }
        }

        let mut actual_width_taken_by_fill_children: Px = Px(0);

        // --- Stage 2: Measure Fill children ---
        let mut fill_nodes_to_measure_group = Vec::new();

        if is_row_effectively_wrap_for_children {
            for i in 0..N {
                let item_behavior = children_items_for_measure[i].1;
                if let DimensionValue::Fill {
                    min: child_fill_min,
                    max: child_fill_max,
                } = item_behavior
                {
                    let child_node_id = input.children_ids[i];
                    let child_constraint_for_measure = Constraint::new(
                        DimensionValue::Wrap {
                            min: child_fill_min,
                            max: child_fill_max,
                        },
                        effective_row_constraint.height,
                    );
                    let child_intrinsic_constraint = input
                        .metadatas
                        .get(&child_node_id)
                        .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                        .constraint;
                    let final_child_constraint =
                        child_intrinsic_constraint.merge(&child_constraint_for_measure);
                    fill_nodes_to_measure_group.push((child_node_id, final_child_constraint, i));
                }
            }
        } else if remaining_width_for_fill > Px(0) {
            let mut current_remaining_fill_budget = remaining_width_for_fill;

            if total_fill_weight > 0.0 {
                for &index in &fill_children_indices_with_weight {
                    if current_remaining_fill_budget == Px(0) {
                        break;
                    }
                    let item_weight = children_items_for_measure[index].0.unwrap();
                    let item_behavior = children_items_for_measure[index].1;
                    let child_node_id = input.children_ids[index];

                    let proportional_width = Px(((item_weight / total_fill_weight)
                        * remaining_width_for_fill.0 as f32)
                        as i32);

                    if let DimensionValue::Fill {
                        min: child_min,
                        max: child_max,
                        ..
                    } = item_behavior
                    {
                        let mut alloc_width = proportional_width;
                        if let Some(max_w) = child_max {
                            alloc_width = alloc_width.min(max_w);
                        }
                        alloc_width = alloc_width.min(current_remaining_fill_budget);
                        if let Some(min_w) = child_min {
                            alloc_width = alloc_width.max(min_w);
                        }
                        alloc_width = alloc_width.min(current_remaining_fill_budget);

                        let child_constraint_for_measure = Constraint::new(
                            DimensionValue::Fixed(alloc_width),
                            effective_row_constraint.height,
                        );
                        let child_intrinsic_constraint = input
                            .metadatas
                            .get(&child_node_id)
                            .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        let size = tessera::measure_node(
                            child_node_id,
                            &final_child_constraint,
                            input.tree,
                            input.metadatas,
                        )?;
                        measured_children_sizes[index] = Some(size);
                        actual_width_taken_by_fill_children += size.width;
                        current_remaining_fill_budget =
                            (current_remaining_fill_budget - size.width).max(Px(0));
                        computed_max_row_height = computed_max_row_height.max(size.height);
                    }
                }
            }

            if !fill_children_indices_without_weight.is_empty()
                && current_remaining_fill_budget > Px(0)
            {
                let num_unweighted_fill = fill_children_indices_without_weight.len();
                let width_per_unweighted_child =
                    current_remaining_fill_budget / (num_unweighted_fill as i32);

                for &index in &fill_children_indices_without_weight {
                    if current_remaining_fill_budget == Px(0) {
                        break;
                    }
                    let item_behavior = children_items_for_measure[index].1;
                    let child_node_id = input.children_ids[index];

                    if let DimensionValue::Fill {
                        min: child_min,
                        max: child_max,
                        ..
                    } = item_behavior
                    {
                        let mut alloc_width = width_per_unweighted_child;
                        if let Some(max_w) = child_max {
                            alloc_width = alloc_width.min(max_w);
                        }
                        alloc_width = alloc_width.min(current_remaining_fill_budget);
                        if let Some(min_w) = child_min {
                            alloc_width = alloc_width.max(min_w);
                        }
                        alloc_width = alloc_width.min(current_remaining_fill_budget);

                        let child_constraint_for_measure = Constraint::new(
                            DimensionValue::Fixed(alloc_width),
                            effective_row_constraint.height,
                        );
                        let child_intrinsic_constraint = input
                            .metadatas
                            .get(&child_node_id)
                            .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        let size = tessera::measure_node(
                            child_node_id,
                            &final_child_constraint,
                            input.tree,
                            input.metadatas,
                        )?;
                        measured_children_sizes[index] = Some(size);
                        actual_width_taken_by_fill_children += size.width;
                        current_remaining_fill_budget =
                            (current_remaining_fill_budget - size.width).max(Px(0));
                        computed_max_row_height = computed_max_row_height.max(size.height);
                    }
                }
            }
        }

        if !fill_nodes_to_measure_group.is_empty() {
            let nodes_for_api: Vec<(_, _)> = fill_nodes_to_measure_group
                .iter()
                .map(|(id, constraint, _idx)| (*id, *constraint))
                .collect();
            let results_map = measure_nodes(nodes_for_api, input.tree, input.metadatas);
            for (child_node_id, _constraint, original_idx) in fill_nodes_to_measure_group {
                let size = results_map
                    .get(&child_node_id)
                    .ok_or_else(|| {
                        MeasurementError::MeasureFnFailed(format!(
                            "Result missing for fill/wrap child {child_node_id:?}"
                        ))
                    })?
                    .clone()?;
                measured_children_sizes[original_idx] = Some(size);
                actual_width_taken_by_fill_children += size.width;
                computed_max_row_height = computed_max_row_height.max(size.height);
            }
        }
        // --- End Stage 2 ---

        let total_children_width = total_width_for_fixed_wrap + actual_width_taken_by_fill_children;

        let mut final_row_width = total_children_width;
        match effective_row_constraint.width {
            DimensionValue::Fixed(w) => final_row_width = w,
            DimensionValue::Wrap { min, max } => {
                if let Some(min_w) = min {
                    final_row_width = final_row_width.max(min_w);
                }
                if let Some(max_w) = max {
                    final_row_width = final_row_width.min(max_w);
                }
            }
            DimensionValue::Fill { min, max } => {
                let parent_provided_width = match input.effective_constraint.width {
                    DimensionValue::Fixed(pw) => Some(pw),
                    DimensionValue::Fill {
                        max: p_max_fill, ..
                    } => p_max_fill,
                    _ => None,
                };
                if let Some(ppw) = parent_provided_width {
                    final_row_width = ppw; // Fill should use parent's provided width
                } else {
                    // When no parent provides width, Fill should wrap content (like a Wrap behavior)
                    final_row_width = total_children_width;
                }
                if let Some(min_w) = min {
                    final_row_width = final_row_width.max(min_w);
                }
                if let Some(max_w) = max {
                    final_row_width = final_row_width.min(max_w);
                }
            }
        };

        let mut final_row_height = computed_max_row_height;
        match effective_row_constraint.height {
            DimensionValue::Fixed(h) => final_row_height = h,
            DimensionValue::Wrap { min, max } => {
                if let Some(min_h) = min {
                    final_row_height = final_row_height.max(min_h);
                }
                if let Some(max_h) = max {
                    final_row_height = final_row_height.min(max_h);
                }
            }
            DimensionValue::Fill { min, max } => {
                let parent_provided_height = match input.effective_constraint.height {
                    DimensionValue::Fixed(ph) => Some(ph),
                    DimensionValue::Fill {
                        max: p_max_fill, ..
                    } => p_max_fill,
                    _ => None,
                };
                if let Some(pph) = parent_provided_height {
                    final_row_height = computed_max_row_height.min(pph);
                }
                if let Some(min_h) = min {
                    final_row_height = final_row_height.max(min_h);
                }
                if let Some(max_h) = max {
                    final_row_height = final_row_height.min(max_h);
                }
            }
        };

        let mut current_x_offset: Px = Px(0);
        for i in 0..N {
            let child_node_id = input.children_ids[i];
            if let Some(size) = measured_children_sizes[i] {
                place_node(
                    child_node_id,
                    PxPosition::new(current_x_offset, Px(0)),
                    input.metadatas,
                );
                current_x_offset += size.width;
            } else {
                let mut meta_entry = input.metadatas.entry(child_node_id).or_default();
                if meta_entry.computed_data.is_none() {
                    meta_entry.computed_data = Some(ComputedData::ZERO);
                }
                place_node(
                    child_node_id,
                    PxPosition::new(current_x_offset, Px(0)),
                    input.metadatas,
                );
            }
        }

        Ok(ComputedData {
            width: final_row_width,
            height: final_row_height,
        })
    }));

    for item in children_items {
        (item.child)();
    }
}
