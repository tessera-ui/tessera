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
    rc::Rc,
};

use rustc_hash::FxHasher;
use slotmap::{SlotMap, new_key_type};

use crate::{
    NodeId, RuntimeSession, RuntimeSessionId,
    context::{ContextMap, with_context_map},
    execution_context::{OrderFrame, with_execution_context_mut},
    prop::{ErasedComponent, Prop},
    runtime::{
        GroupPath, TesseraRuntime, ensure_build_phase_at, reserve_component_identity_in,
        with_replay_scope,
    },
    session::with_runtime_globals,
};

type PropUpdate = Box<dyn FnOnce(&mut dyn Any)>;

const MAX_RECYCLED_QUEUE_CAPACITY: usize = 256;

struct PendingCall {
    declaration: PendingDeclaration,
    configuration: PendingConfiguration,
}

struct PendingDeclaration {
    name: &'static str,
    parent: Option<NodeId>,
    execution_parent: Option<NodeId>,
    identity: u64,
    prop_owner: u64,
    creator: u64,
    group_path: GroupPath,
    key_override: Option<u64>,
    context: ContextMap,
}

struct PendingConfiguration {
    props: Rc<dyn ErasedComponent>,
    updates: Vec<(&'static str, usize, PropUpdate)>,
}

fn pending_props_mut(props: &mut Rc<dyn ErasedComponent>) -> &mut dyn ErasedComponent {
    Rc::get_mut(props).expect("pending props have a single owner")
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
    declaration: PendingDeclaration,
    configuration: Option<PendingConfiguration>,
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
        let entry = self.calls.remove(token.key)?;
        Some(PendingCall {
            declaration: entry.declaration,
            configuration: entry.configuration?,
        })
    }
}

#[derive(Default)]
pub(crate) struct ComponentCalls {
    pending: RefCell<PendingCalls>,
}

fn with_pending_calls<R>(access: impl FnOnce(&RefCell<PendingCalls>) -> R) -> R {
    with_runtime_globals(|globals| access(&globals.component_calls.pending))
}

/// A configuration handle to one registered component use.
pub struct ComponentCall<P: Prop> {
    session: RuntimeSessionId,
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
    fn owning_session(&self) -> RuntimeSession {
        let session = RuntimeSession::current();
        assert_eq!(
            session.as_ref().map(RuntimeSession::identity),
            Some(self.session),
            "component handle belongs to a different runtime session"
        );
        session.expect("validated component session")
    }
    /// Set a property while this call is pending.
    #[track_caller]
    pub fn configure<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P),
    {
        let session = self.owning_session();
        let mut guard =
            begin_configuration(&session.runtime_globals().component_calls.pending, self.id);
        let configuration = guard.configuration.as_mut().expect("active configuration");
        configuration.updates.retain(|entry| entry.0 != property);
        update(
            pending_props_mut(&mut configuration.props)
                .as_any_mut()
                .downcast_mut::<P>()
                .expect("registered component props type"),
        );
    }

    /// Materialize a property in this call's declaration scope before execution.
    #[track_caller]
    pub fn configure_deferred<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P) + 'static,
    {
        self.store(property, false, update);
    }

    /// Append one configuration operation to a property collection.
    #[track_caller]
    pub fn append<F>(&self, property: &'static str, update: F)
    where
        F: FnOnce(&mut P) + 'static,
    {
        self.store(property, true, update);
    }

    #[track_caller]
    fn store<F>(&self, property: &'static str, append: bool, update: F)
    where
        F: FnOnce(&mut P) + 'static,
    {
        let session = self.owning_session();
        let update: PropUpdate = Box::new(move |props| {
            update(
                props
                    .downcast_mut::<P>()
                    .expect("registered component props type"),
            );
        });
        store_update(
            &session.runtime_globals().component_calls.pending,
            self.id,
            property,
            append,
            update,
        );
    }

    /// Expand this call at a controlled runtime boundary.
    pub(crate) fn execute_now(self) {
        let _session = self.owning_session();
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
fn store_update(
    calls: &RefCell<PendingCalls>,
    id: PendingToken,
    property: &'static str,
    append: bool,
    update: PropUpdate,
) {
    let previous = {
        let mut calls = calls.borrow_mut();
        let configuration = calls
            .entry_mut(id)
            .and_then(|entry| entry.configuration.as_mut())
            .unwrap_or_else(|| panic!("component call {} is no longer Pending", id.serial));
        if append {
            let occurrence = configuration
                .updates
                .iter()
                .filter(|entry| entry.0 == property)
                .count();
            configuration.updates.push((property, occurrence, update));
            Vec::new()
        } else {
            let mut previous = Vec::new();
            let mut index = 0;
            while index < configuration.updates.len() {
                if configuration.updates[index].0 == property {
                    previous.push(configuration.updates.remove(index));
                } else {
                    index += 1;
                }
            }
            configuration.updates.push((property, 0, update));
            previous
        }
    };
    drop(previous);
}

struct ConfigurationGuard<'a> {
    calls: &'a RefCell<PendingCalls>,
    id: PendingToken,
    configuration: Option<PendingConfiguration>,
}

#[track_caller]
fn begin_configuration(calls: &RefCell<PendingCalls>, id: PendingToken) -> ConfigurationGuard<'_> {
    let configuration = calls
        .borrow_mut()
        .entry_mut(id)
        .and_then(|entry| entry.configuration.take())
        .unwrap_or_else(|| panic!("component call {} is no longer Pending", id.serial));
    ConfigurationGuard {
        calls,
        id,
        configuration: Some(configuration),
    }
}

impl Drop for ConfigurationGuard<'_> {
    fn drop(&mut self) {
        if let Some(entry) = self.calls.borrow_mut().entry_mut(self.id) {
            entry.configuration = self.configuration.take();
        }
    }
}

/// Register a component use in the active build fragment.
#[track_caller]
pub fn register_component_call<P: Prop>(
    name: &'static str,
    component_type_id: u64,
    props: P,
) -> ComponentCall<P> {
    let (session, id) = register_erased_call(name, component_type_id, Rc::new(props));
    ComponentCall {
        session,
        id,
        marker: PhantomData,
    }
}

#[track_caller]
fn register_erased_call(
    name: &'static str,
    component_type_id: u64,
    props: Rc<dyn ErasedComponent>,
) -> (RuntimeSessionId, PendingToken) {
    let session = RuntimeSession::current().expect("a runtime session must be active");
    let mut context = session.execution_context().borrow_mut();
    ensure_build_phase_at(context.phase_stack.last().copied());
    let globals = session.runtime_globals();
    let mut pending = globals.component_calls.pending.borrow_mut();
    let declaration = {
        assert!(
            !pending.scopes.is_empty(),
            "component declarations require an active build fragment"
        );
        let identity = reserve_component_identity_in(&mut context, component_type_id);
        let parent = globals.with_runtime(|runtime| runtime.component_tree.current_build_node_id());
        PendingDeclaration {
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
        }
    };
    drop(context);
    let serial = pending
        .next_serial
        .checked_add(1)
        .expect("component call token space exhausted");
    pending.next_serial = serial;
    let key = pending.calls.insert(PendingEntry {
        serial,
        declaration,
        configuration: Some(PendingConfiguration {
            props,
            updates: Vec::new(),
        }),
    });
    let token = PendingToken { key, serial };
    pending
        .scopes
        .last_mut()
        .expect("active build fragment")
        .push_back(token);
    (session.identity(), token)
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
    let PendingDeclaration {
        name,
        parent,
        execution_parent,
        identity,
        prop_owner,
        creator,
        group_path,
        key_override,
        context,
    } = call.declaration;
    let PendingConfiguration { mut props, updates } = call.configuration;
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
    // Pending declarations execute before this build can recycle context slots.
    if !updates.is_empty() {
        with_context_map(context.clone(), || {
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
        });
    }
    with_replay_scope(identity, group_path, key_override, context, || props.run());
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
    use std::{
        panic::{AssertUnwindSafe, catch_unwind},
        rc::Rc,
    };

    use crate::{
        RenderSlot,
        execution_context::reset_execution_context,
        prop::Prop,
        runtime::{GroupGuard, RuntimePhase, push_phase, reset_slots},
    };

    use super::{ComponentCall, register_component_call, with_pending_scope};

    #[derive(Clone)]
    struct Props {
        value: u32,
        slot: Option<RenderSlot>,
        recorder: Recorder,
    }

    impl Prop for Props {
        fn prop_eq(&self, other: &Self) -> bool {
            self.value == other.value && self.slot == other.slot
        }

        fn run_component(self: Rc<Self>) {
            self.recorder
                .observed
                .with_mut(|values| values.push((self.value, self.slot)));
        }
    }

    #[derive(Clone, Default)]
    struct Recorder {
        observed: crate::testing::TestProbe<Vec<(u32, Option<RenderSlot>)>>,
    }

    impl Recorder {
        fn declare(&self, value: u32) -> ComponentCall<Props> {
            register_component_call(
                "probe",
                123,
                Props {
                    value,
                    slot: None,
                    recorder: self.clone(),
                },
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
                .with(|values| values.iter().map(|(value, _)| *value).collect())
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
            let [(_, first_slot), (_, second_slot)] = first.as_slice() else {
                panic!("expected two recorded calls");
            };
            let [(_, observed_first), (_, observed_second)] = observed.as_slice() else {
                panic!("expected two replayed calls");
            };
            assert!(first_slot == observed_first);
            assert!(second_slot == observed_second);
            assert!(observed_first != observed_second);
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
            stale.configure("value", |props| {
                reset_slots();
                recorder.declare(2);
                props.value = 99;
            });
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
    fn configuration_rejects_a_handle_from_another_session_with_the_same_token() {
        let owner = crate::RuntimeSession::new();
        let other = crate::RuntimeSession::new();
        let first = Recorder::default();
        let second = Recorder::default();
        owner.run(|| {
            first.build(|| {
                let owned = first.declare(1);
                other.run(|| {
                    second.build(|| {
                        let local = second.declare(2);
                        assert!(owned.id == local.id);
                        assert!(
                            catch_unwind(AssertUnwindSafe(|| {
                                owned.configure("value", |props| props.value = 99);
                            }))
                            .is_err()
                        );
                    });
                });
                owned.configure("value", |props| props.value = 10);
            });
        });
        assert_eq!(first.values(), [10]);
        assert_eq!(second.values(), [2]);
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
