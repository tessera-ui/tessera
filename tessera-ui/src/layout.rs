//! Pure layout specs and record hooks for components.
//!
//! ## Usage
//!
//! Define layout specs for containers and record draw commands in a render
//! pass.

use std::{any::Any, cell::RefCell, collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::{
    ComputeResourceManager, ComputedData, Constraint, MeasurementError, ParentConstraint, Px,
    component_tree::{
        ComponentNodeMetaDatas, ComponentNodeTree, LayoutContext, measure_node, measure_nodes,
    },
    px::PxPosition,
};

#[derive(Clone, Copy)]
pub(crate) struct ChildMeasure {
    pub constraint: Constraint,
    pub size: ComputedData,
    pub consistent: bool,
}

/// Input for a pure layout spec.
pub struct LayoutInput<'a> {
    tree: &'a ComponentNodeTree,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
    metadatas: &'a ComponentNodeMetaDatas,
    compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &'a wgpu::Device,
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: RefCell<HashMap<crate::NodeId, ChildMeasure>>,
}

impl<'a> LayoutInput<'a> {
    pub(crate) fn new(
        tree: &'a ComponentNodeTree,
        parent_constraint: ParentConstraint<'a>,
        children_ids: &'a [crate::NodeId],
        metadatas: &'a ComponentNodeMetaDatas,
        compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
        gpu: &'a wgpu::Device,
        layout_ctx: Option<&'a LayoutContext<'a>>,
    ) -> Self {
        Self {
            tree,
            parent_constraint,
            children_ids,
            metadatas,
            compute_resource_manager,
            gpu,
            layout_ctx,
            measured_children: RefCell::new(HashMap::new()),
        }
    }

    /// Returns the inherited constraint.
    pub const fn parent_constraint(&self) -> ParentConstraint<'a> {
        self.parent_constraint
    }

    /// Returns the children node ids of the current node.
    pub fn children_ids(&self) -> &'a [crate::NodeId] {
        self.children_ids
    }

    pub(crate) fn take_measured_children(&self) -> HashMap<crate::NodeId, ChildMeasure> {
        std::mem::take(&mut *self.measured_children.borrow_mut())
    }

    fn record_child_measure(
        &self,
        child_id: crate::NodeId,
        constraint: Constraint,
        size: ComputedData,
    ) {
        let mut measured_children = self.measured_children.borrow_mut();
        if let Some(entry) = measured_children.get_mut(&child_id) {
            let consistent =
                entry.consistent && entry.constraint == constraint && entry.size == size;
            entry.constraint = constraint;
            entry.size = size;
            entry.consistent = consistent;
        } else {
            measured_children.insert(
                child_id,
                ChildMeasure {
                    constraint,
                    size,
                    consistent: true,
                },
            );
        }
    }

    /// Measures all specified child nodes under the given constraint.
    pub fn measure_children(
        &self,
        nodes_to_measure: Vec<(crate::NodeId, Constraint)>,
    ) -> Result<HashMap<crate::NodeId, ComputedData>, MeasurementError> {
        let constraints: HashMap<crate::NodeId, Constraint> = nodes_to_measure
            .iter()
            .map(|(child_id, constraint)| (*child_id, *constraint))
            .collect();
        let results = measure_nodes(
            nodes_to_measure,
            self.tree,
            self.metadatas,
            self.compute_resource_manager.clone(),
            self.gpu,
            self.layout_ctx,
        );

        let mut successful_results = HashMap::new();
        for (child_id, result) in results {
            match result {
                Ok(size) => {
                    if let Some(constraint) = constraints.get(&child_id) {
                        self.record_child_measure(child_id, *constraint, size);
                    }
                    successful_results.insert(child_id, size);
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Ok(successful_results)
    }

    /// Measures child nodes without recording them for layout cache keys.
    pub fn measure_children_untracked(
        &self,
        nodes_to_measure: Vec<(crate::NodeId, Constraint)>,
    ) -> Result<HashMap<crate::NodeId, ComputedData>, MeasurementError> {
        let results = measure_nodes(
            nodes_to_measure,
            self.tree,
            self.metadatas,
            self.compute_resource_manager.clone(),
            self.gpu,
            self.layout_ctx,
        );

        let mut successful_results = HashMap::new();
        for (child_id, result) in results {
            match result {
                Ok(size) => {
                    successful_results.insert(child_id, size);
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Ok(successful_results)
    }

    /// Measures a single child node under the given constraint.
    pub fn measure_child(
        &self,
        child_id: crate::NodeId,
        constraint: &Constraint,
    ) -> Result<ComputedData, MeasurementError> {
        let size = measure_node(
            child_id,
            constraint,
            self.tree,
            self.metadatas,
            self.compute_resource_manager.clone(),
            self.gpu,
            self.layout_ctx,
        )?;
        self.record_child_measure(child_id, *constraint, size);
        Ok(size)
    }

    /// Measures a child node without recording it for layout cache keys.
    pub fn measure_child_untracked(
        &self,
        child_id: crate::NodeId,
        constraint: &Constraint,
    ) -> Result<ComputedData, MeasurementError> {
        measure_node(
            child_id,
            constraint,
            self.tree,
            self.metadatas,
            self.compute_resource_manager.clone(),
            self.gpu,
            self.layout_ctx,
        )
    }

    /// Measures a single child node using this node's inherited constraint.
    pub fn measure_child_in_parent_constraint(
        &self,
        child_id: crate::NodeId,
    ) -> Result<ComputedData, MeasurementError> {
        self.measure_child(child_id, self.parent_constraint.as_ref())
    }
}

/// Output collected during pure layout.
pub struct LayoutOutput<'a> {
    placements: Vec<(u64, PxPosition)>,
    resolve_instance_key: &'a dyn Fn(crate::NodeId) -> u64,
}

impl<'a> LayoutOutput<'a> {
    pub(crate) fn new(resolve_instance_key: &'a dyn Fn(crate::NodeId) -> u64) -> Self {
        Self {
            placements: Vec::new(),
            resolve_instance_key,
        }
    }

    /// Sets the relative position of a child node.
    pub fn place_child(&mut self, child_id: crate::NodeId, position: PxPosition) {
        let instance_key = (self.resolve_instance_key)(child_id);
        self.placements.push((instance_key, position));
    }

    pub(crate) fn finish(self) -> Vec<(u64, PxPosition)> {
        self.placements
    }
}

/// Cached output from pure layout.
#[derive(Clone)]
pub struct LayoutResult {
    /// The computed size for the node.
    pub size: ComputedData,
    /// Child placements keyed by instance_key.
    pub placements: Vec<(u64, PxPosition)>,
}

/// Input for a render record pass.
pub struct RenderInput<'a> {
    current_node_id: crate::NodeId,
    metadatas: &'a ComponentNodeMetaDatas,
    /// Shared GPU compute resources for the current frame.
    pub compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    /// GPU device for issuing render-side allocations.
    pub gpu: &'a wgpu::Device,
}

impl<'a> RenderInput<'a> {
    pub(crate) fn new(
        current_node_id: crate::NodeId,
        metadatas: &'a ComponentNodeMetaDatas,
        compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
        gpu: &'a wgpu::Device,
    ) -> Self {
        Self {
            current_node_id,
            metadatas,
            compute_resource_manager,
            gpu,
        }
    }

    /// Returns a mutable reference to the metadata of the current node.
    pub fn metadata_mut(
        &self,
    ) -> dashmap::mapref::one::RefMut<'_, crate::NodeId, crate::ComponentNodeMetaData> {
        self.metadatas
            .get_mut(&self.current_node_id)
            .expect("Metadata for current node must exist during record")
    }
}

/// Pure layout spec with an optional render record hook.
pub trait LayoutSpec: Send + Sync + Clone + PartialEq + 'static {
    /// Computes layout for the current node.
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError>;

    /// Records draw/compute commands for the current node.
    fn record(&self, _input: &RenderInput<'_>) {}
}

/// Type-erased layout spec used by the runtime.
#[doc(hidden)]
pub trait LayoutSpecDyn: Send + Sync {
    /// Returns a typed reference for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Measures layout using a type-erased spec.
    fn measure_dyn(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError>;
    /// Records render commands using a type-erased spec.
    fn record_dyn(&self, input: &RenderInput<'_>);
    /// Compares two type-erased specs for equality.
    fn dyn_eq(&self, other: &dyn LayoutSpecDyn) -> bool;
    /// Clones the type-erased spec.
    fn clone_box(&self) -> Box<dyn LayoutSpecDyn>;
}

impl<T> LayoutSpecDyn for T
where
    T: LayoutSpec,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn measure_dyn(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        LayoutSpec::measure(self, input, output)
    }

    fn record_dyn(&self, input: &RenderInput<'_>) {
        LayoutSpec::record(self, input);
    }

    fn dyn_eq(&self, other: &dyn LayoutSpecDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| self == other)
    }

    fn clone_box(&self) -> Box<dyn LayoutSpecDyn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn LayoutSpecDyn> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Default layout spec that stacks children at (0,0) and uses the bounding
/// size.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DefaultLayoutSpec;

impl LayoutSpec for DefaultLayoutSpec {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        if input.children_ids().is_empty() {
            return Ok(ComputedData::min_from_constraint(
                input.parent_constraint().as_ref(),
            ));
        }

        let nodes_to_measure = input
            .children_ids()
            .iter()
            .map(|&child_id| (child_id, *input.parent_constraint().as_ref()))
            .collect();
        let sizes = input.measure_children(nodes_to_measure)?;

        let mut final_width = Px(0);
        let mut final_height = Px(0);
        for (child_id, size) in sizes {
            output.place_child(child_id, PxPosition::ZERO);
            final_width = final_width.max(size.width);
            final_height = final_height.max(size.height);
        }

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}
