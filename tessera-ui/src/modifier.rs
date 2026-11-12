//! Modifier chains for node-local layout, drawing, focus, semantics, and other
//! node-scoped behavior.
//!
//! ## Usage
//!
//! Build reusable modifier chains that attach behavior directly to the current
//! component node.

mod attached;
mod diff;
mod storage;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};

use rustc_hash::FxHasher;
use smallvec::SmallVec;

use self::storage::{Chain as ModifierElement, NodeRef};

pub(crate) use self::attached::{AttachedModifierChain, flush_lifecycle};
pub use self::attached::{ModifierNodeContext, ModifierNodeId};
pub(crate) use self::storage::OwnedNode;

macro_rules! modifier_lifecycle {
    () => {
        /// Called after this modifier node is attached to its component.
        fn on_attach(&self, _context: $crate::modifier::ModifierNodeContext) {}

        /// Called when compatible new inputs update this attached node.
        fn on_update(&self, _context: $crate::modifier::ModifierNodeContext) {}

        /// Called after this node is removed from its component.
        fn on_detach(&self, _context: $crate::modifier::ModifierNodeContext) {}
    };
}

pub(crate) use modifier_lifecycle;

use crate::{
    AccessibilityActionHandler, AccessibilityNode, ComputedData, Constraint, FocusGroupNode,
    FocusProperties, FocusRequester, FocusScopeNode, FocusState, FocusTraversalPolicy, ImeInput,
    KeyboardInput, MeasurementError, PointerInput, PxPosition,
    clipboard::ClipboardInputModifierNode,
    focus::{FocusDirection, FocusNode, FocusRevealRequest},
    layout::{
        LayoutInput, LayoutResult, MeasureScope, PlacementResult, PlacementScope, RenderInput,
    },
    prop::CallbackWith,
    runtime::{TesseraRuntime, ensure_build_phase},
    winit::window::CursorIcon,
};

/// Parent-data payloads collected from modifier nodes.
pub type ParentDataMap = HashMap<TypeId, Box<dyn Any>>;

/// Child measurement entry used by layout modifier nodes.
pub trait LayoutModifierChild {
    /// Measures the wrapped content under the given constraint.
    fn measure(&mut self, constraint: &Constraint) -> Result<ComputedData, MeasurementError>;

    /// Places the wrapped content at the provided relative position.
    fn place(&mut self, position: PxPosition);
}

/// Input passed to layout modifier nodes.
pub struct LayoutModifierInput<'a> {
    /// The original layout input for the current node.
    pub layout_input: &'a LayoutInput<'a>,
}

impl LayoutModifierInput<'_> {
    /// Calculates positions and placement bookkeeping independent of measured
    /// size.
    ///
    /// State reads invalidate placement. Size and constraint calculations must
    /// remain outside this closure.
    pub fn placement<R>(&self, calculate: impl FnOnce() -> R) -> R {
        let _phase = crate::runtime::push_phase(crate::runtime::RuntimePhase::Place);
        calculate()
    }
}

/// Draw continuation used by draw modifier nodes.
pub trait DrawModifierContent {
    /// Records the wrapped content.
    fn draw(&mut self, input: &mut RenderInput<'_>);
}

/// Draw input passed to draw modifier nodes.
pub struct DrawModifierContext<'a, 'b> {
    /// The render input for the current node.
    pub render_input: &'a mut RenderInput<'b>,
}

/// Measures a layout node's children and places them.
///
/// Pass a policy to [`crate::layout::layout`].
pub trait LayoutPolicy: Any {
    /// Measures the children and returns this node's size and placements.
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError>;

    /// Compares measurement-relevant state for layout dirty tracking.
    fn measure_eq(&self, other: &Self) -> bool
    where
        Self: Sized + PartialEq,
    {
        self == other
    }

    /// Compares placement-relevant state for layout dirty tracking.
    fn placement_eq(&self, other: &Self) -> bool
    where
        Self: Sized + PartialEq,
    {
        self == other
    }

    /// Returns whether measurement needs an isolated clone of this policy.
    ///
    /// Most policies are immutable configuration and should leave this false.
    /// Use [`StatefulLayoutPolicy`] when measurement intentionally mutates
    /// interior state that must not affect later build comparisons.
    #[doc(hidden)]
    fn isolate_measurement_state(&self) -> bool {
        false
    }

    /// Recomputes placements from cached child sizes when only placement
    /// inputs changed.
    ///
    /// Returning `None` makes the node measure again instead.
    fn place(
        &self,
        _scope: &PlacementScope<'_>,
        _measured: &PlacementResult,
    ) -> Option<PlacementResult> {
        None
    }
}

/// A modifier that measures and places the content it wraps.
pub trait LayoutModifier: Any {
    modifier_lifecycle!();

    /// Measures the wrapped content and returns the wrapped size.
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError>;

    /// Compares measurement-relevant state for layout dirty tracking.
    fn measure_eq(&self, other: &Self) -> bool
    where
        Self: Sized + PartialEq,
    {
        self == other
    }

    /// Compares placement-relevant state for layout dirty tracking.
    fn placement_eq(&self, other: &Self) -> bool
    where
        Self: Sized + PartialEq,
    {
        self == other
    }
}

/// Explicitly opts a layout policy into a per-node measurement snapshot.
///
/// The wrapped policy is cloned once when its layout node is created or
/// changed. Stable recomposition retains both the configured policy and its
/// current measurement snapshot.
#[derive(Clone)]
pub struct StatefulLayoutPolicy<P>(pub P);

impl<P> PartialEq for StatefulLayoutPolicy<P>
where
    P: LayoutPolicy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P> LayoutPolicy for StatefulLayoutPolicy<P>
where
    P: LayoutPolicy + Clone + PartialEq,
{
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        self.0.measure(scope)
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.0.measure_eq(&other.0)
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.0.placement_eq(&other.0)
    }

    fn place(
        &self,
        scope: &PlacementScope<'_>,
        measured: &PlacementResult,
    ) -> Option<PlacementResult> {
        self.0.place(scope, measured)
    }

    fn isolate_measurement_state(&self) -> bool {
        true
    }
}

/// A node-local placement modifier that transforms the current node position
/// without affecting measured size.
pub trait PlacementModifierNode: Any {
    modifier_lifecycle!();
    /// Returns the concrete modifier name used by layout inspection.
    fn diagnostic_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Returns the transformed node position relative to the parent.
    fn transform_position(&self, position: PxPosition) -> PxPosition;
}

/// A node-local draw modifier.
pub trait DrawModifierNode: Any {
    modifier_lifecycle!();
    /// Returns the concrete modifier name used by paint inspection.
    fn diagnostic_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Records drawing behavior around the wrapped content.
    ///
    /// Drawing uses the content bounds at this position in the modifier chain.
    /// For example, a background before padding includes that padding, while a
    /// background after padding covers only the inner content.
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent);
}

/// A node-local parent-data modifier.
pub trait ParentDataModifierNode: Any {
    modifier_lifecycle!();
    /// Applies parent data visible to the parent layout.
    fn apply_parent_data(&self, data: &mut ParentDataMap);

    /// Returns whether this node supplies the same layout inputs as another.
    ///
    /// The default conservatively invalidates the parent layout when the node
    /// is replaced. Override this for value-based parent data such as weights.
    fn equivalent(&self, _other: &dyn ParentDataModifierNode) -> bool {
        false
    }
}

/// A node-local build modifier.
pub trait BuildModifierNode: Any {
    modifier_lifecycle!();
    /// Applies build-time effects to the current component node.
    fn apply(&self, _runtime: &mut TesseraRuntime) {}

    /// Expands a stateful modifier at attachment time.
    ///
    /// Construct ordinary modifier data here. The context binds node state and
    /// frame subscriptions to this capability in its owning node.
    fn materialize(&self, _context: &mut ModifierBuildContext) -> Option<Modifier> {
        None
    }

    /// Returns this node's nested scroll attachment, when present.
    fn nested_scroll_connection(&self) -> Option<crate::nested_scroll::NestedScrollConnection> {
        None
    }
}

/// Node-owned services available while attaching a stateful modifier.
///
/// Modifier constructors remain reusable values. Their state is allocated when
/// the chain attaches. State survives rebuilds of the same node and capability;
/// inserting unrelated modifier types does not change that ownership.
pub struct ModifierBuildContext {
    instance: u64,
    path: Vec<u64>,
    state_positions: SmallVec<[(TypeId, usize); 4]>,
}

fn next_type_position(positions: &mut SmallVec<[(TypeId, usize); 4]>, id: TypeId) -> usize {
    if let Some((_, next)) = positions.iter_mut().find(|(kind, _)| *kind == id) {
        let position = *next;
        *next = next.checked_add(1).expect("modifier position exhausted");
        position
    } else {
        positions.push((id, 1));
        0
    }
}

fn modifier_type_key(id: TypeId) -> u64 {
    let mut hasher = FxHasher::default();
    id.hash(&mut hasher);
    hasher.finish()
}

impl ModifierBuildContext {
    /// Gets persistent state by value type and occurrence within this capability.
    ///
    /// Calls of a different value type do not shift existing state. Multiple
    /// values of the same type follow their declaration order.
    pub fn state<T: 'static>(&mut self, init: impl FnOnce() -> T) -> crate::State<T> {
        let ordinal = next_type_position(&mut self.state_positions, TypeId::of::<T>());
        crate::runtime::modifier_state(self.instance, &self.path, ordinal, init)
    }

    /// Runs `future` on the UI session while this capability stays attached.
    ///
    /// A running task keeps running when the capability attaches again; a
    /// finished task is replaced by `future`. Detaching cancels the task.
    pub fn launch(&mut self, future: impl std::future::Future<Output = ()> + 'static) {
        let task = self.state(|| None::<crate::Task>);
        if task.with(|task| task.as_ref().is_none_or(crate::Task::is_finished)) {
            let previous =
                task.with_mut(|task| task.replace(crate::async_dispatch::spawn_local(future)));
            drop(previous);
        }
    }

    /// Gets the nearest nested scroll connection above the attached node.
    pub fn nested_scroll_parent(&self) -> Option<crate::nested_scroll::NestedScrollConnection> {
        crate::nested_scroll::NestedScrollConnection::nearest()
    }
}

/// A node-local semantics modifier.
pub trait SemanticsModifierNode: Any {
    modifier_lifecycle!();
    /// Applies accessibility metadata and optional action handling.
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    );
}

/// A node-local pointer input modifier.
pub trait PointerInputModifierNode: Any {
    modifier_lifecycle!();
    /// Handles pointer input for the current node.
    ///
    /// Pointer modifiers should operate on pointer-specific event flow and
    /// local interaction state. Cross-cutting concerns such as semantics,
    /// hover cursors, and IME publication should use their dedicated modifier
    /// or session APIs.
    fn on_pointer_input(&self, input: PointerInput<'_>);
}

/// A node-local hover cursor modifier.
pub trait CursorModifierNode: Any {
    modifier_lifecycle!();
    /// Returns the cursor icon that should be used when the pointer hovers this
    /// node.
    fn cursor_icon(&self) -> CursorIcon;
}

/// A node-local keyboard input modifier.
pub trait KeyboardInputModifierNode: Any {
    modifier_lifecycle!();
    /// Handles keyboard input for the current node.
    fn on_keyboard_input(&self, input: KeyboardInput<'_>);
}

/// A node-local IME input modifier.
pub trait ImeInputModifierNode: Any {
    modifier_lifecycle!();
    /// Handles IME input for the current node.
    fn on_ime_input(&self, input: ImeInput<'_>);
}

/// Low-level modifier primitive API for framework crates that build semantic
/// modifier extensions.
///
/// This trait is intended for crates such as `tessera-foundation` and
/// `tessera-components` that implement higher-level `Modifier` extensions. Most
/// application code should prefer those semantic extensions instead of calling
/// these methods directly.
pub trait ModifierCapabilityExt {
    /// Appends a layout modifier node to the current modifier chain.
    fn push_layout<N>(self, node: N) -> Self
    where
        N: LayoutModifier + PartialEq;

    /// Appends a placement modifier node to the current modifier chain.
    fn push_placement<N>(self, node: N) -> Self
    where
        N: PlacementModifierNode + PartialEq;

    /// Appends a draw modifier node to the current modifier chain.
    fn push_draw<N>(self, node: N) -> Self
    where
        N: DrawModifierNode;

    /// Appends a parent-data modifier node to the current modifier chain.
    fn push_parent_data<N>(self, node: N) -> Self
    where
        N: ParentDataModifierNode;

    /// Appends a build modifier node to the current modifier chain.
    fn push_build<N>(self, node: N) -> Self
    where
        N: BuildModifierNode;

    /// Appends a semantics modifier node to the current modifier chain.
    fn push_semantics<N>(self, node: N) -> Self
    where
        N: SemanticsModifierNode;

    /// Appends a preview pointer-input modifier node to the current modifier
    /// chain.
    fn push_pointer_preview_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode;

    /// Appends a pointer-input modifier node to the current modifier chain.
    fn push_pointer_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode;

    /// Appends a final pointer-input modifier node to the current modifier
    /// chain.
    fn push_pointer_final_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode;

    /// Appends a preview keyboard-input modifier node to the current modifier
    /// chain.
    fn push_keyboard_preview_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode;

    /// Appends a keyboard-input modifier node to the current modifier chain.
    fn push_keyboard_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode;

    /// Appends a clipboard-input modifier for focused content.
    fn push_clipboard_input<N>(self, node: N) -> Self
    where
        N: ClipboardInputModifierNode;

    /// Appends a preview IME-input modifier node to the current modifier chain.
    fn push_ime_preview_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode;

    /// Appends an IME-input modifier node to the current modifier chain.
    fn push_ime_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode;
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum FocusModifierRegistration {
    Target(Option<FocusNode>),
    Scope(Option<FocusScopeNode>),
    Group(Option<FocusGroupNode>),
    Restorer {
        scope: Option<FocusScopeNode>,
        fallback: Option<FocusRequester>,
    },
}

#[derive(Clone, PartialEq, Eq)]
enum FocusModifierOp {
    Requester(FocusRequester),
    Registration(FocusModifierRegistration),
    Properties(Rc<FocusProperties>),
    PreferredTabStop(bool),
    TraversalPolicy(FocusTraversalPolicy),
    ChangedHandler(CallbackWith<FocusState>),
    EventHandler(CallbackWith<FocusState>),
    BeyondBoundsHandler(CallbackWith<FocusDirection, bool>),
    RevealHandler(CallbackWith<FocusRevealRequest, bool>),
}

pub(crate) trait ErasedLayoutModifierNode: 'static {
    fn node(&self) -> &dyn LayoutModifier;

    fn eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn measure_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn placement_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn hash_node(&self, state: &mut dyn Hasher);

    fn as_any(&self) -> &dyn Any;
}

pub(crate) trait ErasedPlacementModifierNode: 'static {
    fn node(&self) -> &dyn PlacementModifierNode;

    fn eq(&self, other: &dyn ErasedPlacementModifierNode) -> bool;

    fn hash_node(&self, state: &mut dyn Hasher);

    fn as_any(&self) -> &dyn Any;
}

struct ComparableLayoutModifierNode<N>
where
    N: LayoutModifier + PartialEq,
{
    node: N,
}

impl<N> ErasedLayoutModifierNode for ComparableLayoutModifierNode<N>
where
    N: LayoutModifier + PartialEq,
{
    fn node(&self) -> &dyn LayoutModifier {
        &self.node
    }

    fn eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node == other.node)
    }

    fn measure_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.measure_eq(&other.node))
    }

    fn placement_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.placement_eq(&other.node))
    }

    fn hash_node(&self, state: &mut dyn Hasher) {
        // Hash by concrete node type. Equal nodes share a type and therefore a
        // hash, which is all the hash/equality contract requires.
        let mut concrete = std::collections::hash_map::DefaultHasher::new();
        TypeId::of::<N>().hash(&mut concrete);
        state.write_u64(concrete.finish());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct ComparablePlacementModifierNode<N>
where
    N: PlacementModifierNode + PartialEq,
{
    node: N,
}

impl<N> ErasedPlacementModifierNode for ComparablePlacementModifierNode<N>
where
    N: PlacementModifierNode + PartialEq,
{
    fn node(&self) -> &dyn PlacementModifierNode {
        &self.node
    }

    fn eq(&self, other: &dyn ErasedPlacementModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node == other.node)
    }

    fn hash_node(&self, state: &mut dyn Hasher) {
        // Hash by concrete node type. Equal nodes share a type and therefore a
        // hash, which is all the hash/equality contract requires.
        let mut concrete = std::collections::hash_map::DefaultHasher::new();
        TypeId::of::<N>().hash(&mut concrete);
        state.write_u64(concrete.finish());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

enum ModifierAction {
    Layout(NodeRef<dyn ErasedLayoutModifierNode>),
    Placement(NodeRef<dyn ErasedPlacementModifierNode>),
    Draw(NodeRef<dyn DrawModifierNode>),
    ParentData(NodeRef<dyn ParentDataModifierNode>),
    Build(NodeRef<dyn BuildModifierNode>),
    BuildOwner(Modifier),
    Semantics(NodeRef<dyn SemanticsModifierNode>),
    Cursor(NodeRef<dyn CursorModifierNode>),
    PointerPreviewInput(NodeRef<dyn PointerInputModifierNode>),
    PointerInput(NodeRef<dyn PointerInputModifierNode>),
    PointerFinalInput(NodeRef<dyn PointerInputModifierNode>),
    KeyboardPreviewInput(NodeRef<dyn KeyboardInputModifierNode>),
    KeyboardInput(NodeRef<dyn KeyboardInputModifierNode>),
    ClipboardInput(NodeRef<dyn ClipboardInputModifierNode>),
    ImePreviewInput(NodeRef<dyn ImeInputModifierNode>),
    ImeInput(NodeRef<dyn ImeInputModifierNode>),
    TerminalDraw(crate::layout::DrawModifierHandle),
    Focus(FocusModifierOp),
    GloballyPositioned(crate::component_tree::GloballyPositionedHandler),
}

impl ModifierAction {
    fn kind(&self) -> ActionKinds {
        match self {
            Self::Layout(_) => ActionKinds::LAYOUT,
            Self::Placement(_) => ActionKinds::PLACEMENT,
            Self::Draw(_) => ActionKinds::DRAW,
            Self::ParentData(_) => ActionKinds::PARENT_DATA,
            Self::Build(_) => ActionKinds::BUILD,
            Self::Semantics(_) => ActionKinds::SEMANTICS,
            Self::Cursor(_)
            | Self::PointerPreviewInput(_)
            | Self::PointerInput(_)
            | Self::PointerFinalInput(_) => ActionKinds::POINTER,
            Self::KeyboardPreviewInput(_)
            | Self::KeyboardInput(_)
            | Self::ImePreviewInput(_)
            | Self::ImeInput(_) => ActionKinds::KEY_INPUT,
            Self::ClipboardInput(_) => ActionKinds::CLIPBOARD,
            Self::Focus(_) => ActionKinds::FOCUS,
            Self::GloballyPositioned(_) => ActionKinds::POSITIONED,
            Self::BuildOwner(_) => ActionKinds::LIFECYCLE,
            Self::TerminalDraw(_) => ActionKinds::default(),
        }
    }
}

/// Action categories present in a modifier chain, combined when chains are
/// joined so capability queries and comparisons can skip absent categories.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct ActionKinds(u16);

impl ActionKinds {
    const LAYOUT: Self = Self(1);
    const PLACEMENT: Self = Self(1 << 1);
    const DRAW: Self = Self(1 << 2);
    const PARENT_DATA: Self = Self(1 << 3);
    const BUILD: Self = Self(1 << 4);
    const SEMANTICS: Self = Self(1 << 5);
    const POINTER: Self = Self(1 << 6);
    const KEY_INPUT: Self = Self(1 << 7);
    const CLIPBOARD: Self = Self(1 << 8);
    const FOCUS: Self = Self(1 << 9);
    const POSITIONED: Self = Self(1 << 10);
    const LIFECYCLE: Self = Self(1 << 11);
    const INPUT_GEOMETRY: Self = Self(Self::POINTER.0 | Self::KEY_INPUT.0);
    const ATTACH_EFFECTS: Self = Self(
        Self::BUILD.0 | Self::SEMANTICS.0 | Self::FOCUS.0 | Self::POSITIONED.0 | Self::LIFECYCLE.0,
    );
    const COMPARED: Self = Self(
        Self::LAYOUT.0 | Self::PLACEMENT.0 | Self::DRAW.0 | Self::PARENT_DATA.0 | Self::POINTER.0,
    );

    fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

#[derive(Clone, Copy)]
pub(crate) enum OrderedModifierAction<'a> {
    Layout,
    Placement(&'a dyn ErasedPlacementModifierNode),
    Draw(&'a dyn DrawModifierNode),
    ParentData,
    Cursor(&'a dyn CursorModifierNode),
    PointerPreviewInput(&'a dyn PointerInputModifierNode),
    PointerInput(&'a dyn PointerInputModifierNode),
    PointerFinalInput(&'a dyn PointerInputModifierNode),
    KeyboardPreviewInput(&'a dyn KeyboardInputModifierNode),
    KeyboardInput(&'a dyn KeyboardInputModifierNode),
    ClipboardInput(&'a dyn ClipboardInputModifierNode),
    ImePreviewInput(&'a dyn ImeInputModifierNode),
    ImeInput(&'a dyn ImeInputModifierNode),
}

/// A persistent handle to a modifier chain.
#[derive(Clone, Default)]
pub struct Modifier {
    element: ModifierElement,
}

/// Focus-specific modifier extensions for [`Modifier`].
pub trait FocusModifierExt {
    /// Registers a focus target for this subtree.
    fn focusable(self) -> Modifier;

    /// Binds an explicit focus requester to this subtree.
    fn focus_requester(self, requester: FocusRequester) -> Modifier;

    /// Applies explicit focus properties to this subtree.
    fn focus_properties(self, properties: FocusProperties) -> Modifier;

    /// Prefers this focus target when Tab enters a group with a single Tab
    /// stop, preserving its other focus properties.
    fn preferred_tab_stop(self, preferred: bool) -> Modifier;

    /// Registers a traversal-only focus group for this subtree.
    fn focus_group(self) -> Modifier;

    /// Registers an explicit focus scope handle for this subtree.
    fn focus_scope_with(self, scope: FocusScopeNode) -> Modifier;

    /// Registers an explicit focus group handle for this subtree.
    fn focus_group_with(self, group: FocusGroupNode) -> Modifier;

    /// Registers a focus scope with restore behavior for this subtree.
    fn focus_restorer(self, fallback: Option<FocusRequester>) -> Modifier;

    /// Registers an explicit focus restorer scope for this subtree.
    fn focus_restorer_with(
        self,
        scope: FocusScopeNode,
        fallback: Option<FocusRequester>,
    ) -> Modifier;

    /// Applies a traversal policy to the current focus group or scope.
    fn focus_traversal_policy(self, policy: FocusTraversalPolicy) -> Modifier;

    /// Registers a callback that runs when the subtree focus state changes.
    fn on_focus_changed<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusState>>;

    /// Registers a callback that observes focus events for this subtree.
    fn on_focus_event<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusState>>;

    /// Registers a callback that moves focus beyond the current viewport.
    fn focus_beyond_bounds_handler<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusDirection, bool>>;

    /// Registers a callback that reveals the focused target within the
    /// viewport.
    fn focus_reveal_handler<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusRevealRequest, bool>>;

    /// Registers a callback invoked with this node's window rectangle after
    /// attachment and whenever the rectangle changes.
    fn on_globally_positioned<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::PxRect>>;

    /// Observes this node's exact fractional rectangle in window coordinates.
    fn on_globally_positioned_precise<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::FloatRect>>;

    /// Observes the transformation from this node's local units to the window.
    fn on_local_to_window<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::LocalToWindow>>;

    /// Observes local layout dimensions and their exact transformation to the
    /// window.
    fn on_layout_coordinates<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::LayoutCoordinates>>;
}

/// Cursor-specific modifier extensions for [`Modifier`].
pub trait CursorModifierExt {
    /// Sets the cursor icon used while the pointer hovers this node.
    fn hover_cursor_icon(self, icon: CursorIcon) -> Modifier;
}

impl Modifier {
    fn with_terminal_draw(mut self, draw: crate::layout::DrawModifierHandle) -> Self {
        if draw.has_node() {
            self = self.push_action(ModifierAction::TerminalDraw(draw));
        }
        self
    }

    /// Observes measured dimensions in local layout units after initial
    /// placement and size changes. Moving or scaling an ancestor does not
    /// trigger a callback unless the local measured size changes. Hidden
    /// layouts resume observation when placed again.
    pub fn on_layout_size_changed<F>(self, handler: F) -> Self
    where
        F: Into<CallbackWith<crate::PxSize>>,
    {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::Size(handler.into()),
        ))
    }

    /// Observes the window-space bounds remaining after ancestor rectangular
    /// clipping and window boundaries. Reports `None` when fully clipped or
    /// when a previously placed layout becomes unplaced. Shape bounds are
    /// included; occlusion by other content is not considered.
    pub fn on_visible_window_bounds<F>(self, handler: F) -> Self
    where
        F: Into<CallbackWith<Option<crate::FloatRect>>>,
    {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::Visible(handler.into()),
        ))
    }

    /// Uses this layout's local units as a coordinate space for descendant
    /// [`Self::on_layout_rect_in_space`] observers, including this layout.
    /// Nested spaces take precedence over their ancestors.
    pub fn layout_coordinate_space(self) -> Self {
        self.push_build(LayoutCoordinateSpace)
    }

    /// Observes this layout's fractional bounds in the nearest declared
    /// [`Self::layout_coordinate_space`]. Without a declared space, observes
    /// window bounds. Calls after initial placement and when these bounds
    /// change; moving or scaling the entire space does not trigger a callback.
    /// Hidden layouts resume observation when placed again.
    pub fn on_layout_rect_in_space<F>(self, handler: F) -> Self
    where
        F: Into<CallbackWith<crate::FloatRect>>,
    {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::InSpace(handler.into()),
        ))
    }

    /// Creates an empty modifier chain.
    pub fn new() -> Self {
        Self::default()
    }

    fn push_action(self, action: ModifierAction) -> Self {
        Self {
            element: self.element.push(action),
        }
    }

    fn push_node<N: 'static, T: ?Sized>(
        self,
        node: N,
        erase: fn(&N) -> &T,
        action: fn(NodeRef<T>) -> ModifierAction,
    ) -> Self {
        Self {
            element: self.element.push_node(node, erase, action),
        }
    }

    /// Appends another modifier chain after this one without flattening it.
    pub fn then(self, other: Modifier) -> Self {
        Self {
            element: self.element.then(other.element),
        }
    }

    pub(crate) fn push_layout<N>(self, node: N) -> Self
    where
        N: LayoutModifier + PartialEq,
    {
        self.push_node(
            ComparableLayoutModifierNode { node },
            |node| node as &dyn ErasedLayoutModifierNode,
            ModifierAction::Layout,
        )
    }

    pub(crate) fn push_placement<N>(self, node: N) -> Self
    where
        N: PlacementModifierNode + PartialEq,
    {
        self.push_node(
            ComparablePlacementModifierNode { node },
            |node| node as &dyn ErasedPlacementModifierNode,
            ModifierAction::Placement,
        )
    }

    pub(crate) fn push_draw<N>(self, node: N) -> Self
    where
        N: DrawModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn DrawModifierNode,
            ModifierAction::Draw,
        )
    }

    pub(crate) fn push_parent_data<N>(self, node: N) -> Self
    where
        N: ParentDataModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn ParentDataModifierNode,
            ModifierAction::ParentData,
        )
    }

    pub(crate) fn push_build<N>(self, node: N) -> Self
    where
        N: BuildModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn BuildModifierNode,
            ModifierAction::Build,
        )
    }

    pub(crate) fn push_semantics<N>(self, node: N) -> Self
    where
        N: SemanticsModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn SemanticsModifierNode,
            ModifierAction::Semantics,
        )
    }

    fn push_cursor<N>(self, node: N) -> Self
    where
        N: CursorModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn CursorModifierNode,
            ModifierAction::Cursor,
        )
    }

    pub(crate) fn push_pointer_preview_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn PointerInputModifierNode,
            ModifierAction::PointerPreviewInput,
        )
    }

    pub(crate) fn push_pointer_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn PointerInputModifierNode,
            ModifierAction::PointerInput,
        )
    }

    pub(crate) fn push_pointer_final_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn PointerInputModifierNode,
            ModifierAction::PointerFinalInput,
        )
    }

    pub(crate) fn push_keyboard_preview_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn KeyboardInputModifierNode,
            ModifierAction::KeyboardPreviewInput,
        )
    }

    pub(crate) fn push_keyboard_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn KeyboardInputModifierNode,
            ModifierAction::KeyboardInput,
        )
    }

    pub(crate) fn push_clipboard_input<N>(self, node: N) -> Self
    where
        N: ClipboardInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn ClipboardInputModifierNode,
            ModifierAction::ClipboardInput,
        )
    }

    pub(crate) fn push_ime_preview_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn ImeInputModifierNode,
            ModifierAction::ImePreviewInput,
        )
    }

    pub(crate) fn push_ime_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode,
    {
        self.push_node(
            node,
            |node| node as &dyn ImeInputModifierNode,
            ModifierAction::ImeInput,
        )
    }

    fn push_focus_requester(self, requester: FocusRequester) -> Self {
        self.push_focus_op(FocusModifierOp::Requester(requester))
    }

    fn push_focus_target(self) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Target(None),
        ))
    }

    fn push_focus_scope_with(self, scope: FocusScopeNode) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Scope(Some(scope)),
        ))
    }

    fn push_focus_group(self) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Group(None),
        ))
    }

    fn push_focus_group_with(self, group: FocusGroupNode) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Group(Some(group)),
        ))
    }

    fn push_focus_restorer(self, fallback: Option<FocusRequester>) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Restorer {
                scope: None,
                fallback,
            },
        ))
    }

    fn push_focus_restorer_with(
        self,
        scope: FocusScopeNode,
        fallback: Option<FocusRequester>,
    ) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Restorer {
                scope: Some(scope),
                fallback,
            },
        ))
    }

    fn push_focus_properties(self, properties: FocusProperties) -> Self {
        self.push_focus_op(FocusModifierOp::Properties(Rc::new(properties)))
    }

    fn push_focus_traversal_policy(self, policy: FocusTraversalPolicy) -> Self {
        self.push_focus_op(FocusModifierOp::TraversalPolicy(policy))
    }

    fn push_focus_changed_handler(self, handler: CallbackWith<FocusState>) -> Self {
        self.push_focus_op(FocusModifierOp::ChangedHandler(handler))
    }

    fn push_focus_event_handler(self, handler: CallbackWith<FocusState>) -> Self {
        self.push_focus_op(FocusModifierOp::EventHandler(handler))
    }

    fn push_focus_beyond_bounds_handler(self, handler: CallbackWith<FocusDirection, bool>) -> Self {
        self.push_focus_op(FocusModifierOp::BeyondBoundsHandler(handler))
    }

    fn push_focus_reveal_handler(self, handler: CallbackWith<FocusRevealRequest, bool>) -> Self {
        self.push_focus_op(FocusModifierOp::RevealHandler(handler))
    }

    fn push_globally_positioned(self, handler: CallbackWith<crate::PxRect>) -> Self {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::Pixels(handler),
        ))
    }

    fn push_focus_op(self, op: FocusModifierOp) -> Self {
        self.push_action(ModifierAction::Focus(op))
    }

    /// Attaches a nested scroll connection to this node.
    pub fn nested_scroll(self, connection: crate::nested_scroll::NestedScrollConnection) -> Self {
        self.push_build(connection)
    }

    pub(crate) fn nested_scroll_connection(
        &self,
    ) -> Option<crate::nested_scroll::NestedScrollConnection> {
        if !self.contains(ActionKinds::BUILD) {
            return None;
        }
        let mut connection = None;
        self.element.for_each_action_outer_first(&mut |action| {
            if let ModifierAction::Build(node) = action {
                if let Some(attached) = node.nested_scroll_connection() {
                    connection = Some(attached);
                }
            }
        });
        connection
    }

    fn contains(&self, kinds: ActionKinds) -> bool {
        self.element.kinds().intersects(kinds)
    }

    fn materialize(self, instance: u64, owner: u64, path: &[u64]) -> Self {
        if !self.contains(ActionKinds::BUILD) {
            return self;
        }
        assert!(path.len() < 64, "recursive modifier materialization");
        let mut positions = SmallVec::new();
        let mut result = Modifier::new();
        let mut keep_start = 0;
        let mut changed = false;
        for (index, action) in self.element.actions().enumerate() {
            let ModifierAction::Build(node) = action else {
                continue;
            };
            let kind = node.as_ref().type_id();
            let position = next_type_position(&mut positions, kind);
            let mut path = path.to_vec();
            path.push(modifier_type_key(kind));
            path.push(position as u64);
            let mut context = ModifierBuildContext {
                instance,
                path: path.clone(),
                state_positions: SmallVec::new(),
            };
            if let Some(modifier) = node.materialize(&mut context) {
                result = result.then(Modifier {
                    element: self.element.slice(keep_start, index - keep_start),
                });
                result = result.push_action(ModifierAction::BuildOwner(Modifier {
                    element: self.element.slice(index, 1),
                }));
                result = result.then(modifier.materialize(instance, owner, &path));
                keep_start = index + 1;
                changed = true;
            }
        }
        if !changed {
            return self;
        }
        result.then(Modifier {
            element: self
                .element
                .slice(keep_start, self.element.len() - keep_start),
        })
    }
    /// Attaches this modifier chain to the current component node.
    pub fn attach(self) {
        ensure_build_phase();

        let (instance, owner) = TesseraRuntime::with(|runtime| {
            let tree = &runtime.component_tree;
            let node = tree
                .executing_call_node()
                .filter(|&id| tree.current_build_node_id() == Some(id))
                .and_then(|id| tree.get(id))
                .expect(
                    "modifiers attach to a node emitted by the current component; \
                     emit one with `layout(Stack)` or `group()`",
                );
            (node.instance_logic_id, node.instance_key)
        });
        let materialized =
            self.materialize(instance, owner, &crate::runtime::modifier_attachment_path());
        let effects = materialized.clone();

        TesseraRuntime::with_mut(|runtime| {
            runtime.append_current_modifier(materialized);
        });
        crate::nested_scroll::attachment_changed();

        let mut accessibility = AccessibilityNode::new();
        let mut action_handler = None;
        let mut has_semantics = false;
        if effects.contains(ActionKinds::ATTACH_EFFECTS) {
            effects.element.fold_actions(&mut |effect| match effect {
                ModifierAction::Build(node) => {
                    TesseraRuntime::with_mut(|runtime| node.apply(runtime));
                }
                ModifierAction::Semantics(node) => {
                    has_semantics = true;
                    node.apply(&mut accessibility, &mut action_handler);
                }
                ModifierAction::Focus(op) => {
                    TesseraRuntime::with_mut(|runtime| {
                        apply_focus_op(runtime::FocusModifierRuntime::new(runtime), op.clone());
                    });
                }
                ModifierAction::GloballyPositioned(handler) => {
                    TesseraRuntime::with_mut(|runtime| {
                        runtime.push_current_globally_positioned_handler(handler.clone());
                    });
                }
                _ => {}
            });
        }

        TesseraRuntime::with_mut(|runtime| {
            runtime
                .set_current_accessibility(has_semantics.then_some(accessibility), action_handler);
        });
    }

    /// Attaches this modifier chain to the current node and then runs `child`.
    pub fn run<F>(self, child: F)
    where
        F: Fn() + 'static,
    {
        self.attach();
        child();
    }

    /// Returns true when the modifier has no actions.
    pub fn is_empty(&self) -> bool {
        self.element.is_empty()
    }

    pub(crate) fn ordered_actions(&self) -> SmallVec<[OrderedModifierAction<'_>; 8]> {
        let mut actions = SmallVec::new();
        self.element.fold_actions(&mut |action| {
            let action = match action {
                ModifierAction::Layout(_) => OrderedModifierAction::Layout,
                ModifierAction::Placement(node) => OrderedModifierAction::Placement(node.as_ref()),
                ModifierAction::Draw(node) => OrderedModifierAction::Draw(node.as_ref()),
                ModifierAction::ParentData(_) => OrderedModifierAction::ParentData,
                ModifierAction::Cursor(node) => OrderedModifierAction::Cursor(node.as_ref()),
                ModifierAction::PointerPreviewInput(node) => {
                    OrderedModifierAction::PointerPreviewInput(node.as_ref())
                }
                ModifierAction::PointerInput(node) => {
                    OrderedModifierAction::PointerInput(node.as_ref())
                }
                ModifierAction::PointerFinalInput(node) => {
                    OrderedModifierAction::PointerFinalInput(node.as_ref())
                }
                ModifierAction::KeyboardPreviewInput(node) => {
                    OrderedModifierAction::KeyboardPreviewInput(node.as_ref())
                }
                ModifierAction::KeyboardInput(node) => {
                    OrderedModifierAction::KeyboardInput(node.as_ref())
                }
                ModifierAction::ClipboardInput(node) => {
                    OrderedModifierAction::ClipboardInput(node.as_ref())
                }
                ModifierAction::ImePreviewInput(node) => {
                    OrderedModifierAction::ImePreviewInput(node.as_ref())
                }
                ModifierAction::ImeInput(node) => OrderedModifierAction::ImeInput(node.as_ref()),
                ModifierAction::Build(_)
                | ModifierAction::Semantics(_)
                | ModifierAction::Focus(_)
                | ModifierAction::GloballyPositioned(_)
                | ModifierAction::BuildOwner(_)
                | ModifierAction::TerminalDraw(_) => return,
            };
            actions.push(action);
        });
        actions
    }

    pub(crate) fn input_geometry_eq(&self, other: &Self) -> bool {
        !self.changes_from(other).input_geometry
    }

    /// Compares this chain with the previously attached one in a single walk.
    pub(crate) fn changes_from(&self, previous: &Self) -> ModifierChanges {
        let kinds = self.element.kinds().union(previous.element.kinds());
        if !kinds.intersects(ActionKinds::COMPARED) || self.element.is_same(&previous.element) {
            return ModifierChanges::default();
        }
        fn collect(modifier: &Modifier) -> SmallVec<[&ModifierAction; 8]> {
            let mut actions = SmallVec::new();
            modifier
                .element
                .fold_actions(&mut |action| actions.push(action));
            actions
        }
        fn select<'a, 'b>(
            actions: &'b [&'a ModifierAction],
            kind: ActionKinds,
        ) -> impl Iterator<Item = &'a ModifierAction> + Clone + 'b {
            actions
                .iter()
                .copied()
                .filter(move |action| action.kind().intersects(kind))
        }
        let (current, previous) = (collect(self), collect(previous));
        let mut changes = ModifierChanges::default();
        if kinds.intersects(ActionKinds::LAYOUT) {
            let (lhs, rhs) = (
                select(&current, ActionKinds::LAYOUT),
                select(&previous, ActionKinds::LAYOUT),
            );
            if lhs.clone().count() != rhs.clone().count() {
                changes.measure = true;
                changes.placement = true;
            }
            for (lhs, rhs) in lhs.zip(rhs) {
                if let (ModifierAction::Layout(lhs), ModifierAction::Layout(rhs)) = (lhs, rhs) {
                    changes.measure |= !lhs.measure_eq(rhs.as_ref());
                    changes.placement |= !lhs.placement_eq(rhs.as_ref());
                }
            }
        }
        if kinds.intersects(ActionKinds::PLACEMENT) {
            let (lhs, rhs) = (
                select(&current, ActionKinds::PLACEMENT),
                select(&previous, ActionKinds::PLACEMENT),
            );
            changes.transform = lhs.clone().count() != rhs.clone().count()
                || lhs.zip(rhs).any(|pair| match pair {
                    (ModifierAction::Placement(lhs), ModifierAction::Placement(rhs)) => {
                        !lhs.eq(rhs.as_ref())
                    }
                    _ => true,
                });
        }
        if kinds.intersects(ActionKinds::DRAW) {
            let (lhs, rhs) = (
                select(&current, ActionKinds::DRAW),
                select(&previous, ActionKinds::DRAW),
            );
            changes.draw = lhs.clone().count() != rhs.clone().count()
                || lhs.zip(rhs).any(|pair| match pair {
                    (ModifierAction::Draw(lhs), ModifierAction::Draw(rhs)) => {
                        !NodeRef::ptr_eq(lhs, rhs)
                    }
                    _ => true,
                });
        }
        if kinds.intersects(ActionKinds::PARENT_DATA) {
            let (lhs, rhs) = (
                select(&current, ActionKinds::PARENT_DATA),
                select(&previous, ActionKinds::PARENT_DATA),
            );
            changes.parent_data = lhs.clone().count() != rhs.clone().count()
                || lhs.zip(rhs).any(|pair| match pair {
                    (ModifierAction::ParentData(lhs), ModifierAction::ParentData(rhs)) => {
                        !NodeRef::ptr_eq(lhs, rhs) && !lhs.equivalent(rhs.as_ref())
                    }
                    _ => true,
                });
        }
        let pointer = |actions: &[&ModifierAction]| {
            actions
                .iter()
                .any(|action| action.kind().intersects(ActionKinds::POINTER))
        };
        let (current_pointer, previous_pointer) = (pointer(&current), pointer(&previous));
        if current_pointer || previous_pointer {
            let shape = |actions: &[&ModifierAction]| {
                actions
                    .iter()
                    .filter(|action| !action.kind().intersects(ActionKinds::ATTACH_EFFECTS))
                    .map(|action| std::mem::discriminant(*action))
                    .collect::<SmallVec<[_; 8]>>()
            };
            changes.input_geometry =
                current_pointer != previous_pointer || shape(&current) != shape(&previous);
        }
        changes
    }

    pub(crate) fn has_pointer_input(&self) -> bool {
        self.contains(ActionKinds::POINTER)
    }

    pub(crate) fn has_input_geometry(&self) -> bool {
        self.contains(ActionKinds::INPUT_GEOMETRY)
    }

    pub(crate) fn resolve_placement(
        &self,
        mut position: PxPosition,
        instance: u64,
    ) -> (PxPosition, Vec<PxPosition>) {
        let pointer = self.has_pointer_input();
        let mut positions = Vec::new();
        if !pointer && !self.contains(ActionKinds::PLACEMENT) {
            return (position, positions);
        }
        self.element.fold_actions(&mut |action| {
            if let ModifierAction::Placement(node) = action {
                crate::work::record(
                    instance,
                    crate::work::WorkPhase::Transform,
                    node.node().diagnostic_name(),
                );
                position = node.node().transform_position(position);
            }
            if pointer && !action.kind().intersects(ActionKinds::ATTACH_EFFECTS) {
                positions.push(position);
            }
        });
        (position, positions)
    }

    fn select_actions<'a, T>(
        &'a self,
        mut select: impl FnMut(&'a ModifierAction) -> Option<T>,
    ) -> SmallVec<[T; 4]> {
        let mut selected = SmallVec::new();
        self.element.fold_actions(&mut |action| {
            if let Some(value) = select(action) {
                selected.push(value);
            }
        });
        selected
    }

    pub(crate) fn layout_modifiers(&self) -> SmallVec<[&dyn LayoutModifier; 4]> {
        if !self.contains(ActionKinds::LAYOUT) {
            return SmallVec::new();
        }
        self.select_actions(|action| match action {
            ModifierAction::Layout(node) => Some(node.node()),
            _ => None,
        })
    }

    pub(crate) fn parent_data<T: Clone + 'static>(&self) -> Option<T> {
        if !self.contains(ActionKinds::PARENT_DATA) {
            return None;
        }
        let mut data = ParentDataMap::default();
        self.element.fold_actions(&mut |action| {
            if let ModifierAction::ParentData(node) = action {
                node.apply_parent_data(&mut data);
            }
        });
        data.get(&TypeId::of::<T>())?.downcast_ref::<T>().cloned()
    }
}

/// Node behavior categories that differ between two attached modifier chains.
#[derive(Clone, Copy, Default)]
pub(crate) struct ModifierChanges {
    pub(crate) measure: bool,
    pub(crate) placement: bool,
    pub(crate) transform: bool,
    pub(crate) draw: bool,
    pub(crate) parent_data: bool,
    pub(crate) input_geometry: bool,
}

impl ModifierCapabilityExt for Modifier {
    fn push_clipboard_input<N>(self, node: N) -> Self
    where
        N: ClipboardInputModifierNode,
    {
        Modifier::push_clipboard_input(self, node)
    }
    fn push_layout<N>(self, node: N) -> Self
    where
        N: LayoutModifier + PartialEq,
    {
        Modifier::push_layout(self, node)
    }

    fn push_placement<N>(self, node: N) -> Self
    where
        N: PlacementModifierNode + PartialEq,
    {
        Modifier::push_placement(self, node)
    }

    fn push_draw<N>(self, node: N) -> Self
    where
        N: DrawModifierNode,
    {
        Modifier::push_draw(self, node)
    }

    fn push_parent_data<N>(self, node: N) -> Self
    where
        N: ParentDataModifierNode,
    {
        Modifier::push_parent_data(self, node)
    }

    fn push_build<N>(self, node: N) -> Self
    where
        N: BuildModifierNode,
    {
        Modifier::push_build(self, node)
    }

    fn push_semantics<N>(self, node: N) -> Self
    where
        N: SemanticsModifierNode,
    {
        Modifier::push_semantics(self, node)
    }

    fn push_pointer_preview_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        Modifier::push_pointer_preview_input(self, node)
    }

    fn push_pointer_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        Modifier::push_pointer_input(self, node)
    }

    fn push_pointer_final_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        Modifier::push_pointer_final_input(self, node)
    }

    fn push_keyboard_preview_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode,
    {
        Modifier::push_keyboard_preview_input(self, node)
    }

    fn push_keyboard_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode,
    {
        Modifier::push_keyboard_input(self, node)
    }

    fn push_ime_preview_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode,
    {
        Modifier::push_ime_preview_input(self, node)
    }

    fn push_ime_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode,
    {
        Modifier::push_ime_input(self, node)
    }
}

#[derive(Clone, Copy)]
struct StaticCursorModifierNode {
    icon: CursorIcon,
}

impl CursorModifierNode for StaticCursorModifierNode {
    fn cursor_icon(&self) -> CursorIcon {
        self.icon
    }
}

impl CursorModifierExt for Modifier {
    fn hover_cursor_icon(self, icon: CursorIcon) -> Modifier {
        self.push_cursor(StaticCursorModifierNode { icon })
    }
}

impl FocusModifierExt for Modifier {
    fn focusable(self) -> Modifier {
        self.push_focus_target()
    }

    fn focus_requester(self, requester: FocusRequester) -> Modifier {
        self.push_focus_requester(requester)
    }

    fn focus_properties(self, properties: FocusProperties) -> Modifier {
        self.push_focus_properties(properties)
    }

    fn preferred_tab_stop(self, preferred: bool) -> Modifier {
        self.push_focus_op(FocusModifierOp::PreferredTabStop(preferred))
    }

    fn focus_group(self) -> Modifier {
        self.push_focus_group()
    }

    fn focus_scope_with(self, scope: FocusScopeNode) -> Modifier {
        self.push_focus_scope_with(scope)
    }

    fn focus_group_with(self, group: FocusGroupNode) -> Modifier {
        self.push_focus_group_with(group)
    }

    fn focus_restorer(self, fallback: Option<FocusRequester>) -> Modifier {
        self.push_focus_restorer(fallback)
    }

    fn focus_restorer_with(
        self,
        scope: FocusScopeNode,
        fallback: Option<FocusRequester>,
    ) -> Modifier {
        self.push_focus_restorer_with(scope, fallback)
    }

    fn focus_traversal_policy(self, policy: FocusTraversalPolicy) -> Modifier {
        self.push_focus_traversal_policy(policy)
    }

    fn on_focus_changed<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusState>>,
    {
        self.push_focus_changed_handler(handler.into())
    }

    fn on_focus_event<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusState>>,
    {
        self.push_focus_event_handler(handler.into())
    }

    fn focus_beyond_bounds_handler<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusDirection, bool>>,
    {
        self.push_focus_beyond_bounds_handler(handler.into())
    }

    fn focus_reveal_handler<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<FocusRevealRequest, bool>>,
    {
        self.push_focus_reveal_handler(handler.into())
    }

    fn on_globally_positioned<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::PxRect>>,
    {
        self.push_globally_positioned(handler.into())
    }

    fn on_globally_positioned_precise<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::FloatRect>>,
    {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::Precise(handler.into()),
        ))
    }

    fn on_local_to_window<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::LocalToWindow>>,
    {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::Transform(handler.into()),
        ))
    }

    fn on_layout_coordinates<F>(self, handler: F) -> Modifier
    where
        F: Into<CallbackWith<crate::LayoutCoordinates>>,
    {
        self.push_action(ModifierAction::GloballyPositioned(
            crate::component_tree::GloballyPositionedHandler::Coordinates(handler.into()),
        ))
    }
}

impl fmt::Debug for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Modifier")
            .field("is_empty", &self.element.is_empty())
            .finish()
    }
}

fn modifier_action_eq(lhs: &ModifierAction, rhs: &ModifierAction) -> bool {
    match (lhs, rhs) {
        (ModifierAction::TerminalDraw(lhs), ModifierAction::TerminalDraw(rhs)) => lhs == rhs,
        (ModifierAction::BuildOwner(lhs), ModifierAction::BuildOwner(rhs)) => lhs == rhs,
        (ModifierAction::Layout(lhs), ModifierAction::Layout(rhs)) => lhs.as_ref().eq(rhs.as_ref()),
        (ModifierAction::Placement(lhs), ModifierAction::Placement(rhs)) => lhs.eq(rhs.as_ref()),
        (ModifierAction::Draw(lhs), ModifierAction::Draw(rhs)) => NodeRef::ptr_eq(lhs, rhs),
        (ModifierAction::ParentData(lhs), ModifierAction::ParentData(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::Build(lhs), ModifierAction::Build(rhs)) => NodeRef::ptr_eq(lhs, rhs),
        (ModifierAction::Semantics(lhs), ModifierAction::Semantics(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::Cursor(lhs), ModifierAction::Cursor(rhs)) => NodeRef::ptr_eq(lhs, rhs),
        (ModifierAction::PointerPreviewInput(lhs), ModifierAction::PointerPreviewInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::PointerInput(lhs), ModifierAction::PointerInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::PointerFinalInput(lhs), ModifierAction::PointerFinalInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::KeyboardPreviewInput(lhs), ModifierAction::KeyboardPreviewInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::KeyboardInput(lhs), ModifierAction::KeyboardInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::ClipboardInput(lhs), ModifierAction::ClipboardInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::ImePreviewInput(lhs), ModifierAction::ImePreviewInput(rhs)) => {
            NodeRef::ptr_eq(lhs, rhs)
        }
        (ModifierAction::ImeInput(lhs), ModifierAction::ImeInput(rhs)) => NodeRef::ptr_eq(lhs, rhs),
        (ModifierAction::Focus(lhs), ModifierAction::Focus(rhs)) => lhs == rhs,
        (ModifierAction::GloballyPositioned(lhs), ModifierAction::GloballyPositioned(rhs)) => {
            lhs == rhs
        }
        _ => false,
    }
}

struct LayoutCoordinateSpace;

impl BuildModifierNode for LayoutCoordinateSpace {
    fn apply(&self, runtime: &mut TesseraRuntime) {
        if let Some(node) = runtime.component_tree.current_node_mut() {
            node.layout_coordinate_space = true;
        }
    }
}

fn hash_rc<T: ?Sized, H: Hasher>(value: &NodeRef<T>, state: &mut H) {
    std::ptr::hash(value.as_ptr(), state);
}

fn hash_modifier_action<H: Hasher>(action: &ModifierAction, state: &mut H) {
    std::mem::discriminant(action).hash(state);
    match action {
        ModifierAction::TerminalDraw(node) => node.concrete_type_id().hash(state),
        ModifierAction::BuildOwner(node) => node.hash(state),
        ModifierAction::Layout(node) => node.as_ref().hash_node(state),
        ModifierAction::Placement(node) => node.hash_node(state),
        ModifierAction::Draw(node) => hash_rc(node, state),
        ModifierAction::ParentData(node) => hash_rc(node, state),
        ModifierAction::Build(node) => hash_rc(node, state),
        ModifierAction::Semantics(node) => hash_rc(node, state),
        ModifierAction::Cursor(node) => hash_rc(node, state),
        ModifierAction::PointerPreviewInput(node) => hash_rc(node, state),
        ModifierAction::PointerInput(node) => hash_rc(node, state),
        ModifierAction::PointerFinalInput(node) => hash_rc(node, state),
        ModifierAction::KeyboardPreviewInput(node) => hash_rc(node, state),
        ModifierAction::KeyboardInput(node) => hash_rc(node, state),
        ModifierAction::ClipboardInput(node) => hash_rc(node, state),
        ModifierAction::ImePreviewInput(node) => hash_rc(node, state),
        ModifierAction::ImeInput(node) => hash_rc(node, state),
        ModifierAction::Focus(op) => hash_focus_op(op, state),
        ModifierAction::GloballyPositioned(handler) => {
            std::ptr::hash(
                handler as *const crate::component_tree::GloballyPositionedHandler,
                state,
            );
        }
    }
}

fn hash_focus_op<H: Hasher>(op: &FocusModifierOp, state: &mut H) {
    std::mem::discriminant(op).hash(state);
    match op {
        FocusModifierOp::Requester(value) => value.hash(state),
        FocusModifierOp::Registration(FocusModifierRegistration::Target(node)) => {
            0u8.hash(state);
            node.map(|node| node.handle_id()).hash(state);
        }
        FocusModifierOp::Registration(FocusModifierRegistration::Scope(node)) => {
            1u8.hash(state);
            node.map(|node| node.handle_id()).hash(state);
        }
        FocusModifierOp::Registration(FocusModifierRegistration::Group(node)) => {
            2u8.hash(state);
            node.map(|node| node.handle_id()).hash(state);
        }
        FocusModifierOp::Registration(FocusModifierRegistration::Restorer { scope, fallback }) => {
            3u8.hash(state);
            scope.map(|node| node.handle_id()).hash(state);
            fallback.hash(state);
        }
        FocusModifierOp::Properties(value) => {
            value.can_focus.hash(state);
            value.can_request_focus.hash(state);
            value.skip_traversal.hash(state);
            value.preferred_tab_stop.hash(state);
            value.next.hash(state);
            value.previous.hash(state);
            value.up.hash(state);
            value.down.hash(state);
            value.left.hash(state);
            value.right.hash(state);
            value.enter.hash(state);
            value.exit.hash(state);
        }
        FocusModifierOp::TraversalPolicy(value) => value.hash(state),
        FocusModifierOp::PreferredTabStop(value) => value.hash(state),
        FocusModifierOp::ChangedHandler(value) => hash_callback_bits(value, state),
        FocusModifierOp::EventHandler(value) => hash_callback_bits(value, state),
        FocusModifierOp::BeyondBoundsHandler(value) => hash_callback_bits(value, state),
        FocusModifierOp::RevealHandler(value) => hash_callback_bits(value, state),
    }
}

fn hash_callback_bits<T, R, H: Hasher>(callback: &CallbackWith<T, R>, state: &mut H) {
    let bytes = unsafe {
        std::slice::from_raw_parts(
            (callback as *const CallbackWith<T, R>).cast::<u8>(),
            std::mem::size_of::<CallbackWith<T, R>>(),
        )
    };
    bytes.hash(state);
}

impl PartialEq for Modifier {
    fn eq(&self, other: &Self) -> bool {
        if self.element.kinds() != other.element.kinds() {
            return false;
        }
        let mut lhs = self.element.actions();
        let mut rhs = other.element.actions();
        loop {
            match (lhs.next(), rhs.next()) {
                (Some(lhs), Some(rhs)) if modifier_action_eq(lhs, rhs) => {}
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

impl Eq for Modifier {}

impl Hash for Modifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut count = 0usize;
        self.element.fold_actions(&mut |action| {
            hash_modifier_action(action, state);
            count += 1;
        });
        count.hash(state);
    }
}

mod runtime {
    use super::FocusModifierRegistration;
    use crate::{
        FocusProperties, FocusRequester, FocusScopeNode, FocusState, FocusTraversalPolicy,
        focus::{FocusDirection, FocusRevealRequest},
        prop::CallbackWith,
        runtime::TesseraRuntime,
    };

    pub(super) struct FocusModifierRuntime<'a> {
        runtime: &'a mut TesseraRuntime,
    }

    impl<'a> FocusModifierRuntime<'a> {
        pub(super) fn new(runtime: &'a mut TesseraRuntime) -> Self {
            Self { runtime }
        }

        pub(super) fn bind_requester(&mut self, requester: FocusRequester) {
            self.runtime.bind_current_focus_requester(requester);
        }

        pub(super) fn ensure_registration(&mut self, registration: FocusModifierRegistration) {
            match registration {
                FocusModifierRegistration::Target(node) => {
                    let node = node.unwrap_or_else(|| {
                        self.runtime
                            .current_focus_target_handle()
                            .unwrap_or_else(|| {
                                crate::runtime::persistent_focus_target_for_current_instance(
                                    "__tessera_focus_target",
                                )
                            })
                    });
                    self.runtime.ensure_current_focus_target(node);
                }
                FocusModifierRegistration::Scope(scope) => {
                    let scope = scope.unwrap_or_else(|| {
                        self.runtime
                            .current_focus_scope_handle()
                            .unwrap_or_else(|| {
                                crate::runtime::persistent_focus_scope_for_current_instance(
                                    "__tessera_focus_scope",
                                )
                            })
                    });
                    self.runtime.ensure_current_focus_scope(scope);
                }
                FocusModifierRegistration::Group(group) => {
                    let group = group.unwrap_or_else(|| {
                        self.runtime
                            .current_focus_group_handle()
                            .unwrap_or_else(|| {
                                crate::runtime::persistent_focus_group_for_current_instance(
                                    "__tessera_focus_group",
                                )
                            })
                    });
                    self.runtime.ensure_current_focus_group(group);
                }
                FocusModifierRegistration::Restorer { scope, fallback } => {
                    let scope: FocusScopeNode = scope.unwrap_or_else(|| {
                        self.runtime
                            .current_focus_scope_handle()
                            .unwrap_or_else(|| {
                                crate::runtime::persistent_focus_scope_for_current_instance(
                                    "__tessera_focus_scope",
                                )
                            })
                    });
                    self.runtime.ensure_current_focus_scope(scope);
                    if let Some(fallback) = fallback {
                        self.runtime.set_current_focus_restorer_fallback(fallback);
                    }
                }
            }
        }

        pub(super) fn set_properties(&mut self, properties: FocusProperties) {
            self.runtime.set_current_focus_properties(properties);
        }

        pub(super) fn set_preferred_tab_stop(&mut self, preferred: bool) {
            self.runtime.set_current_preferred_tab_stop(preferred);
        }

        pub(super) fn set_traversal_policy(&mut self, policy: FocusTraversalPolicy) {
            self.runtime.set_current_focus_traversal_policy(policy);
        }

        pub(super) fn set_changed_handler(&mut self, handler: CallbackWith<FocusState>) {
            self.runtime.set_current_focus_changed_handler(handler);
        }

        pub(super) fn set_event_handler(&mut self, handler: CallbackWith<FocusState>) {
            self.runtime.set_current_focus_event_handler(handler);
        }

        pub(super) fn set_beyond_bounds_handler(
            &mut self,
            handler: CallbackWith<FocusDirection, bool>,
        ) {
            self.runtime
                .set_current_focus_beyond_bounds_handler(handler);
        }

        pub(super) fn set_reveal_handler(
            &mut self,
            handler: CallbackWith<FocusRevealRequest, bool>,
        ) {
            self.runtime.set_current_focus_reveal_handler(handler);
        }
    }
}

fn apply_focus_op(runtime: runtime::FocusModifierRuntime<'_>, op: FocusModifierOp) {
    let mut runtime = runtime;
    match op {
        FocusModifierOp::Requester(requester) => runtime.bind_requester(requester),
        FocusModifierOp::Registration(registration) => runtime.ensure_registration(registration),
        FocusModifierOp::Properties(properties) => runtime.set_properties(*properties),
        FocusModifierOp::PreferredTabStop(preferred) => runtime.set_preferred_tab_stop(preferred),
        FocusModifierOp::TraversalPolicy(policy) => runtime.set_traversal_policy(policy),
        FocusModifierOp::ChangedHandler(handler) => runtime.set_changed_handler(handler),
        FocusModifierOp::EventHandler(handler) => runtime.set_event_handler(handler),
        FocusModifierOp::BeyondBoundsHandler(handler) => runtime.set_beyond_bounds_handler(handler),
        FocusModifierOp::RevealHandler(handler) => runtime.set_reveal_handler(handler),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::TypeId,
        cell::Cell,
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        rc::Rc,
    };

    use super::{
        BuildModifierNode, DrawModifierContent, DrawModifierContext, DrawModifierNode,
        FocusModifierOp, Modifier, ModifierAction, ModifierBuildContext, ParentDataMap,
        ParentDataModifierNode,
    };

    fn flag(value: bool) -> Modifier {
        Modifier::new().push_action(ModifierAction::Focus(FocusModifierOp::PreferredTabStop(
            value,
        )))
    }

    #[test]
    fn modifier_equality_flattens_different_groupings_and_checks_all_actions() {
        let left = flag(true).then(flag(false)).then(flag(true));
        let right = flag(true).then(flag(false).then(flag(true)));
        assert_eq!(left, right);

        let mut left_hash = DefaultHasher::new();
        let mut right_hash = DefaultHasher::new();
        left.hash(&mut left_hash);
        right.hash(&mut right_hash);
        assert_eq!(left_hash.finish(), right_hash.finish());

        assert_ne!(left, flag(true).then(flag(true)).then(flag(true)));
        assert_ne!(left, right.then(flag(false)));
    }

    #[test]
    fn draw_modifier_changes_are_reported_without_layout_changes() {
        let old = Modifier::new().push_draw(crate::layout::NoopDrawModifier);
        let same = old.clone();
        let next = Modifier::new().push_draw(crate::layout::NoopDrawModifier);
        assert!(!same.changes_from(&old).draw);
        let changes = next.changes_from(&old);
        assert!(changes.draw);
        assert!(!changes.measure);
        assert!(!changes.transform);
    }

    #[derive(Clone, Copy, PartialEq)]
    struct Width(u32);

    impl crate::modifier::LayoutModifier for Width {
        fn measure(
            &self,
            _: &crate::LayoutModifierInput<'_>,
            _: &mut dyn crate::LayoutModifierChild,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            unreachable!("modifier identity fixtures are not measured")
        }
    }

    #[derive(Clone)]
    struct LocalLayout(Rc<Cell<u32>>);

    impl PartialEq for LocalLayout {
        fn eq(&self, other: &Self) -> bool {
            self.0.get() == other.0.get()
        }
    }

    impl crate::modifier::LayoutModifier for LocalLayout {
        fn measure(
            &self,
            _: &crate::LayoutModifierInput<'_>,
            _: &mut dyn crate::LayoutModifierChild,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            unreachable!("modifier identity fixtures are not measured")
        }
    }

    struct LocalDraw(Rc<Cell<u32>>);

    impl DrawModifierNode for LocalDraw {
        fn draw(
            &self,
            ctx: &mut DrawModifierContext<'_, '_>,
            content: &mut dyn DrawModifierContent,
        ) {
            assert_eq!(self.0.get(), 24);
            content.draw(ctx.render_input);
        }
    }

    #[test]
    fn session_local_nodes_can_be_shared_and_released_without_send_sync() {
        let state = Rc::new(Cell::new(24));
        let modifier = Modifier::new()
            .push_layout(LocalLayout(state.clone()))
            .push_draw(LocalDraw(state.clone()));
        let shared = modifier.clone();
        assert!(!shared.changes_from(&modifier).measure);
        assert!(!shared.changes_from(&modifier).draw);
        assert_eq!(Rc::strong_count(&state), 3);
        drop(modifier);
        drop(shared);
        assert_eq!(Rc::strong_count(&state), 1);
    }

    #[derive(Clone)]
    struct LocalParent(Rc<Cell<u32>>);

    impl ParentDataModifierNode for LocalParent {
        fn apply_parent_data(&self, data: &mut ParentDataMap) {
            data.insert(TypeId::of::<Self>(), Box::new(self.clone()));
        }
    }

    #[test]
    fn parent_data_accepts_session_local_values() {
        let state = Rc::new(Cell::new(7));
        let modifier = Modifier::new().push_parent_data(LocalParent(state.clone()));
        let payload = modifier
            .parent_data::<LocalParent>()
            .expect("local parent data");
        assert_eq!(payload.0.get(), 7);
        drop(payload);
        drop(modifier);
        assert_eq!(Rc::strong_count(&state), 1);
    }

    #[test]
    fn layout_values_are_compared_across_new_chains() {
        let first = Modifier::new().push_layout(Width(24));
        let second = Modifier::new().push_layout(Width(24));
        assert_eq!(first, second);
        let mut first_hash = DefaultHasher::new();
        let mut second_hash = DefaultHasher::new();
        first.hash(&mut first_hash);
        second.hash(&mut second_hash);
        assert_eq!(first_hash.finish(), second_hash.finish());
        assert!(!first.changes_from(&second).measure);
        assert!(
            Modifier::new()
                .push_layout(Width(25))
                .changes_from(&first)
                .measure
        );
    }

    #[test]
    fn draw_actions_reuse_identity_only_for_clones_and_keep_order() {
        let original = Modifier::new()
            .push_layout(Width(24))
            .push_draw(crate::layout::NoopDrawModifier);
        let cloned = original.clone();
        let recreated = Modifier::new()
            .push_layout(Width(24))
            .push_draw(crate::layout::NoopDrawModifier);
        assert!(!cloned.changes_from(&original).draw);
        assert!(recreated.changes_from(&original).draw);
        assert!(!recreated.changes_from(&original).measure);
        assert_eq!(original.ordered_actions().len(), 2);
        assert!(matches!(
            original.ordered_actions()[0],
            super::OrderedModifierAction::Layout
        ));
        assert!(matches!(
            original.ordered_actions()[1],
            super::OrderedModifierAction::Draw(_)
        ));
        assert_ne!(original, recreated);
    }

    struct ExpandWidth;

    impl BuildModifierNode for ExpandWidth {
        fn materialize(&self, _: &mut ModifierBuildContext) -> Option<Modifier> {
            Some(Modifier::new().push_layout(Width(42)))
        }
    }

    #[test]
    fn two_action_materialization_retains_the_original_order() {
        let color = Modifier::new().push_draw(crate::layout::NoopDrawModifier);
        let before = Modifier::new().push_build(ExpandWidth).then(color.clone());
        let after = color.then(Modifier::new().push_build(ExpandWidth));
        let before = before.materialize(1, 1, &[]);
        let after = after.materialize(1, 1, &[]);
        let before = before.ordered_actions();
        let after = after.ordered_actions();
        assert!(matches!(
            before.as_slice(),
            [
                super::OrderedModifierAction::Layout,
                super::OrderedModifierAction::Draw(_)
            ]
        ));
        assert!(matches!(
            after.as_slice(),
            [
                super::OrderedModifierAction::Draw(_),
                super::OrderedModifierAction::Layout
            ]
        ));
    }
}
