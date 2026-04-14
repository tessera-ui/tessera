//! Layout policies and measurement.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    ComputeResourceManager, ComputedData, Constraint, MeasurementError, ParentConstraint, Px,
    RenderSlot,
    component_tree::{
        ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, LayoutContext,
        measure_node,
    },
    modifier::{Modifier, OrderedModifierAction, ParentDataMap},
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
    metadatas: *mut ComponentNodeMetaDatas,
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: RefCell<HashMap<crate::NodeId, ChildMeasure>>,
}

/// A direct child layout node available during a measure pass.
#[derive(Clone, Copy)]
pub struct LayoutChild<'a> {
    node_id: crate::NodeId,
    instance_key: u64,
    tree: &'a ComponentNodeTree,
    metadatas: *mut ComponentNodeMetaDatas,
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: &'a RefCell<HashMap<crate::NodeId, ChildMeasure>>,
}

impl PartialEq for LayoutChild<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

impl Eq for LayoutChild<'_> {}

impl Hash for LayoutChild<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
    }
}

impl<'a> LayoutChild<'a> {
    /// Measures this child under the given constraint.
    pub fn measure(&self, constraint: &Constraint) -> Result<MeasuredChild, MeasurementError> {
        // SAFETY: Layout measurement is single-threaded. `LayoutChild` handles
        // are created from a unique metadata borrow that outlives the measure
        // pass and are only used during that pass.
        let metadatas = unsafe { &mut *self.metadatas };
        let size = measure_node(
            self.node_id,
            constraint,
            self.tree,
            metadatas,
            self.layout_ctx,
        )?;
        let mut measured_children = self.measured_children.borrow_mut();
        if let Some(entry) = measured_children.get_mut(&self.node_id) {
            let consistent =
                entry.consistent && entry.constraint == *constraint && entry.size == size;
            entry.constraint = *constraint;
            entry.size = size;
            entry.consistent = consistent;
        } else {
            measured_children.insert(
                self.node_id,
                ChildMeasure {
                    constraint: *constraint,
                    size,
                    consistent: true,
                },
            );
        }
        Ok(MeasuredChild {
            instance_key: self.instance_key,
            width: size.width,
            height: size.height,
        })
    }

    /// Measures this child without recording it for layout cache keys.
    pub fn measure_untracked(
        &self,
        constraint: &Constraint,
    ) -> Result<MeasuredChild, MeasurementError> {
        // SAFETY: See `LayoutChild::measure`.
        let metadatas = unsafe { &mut *self.metadatas };
        let size = measure_node(
            self.node_id,
            constraint,
            self.tree,
            metadatas,
            self.layout_ctx,
        )?;
        Ok(MeasuredChild {
            instance_key: self.instance_key,
            width: size.width,
            height: size.height,
        })
    }

    /// Reads a typed parent-data payload from this direct child layout node.
    pub fn parent_data<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let node = self.tree.get(self.node_id)?;
        let mut data: ParentDataMap = HashMap::default();
        for action in node.get().modifier.ordered_actions() {
            if let OrderedModifierAction::ParentData(node) = action {
                node.apply_parent_data(&mut data);
            }
        }
        let value = data.get(&TypeId::of::<T>())?;
        value.downcast_ref::<T>().cloned()
    }

    pub(crate) const fn instance_key(&self) -> u64 {
        self.instance_key
    }
}

/// A measured child returned from [`LayoutChild::measure`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasuredChild {
    instance_key: u64,
    /// The measured width of the child.
    pub width: Px,
    /// The measured height of the child.
    pub height: Px,
}

impl MeasuredChild {
    /// Returns the measured size for this child.
    pub const fn size(&self) -> ComputedData {
        ComputedData {
            width: self.width,
            height: self.height,
        }
    }

    pub(crate) const fn instance_key(&self) -> u64 {
        self.instance_key
    }
}

/// A direct child available during a placement-only pass.
#[derive(Clone, Copy)]
pub struct PlacementChild<'a> {
    node_id: crate::NodeId,
    instance_key: u64,
    tree: &'a ComponentNodeTree,
    size: ComputedData,
}

impl<'a> PlacementChild<'a> {
    /// Returns the cached measured size from the preceding measure pass.
    pub const fn size(&self) -> ComputedData {
        self.size
    }

    /// Reads a typed parent-data payload from the child.
    pub fn parent_data<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let node = self.tree.get(self.node_id)?;
        let mut data: ParentDataMap = HashMap::default();
        for action in node.get().modifier.ordered_actions() {
            if let OrderedModifierAction::ParentData(node) = action {
                node.apply_parent_data(&mut data);
            }
        }
        let value = data.get(&TypeId::of::<T>())?;
        value.downcast_ref::<T>().cloned()
    }

    pub(crate) const fn instance_key(&self) -> u64 {
        self.instance_key
    }
}

/// Shared measurement helpers available during a layout pass.
pub struct MeasureScope<'a> {
    tree: &'a ComponentNodeTree,
    metadatas: *mut ComponentNodeMetaDatas,
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: &'a RefCell<HashMap<crate::NodeId, ChildMeasure>>,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
}

impl<'a> MeasureScope<'a> {
    pub(crate) fn new(
        tree: &'a ComponentNodeTree,
        metadatas: *mut ComponentNodeMetaDatas,
        layout_ctx: Option<&'a LayoutContext<'a>>,
        measured_children: &'a RefCell<HashMap<crate::NodeId, ChildMeasure>>,
        parent_constraint: ParentConstraint<'a>,
        children_ids: &'a [crate::NodeId],
    ) -> Self {
        Self {
            tree,
            metadatas,
            layout_ctx,
            measured_children,
            parent_constraint,
            children_ids,
        }
    }

    /// Returns the inherited constraint.
    pub const fn parent_constraint(&self) -> ParentConstraint<'a> {
        self.parent_constraint
    }

    /// Returns the direct child layout nodes of the current node.
    pub fn children(&self) -> Vec<LayoutChild<'_>> {
        self.children_ids
            .iter()
            .map(|&node_id| {
                let instance_key = self
                    .tree
                    .get(node_id)
                    .expect("Direct child layout node must exist")
                    .get()
                    .instance_key;
                LayoutChild {
                    node_id,
                    instance_key,
                    tree: self.tree,
                    metadatas: self.metadatas,
                    layout_ctx: self.layout_ctx,
                    measured_children: self.measured_children,
                }
            })
            .collect()
    }
}

/// Shared placement helpers available during a cached placement pass.
pub struct PlacementScope<'a> {
    tree: &'a ComponentNodeTree,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
    child_sizes: &'a HashMap<crate::NodeId, ComputedData>,
    size: ComputedData,
}

impl<'a> PlacementScope<'a> {
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

    /// Returns the direct child layout nodes of the current node.
    pub fn children(&self) -> Vec<PlacementChild<'_>> {
        self.children_ids
            .iter()
            .map(|&node_id| {
                let node = self
                    .tree
                    .get(node_id)
                    .expect("Direct child layout node must exist");
                PlacementChild {
                    node_id,
                    instance_key: node.get().instance_key,
                    tree: self.tree,
                    size: self
                        .child_sizes
                        .get(&node_id)
                        .copied()
                        .expect("Placement child size must exist for a direct child layout node"),
                }
            })
            .collect()
    }

    /// Returns the cached size of the current node.
    pub const fn size(&self) -> ComputedData {
        self.size
    }
}

impl<'a> LayoutInput<'a> {
    pub(crate) fn new(
        tree: &'a ComponentNodeTree,
        parent_constraint: ParentConstraint<'a>,
        children_ids: &'a [crate::NodeId],
        metadatas: *mut ComponentNodeMetaDatas,
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

    pub(crate) fn measure_scope(&self) -> MeasureScope<'_> {
        MeasureScope::new(
            self.tree,
            self.metadatas,
            self.layout_ctx,
            &self.measured_children,
            self.parent_constraint,
            self.children_ids,
        )
    }

    /// Returns the direct child layout nodes of the current node.
    pub fn children(&self) -> Vec<LayoutChild<'_>> {
        self.children_ids
            .iter()
            .map(|&node_id| {
                let instance_key = self
                    .tree
                    .get(node_id)
                    .expect("Direct child layout node must exist")
                    .get()
                    .instance_key;
                LayoutChild {
                    node_id,
                    instance_key,
                    tree: self.tree,
                    metadatas: self.metadatas,
                    layout_ctx: self.layout_ctx,
                    measured_children: &self.measured_children,
                }
            })
            .collect()
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

impl LayoutResult {
    /// Creates a layout result for the current node.
    pub const fn new(size: ComputedData) -> Self {
        Self {
            size,
            placements: Vec::new(),
        }
    }

    /// Places a child relative to the current node origin.
    pub fn place_child<T>(&mut self, child: T, position: PxPosition)
    where
        T: LayoutPlacementTarget,
    {
        self.placements.push((child.instance_key(), position));
    }

    /// Replaces the computed size while preserving recorded placements.
    pub const fn with_size(mut self, size: ComputedData) -> Self {
        self.size = size;
        self
    }

    /// Consumes the result and returns only the recorded placements.
    pub fn into_placements(self) -> Vec<(u64, PxPosition)> {
        self.placements
    }
}

impl Default for LayoutResult {
    fn default() -> Self {
        Self::new(ComputedData::ZERO)
    }
}

/// A direct child handle that can be placed by [`LayoutResult`].
pub trait LayoutPlacementTarget: Copy {
    #[doc(hidden)]
    fn instance_key(&self) -> u64;
}

impl LayoutPlacementTarget for LayoutChild<'_> {
    fn instance_key(&self) -> u64 {
        self.instance_key()
    }
}

impl LayoutPlacementTarget for MeasuredChild {
    fn instance_key(&self) -> u64 {
        self.instance_key()
    }
}

impl LayoutPlacementTarget for PlacementChild<'_> {
    fn instance_key(&self) -> u64 {
        self.instance_key()
    }
}

/// Input for a render record pass.
pub struct RenderInput<'a> {
    current_node_id: crate::NodeId,
    metadatas: &'a mut ComponentNodeMetaDatas,
    /// Mutable GPU compute resources for the current frame.
    pub compute_resource_manager: &'a mut ComputeResourceManager,
    /// GPU device for issuing render-side allocations.
    pub gpu: &'a wgpu::Device,
}

impl<'a> RenderInput<'a> {
    pub(crate) fn new(
        current_node_id: crate::NodeId,
        metadatas: &'a mut ComponentNodeMetaDatas,
        compute_resource_manager: &'a mut ComputeResourceManager,
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
    pub fn metadata_mut(&mut self) -> RenderMetadataMut<'_> {
        let metadata = self
            .metadatas
            .get_mut(&self.current_node_id)
            .expect("Metadata for current node must exist during record");
        RenderMetadataMut { metadata }
    }
}

/// Mutable render metadata available during the record pass.
pub struct RenderMetadataMut<'a> {
    metadata: &'a mut ComponentNodeMetaData,
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
    fn measure(&self, scope: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError>;

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
    /// Returns placements when the placement pass was handled without
    /// remeasurement.
    fn place_children(&self, _scope: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>> {
        None
    }
}

/// Render policy for recording draw and compute commands for the current node.
pub trait RenderPolicy: Send + Sync + Clone + PartialEq + 'static {
    /// Records draw and compute commands for the current node.
    fn record(&self, _input: &mut RenderInput<'_>) {}
}

/// Type-erased layout policy used by the runtime.
#[doc(hidden)]
pub trait LayoutPolicyDyn: Send + Sync {
    /// Returns a typed reference for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Measures layout using a type-erased policy.
    fn measure_dyn(&self, scope: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError>;
    /// Recomputes child placements using cached child measurements.
    fn place_children_dyn(&self, scope: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>>;
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

    fn measure_dyn(&self, scope: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        LayoutPolicy::measure(self, scope)
    }

    fn place_children_dyn(&self, scope: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>> {
        LayoutPolicy::place_children(self, scope)
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
    fn record_dyn(&self, input: &mut RenderInput<'_>);
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

    fn record_dyn(&self, input: &mut RenderInput<'_>) {
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

/// Type-erased layout policy handle consumed by [`layout`].
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

/// Type-erased render policy handle consumed by [`layout`].
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

/// # layout
///
/// Emit an explicit layout node with a layout policy, render policy, modifier
/// chain, and optional child slot.
///
/// ## Usage
///
/// Build framework or internal components that need to define explicit layout
/// and rendering behavior inside a tessera composition boundary.
///
/// ## Parameters
///
/// - `layout_policy` - pure layout policy used for measuring and placing the
///   emitted layout node, defaulting to [`DefaultLayoutPolicy`]
/// - `render_policy` - render policy used to record draw and compute commands
///   for the emitted layout node, defaulting to [`NoopRenderPolicy`]
/// - `modifier` - node-local modifier chain attached before child content is
///   emitted, defaulting to an empty [`Modifier`]
/// - `child` - optional child slot rendered as the emitted node's content
///   subtree
///
/// ## Examples
/// ```
/// use tessera_ui::{
///     Modifier, NoopRenderPolicy, RenderSlot,
///     layout::{DefaultLayoutPolicy, layout},
/// };
///
/// #[tessera_ui::tessera]
/// fn primitive_example(modifier: Modifier, child: RenderSlot) {
///     layout()
///         .layout_policy(DefaultLayoutPolicy)
///         .render_policy(NoopRenderPolicy)
///         .modifier(modifier)
///         .child(move || child.render());
/// }
/// ```
#[tessera(crate)]
pub fn layout(
    #[prop(into)] layout_policy: Option<LayoutPolicyHandle>,
    #[prop(into)] render_policy: Option<RenderPolicyHandle>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let layout_policy = layout_policy.unwrap_or(LayoutPolicyHandle::from(DefaultLayoutPolicy));
    let render_policy = render_policy.unwrap_or(RenderPolicyHandle::from(NoopRenderPolicy));
    let modifier = modifier.unwrap_or_default();
    let layout_node_component_type_id = layout_node_component_type_id();
    let layout_node_id =
        crate::__private::register_layout_node("layout", layout_node_component_type_id);
    let _layout_node_ctx_guard = crate::__private::push_current_node(
        layout_node_id,
        layout_node_component_type_id,
        "layout",
    );
    let layout_instance_key = crate::__private::current_instance_key();
    let layout_instance_logic_id = crate::__private::current_instance_logic_id();
    let _layout_scope_guard = {
        struct LayoutNodeScopeGuard;

        impl Drop for LayoutNodeScopeGuard {
            fn drop(&mut self) {
                crate::__private::finish_component_node();
            }
        }

        LayoutNodeScopeGuard
    };

    crate::__private::set_current_node_identity(layout_instance_key, layout_instance_logic_id);
    modifier.attach();
    TesseraRuntime::with_mut(|runtime| {
        runtime.set_current_layout_policy_boxed(layout_policy.into_box());
        runtime.set_current_render_policy_boxed(render_policy.into_box());
    });
    if let Some(child) = child {
        child.render();
    }
}

fn layout_node_component_type_id() -> u64 {
    let mut hasher = DefaultHasher::new();
    "layout_node".hash(&mut hasher);
    hasher.finish()
}

/// Default layout policy that stacks children at (0,0) and uses the bounding
/// size.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DefaultLayoutPolicy;

impl LayoutPolicy for DefaultLayoutPolicy {
    fn measure(&self, scope: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = scope.children();
        if children.is_empty() {
            return Ok(LayoutResult::new(ComputedData::min_from_constraint(
                scope.parent_constraint().as_ref(),
            )));
        }

        let mut result = LayoutResult::new(ComputedData::ZERO);
        let mut final_width = Px(0);
        let mut final_height = Px(0);
        for child in children {
            let measurement = child.measure(scope.parent_constraint().as_ref())?;
            result.place_child(measurement, PxPosition::ZERO);
            let size = measurement.size();
            final_width = final_width.max(size.width);
            final_height = final_height.max(size.height);
        }

        result.size = ComputedData {
            width: final_width,
            height: final_height,
        };
        Ok(result)
    }
}

/// Default render policy that emits no draw commands.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoopRenderPolicy;

impl RenderPolicy for NoopRenderPolicy {}
