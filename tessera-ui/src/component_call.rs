//! Registered component calls and build-scoped configuration.
//!
//! ## Usage
//!
//! Configure declared component uses before the runtime expands their content.

use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use rustc_hash::{FxHashMap as HashMap, FxHasher};

use crate::{
    NodeId,
    context::{ContextMap, current_context_map, with_context_snapshot},
    execution_context::{OrderFrame, with_execution_context, with_execution_context_mut},
    prop::{ErasedComponentRunner, ErasedProp, Prop},
    runtime::{
        TesseraRuntime, current_group_path, current_instance_key_override,
        current_replay_boundary_instance_key_from_scope, ensure_build_phase,
        reserve_component_identity, with_replay_scope,
    },
};

type PropUpdate = Box<dyn FnOnce(&mut dyn Any) + Send + Sync>;

struct PendingCall {
    name: &'static str,
    parent: Option<NodeId>,
    execution_parent: Option<NodeId>,
    identity: u64,
    prop_owner: u64,
    creator: u64,
    group_path: Vec<u64>,
    key_override: Option<u64>,
    context: ContextMap,
    props: Box<dyn ErasedProp>,
    runner: Arc<dyn ErasedComponentRunner>,
    updates: Vec<(&'static str, usize, PropUpdate)>,
}

#[derive(Default)]
struct PendingCalls {
    calls: HashMap<u64, PendingCall>,
    scopes: Vec<VecDeque<u64>>,
    reset_generation: u64,
}

thread_local! {
    static PENDING_CALLS: RefCell<PendingCalls> = RefCell::new(PendingCalls::default());
    static ACTIVE_RUNNER: RefCell<Option<Arc<dyn ErasedComponentRunner>>> = RefCell::new(None);
}

/// Return the runner executing the current registered use.
pub fn current_component_runner() -> Option<Arc<dyn ErasedComponentRunner>> {
    ACTIVE_RUNNER.with(|runner| runner.borrow().clone())
}

struct RunnerGuard(Option<Arc<dyn ErasedComponentRunner>>);

impl Drop for RunnerGuard {
    fn drop(&mut self) {
        let removed = ACTIVE_RUNNER.with(|runner| runner.replace(self.0.take()));
        drop(removed);
    }
}

pub(crate) fn with_component_runner<R>(
    runner: &Arc<dyn ErasedComponentRunner>,
    run: impl FnOnce() -> R,
) -> R {
    let _guard = RunnerGuard(ACTIVE_RUNNER.with(|active| active.replace(Some(Arc::clone(runner)))));
    run()
}

/// A configuration handle to one registered component use.
pub struct ComponentCall<P: Prop> {
    id: u64,
    marker: PhantomData<fn(P)>,
}

impl<P: Prop> Copy for ComponentCall<P> {}

impl<P: Prop> Clone for ComponentCall<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: Prop> ComponentCall<P> {
    /// Set a property while this call is pending.
    #[track_caller]
    pub fn configure<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P),
    {
        let mut guard = begin_configuration(self.id);
        let call = guard.call.as_mut().expect("active configuration");
        clear_property_updates(call, property);
        update(
            call.props
                .as_any_mut()
                .downcast_mut::<P>()
                .expect("registered component props type"),
        );
    }

    /// Materialize a property in this call's declaration scope before execution.
    #[track_caller]
    pub fn configure_deferred<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P) + Send + Sync + 'static,
    {
        self.store(property, false, update);
    }

    /// Append one configuration operation to a property collection.
    #[track_caller]
    pub fn append<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P) + Send + Sync + 'static,
    {
        self.store(property, true, update);
    }

    #[track_caller]
    fn store<F>(&self, property: &'static str, append: bool, update: F)
    where
        F: FnOnce(&mut P) + Send + Sync + 'static,
    {
        let update: PropUpdate = Box::new(move |props| {
            update(
                props
                    .downcast_mut::<P>()
                    .expect("registered component props type"),
            );
        });
        store_update(self.id, property, append, update);
    }

    /// Expand this call at a controlled runtime boundary.
    pub(crate) fn execute_now(self) {
        loop {
            let next = PENDING_CALLS.with(|calls| {
                calls
                    .borrow_mut()
                    .scopes
                    .last_mut()
                    .expect("active render-slot fragment")
                    .pop_front()
            });
            let id = next.expect("render-slot call must belong to the active fragment");
            execute_call(id);
            if id == self.id {
                break;
            }
        }
    }
}

#[track_caller]
fn store_update(id: u64, property: &'static str, append: bool, update: PropUpdate) {
    let previous = PENDING_CALLS.with(|calls| {
        let mut calls = calls.borrow_mut();
        let call = calls
            .calls
            .get_mut(&id)
            .unwrap_or_else(|| panic!("component call {} is no longer Pending", id));
        if append {
            let occurrence = call
                .updates
                .iter()
                .filter(|entry| entry.0 == property)
                .count();
            call.updates.push((property, occurrence, update));
            Vec::new()
        } else {
            let mut previous = Vec::new();
            let mut index = 0;
            while index < call.updates.len() {
                if call.updates[index].0 == property {
                    previous.push(call.updates.remove(index));
                } else {
                    index += 1;
                }
            }
            call.updates.push((property, 0, update));
            previous
        }
    });
    drop(previous);
}

struct ConfigurationGuard {
    id: u64,
    generation: u64,
    call: Option<PendingCall>,
}

#[track_caller]
fn begin_configuration(id: u64) -> ConfigurationGuard {
    let (call, generation) = PENDING_CALLS.with(|calls| {
        let mut calls = calls.borrow_mut();
        let call = calls
            .calls
            .remove(&id)
            .unwrap_or_else(|| panic!("component call {id} is no longer Pending"));
        (call, calls.reset_generation)
    });
    ConfigurationGuard {
        id,
        generation,
        call: Some(call),
    }
}

fn clear_property_updates(call: &mut PendingCall, property: &'static str) {
    call.updates.retain(|entry| entry.0 != property);
}

impl Drop for ConfigurationGuard {
    fn drop(&mut self) {
        PENDING_CALLS.with(|calls| {
            let mut calls = calls.borrow_mut();
            if calls.reset_generation == self.generation {
                calls
                    .calls
                    .insert(self.id, self.call.take().expect("active configuration"));
            }
        });
    }
}

/// Register a component use in the active build fragment.
#[track_caller]
pub fn register_component_call<P: Prop>(
    name: &'static str,
    component_type_id: u64,
    props: P,
    runner: Arc<dyn ErasedComponentRunner>,
) -> ComponentCall<P> {
    let id = register_erased_call(name, component_type_id, Box::new(props), runner);
    ComponentCall {
        id,
        marker: PhantomData,
    }
}

#[track_caller]
fn register_erased_call(
    name: &'static str,
    component_type_id: u64,
    props: Box<dyn ErasedProp>,
    runner: Arc<dyn ErasedComponentRunner>,
) -> u64 {
    ensure_build_phase();
    PENDING_CALLS.with(|calls| {
        assert!(
            !calls.borrow().scopes.is_empty(),
            "component declarations require an active build fragment"
        );
    });
    static NEXT_CALL_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_CALL_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .expect("component call token space exhausted");
    let identity = reserve_component_identity(component_type_id);
    let call = PendingCall {
        name,
        parent: TesseraRuntime::with(|runtime| runtime.component_tree.current_build_node_id()),
        execution_parent: with_execution_context(|context| {
            context.node_context_stack.last().copied()
        }),
        identity,
        prop_owner: with_execution_context(|context| {
            context.instance_logic_id_stack.last().copied().unwrap_or(0)
        }),
        creator: current_replay_boundary_instance_key_from_scope().unwrap_or(0),
        group_path: current_group_path(),
        key_override: current_instance_key_override(),
        context: current_context_map(),
        props,
        runner,
        updates: Vec::new(),
    };
    PENDING_CALLS.with(|calls| {
        let mut calls = calls.borrow_mut();
        calls
            .scopes
            .last_mut()
            .expect("active build fragment")
            .push_back(id);
        calls.calls.insert(id, call);
    });
    id
}

struct ScopeGuard {
    root_order_frame: bool,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let abandoned = PENDING_CALLS.with(|calls| {
            let mut calls = calls.borrow_mut();
            let pending = calls.scopes.pop().expect("balanced build fragments");
            pending
                .into_iter()
                .filter_map(|id| calls.calls.remove(&id))
                .collect::<Vec<_>>()
        });
        if self.root_order_frame {
            with_execution_context_mut(|context| {
                context.order_frame_stack.pop();
            });
        }
        drop(abandoned);
    }
}

/// Build a fragment and expand all uses registered by that fragment.
pub fn with_pending_scope<R>(build: impl FnOnce() -> R) -> R {
    let _guard = begin_pending_scope();
    let result = build();
    drain_pending_scope();
    result
}

fn begin_pending_scope() -> ScopeGuard {
    let root_order_frame = with_execution_context_mut(|context| {
        if context.order_frame_stack.is_empty() {
            context.order_frame_stack.push(OrderFrame::default());
            true
        } else {
            false
        }
    });
    PENDING_CALLS.with(|calls| calls.borrow_mut().scopes.push(VecDeque::new()));
    ScopeGuard { root_order_frame }
}

fn drain_pending_scope() {
    loop {
        let next = PENDING_CALLS.with(|calls| {
            calls
                .borrow_mut()
                .scopes
                .last_mut()
                .expect("active build fragment")
                .pop_front()
        });
        let Some(id) = next else { break };
        execute_call(id);
    }
}

struct ParentGuard {
    queue: Option<Vec<NodeId>>,
    node_stack: Option<Vec<NodeId>>,
}

impl Drop for ParentGuard {
    fn drop(&mut self) {
        if let Some(queue) = self.queue.take() {
            TesseraRuntime::with_mut(|runtime| runtime.component_tree.restore_build_queue(queue));
        }
        if let Some(node_stack) = self.node_stack.take() {
            with_execution_context_mut(|context| context.node_context_stack = node_stack);
        }
    }
}

struct PropScopeGuard {
    group_path: Vec<u64>,
    key_stack: Vec<u64>,
}

impl Drop for PropScopeGuard {
    fn drop(&mut self) {
        with_execution_context_mut(|context| {
            context.group_path_stack = std::mem::take(&mut self.group_path);
            context.instance_key_stack = std::mem::take(&mut self.key_stack);
            context.instance_logic_id_stack.pop();
            context.current_component_instance_stack.pop();
            context.order_frame_stack.pop();
        });
    }
}

fn execute_call(id: u64) {
    let call = PENDING_CALLS.with(|calls| calls.borrow_mut().calls.remove(&id));
    let Some(mut call) = call else { return };
    let _parent_guard = ParentGuard {
        queue: TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.replace_build_parent(call.parent)
        }),
        node_stack: with_execution_context_mut(|context| {
            if context.node_context_stack.last().copied() == call.execution_parent {
                None
            } else {
                Some(std::mem::replace(
                    &mut context.node_context_stack,
                    call.execution_parent.into_iter().collect(),
                ))
            }
        }),
    };
    with_context_snapshot(&call.context, || {
        for (property, occurrence, update) in call.updates.drain(..) {
            let mut hasher = FxHasher::default();
            (call.identity, call.name, property, occurrence).hash(&mut hasher);
            let (group_path, key_stack) = with_execution_context_mut(|context| {
                context.instance_logic_id_stack.push(call.prop_owner);
                context.current_component_instance_stack.push(call.creator);
                context.order_frame_stack.push(OrderFrame::default());
                (
                    std::mem::replace(&mut context.group_path_stack, vec![hasher.finish()]),
                    std::mem::replace(
                        &mut context.instance_key_stack,
                        call.key_override.into_iter().collect(),
                    ),
                )
            });
            let _guard = PropScopeGuard {
                group_path,
                key_stack,
            };
            update(call.props.as_any_mut());
        }
        with_replay_scope(call.identity, &call.group_path, call.key_override, || {
            with_component_runner(&call.runner, || call.runner.run(call.props.as_ref()));
        });
    });
}

pub(crate) fn reset_pending_calls() {
    let removed = PENDING_CALLS.with(|calls| {
        let mut calls = calls.borrow_mut();
        for scope in &mut calls.scopes {
            scope.clear();
        }
        calls.reset_generation = calls.reset_generation.wrapping_add(1);
        std::mem::take(&mut calls.calls)
    });
    drop(removed);
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    use crate::{
        RenderSlot,
        execution_context::reset_execution_context,
        prop::{Prop, make_component_runner},
        runtime::{GroupGuard, RuntimePhase, push_phase, reset_slots},
    };

    use super::{ComponentCall, register_component_call, with_pending_scope};

    #[derive(Clone, Default, PartialEq)]
    struct Props {
        value: u32,
        slot: Option<RenderSlot>,
    }

    impl Prop for Props {
        fn prop_eq(&self, other: &Self) -> bool {
            self == other
        }
    }

    thread_local! {
        static OBSERVED: RefCell<Vec<Props>> = const { RefCell::new(Vec::new()) };
    }

    fn observe(props: &Props) {
        OBSERVED.with(|observed| observed.borrow_mut().push(props.clone()));
    }

    fn declare(value: u32) -> ComponentCall<Props> {
        register_component_call(
            "probe",
            123,
            Props { value, slot: None },
            make_component_runner(observe),
        )
    }

    fn build<R>(body: impl FnOnce() -> R) -> R {
        reset_execution_context();
        OBSERVED.with(|observed| observed.borrow_mut().clear());
        let _phase = push_phase(RuntimePhase::Build);
        with_pending_scope(body)
    }

    fn values() -> Vec<u32> {
        OBSERVED.with(|observed| observed.borrow().iter().map(|props| props.value).collect())
    }

    #[test]
    fn registered_uses_survive_handle_disposal_and_execute_in_declaration_order() {
        struct OwnedHandle(ComponentCall<Props>);
        build(|| {
            let first = OwnedHandle(declare(1));
            let second = declare(2);
            {
                let _group = GroupGuard::new(17);
                second.configure("value", |props| props.value = 20);
                first.0.configure("value", |props| props.value = 10);
            }
            drop(first);
            std::mem::forget(OwnedHandle(declare(3)));
            assert!(values().is_empty());
        });
        assert_eq!(values(), [10, 20, 3]);
    }

    #[test]
    fn only_the_final_property_configuration_is_materialized() {
        build(|| {
            let call = declare(1);
            call.configure_deferred("value", |_| panic!("superseded configuration executed"));
            call.configure_deferred("value", |props| props.value = 9);
        });
        assert_eq!(values(), [9]);
    }

    #[test]
    fn stale_handles_cannot_configure_a_new_registered_use() {
        let stale = build(|| declare(1));
        build(|| {
            declare(2);
            let error = catch_unwind(AssertUnwindSafe(|| {
                stale.configure("value", |props| props.value = 99);
            }))
            .expect_err("a committed handle must fail");
            let message = error
                .downcast_ref::<String>()
                .expect("call status diagnostic");
            assert!(message.contains("is no longer Pending"));
        });
        assert_eq!(values(), [2]);
    }

    #[test]
    fn property_slot_identity_is_independent_of_configuration_groups_and_order() {
        reset_slots();
        let configure = |call: ComponentCall<Props>| {
            call.configure_deferred("slot", |props| props.slot = Some(RenderSlot::new(|| {})));
        };
        build(|| {
            let first = declare(1);
            let second = declare(2);
            configure(first);
            configure(second);
        });
        let first = OBSERVED.with(|observed| observed.borrow().clone());
        build(|| {
            let a = declare(1);
            let b = declare(2);
            let _group = GroupGuard::new(81);
            configure(b);
            configure(a);
        });
        OBSERVED.with(|observed| {
            let observed = observed.borrow();
            assert!(first[0].slot == observed[0].slot);
            assert!(first[1].slot == observed[1].slot);
            assert!(observed[0].slot != observed[1].slot);
        });
    }

    #[test]
    fn aborted_fragments_discard_their_pending_uses() {
        build(|| {
            let mut abandoned = None;
            let error = catch_unwind(AssertUnwindSafe(|| {
                with_pending_scope(|| {
                    abandoned = Some(declare(99));
                    panic!("abort fragment");
                });
            }));
            assert!(error.is_err());
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    abandoned
                        .unwrap()
                        .configure("value", |props| props.value = 99);
                }))
                .is_err()
            );
            declare(7);
        });
        assert_eq!(values(), [7]);
    }

    #[test]
    fn runtime_reset_invalidates_pending_handles_without_reusing_their_tokens() {
        build(|| {
            let stale = declare(1);
            reset_slots();
            declare(2);
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    stale.configure("value", |props| props.value = 99);
                }))
                .is_err()
            );
        });
        assert_eq!(values(), [2]);
    }

    #[test]
    fn configuration_can_register_more_calls_without_borrowing_the_pending_registry() {
        build(|| {
            let first = declare(1);
            first.configure("value", |props| {
                props.value = 10;
                declare(2);
            });
        });
        assert_eq!(values(), [10, 2]);
    }

    #[test]
    fn superseded_configuration_captures_drop_outside_the_pending_registry_borrow() {
        struct RegisterOnDrop;

        impl Drop for RegisterOnDrop {
            fn drop(&mut self) {
                declare(2);
            }
        }

        build(|| {
            let first = declare(1);
            let capture = RegisterOnDrop;
            first.configure_deferred("value", move |_| drop(capture));
            first.configure("value", |props| props.value = 10);
        });
        assert_eq!(values(), [10, 2]);
    }
}
