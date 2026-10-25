//! Modifier chains for node-local layout, drawing, focus, semantics, and other
//! node-scoped behavior.
//!
//! ## Usage
//!
//! Build reusable modifier chains that attach behavior directly to the current
//! component node.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

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
pub type ParentDataMap = HashMap<TypeId, Arc<dyn Any + Send + Sync>>;

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

/// A node-local layout modifier.
///
/// This is the single layout measurement contract shared by the innermost
/// layout node (the measure policy applied at the innermost layer) and by
/// wrapping layout modifiers. A node overrides either [`Self::measure`] (pure
/// measure, applied at the innermost layer) or [`Self::measure_content`]
/// (wrapping measure). An innermost node may override [`Self::place_children`]
/// to update placements against cached child sizes. Wrapping modifiers can be
/// measured again around that cached result to update their content bounds.
pub trait LayoutModifierNode: Send + Sync + 'static {
    /// Computes layout for this node when it is applied at the innermost layer.
    fn measure(&self, _scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Err(MeasurementError::MeasureFnFailed(
            "layout node does not implement pure measure".to_string(),
        ))
    }

    /// Measures and places the wrapped content.
    fn measure_content(
        &self,
        _input: &LayoutModifierInput<'_>,
        _child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        Err(MeasurementError::MeasureFnFailed(
            "layout node does not implement wrapped measure".to_string(),
        ))
    }

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

    /// Recomputes child placements from cached child sizes.
    ///
    /// An innermost layout node derives placements from the cached child sizes
    /// in `scope`. Returning `None` means it cannot place without re-measuring.
    /// Wrapping modifiers use [`Self::measure_content`] around the cached
    /// result so drawing receives the same content bounds as child
    /// placement.
    fn place_children(
        &self,
        _scope: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        None
    }
}

/// A node-local placement modifier that transforms the current node position
/// without affecting measured size.
pub trait PlacementModifierNode: Send + Sync + 'static {
    /// Returns the transformed node position relative to the parent.
    fn transform_position(&self, position: PxPosition) -> PxPosition;
}

/// A node-local draw modifier.
pub trait DrawModifierNode: Send + Sync + 'static {
    /// Records drawing behavior around the wrapped content.
    ///
    /// Drawing uses the content bounds at this position in the modifier chain.
    /// For example, a background before padding includes that padding, while a
    /// background after padding covers only the inner content.
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent);
}

/// A node-local parent-data modifier.
pub trait ParentDataModifierNode: Any + Send + Sync + 'static {
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
pub trait BuildModifierNode: Send + Sync + 'static {
    /// Applies build-time effects to the current component node.
    fn apply(&self, runtime: &mut TesseraRuntime);
}

/// A node-local semantics modifier.
pub trait SemanticsModifierNode: Send + Sync + 'static {
    /// Applies accessibility metadata and optional action handling.
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    );
}

/// A node-local pointer input modifier.
pub trait PointerInputModifierNode: Send + Sync + 'static {
    /// Handles pointer input for the current node.
    ///
    /// Pointer modifiers should operate on pointer-specific event flow and
    /// local interaction state. Cross-cutting concerns such as semantics,
    /// hover cursors, and IME publication should use their dedicated modifier
    /// or session APIs.
    fn on_pointer_input(&self, input: PointerInput<'_>);
}

/// A node-local hover cursor modifier.
pub trait CursorModifierNode: Send + Sync + 'static {
    /// Returns the cursor icon that should be used when the pointer hovers this
    /// node.
    fn cursor_icon(&self) -> CursorIcon;
}

/// A node-local keyboard input modifier.
pub trait KeyboardInputModifierNode: Send + Sync + 'static {
    /// Handles keyboard input for the current node.
    fn on_keyboard_input(&self, input: KeyboardInput<'_>);
}

/// A node-local IME input modifier.
pub trait ImeInputModifierNode: Send + Sync + 'static {
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
        N: LayoutModifierNode + PartialEq;

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
    Properties(FocusProperties),
    PreferredTabStop(bool),
    TraversalPolicy(FocusTraversalPolicy),
    ChangedHandler(CallbackWith<FocusState>),
    EventHandler(CallbackWith<FocusState>),
    BeyondBoundsHandler(CallbackWith<FocusDirection, bool>),
    RevealHandler(CallbackWith<FocusRevealRequest, bool>),
}

pub(crate) trait ErasedLayoutModifierNode: Send + Sync + 'static {
    fn node(&self) -> Arc<dyn LayoutModifierNode>;

    fn eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn measure_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn placement_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn hash_node(&self, state: &mut dyn Hasher);

    fn as_any(&self) -> &dyn Any;
}

pub(crate) trait ErasedPlacementModifierNode: Send + Sync + 'static {
    fn node(&self) -> Arc<dyn PlacementModifierNode>;

    fn eq(&self, other: &dyn ErasedPlacementModifierNode) -> bool;

    fn hash_node(&self, state: &mut dyn Hasher);

    fn as_any(&self) -> &dyn Any;
}

struct ComparableLayoutModifierNode<N>
where
    N: LayoutModifierNode + PartialEq,
{
    node: Arc<N>,
}

impl<N> ErasedLayoutModifierNode for ComparableLayoutModifierNode<N>
where
    N: LayoutModifierNode + PartialEq,
{
    fn node(&self) -> Arc<dyn LayoutModifierNode> {
        self.node.clone()
    }

    fn eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.as_ref() == other.node.as_ref())
    }

    fn measure_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.measure_eq(other.node.as_ref()))
    }

    fn placement_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.placement_eq(other.node.as_ref()))
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
    node: Arc<N>,
}

impl<N> ErasedPlacementModifierNode for ComparablePlacementModifierNode<N>
where
    N: PlacementModifierNode + PartialEq,
{
    fn node(&self) -> Arc<dyn PlacementModifierNode> {
        self.node.clone()
    }

    fn eq(&self, other: &dyn ErasedPlacementModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.as_ref() == other.node.as_ref())
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

#[derive(Clone)]
enum ModifierAction {
    Layout(Arc<dyn ErasedLayoutModifierNode>),
    Placement(Arc<dyn ErasedPlacementModifierNode>),
    Draw(Arc<dyn DrawModifierNode>),
    ParentData(Arc<dyn ParentDataModifierNode>),
    Build(Arc<dyn BuildModifierNode>),
    Semantics(Arc<dyn SemanticsModifierNode>),
    Cursor(Arc<dyn CursorModifierNode>),
    PointerPreviewInput(Arc<dyn PointerInputModifierNode>),
    PointerInput(Arc<dyn PointerInputModifierNode>),
    PointerFinalInput(Arc<dyn PointerInputModifierNode>),
    KeyboardPreviewInput(Arc<dyn KeyboardInputModifierNode>),
    KeyboardInput(Arc<dyn KeyboardInputModifierNode>),
    ClipboardInput(Arc<dyn ClipboardInputModifierNode>),
    ImePreviewInput(Arc<dyn ImeInputModifierNode>),
    ImeInput(Arc<dyn ImeInputModifierNode>),
    Focus(FocusModifierOp),
    GloballyPositioned(crate::component_tree::GloballyPositionedHandler),
}

#[derive(Clone)]
pub(crate) enum OrderedModifierAction {
    Layout(Arc<dyn ErasedLayoutModifierNode>),
    Placement(Arc<dyn ErasedPlacementModifierNode>),
    Draw(Arc<dyn DrawModifierNode>),
    ParentData(Arc<dyn ParentDataModifierNode>),
    Cursor(Arc<dyn CursorModifierNode>),
    PointerPreviewInput(Arc<dyn PointerInputModifierNode>),
    PointerInput(Arc<dyn PointerInputModifierNode>),
    PointerFinalInput(Arc<dyn PointerInputModifierNode>),
    KeyboardPreviewInput(Arc<dyn KeyboardInputModifierNode>),
    KeyboardInput(Arc<dyn KeyboardInputModifierNode>),
    ClipboardInput(Arc<dyn ClipboardInputModifierNode>),
    ImePreviewInput(Arc<dyn ImeInputModifierNode>),
    ImeInput(Arc<dyn ImeInputModifierNode>),
}

/// Compose-style modifier element.
///
/// `Combined` keeps the outer chain and the inner chain. `then` does not
/// flatten or rebuild individual links. Equality and hashing walk the folded
/// action sequence, so an empty side disappears: `a.then(empty) == a`.
#[derive(Clone)]
enum ModifierElement {
    Empty,
    Action(ModifierAction),
    Combined {
        outer: Box<ModifierElement>,
        inner: Box<ModifierElement>,
    },
}

impl Default for ModifierElement {
    fn default() -> Self {
        Self::Empty
    }
}

impl ModifierElement {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    fn then(self, inner: Self) -> Self {
        match (self, inner) {
            (lhs, Self::Empty) => lhs,
            (Self::Empty, rhs) => rhs,
            (outer, inner) => Self::Combined {
                outer: Box::new(outer),
                inner: Box::new(inner),
            },
        }
    }

    fn fold_actions<F>(&self, visit: &mut F)
    where
        F: FnMut(&ModifierAction),
    {
        match self {
            Self::Empty => {}
            Self::Action(action) => visit(action),
            Self::Combined { outer, inner } => {
                outer.fold_actions(visit);
                inner.fold_actions(visit);
            }
        }
    }

    fn for_each_action_outer_first<F>(&self, visit: &mut F)
    where
        F: FnMut(ModifierAction),
    {
        match self {
            Self::Empty => {}
            Self::Action(action) => visit(action.clone()),
            Self::Combined { outer, inner } => {
                inner.for_each_action_outer_first(visit);
                outer.for_each_action_outer_first(visit);
            }
        }
    }
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
    /// Creates an empty modifier chain.
    pub fn new() -> Self {
        ensure_build_phase();
        Self::default()
    }

    fn push_action(self, action: ModifierAction) -> Self {
        ensure_build_phase();
        Self {
            element: self.element.then(ModifierElement::Action(action)),
        }
    }

    /// Appends another modifier chain after this one without flattening it.
    pub fn then(self, other: Modifier) -> Self {
        ensure_build_phase();
        Self {
            element: self.element.then(other.element),
        }
    }

    pub(crate) fn push_layout<N>(self, node: N) -> Self
    where
        N: LayoutModifierNode + PartialEq,
    {
        self.push_action(ModifierAction::Layout(Arc::new(
            ComparableLayoutModifierNode {
                node: Arc::new(node),
            },
        )))
    }

    pub(crate) fn push_placement<N>(self, node: N) -> Self
    where
        N: PlacementModifierNode + PartialEq,
    {
        self.push_action(ModifierAction::Placement(Arc::new(
            ComparablePlacementModifierNode {
                node: Arc::new(node),
            },
        )))
    }

    pub(crate) fn push_draw<N>(self, node: N) -> Self
    where
        N: DrawModifierNode,
    {
        self.push_action(ModifierAction::Draw(Arc::new(node)))
    }

    pub(crate) fn push_parent_data<N>(self, node: N) -> Self
    where
        N: ParentDataModifierNode,
    {
        self.push_action(ModifierAction::ParentData(Arc::new(node)))
    }

    pub(crate) fn push_build<N>(self, node: N) -> Self
    where
        N: BuildModifierNode,
    {
        self.push_action(ModifierAction::Build(Arc::new(node)))
    }

    pub(crate) fn push_semantics<N>(self, node: N) -> Self
    where
        N: SemanticsModifierNode,
    {
        self.push_action(ModifierAction::Semantics(Arc::new(node)))
    }

    fn push_cursor<N>(self, node: N) -> Self
    where
        N: CursorModifierNode,
    {
        self.push_action(ModifierAction::Cursor(Arc::new(node)))
    }

    pub(crate) fn push_pointer_preview_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        self.push_action(ModifierAction::PointerPreviewInput(Arc::new(node)))
    }

    pub(crate) fn push_pointer_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        self.push_action(ModifierAction::PointerInput(Arc::new(node)))
    }

    pub(crate) fn push_pointer_final_input<N>(self, node: N) -> Self
    where
        N: PointerInputModifierNode,
    {
        self.push_action(ModifierAction::PointerFinalInput(Arc::new(node)))
    }

    pub(crate) fn push_keyboard_preview_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode,
    {
        self.push_action(ModifierAction::KeyboardPreviewInput(Arc::new(node)))
    }

    pub(crate) fn push_keyboard_input<N>(self, node: N) -> Self
    where
        N: KeyboardInputModifierNode,
    {
        self.push_action(ModifierAction::KeyboardInput(Arc::new(node)))
    }

    pub(crate) fn push_clipboard_input<N>(self, node: N) -> Self
    where
        N: ClipboardInputModifierNode,
    {
        self.push_action(ModifierAction::ClipboardInput(Arc::new(node)))
    }

    pub(crate) fn push_ime_preview_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode,
    {
        self.push_action(ModifierAction::ImePreviewInput(Arc::new(node)))
    }

    pub(crate) fn push_ime_input<N>(self, node: N) -> Self
    where
        N: ImeInputModifierNode,
    {
        self.push_action(ModifierAction::ImeInput(Arc::new(node)))
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
        self.push_focus_op(FocusModifierOp::Properties(properties))
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

    /// Attaches this modifier chain to the current component node.
    pub fn attach(self) {
        ensure_build_phase();

        let mut actions = Vec::new();
        self.element.for_each_action_outer_first(&mut |action| {
            actions.push(action);
        });
        TesseraRuntime::with_mut(|runtime| {
            runtime.append_current_modifier(self.clone());
        });

        let mut accessibility = AccessibilityNode::new();
        let mut action_handler = None;
        let mut has_semantics = false;
        for action in actions.into_iter().rev() {
            match action {
                ModifierAction::Build(node) => {
                    TesseraRuntime::with_mut(|runtime| node.apply(runtime));
                }
                ModifierAction::Semantics(node) => {
                    has_semantics = true;
                    node.apply(&mut accessibility, &mut action_handler);
                }
                ModifierAction::Focus(op) => {
                    TesseraRuntime::with_mut(|runtime| {
                        apply_focus_op(runtime::FocusModifierRuntime::new(runtime), op);
                    });
                }
                ModifierAction::GloballyPositioned(handler) => {
                    TesseraRuntime::with_mut(|runtime| {
                        runtime.push_current_globally_positioned_handler(handler);
                    });
                }
                _ => {}
            }
        }

        TesseraRuntime::with_mut(|runtime| {
            runtime.set_current_accessibility(has_semantics.then_some(accessibility));
            runtime.set_current_accessibility_action_handler(action_handler);
        });
    }

    /// Attaches this modifier chain to the current node and then runs `child`.
    pub fn run<F>(self, child: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.attach();
        child();
    }

    /// Returns true when the modifier has no actions.
    pub fn is_empty(&self) -> bool {
        self.element.is_empty()
    }

    pub(crate) fn ordered_actions(&self) -> Vec<OrderedModifierAction> {
        let mut actions = Vec::new();
        self.element.fold_actions(&mut |action| {
            actions.push(action.clone());
        });
        actions
            .into_iter()
            .filter_map(|action| match action {
                ModifierAction::Layout(node) => Some(OrderedModifierAction::Layout(node)),
                ModifierAction::Placement(node) => Some(OrderedModifierAction::Placement(node)),
                ModifierAction::Draw(node) => Some(OrderedModifierAction::Draw(node)),
                ModifierAction::ParentData(node) => Some(OrderedModifierAction::ParentData(node)),
                ModifierAction::Cursor(node) => Some(OrderedModifierAction::Cursor(node)),
                ModifierAction::PointerPreviewInput(node) => {
                    Some(OrderedModifierAction::PointerPreviewInput(node))
                }
                ModifierAction::PointerInput(node) => {
                    Some(OrderedModifierAction::PointerInput(node))
                }
                ModifierAction::PointerFinalInput(node) => {
                    Some(OrderedModifierAction::PointerFinalInput(node))
                }
                ModifierAction::KeyboardPreviewInput(node) => {
                    Some(OrderedModifierAction::KeyboardPreviewInput(node))
                }
                ModifierAction::KeyboardInput(node) => {
                    Some(OrderedModifierAction::KeyboardInput(node))
                }
                ModifierAction::ClipboardInput(node) => {
                    Some(OrderedModifierAction::ClipboardInput(node))
                }
                ModifierAction::ImePreviewInput(node) => {
                    Some(OrderedModifierAction::ImePreviewInput(node))
                }
                ModifierAction::ImeInput(node) => Some(OrderedModifierAction::ImeInput(node)),
                ModifierAction::Build(_)
                | ModifierAction::Semantics(_)
                | ModifierAction::Focus(_)
                | ModifierAction::GloballyPositioned(_) => None,
            })
            .collect()
    }

    pub(crate) fn input_geometry_eq(&self, other: &Self) -> bool {
        let shape = |modifier: &Self| {
            let mut pointer = false;
            let mut actions = Vec::new();
            modifier.element.fold_actions(&mut |action| {
                pointer |= matches!(
                    action,
                    ModifierAction::Cursor(_)
                        | ModifierAction::PointerPreviewInput(_)
                        | ModifierAction::PointerInput(_)
                        | ModifierAction::PointerFinalInput(_)
                );
                if !matches!(
                    action,
                    ModifierAction::Build(_)
                        | ModifierAction::Semantics(_)
                        | ModifierAction::Focus(_)
                        | ModifierAction::GloballyPositioned(_)
                ) {
                    actions.push(std::mem::discriminant(action));
                }
            });
            (pointer, actions)
        };
        let (left_pointer, left) = shape(self);
        let (right_pointer, right) = shape(other);
        (!left_pointer && !right_pointer) || (left_pointer == right_pointer && left == right)
    }

    pub(crate) fn has_pointer_input(&self) -> bool {
        let mut pointer = false;
        self.element.fold_actions(&mut |action| {
            pointer |= matches!(
                action,
                ModifierAction::Cursor(_)
                    | ModifierAction::PointerPreviewInput(_)
                    | ModifierAction::PointerInput(_)
                    | ModifierAction::PointerFinalInput(_)
            );
        });
        pointer
    }

    pub(crate) fn resolve_placement(
        &self,
        mut position: PxPosition,
    ) -> (PxPosition, Vec<PxPosition>) {
        let pointer = self.has_pointer_input();
        let mut positions = Vec::new();
        self.element.fold_actions(&mut |action| {
            if let ModifierAction::Placement(node) = action {
                position = node.node().transform_position(position);
            }
            if pointer
                && !matches!(
                    action,
                    ModifierAction::Build(_)
                        | ModifierAction::Semantics(_)
                        | ModifierAction::Focus(_)
                        | ModifierAction::GloballyPositioned(_)
                )
            {
                positions.push(position);
            }
        });
        (position, positions)
    }

    pub(crate) fn layout_measure_eq(&self, other: &Self) -> bool {
        let lhs: Vec<_> = self
            .ordered_actions()
            .into_iter()
            .filter_map(|action| match action {
                OrderedModifierAction::Layout(node) => Some(node),
                _ => None,
            })
            .collect();
        let rhs: Vec<_> = other
            .ordered_actions()
            .into_iter()
            .filter_map(|action| match action {
                OrderedModifierAction::Layout(node) => Some(node),
                _ => None,
            })
            .collect();
        lhs.len() == rhs.len()
            && lhs
                .iter()
                .zip(rhs.iter())
                .all(|(lhs, rhs)| lhs.measure_eq(rhs.as_ref()))
    }

    pub(crate) fn layout_placement_eq(&self, other: &Self) -> bool {
        let nodes = |modifier: &Self| {
            modifier
                .ordered_actions()
                .into_iter()
                .filter_map(|action| match action {
                    OrderedModifierAction::Layout(node) => Some(node),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let lhs = nodes(self);
        let rhs = nodes(other);
        lhs.len() == rhs.len()
            && lhs
                .iter()
                .zip(&rhs)
                .all(|(lhs, rhs)| lhs.placement_eq(rhs.as_ref()))
    }

    pub(crate) fn position_eq(&self, other: &Self) -> bool {
        let nodes = |modifier: &Self| {
            modifier
                .ordered_actions()
                .into_iter()
                .filter_map(|action| match action {
                    OrderedModifierAction::Placement(node) => Some(node),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let lhs = nodes(self);
        let rhs = nodes(other);
        lhs.len() == rhs.len() && lhs.iter().zip(&rhs).all(|(lhs, rhs)| lhs.eq(rhs.as_ref()))
    }

    pub(crate) fn parent_data_eq(&self, other: &Self) -> bool {
        let nodes = |modifier: &Self| {
            modifier
                .ordered_actions()
                .into_iter()
                .filter_map(|action| match action {
                    OrderedModifierAction::ParentData(node) => Some(node),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let lhs = nodes(self);
        let rhs = nodes(other);
        lhs.len() == rhs.len()
            && lhs
                .iter()
                .zip(&rhs)
                .all(|(lhs, rhs)| Arc::ptr_eq(lhs, rhs) || lhs.equivalent(rhs.as_ref()))
    }
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
        N: LayoutModifierNode + PartialEq,
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
        (ModifierAction::Layout(lhs), ModifierAction::Layout(rhs)) => lhs.eq(rhs.as_ref()),
        (ModifierAction::Placement(lhs), ModifierAction::Placement(rhs)) => lhs.eq(rhs.as_ref()),
        (ModifierAction::Draw(lhs), ModifierAction::Draw(rhs)) => Arc::ptr_eq(lhs, rhs),
        (ModifierAction::ParentData(lhs), ModifierAction::ParentData(rhs)) => Arc::ptr_eq(lhs, rhs),
        (ModifierAction::Build(lhs), ModifierAction::Build(rhs)) => Arc::ptr_eq(lhs, rhs),
        (ModifierAction::Semantics(lhs), ModifierAction::Semantics(rhs)) => Arc::ptr_eq(lhs, rhs),
        (ModifierAction::Cursor(lhs), ModifierAction::Cursor(rhs)) => Arc::ptr_eq(lhs, rhs),
        (ModifierAction::PointerPreviewInput(lhs), ModifierAction::PointerPreviewInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::PointerInput(lhs), ModifierAction::PointerInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::PointerFinalInput(lhs), ModifierAction::PointerFinalInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::KeyboardPreviewInput(lhs), ModifierAction::KeyboardPreviewInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::KeyboardInput(lhs), ModifierAction::KeyboardInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::ClipboardInput(lhs), ModifierAction::ClipboardInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::ImePreviewInput(lhs), ModifierAction::ImePreviewInput(rhs)) => {
            Arc::ptr_eq(lhs, rhs)
        }
        (ModifierAction::ImeInput(lhs), ModifierAction::ImeInput(rhs)) => Arc::ptr_eq(lhs, rhs),
        (ModifierAction::Focus(lhs), ModifierAction::Focus(rhs)) => lhs == rhs,
        (ModifierAction::GloballyPositioned(lhs), ModifierAction::GloballyPositioned(rhs)) => {
            lhs == rhs
        }
        _ => false,
    }
}

fn hash_arc<T: ?Sized, H: Hasher>(value: &Arc<T>, state: &mut H) {
    std::ptr::hash(Arc::as_ptr(value), state);
}

fn hash_modifier_action<H: Hasher>(action: &ModifierAction, state: &mut H) {
    std::mem::discriminant(action).hash(state);
    match action {
        ModifierAction::Layout(node) => node.hash_node(state),
        ModifierAction::Placement(node) => node.hash_node(state),
        ModifierAction::Draw(node) => hash_arc(node, state),
        ModifierAction::ParentData(node) => hash_arc(node, state),
        ModifierAction::Build(node) => hash_arc(node, state),
        ModifierAction::Semantics(node) => hash_arc(node, state),
        ModifierAction::Cursor(node) => hash_arc(node, state),
        ModifierAction::PointerPreviewInput(node) => hash_arc(node, state),
        ModifierAction::PointerInput(node) => hash_arc(node, state),
        ModifierAction::PointerFinalInput(node) => hash_arc(node, state),
        ModifierAction::KeyboardPreviewInput(node) => hash_arc(node, state),
        ModifierAction::KeyboardInput(node) => hash_arc(node, state),
        ModifierAction::ClipboardInput(node) => hash_arc(node, state),
        ModifierAction::ImePreviewInput(node) => hash_arc(node, state),
        ModifierAction::ImeInput(node) => hash_arc(node, state),
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
        let mut lhs_len = 0usize;
        self.element.fold_actions(&mut |_| lhs_len += 1);
        let mut rhs_len = 0usize;
        other.element.fold_actions(&mut |_| rhs_len += 1);
        if lhs_len != rhs_len {
            return false;
        }
        let mut index = 0usize;
        let mut equal = true;
        self.element.fold_actions(&mut |lhs| {
            if !equal {
                return;
            }
            let mut seen = 0usize;
            other.element.fold_actions(&mut |rhs| {
                if seen == index {
                    equal &= modifier_action_eq(lhs, rhs);
                }
                seen += 1;
            });
            index += 1;
        });
        equal
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
        FocusModifierOp::Properties(properties) => runtime.set_properties(properties),
        FocusModifierOp::PreferredTabStop(preferred) => runtime.set_preferred_tab_stop(preferred),
        FocusModifierOp::TraversalPolicy(policy) => runtime.set_traversal_policy(policy),
        FocusModifierOp::ChangedHandler(handler) => runtime.set_changed_handler(handler),
        FocusModifierOp::EventHandler(handler) => runtime.set_event_handler(handler),
        FocusModifierOp::BeyondBoundsHandler(handler) => runtime.set_beyond_bounds_handler(handler),
        FocusModifierOp::RevealHandler(handler) => runtime.set_reveal_handler(handler),
    }
}
