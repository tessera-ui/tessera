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

use smallvec::SmallVec;

use crate::{
    AccessibilityActionHandler, AccessibilityNode, ComputedData, Constraint, FocusGroupNode,
    FocusProperties, FocusRequester, FocusScopeNode, FocusState, FocusTraversalPolicy, ImeInput,
    KeyboardInput, MeasurementError, PointerInput, PxPosition,
    focus::{FocusDirection, FocusNode, FocusRevealRequest},
    layout::{LayoutInput, RenderInput},
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

/// Output produced by a layout modifier node.
pub struct LayoutModifierOutput {
    /// Final computed size for the current node.
    pub size: ComputedData,
}

/// Draw continuation used by draw modifier nodes.
pub trait DrawModifierContent {
    /// Records the wrapped content.
    fn draw(&mut self, input: &RenderInput<'_>);
}

/// Draw input passed to draw modifier nodes.
pub struct DrawModifierContext<'a> {
    /// The render input for the current node.
    pub render_input: &'a RenderInput<'a>,
}

/// A node-local layout modifier.
pub trait LayoutModifierNode: Send + Sync + 'static {
    /// Measures and places the wrapped content.
    fn measure(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutModifierOutput, MeasurementError>;
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
    fn draw(&self, ctx: &mut DrawModifierContext<'_>, content: &mut dyn DrawModifierContent);
}

/// A node-local parent-data modifier.
pub trait ParentDataModifierNode: Send + Sync + 'static {
    /// Applies parent data visible to the parent layout.
    fn apply_parent_data(&self, data: &mut ParentDataMap);
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
    TraversalPolicy(FocusTraversalPolicy),
    ChangedHandler(CallbackWith<FocusState>),
    EventHandler(CallbackWith<FocusState>),
    BeyondBoundsHandler(CallbackWith<FocusDirection, bool>),
    RevealHandler(CallbackWith<FocusRevealRequest, bool>),
}

pub(crate) trait ErasedLayoutModifierNode: Send + Sync + 'static {
    fn node(&self) -> Arc<dyn LayoutModifierNode>;

    fn measure_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn placement_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool;

    fn as_any(&self) -> &dyn Any;
}

pub(crate) trait ErasedPlacementModifierNode: Send + Sync + 'static {
    fn node(&self) -> Arc<dyn PlacementModifierNode>;

    fn eq(&self, other: &dyn ErasedPlacementModifierNode) -> bool;

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

    fn measure_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.node.as_ref() == other.node.as_ref())
    }

    fn placement_eq(&self, other: &dyn ErasedLayoutModifierNode) -> bool {
        self.measure_eq(other)
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
    ImePreviewInput(Arc<dyn ImeInputModifierNode>),
    ImeInput(Arc<dyn ImeInputModifierNode>),
    Focus(FocusModifierOp),
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
    ImePreviewInput(Arc<dyn ImeInputModifierNode>),
    ImeInput(Arc<dyn ImeInputModifierNode>),
}

#[derive(Clone)]
struct ModifierLink {
    prev: Option<Arc<ModifierLink>>,
    action: ModifierAction,
}

fn collect_actions(mut node: Option<Arc<ModifierLink>>) -> Vec<ModifierAction> {
    let mut actions: SmallVec<[ModifierAction; 8]> = SmallVec::new();
    while let Some(current) = node {
        actions.push(current.action.clone());
        node = current.prev.clone();
    }
    actions.into_vec()
}

fn collect_actions_in_source_order(node: Option<Arc<ModifierLink>>) -> Vec<ModifierAction> {
    let mut actions = collect_actions(node);
    actions.reverse();
    actions
}

/// A persistent handle to a modifier chain.
#[derive(Clone, Default)]
pub struct Modifier {
    tail: Option<Arc<ModifierLink>>,
}

/// Focus-specific modifier extensions for [`Modifier`].
pub trait FocusModifierExt {
    /// Registers a focus target for this subtree.
    fn focusable(self) -> Modifier;

    /// Binds an explicit focus requester to this subtree.
    fn focus_requester(self, requester: FocusRequester) -> Modifier;

    /// Applies explicit focus properties to this subtree.
    fn focus_properties(self, properties: FocusProperties) -> Modifier;

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
            tail: Some(Arc::new(ModifierLink {
                prev: self.tail,
                action,
            })),
        }
    }

    /// Appends another modifier chain after this one.
    pub fn then(mut self, other: Modifier) -> Self {
        let actions = collect_actions(other.tail);
        for action in actions.into_iter().rev() {
            self = self.push_action(action);
        }
        self
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

    fn push_focus_op(self, op: FocusModifierOp) -> Self {
        self.push_action(ModifierAction::Focus(op))
    }

    /// Attaches this modifier chain to the current component node.
    pub fn attach(self) {
        ensure_build_phase();

        let actions = collect_actions(self.tail.clone());
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
        self.tail.is_none()
    }

    pub(crate) fn ordered_actions(&self) -> Vec<OrderedModifierAction> {
        collect_actions_in_source_order(self.tail.clone())
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
                ModifierAction::ImePreviewInput(node) => {
                    Some(OrderedModifierAction::ImePreviewInput(node))
                }
                ModifierAction::ImeInput(node) => Some(OrderedModifierAction::ImeInput(node)),
                ModifierAction::Build(_)
                | ModifierAction::Semantics(_)
                | ModifierAction::Focus(_) => None,
            })
            .collect()
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
        #[derive(Clone)]
        enum PlacementComparableAction {
            Layout(Arc<dyn ErasedLayoutModifierNode>),
            Placement(Arc<dyn ErasedPlacementModifierNode>),
        }

        let lhs: Vec<_> = self
            .ordered_actions()
            .into_iter()
            .filter_map(|action| match action {
                OrderedModifierAction::Layout(node) => {
                    Some(PlacementComparableAction::Layout(node))
                }
                OrderedModifierAction::Placement(node) => {
                    Some(PlacementComparableAction::Placement(node))
                }
                _ => None,
            })
            .collect();
        let rhs: Vec<_> = other
            .ordered_actions()
            .into_iter()
            .filter_map(|action| match action {
                OrderedModifierAction::Layout(node) => {
                    Some(PlacementComparableAction::Layout(node))
                }
                OrderedModifierAction::Placement(node) => {
                    Some(PlacementComparableAction::Placement(node))
                }
                _ => None,
            })
            .collect();
        lhs.len() == rhs.len()
            && lhs
                .iter()
                .zip(rhs.iter())
                .all(|(lhs, rhs)| match (lhs, rhs) {
                    (
                        PlacementComparableAction::Layout(lhs),
                        PlacementComparableAction::Layout(rhs),
                    ) => lhs.placement_eq(rhs.as_ref()),
                    (
                        PlacementComparableAction::Placement(lhs),
                        PlacementComparableAction::Placement(rhs),
                    ) => lhs.eq(rhs.as_ref()),
                    _ => false,
                })
    }
}

impl ModifierCapabilityExt for Modifier {
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
}

impl fmt::Debug for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Modifier")
            .field("is_empty", &self.tail.is_none())
            .finish()
    }
}

impl PartialEq for Modifier {
    fn eq(&self, other: &Self) -> bool {
        match (&self.tail, &other.tail) {
            (None, None) => true,
            (Some(lhs), Some(rhs)) => Arc::ptr_eq(lhs, rhs),
            _ => false,
        }
    }
}

impl Eq for Modifier {}

impl Hash for Modifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.tail {
            Some(node) => std::ptr::hash(Arc::as_ptr(node), state),
            None => 0u8.hash(state),
        }
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
        FocusModifierOp::TraversalPolicy(policy) => runtime.set_traversal_policy(policy),
        FocusModifierOp::ChangedHandler(handler) => runtime.set_changed_handler(handler),
        FocusModifierOp::EventHandler(handler) => runtime.set_event_handler(handler),
        FocusModifierOp::BeyondBoundsHandler(handler) => runtime.set_beyond_bounds_handler(handler),
        FocusModifierOp::RevealHandler(handler) => runtime.set_reveal_handler(handler),
    }
}
