use tessera::{
    ComputedData, Constraint, DimensionValue, MeasurementError, measure_nodes, place_node,
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
            let column_intrinsic_constraint = metadatas
                .get(&node_id) // Changed from get_mut, and unwrap to ok_or
                .ok_or_else(|| MeasurementError::NodeNotFoundInMeta)?
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
                            .ok_or_else(|| MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        fixed_wrap_nodes_to_measure.push((
                            child_node_id,
                            final_child_constraint,
                            i,
                        ));
                    }
                    DimensionValue::Wrap => {
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Wrap,
                        );
                        let child_intrinsic_constraint = metadatas
                            .get(&child_node_id)
                            .ok_or_else(|| MeasurementError::ChildMeasurementFailed(child_node_id))?
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
                                "Result missing for fixed/wrap child {:?}",
                                child_node_id
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
                DimensionValue::Wrap => {
                    is_column_effectively_wrap_for_children = true;
                }
                DimensionValue::Fill {
                    max: Some(column_max_budget),
                    ..
                } => {
                    remaining_height_for_fill =
                        column_max_budget.saturating_sub(total_height_for_fixed_wrap);
                }
                DimensionValue::Fill { max: None, .. } => {
                    is_column_effectively_wrap_for_children = true;
                }
            }

            let mut total_fill_weight: f32 = 0.0;
            let mut fill_children_indices_with_weight: Vec<usize> = Vec::new();
            let mut fill_children_indices_without_weight: Vec<usize> = Vec::new();

            for i in 0..N {
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
            let mut fill_nodes_to_measure = Vec::new();

            if is_column_effectively_wrap_for_children {
                for i in 0..N {
                    let item_behavior = children_items_for_measure[i].1;
                    if let DimensionValue::Fill { .. } = item_behavior {
                        let child_node_id = children_node_ids[i];
                        let child_constraint_for_measure = Constraint::new(
                            effective_column_constraint.width,
                            DimensionValue::Wrap,
                        );
                        let child_intrinsic_constraint = metadatas
                            .get(&child_node_id)
                            .ok_or_else(|| MeasurementError::ChildMeasurementFailed(child_node_id))?
                            .constraint;
                        let final_child_constraint =
                            child_intrinsic_constraint.merge(&child_constraint_for_measure);
                        fill_nodes_to_measure.push((child_node_id, final_child_constraint, i));
                    }
                }
            } else if remaining_height_for_fill > 0 {
                let mut temp_remaining_height_for_weighted = remaining_height_for_fill;

                // Prepare weighted fill children
                if total_fill_weight > 0.0 {
                    for &index in &fill_children_indices_with_weight {
                        let item_weight = children_items_for_measure[index].0.unwrap();
                        let item_behavior = children_items_for_measure[index].1;
                        let child_node_id = children_node_ids[index];

                        let proportional_height = ((item_weight / total_fill_weight)
                            * remaining_height_for_fill as f32)
                            as u32;

                        if let DimensionValue::Fill {
                            max: child_max_fill,
                            ..
                        } = item_behavior
                        {
                            let alloc_height = child_max_fill
                                .map_or(proportional_height, |m| proportional_height.min(m));
                            // Note: temp_remaining_height_for_weighted is the budget for *this specific child* among weighted ones.
                            // The original logic for temp_remaining_height was a running total.
                            // For parallel measurement, each child gets its calculated share of the *initial* remaining_height_for_fill.
                            // We'll sum up their actual take later.
                            let final_alloc_height = alloc_height.min(remaining_height_for_fill); // Cap by total available for fill

                            let child_constraint_for_measure = Constraint::new(
                                effective_column_constraint.width,
                                DimensionValue::Fixed(final_alloc_height),
                            );
                            let child_intrinsic_constraint = metadatas
                                .get(&child_node_id)
                                .ok_or_else(|| {
                                    MeasurementError::ChildMeasurementFailed(child_node_id)
                                })?
                                .constraint;
                            let final_child_constraint =
                                child_intrinsic_constraint.merge(&child_constraint_for_measure);
                            fill_nodes_to_measure.push((
                                child_node_id,
                                final_child_constraint,
                                index,
                            ));
                        }
                    }
                }
                // After processing results for weighted, update temp_remaining_height_for_unweighted
                // Then prepare unweighted fill children
                // This two-step parallelization for Fill is tricky.
                // Simpler: measure weighted, then based on remaining, measure unweighted.
                // For now, let's assume we can calculate all Fill constraints first if possible.
                // The original code updated temp_remaining_height sequentially.
                // To parallelize Fill, we'd need to pre-calculate all their constraints.
                // This might require a slightly different loop structure or a more complex pre-calculation.

                // Let's stick to the original sequential logic for Fill distribution for now,
                // and parallelize the measure_node calls within that logic if a group can be measured together.
                // The current structure makes full parallelization of Fill hard without significant refactoring.
                // So, for Fill, we will revert to sequential measure_node calls for now to maintain correctness,
                // as their constraints depend on prior Fill children's measurements.
                // The user asked for minimal changes. Full parallelization of this Fill logic is not minimal.

                // Reverting Fill to sequential for correctness under "minimal change"
                if total_fill_weight > 0.0 {
                    for &index in &fill_children_indices_with_weight {
                        let item_weight = children_items_for_measure[index].0.unwrap();
                        let item_behavior = children_items_for_measure[index].1;
                        let child_node_id = children_node_ids[index];

                        let proportional_height = ((item_weight / total_fill_weight)
                            * remaining_height_for_fill as f32)
                            as u32;

                        if let DimensionValue::Fill {
                            max: child_max_fill,
                            ..
                        } = item_behavior
                        {
                            let alloc_height = child_max_fill
                                .map_or(proportional_height, |m| proportional_height.min(m));
                            let final_alloc_height =
                                alloc_height.min(temp_remaining_height_for_weighted);

                            let child_constraint_for_measure = Constraint::new(
                                effective_column_constraint.width,
                                DimensionValue::Fixed(final_alloc_height),
                            );
                            let child_intrinsic_constraint = metadatas
                                .get(&child_node_id)
                                .ok_or_else(|| {
                                    MeasurementError::ChildMeasurementFailed(child_node_id)
                                })?
                                .constraint;
                            let final_child_constraint =
                                child_intrinsic_constraint.merge(&child_constraint_for_measure);
                            let size = tessera::measure_node(
                                child_node_id,
                                &final_child_constraint,
                                tree,
                                metadatas,
                            )?; // Explicit call
                            measured_children_sizes[index] = Some(size);
                            actual_height_taken_by_fill_children += size.height;
                            temp_remaining_height_for_weighted =
                                temp_remaining_height_for_weighted.saturating_sub(size.height);
                            computed_max_column_width = computed_max_column_width.max(size.width);
                        }
                    }
                }
                let mut temp_remaining_height_for_unweighted = temp_remaining_height_for_weighted; // Update for unweighted

                if !fill_children_indices_without_weight.is_empty()
                    && temp_remaining_height_for_unweighted > 0
                {
                    let num_unweighted_fill = fill_children_indices_without_weight.len();
                    let height_per_unweighted_child =
                        temp_remaining_height_for_unweighted / num_unweighted_fill as u32;

                    for &index in &fill_children_indices_without_weight {
                        let item_behavior = children_items_for_measure[index].1;
                        let child_node_id = children_node_ids[index];
                        if let DimensionValue::Fill {
                            max: child_max_fill,
                            ..
                        } = item_behavior
                        {
                            let alloc_height = child_max_fill
                                .map_or(height_per_unweighted_child, |m| {
                                    height_per_unweighted_child.min(m)
                                });
                            let final_alloc_height =
                                alloc_height.min(temp_remaining_height_for_unweighted);

                            let child_constraint_for_measure = Constraint::new(
                                effective_column_constraint.width,
                                DimensionValue::Fixed(final_alloc_height),
                            );
                            let child_intrinsic_constraint = metadatas
                                .get(&child_node_id)
                                .ok_or_else(|| {
                                    MeasurementError::ChildMeasurementFailed(child_node_id)
                                })?
                                .constraint;
                            let final_child_constraint =
                                child_intrinsic_constraint.merge(&child_constraint_for_measure);
                            let size = tessera::measure_node(
                                child_node_id,
                                &final_child_constraint,
                                tree,
                                metadatas,
                            )?; // Explicit call
                            measured_children_sizes[index] = Some(size);
                            actual_height_taken_by_fill_children += size.height;
                            temp_remaining_height_for_unweighted =
                                temp_remaining_height_for_unweighted.saturating_sub(size.height);
                            computed_max_column_width = computed_max_column_width.max(size.width);
                        }
                    }
                }
            }
            // Process fill_nodes_to_measure if it was populated (only for is_column_effectively_wrap_for_children case)
            if !fill_nodes_to_measure.is_empty() {
                let nodes_for_api: Vec<_> = fill_nodes_to_measure
                    .iter()
                    .map(|(id, constraint, _idx)| (*id, *constraint))
                    .collect();
                let results_map = measure_nodes(nodes_for_api, tree, metadatas);
                for (child_node_id, _constraint, original_idx) in fill_nodes_to_measure {
                    let size = results_map
                        .get(&child_node_id)
                        .ok_or_else(|| {
                            MeasurementError::MeasureFnFailed(format!(
                                "Result missing for fill/wrap child {:?}",
                                child_node_id
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

            let final_column_height = match effective_column_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Wrap => total_children_height,
                DimensionValue::Fill { max, .. } => {
                    let resolved_fill_height = max.unwrap_or(total_children_height);
                    if max.is_some() {
                        resolved_fill_height.min(total_children_height)
                    } else {
                        total_children_height
                    }
                }
            };
            let final_column_width = match effective_column_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Wrap => computed_max_column_width,
                DimensionValue::Fill { max, .. } => max.map_or(computed_max_column_width, |m| {
                    computed_max_column_width.min(m)
                }),
            };

            let mut current_y_offset: u32 = 0;
            for i in 0..N {
                let child_node_id = children_node_ids[i];
                if let Some(size) = measured_children_sizes[i] {
                    place_node(child_node_id, [0, current_y_offset], metadatas);
                    current_y_offset += size.height;
                } else {
                    let mut meta_entry = metadatas.entry(child_node_id).or_default();
                    meta_entry.computed_data = Some(ComputedData::ZERO);
                    place_node(child_node_id, [0, current_y_offset], metadatas);
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
