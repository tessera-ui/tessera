//! Layout policies and measurement.
//!
//! ## Usage
//!
//! Measure and align content while reserving independent interaction bounds.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    ComputeResourceManager, ComputedData, Constraint, MeasurementError, ParentConstraint, Px,
    RenderSlot,
    component_tree::{
        ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, DrawLayoutBounds,
        LayoutContext, LayoutDiagnosticsCollector, LayoutSnapshotMap, measure_node,
    },
    modifier::{
        DrawModifierContent, DrawModifierContext, DrawModifierNode, LayoutModifierNode, Modifier,
        OrderedModifierAction, ParentDataMap,
    },
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

pub(crate) struct ChildProbe {
    constraint: Constraint,
    size: ComputedData,
}

/// Input for a pure layout policy.
pub struct LayoutInput<'a> {
    tree: &'a ComponentNodeTree,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: HashMap<crate::NodeId, ChildMeasure>,
    probes: HashMap<crate::NodeId, ChildProbe>,
    intrinsic_dependencies: bool,
}

/// A direct child layout node available during a measure pass.
#[derive(Clone, Copy)]
pub struct LayoutChild {
    node_id: crate::NodeId,
    instance_key: u64,
    index: usize,
}

impl PartialEq for LayoutChild {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

impl Eq for LayoutChild {}

impl Hash for LayoutChild {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
    }
}

impl LayoutChild {
    /// Measures this child under the given constraint.
    ///
    /// The child is borrowed by index from the current measure scope so the
    /// metadata map stays uniquely borrowed for the duration of the call.
    pub fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
        constraint: &Constraint,
    ) -> Result<MeasuredChild, MeasurementError> {
        scope.measure_child(self.index, constraint, true)
    }

    /// Probes this child under the given constraint.
    ///
    /// A later probe or measure of the same child under the same constraint
    /// reuses this result and does not call into the child again. A different
    /// constraint is measured normally; only the final [`Self::measure`] is
    /// recorded for the parent snapshot cache.
    pub fn probe(
        &self,
        scope: &mut MeasureScope<'_>,
        constraint: &Constraint,
    ) -> Result<MeasuredChild, MeasurementError> {
        scope.measure_child(self.index, constraint, false)
    }

    /// Probes natural content dimensions without publishing layout or
    /// placements.
    ///
    /// Minimum interactive-size padding is excluded. Other layout constraints
    /// and content padding still apply. Use this to choose shared visual
    /// dimensions before measuring the children normally for interaction.
    /// Results and metadata are isolated from the normal layout cache.
    pub fn probe_intrinsic(
        &self,
        scope: &mut MeasureScope<'_>,
        constraint: &Constraint,
    ) -> Result<MeasuredChild, MeasurementError> {
        *scope.intrinsic_dependencies = true;
        if scope.is_intrinsic() {
            return scope.measure_child(self.index, constraint, false);
        }
        let mut metadatas = ComponentNodeMetaDatas::new();
        let mut snapshots = LayoutSnapshotMap::default();
        let mut diagnostics = LayoutDiagnosticsCollector::default();
        let empty = Default::default();
        let context = LayoutContext::intrinsic_probe(
            &mut snapshots,
            &empty,
            &mut diagnostics,
            scope.layout_ctx,
        );
        let size = measure_node(
            self.node_id,
            constraint,
            scope.tree,
            &mut metadatas,
            Some(&context),
        )?;
        Ok(MeasuredChild {
            instance_key: self.instance_key,
            width: size.width,
            height: size.height,
        })
    }

    /// Reads a typed parent-data payload from this direct child layout node.
    pub fn parent_data<T>(&self, scope: &MeasureScope<'_>) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let node = scope.tree.get(self.node_id)?;
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
    metadatas: &'a mut ComponentNodeMetaDatas,
    layout_ctx: Option<&'a LayoutContext<'a>>,
    measured_children: &'a mut HashMap<crate::NodeId, ChildMeasure>,
    probes: &'a mut HashMap<crate::NodeId, ChildProbe>,
    intrinsic_dependencies: &'a mut bool,
    parent_constraint: ParentConstraint<'a>,
    children_ids: &'a [crate::NodeId],
}

impl<'a> MeasureScope<'a> {
    pub(crate) fn new(
        tree: &'a ComponentNodeTree,
        metadatas: &'a mut ComponentNodeMetaDatas,
        layout_ctx: Option<&'a LayoutContext<'a>>,
        measured_children: &'a mut HashMap<crate::NodeId, ChildMeasure>,
        probes: &'a mut HashMap<crate::NodeId, ChildProbe>,
        intrinsic_dependencies: &'a mut bool,
        parent_constraint: ParentConstraint<'a>,
        children_ids: &'a [crate::NodeId],
    ) -> Self {
        Self {
            tree,
            metadatas,
            layout_ctx,
            measured_children,
            probes,
            intrinsic_dependencies,
            parent_constraint,
            children_ids,
        }
    }

    /// Returns the inherited constraint.
    pub const fn parent_constraint(&self) -> ParentConstraint<'a> {
        self.parent_constraint
    }

    /// Calculates child positions without affecting measured sizes or
    /// constraints.
    ///
    /// State read here invalidates placement instead of measurement. Implement
    /// [`LayoutModifierNode::place_children`] to update those positions using
    /// cached child sizes. Keep size and constraint calculations outside this
    /// closure.
    pub fn placement<R>(&self, calculate: impl FnOnce() -> R) -> R {
        let _phase = crate::runtime::push_phase(crate::runtime::RuntimePhase::Place);
        calculate()
    }

    /// Returns whether this pass is probing natural content dimensions.
    pub fn is_intrinsic(&self) -> bool {
        self.layout_ctx.is_some_and(|context| context.intrinsic)
    }

    /// Returns copyable child identities. Measurement borrows this scope by
    /// child index so metadata stays uniquely borrowed for each call.
    pub fn children(&self) -> Vec<LayoutChild> {
        self.children_ids
            .iter()
            .enumerate()
            .map(|(index, &node_id)| {
                let instance_key = self
                    .tree
                    .get(node_id)
                    .unwrap_or_else(|| {
                        let owner = crate::runtime::current_node_id();
                        let ancestry = owner
                            .into_iter()
                            .flat_map(|id| id.ancestors(self.tree))
                            .map(|id| (id, self.tree[id].get().fn_name.clone()))
                            .collect::<Vec<_>>();
                        panic!(
                            "Direct child layout node must exist: child={node_id:?}, owner={owner:?}, ancestry={ancestry:?}"
                        )
                    })
                    .get()
                    .instance_key;
                LayoutChild {
                    node_id,
                    instance_key,
                    index,
                }
            })
            .collect()
    }

    pub(crate) fn measure_at(
        &mut self,
        index: usize,
        constraint: &Constraint,
    ) -> Result<MeasuredChild, MeasurementError> {
        self.measure_child(index, constraint, true)
    }

    fn measure_child(
        &mut self,
        index: usize,
        constraint: &Constraint,
        record: bool,
    ) -> Result<MeasuredChild, MeasurementError> {
        let node_id = *self
            .children_ids
            .get(index)
            .expect("Layout child index must refer to a direct child");
        let instance_key = self
            .tree
            .get(node_id)
            .expect("Direct child layout node must exist")
            .get()
            .instance_key;

        if let Some(probe) = self.probes.get(&node_id) {
            if probe.constraint == *constraint {
                let size = probe.size;
                if record {
                    self.record_measure(node_id, *constraint, size);
                }
                return Ok(MeasuredChild {
                    instance_key,
                    width: size.width,
                    height: size.height,
                });
            }
        }

        let size = measure_node(
            node_id,
            constraint,
            self.tree,
            self.metadatas,
            self.layout_ctx,
        )?;
        self.probes.insert(
            node_id,
            ChildProbe {
                constraint: *constraint,
                size,
            },
        );
        if record {
            self.record_measure(node_id, *constraint, size);
        }
        Ok(MeasuredChild {
            instance_key,
            width: size.width,
            height: size.height,
        })
    }

    fn record_measure(
        &mut self,
        node_id: crate::NodeId,
        constraint: Constraint,
        size: ComputedData,
    ) {
        if let Some(entry) = self.measured_children.get_mut(&node_id) {
            let consistent =
                entry.consistent && entry.constraint == constraint && entry.size == size;
            entry.constraint = constraint;
            entry.size = size;
            entry.consistent = consistent;
        } else {
            self.measured_children.insert(
                node_id,
                ChildMeasure {
                    constraint,
                    size,
                    consistent: true,
                },
            );
        }
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
        layout_ctx: Option<&'a LayoutContext<'a>>,
    ) -> Self {
        Self {
            tree,
            parent_constraint,
            children_ids,
            layout_ctx,
            measured_children: HashMap::new(),
            probes: HashMap::new(),
            intrinsic_dependencies: false,
        }
    }

    /// Returns the inherited constraint.
    pub const fn parent_constraint(&self) -> ParentConstraint<'a> {
        self.parent_constraint
    }

    /// Returns whether minimum interactive-size padding should be excluded.
    pub fn is_intrinsic(&self) -> bool {
        self.layout_ctx.is_some_and(|context| context.intrinsic)
    }

    pub(crate) fn take_measured_children(&mut self) -> HashMap<crate::NodeId, ChildMeasure> {
        std::mem::take(&mut self.measured_children)
    }

    pub(crate) fn has_intrinsic_dependencies(&self) -> bool {
        self.intrinsic_dependencies
    }

    pub(crate) fn extend_measured_children(
        &mut self,
        children: HashMap<crate::NodeId, ChildMeasure>,
    ) {
        for (child_id, measurement) in children {
            if let Some(entry) = self.measured_children.get_mut(&child_id) {
                let consistent = entry.consistent
                    && entry.constraint == measurement.constraint
                    && entry.size == measurement.size
                    && measurement.consistent;
                entry.constraint = measurement.constraint;
                entry.size = measurement.size;
                entry.consistent = consistent;
            } else {
                self.measured_children.insert(child_id, measurement);
            }
        }
    }

    pub(crate) fn measure_scope<'s>(
        &'s mut self,
        metadatas: &'s mut ComponentNodeMetaDatas,
    ) -> MeasureScope<'s> {
        MeasureScope::new(
            self.tree,
            metadatas,
            self.layout_ctx,
            &mut self.measured_children,
            &mut self.probes,
            &mut self.intrinsic_dependencies,
            self.parent_constraint,
            self.children_ids,
        )
    }

    /// Returns the direct child layout nodes of the current node.
    pub fn children(&self) -> Vec<LayoutChild> {
        self.children_ids
            .iter()
            .enumerate()
            .map(|(index, &node_id)| {
                let instance_key = self
                    .tree
                    .get(node_id)
                    .expect("Direct child layout node must exist")
                    .get()
                    .instance_key;
                LayoutChild {
                    node_id,
                    instance_key,
                    index,
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
        self.place(child.instance_key(), position);
    }

    /// Places a direct child by its `instance_key`.
    pub fn place(&mut self, instance_key: u64, position: PxPosition) {
        self.placements.push((instance_key, position));
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

    /// Consumes the result and wraps the recorded placements.
    pub fn into_placement_result(self) -> PlacementResult {
        PlacementResult {
            placements: self.placements,
        }
    }
}

/// Child placements produced by a measure pass, relative to the node origin.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlacementResult {
    placements: Vec<(u64, PxPosition)>,
}

impl PlacementResult {
    /// Creates an empty placement result.
    pub const fn new() -> Self {
        Self {
            placements: Vec::new(),
        }
    }

    /// Wraps an existing list of placements.
    pub fn from_placements(placements: Vec<(u64, PxPosition)>) -> Self {
        Self { placements }
    }

    /// Records a placement for a direct child `instance_key`.
    pub fn place(&mut self, instance_key: u64, position: PxPosition) {
        self.placements.push((instance_key, position));
    }

    /// Returns whether nothing was placed.
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Returns the number of recorded placements.
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Borrows the recorded placements.
    pub fn items(&self) -> &[(u64, PxPosition)] {
        &self.placements
    }

    /// Iterates over the recorded placements.
    pub fn iter(&self) -> std::slice::Iter<'_, (u64, PxPosition)> {
        self.placements.iter()
    }

    /// Consumes the result and returns the recorded placements.
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

impl LayoutPlacementTarget for LayoutChild {
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
    pub(crate) draw_bounds: DrawLayoutBounds,
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
        let size = metadatas
            .get(&current_node_id)
            .and_then(|metadata| metadata.computed_data)
            .unwrap_or(ComputedData::ZERO);
        Self {
            current_node_id,
            metadatas,
            draw_bounds: DrawLayoutBounds {
                position: PxPosition::ZERO,
                size,
            },
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
        RenderMetadataMut {
            metadata,
            draw_bounds: self.draw_bounds,
        }
    }

    pub(crate) fn recorded_op_count(&self) -> usize {
        self.metadatas
            .get(&self.current_node_id)
            .expect("record metadata")
            .fragment
            .ops()
            .len()
    }

    pub(crate) fn draw_layers(&self) -> &[Option<DrawLayoutBounds>] {
        self.metadatas
            .get(&self.current_node_id)
            .expect("record metadata")
            .draw_layout
            .layers
            .as_deref()
            .unwrap_or_default()
    }

    pub(crate) fn finish_draw_range(&mut self, start: usize, bounds: DrawLayoutBounds) {
        let metadata = self
            .metadatas
            .get_mut(&self.current_node_id)
            .expect("record metadata");
        for op in metadata.fragment.ops_mut().iter_mut().skip(start) {
            if op.write == Some(crate::render_graph::RenderResourceId::SceneColor) {
                op.position_override =
                    Some(bounds.position + op.position_override.unwrap_or(PxPosition::ZERO));
            }
            op.size_override.get_or_insert(bounds.size.into());
        }
    }
}

/// Mutable render metadata available during the record pass.
pub struct RenderMetadataMut<'a> {
    metadata: &'a mut ComponentNodeMetaData,
    draw_bounds: DrawLayoutBounds,
}

impl RenderMetadataMut<'_> {
    /// Returns the measured size at the current drawing layer.
    pub fn computed_data(&self) -> Option<ComputedData> {
        self.metadata.computed_data.map(|_| self.draw_bounds.size)
    }

    /// Returns the render fragment for the current node.
    pub fn fragment_mut(&mut self) -> &mut RenderFragment {
        self.metadata.fragment_mut()
    }

    /// Enables or disables child clipping for the current node.
    pub fn set_clips_children(&mut self, clips_children: bool) {
        self.metadata.clips_children = clips_children;
        if clips_children
            && self.metadata.clip_region.is_none()
            && (self.draw_bounds.position != PxPosition::ZERO
                || self.metadata.computed_data != Some(self.draw_bounds.size))
        {
            self.set_clip_shape(crate::ClipShape::RoundedRectangle {
                radii: [0.0; 4],
                exponents: [2.0; 4],
            });
        }
    }

    /// Clips the node and its descendants to a shape within its bounds.
    pub fn set_clip_shape(&mut self, shape: crate::ClipShape) {
        self.metadata.clips_children = true;
        let size = self
            .computed_data()
            .expect("shape clipping requires measured bounds");
        self.metadata.clip_region = Some(crate::ClipRegion {
            position: self.draw_bounds.position,
            size: [size.width.to_f32(), size.height.to_f32()],
            shape,
        });
    }

    /// Clips content to a fractional shape relative to the current drawing
    /// layer.
    pub fn set_clip_region(&mut self, mut region: crate::ClipRegion) {
        self.metadata.clips_children = true;
        region.position = region.position + self.draw_bounds.position;
        self.metadata.clip_region = Some(region);
    }

    /// Multiplies the current node opacity by the provided factor.
    pub fn multiply_opacity(&mut self, opacity: f32) {
        self.metadata.opacity *= opacity;
    }
}

/// Type-erased layout node used by the runtime.
#[doc(hidden)]
pub trait LayoutModifierDyn: Send + Sync {
    /// Returns a typed reference for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Measures layout using a type-erased node.
    fn measure_dyn(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError>;
    /// Recomputes child placements using cached child measurements.
    fn place_children_dyn(
        &self,
        scope: &PlacementScope<'_>,
        inner: &PlacementResult,
    ) -> Option<PlacementResult>;
    /// Compares two type-erased nodes for equality.
    fn dyn_eq(&self, other: &dyn LayoutModifierDyn) -> bool;
    /// Compares two type-erased nodes for measurement-relevant equality.
    fn dyn_measure_eq(&self, other: &dyn LayoutModifierDyn) -> bool;
    /// Compares two type-erased nodes for placement-relevant equality.
    fn dyn_placement_eq(&self, other: &dyn LayoutModifierDyn) -> bool;
    /// Clones the type-erased node.
    fn clone_box(&self) -> Box<dyn LayoutModifierDyn>;
}

impl<T> LayoutModifierDyn for T
where
    T: LayoutModifierNode + Clone + PartialEq,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn measure_dyn(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        LayoutModifierNode::measure(self, scope)
    }

    fn place_children_dyn(
        &self,
        scope: &PlacementScope<'_>,
        inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        LayoutModifierNode::place_children(self, scope, inner)
    }

    fn dyn_eq(&self, other: &dyn LayoutModifierDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| self == other)
    }

    fn dyn_measure_eq(&self, other: &dyn LayoutModifierDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| LayoutModifierNode::measure_eq(self, other))
    }

    fn dyn_placement_eq(&self, other: &dyn LayoutModifierDyn) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|other| LayoutModifierNode::placement_eq(self, other))
    }

    fn clone_box(&self) -> Box<dyn LayoutModifierDyn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn LayoutModifierDyn> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Type-erased layout node handle consumed by [`layout`].
///
/// Most callers do not construct this type directly. Instead, pass any
/// [`LayoutModifierNode`] to the `policy(...)` builder method and rely on the
/// `From<T>` conversion.
#[derive(Clone)]
pub struct LayoutModifierHandle {
    node: Box<dyn LayoutModifierDyn>,
}

impl LayoutModifierHandle {
    pub(crate) fn into_box(self) -> Box<dyn LayoutModifierDyn> {
        self.node
    }
}

impl Default for LayoutModifierHandle {
    fn default() -> Self {
        Self {
            node: Box::new(DefaultLayoutNode),
        }
    }
}

impl PartialEq for LayoutModifierHandle {
    fn eq(&self, other: &Self) -> bool {
        self.node.dyn_eq(other.node.as_ref())
    }
}

impl Prop for LayoutModifierHandle {
    fn prop_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<S> From<S> for LayoutModifierHandle
where
    S: LayoutModifierNode + Clone + PartialEq,
{
    fn from(node: S) -> Self {
        Self {
            node: Box::new(node),
        }
    }
}

/// Type-erased draw modifier handle consumed by [`layout`].
///
/// Most callers do not construct this type directly. Instead, pass any
/// [`DrawModifierNode`] to the `draw(...)` builder method and rely on the
/// `From<T>` conversion.
#[derive(Clone)]
pub struct DrawModifierHandle {
    node: Box<dyn ErasedDrawModifierNode>,
}

impl DrawModifierHandle {
    pub(crate) fn into_node(self) -> Box<dyn DrawModifierNode> {
        self.node.into_node()
    }
}

impl Default for DrawModifierHandle {
    fn default() -> Self {
        Self {
            node: Box::new(NoopDrawModifier),
        }
    }
}

impl PartialEq for DrawModifierHandle {
    fn eq(&self, other: &Self) -> bool {
        self.node.dyn_eq(other.node.as_ref())
    }
}

impl Prop for DrawModifierHandle {
    fn prop_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<N> From<N> for DrawModifierHandle
where
    N: DrawModifierNode + Clone + PartialEq,
{
    fn from(node: N) -> Self {
        Self {
            node: Box::new(node),
        }
    }
}

/// Type-erased draw modifier node used by [`DrawModifierHandle`].
trait ErasedDrawModifierNode: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn into_node(self: Box<Self>) -> Box<dyn DrawModifierNode>;
    fn dyn_eq(&self, other: &dyn ErasedDrawModifierNode) -> bool;
    fn clone_box(&self) -> Box<dyn ErasedDrawModifierNode>;
}

impl<N> ErasedDrawModifierNode for N
where
    N: DrawModifierNode + Clone + PartialEq,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_node(self: Box<Self>) -> Box<dyn DrawModifierNode> {
        self
    }

    fn dyn_eq(&self, other: &dyn ErasedDrawModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<N>()
            .is_some_and(|other| self == other)
    }

    fn clone_box(&self) -> Box<dyn ErasedDrawModifierNode> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ErasedDrawModifierNode> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// # layout
///
/// Emit an explicit layout node with a layout policy, draw modifier, modifier
/// chain, and optional child slot.
///
/// ## Usage
///
/// Build framework or internal components that need to define explicit layout
/// and rendering behavior inside a tessera composition boundary.
///
/// ## Parameters
///
/// - `policy` - the unified layout node used for measuring and placing the
///   emitted layout node, defaulting to [`DefaultLayoutNode`]
/// - `draw` - draw modifier used to record draw and compute commands for the
///   emitted layout node, defaulting to [`NoopDrawModifier`]
/// - `modifier` - node-local modifier chain attached before child content is
///   emitted, defaulting to an empty [`Modifier`]
/// - `child` - optional child slot rendered as the emitted node's content
///   subtree
///
/// ## Examples
/// ```
/// use tessera_ui::{
///     Modifier, NoopDrawModifier, RenderSlot,
///     layout::{DefaultLayoutNode, layout},
/// };
///
/// #[tessera_ui::tessera]
/// fn primitive_example(modifier: Modifier, child: RenderSlot) {
///     layout()
///         .policy(DefaultLayoutNode)
///         .draw(NoopDrawModifier)
///         .modifier(modifier)
///         .child(move || child.render());
/// }
/// ```
#[tessera(crate)]
pub fn layout(
    #[prop(into)] policy: Option<LayoutModifierHandle>,
    #[prop(into)] draw: Option<DrawModifierHandle>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let policy = policy.unwrap_or(LayoutModifierHandle::from(DefaultLayoutNode));
    let draw = draw.unwrap_or(DrawModifierHandle::from(NoopDrawModifier));
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
        runtime.set_current_layout_node_boxed(policy.into_box());
        runtime.set_current_draw_node_boxed(draw.into_node());
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

/// Default layout node that stacks children at (0,0) and uses the bounding
/// size.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DefaultLayoutNode;

impl LayoutModifierNode for DefaultLayoutNode {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child_count = scope.children().len();
        if child_count == 0 {
            return Ok(LayoutResult::new(ComputedData::min_from_constraint(
                scope.parent_constraint().as_ref(),
            )));
        }

        let mut result = LayoutResult::new(ComputedData::ZERO);
        let mut final_width = Px(0);
        let mut final_height = Px(0);
        let constraint = *scope.parent_constraint().as_ref();
        for index in 0..child_count {
            let measurement = scope.measure_at(index, &constraint)?;
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

    fn place_children(
        &self,
        scope: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        // Children are stacked at the origin, so placements only depend on the
        // cached child identities, not on their measured sizes.
        let mut result = PlacementResult::new();
        for child in scope.children() {
            result.place(child.instance_key(), PxPosition::ZERO);
        }
        Some(result)
    }
}

/// Default draw modifier that records the wrapped content unchanged.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoopDrawModifier;

impl DrawModifierNode for NoopDrawModifier {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        content.draw(ctx.render_input);
    }
}
