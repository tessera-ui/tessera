use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
    sync::Arc,
};

use indextree::NodeId;
use rustc_hash::FxHashMap;
use tracing::debug;
use winit::window::CursorIcon;

use crate::{
    Px,
    accessibility::{AccessibilityActionHandler, AccessibilityNode},
    cursor::{CursorEventContent, PointerChange},
    focus::{
        FocusDirection, FocusRegistration, FocusRequester, FocusRevealRequest, FocusState,
        FocusTraversalPolicy,
    },
    layout::{LayoutInput, LayoutPolicyDyn, LayoutResult, PlacementScope, RenderPolicyDyn},
    modifier::{
        LayoutModifierChild, LayoutModifierInput, LayoutModifierNode, Modifier,
        OrderedModifierAction,
    },
    prop::CallbackWith,
    px::{PxPosition, PxSize},
    render_graph::RenderFragment,
    runtime::{
        RuntimePhase, push_current_component_instance_key,
        push_current_node_with_instance_logic_id, push_phase,
    },
    time::Instant,
};

use super::{
    LayoutContext, LayoutSnapshotEntry,
    constraint::{Constraint, ParentConstraint},
    nearest_replay_boundary_instance_key,
};

#[cfg(feature = "profiling")]
use crate::profiler::{Phase as ProfilerPhase, ScopeGuard as ProfilerScopeGuard};

/// A ComponentNode is a node in the component tree.
/// It represents all information about a component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeRole {
    Composition,
    Layout,
}

pub(crate) struct ComponentNode {
    /// Component function's name, for debugging purposes.
    pub(crate) fn_name: String,
    /// Whether this tree node represents a composition boundary or an explicit
    /// layout node.
    pub(crate) role: NodeRole,
    /// Stable logic identifier of this concrete component instance.
    pub(crate) instance_logic_id: u64,
    /// Stable instance identifier for this node in the current frame.
    pub(crate) instance_key: u64,
    /// Pointer input handlers for capture stage (root to leaf).
    pub(crate) pointer_preview_handlers: Vec<Box<PointerInputHandlerFn>>,
    /// Pointer input handlers for bubble stage (leaf to root).
    pub(crate) pointer_handlers: Vec<Box<PointerInputHandlerFn>>,
    /// Pointer input handlers for final stage (root to leaf).
    pub(crate) pointer_final_handlers: Vec<Box<PointerInputHandlerFn>>,
    /// Keyboard input handlers for preview stage (root to leaf).
    pub(crate) keyboard_preview_handlers: Vec<Box<KeyboardInputHandlerFn>>,
    /// Keyboard input handlers for bubble stage (leaf to root).
    pub(crate) keyboard_handlers: Vec<Box<KeyboardInputHandlerFn>>,
    /// IME input handlers for preview stage (root to leaf).
    pub(crate) ime_preview_handlers: Vec<Box<ImeInputHandlerFn>>,
    /// IME input handlers for bubble stage (leaf to root).
    pub(crate) ime_handlers: Vec<Box<ImeInputHandlerFn>>,
    /// Optional focus requester bound to this component node.
    pub(crate) focus_requester_binding: Option<FocusRequester>,
    /// Optional focus registration attached to this component node.
    pub(crate) focus_registration: Option<FocusRegistration>,
    /// Optional fallback target used when a focus scope restores focus.
    pub(crate) focus_restorer_fallback: Option<FocusRequester>,
    /// Optional traversal policy used when directional navigation starts inside
    /// this focus group or scope.
    pub(crate) focus_traversal_policy: Option<FocusTraversalPolicy>,
    /// Optional callback invoked when the node's focus state changes.
    pub(crate) focus_changed_handler: Option<FocusChangedHandler>,
    /// Optional callback invoked when the node participates in a focus event.
    pub(crate) focus_event_handler: Option<FocusEventHandler>,
    /// Optional callback used to expand virtualized content for focus search.
    pub(crate) focus_beyond_bounds_handler: Option<FocusBeyondBoundsHandler>,
    /// Optional callback used to reveal the focused rectangle inside a
    /// container.
    pub(crate) focus_reveal_handler: Option<FocusRevealHandler>,
    /// Node-local modifier chain attached during build.
    pub(crate) modifier: Modifier,
    /// Pure layout policy for measure and placement passes.
    pub(crate) layout_policy: Box<dyn LayoutPolicyDyn>,
    /// Render policy for recording draw and compute commands.
    pub(crate) render_policy: Box<dyn RenderPolicyDyn>,
    /// Optional replay metadata for subtree-level rerun.
    pub(crate) replay: Option<crate::prop::ComponentReplayData>,
    /// Whether props are equal to the previous frame snapshot.
    pub(crate) props_unchanged_from_previous: bool,
}

/// Contains metadata of the component node.
pub(crate) struct ComponentNodeMetaData {
    /// The computed data (size) of the node.
    /// None if the node is not computed yet.
    pub computed_data: Option<ComputedData>,
    /// Whether the layout cache was hit for this node in the current frame.
    pub layout_cache_hit: bool,
    /// Placement order among siblings within the parent layout result.
    pub placement_order: Option<u64>,
    /// The node's start position, relative to its parent.
    /// None if the node is not placed yet.
    pub rel_position: Option<PxPosition>,
    /// The node's start position, relative to the root window.
    /// Before placement modifiers are applied.
    /// This will be computed during drawing command's generation.
    /// None if the node is not drawn yet.
    pub base_abs_position: Option<PxPosition>,
    /// The node's start position, relative to the root window.
    /// After placement modifiers are applied.
    /// This will be computed during drawing command's generation.
    /// None if the node is not drawn yet.
    pub abs_position: Option<PxPosition>,
    /// The effective clipping rectangle for this node, considering all its
    /// ancestors. This is calculated once per frame before event handling.
    pub event_clip_rect: Option<crate::PxRect>,
    /// Render fragment associated with this node.
    ///
    /// Commands are collected during the record phase and merged into the frame
    /// graph during rendering.
    pub(crate) fragment: RenderFragment,
    /// Whether this node clips its children.
    pub clips_children: bool,
    /// Opacity multiplier applied to this node and its descendants.
    pub opacity: f32,
    /// Accessibility information for this node.
    pub accessibility: Option<AccessibilityNode>,
    /// Handler for accessibility actions on this node.
    pub accessibility_action_handler: Option<AccessibilityActionHandler>,
}

impl ComponentNodeMetaData {
    /// Creates a new `ComponentNodeMetaData` with default values.
    pub fn none() -> Self {
        Self {
            computed_data: None,
            layout_cache_hit: false,
            placement_order: None,
            rel_position: None,
            base_abs_position: None,
            abs_position: None,
            event_clip_rect: None,
            fragment: RenderFragment::default(),
            clips_children: false,
            opacity: 1.0,
            accessibility: None,
            accessibility_action_handler: None,
        }
    }

    /// Returns a mutable render fragment for this node.
    pub fn fragment_mut(&mut self) -> &mut RenderFragment {
        &mut self.fragment
    }

    /// Takes the current fragment and replaces it with an empty one.
    pub(crate) fn take_fragment(&mut self) -> RenderFragment {
        std::mem::take(&mut self.fragment)
    }
}

impl Default for ComponentNodeMetaData {
    fn default() -> Self {
        Self::none()
    }
}

pub(crate) fn direct_layout_children(node_id: NodeId, tree: &ComponentNodeTree) -> Vec<NodeId> {
    fn collect(node_id: NodeId, tree: &ComponentNodeTree, output: &mut Vec<NodeId>) {
        let Some(node_ref) = tree.get(node_id) else {
            return;
        };
        if node_ref.get().role == NodeRole::Layout {
            output.push(node_id);
            return;
        }

        for child_id in node_id.children(tree) {
            collect(child_id, tree, output);
        }
    }

    let mut children = Vec::new();
    for child_id in node_id.children(tree) {
        collect(child_id, tree, &mut children);
    }
    children
}

fn reset_frame_metadata(node_id: NodeId, component_node_metadatas: &mut ComponentNodeMetaDatas) {
    let metadata = component_node_metadatas.entry_or_default(node_id);
    metadata.computed_data = None;
    metadata.layout_cache_hit = false;
    metadata.placement_order = None;
    metadata.rel_position = None;
    metadata.base_abs_position = None;
    metadata.abs_position = None;
    metadata.event_clip_rect = None;
    metadata.fragment = RenderFragment::default();
    metadata.clips_children = false;
    metadata.opacity = 1.0;
}

/// A tree of component nodes, using `indextree::Arena` for storage.
pub(crate) type ComponentNodeTree = indextree::Arena<ComponentNode>;

/// Stores component metadata for a single UI tree.
#[derive(Default)]
pub(crate) struct ComponentNodeMetaDatas {
    entries: FxHashMap<NodeId, ComponentNodeMetaData>,
}

impl ComponentNodeMetaDatas {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn insert(
        &mut self,
        node_id: NodeId,
        metadata: ComponentNodeMetaData,
    ) -> Option<ComponentNodeMetaData> {
        self.entries.insert(node_id, metadata)
    }

    pub(crate) fn remove(&mut self, node_id: &NodeId) -> Option<ComponentNodeMetaData> {
        self.entries.remove(node_id)
    }

    pub(crate) fn get(&self, node_id: &NodeId) -> Option<&ComponentNodeMetaData> {
        self.entries.get(node_id)
    }

    pub(crate) fn get_mut(&mut self, node_id: &NodeId) -> Option<&mut ComponentNodeMetaData> {
        self.entries.get_mut(node_id)
    }

    pub(crate) fn entry_or_default(&mut self, node_id: NodeId) -> &mut ComponentNodeMetaData {
        self.entries.entry(node_id).or_default()
    }

    #[cfg(feature = "testing")]
    pub(crate) fn with_entries<R>(
        &self,
        f: impl FnOnce(&FxHashMap<NodeId, ComponentNodeMetaData>) -> R,
    ) -> R {
        f(&self.entries)
    }
}

/// Represents errors that can occur during node measurement.
#[derive(Debug, Clone, PartialEq)]
pub enum MeasurementError {
    /// Indicates that the specified node was not found in the component tree.
    NodeNotFoundInTree,
    /// Indicates that metadata for the specified node was not found (currently
    /// not a primary error source in measure_node).
    NodeNotFoundInMeta,
    /// Indicates that the layout policy for a node failed. Contains a string
    /// detailing the failure.
    MeasureFnFailed(String),
    /// Indicates that the measurement of a child node failed during a
    /// parent's layout calculation. Contains the `NodeId` of the child
    /// that failed.
    ChildMeasurementFailed(NodeId),
}

/// Pointer input dispatch pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEventPass {
    /// Dispatch from root to leaf before the main pointer pass.
    Initial,
    /// Main pointer dispatch from leaf to root.
    Main,
    /// Final pointer dispatch from root to leaf.
    Final,
}

/// Pointer-specific input handler.
pub type PointerInputHandlerFn = dyn Fn(PointerInput) + Send + Sync;
/// Keyboard-specific input handler.
pub type KeyboardInputHandlerFn = dyn Fn(KeyboardInput) + Send + Sync;
/// IME-specific input handler.
pub type ImeInputHandlerFn = dyn Fn(ImeInput) + Send + Sync;
/// Focus-changed callback attached to a component node.
pub type FocusChangedHandler = CallbackWith<FocusState>;
/// Focus-event callback attached to a component node.
pub type FocusEventHandler = CallbackWith<FocusState>;
/// Beyond-bounds focus callback attached to a virtualized container node.
pub type FocusBeyondBoundsHandler = CallbackWith<FocusDirection, bool>;
/// Reveal callback attached to a scrollable container node.
pub type FocusRevealHandler = CallbackWith<FocusRevealRequest, bool>;

/// Input for pointer handlers.
///
/// This type carries pointer event payload, local geometry, and event
/// consumption helpers. Side effects that are not pointer-specific should use
/// dedicated APIs such as semantics modifiers, hover cursor modifiers, window
/// action helpers, or the IME session bridge instead of a generic request bag.
pub struct PointerInput<'a> {
    /// Current pointer dispatch pass.
    pub pass: PointerEventPass,
    /// The size of the component node, computed during the measure stage.
    pub computed_data: ComputedData,
    /// The position of the cursor, if available.
    /// Relative to the root position of the component.
    pub cursor_position_rel: Option<PxPosition>,
    /// Absolute cursor position in window coordinates.
    pub(crate) cursor_position_abs: &'a mut Option<PxPosition>,
    /// Pointer changes from the event loop, if any.
    pub pointer_changes: &'a mut Vec<PointerChange>,
    /// The current state of the keyboard modifiers at the time of the event.
    pub key_modifiers: winit::keyboard::ModifiersState,
    pub(crate) ime_request: &'a mut Option<ImeRequest>,
    pub(crate) request_window_drag: &'a mut bool,
}

impl PointerInput<'_> {
    /// Marks all current pointer changes as consumed.
    pub fn consume_pointer_changes(&mut self) {
        for change in self.pointer_changes.iter_mut() {
            change.consume();
        }
    }

    /// Returns whether any current pointer change is an unconsumed release.
    pub fn has_unconsumed_release(&self) -> bool {
        self.pointer_changes.iter().any(|change| {
            !change.is_consumed() && matches!(change.content, CursorEventContent::Released(_))
        })
    }

    /// Returns the absolute cursor position in window coordinates.
    pub fn cursor_position_abs(&self) -> Option<PxPosition> {
        *self.cursor_position_abs
    }

    /// Blocks pointer input to other components.
    pub fn block_cursor(&mut self) {
        self.cursor_position_abs.take();
        self.consume_pointer_changes();
    }

    /// Blocks pointer changes to other components.
    pub fn block_all(&mut self) {
        self.block_cursor();
    }

    /// Begins a system drag move for the current window.
    pub fn drag_window(&mut self) {
        *self.request_window_drag = true;
    }

    /// Returns the IME session bridge for the current frame.
    pub fn ime_session(&mut self) -> ImeSession<'_> {
        ImeSession {
            request: self.ime_request,
        }
    }
}

/// Input for keyboard handlers.
///
/// This type carries keyboard event payload and event consumption helpers.
/// IME updates, semantics, and other side effects are exposed through narrower
/// dedicated APIs rather than a general-purpose request sink.
pub struct KeyboardInput<'a> {
    /// The size of the component node, computed during the measure stage.
    pub computed_data: ComputedData,
    /// Keyboard events from the event loop, if any.
    pub keyboard_events: &'a mut Vec<winit::event::KeyEvent>,
    /// The current state of the keyboard modifiers at the time of the event.
    pub key_modifiers: winit::keyboard::ModifiersState,
    pub(crate) ime_request: &'a mut Option<ImeRequest>,
}

impl KeyboardInput<'_> {
    /// Blocks keyboard events to other components.
    pub fn block_keyboard(&mut self) {
        self.keyboard_events.clear();
    }

    /// Returns the IME session bridge for the current frame.
    pub fn ime_session(&mut self) -> ImeSession<'_> {
        ImeSession {
            request: self.ime_request,
        }
    }
}

/// Input for IME handlers.
///
/// This type carries IME event payload and event consumption helpers. IME
/// snapshot publication is performed through [`ImeSession`].
pub struct ImeInput<'a> {
    /// The size of the component node, computed during the measure stage.
    pub computed_data: ComputedData,
    /// IME events from the event loop, if any.
    pub ime_events: &'a mut Vec<winit::event::Ime>,
    pub(crate) ime_request: &'a mut Option<ImeRequest>,
}

impl ImeInput<'_> {
    /// Blocks IME events to other components.
    pub fn block_ime(&mut self) {
        self.ime_events.clear();
    }

    /// Returns the IME session bridge for the current frame.
    pub fn ime_session(&mut self) -> ImeSession<'_> {
        ImeSession {
            request: self.ime_request,
        }
    }
}

/// A collection of requests that components can make to the windowing system
/// for the current frame. This struct's lifecycle is confined to a single
/// `compute` pass.
#[derive(Default, Debug)]
pub(crate) struct WindowRequests {
    /// The cursor icon requested by a component. If multiple components request
    /// a cursor, the last one to make a request in a frame "wins", since
    /// it's executed later.
    pub cursor_icon: CursorIcon,
    /// An Input Method Editor (IME) request.
    /// If multiple components request IME, the one from the "newer" component
    /// (which is processed later in the state handling pass) will overwrite
    /// previous requests.
    pub ime_request: Option<ImeRequest>,
    /// Whether a node requested a native window drag for the current frame.
    pub request_window_drag: bool,
}

/// Frame-local IME bridge used by input handlers to publish text input state.
///
/// Use this bridge when an input handler needs to expose the current text-input
/// snapshot to the renderer-facing IME integration without mutating renderer
/// aggregation state directly.
pub struct ImeSession<'a> {
    request: &'a mut Option<ImeRequest>,
}

impl ImeSession<'_> {
    /// Publishes the IME snapshot for the current frame.
    pub fn update(&mut self, request: ImeRequest) {
        *self.request = Some(request);
    }

    /// Clears the IME snapshot for the current frame.
    pub fn clear(&mut self) {
        *self.request = None;
    }
}

/// A request to the windowing system to open an Input Method Editor (IME).
/// This is typically used for text input components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImeRequest {
    /// The size of the area where the IME is requested.
    pub size: PxSize,
    /// The position of the IME anchor relative to the requesting node.
    pub local_position: PxPosition,
    /// The current ordered selection range in the backing text buffer.
    pub selection_range: Option<std::ops::Range<usize>>,
    /// The current ordered composition range in the backing text buffer.
    pub composition_range: Option<std::ops::Range<usize>>,
    /// The absolute position where the IME should be placed.
    /// This is set internally by the component tree during the compute pass.
    pub(crate) position: Option<PxPosition>, // should be setted in tessera node tree compute
}

impl ImeRequest {
    /// Creates a new IME request with the target input area size.
    ///
    /// The absolute position is injected during the compute pass.
    pub fn new(size: PxSize) -> Self {
        Self {
            size,
            local_position: PxPosition::ZERO,
            selection_range: None,
            composition_range: None,
            position: None, // Position will be set during the compute phase
        }
    }

    /// Sets the position of the IME anchor relative to the requesting node.
    pub fn with_local_position(mut self, local_position: PxPosition) -> Self {
        self.local_position = local_position;
        self
    }

    /// Sets the current ordered text selection range.
    pub fn with_selection_range(mut self, selection_range: Option<std::ops::Range<usize>>) -> Self {
        self.selection_range = selection_range;
        self
    }

    /// Sets the current ordered text composition range.
    pub fn with_composition_range(
        mut self,
        composition_range: Option<std::ops::Range<usize>>,
    ) -> Self {
        self.composition_range = composition_range;
        self
    }
}

fn apply_layout_placements(
    placements: &[(u64, PxPosition)],
    tree: &ComponentNodeTree,
    children: &[NodeId],
    component_node_metadatas: &mut ComponentNodeMetaDatas,
) {
    if placements.is_empty() || children.is_empty() {
        return;
    }
    let mut child_map = HashMap::new();
    for child_id in children {
        if let Some(child) = tree.get(*child_id) {
            child_map.insert(child.get().instance_key, *child_id);
        }
    }
    for (placement_order, (instance_key, position)) in placements.iter().enumerate() {
        if let Some(child_id) = child_map.get(instance_key) {
            place_node(
                *child_id,
                *position,
                placement_order as u64,
                component_node_metadatas,
            );
        }
    }
}

fn restore_cached_subtree_metadata(
    node_id: NodeId,
    rel_position: Option<PxPosition>,
    tree: &ComponentNodeTree,
    component_node_metadatas: &mut ComponentNodeMetaDatas,
    layout_ctx: &LayoutContext<'_>,
) -> bool {
    let Some(node) = tree.get(node_id) else {
        return false;
    };
    let instance_key = node.get().instance_key;
    let Some(entry) = layout_ctx.snapshot(instance_key) else {
        return false;
    };

    let size = entry.layout_result.size;
    let placements = entry.layout_result.placements.clone();

    {
        let metadata = component_node_metadatas.entry_or_default(node_id);
        metadata.computed_data = Some(size);
        metadata.layout_cache_hit = true;
        if let Some(position) = rel_position {
            metadata.rel_position = Some(position);
        }
    }

    let mut child_positions = HashMap::new();
    let mut child_orders = HashMap::new();
    for (placement_order, (child_key, child_pos)) in placements.into_iter().enumerate() {
        child_positions.insert(child_key, child_pos);
        child_orders.insert(child_key, placement_order as u64);
    }

    for child_id in node_id.children(tree) {
        let child_layout = tree.get(child_id).map(|child| {
            let instance_key = child.get().instance_key;
            (
                child_positions.get(&instance_key).copied(),
                child_orders.get(&instance_key).copied(),
            )
        });
        let (child_rel_position, child_placement_order) = child_layout.unwrap_or((None, None));
        if let Some(placement_order) = child_placement_order {
            component_node_metadatas
                .entry_or_default(child_id)
                .placement_order = Some(placement_order);
        }
        if !restore_cached_subtree_metadata(
            child_id,
            child_rel_position,
            tree,
            component_node_metadatas,
            layout_ctx,
        ) {
            return false;
        }
    }

    true
}

#[derive(Clone)]
struct MeasuredNodeLayout {
    size: ComputedData,
    placements: Vec<(u64, PxPosition)>,
    measured_children: HashMap<NodeId, crate::layout::ChildMeasure>,
}

struct MeasureLayoutContext<'a, 'ctx> {
    tree: &'a ComponentNodeTree,
    children: &'a [NodeId],
    component_node_metadatas: *mut ComponentNodeMetaDatas,
    layout_ctx: Option<&'a LayoutContext<'ctx>>,
}

fn measure_base_layout(
    layout_policy: &dyn LayoutPolicyDyn,
    layout_ctx: &MeasureLayoutContext<'_, '_>,
    constraint: &Constraint,
) -> Result<MeasuredNodeLayout, MeasurementError> {
    let input = LayoutInput::new(
        layout_ctx.tree,
        ParentConstraint::new(constraint),
        layout_ctx.children,
        layout_ctx.component_node_metadatas,
        layout_ctx.layout_ctx,
    );
    let scope = input.measure_scope();
    let layout_result = layout_policy.measure_dyn(&scope)?;
    Ok(MeasuredNodeLayout {
        size: layout_result.size,
        placements: layout_result.placements,
        measured_children: input.take_measured_children(),
    })
}

fn measure_with_layout_modifiers(
    modifiers: &[Arc<dyn LayoutModifierNode>],
    layout_policy: &dyn LayoutPolicyDyn,
    layout_ctx: &MeasureLayoutContext<'_, '_>,
    constraint: &Constraint,
) -> Result<MeasuredNodeLayout, MeasurementError> {
    if modifiers.is_empty() {
        return measure_base_layout(layout_policy, layout_ctx, constraint);
    }

    struct ModifierChildRunner<'a, 'b> {
        next: &'b mut dyn FnMut(&Constraint) -> Result<MeasuredNodeLayout, MeasurementError>,
        last: Option<MeasuredNodeLayout>,
        placements: Vec<(u64, PxPosition)>,
        _marker: std::marker::PhantomData<&'a ()>,
    }

    impl LayoutModifierChild for ModifierChildRunner<'_, '_> {
        fn measure(&mut self, constraint: &Constraint) -> Result<ComputedData, MeasurementError> {
            let measured = (self.next)(constraint)?;
            let size = measured.size;
            self.last = Some(measured);
            Ok(size)
        }

        fn place(&mut self, position: PxPosition) {
            let measured = self
                .last
                .as_ref()
                .expect("layout modifier child must be measured before placement");
            for (instance_key, child_position) in &measured.placements {
                self.placements
                    .push((*instance_key, position + *child_position));
            }
        }
    }

    let head = &modifiers[0];
    let tail = &modifiers[1..];
    let mut next = |next_constraint: &Constraint| {
        measure_with_layout_modifiers(tail, layout_policy, layout_ctx, next_constraint)
    };
    let input = LayoutInput::new(
        layout_ctx.tree,
        ParentConstraint::new(constraint),
        layout_ctx.children,
        layout_ctx.component_node_metadatas,
        layout_ctx.layout_ctx,
    );
    let modifier_input = LayoutModifierInput {
        layout_input: &input,
    };
    let mut child = ModifierChildRunner {
        next: &mut next,
        last: None,
        placements: Vec::new(),
        _marker: std::marker::PhantomData,
    };
    let size = head.measure(&modifier_input, &mut child)?.size;
    if let Some(measured) = child.last.as_ref() {
        input.extend_measured_children(measured.measured_children.clone());
    }
    Ok(MeasuredNodeLayout {
        size,
        placements: child.placements,
        measured_children: input.take_measured_children(),
    })
}

fn relayout_base_layout(
    layout_policy: &dyn LayoutPolicyDyn,
    tree: &ComponentNodeTree,
    children: &[NodeId],
    cached_child_sizes: &[ComputedData],
    constraint: &Constraint,
    cached_size: ComputedData,
) -> Option<Vec<(u64, PxPosition)>> {
    if cached_child_sizes.len() != children.len() {
        return None;
    }

    let child_sizes: HashMap<NodeId, ComputedData> = children
        .iter()
        .copied()
        .zip(cached_child_sizes.iter().copied())
        .collect();
    let scope = PlacementScope::new(
        tree,
        ParentConstraint::new(constraint),
        children,
        &child_sizes,
        cached_size,
    );
    layout_policy.place_children_dyn(&scope)
}

/// Measures a single node recursively, returning its size or an error.
pub(crate) fn measure_node(
    node_id: NodeId,
    parent_constraint: &Constraint,
    tree: &ComponentNodeTree,
    component_node_metadatas: &mut ComponentNodeMetaDatas,
    layout_ctx: Option<&LayoutContext<'_>>,
) -> Result<ComputedData, MeasurementError> {
    let node_data_ref = tree
        .get(node_id)
        .ok_or(MeasurementError::NodeNotFoundInTree)?;
    let node_data = node_data_ref.get();
    #[cfg(feature = "profiling")]
    let mut profiler_guard = Some(ProfilerScopeGuard::new(
        ProfilerPhase::Measure,
        Some(node_id),
        node_data_ref.parent(),
        Some(node_data.fn_name.as_str()),
    ));

    let children = direct_layout_children(node_id, tree);
    let timer = Instant::now();

    debug!(
        "Measuring node {} with {} children, parent constraint: {:?}",
        node_data.fn_name,
        children.len(),
        parent_constraint
    );

    // Ensure thread-local current node context for nested control-flow
    // instrumentation.
    let _node_ctx_guard = push_current_node_with_instance_logic_id(
        node_id,
        node_data.instance_logic_id,
        node_data.fn_name.as_str(),
    );
    let replay_boundary_instance_key = nearest_replay_boundary_instance_key(node_id, tree);
    let _instance_ctx_guard = push_current_component_instance_key(replay_boundary_instance_key);
    let _phase_guard = push_phase(RuntimePhase::Measure);

    let layout_policy = &node_data.layout_policy;
    let layout_modifiers: Vec<_> = node_data
        .modifier
        .ordered_actions()
        .into_iter()
        .filter_map(|action| match action {
            OrderedModifierAction::Layout(node) => Some(node.node()),
            _ => None,
        })
        .collect();
    if let Some(layout_ctx) = layout_ctx {
        layout_ctx.inc_measure_node_calls();
        if let Some(entry) = layout_ctx.snapshot(node_data.instance_key) {
            let same_constraint = entry.constraint_key == *parent_constraint;
            let node_self_measure_dirty = layout_ctx
                .measure_self_nodes
                .contains(&node_data.instance_key);
            let node_self_placement_dirty = layout_ctx
                .placement_self_nodes
                .contains(&node_data.instance_key);
            let node_effective_dirty = layout_ctx
                .dirty_effective_nodes
                .contains(&node_data.instance_key);
            let has_all_child_constraints = entry.child_constraints.len() == children.len();
            let has_all_child_sizes = entry.child_sizes.len() == children.len();
            let can_try_reuse = same_constraint
                && !node_self_measure_dirty
                && has_all_child_constraints
                && (!node_effective_dirty || has_all_child_sizes);
            if can_try_reuse {
                let cached_result = entry.layout_result.clone();
                let cached_child_constraints = entry.child_constraints.clone();
                let cached_child_sizes = entry.child_sizes.clone();

                if !node_effective_dirty
                    && restore_cached_subtree_metadata(
                        node_id,
                        None,
                        tree,
                        component_node_metadatas,
                        layout_ctx,
                    )
                {
                    apply_layout_placements(
                        &cached_result.placements,
                        tree,
                        &children,
                        component_node_metadatas,
                    );
                    layout_ctx.inc_cache_hit_direct();
                    return Ok(cached_result.size);
                }

                let dirty_children: Vec<(usize, NodeId, Constraint)> = children
                    .iter()
                    .enumerate()
                    .filter_map(|(index, child_id)| {
                        tree.get(*child_id).and_then(|child| {
                            if layout_ctx
                                .dirty_effective_nodes
                                .contains(&child.get().instance_key)
                            {
                                Some((index, *child_id, cached_child_constraints[index]))
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                let mut child_size_changed = false;
                for (index, child_id, _constraint) in &dirty_children {
                    let size = measure_node(
                        *child_id,
                        &cached_child_constraints[*index],
                        tree,
                        component_node_metadatas,
                        Some(layout_ctx),
                    )?;
                    if size != cached_child_sizes[*index] {
                        child_size_changed = true;
                        break;
                    }
                }
                if child_size_changed {
                    layout_ctx.inc_cache_miss_child_size();
                } else {
                    let mut restored = true;
                    for child_id in &children {
                        let Some(child) = tree.get(*child_id) else {
                            restored = false;
                            break;
                        };
                        if layout_ctx
                            .dirty_effective_nodes
                            .contains(&child.get().instance_key)
                        {
                            continue;
                        }
                        if !restore_cached_subtree_metadata(
                            *child_id,
                            None,
                            tree,
                            component_node_metadatas,
                            layout_ctx,
                        ) {
                            restored = false;
                            break;
                        }
                    }
                    if restored {
                        reset_frame_metadata(node_id, component_node_metadatas);
                        let placements = if node_self_placement_dirty {
                            match relayout_base_layout(
                                layout_policy.as_ref(),
                                tree,
                                &children,
                                &cached_child_sizes,
                                parent_constraint,
                                cached_result.size,
                            ) {
                                Some(placements) => placements,
                                None => {
                                    layout_ctx.inc_cache_miss_dirty_self();
                                    // This node can reuse measurement but not placements; fall back
                                    // to a full measure+place pass below.
                                    Vec::new()
                                }
                            }
                        } else {
                            cached_result.placements.clone()
                        };
                        if node_self_placement_dirty && placements.is_empty() {
                            // Fall through to the full measure path below.
                        } else {
                            apply_layout_placements(
                                &placements,
                                tree,
                                &children,
                                component_node_metadatas,
                            );
                            if let Some(metadata) = component_node_metadatas.get_mut(&node_id) {
                                metadata.computed_data = Some(cached_result.size);
                                metadata.layout_cache_hit = true;
                            }
                            if node_self_placement_dirty {
                                layout_ctx.insert_snapshot(
                                    node_data.instance_key,
                                    LayoutSnapshotEntry {
                                        constraint_key: *parent_constraint,
                                        layout_result: LayoutResult {
                                            size: cached_result.size,
                                            placements,
                                        },
                                        child_constraints: cached_child_constraints,
                                        child_sizes: cached_child_sizes,
                                    },
                                );
                            }
                            layout_ctx.inc_cache_hit_boundary();
                            return Ok(cached_result.size);
                        }
                    }
                }
            }
            if !same_constraint {
                layout_ctx.inc_cache_miss_constraint();
            } else if node_self_measure_dirty || node_self_placement_dirty {
                layout_ctx.inc_cache_miss_dirty_self();
            } else if node_effective_dirty {
                layout_ctx.inc_cache_miss_child_size();
            } else {
                layout_ctx.inc_cache_miss_no_entry();
            }
        } else {
            layout_ctx.inc_cache_miss_no_entry();
        }
    }

    reset_frame_metadata(node_id, component_node_metadatas);
    let measure_layout_ctx = MeasureLayoutContext {
        tree,
        children: &children,
        component_node_metadatas,
        layout_ctx,
    };
    let measured = measure_with_layout_modifiers(
        &layout_modifiers,
        layout_policy.as_ref(),
        &measure_layout_ctx,
        parent_constraint,
    )?;
    let size = measured.size;
    let measured_children = measured.measured_children;
    let placements = measured.placements;
    apply_layout_placements(&placements, tree, &children, component_node_metadatas);

    component_node_metadatas
        .entry_or_default(node_id)
        .computed_data = Some(size);

    #[cfg(feature = "profiling")]
    if let Some(guard) = &mut profiler_guard {
        guard.set_computed_size(size.width.0, size.height.0);
    }

    if let Some(layout_ctx) = layout_ctx {
        let mut cacheable = true;
        let mut child_constraints = Vec::with_capacity(children.len());
        let mut child_sizes = Vec::with_capacity(children.len());
        for child_id in &children {
            let Some(measurement) = measured_children.get(child_id) else {
                cacheable = false;
                break;
            };
            if !measurement.consistent {
                cacheable = false;
                break;
            }
            child_constraints.push(measurement.constraint);
            child_sizes.push(measurement.size);
        }
        if cacheable {
            let layout_result = LayoutResult { size, placements };
            layout_ctx.insert_snapshot(
                node_data.instance_key,
                LayoutSnapshotEntry {
                    constraint_key: *parent_constraint,
                    layout_result,
                    child_constraints,
                    child_sizes,
                },
            );
            layout_ctx.inc_cache_store_count();
        } else {
            layout_ctx.remove_snapshot(node_data.instance_key);
            layout_ctx.inc_cache_drop_non_cacheable_count();
        }
    }

    debug!(
        "Measured node {} in {:?} with size {:?}",
        node_data.fn_name,
        timer.elapsed(),
        size
    );

    #[cfg(feature = "profiling")]
    if let Some(guard) = &mut profiler_guard {
        guard.set_computed_size(size.width.0, size.height.0);
    }

    Ok(size)
}

/// Places a node at the specified relative position within its parent.
pub(crate) fn place_node(
    node: indextree::NodeId,
    rel_position: PxPosition,
    placement_order: u64,
    component_node_metadatas: &mut ComponentNodeMetaDatas,
) {
    let metadata = component_node_metadatas.entry_or_default(node);
    metadata.rel_position = Some(rel_position);
    metadata.placement_order = Some(placement_order);
}

/// Layout information computed at the measure stage, representing the size of a
/// node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputedData {
    /// The resolved width of the node in physical pixels.
    pub width: Px,
    /// The resolved height of the node in physical pixels.
    pub height: Px,
}

impl Add for ComputedData {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl AddAssign for ComputedData {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl ComputedData {
    /// Zero-sized layout data.
    pub const ZERO: Self = Self {
        width: Px(0),
        height: Px(0),
    };

    /// Calculates the minimum size guaranteed by a constraint interval.
    pub fn min_from_constraint(constraint: &Constraint) -> Self {
        let width = constraint.width.min;
        let height = constraint.height.min;
        Self { width, height }
    }

    /// Returns the component-wise minimum of two computed sizes.
    pub fn min(self, rhs: Self) -> Self {
        Self {
            width: self.width.min(rhs.width),
            height: self.height.min(rhs.height),
        }
    }

    /// Returns the component-wise maximum of two computed sizes.
    pub fn max(self, rhs: Self) -> Self {
        Self {
            width: self.width.max(rhs.width),
            height: self.height.max(rhs.height),
        }
    }
}
