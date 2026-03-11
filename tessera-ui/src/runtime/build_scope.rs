use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use rustc_hash::FxHashSet as HashSet;

use crate::{
    NodeId,
    execution_context::{
        OrderCounterKind, next_child_instance_call_index, next_order_counter, pop_order_frame,
        push_order_frame, with_execution_context, with_execution_context_mut,
    },
};

pub(crate) fn current_component_instance_key_from_scope() -> Option<u64> {
    with_execution_context(|context| context.current_component_instance_stack.last().copied())
}

pub(crate) fn take_next_node_instance_logic_id_override() -> Option<u64> {
    with_execution_context_mut(|context| context.next_node_instance_logic_id_override.take())
}

/// Runs `f` inside a replay scope restored from a previously recorded component
/// snapshot.
///
/// The replay scope restores:
/// - the control-flow group path active at the original call site
/// - the keyed-instance override active at the original call site
/// - a one-shot instance-logic-id override for the replayed component root
pub(crate) fn with_replay_scope<R>(
    instance_logic_id: u64,
    group_path: &[u64],
    instance_key_override: Option<u64>,
    f: impl FnOnce() -> R,
) -> R {
    struct ReplayScopeGuard {
        previous_group_path: Option<Vec<u64>>,
        previous_instance_key_stack: Option<Vec<u64>>,
        previous_instance_logic_id_override: Option<Option<u64>>,
    }

    impl Drop for ReplayScopeGuard {
        fn drop(&mut self) {
            if let Some(previous_group_path) = self.previous_group_path.take() {
                with_execution_context_mut(|context| {
                    context.group_path_stack = previous_group_path;
                });
            }
            if let Some(previous_instance_key_stack) = self.previous_instance_key_stack.take() {
                with_execution_context_mut(|context| {
                    context.instance_key_stack = previous_instance_key_stack;
                });
            }
            if let Some(previous_instance_logic_id_override) =
                self.previous_instance_logic_id_override.take()
            {
                with_execution_context_mut(|context| {
                    context.next_node_instance_logic_id_override =
                        previous_instance_logic_id_override;
                });
            }
        }
    }

    let previous_group_path = with_execution_context_mut(|context| {
        std::mem::replace(&mut context.group_path_stack, group_path.to_vec())
    });
    let previous_instance_key_stack = with_execution_context_mut(|context| {
        let next_stack = instance_key_override.into_iter().collect::<Vec<_>>();
        std::mem::replace(&mut context.instance_key_stack, next_stack)
    });
    let previous_instance_logic_id_override = with_execution_context_mut(|context| {
        context
            .next_node_instance_logic_id_override
            .replace(instance_logic_id)
    });
    let _guard = ReplayScopeGuard {
        previous_group_path: Some(previous_group_path),
        previous_instance_key_stack: Some(previous_instance_key_stack),
        previous_instance_logic_id_override: Some(previous_instance_logic_id_override),
    };

    f()
}

pub(crate) fn with_build_dirty_instance_keys<R>(
    dirty_instance_keys: &HashSet<u64>,
    f: impl FnOnce() -> R,
) -> R {
    struct BuildDirtyScopeGuard {
        popped: bool,
    }

    impl Drop for BuildDirtyScopeGuard {
        fn drop(&mut self) {
            if self.popped {
                return;
            }
            with_execution_context_mut(|context| {
                let popped = context.build_dirty_instance_keys_stack.pop();
                debug_assert!(
                    popped.is_some(),
                    "BUILD_DIRTY_INSTANCE_KEYS_STACK underflow: attempted to pop from empty stack"
                );
            });
            self.popped = true;
        }
    }

    with_execution_context_mut(|context| {
        context
            .build_dirty_instance_keys_stack
            .push(Arc::new(dirty_instance_keys.clone()));
    });
    let _guard = BuildDirtyScopeGuard { popped: false };
    f()
}

pub(crate) fn is_instance_key_build_dirty(instance_key: u64) -> bool {
    with_execution_context(|context| {
        context
            .build_dirty_instance_keys_stack
            .last()
            .is_some_and(|dirty_instance_keys| dirty_instance_keys.contains(&instance_key))
    })
}

/// Guard that records the current component node id for the calling thread.
/// Nested components push their id and pop on drop, forming a stack.
pub struct NodeContextGuard {
    popped: bool,
    instance_logic_id_popped: bool,
    #[cfg(feature = "profiling")]
    profiling_guard: Option<crate::profiler::ScopeGuard>,
}

pub struct CurrentComponentInstanceGuard {
    popped: bool,
}

/// Execution phase for `remember` usage checks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuntimePhase {
    /// Component render/build phase (allowed for `remember`).
    Build,
    /// Measurement phase (disallowed for `remember`).
    Measure,
    /// Input handling phase (disallowed for `remember`).
    Input,
}

/// Guard for execution phase stack.
pub struct PhaseGuard {
    popped: bool,
}

impl PhaseGuard {
    /// Pop the current phase immediately.
    pub fn pop(mut self) {
        if !self.popped {
            pop_phase();
            self.popped = true;
        }
    }
}

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_phase();
            self.popped = true;
        }
    }
}

impl NodeContextGuard {
    /// Pop the current node id immediately. Usually you rely on `Drop` instead.
    pub fn pop(mut self) {
        if !self.popped {
            pop_current_node();
            self.popped = true;
        }
        if !self.instance_logic_id_popped {
            pop_instance_logic_id();
            self.instance_logic_id_popped = true;
        }
    }
}

impl Drop for NodeContextGuard {
    fn drop(&mut self) {
        #[cfg(feature = "profiling")]
        {
            let _ = self.profiling_guard.take();
        }
        if !self.popped {
            pop_current_node();
            self.popped = true;
        }
        if !self.instance_logic_id_popped {
            pop_instance_logic_id();
            self.instance_logic_id_popped = true;
        }
    }
}

impl CurrentComponentInstanceGuard {
    /// Pop the current component instance key immediately. Usually you rely on
    /// `Drop` instead.
    pub fn pop(mut self) {
        if !self.popped {
            pop_current_component_instance_key();
            self.popped = true;
        }
    }
}

impl Drop for CurrentComponentInstanceGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_current_component_instance_key();
            self.popped = true;
        }
    }
}

/// Push the given node id as the current executing component for this thread.
pub fn push_current_node(
    node_id: NodeId,
    component_type_id: u64,
    fn_name: &str,
) -> NodeContextGuard {
    #[cfg(not(feature = "profiling"))]
    let _ = fn_name;
    #[allow(unused_variables)]
    let parent_node_id = with_execution_context_mut(|context| {
        let parent = context.node_context_stack.last().copied();
        context.node_context_stack.push(node_id);
        parent
    });

    let parent_call_index = next_child_instance_call_index();
    let parent_instance_logic_id =
        with_execution_context(|context| context.instance_logic_id_stack.last().copied())
            .unwrap_or(0);

    let group_path_hash = current_group_path_hash();
    let has_group_path = with_execution_context(|context| !context.group_path_stack.is_empty());

    let instance_salt = if let Some(key_hash) = current_instance_key_override() {
        hash_components(&[&key_hash, &group_path_hash, &parent_call_index])
    } else if has_group_path {
        hash_components(&[&group_path_hash, &parent_call_index])
    } else {
        parent_call_index
    };

    let instance_logic_id =
        if let Some(instance_logic_id_override) = take_next_node_instance_logic_id_override() {
            instance_logic_id_override
        } else if parent_call_index == 0
            && parent_instance_logic_id == 0
            && current_instance_key_override().is_none()
            && !has_group_path
        {
            component_type_id
        } else {
            hash_components(&[
                &component_type_id,
                &parent_instance_logic_id,
                &instance_salt,
            ])
        };

    with_execution_context_mut(|context| {
        context.instance_logic_id_stack.push(instance_logic_id);
    });

    push_order_frame();

    #[cfg(feature = "profiling")]
    let profiling_guard = match current_phase() {
        Some(RuntimePhase::Build) => {
            crate::profiler::make_build_scope_guard(node_id, parent_node_id, fn_name)
        }
        _ => None,
    };

    NodeContextGuard {
        popped: false,
        instance_logic_id_popped: false,
        #[cfg(feature = "profiling")]
        profiling_guard,
    }
}

/// Push the given node id with an already resolved instance logic id.
///
/// This is used outside build (for example input/measure), where component
/// identity must be restored from the recorded tree node rather than derived
/// from call order.
pub fn push_current_node_with_instance_logic_id(
    node_id: NodeId,
    instance_logic_id: u64,
    fn_name: &str,
) -> NodeContextGuard {
    #[cfg(not(feature = "profiling"))]
    let _ = fn_name;
    #[allow(unused_variables)]
    let parent_node_id = with_execution_context_mut(|context| {
        let parent = context.node_context_stack.last().copied();
        context.node_context_stack.push(node_id);
        parent
    });

    let _ = next_child_instance_call_index();

    with_execution_context_mut(|context| {
        context.instance_logic_id_stack.push(instance_logic_id);
    });
    push_order_frame();

    #[cfg(feature = "profiling")]
    let profiling_guard = match current_phase() {
        Some(RuntimePhase::Build) => {
            crate::profiler::make_build_scope_guard(node_id, parent_node_id, fn_name)
        }
        _ => None,
    };

    NodeContextGuard {
        popped: false,
        instance_logic_id_popped: false,
        #[cfg(feature = "profiling")]
        profiling_guard,
    }
}

/// Push the current component instance key for the active execution scope.
pub fn push_current_component_instance_key(instance_key: u64) -> CurrentComponentInstanceGuard {
    with_execution_context_mut(|context| {
        context.current_component_instance_stack.push(instance_key);
    });
    CurrentComponentInstanceGuard { popped: false }
}

fn pop_current_component_instance_key() {
    with_execution_context_mut(|context| {
        let popped = context.current_component_instance_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current component instance key from an empty stack"
        );
    });
}

fn pop_current_node() {
    with_execution_context_mut(|context| {
        let popped = context.node_context_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current node from an empty stack"
        );
    });
    pop_order_frame("ORDER_FRAME_STACK underflow: attempted to pop from empty stack");
}

/// Get the node id at the top of the thread-local component stack.
pub fn current_node_id() -> Option<NodeId> {
    with_execution_context(|context| context.node_context_stack.last().copied())
}

fn current_instance_logic_id_opt() -> Option<u64> {
    with_execution_context(|context| context.instance_logic_id_stack.last().copied())
}

/// Returns the current component instance logic id.
#[doc(hidden)]
pub fn current_instance_logic_id() -> u64 {
    current_instance_logic_id_opt()
        .expect("current_instance_logic_id must be called inside a component")
}

/// Returns the instance key for the current component call site.
#[doc(hidden)]
pub fn current_instance_key() -> u64 {
    let instance_logic_id = current_instance_logic_id_opt()
        .expect("current_instance_key must be called inside a component");
    let group_path_hash = current_group_path_hash();
    hash_components(&[&instance_logic_id, &group_path_hash])
}

fn pop_instance_logic_id() {
    with_execution_context_mut(|context| {
        let _ = context.instance_logic_id_stack.pop();
    });
}

/// Push an execution phase for the current thread.
pub fn push_phase(phase: RuntimePhase) -> PhaseGuard {
    with_execution_context_mut(|context| {
        context.phase_stack.push(phase);
    });
    PhaseGuard { popped: false }
}

fn pop_phase() {
    with_execution_context_mut(|context| {
        let popped = context.phase_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop execution phase from an empty stack"
        );
    });
}

pub(crate) fn current_phase() -> Option<RuntimePhase> {
    with_execution_context(|context| context.phase_stack.last().copied())
}

pub(crate) fn push_group_id(group_id: u64) {
    with_execution_context_mut(|context| {
        context.group_path_stack.push(group_id);
    });
}

pub(crate) fn pop_group_id(expected_group_id: u64) {
    with_execution_context_mut(|context| {
        if let Some(popped) = context.group_path_stack.pop() {
            debug_assert_eq!(
                popped, expected_group_id,
                "Unbalanced GroupGuard stack: expected {}, got {}",
                expected_group_id, popped
            );
        } else {
            debug_assert!(false, "Attempted to pop GroupGuard from an empty stack");
        }
    });
}

pub(crate) fn current_group_path() -> Vec<u64> {
    with_execution_context(|context| context.group_path_stack.clone())
}

pub(crate) fn current_group_path_hash() -> u64 {
    with_execution_context(|context| hash_components(&[&context.group_path_stack[..]]))
}

pub(crate) fn current_instance_key_override() -> Option<u64> {
    with_execution_context(|context| context.instance_key_stack.last().copied())
}

/// RAII guard that tracks control-flow grouping for the current component node.
///
/// A guard pushes the provided group id when constructed and pops it when
/// dropped, ensuring grouping stays balanced even with early returns or panics.
pub struct GroupGuard {
    group_id: u64,
}

impl GroupGuard {
    /// Push a group id onto the current component's group stack.
    pub fn new(group_id: u64) -> Self {
        push_group_id(group_id);
        push_order_frame();
        Self { group_id }
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        pop_order_frame("ORDER_FRAME_STACK underflow: attempted to pop GroupGuard frame");
        pop_group_id(self.group_id);
    }
}

/// RAII guard for path-only control-flow groups, primarily used for loop
/// bodies.
///
/// Unlike [`GroupGuard`], this guard does not create a new local order frame.
/// Repeated iterations therefore continue consuming sibling call indices from
/// the surrounding component scope instead of restarting from zero each time.
pub struct PathGroupGuard {
    group_id: u64,
}

impl PathGroupGuard {
    /// Push a group id onto the current component's group stack without
    /// resetting local call-order counters.
    pub fn new(group_id: u64) -> Self {
        push_group_id(group_id);
        Self { group_id }
    }
}

impl Drop for PathGroupGuard {
    fn drop(&mut self) {
        pop_group_id(self.group_id);
    }
}

/// RAII guard that sets a stable instance key for the duration of a block.
pub struct InstanceKeyGuard {
    key_hash: u64,
}

impl InstanceKeyGuard {
    /// Push a key hash for instance identity.
    pub fn new(key_hash: u64) -> Self {
        with_execution_context_mut(|context| {
            context.instance_key_stack.push(key_hash);
        });
        Self { key_hash }
    }
}

impl Drop for InstanceKeyGuard {
    fn drop(&mut self) {
        with_execution_context_mut(|context| {
            let popped = context.instance_key_stack.pop();
            debug_assert_eq!(
                popped,
                Some(self.key_hash),
                "Unbalanced InstanceKeyGuard stack"
            );
        });
    }
}

pub(crate) fn hash_components<H: Hash + ?Sized>(parts: &[&H]) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}

pub(crate) fn compute_context_slot_key() -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();

    let call_counter = next_order_counter(
        OrderCounterKind::Context,
        "ORDER_FRAME_STACK is empty; provide_context must be called inside a component",
    );

    let slot_hash = hash_components(&[&group_path_hash, &call_counter]);
    (instance_logic_id, slot_hash)
}

pub(crate) fn compute_slot_key<K: Hash>(key: &K) -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();
    let key_hash = hash_components(&[key]);

    let call_counter = next_order_counter(
        OrderCounterKind::Remember,
        "ORDER_FRAME_STACK is empty; remember must be called inside a component",
    );

    let slot_hash = hash_components(&[&group_path_hash, &key_hash, &call_counter]);
    (instance_logic_id, slot_hash)
}

pub(crate) fn compute_functor_slot_key<K: Hash>(key: &K) -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();
    let key_hash = hash_components(&[key]);

    let call_counter = next_order_counter(
        OrderCounterKind::Functor,
        "ORDER_FRAME_STACK is empty; callback constructors must be called inside a component",
    );

    let slot_hash = hash_components(&[&group_path_hash, &key_hash, &call_counter]);
    (instance_logic_id, slot_hash)
}

pub(crate) fn ensure_build_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) => {}
        Some(RuntimePhase::Measure) => {
            panic!("remember must not be called inside measure; move state to component render")
        }
        Some(RuntimePhase::Input) => {
            panic!(
                "remember must not be called inside typed input handlers; move state to component render"
            )
        }
        None => panic!(
            "remember must be called inside a tessera component. Ensure you're calling this from within a function annotated with #[tessera]."
        ),
    }
}

/// Groups the execution of a block of code with a stable key.
///
/// This is useful for maintaining state identity in dynamic lists or loops
/// where the order of items might change.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Prop, key, remember, tessera};
///
/// #[derive(Clone, Prop)]
/// struct MyListArgs {
///     items: Vec<String>,
/// }
///
/// #[tessera]
/// fn my_list(args: &MyListArgs) {
///     for item in args.items.iter() {
///         key(item.clone(), || {
///             let state = remember(|| 0);
///         });
///     }
/// }
/// ```
pub fn key<K, F, R>(key: K, block: F) -> R
where
    K: Hash,
    F: FnOnce() -> R,
{
    let key_hash = hash_components(&[&key]);
    let _group_guard = GroupGuard::new(key_hash);
    let _instance_guard = InstanceKeyGuard::new(key_hash);
    block()
}
