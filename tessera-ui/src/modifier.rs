//! Modifier chains for layout, focus, and drawing behavior.
//!
//! ## Usage
//!
//! Build reusable modifier chains that wrap component subtrees with shared
//! behavior.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use smallvec::SmallVec;

use crate::{
    FocusGroupNode, FocusNode, FocusProperties, FocusRequester, FocusScopeNode, FocusState,
    FocusTraversalPolicy,
    prop::CallbackWith,
    runtime::{
        TesseraRuntime, ensure_build_phase, persistent_focus_group_for_current_instance,
        persistent_focus_scope_for_current_instance, persistent_focus_target_for_current_instance,
    },
};

/// A child subtree builder used by modifier wrappers.
pub type ModifierChild = Box<dyn Fn() + Send + Sync + 'static>;

/// A wrapper function that receives a child builder and returns a wrapped child
/// builder.
pub type ModifierWrapper = CallbackWith<ModifierChild, ModifierChild>;

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

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct FocusModifierConfig {
    pub requester: Option<FocusRequester>,
    pub registration: Option<FocusModifierRegistration>,
    pub properties: Option<FocusProperties>,
    pub traversal_policy: Option<FocusTraversalPolicy>,
    pub changed_handler: Option<CallbackWith<FocusState>>,
    pub event_handler: Option<CallbackWith<FocusState>>,
}

impl FocusModifierConfig {
    fn merge(&mut self, op: FocusModifierOp) {
        match op {
            FocusModifierOp::Requester(requester) => {
                self.requester = Some(requester);
            }
            FocusModifierOp::Registration(registration) => {
                self.registration = Some(registration);
            }
            FocusModifierOp::Properties(properties) => {
                self.properties = Some(properties);
            }
            FocusModifierOp::TraversalPolicy(policy) => {
                self.traversal_policy = Some(policy);
            }
            FocusModifierOp::ChangedHandler(handler) => {
                self.changed_handler = Some(handler);
            }
            FocusModifierOp::EventHandler(handler) => {
                self.event_handler = Some(handler);
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum FocusModifierOp {
    Requester(FocusRequester),
    Registration(FocusModifierRegistration),
    Properties(FocusProperties),
    TraversalPolicy(FocusTraversalPolicy),
    ChangedHandler(CallbackWith<FocusState>),
    EventHandler(CallbackWith<FocusState>),
}

#[derive(Clone)]
enum ModifierAction {
    Wrapper(ModifierWrapper),
    Focus(FocusModifierOp),
}

#[derive(Clone)]
struct ModifierNode {
    prev: Option<Arc<ModifierNode>>,
    action: ModifierAction,
}

fn collect_actions(mut node: Option<Arc<ModifierNode>>) -> Vec<ModifierAction> {
    let mut actions: SmallVec<[ModifierAction; 8]> = SmallVec::new();
    while let Some(current) = node {
        actions.push(current.action.clone());
        node = current.prev.clone();
    }
    actions.into_vec()
}

/// A persistent handle to a modifier chain.
///
/// The modifier chain is immutable and replay-safe across frames.
#[derive(Clone, Default)]
pub struct Modifier {
    tail: Option<Arc<ModifierNode>>,
}

/// Focus-specific modifier extensions for [`Modifier`].
///
/// These APIs live in the core crate so focus behavior can be reused without
/// depending on the component library.
///
/// Typical usage is to remember a [`crate::FocusRequester`], bind it to a
/// modifier, and then declare the current subtree as focusable.
///
/// # Examples
///
/// ```
/// use tessera_ui::modifier::FocusModifierExt as _;
/// use tessera_ui::{FocusProperties, FocusRequester, Modifier, remember, tessera};
///
/// #[tessera]
/// fn focusable_field() {
///     let requester = remember(FocusRequester::new).get();
///     let _modifier = Modifier::new()
///         .focus_requester(requester)
///         .focusable()
///         .focus_properties(FocusProperties::new().can_focus(true))
///         .on_focus_changed(|state: tessera_ui::FocusState| {
///             let _ = state.is_focused();
///         });
/// }
/// ```
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
}

impl Modifier {
    /// Creates an empty modifier chain.
    pub fn new() -> Self {
        ensure_build_phase();
        Self::default()
    }

    /// Appends a wrapper action and returns the updated modifier.
    pub fn push_wrapper<F, Child>(self, wrapper: F) -> Self
    where
        F: Fn(ModifierChild) -> Child + Send + Sync + 'static,
        Child: Fn() + Send + Sync + 'static,
    {
        self.push_wrapper_shared(CallbackWith::new(
            move |child: ModifierChild| -> ModifierChild { Box::new(wrapper(child)) },
        ))
    }

    fn push_wrapper_shared(self, wrapper: ModifierWrapper) -> Self {
        ensure_build_phase();
        Self {
            tail: Some(Arc::new(ModifierNode {
                prev: self.tail,
                action: ModifierAction::Wrapper(wrapper),
            })),
        }
    }

    #[doc(hidden)]
    pub fn push_focus_requester(self, requester: FocusRequester) -> Self {
        self.push_focus_op(FocusModifierOp::Requester(requester))
    }

    #[doc(hidden)]
    pub fn push_focus_target(self) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Target(None),
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_target_with(self, node: FocusNode) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Target(Some(node)),
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_scope(self) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Scope(None),
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_scope_with(self, scope: FocusScopeNode) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Scope(Some(scope)),
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_group(self) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Group(None),
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_group_with(self, group: FocusGroupNode) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Group(Some(group)),
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_restorer(self, fallback: Option<FocusRequester>) -> Self {
        self.push_focus_op(FocusModifierOp::Registration(
            FocusModifierRegistration::Restorer {
                scope: None,
                fallback,
            },
        ))
    }

    #[doc(hidden)]
    pub fn push_focus_restorer_with(
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

    #[doc(hidden)]
    pub fn push_focus_properties(self, properties: FocusProperties) -> Self {
        self.push_focus_op(FocusModifierOp::Properties(properties))
    }

    #[doc(hidden)]
    pub fn push_focus_traversal_policy(self, policy: FocusTraversalPolicy) -> Self {
        self.push_focus_op(FocusModifierOp::TraversalPolicy(policy))
    }

    #[doc(hidden)]
    pub fn push_focus_changed_handler(self, handler: CallbackWith<FocusState>) -> Self {
        self.push_focus_op(FocusModifierOp::ChangedHandler(handler))
    }

    #[doc(hidden)]
    pub fn push_focus_event_handler(self, handler: CallbackWith<FocusState>) -> Self {
        self.push_focus_op(FocusModifierOp::EventHandler(handler))
    }

    fn push_focus_op(self, op: FocusModifierOp) -> Self {
        ensure_build_phase();
        Self {
            tail: Some(Arc::new(ModifierNode {
                prev: self.tail,
                action: ModifierAction::Focus(op),
            })),
        }
    }

    /// Applies all wrappers to `child` and returns the wrapped child builder.
    pub fn apply<F>(self, child: F) -> ModifierChild
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.apply_child(Box::new(child))
    }

    fn apply_child(self, child: ModifierChild) -> ModifierChild {
        ensure_build_phase();

        if self.tail.is_none() {
            return child;
        }

        let actions = collect_actions(self.tail);
        let mut child = child;
        let mut pending_focus = FocusModifierConfig::default();
        let mut has_pending_focus = false;

        for action in actions.into_iter().rev() {
            match action {
                ModifierAction::Focus(op) => {
                    pending_focus.merge(op);
                    has_pending_focus = true;
                }
                ModifierAction::Wrapper(wrapper) => {
                    if has_pending_focus {
                        child = apply_focus_modifier(pending_focus, child);
                        pending_focus = FocusModifierConfig::default();
                        has_pending_focus = false;
                    }
                    child = wrapper.call(child);
                }
            }
        }

        if has_pending_focus {
            child = apply_focus_modifier(pending_focus, child);
        }

        child
    }

    /// Applies all wrappers to `child` and immediately runs the resulting tree.
    pub fn run<F>(self, child: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let child = self.apply(child);
        child();
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
            Some(node) => {
                // Pointer identity is the stable key for modifier replay/prop comparison.
                std::ptr::hash(Arc::as_ptr(node), state);
            }
            None => {
                0u8.hash(state);
            }
        }
    }
}

fn apply_focus_modifier(config: FocusModifierConfig, child: ModifierChild) -> ModifierChild {
    Box::new(move || {
        TesseraRuntime::with_mut(|runtime| {
            if let Some(requester) = config.requester {
                runtime.bind_current_focus_requester(requester);
            }

            match config.registration {
                Some(FocusModifierRegistration::Target(node)) => {
                    let node = node.unwrap_or_else(|| {
                        runtime.current_focus_target_handle().unwrap_or_else(|| {
                            persistent_focus_target_for_current_instance("__tessera_focus_target")
                        })
                    });
                    runtime.ensure_current_focus_target(node);
                }
                Some(FocusModifierRegistration::Scope(scope)) => {
                    let scope = scope.unwrap_or_else(|| {
                        runtime.current_focus_scope_handle().unwrap_or_else(|| {
                            persistent_focus_scope_for_current_instance("__tessera_focus_scope")
                        })
                    });
                    runtime.ensure_current_focus_scope(scope);
                }
                Some(FocusModifierRegistration::Group(group)) => {
                    let group = group.unwrap_or_else(|| {
                        runtime.current_focus_group_handle().unwrap_or_else(|| {
                            persistent_focus_group_for_current_instance("__tessera_focus_group")
                        })
                    });
                    runtime.ensure_current_focus_group(group);
                }
                Some(FocusModifierRegistration::Restorer { scope, fallback }) => {
                    let scope = scope.unwrap_or_else(|| {
                        runtime.current_focus_scope_handle().unwrap_or_else(|| {
                            persistent_focus_scope_for_current_instance("__tessera_focus_scope")
                        })
                    });
                    runtime.ensure_current_focus_scope(scope);
                    if let Some(fallback) = fallback {
                        runtime.set_current_focus_restorer_fallback(fallback);
                    }
                }
                None => {}
            }

            if let Some(properties) = config.properties {
                runtime.set_current_focus_properties(properties);
            }
            if let Some(policy) = config.traversal_policy {
                runtime.set_current_focus_traversal_policy(policy);
            }
            if let Some(handler) = config.changed_handler {
                runtime.set_current_focus_changed_handler(handler);
            }
            if let Some(handler) = config.event_handler {
                runtime.set_current_focus_event_handler(handler);
            }
        });
        child();
    })
}
