//! Focus ownership, traversal, and restoration for interactive UI.
//!
//! ## Usage
//!
//! Register focus targets, scopes, and traversal rules for keyboard and IME
//! input.

use std::{
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    NodeId, PxRect,
    component_tree::{ComponentNodeMetaDatas, ComponentNodeTree},
    execution_context::{with_execution_context, with_execution_context_mut},
    prop::CallbackWith,
    px::PxSize,
    runtime::{
        TesseraRuntime, focus_read_subscribers, focus_requester_read_subscribers,
        has_persistent_focus_handle, record_component_invalidation_for_instance_key,
        track_focus_read_dependency, track_focus_requester_read_dependency,
    },
};

static NEXT_FOCUS_TARGET_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_FOCUS_REQUESTER_ID: AtomicU64 = AtomicU64::new(1);
const ROOT_SCOPE_ID: FocusHandleId = 0;

pub(crate) type FocusHandleId = u64;
pub(crate) type FocusRequesterId = u64;

fn next_focus_handle_id() -> FocusHandleId {
    NEXT_FOCUS_TARGET_ID.fetch_add(1, Ordering::Relaxed)
}

fn next_focus_requester_id() -> FocusRequesterId {
    NEXT_FOCUS_REQUESTER_ID.fetch_add(1, Ordering::Relaxed)
}

fn with_bound_focus_owner<R>(f: impl FnOnce(&FocusOwner) -> R) -> Option<R> {
    let ptr = with_execution_context(|context| context.current_focus_owner_stack.last().copied())?;
    // SAFETY: The binding guard only pushes a pointer that remains valid for
    // the duration of the handler dispatch on the current thread.
    Some(unsafe { f(ptr.as_ref()) })
}

fn with_bound_focus_owner_mut<R>(f: impl FnOnce(&mut FocusOwner) -> R) -> Option<R> {
    let ptr = with_execution_context(|context| context.current_focus_owner_stack.last().copied())?;
    // SAFETY: The binding guard only pushes a pointer that remains valid for
    // the duration of the handler dispatch on the current thread.
    Some(unsafe { f(&mut *ptr.as_ptr()) })
}

fn with_focus_owner<R>(f: impl FnOnce(&FocusOwner) -> R) -> R {
    let mut f = Some(f);
    if let Some(result) = with_bound_focus_owner(|owner| {
        f.take().expect("focus owner callback should run once")(owner)
    }) {
        return result;
    }
    TesseraRuntime::with(|runtime| {
        f.take().expect("focus owner callback should run once")(
            runtime.component_tree.focus_owner(),
        )
    })
}

fn with_focus_owner_mut<R>(f: impl FnOnce(&mut FocusOwner) -> R) -> R {
    let mut f = Some(f);
    if let Some(result) = with_bound_focus_owner_mut(|owner| {
        f.take().expect("focus owner callback should run once")(owner)
    }) {
        return result;
    }
    let result = TesseraRuntime::with_mut(|runtime| {
        f.take().expect("focus owner callback should run once")(
            runtime.component_tree.focus_owner_mut(),
        )
    });
    flush_pending_focus_callbacks();
    result
}

pub(crate) struct FocusOwnerBindingGuard {
    active: bool,
}

impl Drop for FocusOwnerBindingGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        with_execution_context_mut(|context| {
            let popped = context.current_focus_owner_stack.pop();
            debug_assert!(popped.is_some(), "focus owner binding stack underflow");
        });
        self.active = false;
    }
}

pub(crate) fn bind_focus_owner(owner: &mut FocusOwner) -> FocusOwnerBindingGuard {
    with_execution_context_mut(|context| {
        context.current_focus_owner_stack.push(NonNull::from(owner));
    });
    FocusOwnerBindingGuard { active: true }
}

/// Focus state for a registered focus node.
///
/// Use this to render focus-dependent UI and distinguish between the primary
/// focused target and its focused ancestors.
///
/// # Examples
///
/// ```
/// use tessera_ui::FocusState;
///
/// assert!(!FocusState::Inactive.has_focus());
/// assert!(FocusState::ActiveParent.has_focus());
/// assert!(FocusState::Active.is_focused());
/// assert!(FocusState::Captured.is_captured());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusState {
    /// The node is not participating in the active focus path.
    #[default]
    Inactive,
    /// A descendant of this node currently holds focus.
    ActiveParent,
    /// This node currently holds focus.
    Active,
    /// This node currently holds captured focus.
    Captured,
}

impl FocusState {
    /// Returns `true` when this node is in the active focus path.
    pub fn has_focus(self) -> bool {
        !matches!(self, Self::Inactive)
    }

    /// Returns `true` when this node is the primary focused node.
    pub fn is_focused(self) -> bool {
        matches!(self, Self::Active | Self::Captured)
    }

    /// Returns `true` when this node has captured focus.
    pub fn is_captured(self) -> bool {
        matches!(self, Self::Captured)
    }
}

/// Focus traversal direction for owner-level movement.
///
/// These directions are used by [`FocusManager`], [`FocusScopeNode`], and
/// [`FocusGroupNode`] when keyboard or programmatic traversal moves focus.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FocusDirection {
    /// Move to the next focusable target in traversal order.
    Next,
    /// Move to the previous focusable target in traversal order.
    Previous,
    /// Move focus upward.
    Up,
    /// Move focus downward.
    Down,
    /// Move focus leftward.
    Left,
    /// Move focus rightward.
    Right,
    /// Enter a nested focus group or scope.
    Enter,
    /// Exit the current focus group or scope.
    Exit,
}

/// Strategy used by a focus traversal policy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FocusTraversalStrategy {
    /// Traverse in declaration order using previous/next semantics.
    Linear,
    /// Traverse horizontally using left/right arrows.
    Horizontal,
    /// Traverse vertically using up/down arrows.
    Vertical,
    /// Traverse spatially using candidate rectangles.
    Spatial,
}

/// Policy applied by a focus scope or group when traversal starts inside it.
///
/// Use policies to describe how a container handles directional keys and
/// whether `Tab` navigation should stay inside the container.
///
/// # Examples
///
/// ```
/// use tessera_ui::{FocusTraversalPolicy, FocusTraversalStrategy};
///
/// let policy = FocusTraversalPolicy::vertical()
///     .wrap(true)
///     .tab_navigation(true);
///
/// assert_eq!(policy.strategy, FocusTraversalStrategy::Vertical);
/// assert!(policy.wrap);
/// assert!(policy.tab_navigation);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FocusTraversalPolicy {
    /// Traversal strategy used for arrow-key navigation.
    pub strategy: FocusTraversalStrategy,
    /// Whether traversal wraps when it reaches either end of the group.
    pub wrap: bool,
    /// Whether `Tab` and `Shift+Tab` should be handled by this policy.
    pub tab_navigation: bool,
}

impl FocusTraversalPolicy {
    /// Creates a new traversal policy for the given strategy.
    pub const fn new(strategy: FocusTraversalStrategy) -> Self {
        Self {
            strategy,
            wrap: false,
            tab_navigation: false,
        }
    }

    /// Creates a linear traversal policy.
    pub const fn linear() -> Self {
        Self::new(FocusTraversalStrategy::Linear)
    }

    /// Creates a horizontal traversal policy.
    pub const fn horizontal() -> Self {
        Self::new(FocusTraversalStrategy::Horizontal)
    }

    /// Creates a vertical traversal policy.
    pub const fn vertical() -> Self {
        Self::new(FocusTraversalStrategy::Vertical)
    }

    /// Creates a spatial traversal policy.
    pub const fn spatial() -> Self {
        Self::new(FocusTraversalStrategy::Spatial)
    }

    /// Sets whether traversal wraps when it reaches either end.
    pub const fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Sets whether `Tab` and `Shift+Tab` should be handled inside this
    /// policy owner.
    pub const fn tab_navigation(mut self, tab_navigation: bool) -> Self {
        self.tab_navigation = tab_navigation;
        self
    }

    fn scoped_direction(self, direction: FocusDirection) -> Option<FocusDirection> {
        if self.tab_navigation {
            match direction {
                FocusDirection::Next | FocusDirection::Previous => return Some(direction),
                _ => {}
            }
        }

        match self.strategy {
            FocusTraversalStrategy::Linear => match direction {
                FocusDirection::Left | FocusDirection::Up => Some(FocusDirection::Previous),
                FocusDirection::Right | FocusDirection::Down => Some(FocusDirection::Next),
                _ => None,
            },
            FocusTraversalStrategy::Horizontal => match direction {
                FocusDirection::Left => Some(FocusDirection::Previous),
                FocusDirection::Right => Some(FocusDirection::Next),
                _ => None,
            },
            FocusTraversalStrategy::Vertical => match direction {
                FocusDirection::Up => Some(FocusDirection::Previous),
                FocusDirection::Down => Some(FocusDirection::Next),
                _ => None,
            },
            FocusTraversalStrategy::Spatial => match direction {
                FocusDirection::Left
                | FocusDirection::Right
                | FocusDirection::Up
                | FocusDirection::Down => Some(direction),
                _ => None,
            },
        }
    }
}

/// A request for scroll containers to reveal a focused rectangle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FocusRevealRequest {
    /// The absolute rectangle of the focused target.
    pub target_rect: PxRect,
    /// The absolute visible viewport of the current reveal container.
    pub viewport_rect: PxRect,
}

impl FocusRevealRequest {
    /// Creates a new focus reveal request.
    pub(crate) const fn new(target_rect: PxRect, viewport_rect: PxRect) -> Self {
        Self {
            target_rect,
            viewport_rect,
        }
    }
}

/// Focus properties used when registering a focus node on the current
/// component.
///
/// Use properties to enable or disable focus participation and to declare
/// explicit traversal neighbors for complex layouts.
///
/// # Examples
///
/// ```
/// use tessera_ui::{FocusProperties, FocusRequester};
///
/// let next = FocusRequester::new();
/// let right = FocusRequester::new();
/// let props = FocusProperties::new()
///     .can_focus(true)
///     .skip_traversal(false)
///     .next(next)
///     .right(right);
///
/// assert_eq!(props.next, Some(next));
/// assert_eq!(props.right, Some(right));
/// assert!(props.can_focus);
/// assert!(!props.skip_traversal);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusProperties {
    /// Whether this node may receive focus.
    pub can_focus: bool,
    /// Whether this node may be the target of focus requests.
    pub can_request_focus: bool,
    /// Whether traversal should skip this node.
    pub skip_traversal: bool,
    /// Explicit target for `Next` traversal.
    pub next: Option<FocusRequester>,
    /// Explicit target for `Previous` traversal.
    pub previous: Option<FocusRequester>,
    /// Explicit target for `Up` traversal.
    pub up: Option<FocusRequester>,
    /// Explicit target for `Down` traversal.
    pub down: Option<FocusRequester>,
    /// Explicit target for `Left` traversal.
    pub left: Option<FocusRequester>,
    /// Explicit target for `Right` traversal.
    pub right: Option<FocusRequester>,
    /// Explicit target when entering a nested focus region.
    pub enter: Option<FocusRequester>,
    /// Explicit target when exiting a focus region.
    pub exit: Option<FocusRequester>,
}

impl FocusProperties {
    /// Creates the default focus properties for focus targets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether this node may receive focus.
    pub fn can_focus(mut self, can_focus: bool) -> Self {
        self.can_focus = can_focus;
        self
    }

    /// Sets whether this node may be the target of focus requests.
    pub fn can_request_focus(mut self, can_request_focus: bool) -> Self {
        self.can_request_focus = can_request_focus;
        self
    }

    /// Sets whether traversal should skip this node.
    pub fn skip_traversal(mut self, skip_traversal: bool) -> Self {
        self.skip_traversal = skip_traversal;
        self
    }

    /// Sets the explicit requester for `Next` traversal.
    pub fn next(mut self, requester: FocusRequester) -> Self {
        self.next = Some(requester);
        self
    }

    /// Sets the explicit requester for `Previous` traversal.
    pub fn previous(mut self, requester: FocusRequester) -> Self {
        self.previous = Some(requester);
        self
    }

    /// Sets the explicit requester for `Up` traversal.
    pub fn up(mut self, requester: FocusRequester) -> Self {
        self.up = Some(requester);
        self
    }

    /// Sets the explicit requester for `Down` traversal.
    pub fn down(mut self, requester: FocusRequester) -> Self {
        self.down = Some(requester);
        self
    }

    /// Sets the explicit requester for `Left` traversal.
    pub fn left(mut self, requester: FocusRequester) -> Self {
        self.left = Some(requester);
        self
    }

    /// Sets the explicit requester for `Right` traversal.
    pub fn right(mut self, requester: FocusRequester) -> Self {
        self.right = Some(requester);
        self
    }

    /// Sets the explicit requester used when moving focus into a group.
    pub fn enter(mut self, requester: FocusRequester) -> Self {
        self.enter = Some(requester);
        self
    }

    /// Sets the explicit requester used when moving focus out of a group.
    pub fn exit(mut self, requester: FocusRequester) -> Self {
        self.exit = Some(requester);
        self
    }
}

impl Default for FocusProperties {
    fn default() -> Self {
        Self {
            can_focus: true,
            can_request_focus: true,
            skip_traversal: false,
            next: None,
            previous: None,
            up: None,
            down: None,
            left: None,
            right: None,
            enter: None,
            exit: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusRegistrationKind {
    Target,
    Scope,
    Group,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FocusRegistration {
    pub(crate) id: FocusHandleId,
    pub(crate) kind: FocusRegistrationKind,
    pub(crate) properties: FocusProperties,
}

impl FocusRegistration {
    pub(crate) fn target(node: FocusNode) -> Self {
        Self {
            id: node.id,
            kind: FocusRegistrationKind::Target,
            properties: FocusProperties::default(),
        }
    }

    pub(crate) fn scope(scope: FocusScopeNode) -> Self {
        Self {
            id: scope.id,
            kind: FocusRegistrationKind::Scope,
            properties: FocusProperties::default().can_focus(false),
        }
    }

    pub(crate) fn group(group: FocusGroupNode) -> Self {
        Self {
            id: group.id,
            kind: FocusRegistrationKind::Group,
            properties: FocusProperties::default()
                .can_focus(false)
                .can_request_focus(false),
        }
    }
}

/// Imperative handle used to request focus for a registered target.
///
/// Requesters are stable handles that can be stored in component state and
/// passed across component boundaries without exposing the underlying focus
/// tree.
///
/// # Examples
///
/// ```
/// use tessera_ui::FocusRequester;
///
/// let first = FocusRequester::new();
/// let second = FocusRequester::new();
/// assert_ne!(first, second);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusRequester {
    id: FocusRequesterId,
}

impl FocusRequester {
    /// Creates a new focus requester.
    pub fn new() -> Self {
        Self {
            id: next_focus_requester_id(),
        }
    }

    /// Requests focus for the bound target.
    pub fn request_focus(&self) {
        with_focus_owner_mut(|owner| owner.request_focus_by_requester(self.id));
    }

    /// Clears focus from the bound target if it currently holds it.
    pub fn clear_focus(&self) {
        with_focus_owner_mut(|owner| owner.clear_focus_by_requester(self.id));
    }

    /// Alias for [`Self::clear_focus`].
    pub fn unfocus(&self) {
        self.clear_focus();
    }

    /// Captures focus on this node, preventing focus from moving elsewhere
    /// until released or force-cleared.
    pub fn capture_focus(&self) {
        with_focus_owner_mut(|owner| owner.capture_focus_by_requester(self.id));
    }

    /// Releases captured focus from this node.
    pub fn free_focus(&self) {
        with_focus_owner_mut(|owner| owner.free_focus_by_requester(self.id));
    }

    /// Returns the current focus state for the bound target.
    pub fn state(&self) -> FocusState {
        track_focus_requester_read_dependency(self.id);
        with_focus_owner(|owner| owner.state_of_requester(self.id))
    }

    /// Returns `true` when this node is in the active focus path.
    pub fn has_focus(&self) -> bool {
        self.state().has_focus()
    }

    /// Returns `true` when this node is the primary focused node.
    pub fn is_focused(&self) -> bool {
        self.state().is_focused()
    }

    /// Returns `true` when this node has captured focus.
    pub fn is_captured(&self) -> bool {
        self.state().is_captured()
    }

    pub(crate) fn requester_id(self) -> FocusRequesterId {
        self.id
    }
}

impl Default for FocusRequester {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent focus target handle.
///
/// This is the low-level target identity used by focus modifiers. Most code
/// should prefer [`FocusRequester`] and [`crate::modifier::FocusModifierExt`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FocusNode {
    id: FocusHandleId,
}

impl FocusNode {
    pub(crate) fn new() -> Self {
        Self {
            id: next_focus_handle_id(),
        }
    }

    pub(crate) fn handle_id(self) -> FocusHandleId {
        self.id
    }

    pub(crate) fn from_handle_id(id: FocusHandleId) -> Self {
        Self { id }
    }
}

impl Default for FocusNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent focus scope handle.
///
/// Scopes own restore behavior and provide imperative traversal within a
/// subtree.
///
/// # Examples
///
/// ```
/// use tessera_ui::FocusScopeNode;
///
/// let first = FocusScopeNode::new();
/// let second = FocusScopeNode::new();
/// assert_ne!(first, second);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusScopeNode {
    id: FocusHandleId,
}

impl FocusScopeNode {
    /// Creates a new persistent focus scope handle.
    pub fn new() -> Self {
        Self {
            id: next_focus_handle_id(),
        }
    }

    pub(crate) fn handle_id(self) -> FocusHandleId {
        self.id
    }

    pub(crate) fn from_handle_id(id: FocusHandleId) -> Self {
        Self { id }
    }

    /// Requests focus within this scope, restoring the last focused descendant
    /// when possible.
    pub fn request_focus(&self) {
        with_focus_owner_mut(|owner| owner.request_focus(self.id));
    }

    /// Clears focus from this scope.
    pub fn clear_focus(&self) {
        with_focus_owner_mut(|owner| owner.clear_focus(self.id));
    }

    /// Restores focus to the last focused descendant of this scope.
    pub fn restore_focus(&self) {
        with_focus_owner_mut(|owner| owner.restore_focus(self.id));
    }

    /// Moves focus to another focusable descendant inside this scope.
    pub fn move_focus(&self, direction: FocusDirection) -> bool {
        with_focus_owner_mut(|owner| owner.move_focus_in_scope(self.id, direction, false))
    }

    /// Cycles focus inside this scope, wrapping when traversal reaches an end.
    pub fn cycle_focus(&self, direction: FocusDirection) -> bool {
        with_focus_owner_mut(|owner| owner.move_focus_in_scope(self.id, direction, true))
    }

    /// Returns the current focus state for this scope.
    pub fn state(&self) -> FocusState {
        track_focus_read_dependency(self.id);
        with_focus_owner(|owner| owner.state_of(self.id))
    }

    /// Returns `true` when this scope participates in the active focus path.
    pub fn has_focus(&self) -> bool {
        self.state().has_focus()
    }

    /// Returns `true` when this scope itself is the primary focused node.
    pub fn is_focused(&self) -> bool {
        self.state().is_focused()
    }
}

impl Default for FocusScopeNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistent traversal-only focus group handle.
///
/// Groups define traversal boundaries without taking on scope restore
/// semantics.
///
/// # Examples
///
/// ```
/// use tessera_ui::FocusGroupNode;
///
/// let first = FocusGroupNode::new();
/// let second = FocusGroupNode::new();
/// assert_ne!(first, second);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusGroupNode {
    id: FocusHandleId,
}

impl FocusGroupNode {
    /// Creates a new persistent focus group handle.
    pub fn new() -> Self {
        Self {
            id: next_focus_handle_id(),
        }
    }

    pub(crate) fn handle_id(self) -> FocusHandleId {
        self.id
    }

    pub(crate) fn from_handle_id(id: FocusHandleId) -> Self {
        Self { id }
    }

    /// Moves focus to another focusable descendant inside this group.
    pub fn move_focus(&self, direction: FocusDirection) -> bool {
        with_focus_owner_mut(|owner| owner.move_focus_in_scope(self.id, direction, false))
    }

    /// Cycles focus inside this group, wrapping when traversal reaches an end.
    pub fn cycle_focus(&self, direction: FocusDirection) -> bool {
        with_focus_owner_mut(|owner| owner.move_focus_in_scope(self.id, direction, true))
    }

    /// Returns the current focus state for this group.
    pub fn state(&self) -> FocusState {
        track_focus_read_dependency(self.id);
        with_focus_owner(|owner| owner.state_of(self.id))
    }

    /// Returns `true` when this group participates in the active focus path.
    pub fn has_focus(&self) -> bool {
        self.state().has_focus()
    }

    /// Returns `true` when this group itself is the primary focused node.
    pub fn is_focused(&self) -> bool {
        self.state().is_focused()
    }
}

impl Default for FocusGroupNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Owner-scoped focus manager for traversal and forced clear operations.
///
/// Use the focus manager to trigger application-level traversal without
/// storing a specific scope or requester.
///
/// # Examples
///
/// ```
/// use tessera_ui::FocusManager;
///
/// let _manager = FocusManager::current();
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct FocusManager;

impl FocusManager {
    /// Returns the focus manager for the current component tree.
    pub fn current() -> Self {
        Self
    }

    /// Clears the current focus. Set `force` to `true` to break captured focus.
    pub fn clear_focus(self, force: bool) {
        with_focus_owner_mut(|owner| {
            owner.clear_focus_global(force);
        });
    }

    /// Moves focus in the requested direction.
    pub fn move_focus(self, direction: FocusDirection) -> bool {
        with_focus_owner_mut(|owner| owner.move_focus(direction))
    }
}

#[derive(Clone, Copy, Debug)]
struct FocusAttachment {
    host_instance_key: u64,
    live_node_id: Option<NodeId>,
    focus_rect: Option<PxRect>,
    traversal_order: Option<u64>,
    parent: FocusHandleId,
    scope_parent: FocusHandleId,
}

impl FocusAttachment {
    fn root_scope() -> Self {
        Self {
            host_instance_key: 0,
            live_node_id: None,
            focus_rect: None,
            traversal_order: None,
            parent: ROOT_SCOPE_ID,
            scope_parent: ROOT_SCOPE_ID,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct FocusTreeNode {
    kind: FocusRegistrationKind,
    attachment: Option<FocusAttachment>,
    last_parent: FocusHandleId,
    last_scope_parent: FocusHandleId,
    props: FocusProperties,
    traversal_policy: Option<FocusTraversalPolicy>,
    state: FocusState,
    last_focused_descendant: Option<FocusHandleId>,
    restorer_fallback: Option<FocusRequesterId>,
}

impl FocusTreeNode {
    fn new_root_scope() -> Self {
        Self {
            kind: FocusRegistrationKind::Scope,
            attachment: Some(FocusAttachment::root_scope()),
            last_parent: ROOT_SCOPE_ID,
            last_scope_parent: ROOT_SCOPE_ID,
            props: FocusProperties::default()
                .can_focus(false)
                .can_request_focus(false)
                .skip_traversal(true),
            traversal_policy: None,
            state: FocusState::Inactive,
            last_focused_descendant: None,
            restorer_fallback: None,
        }
    }

    fn new(kind: FocusRegistrationKind) -> Self {
        Self {
            kind,
            attachment: None,
            last_parent: ROOT_SCOPE_ID,
            last_scope_parent: ROOT_SCOPE_ID,
            props: FocusProperties::default(),
            traversal_policy: None,
            state: FocusState::Inactive,
            last_focused_descendant: None,
            restorer_fallback: None,
        }
    }

    fn is_attached(&self) -> bool {
        self.attachment.is_some()
    }
}

#[derive(Clone)]
pub(crate) struct PendingFocusCallbackInvocation {
    callback: CallbackWith<FocusState>,
    state: FocusState,
}

impl PendingFocusCallbackInvocation {
    pub(crate) fn new(callback: CallbackWith<FocusState>, state: FocusState) -> Self {
        Self { callback, state }
    }

    pub(crate) fn invoke(self) {
        self.callback.call(self.state);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FocusNotification {
    pub handle_id: FocusHandleId,
    pub state: FocusState,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug)]
enum FocusCommand {
    Request(FocusHandleId),
    Clear(FocusHandleId),
    Capture(FocusHandleId),
    Free(FocusHandleId),
    Restore(FocusHandleId),
}

/// Per-component-tree focus owner.
#[derive(Default)]
pub(crate) struct FocusOwner {
    nodes: HashMap<FocusHandleId, FocusTreeNode>,
    children_by_parent: HashMap<FocusHandleId, Vec<FocusHandleId>>,
    requester_bindings: HashMap<FocusRequesterId, FocusHandleId>,
    requesters_by_handle: HashMap<FocusHandleId, Vec<FocusRequesterId>>,
    active: Option<FocusHandleId>,
    captured: Option<FocusHandleId>,
    suspended: Option<FocusHandleId>,
    owner_focused: bool,
    pending: Vec<FocusCommand>,
    pending_notifications: Vec<FocusNotification>,
    pending_reveal: Option<FocusHandleId>,
}

impl FocusOwner {
    pub(crate) fn new() -> Self {
        let mut nodes = HashMap::default();
        nodes.insert(ROOT_SCOPE_ID, FocusTreeNode::new_root_scope());
        Self {
            nodes,
            children_by_parent: HashMap::default(),
            requester_bindings: HashMap::default(),
            requesters_by_handle: HashMap::default(),
            active: None,
            captured: None,
            suspended: None,
            owner_focused: true,
            pending: Vec::new(),
            pending_notifications: Vec::new(),
            pending_reveal: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    pub(crate) fn state_of(&self, id: FocusHandleId) -> FocusState {
        self.nodes
            .get(&id)
            .filter(|node| node.is_attached())
            .map(|node| node.state)
            .unwrap_or(FocusState::Inactive)
    }

    pub(crate) fn active_component_node_id(&self) -> Option<NodeId> {
        let active = self.active?;
        self.nodes
            .get(&active)
            .and_then(|node| node.attachment)
            .and_then(|attachment| attachment.live_node_id)
    }

    pub(crate) fn component_node_id_of(&self, id: FocusHandleId) -> Option<NodeId> {
        self.nodes
            .get(&id)
            .and_then(|node| node.attachment)
            .and_then(|attachment| attachment.live_node_id)
    }

    pub(crate) fn active_handle_id(&self) -> Option<FocusHandleId> {
        self.active
    }

    pub(crate) fn take_pending_reveal(&mut self) -> Option<FocusHandleId> {
        self.pending_reveal.take()
    }

    pub(crate) fn state_of_requester(&self, id: FocusRequesterId) -> FocusState {
        self.requester_bindings
            .get(&id)
            .copied()
            .map(|handle_id| self.state_of(handle_id))
            .unwrap_or(FocusState::Inactive)
    }

    pub(crate) fn request_focus_by_requester(&mut self, id: FocusRequesterId) {
        if let Some(handle_id) = self.requester_bindings.get(&id).copied() {
            self.request_focus(handle_id);
        }
    }

    pub(crate) fn clear_focus_by_requester(&mut self, id: FocusRequesterId) {
        if let Some(handle_id) = self.requester_bindings.get(&id).copied() {
            self.clear_focus(handle_id);
        }
    }

    pub(crate) fn capture_focus_by_requester(&mut self, id: FocusRequesterId) {
        if let Some(handle_id) = self.requester_bindings.get(&id).copied() {
            self.capture_focus(handle_id);
        }
    }

    pub(crate) fn free_focus_by_requester(&mut self, id: FocusRequesterId) {
        if let Some(handle_id) = self.requester_bindings.get(&id).copied() {
            self.free_focus(handle_id);
        }
    }

    pub(crate) fn request_focus(&mut self, id: FocusHandleId) {
        if !self.apply_command(FocusCommand::Request(id)) {
            self.pending.push(FocusCommand::Request(id));
        }
    }

    pub(crate) fn clear_focus(&mut self, id: FocusHandleId) {
        if !self.apply_command(FocusCommand::Clear(id)) {
            self.pending.push(FocusCommand::Clear(id));
        }
    }

    pub(crate) fn capture_focus(&mut self, id: FocusHandleId) {
        if !self.apply_command(FocusCommand::Capture(id)) {
            self.pending.push(FocusCommand::Capture(id));
        }
    }

    pub(crate) fn free_focus(&mut self, id: FocusHandleId) {
        if !self.apply_command(FocusCommand::Free(id)) {
            self.pending.push(FocusCommand::Free(id));
        }
    }

    pub(crate) fn restore_focus(&mut self, id: FocusHandleId) {
        if !self.apply_command(FocusCommand::Restore(id)) {
            self.pending.push(FocusCommand::Restore(id));
        }
    }

    pub(crate) fn set_owner_focused(&mut self, focused: bool) {
        if self.owner_focused == focused {
            return;
        }

        let previous_states = self.snapshot_live_states();
        self.owner_focused = focused;
        if focused {
            if let Some(suspended) = self.suspended.take() {
                if let Some(target) = self.resolve_request_target(suspended) {
                    self.active = Some(target);
                    self.pending_reveal = Some(target);
                } else {
                    self.suspended = Some(suspended);
                }
            }
        } else {
            self.suspended = self.active;
            self.active = None;
            self.captured = None;
            self.pending_reveal = None;
        }
        self.recompute_states();
        self.notify_state_changes(previous_states);
    }

    pub(crate) fn sync_from_component_tree(&mut self, root_node: NodeId, tree: &ComponentNodeTree) {
        self.nodes
            .entry(ROOT_SCOPE_ID)
            .or_insert_with(FocusTreeNode::new_root_scope);

        let previous_states = self.snapshot_live_states();
        let previous_requester_bindings = self.requester_bindings.clone();
        for node in self.nodes.values_mut() {
            let is_root_scope = matches!(
                node.attachment,
                Some(FocusAttachment {
                    host_instance_key: 0,
                    ..
                })
            );
            if !is_root_scope {
                node.attachment = None;
                node.state = FocusState::Inactive;
            }
        }
        self.children_by_parent.clear();
        self.children_by_parent.insert(ROOT_SCOPE_ID, Vec::new());
        self.requester_bindings.clear();
        self.requesters_by_handle.clear();
        if let Some(root) = self.nodes.get_mut(&ROOT_SCOPE_ID) {
            root.attachment = Some(FocusAttachment::root_scope());
            root.state = FocusState::Inactive;
        }

        let mut seen = HashSet::default();
        self.collect_focus_nodes(root_node, tree, ROOT_SCOPE_ID, ROOT_SCOPE_ID, &mut seen);

        self.repair_after_sync();
        self.recompute_states();
        self.notify_state_changes(previous_states);
        self.notify_requester_binding_changes(previous_requester_bindings);
    }

    pub(crate) fn sync_layout_from_component_tree(
        &mut self,
        root_node: NodeId,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
    ) {
        let handles_by_node_id: HashMap<NodeId, FocusHandleId> = self
            .nodes
            .iter()
            .filter_map(|(&handle_id, node)| {
                node.attachment
                    .and_then(|attachment| attachment.live_node_id)
                    .map(|node_id| (node_id, handle_id))
            })
            .collect();

        for node in self.nodes.values_mut() {
            if let Some(attachment) = node.attachment.as_mut() {
                attachment.focus_rect = None;
                attachment.traversal_order = None;
            }
        }

        let mut next_traversal_order = 0_u64;
        self.assign_layout_info_for_subtree(
            root_node,
            tree,
            metadatas,
            &handles_by_node_id,
            &mut next_traversal_order,
        );
        self.sort_children_by_traversal_order();
    }

    pub(crate) fn commit_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }

        let previous_states = self.snapshot_live_states();
        let commands = std::mem::take(&mut self.pending);
        for command in commands {
            if !self.apply_command_inner(command) {
                self.pending.push(command);
            }
        }
        self.recompute_states();
        self.notify_state_changes(previous_states);
    }

    pub(crate) fn take_pending_notifications(&mut self) -> Vec<FocusNotification> {
        std::mem::take(&mut self.pending_notifications)
    }

    pub(crate) fn clear_focus_global(&mut self, force: bool) -> bool {
        let previous_states = self.snapshot_live_states();
        let cleared = if self.active.is_some() && (force || self.captured.is_none()) {
            self.active = None;
            self.captured = None;
            self.pending_reveal = None;
            true
        } else {
            false
        };
        self.recompute_states();
        self.notify_state_changes(previous_states);
        cleared
    }

    pub(crate) fn move_focus(&mut self, direction: FocusDirection) -> bool {
        if !self.owner_focused {
            return false;
        }
        let previous_states = self.snapshot_live_states();
        let moved = self
            .resolve_policy_move_target(direction)
            .or_else(|| self.resolve_move_target(direction))
            .is_some_and(|target| self.set_active(target));
        self.recompute_states();
        self.notify_state_changes(previous_states);
        moved
    }

    pub(crate) fn move_focus_in_scope(
        &mut self,
        scope_id: FocusHandleId,
        direction: FocusDirection,
        wrap: bool,
    ) -> bool {
        if !self.owner_focused {
            return false;
        }
        let previous_states = self.snapshot_live_states();
        let moved = self
            .resolve_move_target_in_scope(scope_id, direction, wrap)
            .is_some_and(|target| self.set_active(target));
        self.recompute_states();
        self.notify_state_changes(previous_states);
        moved
    }

    fn resolve_policy_move_target(&self, direction: FocusDirection) -> Option<FocusHandleId> {
        let active = self
            .active
            .and_then(|current| self.resolve_request_target(current))?;
        let (owner_id, policy) = self.nearest_traversal_policy_owner(active)?;
        let scoped_direction = policy.scoped_direction(direction)?;
        self.resolve_move_target_in_scope(owner_id, scoped_direction, policy.wrap)
    }

    fn nearest_traversal_policy_owner(
        &self,
        start: FocusHandleId,
    ) -> Option<(FocusHandleId, FocusTraversalPolicy)> {
        let mut current = Some(start);
        while let Some(id) = current {
            let node = self.nodes.get(&id)?;
            if matches!(
                node.kind,
                FocusRegistrationKind::Scope | FocusRegistrationKind::Group
            ) && let Some(policy) = node.traversal_policy
            {
                return Some((id, policy));
            }
            current = (id != ROOT_SCOPE_ID).then_some(node.last_parent);
        }
        None
    }

    fn collect_focus_nodes(
        &mut self,
        node_id: NodeId,
        tree: &ComponentNodeTree,
        parent_focus_id: FocusHandleId,
        current_scope_id: FocusHandleId,
        seen: &mut HashSet<FocusHandleId>,
    ) {
        let Some(node_ref) = tree.get(node_id) else {
            return;
        };
        let node = node_ref.get();

        let mut next_parent_focus_id = parent_focus_id;
        let mut next_scope_id = current_scope_id;

        if let Some(registration) = node.focus_registration {
            if !seen.insert(registration.id) {
                debug_assert!(
                    false,
                    "Focus handle {} was registered on multiple component nodes in the same frame",
                    registration.id
                );
            }
            let entry = self
                .nodes
                .entry(registration.id)
                .or_insert_with(|| FocusTreeNode::new(registration.kind));
            entry.kind = registration.kind;
            entry.last_parent = parent_focus_id;
            entry.last_scope_parent = current_scope_id;
            entry.attachment = Some(FocusAttachment {
                host_instance_key: node.instance_key,
                live_node_id: Some(node_id),
                focus_rect: None,
                traversal_order: None,
                parent: parent_focus_id,
                scope_parent: current_scope_id,
            });
            entry.props = registration.properties;
            entry.traversal_policy = node.focus_traversal_policy;
            entry.restorer_fallback = (registration.kind == FocusRegistrationKind::Scope)
                .then(|| {
                    node.focus_restorer_fallback
                        .map(|fallback| fallback.requester_id())
                })
                .flatten();

            self.children_by_parent
                .entry(parent_focus_id)
                .or_default()
                .push(registration.id);
            self.children_by_parent.entry(registration.id).or_default();
            if let Some(requester) = node.focus_requester_binding {
                let replaced = self
                    .requester_bindings
                    .insert(requester.requester_id(), registration.id);
                debug_assert!(
                    replaced.is_none() || replaced == Some(registration.id),
                    "FocusRequester {} was bound to multiple focus nodes in the same frame",
                    requester.requester_id()
                );
                self.requesters_by_handle
                    .entry(registration.id)
                    .or_default()
                    .push(requester.requester_id());
            }

            next_parent_focus_id = registration.id;
            if registration.kind == FocusRegistrationKind::Scope {
                next_scope_id = registration.id;
            }
        } else if node.focus_requester_binding.is_some() {
            debug_assert!(
                false,
                "focus_requester requires focus_target or focus_scope on the same component node"
            );
        }

        for child_id in node_id.children(tree) {
            self.collect_focus_nodes(child_id, tree, next_parent_focus_id, next_scope_id, seen);
        }
    }

    fn assign_layout_info_for_subtree(
        &mut self,
        node_id: NodeId,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        handles_by_node_id: &HashMap<NodeId, FocusHandleId>,
        next_traversal_order: &mut u64,
    ) {
        if let Some(handle_id) = handles_by_node_id.get(&node_id).copied() {
            let focus_rect = metadatas.get(&node_id).and_then(|metadata| {
                let abs_position = metadata.abs_position?;
                let computed_data = metadata.computed_data?;
                let node_rect = PxRect::from_position_size(
                    abs_position,
                    PxSize::new(computed_data.width, computed_data.height),
                );
                Some(if let Some(clip_rect) = metadata.event_clip_rect {
                    clip_rect.intersection(&node_rect).unwrap_or(PxRect::ZERO)
                } else {
                    node_rect
                })
            });
            if let Some(node) = self.nodes.get_mut(&handle_id)
                && let Some(attachment) = node.attachment.as_mut()
            {
                attachment.focus_rect = focus_rect;
                attachment.traversal_order = Some(*next_traversal_order);
            }
            *next_traversal_order += 1;
        }

        let mut children: Vec<_> = node_id.children(tree).enumerate().collect();
        children.sort_by_key(|(composition_order, child_id)| {
            let placement_order = metadatas
                .get(child_id)
                .and_then(|metadata| metadata.placement_order)
                .unwrap_or(u64::MAX);
            (placement_order, *composition_order as u64)
        });

        for (_, child_id) in children {
            self.assign_layout_info_for_subtree(
                child_id,
                tree,
                metadatas,
                handles_by_node_id,
                next_traversal_order,
            );
        }
    }

    fn sort_children_by_traversal_order(&mut self) {
        let order_by_handle: HashMap<FocusHandleId, (u64, u64)> = self
            .nodes
            .iter()
            .map(|(&handle_id, node)| {
                (
                    handle_id,
                    (
                        node.attachment
                            .and_then(|attachment| attachment.traversal_order)
                            .unwrap_or(u64::MAX),
                        node.attachment
                            .map(|attachment| attachment.host_instance_key)
                            .unwrap_or(u64::MAX),
                    ),
                )
            })
            .collect();

        for children in self.children_by_parent.values_mut() {
            children.sort_by_key(|child| {
                order_by_handle
                    .get(child)
                    .copied()
                    .unwrap_or((u64::MAX, u64::MAX))
            });
        }
    }

    fn repair_after_sync(&mut self) {
        if self
            .active
            .is_some_and(|id| !self.nodes.get(&id).is_some_and(|node| node.is_attached()))
        {
            let previous_active = self.active;
            if !previous_active.is_some_and(|id| self.should_wait_for_reattach(id)) {
                let previous_active = self.active.take();
                self.active = previous_active.and_then(|id| self.restore_from_detached(id));
                self.captured = None;
                if let Some(active) = self.active {
                    self.update_scope_restore_chain(active);
                    self.pending_reveal = Some(active);
                }
            }
        }

        if self
            .captured
            .is_some_and(|id| !self.nodes.get(&id).is_some_and(|node| node.is_attached()))
        {
            self.captured = None;
        }

        if self.owner_focused
            && self.active.is_none()
            && let Some(suspended) = self.suspended
            && let Some(target) = self.resolve_request_target(suspended)
        {
            self.active = Some(target);
            self.suspended = None;
            self.pending_reveal = Some(target);
        }

        if self.active.is_some_and(|id| {
            self.nodes.get(&id).is_some_and(|node| {
                node.is_attached() && (!self.can_focus(id) || !self.can_request_focus(id))
            })
        }) {
            self.active = None;
            self.captured = None;
            self.pending_reveal = None;
        }

        let live_ids = self
            .nodes
            .iter()
            .filter_map(|(&id, node)| node.is_attached().then_some(id))
            .collect::<HashSet<_>>();
        for node in self.nodes.values_mut() {
            if node
                .last_focused_descendant
                .is_some_and(|last| !live_ids.contains(&last))
            {
                node.last_focused_descendant = None;
            }
            if node
                .restorer_fallback
                .is_some_and(|fallback| !self.requester_bindings.contains_key(&fallback))
            {
                node.restorer_fallback = None;
            }
        }
    }

    fn should_wait_for_reattach(&self, id: FocusHandleId) -> bool {
        let Some(node) = self.nodes.get(&id) else {
            return false;
        };
        if node.is_attached() {
            return false;
        }
        if has_persistent_focus_handle(id) {
            return true;
        }
        let waiting_scope_id = node.last_scope_parent;
        waiting_scope_id != ROOT_SCOPE_ID
            && self
                .nodes
                .get(&waiting_scope_id)
                .is_some_and(|scope| !scope.is_attached())
    }

    pub(crate) fn remove_handles(
        &mut self,
        removed_handles: &HashSet<FocusHandleId>,
        removed_requesters: &HashSet<FocusRequesterId>,
    ) {
        if removed_handles.is_empty() && removed_requesters.is_empty() {
            return;
        }

        let previous_states = self.snapshot_live_states();
        let previous_requester_bindings = self.requester_bindings.clone();

        for requester_id in removed_requesters {
            self.requester_bindings.remove(requester_id);
        }
        for requesters in self.requesters_by_handle.values_mut() {
            requesters.retain(|requester_id| !removed_requesters.contains(requester_id));
        }

        let active_was_removed = self.active.filter(|id| removed_handles.contains(id));
        if active_was_removed.is_some() {
            self.active = active_was_removed.and_then(|id| self.restore_from_detached(id));
            self.captured = None;
            if let Some(active) = self.active {
                self.update_scope_restore_chain(active);
                self.pending_reveal = Some(active);
            } else {
                self.pending_reveal = None;
            }
        }

        if self
            .captured
            .is_some_and(|id| removed_handles.contains(&id))
        {
            self.captured = None;
        }
        if self
            .suspended
            .is_some_and(|id| removed_handles.contains(&id))
        {
            self.suspended = None;
        }

        for handle_id in removed_handles {
            self.nodes.remove(handle_id);
            self.children_by_parent.remove(handle_id);
            self.requesters_by_handle.remove(handle_id);
        }
        for children in self.children_by_parent.values_mut() {
            children.retain(|child| !removed_handles.contains(child));
        }
        for node in self.nodes.values_mut() {
            if node
                .last_focused_descendant
                .is_some_and(|candidate| removed_handles.contains(&candidate))
            {
                node.last_focused_descendant = None;
            }
            if node
                .restorer_fallback
                .is_some_and(|requester_id| removed_requesters.contains(&requester_id))
            {
                node.restorer_fallback = None;
            }
        }

        self.recompute_states();
        self.notify_state_changes(previous_states);
        self.notify_requester_binding_changes(previous_requester_bindings);
    }

    fn apply_command(&mut self, command: FocusCommand) -> bool {
        let previous_states = self.snapshot_live_states();
        let applied = self.apply_command_inner(command);
        self.recompute_states();
        self.notify_state_changes(previous_states);
        applied
    }

    fn apply_command_inner(&mut self, command: FocusCommand) -> bool {
        match command {
            FocusCommand::Request(id) => {
                if !self.owner_focused {
                    self.suspended = Some(id);
                    return true;
                }
                let Some(target) = self.resolve_request_target(id) else {
                    return false;
                };
                self.set_active(target)
            }
            FocusCommand::Clear(id) => self.clear_active_for(id),
            FocusCommand::Capture(id) => {
                if self.active == Some(id)
                    && self.nodes.get(&id).is_some_and(|node| node.is_attached())
                {
                    self.captured = Some(id);
                    true
                } else {
                    false
                }
            }
            FocusCommand::Free(id) => {
                if self.captured == Some(id) {
                    self.captured = None;
                    true
                } else {
                    false
                }
            }
            FocusCommand::Restore(id) => {
                let Some(target) = self.resolve_restore_candidate(id) else {
                    return false;
                };
                self.set_active(target)
            }
        }
    }

    fn set_active(&mut self, target: FocusHandleId) -> bool {
        if self.captured.is_some() && self.captured != Some(target) {
            return false;
        }
        let changed = self.active != Some(target) || self.captured.is_some_and(|id| id != target);
        self.active = Some(target);
        if self.captured.is_some_and(|id| id != target) {
            self.captured = None;
        }
        self.update_scope_restore_chain(target);
        if changed {
            self.pending_reveal = Some(target);
        }
        changed
    }

    fn clear_active_for(&mut self, id: FocusHandleId) -> bool {
        let Some(active) = self.active else {
            return false;
        };
        let affects_active = active == id || self.is_ancestor(id, active);
        if !affects_active {
            return false;
        }
        if self.captured.is_some() {
            return false;
        }

        self.active = None;
        self.captured = None;
        self.pending_reveal = None;
        true
    }

    fn resolve_move_target(&self, direction: FocusDirection) -> Option<FocusHandleId> {
        if let Some(active) = self.active
            && let Some(explicit) = self.resolve_explicit_direction_target(active, direction)
        {
            return Some(explicit);
        }

        if matches!(
            direction,
            FocusDirection::Left
                | FocusDirection::Right
                | FocusDirection::Up
                | FocusDirection::Down
        ) && let Some(target) = self.resolve_geometric_move_target(direction)
        {
            return Some(target);
        }

        let candidates = self.traversable_focus_handles();
        if candidates.is_empty() {
            return None;
        }

        let forward = matches!(
            direction,
            FocusDirection::Next
                | FocusDirection::Down
                | FocusDirection::Right
                | FocusDirection::Enter
        );

        let Some(active) = self
            .active
            .and_then(|current| self.resolve_request_target(current))
        else {
            return if forward {
                candidates.first().copied()
            } else {
                candidates.last().copied()
            };
        };

        let Some(index) = candidates.iter().position(|candidate| *candidate == active) else {
            return if forward {
                candidates.first().copied()
            } else {
                candidates.last().copied()
            };
        };

        if forward {
            candidates.get(index + 1).copied()
        } else if index > 0 {
            candidates.get(index - 1).copied()
        } else {
            None
        }
    }

    fn resolve_move_target_in_scope(
        &self,
        scope_id: FocusHandleId,
        direction: FocusDirection,
        wrap: bool,
    ) -> Option<FocusHandleId> {
        let candidates = self.traversable_focus_handles_in_scope(scope_id);
        if candidates.is_empty() {
            return None;
        }

        let active = self
            .active
            .and_then(|current| self.resolve_request_target(current))
            .filter(|current| self.is_ancestor(scope_id, *current));

        if let Some(active) = active
            && let Some(explicit) = self.resolve_explicit_direction_target(active, direction)
            && self.is_ancestor(scope_id, explicit)
        {
            return Some(explicit);
        }

        if matches!(
            direction,
            FocusDirection::Left
                | FocusDirection::Right
                | FocusDirection::Up
                | FocusDirection::Down
        ) && let Some(target) =
            self.resolve_geometric_move_target_in_candidates(direction, &candidates, active)
        {
            return Some(target);
        }

        let forward = matches!(
            direction,
            FocusDirection::Next
                | FocusDirection::Down
                | FocusDirection::Right
                | FocusDirection::Enter
        );

        let Some(active) = active else {
            return if forward {
                candidates.first().copied()
            } else {
                candidates.last().copied()
            };
        };

        let Some(index) = candidates.iter().position(|candidate| *candidate == active) else {
            return if forward {
                candidates.first().copied()
            } else {
                candidates.last().copied()
            };
        };

        if forward {
            candidates
                .get(index + 1)
                .copied()
                .or_else(|| wrap.then_some(candidates.first().copied()).flatten())
        } else if index > 0 {
            candidates.get(index - 1).copied()
        } else if wrap {
            candidates.last().copied()
        } else {
            None
        }
    }

    fn resolve_geometric_move_target(&self, direction: FocusDirection) -> Option<FocusHandleId> {
        let active = self.active.and_then(|id| self.resolve_request_target(id))?;
        let focused_rect = self.nodes.get(&active)?.attachment?.focus_rect?;
        let mut best_candidate = initial_best_candidate_rect(focused_rect, direction);
        let mut best_handle = None;

        for handle_id in self.traversable_focus_handles() {
            if handle_id == active {
                continue;
            }
            let Some(candidate_rect) = self
                .nodes
                .get(&handle_id)
                .and_then(|node| node.attachment)
                .and_then(|attachment| attachment.focus_rect)
            else {
                continue;
            };
            if !is_eligible_focus_rect(candidate_rect) {
                continue;
            }
            if is_better_focus_candidate(candidate_rect, best_candidate, focused_rect, direction) {
                best_candidate = candidate_rect;
                best_handle = Some(handle_id);
            }
        }

        best_handle
    }

    fn resolve_geometric_move_target_in_candidates(
        &self,
        direction: FocusDirection,
        candidates: &[FocusHandleId],
        active: Option<FocusHandleId>,
    ) -> Option<FocusHandleId> {
        let active = active?;
        let focused_rect = self.nodes.get(&active)?.attachment?.focus_rect?;
        let mut best_candidate = initial_best_candidate_rect(focused_rect, direction);
        let mut best_handle = None;

        for &handle_id in candidates {
            if handle_id == active {
                continue;
            }
            let Some(candidate_rect) = self
                .nodes
                .get(&handle_id)
                .and_then(|node| node.attachment)
                .and_then(|attachment| attachment.focus_rect)
            else {
                continue;
            };
            if !is_eligible_focus_rect(candidate_rect) {
                continue;
            }
            if is_better_focus_candidate(candidate_rect, best_candidate, focused_rect, direction) {
                best_candidate = candidate_rect;
                best_handle = Some(handle_id);
            }
        }

        best_handle
    }

    fn resolve_request_target(&self, id: FocusHandleId) -> Option<FocusHandleId> {
        let node = self.nodes.get(&id)?;
        if !node.is_attached() {
            return None;
        }
        match node.kind {
            FocusRegistrationKind::Target => self.can_request_focus(id).then_some(id),
            FocusRegistrationKind::Scope => self
                .resolve_restore_candidate(id)
                .or_else(|| self.first_focusable_descendant(id))
                .or_else(|| self.can_request_focus(id).then_some(id)),
            FocusRegistrationKind::Group => self
                .first_focusable_descendant(id)
                .or_else(|| self.can_request_focus(id).then_some(id)),
        }
    }

    fn resolve_requester_target(&self, requester_id: FocusRequesterId) -> Option<FocusHandleId> {
        self.requester_bindings.get(&requester_id).copied()
    }

    fn resolve_explicit_direction_target(
        &self,
        id: FocusHandleId,
        direction: FocusDirection,
    ) -> Option<FocusHandleId> {
        let requester = self.nodes.get(&id).and_then(|node| match direction {
            FocusDirection::Next => node.props.next,
            FocusDirection::Previous => node.props.previous,
            FocusDirection::Up => node.props.up,
            FocusDirection::Down => node.props.down,
            FocusDirection::Left => node.props.left,
            FocusDirection::Right => node.props.right,
            FocusDirection::Enter => node.props.enter,
            FocusDirection::Exit => node.props.exit,
        })?;
        let handle_id = self.resolve_requester_target(requester.requester_id())?;
        self.resolve_request_target(handle_id)
    }

    fn resolve_restore_candidate(&self, id: FocusHandleId) -> Option<FocusHandleId> {
        let scope = self.nodes.get(&id)?;
        if !scope.is_attached() || scope.kind != FocusRegistrationKind::Scope {
            return None;
        }
        scope
            .last_focused_descendant
            .filter(|candidate| self.can_request_focus(*candidate))
            .or_else(|| {
                scope
                    .restorer_fallback
                    .and_then(|requester_id| self.resolve_requester_target(requester_id))
                    .and_then(|candidate| self.resolve_request_target(candidate))
            })
            .or_else(|| self.first_focusable_descendant(id))
    }

    fn traversable_focus_handles(&self) -> Vec<FocusHandleId> {
        let mut ordered = Vec::new();
        self.collect_traversable_focus_handles(ROOT_SCOPE_ID, &mut ordered);
        ordered
    }

    fn traversable_focus_handles_in_scope(&self, scope_id: FocusHandleId) -> Vec<FocusHandleId> {
        let mut ordered = Vec::new();
        self.collect_traversable_focus_handles(scope_id, &mut ordered);
        ordered
    }

    fn collect_traversable_focus_handles(
        &self,
        parent: FocusHandleId,
        ordered: &mut Vec<FocusHandleId>,
    ) {
        let Some(children) = self.children_by_parent.get(&parent) else {
            return;
        };
        for &child in children {
            if self.is_traversable_focus_handle(child) {
                ordered.push(child);
            }
            self.collect_traversable_focus_handles(child, ordered);
        }
    }

    fn is_traversable_focus_handle(&self, id: FocusHandleId) -> bool {
        self.nodes.get(&id).is_some_and(|node| {
            node.is_attached()
                && !node.props.skip_traversal
                && node.props.can_focus
                && node.props.can_request_focus
        })
    }

    fn first_focusable_descendant(&self, parent: FocusHandleId) -> Option<FocusHandleId> {
        let mut stack = self
            .children_by_parent
            .get(&parent)
            .cloned()
            .unwrap_or_default();
        while let Some(id) = stack.first().copied() {
            stack.remove(0);
            if self.can_request_focus(id) {
                return Some(id);
            }
            if let Some(children) = self.children_by_parent.get(&id) {
                let mut descendants = children.clone();
                descendants.extend(stack);
                stack = descendants;
            }
        }
        None
    }

    fn restore_from_detached(&self, id: FocusHandleId) -> Option<FocusHandleId> {
        let mut current_scope = self.nodes.get(&id).map(|node| node.last_scope_parent)?;
        loop {
            if let Some(candidate) = self.resolve_restore_candidate(current_scope) {
                return Some(candidate);
            }
            if current_scope == ROOT_SCOPE_ID {
                break;
            }
            current_scope = self
                .nodes
                .get(&current_scope)
                .map(|node| node.last_scope_parent)
                .unwrap_or(ROOT_SCOPE_ID);
        }
        None
    }

    fn update_scope_restore_chain(&mut self, target: FocusHandleId) {
        let mut current = Some(target);
        while let Some(id) = current {
            if let Some(node) = self.nodes.get_mut(&id)
                && node.kind == FocusRegistrationKind::Scope
            {
                node.last_focused_descendant = Some(target);
            }
            if id == ROOT_SCOPE_ID {
                break;
            }
            current = self
                .nodes
                .get(&id)
                .and_then(|node| node.attachment)
                .map(|attachment| attachment.scope_parent);
            if current == Some(id) {
                break;
            }
        }
        if let Some(root) = self.nodes.get_mut(&ROOT_SCOPE_ID) {
            root.last_focused_descendant = Some(target);
        }
    }

    fn snapshot_live_states(&self) -> HashMap<FocusHandleId, FocusState> {
        self.nodes
            .iter()
            .filter_map(|(&id, node)| node.is_attached().then_some((id, node.state)))
            .collect()
    }

    fn notify_requester_binding_changes(
        &self,
        previous_bindings: HashMap<FocusRequesterId, FocusHandleId>,
    ) {
        let mut changed = HashSet::default();
        for (&requester_id, &previous_handle_id) in &previous_bindings {
            let current = self.requester_bindings.get(&requester_id).copied();
            if current != Some(previous_handle_id) {
                changed.insert(requester_id);
            }
        }
        for (&requester_id, &current_handle_id) in &self.requester_bindings {
            let previous = previous_bindings.get(&requester_id).copied();
            if previous != Some(current_handle_id) {
                changed.insert(requester_id);
            }
        }
        for requester_id in changed {
            for reader in focus_requester_read_subscribers(requester_id) {
                record_component_invalidation_for_instance_key(reader);
            }
        }
    }

    fn notify_state_changes(&mut self, previous_states: HashMap<FocusHandleId, FocusState>) {
        let mut changed = HashSet::default();
        let mut events = HashSet::default();
        let mut changed_requesters = HashSet::default();

        for (&id, &previous) in &previous_states {
            let current = self
                .nodes
                .get(&id)
                .filter(|node| node.is_attached())
                .map(|node| node.state)
                .unwrap_or(FocusState::Inactive);
            if previous != current {
                changed.insert(id);
            }
            if previous.has_focus() {
                events.insert(id);
            }
        }

        for (&id, node) in &self.nodes {
            if !node.is_attached() {
                continue;
            }
            let previous = previous_states
                .get(&id)
                .copied()
                .unwrap_or(FocusState::Inactive);
            if previous != node.state {
                changed.insert(id);
            }
            if node.state.has_focus() {
                events.insert(id);
            }
        }

        for &id in &changed {
            let state = self
                .nodes
                .get(&id)
                .filter(|node| node.is_attached())
                .map(|node| node.state)
                .unwrap_or(FocusState::Inactive);
            self.pending_notifications.push(FocusNotification {
                handle_id: id,
                state,
                changed: true,
            });
            for reader in focus_read_subscribers(id) {
                record_component_invalidation_for_instance_key(reader);
            }
            if let Some(node) = self.nodes.get(&id)
                && let Some(attachment) = node.attachment
                && attachment.host_instance_key != 0
            {
                record_component_invalidation_for_instance_key(attachment.host_instance_key);
            }
            if let Some(requesters) = self.requesters_by_handle.get(&id) {
                changed_requesters.extend(requesters.iter().copied());
            }
        }

        for id in events {
            if changed.contains(&id) {
                continue;
            }
            let state = self
                .nodes
                .get(&id)
                .filter(|node| node.is_attached())
                .map(|node| node.state)
                .unwrap_or(FocusState::Inactive);
            self.pending_notifications.push(FocusNotification {
                handle_id: id,
                state,
                changed: false,
            });
        }

        for requester_id in changed_requesters {
            for reader in focus_requester_read_subscribers(requester_id) {
                record_component_invalidation_for_instance_key(reader);
            }
        }
    }

    fn recompute_states(&mut self) {
        for node in self.nodes.values_mut() {
            if node.is_attached() {
                node.state = FocusState::Inactive;
            }
        }
        let Some(active) = self.active else {
            return;
        };
        if !self
            .nodes
            .get(&active)
            .is_some_and(|node| node.is_attached())
        {
            return;
        }

        let mut current = Some(active);
        let active_state = if self.captured == Some(active) {
            FocusState::Captured
        } else {
            FocusState::Active
        };
        let mut first = true;
        while let Some(id) = current {
            if let Some(node) = self.nodes.get_mut(&id)
                && node.is_attached()
            {
                node.state = if first {
                    active_state
                } else {
                    FocusState::ActiveParent
                };
            }
            if id == ROOT_SCOPE_ID {
                break;
            }
            current = self
                .nodes
                .get(&id)
                .and_then(|node| node.attachment)
                .map(|attachment| attachment.parent);
            first = false;
        }
    }

    fn can_focus(&self, id: FocusHandleId) -> bool {
        self.nodes
            .get(&id)
            .is_some_and(|node| node.is_attached() && node.props.can_focus)
    }

    fn can_request_focus(&self, id: FocusHandleId) -> bool {
        self.nodes.get(&id).is_some_and(|node| {
            node.is_attached() && node.props.can_focus && node.props.can_request_focus
        })
    }

    fn is_ancestor(&self, ancestor: FocusHandleId, descendant: FocusHandleId) -> bool {
        let mut current = Some(descendant);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            if id == ROOT_SCOPE_ID {
                break;
            }
            current = self
                .nodes
                .get(&id)
                .and_then(|node| node.attachment)
                .map(|attachment| attachment.parent);
        }
        false
    }
}

fn is_eligible_focus_rect(rect: PxRect) -> bool {
    rect.width.0 > 0 && rect.height.0 > 0
}

fn initial_best_candidate_rect(focused_rect: PxRect, direction: FocusDirection) -> PxRect {
    match direction {
        FocusDirection::Left => PxRect::new(
            focused_rect.x + focused_rect.width + crate::Px::new(1),
            focused_rect.y,
            focused_rect.width,
            focused_rect.height,
        ),
        FocusDirection::Right => PxRect::new(
            focused_rect.x - focused_rect.width - crate::Px::new(1),
            focused_rect.y,
            focused_rect.width,
            focused_rect.height,
        ),
        FocusDirection::Up => PxRect::new(
            focused_rect.x,
            focused_rect.y + focused_rect.height + crate::Px::new(1),
            focused_rect.width,
            focused_rect.height,
        ),
        FocusDirection::Down => PxRect::new(
            focused_rect.x,
            focused_rect.y - focused_rect.height - crate::Px::new(1),
            focused_rect.width,
            focused_rect.height,
        ),
        _ => focused_rect,
    }
}

fn is_better_focus_candidate(
    proposed_candidate: PxRect,
    current_candidate: PxRect,
    focused_rect: PxRect,
    direction: FocusDirection,
) -> bool {
    if !is_focus_candidate(proposed_candidate, focused_rect, direction) {
        return false;
    }
    if !is_focus_candidate(current_candidate, focused_rect, direction) {
        return true;
    }
    if focus_beam_beats(
        focused_rect,
        proposed_candidate,
        current_candidate,
        direction,
    ) {
        return true;
    }
    if focus_beam_beats(
        focused_rect,
        current_candidate,
        proposed_candidate,
        direction,
    ) {
        return false;
    }
    weighted_focus_distance(proposed_candidate, focused_rect, direction)
        < weighted_focus_distance(current_candidate, focused_rect, direction)
}

fn is_focus_candidate(rect: PxRect, focused_rect: PxRect, direction: FocusDirection) -> bool {
    match direction {
        FocusDirection::Left => {
            (focused_rect_right(focused_rect) > focused_rect_right(rect)
                || focused_rect.x >= focused_rect_right(rect))
                && focused_rect.x > rect.x
        }
        FocusDirection::Right => {
            (focused_rect.x < rect.x || focused_rect_right(focused_rect) <= rect.x)
                && focused_rect_right(focused_rect) < focused_rect_right(rect)
        }
        FocusDirection::Up => {
            (focused_rect_bottom(focused_rect) > focused_rect_bottom(rect)
                || focused_rect.y >= focused_rect_bottom(rect))
                && focused_rect.y > rect.y
        }
        FocusDirection::Down => {
            (focused_rect.y < rect.y || focused_rect_bottom(focused_rect) <= rect.y)
                && focused_rect_bottom(focused_rect) < focused_rect_bottom(rect)
        }
        _ => false,
    }
}

fn weighted_focus_distance(rect: PxRect, focused_rect: PxRect, direction: FocusDirection) -> i64 {
    let major = major_axis_distance(rect, focused_rect, direction);
    let minor = minor_axis_distance(rect, focused_rect, direction);
    13 * major * major + minor * minor
}

fn focus_beam_beats(
    source: PxRect,
    rect1: PxRect,
    rect2: PxRect,
    direction: FocusDirection,
) -> bool {
    if rect2_in_source_beam(rect2, source, direction)
        || !rect2_in_source_beam(rect1, source, direction)
    {
        return false;
    }
    if !is_in_direction_of_search(rect2, source, direction) {
        return true;
    }
    if matches!(direction, FocusDirection::Left | FocusDirection::Right) {
        return true;
    }
    major_axis_distance(rect1, source, direction)
        < major_axis_distance_to_far_edge(rect2, source, direction)
}

fn rect2_in_source_beam(rect: PxRect, source: PxRect, direction: FocusDirection) -> bool {
    match direction {
        FocusDirection::Left | FocusDirection::Right => {
            focused_rect_bottom(rect) > source.y && rect.y < focused_rect_bottom(source)
        }
        FocusDirection::Up | FocusDirection::Down => {
            focused_rect_right(rect) > source.x && rect.x < focused_rect_right(source)
        }
        _ => false,
    }
}

fn is_in_direction_of_search(rect: PxRect, source: PxRect, direction: FocusDirection) -> bool {
    match direction {
        FocusDirection::Left => source.x >= focused_rect_right(rect),
        FocusDirection::Right => focused_rect_right(source) <= rect.x,
        FocusDirection::Up => source.y >= focused_rect_bottom(rect),
        FocusDirection::Down => focused_rect_bottom(source) <= rect.y,
        _ => false,
    }
}

fn major_axis_distance(rect: PxRect, focused_rect: PxRect, direction: FocusDirection) -> i64 {
    let distance = match direction {
        FocusDirection::Left => focused_rect.x.0 - focused_rect_right(rect).0,
        FocusDirection::Right => rect.x.0 - focused_rect_right(focused_rect).0,
        FocusDirection::Up => focused_rect.y.0 - focused_rect_bottom(rect).0,
        FocusDirection::Down => rect.y.0 - focused_rect_bottom(focused_rect).0,
        _ => 0,
    };
    i64::from(distance.max(0))
}

fn major_axis_distance_to_far_edge(rect: PxRect, source: PxRect, direction: FocusDirection) -> i64 {
    let distance = match direction {
        FocusDirection::Left => source.x.0 - rect.x.0,
        FocusDirection::Right => focused_rect_right(rect).0 - focused_rect_right(source).0,
        FocusDirection::Up => source.y.0 - rect.y.0,
        FocusDirection::Down => focused_rect_bottom(rect).0 - focused_rect_bottom(source).0,
        _ => 1,
    };
    i64::from(distance.max(1))
}

fn minor_axis_distance(rect: PxRect, focused_rect: PxRect, direction: FocusDirection) -> i64 {
    let focused_center = match direction {
        FocusDirection::Left | FocusDirection::Right => {
            focused_rect.y.0 + focused_rect.height.0 / 2
        }
        FocusDirection::Up | FocusDirection::Down => focused_rect.x.0 + focused_rect.width.0 / 2,
        _ => 0,
    };
    let candidate_center = match direction {
        FocusDirection::Left | FocusDirection::Right => rect.y.0 + rect.height.0 / 2,
        FocusDirection::Up | FocusDirection::Down => rect.x.0 + rect.width.0 / 2,
        _ => 0,
    };
    i64::from(focused_center - candidate_center)
}

fn focused_rect_right(rect: PxRect) -> crate::Px {
    rect.x + rect.width
}

fn focused_rect_bottom(rect: PxRect) -> crate::Px {
    rect.y + rect.height
}

pub(crate) fn flush_pending_focus_callbacks() {
    let callbacks = TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .take_pending_focus_callback_invocations()
    });
    for callback in callbacks {
        callback.invoke();
    }
}
