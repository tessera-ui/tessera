use tessera::{
    ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Px, PxPosition, measure_nodes,
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
        Self::new(
            child,
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
            None,
        )
    }

    /// Helper to create a ColumnItem that is fixed height.
    pub fn fixed(child: Box<dyn FnOnce() + Send + Sync>, height: Dp) -> Self {
        Self::new(child, DimensionValue::Fixed(height.to_pixels_u32()), None)
    }

    /// Helper to create a ColumnItem that fills available space (height),
    /// optionally with a weight and max_height.
    pub fn fill(
        child: Box<dyn FnOnce() + Send + Sync>,
        weight: Option<f32>,
        max_height: Option<Dp>,
    ) -> Self {
        Self::new(
            child,
            DimensionValue::Fill {
                min: None, // Add min field
                max: max_height.as_ref().map(Dp::to_pixels_u32),
            },
            weight,
        )
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
            height_behavior: DimensionValue::Wrap {
                min: None,
                max: None,
            }, // Default to Wrap for height
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
            let column_intrinsic_constraint = metadatas
                .get(&node_id)
                .ok_or(MeasurementError::NodeNotFoundInMeta)?
                .constraint;
            let effective_column_constraint =
                column_intrinsic_constraint.merge(column_parent_constraint);

            let mut measured_children_sizes: Vec<Option<ComputedData>> = vec![None; N];
            let mut total_height_for_fixed_wrap: u32 = 0;
            let mut computed_max_column_width: u32 = 0;

            // --- Stage 1: Measure Fixed and Wrap children ---
            let mut fixed_wrap_nodes_to_measure = Vec::new();
            for i in 0..N {
                let item_behavior = children_items_for_measure[i].1;
                let child_node_id = children_node_ids[i];
                match item_behavior {
                    DimensionValue::Fixed(fixed_height) => {
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Fixed(fixed_height),
                        );
                        let child_intrinsic_constraint = metadatas
                            .get(&child_node_id)
                            .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        fixed_wrap_nodes_to_measure.push((
                            child_node_id,
                            final_child_constraint,
                            i,
                        ));
                    }
                    DimensionValue::Wrap { .. } => {
                        // Updated match for Wrap
                        // For Wrap children, their height constraint is Wrap, but they respect column's width constraint.
                        // The min/max from the child's Wrap behavior will be part of its intrinsic constraint,
                        // which gets merged.
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Wrap {
                                min: None,
                                max: None,
                            }, // Pass a basic Wrap, intrinsic will merge its min/max
                        );
                        let child_intrinsic_constraint = metadatas
                            .get(&child_node_id)
                            .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        fixed_wrap_nodes_to_measure.push((
                            child_node_id,
                            final_child_constraint,
                            i,
                        ));
                    }
                    DimensionValue::Fill { .. } => {} // Fill children measured later
                }
            }

            if !fixed_wrap_nodes_to_measure.is_empty() {
                let nodes_for_api: Vec<_> = fixed_wrap_nodes_to_measure
                    .iter()
                    .map(|(id, constraint, _idx)| (*id, *constraint))
                    .collect();
                let results_map = measure_nodes(nodes_for_api, tree, metadatas);
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
                    total_height_for_fixed_wrap += size.height;
                    computed_max_column_width = computed_max_column_width.max(size.width);
                }
            }
            // --- End Stage 1 ---

            let mut remaining_height_for_fill: u32 = 0;
            let mut is_column_effectively_wrap_for_children = false;

            match effective_column_constraint.height {
                DimensionValue::Fixed(column_fixed_height) => {
                    remaining_height_for_fill =
                        column_fixed_height.saturating_sub(total_height_for_fixed_wrap);
                }
                DimensionValue::Wrap {
                    max: col_wrap_max, ..
                } => {
                    // Consider column's own wrap max
                    if let Some(max_h) = col_wrap_max {
                        remaining_height_for_fill =
                            max_h.saturating_sub(total_height_for_fixed_wrap);
                        // If max_h is already less than fixed/wrap, remaining could be 0 or negative (handled by saturating_sub)
                    } else {
                        is_column_effectively_wrap_for_children = true; // No max, so fill children wrap
                    }
                }
                DimensionValue::Fill {
                    max: Some(column_max_budget),
                    ..
                } => {
                    remaining_height_for_fill =
                        column_max_budget.saturating_sub(total_height_for_fixed_wrap);
                }
                DimensionValue::Fill { max: None, .. } => {
                    // This means the column itself can grow indefinitely if parent allows.
                    // So, Fill children effectively behave as Wrap unless they have their own max.
                    is_column_effectively_wrap_for_children = true;
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

            let mut actual_height_taken_by_fill_children: u32 = 0;

            // --- Stage 2: Measure Fill children ---
            let mut fill_nodes_to_measure_group = Vec::new();

            if is_column_effectively_wrap_for_children {
                // If column wraps or has unbounded fill, Fill children also wrap
                for i in 0..N {
                    let item_behavior = children_items_for_measure[i].1;
                    if let DimensionValue::Fill {
                        min: child_fill_min,
                        max: child_fill_max,
                    } = item_behavior
                    {
                        let child_node_id = children_node_ids[i];
                        // Child's Fill becomes Wrap, but respects its own min/max from Fill.
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Wrap {
                                min: child_fill_min,
                                max: child_fill_max,
                            },
                        );
                        let child_intrinsic_constraint = metadatas
                            .get(&child_node_id)
                            .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        fill_nodes_to_measure_group.push((
                            child_node_id,
                            final_child_constraint,
                            i,
                        ));
                    }
                }
            } else if remaining_height_for_fill > 0 {
                // Distribute remaining_height_for_fill among Fill children
                // This part is complex for full parallelization due to dependencies.
                // For now, simplified sequential logic for Fill distribution to ensure correctness.
                let mut current_remaining_fill_budget = remaining_height_for_fill;

                if total_fill_weight > 0.0 {
                    for &index in &fill_children_indices_with_weight {
                        if current_remaining_fill_budget == 0 {
                            break;
                        }
                        let item_weight = children_items_for_measure[index].0.unwrap();
                        let item_behavior = children_items_for_measure[index].1; // This is DimensionValue::Fill
                        let child_node_id = children_node_ids[index];

                        let proportional_height = ((item_weight / total_fill_weight)
                            * remaining_height_for_fill as f32)
                            as u32;

                        if let DimensionValue::Fill {
                            min: child_min,
                            max: child_max,
                            ..
                        } = item_behavior
                        {
                            let mut alloc_height = proportional_height;
                            if let Some(max_h) = child_max {
                                alloc_height = alloc_height.min(max_h);
                            }
                            alloc_height = alloc_height.min(current_remaining_fill_budget); // Cannot exceed current budget
                            if let Some(min_h) = child_min {
                                alloc_height = alloc_height.max(min_h);
                            }
                            alloc_height = alloc_height.min(current_remaining_fill_budget); // Re-cap after min

                            let child_constraint_for_measure = Constraint::new(
                                effective_column_constraint.width,
                                DimensionValue::Fixed(alloc_height),
                            );
                            let child_intrinsic_constraint = metadatas
                                .get(&child_node_id)
                                .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                                .constraint;
                            let final_child_constraint =
                                child_intrinsic_constraint.merge(&child_constraint_for_measure);
                            // Measure this child individually
                            let size = tessera::measure_node(
                                child_node_id,
                                &final_child_constraint,
                                tree,
                                metadatas,
                            )?;
                            measured_children_sizes[index] = Some(size);
                            actual_height_taken_by_fill_children += size.height;
                            current_remaining_fill_budget =
                                current_remaining_fill_budget.saturating_sub(size.height);
                            computed_max_column_width = computed_max_column_width.max(size.width);
                        }
                    }
                }

                if !fill_children_indices_without_weight.is_empty()
                    && current_remaining_fill_budget > 0
                {
                    let num_unweighted_fill = fill_children_indices_without_weight.len();
                    let height_per_unweighted_child =
                        current_remaining_fill_budget / num_unweighted_fill as u32;

                    for &index in &fill_children_indices_without_weight {
                        if current_remaining_fill_budget == 0 {
                            break;
                        }
                        let item_behavior = children_items_for_measure[index].1; // This is DimensionValue::Fill
                        let child_node_id = children_node_ids[index];

                        if let DimensionValue::Fill {
                            min: child_min,
                            max: child_max,
                            ..
                        } = item_behavior
                        {
                            let mut alloc_height = height_per_unweighted_child;
                            if let Some(max_h) = child_max {
                                alloc_height = alloc_height.min(max_h);
                            }
                            alloc_height = alloc_height.min(current_remaining_fill_budget);
                            if let Some(min_h) = child_min {
                                alloc_height = alloc_height.max(min_h);
                            }
                            alloc_height = alloc_height.min(current_remaining_fill_budget);

                            let child_constraint_for_measure = Constraint::new(
                                effective_column_constraint.width,
                                DimensionValue::Fixed(alloc_height),
                            );
                            let child_intrinsic_constraint = metadatas
                                .get(&child_node_id)
                                .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                                .constraint;
                            let final_child_constraint =
                                child_intrinsic_constraint.merge(&child_constraint_for_measure);
                            // Measure this child individually
                            let size = tessera::measure_node(
                                child_node_id,
                                &final_child_constraint,
                                tree,
                                metadatas,
                            )?;
                            measured_children_sizes[index] = Some(size);
                            actual_height_taken_by_fill_children += size.height;
                            current_remaining_fill_budget =
                                current_remaining_fill_budget.saturating_sub(size.height);
                            computed_max_column_width = computed_max_column_width.max(size.width);
                        }
                    }
                }
            }

            if !fill_nodes_to_measure_group.is_empty() {
                let nodes_for_api: Vec<_> = fill_nodes_to_measure_group
                    .iter()
                    .map(|(id, constraint, _idx)| (*id, *constraint))
                    .collect();
                let results_map = measure_nodes(nodes_for_api, tree, metadatas);
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
                    actual_height_taken_by_fill_children += size.height;
                    computed_max_column_width = computed_max_column_width.max(size.width);
                }
            }
            // --- End Stage 2 ---

            let total_children_height =
                total_height_for_fixed_wrap + actual_height_taken_by_fill_children;

            let mut final_column_height = total_children_height;
            match effective_column_constraint.height {
                DimensionValue::Fixed(h) => final_column_height = h,
                DimensionValue::Wrap { min, max } => {
                    if let Some(min_h) = min {
                        final_column_height = final_column_height.max(min_h);
                    }
                    if let Some(max_h) = max {
                        final_column_height = final_column_height.min(max_h);
                    }
                }
                DimensionValue::Fill { min, max } => {
                    // For Fill, the column's height is constrained by its parent.
                    // The `total_children_height` is what children want.
                    // If parent provides a fixed height (via merge), that's the budget.
                    // If parent provides a max fill height, that's the budget.
                    // The column's own min/max from its Fill constraint also apply.
                    let parent_provided_height = match column_parent_constraint.height {
                        DimensionValue::Fixed(ph) => Some(ph),
                        DimensionValue::Fill {
                            max: p_max_fill, ..
                        } => p_max_fill,
                        _ => None, // Parent is Wrap or unbounded Fill, so no hard limit from parent
                    };

                    if let Some(pph) = parent_provided_height {
                        final_column_height = total_children_height.min(pph);
                    } // else, it's effectively wrapping content or bounded by its own max.

                    if let Some(min_h) = min {
                        final_column_height = final_column_height.max(min_h);
                    }
                    if let Some(max_h) = max {
                        final_column_height = final_column_height.min(max_h);
                    }
                }
            };

            let mut final_column_width = computed_max_column_width;
            match effective_column_constraint.width {
                DimensionValue::Fixed(w) => final_column_width = w,
                DimensionValue::Wrap { min, max } => {
                    if let Some(min_w) = min {
                        final_column_width = final_column_width.max(min_w);
                    }
                    if let Some(max_w) = max {
                        final_column_width = final_column_width.min(max_w);
                    }
                }
                DimensionValue::Fill { min, max } => {
                    let parent_provided_width = match column_parent_constraint.width {
                        DimensionValue::Fixed(pw) => Some(pw),
                        DimensionValue::Fill {
                            max: p_max_fill, ..
                        } => p_max_fill,
                        _ => None,
                    };

                    if let Some(ppw) = parent_provided_width {
                        final_column_width = ppw; // Fill should use parent's provided width
                    } else {
                        // When no parent provides width, Fill should wrap content (like a Wrap behavior)
                        final_column_width = computed_max_column_width;
                    }
                    if let Some(min_w) = min {
                        final_column_width = final_column_width.max(min_w);
                    }
                    if let Some(max_w) = max {
                        final_column_width = final_column_width.min(max_w);
                    }
                }
            };

            let mut current_y_offset: u32 = 0;
            for i in 0..N {
                let child_node_id = children_node_ids[i];
                if let Some(size) = measured_children_sizes[i] {
                    place_node(
                        child_node_id,
                        PxPosition::new(Px(0), Px(current_y_offset as i32)),
                        metadatas,
                    );
                    current_y_offset += size.height;
                } else {
                    // This case should ideally not be hit if all measurements are successful or errors handled.
                    // If a Fill child got 0 budget and wasn't measured, its size is 0.
                    let mut meta_entry = metadatas.entry(child_node_id).or_default();
                    if meta_entry.computed_data.is_none() {
                        // Only set if not already set (e.g. by an error path)
                        meta_entry.computed_data = Some(ComputedData::ZERO);
                    }
                    place_node(
                        child_node_id,
                        PxPosition::new(Px(0), Px(current_y_offset as i32)),
                        metadatas,
                    );
                }
            }

            Ok(ComputedData {
                width: final_column_width,
                height: final_column_height,
            })
        },
    ));

    for item in children_items {
        (item.child)();
    }
}
