//! Layout policies and measurement.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    sync::Arc,
};

use parking_lot::RwLock;

use crate::{
    ComputeResourceManager, ComputedData, Constraint, MeasurementError, ParentConstraint, Px,
    RenderSlot,
    component_tree::{
        ComponentNodeMetaDatas, ComponentNodeTree, LayoutContext, measure_node, measure_nodes,
    },
    modifier::{Modifier, ParentDataMap},
    prop::Prop,
    px::PxPosition,
    render_graph::RenderFragment,
    runtime::TesseraRuntime,
    tessera,
};

#[derive(Clone, Copy)]
pub(crate) struct ChildMeasure {
    pub constraint: Constraint,
    pub size: ComputedData,
    pub consistent: bool,
}

/// Input for a pure layout policy.
pub struct LayoutInput<'a> {
    tree: &'a ComponentNodeTree,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
    metadatas: &'a ComponentNodeMetaDatas,
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: RefCell<HashMap<crate::NodeId, ChildMeasure>>,
}

impl<'a> LayoutInput<'a> {
    pub(crate) fn new(
        tree: &'a ComponentNodeTree,
        parent_constraint: ParentConstraint<'a>,
        children_ids: &'a [crate::NodeId],
        metadatas: &'a ComponentNodeMetaDatas,
        layout_ctx: Option<&'a LayoutContext<'a>>,
    ) -> Self {
        Self {
            tree,
            parent_constraint,
            children_ids,
            metadatas,
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

    pub(crate) fn extend_measured_children(&self, children: HashMap<crate::NodeId, ChildMeasure>) {
        let mut measured_children = self.measured_children.borrow_mut();
        for (child_id, measurement) in children {
            if let Some(entry) = measured_children.get_mut(&child_id) {
                let consistent = entry.consistent
                    && entry.constraint == measurement.constraint
                    && entry.size == measurement.size
                    && measurement.consistent;
                entry.constraint = measurement.constraint;
                entry.size = measurement.size;
                entry.consistent = consistent;
            } else {
                measured_children.insert(child_id, measurement);
            }
        }
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
        let results = measure_nodes(nodes_to_measure, self.tree, self.metadatas, self.layout_ctx);

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
        let results = measure_nodes(nodes_to_measure, self.tree, self.metadatas, self.layout_ctx);

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

    /// Reads a typed parent-data payload from an immediate child node.
    pub fn child_parent_data<T>(&self, child_id: crate::NodeId) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let node = self.tree.get(child_id)?;
        let mut data: ParentDataMap = HashMap::default();
        node.get().modifier.apply_parent_data(&mut data);
        let value = data.get(&TypeId::of::<T>())?;
        value.downcast_ref::<T>().cloned()
    }
}

/// Input for a placement-only pass that reuses cached child measurements.
pub struct PlacementInput<'a> {
    tree: &'a ComponentNodeTree,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
    child_sizes: &'a HashMap<crate::NodeId, ComputedData>,
    size: ComputedData,
}

impl<'a> PlacementInput<'a> {
    pub(crate) fn new(
        tree: &'a ComponentNodeTree,
        parent_constraint: ParentConstraint<'a>,
        children_ids: &'a [crate::NodeId],
        child_sizes: &'a HashMap<crate::NodeId, ComputedData>,
        size: ComputedData,
    ) -> Self {
        Self {
            tree,
            parent_constraint,
            children_ids,
            child_sizes,
            size,
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

    /// Returns the measured size of a direct child from the cached measurement
    /// pass.
    pub fn child_size(&self, child_id: crate::NodeId) -> Option<ComputedData> {
        self.child_sizes.get(&child_id).copied()
    }

    /// Returns the cached size of the current node.
    pub const fn size(&self) -> ComputedData {
        self.size
    }

    /// Reads a typed parent-data payload from an immediate child node.
    pub fn child_parent_data<T>(&self, child_id: crate::NodeId) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let node = self.tree.get(child_id)?;
        let mut data: ParentDataMap = HashMap::default();
        node.get().modifier.apply_parent_data(&mut data);
        let value = data.get(&TypeId::of::<T>())?;
        value.downcast_ref::<T>().cloned()
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

    pub(crate) fn place_instance_key(&mut self, instance_key: u64, position: PxPosition) {
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

    /// Returns a mutable render metadata handle for the current node.
    pub fn metadata_mut(&self) -> RenderMetadataMut<'_> {
        let metadata = self
            .metadatas
            .get_mut(&self.current_node_id)
            .expect("Metadata for current node must exist during record");
        RenderMetadataMut { metadata }
    }
}

/// Mutable render metadata available during the record pass.
pub struct RenderMetadataMut<'a> {
    metadata: dashmap::mapref::one::RefMut<
        'a,
        crate::NodeId,
        crate::component_tree::ComponentNodeMetaData,
    >,
}

impl RenderMetadataMut<'_> {
    /// Returns the computed size of the current node.
    pub fn computed_data(&self) -> Option<ComputedData> {
        self.metadata.computed_data
    }

    /// Returns the render fragment for the current node.
    pub fn fragment_mut(&mut self) -> &mut RenderFragment {
        self.metadata.fragment_mut()
    }

    /// Enables or disables child clipping for the current node.
    pub fn set_clips_children(&mut self, clips_children: bool) {
        self.metadata.clips_children = clips_children;
    }

    /// Multiplies the current node opacity by the provided factor.
    pub fn multiply_opacity(&mut self, opacity: f32) {
        self.metadata.opacity *= opacity;
    }
}

/// Pure layout policy for measuring and placing child nodes.
pub trait LayoutPolicy: Send + Sync + Clone + PartialEq + 'static {
    /// Computes layout for the current node.
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError>;

    /// Compares measurement-relevant state for layout dirty tracking.
    fn measure_eq(&self, other: &Self) -> bool {
        self == other
    }

    /// Compares placement-relevant state for layout dirty tracking.
    fn placement_eq(&self, other: &Self) -> bool {
        self == other
    }

    /// Recomputes child placements using cached child measurements.
    ///
    /// Returns `true` when the placement pass was handled without
    /// remeasurement.
    fn place_children(&self, _input: &PlacementInput<'_>, _output: &mut LayoutOutput<'_>) -> bool {
        false
    }
}

/// Render policy for recording draw and compute commands for the current node.
pub trait RenderPolicy: Send + Sync + Clone + PartialEq + 'static {
    /// Records draw and compute commands for the current node.
    fn record(&self, _input: &RenderInput<'_>) {}
}

/// Type-erased layout policy used by the runtime.
#[doc(hidden)]
pub trait LayoutPolicyDyn: Send + Sync {
    /// Returns a typed reference for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Measures layout using a type-erased policy.
    fn measure_dyn(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError>;
    /// Recomputes child placements using cached child measurements.
    fn place_children_dyn(&self, input: &PlacementInput<'_>, output: &mut LayoutOutput<'_>)
    -> bool;
    /// Compares two type-erased policies for equality.
    fn dyn_eq(&self, other: &dyn LayoutPolicyDyn) -> bool;
    /// Compares two type-erased policies for measurement-relevant equality.
    fn dyn_measure_eq(&self, other: &dyn LayoutPolicyDyn) -> bool;
    /// Compares two type-erased policies for placement-relevant equality.
    fn dyn_placement_eq(&self, other: &dyn LayoutPolicyDyn) -> bool;
    /// Clones the type-erased policy.
    fn clone_box(&self) -> Box<dyn LayoutPolicyDyn>;
}

impl<T> LayoutPolicyDyn for T
where
    T: LayoutPolicy,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn measure_dyn(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        LayoutPolicy::measure(self, input, output)
    }

    fn place_children_dyn(
        &self,
        input: &PlacementInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> bool {
        LayoutPolicy::place_children(self, input, output)
    }

    fn dyn_eq(&self, other: &dyn LayoutPolicyDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| self == other)
    }

    fn dyn_measure_eq(&self, other: &dyn LayoutPolicyDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| LayoutPolicy::measure_eq(self, other))
    }

    fn dyn_placement_eq(&self, other: &dyn LayoutPolicyDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| LayoutPolicy::placement_eq(self, other))
    }

    fn clone_box(&self) -> Box<dyn LayoutPolicyDyn> {
        Box::new(self.clone())
    }
}

/// Type-erased render policy used by the runtime.
#[doc(hidden)]
pub trait RenderPolicyDyn: Send + Sync {
    /// Returns a typed reference for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Records render commands using a type-erased policy.
    fn record_dyn(&self, input: &RenderInput<'_>);
    /// Compares two type-erased policies for equality.
    fn dyn_eq(&self, other: &dyn RenderPolicyDyn) -> bool;
    /// Clones the type-erased policy.
    fn clone_box(&self) -> Box<dyn RenderPolicyDyn>;
}

impl<T> RenderPolicyDyn for T
where
    T: RenderPolicy,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn record_dyn(&self, input: &RenderInput<'_>) {
        RenderPolicy::record(self, input);
    }

    fn dyn_eq(&self, other: &dyn RenderPolicyDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| self == other)
    }

    fn clone_box(&self) -> Box<dyn RenderPolicyDyn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn LayoutPolicyDyn> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Clone for Box<dyn RenderPolicyDyn> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Type-erased layout policy handle consumed by [`layout_primitive`].
///
/// Most callers do not construct this type directly. Instead, pass any
/// [`LayoutPolicy`] to the `layout_policy(...)` builder method and rely on the
/// `From<T>` conversion.
#[derive(Clone)]
pub struct LayoutPolicyHandle {
    policy: Box<dyn LayoutPolicyDyn>,
}

impl LayoutPolicyHandle {
    pub(crate) fn into_box(self) -> Box<dyn LayoutPolicyDyn> {
        self.policy
    }
}

impl Default for LayoutPolicyHandle {
    fn default() -> Self {
        Self {
            policy: Box::new(DefaultLayoutPolicy),
        }
    }
}

impl PartialEq for LayoutPolicyHandle {
    fn eq(&self, other: &Self) -> bool {
        self.policy.dyn_eq(other.policy.as_ref())
    }
}

impl Prop for LayoutPolicyHandle {
    fn prop_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<S> From<S> for LayoutPolicyHandle
where
    S: LayoutPolicy,
{
    fn from(policy: S) -> Self {
        Self {
            policy: Box::new(policy),
        }
    }
}

/// Type-erased render policy handle consumed by [`layout_primitive`].
///
/// Most callers do not construct this type directly. Instead, pass any
/// [`RenderPolicy`] to the `render_policy(...)` builder method and rely on the
/// `From<T>` conversion.
#[derive(Clone)]
pub struct RenderPolicyHandle {
    policy: Box<dyn RenderPolicyDyn>,
}

impl RenderPolicyHandle {
    pub(crate) fn into_box(self) -> Box<dyn RenderPolicyDyn> {
        self.policy
    }
}

impl Default for RenderPolicyHandle {
    fn default() -> Self {
        Self {
            policy: Box::new(NoopRenderPolicy),
        }
    }
}

impl PartialEq for RenderPolicyHandle {
    fn eq(&self, other: &Self) -> bool {
        self.policy.dyn_eq(other.policy.as_ref())
    }
}

impl Prop for RenderPolicyHandle {
    fn prop_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<S> From<S> for RenderPolicyHandle
where
    S: RenderPolicy,
{
    fn from(policy: S) -> Self {
        Self {
            policy: Box::new(policy),
        }
    }
}

/// # layout_primitive
///
/// Attach a layout policy, render policy, modifier chain, and optional child
/// slot to the current component node.
///
/// ## Usage
///
/// Build framework or internal components that need to define custom node
/// layout and rendering behavior.
///
/// ## Parameters
///
/// - `layout_policy` - pure layout policy used for measuring and placing the
///   current node
/// - `render_policy` - render policy used to record draw and compute commands
///   for the current node
/// - `modifier` - node-local modifier chain attached before child content is
///   emitted
/// - `child` - optional child slot rendered as this node's content subtree
///
/// ## Examples
/// ```
/// use tessera_ui::{
///     Modifier, NoopRenderPolicy, RenderSlot,
///     layout::{DefaultLayoutPolicy, layout_primitive},
/// };
///
/// #[tessera_ui::tessera]
/// fn primitive_example(modifier: Modifier, child: RenderSlot) {
///     layout_primitive()
///         .layout_policy(DefaultLayoutPolicy)
///         .render_policy(NoopRenderPolicy)
///         .modifier(modifier)
///         .child(move || child.render());
/// }
/// ```
#[tessera(crate)]
pub fn layout_primitive(
    #[prop(into)] layout_policy: LayoutPolicyHandle,
    #[prop(into)] render_policy: RenderPolicyHandle,
    modifier: Modifier,
    child: Option<RenderSlot>,
) {
    modifier.attach();
    TesseraRuntime::with_mut(|runtime| {
        runtime.set_current_layout_policy_boxed(layout_policy.into_box());
        runtime.set_current_render_policy_boxed(render_policy.into_box());
    });
    if let Some(child) = child {
        child.render();
    }
}

/// Default layout policy that stacks children at (0,0) and uses the bounding
/// size.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DefaultLayoutPolicy;

impl LayoutPolicy for DefaultLayoutPolicy {
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

/// Default render policy that emits no draw commands.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoopRenderPolicy;

impl RenderPolicy for NoopRenderPolicy {}
