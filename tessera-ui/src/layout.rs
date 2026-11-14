//! Layout policies and measurement.
//!
//! ## Usage
//!
//! Measure and align content while reserving independent interaction bounds.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::{
    ComputedData, Constraint, MeasurementError, ParentConstraint, Px, RenderSlot,
    component_tree::{
        ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, DrawLayoutBounds,
        LayoutContext, LayoutDiagnosticsCollector, LayoutSnapshotMap, NodeRole, measure_node,
    },
    modifier::{
        DrawModifierContent, DrawModifierContext, DrawModifierNode, LayoutPolicy, Modifier,
    },
    px::PxPosition,
    render_graph::RenderFragment,
    runtime::{emit_node, set_emitted_node_draw},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ChildMeasure {
    pub constraint: Constraint,
    pub size: ComputedData,
    pub consistent: bool,
    pub consumed_insets: crate::Insets,
}

pub(crate) struct ChildProbe {
    consumed_insets: crate::Insets,
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
    /// Repeated probes or measures under the same constraint reuse this result.
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
        let consumed_insets = std::cell::Cell::new(
            scope
                .layout_ctx
                .map_or_else(Default::default, |context| context.consumed_insets.get()),
        );
        let context = LayoutContext::intrinsic_probe(
            &consumed_insets,
            &mut snapshots,
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
        T: Clone + 'static,
    {
        let node = scope.tree.get(self.node_id)?;
        node.get().modifier.parent_data()
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
    /// Returns the child's measured size.
    pub const fn size(&self) -> ComputedData {
        self.size
    }

    /// Reads a typed parent-data payload from the child.
    pub fn parent_data<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        let node = self.tree.get(self.node_id)?;
        node.get().modifier.parent_data()
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
    /// [`LayoutPolicy::place`] to update those positions using
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
                            .map(|id| (id, self.tree[id].get().fn_name))
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

        let consumed_insets = self
            .layout_ctx
            .map_or_else(Default::default, |context| context.consumed_insets.get());
        if let Some(probe) = self.probes.get(&node_id) {
            if probe.constraint == *constraint && probe.consumed_insets == consumed_insets {
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
                consumed_insets,
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
        let consumed_insets = self
            .layout_ctx
            .map_or_else(Default::default, |context| context.consumed_insets.get());
        if let Some(entry) = self.measured_children.get_mut(&node_id) {
            let consistent = entry.consistent
                && entry.constraint == constraint
                && entry.size == size
                && entry.consumed_insets == consumed_insets;
            entry.constraint = constraint;
            entry.size = size;
            entry.consistent = consistent;
            entry.consumed_insets = consumed_insets;
        } else {
            self.measured_children.insert(
                node_id,
                ChildMeasure {
                    consumed_insets,
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
    /// Returns window inset space already occupied by enclosing layout modifiers.
    pub fn consumed_window_insets(&self) -> crate::Insets {
        self.layout_ctx
            .map_or_else(Default::default, |context| context.consumed_insets.get())
    }

    /// Measures nested content while marking the supplied edge lengths as occupied.
    /// The enclosing consumption is restored before returning.
    pub fn with_consumed_window_insets<R>(
        &self,
        insets: crate::Insets,
        measure: impl FnOnce() -> R,
    ) -> R {
        if let Some(context) = self.layout_ctx {
            context.with_consumed_insets(self.consumed_window_insets().union(insets), measure)
        } else {
            measure()
        }
    }

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
                    && entry.consumed_insets == measurement.consumed_insets
                    && measurement.consistent;
                entry.constraint = measurement.constraint;
                entry.size = measurement.size;
                entry.consumed_insets = measurement.consumed_insets;
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

/// A node's measured size and child placements.
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
}

impl<'a> RenderInput<'a> {
    pub(crate) fn new(
        current_node_id: crate::NodeId,
        metadatas: &'a mut ComponentNodeMetaDatas,
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
            op.position_override =
                Some(bounds.position + op.position_override.unwrap_or(PxPosition::ZERO));
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
            && self.metadata.clip_region().is_none()
            && (self.draw_bounds.position != PxPosition::ZERO
                || self.metadata.computed_data != Some(self.draw_bounds.size))
        {
            self.set_clip_shape(crate::render::ClipShape::RoundedRectangle {
                radii: [0.0; 4],
                exponents: [2.0; 4],
            });
        }
    }

    /// Clips the node and its descendants to a shape within its bounds.
    pub fn set_clip_shape(&mut self, shape: crate::render::ClipShape) {
        self.metadata.clips_children = true;
        let size = self
            .computed_data()
            .expect("shape clipping requires measured bounds");
        self.metadata
            .set_clip_region(Some(crate::render::ClipRegion {
                position: self.draw_bounds.position,
                size: [size.width.to_f32(), size.height.to_f32()],
                shape,
            }));
    }

    /// Clips content to a fractional shape relative to the current drawing
    /// layer.
    pub fn set_clip_region(&mut self, mut region: crate::render::ClipRegion) {
        self.metadata.clips_children = true;
        region.position = region.position + self.draw_bounds.position;
        self.metadata.set_clip_region(Some(region));
    }

    /// Multiplies the current node opacity by the provided factor.
    pub fn multiply_opacity(&mut self, opacity: f32) {
        self.metadata.opacity *= opacity;
    }
}

/// Type-erased layout node used by the runtime.
#[doc(hidden)]
pub(crate) trait LayoutPolicyDyn {
    /// Returns a typed reference for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Measures layout using a type-erased node.
    fn measure_dyn(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError>;
    /// Recomputes child placements using cached child measurements.
    fn place_dyn(
        &self,
        scope: &PlacementScope<'_>,
        inner: &PlacementResult,
    ) -> Option<PlacementResult>;
    /// Compares two type-erased nodes for equality.
    fn dyn_eq(&self, other: &dyn LayoutPolicyDyn) -> bool;
    /// Compares two type-erased nodes for measurement-relevant equality.
    fn dyn_measure_eq(&self, other: &dyn LayoutPolicyDyn) -> bool;
    /// Compares two type-erased nodes for placement-relevant equality.
    fn dyn_placement_eq(&self, other: &dyn LayoutPolicyDyn) -> bool;
    /// Reports whether measurement uses an isolated state snapshot.
    fn isolate_measurement_state(&self) -> bool;
    /// Clones a policy for an explicitly stateful measurement snapshot.
    fn clone_box(&self) -> Box<dyn LayoutPolicyDyn>;
}

impl<T> LayoutPolicyDyn for T
where
    T: LayoutPolicy + Clone + PartialEq,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn measure_dyn(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        crate::work::record_policy::<T>(if scope.is_intrinsic() {
            crate::work::WorkPhase::IntrinsicMeasure
        } else {
            crate::work::WorkPhase::Measure
        });
        LayoutPolicy::measure(self, scope)
    }

    fn place_dyn(
        &self,
        scope: &PlacementScope<'_>,
        inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        crate::work::record_policy::<T>(crate::work::WorkPhase::Place);
        LayoutPolicy::place(self, scope, inner)
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

    fn isolate_measurement_state(&self) -> bool {
        LayoutPolicy::isolate_measurement_state(self)
    }

    fn clone_box(&self) -> Box<dyn LayoutPolicyDyn> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Default)]
pub(crate) struct LayoutPolicyHandle {
    node: Option<std::rc::Rc<dyn LayoutPolicyDyn>>,
}

impl LayoutPolicyHandle {
    pub(crate) fn as_policy(&self) -> &dyn LayoutPolicyDyn {
        self.node
            .as_deref()
            .unwrap_or(&Stack as &dyn LayoutPolicyDyn)
    }

    pub(crate) fn matches<P>(&self, candidate: &P) -> bool
    where
        P: LayoutPolicy + Clone + PartialEq,
    {
        self.as_policy()
            .as_any()
            .downcast_ref::<P>()
            .is_some_and(|current| current == candidate)
    }

    pub(crate) fn update_if_changed<P>(&mut self, candidate: &P) -> bool
    where
        P: LayoutPolicy + Clone + PartialEq,
    {
        if self.matches(candidate) {
            return false;
        }
        if TypeId::of::<P>() == TypeId::of::<Stack>() {
            *self = Self::default();
            return true;
        }
        *self = Self {
            node: Some(std::rc::Rc::new(candidate.clone())),
        };
        true
    }
}

impl PartialEq for LayoutPolicyHandle {
    fn eq(&self, other: &Self) -> bool {
        self.as_policy().dyn_eq(other.as_policy())
    }
}

impl<S> From<S> for LayoutPolicyHandle
where
    S: LayoutPolicy + Clone + PartialEq,
{
    fn from(node: S) -> Self {
        if TypeId::of::<S>() == TypeId::of::<Stack>() {
            return Self::default();
        }
        Self {
            node: Some(std::rc::Rc::new(node)),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct DrawModifierHandle {
    node: Option<std::rc::Rc<dyn ErasedDrawModifierNode>>,
}

impl DrawModifierHandle {
    pub(crate) fn new<N>(node: N) -> Self
    where
        N: DrawModifierNode + PartialEq,
    {
        if TypeId::of::<N>() == TypeId::of::<NoopDrawModifier>() {
            return Self::default();
        }
        Self {
            node: Some(std::rc::Rc::new(node)),
        }
    }

    pub(crate) fn matches<N>(&self, candidate: &N) -> bool
    where
        N: DrawModifierNode + PartialEq,
    {
        match &self.node {
            Some(node) => node
                .as_any()
                .downcast_ref::<N>()
                .is_some_and(|node| node == candidate),
            None => TypeId::of::<N>() == TypeId::of::<NoopDrawModifier>(),
        }
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.node, &other.node) {
            (None, None) => true,
            (Some(lhs), Some(rhs)) => std::rc::Rc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }

    pub(crate) fn has_node(&self) -> bool {
        self.node.is_some()
    }

    pub(crate) fn concrete_type_id(&self) -> std::any::TypeId {
        self.node
            .as_ref()
            .map_or(std::any::TypeId::of::<NoopDrawModifier>(), |node| {
                node.as_any().type_id()
            })
    }
}

impl AsRef<dyn DrawModifierNode> for DrawModifierHandle {
    fn as_ref(&self) -> &dyn DrawModifierNode {
        self.node
            .as_ref()
            .map_or(&NoopDrawModifier, |node| node.node())
    }
}

impl PartialEq for DrawModifierHandle {
    fn eq(&self, other: &Self) -> bool {
        match (&self.node, &other.node) {
            (None, None) => true,
            (Some(lhs), Some(rhs)) => lhs.dyn_eq(rhs.as_ref()),
            _ => false,
        }
    }
}

trait ErasedDrawModifierNode {
    fn as_any(&self) -> &dyn Any;
    fn node(&self) -> &dyn DrawModifierNode;
    fn dyn_eq(&self, other: &dyn ErasedDrawModifierNode) -> bool;
}

impl<N> ErasedDrawModifierNode for N
where
    N: DrawModifierNode + PartialEq,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn node(&self) -> &dyn DrawModifierNode {
        self
    }

    fn dyn_eq(&self, other: &dyn ErasedDrawModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<N>()
            .is_some_and(|other| self == other)
    }
}

/// Emits the layout node of the current component, measuring and placing its
/// children with `policy`.
///
/// Each component, slot, or build closure emits at most one node, by calling
/// either `layout` or [`group`] once; `cargo tessera check` rejects code that
/// could emit more (T0007). Configure the returned node immediately. Content
/// declared after this call, including [`LayoutNode::child`], is laid out by
/// this node. Use [`Stack`] to stack children at the origin.
///
/// Wrap policies that mutate interior state during
/// measurement in [`crate::modifier::StatefulLayoutPolicy`].
///
/// # Examples
///
/// ```
/// use tessera_ui::{
///     Modifier, NoopDrawModifier, RenderSlot,
///     layout::{Stack, layout},
/// };
///
/// #[tessera_ui::tessera]
/// fn primitive_example(modifier: Modifier, child: RenderSlot) {
///     layout(Stack)
///         .draw(NoopDrawModifier)
///         .modifier(modifier)
///         .child(child);
/// }
/// ```
pub fn layout<P>(policy: P) -> LayoutNode
where
    P: LayoutPolicy + Clone + PartialEq,
{
    emit_node(NodeRole::Layout, &policy);
    LayoutNode { _private: () }
}

/// Attaches node-local semantics, focus, or input to the current component's
/// content without introducing a layout boundary.
///
/// Children are measured and placed by the nearest enclosing layout node. Like
/// [`layout`], it emits the current component's single node.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Modifier, layout::group};
///
/// #[tessera_ui::tessera]
/// fn grouped_row(modifier: Modifier) {
///     group().modifier(modifier).child(|| {});
/// }
/// ```
pub fn group() -> GroupNode {
    emit_node(NodeRole::Group, &Stack);
    GroupNode { _private: () }
}

/// The node emitted by [`layout`].
pub struct LayoutNode {
    _private: (),
}

impl LayoutNode {
    /// Records the node's drawing; defaults to [`NoopDrawModifier`].
    pub fn draw(self, draw: impl DrawModifierNode + PartialEq) -> Self {
        set_emitted_node_draw(draw);
        self
    }

    /// Attaches a node-local modifier chain.
    pub fn modifier(self, modifier: Modifier) -> Self {
        modifier.attach();
        self
    }

    /// Expands the content laid out by this node.
    pub fn child(self, child: impl Into<RenderSlot>) {
        child.into().render();
    }
}

/// The node emitted by [`group`].
pub struct GroupNode {
    _private: (),
}

impl GroupNode {
    /// Attaches semantics, focus, and input modifiers for the grouped content.
    pub fn modifier(self, modifier: Modifier) -> Self {
        modifier.attach();
        self
    }

    /// Expands the grouped content.
    pub fn child(self, child: impl Into<RenderSlot>) {
        child.into().render();
    }
}
/// Stacks children at the origin and sizes the node to their bounding box.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Stack;

impl LayoutPolicy for Stack {
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

    fn place(
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
