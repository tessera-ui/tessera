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
    sync::Arc,
};

use rustc_hash::FxHasher;
use slotmap::{SlotMap, new_key_type};

use crate::{
    NodeId,
    context::{ContextMap, with_context_snapshot},
    execution_context::{OrderFrame, with_execution_context, with_execution_context_mut},
    prop::{ComponentReplayData, ErasedComponentRunner, ErasedProp, Prop},
    runtime::{
        GroupPath, TesseraRuntime, ensure_build_phase, reserve_component_identity,
        with_replay_scope,
    },
    session::with_runtime_globals,
};

type PropUpdate = Box<dyn FnOnce(&mut dyn Any) + Send + Sync>;

const MAX_RECYCLED_QUEUE_CAPACITY: usize = 256;

struct PendingCall {
    name: &'static str,
    parent: Option<NodeId>,
    execution_parent: Option<NodeId>,
    identity: u64,
    prop_owner: u64,
    creator: u64,
    group_path: GroupPath,
    key_override: Option<u64>,
    context: ContextMap,
    props: Arc<dyn ErasedProp>,
    runner: Arc<dyn ErasedComponentRunner>,
    updates: Vec<(&'static str, usize, PropUpdate)>,
}

fn pending_props_mut(props: &mut Arc<dyn ErasedProp>) -> &mut dyn ErasedProp {
    Arc::get_mut(props).expect("pending props have a single owner")
}

new_key_type! {
    struct PendingKey;
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct PendingToken {
    key: PendingKey,
    serial: u64,
}

struct PendingEntry {
    serial: u64,
    call: Option<PendingCall>,
}

#[derive(Default)]
struct PendingCalls {
    calls: SlotMap<PendingKey, PendingEntry>,
    scopes: Vec<VecDeque<PendingToken>>,
    recycled_queues: Vec<VecDeque<PendingToken>>,
    next_serial: u64,
}

impl PendingCalls {
    fn entry_mut(&mut self, token: PendingToken) -> Option<&mut PendingEntry> {
        self.calls
            .get_mut(token.key)
            .filter(|entry| entry.serial == token.serial)
    }

    fn remove(&mut self, token: PendingToken) -> Option<PendingCall> {
        self.entry_mut(token)?;
        self.calls.remove(token.key)?.call
    }
}

#[derive(Default)]
pub(crate) struct ComponentCalls {
    pending: RefCell<PendingCalls>,
    active: RefCell<Option<ComponentReplayData>>,
}

fn with_pending_calls<R>(access: impl FnOnce(&RefCell<PendingCalls>) -> R) -> R {
    with_runtime_globals(|globals| access(&globals.component_calls.pending))
}

fn replace_active_component(component: Option<ComponentReplayData>) -> Option<ComponentReplayData> {
    with_runtime_globals(|globals| globals.component_calls.active.replace(component))
}

pub(crate) fn active_component_replay<P: Prop>(props: &P) -> Option<ComponentReplayData> {
    with_runtime_globals(|globals| {
        let active = globals.component_calls.active.borrow();
        let active = active.as_ref()?;
        let active_props = active.props.as_any().downcast_ref::<P>()?;
        std::ptr::eq(active_props, props).then(|| active.clone())
    })
}

struct ActiveComponentGuard(Option<ComponentReplayData>);

impl Drop for ActiveComponentGuard {
    fn drop(&mut self) {
        let removed = replace_active_component(self.0.take());
        drop(removed);
    }
}

pub(crate) fn run_component(component: &ComponentReplayData) {
    let _guard = ActiveComponentGuard(replace_active_component(Some(component.clone())));
    component.runner.run(component.props.as_ref());
}

/// A configuration handle to one registered component use.
pub struct ComponentCall<P: Prop> {
    session: u128,
    id: PendingToken,
    marker: PhantomData<fn(P)>,
}

impl<P: Prop> Copy for ComponentCall<P> {}

impl<P: Prop> Clone for ComponentCall<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: Prop> ComponentCall<P> {
    fn validate_session(&self) {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.session),
            "component handle belongs to a different runtime session"
        );
    }
    /// Set a property while this call is pending.
    #[track_caller]
    pub fn configure<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P),
    {
        self.validate_session();
        let mut guard = begin_configuration(self.id);
        let call = guard.call.as_mut().expect("active configuration");
        clear_property_updates(call, property);
        update(
            pending_props_mut(&mut call.props)
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
        self.validate_session();
        self.store(property, false, update);
    }

    /// Append one configuration operation to a property collection.
    #[track_caller]
    pub fn append<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P) + Send + Sync + 'static,
    {
        self.validate_session();
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
        self.validate_session();
        loop {
            let (id, call) =
                take_next_call().expect("render-slot call must belong to the active fragment");
            if let Some(call) = call {
                execute_call(call);
            }
            if id == self.id {
                break;
            }
        }
    }
}

#[track_caller]
fn store_update(id: PendingToken, property: &'static str, append: bool, update: PropUpdate) {
    let previous = with_pending_calls(|calls| {
        let mut calls = calls.borrow_mut();
        let call = calls
            .entry_mut(id)
            .and_then(|entry| entry.call.as_mut())
            .unwrap_or_else(|| panic!("component call {} is no longer Pending", id.serial));
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
    id: PendingToken,
    call: Option<PendingCall>,
}

#[track_caller]
fn begin_configuration(id: PendingToken) -> ConfigurationGuard {
    let call = with_pending_calls(|calls| {
        calls
            .borrow_mut()
            .entry_mut(id)
            .and_then(|entry| entry.call.take())
            .unwrap_or_else(|| panic!("component call {} is no longer Pending", id.serial))
    });
    ConfigurationGuard {
        id,
        call: Some(call),
    }
}

fn clear_property_updates(call: &mut PendingCall, property: &'static str) {
    call.updates.retain(|entry| entry.0 != property);
}

impl Drop for ConfigurationGuard {
    fn drop(&mut self) {
        with_pending_calls(|calls| {
            if let Some(entry) = calls.borrow_mut().entry_mut(self.id) {
                entry.call = self.call.take();
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
    let id = register_erased_call(name, component_type_id, Arc::new(props), runner);
    ComponentCall {
        session: crate::session::current_session_identity().expect("active component session"),
        id,
        marker: PhantomData,
    }
}

#[track_caller]
fn register_erased_call(
    name: &'static str,
    component_type_id: u64,
    props: Arc<dyn ErasedProp>,
    runner: Arc<dyn ErasedComponentRunner>,
) -> PendingToken {
    ensure_build_phase();
    with_pending_calls(|pending| {
        assert!(
            !pending.borrow().scopes.is_empty(),
            "component declarations require an active build fragment"
        );
        let identity = reserve_component_identity(component_type_id);
        let parent = TesseraRuntime::with(|runtime| runtime.component_tree.current_build_node_id());
        let call = with_execution_context(|context| PendingCall {
            name,
            parent,
            execution_parent: context.node_context_stack.last().copied(),
            identity,
            prop_owner: context.instance_logic_id_stack.last().copied().unwrap_or(0),
            creator: context
                .current_component_instance_stack
                .last()
                .copied()
                .unwrap_or(0),
            group_path: context.group_path,
            key_override: context.instance_key_override,
            context: context.context_map.clone(),
            props,
            runner,
            updates: Vec::new(),
        });
        let mut pending = pending.borrow_mut();
        let serial = pending
            .next_serial
            .checked_add(1)
            .expect("component call token space exhausted");
        pending.next_serial = serial;
        let key = pending.calls.insert(PendingEntry {
            serial,
            call: Some(call),
        });
        let token = PendingToken { key, serial };
        pending
            .scopes
            .last_mut()
            .expect("active build fragment")
            .push_back(token);
        token
    })
}

struct ScopeGuard {
    root_order_frame: bool,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let abandoned = with_pending_calls(|calls| {
            let mut calls = calls.borrow_mut();
            let mut pending = calls.scopes.pop().expect("balanced build fragments");
            let abandoned = pending
                .drain(..)
                .filter_map(|id| calls.remove(id))
                .collect::<Vec<_>>();
            if (1..=MAX_RECYCLED_QUEUE_CAPACITY).contains(&pending.capacity()) {
                calls.recycled_queues.push(pending);
            }
            abandoned
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
    with_pending_calls(|calls| {
        let mut calls = calls.borrow_mut();
        let queue = calls.recycled_queues.pop().unwrap_or_default();
        calls.scopes.push(queue);
    });
    ScopeGuard { root_order_frame }
}

fn drain_pending_scope() {
    while let Some((_, call)) = take_next_call() {
        if let Some(call) = call {
            execute_call(call);
        }
    }
}

fn take_next_call() -> Option<(PendingToken, Option<PendingCall>)> {
    with_pending_calls(|calls| {
        let mut calls = calls.borrow_mut();
        let id = calls
            .scopes
            .last_mut()
            .expect("active build fragment")
            .pop_front()?;
        Some((id, calls.remove(id)))
    })
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
    group_path: GroupPath,
    key_override: Option<u64>,
}

impl Drop for PropScopeGuard {
    fn drop(&mut self) {
        with_execution_context_mut(|context| {
            context.group_path = self.group_path;
            context.instance_key_override = self.key_override;
            context.instance_logic_id_stack.pop();
            context.current_component_instance_stack.pop();
            context.order_frame_stack.pop();
        });
    }
}

fn execute_call(call: PendingCall) {
    let PendingCall {
        name,
        parent,
        execution_parent,
        identity,
        prop_owner,
        creator,
        group_path,
        key_override,
        context,
        mut props,
        runner,
        updates,
    } = call;
    let _parent_guard = ParentGuard {
        queue: TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.replace_build_parent(parent)
        }),
        node_stack: with_execution_context_mut(|context| {
            if context.node_context_stack.last().copied() == execution_parent {
                None
            } else {
                Some(std::mem::replace(
                    &mut context.node_context_stack,
                    execution_parent.into_iter().collect(),
                ))
            }
        }),
    };
    with_context_snapshot(&context, || {
        for (property, occurrence, update) in updates {
            let mut hasher = FxHasher::default();
            (identity, name, property, occurrence).hash(&mut hasher);
            let (group_path, key_override) = with_execution_context_mut(|context| {
                context.instance_logic_id_stack.push(prop_owner);
                context.current_component_instance_stack.push(creator);
                context.order_frame_stack.push(OrderFrame::default());
                (
                    std::mem::replace(
                        &mut context.group_path,
                        GroupPath::default().extend(hasher.finish()),
                    ),
                    std::mem::replace(&mut context.instance_key_override, key_override),
                )
            });
            let _guard = PropScopeGuard {
                group_path,
                key_override,
            };
            update(pending_props_mut(&mut props).as_any_mut());
        }
        with_replay_scope(identity, group_path, key_override, || {
            run_component(&ComponentReplayData { runner, props });
        });
    });
}

pub(crate) fn reset_pending_calls() {
    let removed = with_pending_calls(|calls| {
        let mut calls = calls.borrow_mut();
        for scope in &mut calls.scopes {
            scope.clear();
        }
        calls.recycled_queues.clear();
        std::mem::take(&mut calls.calls)
    });
    drop(removed);
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::{
        RenderSlot,
        execution_context::reset_execution_context,
        prop::{ErasedComponentRunner, ErasedProp, Prop},
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

    #[derive(Clone, Default)]
    struct Recorder {
        observed: crate::testing::TestProbe<Vec<Props>>,
    }

    impl ErasedComponentRunner for Recorder {
        fn run(&self, props: &dyn ErasedProp) {
            let props = props.as_any().downcast_ref::<Props>().unwrap();
            self.observed.with_mut(|values| values.push(props.clone()));
        }
    }

    impl Recorder {
        fn declare(&self, value: u32) -> ComponentCall<Props> {
            register_component_call(
                "probe",
                123,
                Props { value, slot: None },
                std::sync::Arc::new(self.clone()),
            )
        }

        fn build<R>(&self, body: impl FnOnce() -> R) -> R {
            reset_execution_context();
            self.observed.with_mut(Vec::clear);
            let _phase = push_phase(RuntimePhase::Build);
            with_pending_scope(body)
        }

        fn values(&self) -> Vec<u32> {
            self.observed
                .with(|values| values.iter().map(|props| props.value).collect())
        }
    }

    #[test]
    fn registered_uses_survive_handle_disposal_and_execute_in_declaration_order() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        struct OwnedHandle(ComponentCall<Props>);
        recorder.build(|| {
            let first = OwnedHandle(recorder.declare(1));
            let second = recorder.declare(2);
            {
                let _group = GroupGuard::new(17);
                second.configure("value", |props| props.value = 20);
                first.0.configure("value", |props| props.value = 10);
            }
            drop(first);
            std::mem::forget(OwnedHandle(recorder.declare(3)));
            assert!(recorder.values().is_empty());
        });
        assert_eq!(recorder.values(), [10, 20, 3]);
    }

    #[test]
    fn only_the_final_property_configuration_is_materialized() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        recorder.build(|| {
            let call = recorder.declare(1);
            call.configure_deferred("value", |_| panic!("superseded configuration executed"));
            call.configure_deferred("value", |props| props.value = 9);
        });
        assert_eq!(recorder.values(), [9]);
    }

    #[test]
    fn stale_handles_cannot_configure_a_new_registered_use() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        let stale = recorder.build(|| recorder.declare(1));
        recorder.build(|| {
            recorder.declare(2);
            let error = catch_unwind(AssertUnwindSafe(|| {
                stale.configure("value", |props| props.value = 99);
            }))
            .expect_err("a committed handle must fail");
            let message = error
                .downcast_ref::<String>()
                .expect("call status diagnostic");
            assert!(message.contains("is no longer Pending"));
        });
        assert_eq!(recorder.values(), [2]);
    }

    #[test]
    fn property_slot_identity_is_independent_of_configuration_groups_and_order() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        reset_slots();
        let configure = |call: ComponentCall<Props>| {
            call.configure_deferred("slot", |props| props.slot = Some(RenderSlot::new(|| {})));
        };
        recorder.build(|| {
            let first = recorder.declare(1);
            let second = recorder.declare(2);
            configure(first);
            configure(second);
        });
        let first = recorder.observed.get();
        recorder.build(|| {
            let a = recorder.declare(1);
            let b = recorder.declare(2);
            let _group = GroupGuard::new(81);
            configure(b);
            configure(a);
        });
        recorder.observed.with(|observed| {
            assert!(first[0].slot == observed[0].slot);
            assert!(first[1].slot == observed[1].slot);
            assert!(observed[0].slot != observed[1].slot);
        });
    }

    #[test]
    fn aborted_fragments_discard_their_pending_uses() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        recorder.build(|| {
            let mut abandoned = None;
            let error = catch_unwind(AssertUnwindSafe(|| {
                with_pending_scope(|| {
                    abandoned = Some(recorder.declare(99));
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
            recorder.declare(7);
        });
        assert_eq!(recorder.values(), [7]);
    }

    #[test]
    fn runtime_reset_invalidates_pending_handles_without_reusing_their_tokens() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        recorder.build(|| {
            let stale = recorder.declare(1);
            reset_slots();
            recorder.declare(2);
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    stale.configure("value", |props| props.value = 99);
                }))
                .is_err()
            );
        });
        assert_eq!(recorder.values(), [2]);
    }

    #[test]
    fn configuration_can_register_more_calls_without_borrowing_the_pending_registry() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        recorder.build(|| {
            let first = recorder.declare(1);
            first.configure("value", |props| {
                props.value = 10;
                recorder.declare(2);
            });
        });
        assert_eq!(recorder.values(), [10, 2]);
    }

    #[test]
    fn superseded_configuration_captures_drop_outside_the_pending_registry_borrow() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        struct RegisterOnDrop(Recorder);

        impl Drop for RegisterOnDrop {
            fn drop(&mut self) {
                self.0.declare(2);
            }
        }

        recorder.build(|| {
            let first = recorder.declare(1);
            let capture = RegisterOnDrop(recorder.clone());
            first.configure_deferred("value", move |_| drop(capture));
            first.configure("value", |props| props.value = 10);
        });
        assert_eq!(recorder.values(), [10, 2]);
    }
}
