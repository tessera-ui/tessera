//! Registered component calls and build-scoped configuration.
//!
//! ## Usage
//!
//! Configure declared component uses before the runtime expands their content.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::VecDeque,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use rustc_hash::{FxHashMap, FxHasher};
use slotmap::{SlotMap, new_key_type};
use smallbox::{SmallBox, smallbox, space::S4};
use smallvec::SmallVec;

use crate::{
    NodeId, RuntimeSession, RuntimeSessionId,
    context::{ContextMap, with_context_map},
    execution_context::{OrderFrame, with_execution_context_mut},
    prop::{PendingProps, Prop},
    runtime::{
        GroupPath, TesseraRuntime, ensure_build_phase_at, reserve_component_identity_in,
        with_replay_scope,
    },
    session::with_runtime_globals,
};

// Typical callback and content captures stay inline.
type PropUpdate = SmallBox<dyn FnMut(&mut dyn Any), S4>;
// Most declarations configure at most a couple of content or callback props.
type PropUpdates = SmallVec<[(&'static str, usize, PropUpdate); 2]>;

const MAX_RECYCLED_QUEUE_CAPACITY: usize = 256;

struct PendingCall {
    declaration: PendingDeclaration,
    configuration: PendingConfiguration,
}

impl PendingCall {
    fn execute(self) {
        let PendingConfiguration { props, updates } = self.configuration;
        (props.execute)(self.declaration, updates, props.index);
    }

    fn discard(self) {
        let PendingConfiguration { props, updates } = self.configuration;
        drop(updates);
        (props.discard)(props.index);
    }
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
    props: PropsSlot,
    updates: PropUpdates,
}

/// Declared props stored in the per-type slab until execution.
#[derive(Clone, Copy)]
struct PropsSlot {
    index: u32,
    execute: fn(PendingDeclaration, PropUpdates, u32),
    discard: fn(u32),
}

fn execute_pending<P: Prop>(declaration: PendingDeclaration, updates: PropUpdates, index: u32) {
    let props = with_pending_calls(|calls| calls.borrow_mut().props.slab::<P>().take(index));
    execute_call(declaration, updates, &mut Some(props));
}

fn discard_pending<P: Prop>(index: u32) {
    let props = with_pending_calls(|calls| calls.borrow_mut().props.slab::<P>().take(index));
    drop(props);
}

struct PropSlab<P> {
    values: Vec<Option<P>>,
    free: Vec<u32>,
}

impl<P> PropSlab<P> {
    fn insert(&mut self, props: P) -> u32 {
        if let Some(index) = self.free.pop() {
            self.values[index as usize] = Some(props);
            return index;
        }
        let index = u32::try_from(self.values.len()).expect("pending props capacity exhausted");
        self.values.push(Some(props));
        index
    }

    fn get_mut(&mut self, index: u32) -> &mut P {
        self.values[index as usize]
            .as_mut()
            .expect("pending props are present")
    }

    fn lend(&mut self, index: u32) -> P {
        self.values[index as usize]
            .take()
            .expect("pending props are present")
    }

    fn restore(&mut self, index: u32, props: P) {
        self.values[index as usize] = Some(props);
    }

    fn release(&mut self, index: u32) {
        self.free.push(index);
    }

    fn take(&mut self, index: u32) -> P {
        let props = self.lend(index);
        self.release(index);
        props
    }
}

/// Pending props are stored by value so a declaration that is skipped as
/// unchanged never allocates.
#[derive(Default)]
struct PropSlabs(FxHashMap<TypeId, Box<dyn Any>>);

impl PropSlabs {
    fn slab<P: Prop>(&mut self) -> &mut PropSlab<P> {
        self.0
            .entry(TypeId::of::<P>())
            .or_insert_with(|| {
                Box::new(PropSlab::<P> {
                    values: Vec::new(),
                    free: Vec::new(),
                })
            })
            .downcast_mut()
            .expect("prop slab type")
    }
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
    props: PropSlabs,
    scopes: Vec<VecDeque<PendingToken>>,
    recycled_queues: Vec<VecDeque<PendingToken>>,
    next_serial: u64,
    epoch: u64,
}

fn pending_entry(
    calls: &mut SlotMap<PendingKey, PendingEntry>,
    token: PendingToken,
) -> Option<&mut PendingEntry> {
    calls
        .get_mut(token.key)
        .filter(|entry| entry.serial == token.serial)
}

#[track_caller]
fn pending_configuration(
    calls: &mut SlotMap<PendingKey, PendingEntry>,
    token: PendingToken,
) -> &mut PendingConfiguration {
    pending_entry(calls, token)
        .and_then(|entry| entry.configuration.as_mut())
        .unwrap_or_else(|| panic!("component call {} is no longer Pending", token.serial))
}

impl PendingCalls {
    fn push(&mut self, declaration: PendingDeclaration, props: PropsSlot) -> PendingToken {
        let serial = self
            .next_serial
            .checked_add(1)
            .expect("component call token space exhausted");
        self.next_serial = serial;
        let key = self.calls.insert(PendingEntry {
            serial,
            declaration,
            configuration: Some(PendingConfiguration {
                props,
                updates: SmallVec::new(),
            }),
        });
        let token = PendingToken { key, serial };
        self.scopes
            .last_mut()
            .expect("active build fragment")
            .push_back(token);
        token
    }

    fn remove(&mut self, token: PendingToken) -> Option<PendingCall> {
        pending_entry(&mut self.calls, token)?;
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
        let mut guard = ConfigurationGuard::<P>::begin(
            &session.runtime_globals().component_calls.pending,
            self.id,
        );
        let configuration = guard.configuration.as_mut().expect("active configuration");
        configuration.updates.retain(|entry| entry.0 != property);
        update(guard.props.as_mut().expect("active configuration"));
    }

    /// Assign a generated value property without detaching its pending call.
    /// The update must only move the previous value out; it is dropped after
    /// the pending registry borrow ends. Reentrant updates use `configure`.
    #[doc(hidden)]
    #[track_caller]
    pub fn configure_value<T>(&self, property: &'static str, update: impl FnOnce(&mut P) -> T) {
        let session = self.owning_session();
        let calls = &session.runtime_globals().component_calls.pending;
        let mut pending = calls.borrow_mut();
        let PendingCalls {
            calls: entries,
            props,
            ..
        } = &mut *pending;
        let configuration = pending_configuration(entries, self.id);
        if configuration
            .updates
            .iter()
            .any(|entry| entry.0 == property)
        {
            drop(pending);
            let mut previous = None;
            self.configure(property, |props| previous = Some(update(props)));
            drop(previous);
            return;
        }
        let previous = update(props.slab::<P>().get_mut(configuration.props.index));
        drop(pending);
        drop(previous);
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
        // `SmallBox` cannot call `dyn FnOnce` by value; each update runs once.
        let mut update = Some(update);
        let update: PropUpdate = smallbox!(move |props: &mut dyn Any| {
            (update.take().expect("property update runs once"))(
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
        let session = self.owning_session();
        let calls = &session.runtime_globals().component_calls.pending;
        loop {
            let (id, call) =
                take_next_call(calls).expect("render-slot call must belong to the active fragment");
            if let Some(call) = call {
                call.execute();
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
        let configuration = pending_configuration(&mut calls.calls, id);
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

/// Detaches a pending configuration and its props so the update may register
/// more calls.
struct ConfigurationGuard<'a, P: Prop> {
    calls: &'a RefCell<PendingCalls>,
    id: PendingToken,
    epoch: u64,
    configuration: Option<PendingConfiguration>,
    props: Option<P>,
}

impl<'a, P: Prop> ConfigurationGuard<'a, P> {
    #[track_caller]
    fn begin(calls: &'a RefCell<PendingCalls>, id: PendingToken) -> Self {
        let mut pending = calls.borrow_mut();
        let PendingCalls {
            calls: entries,
            props,
            epoch,
            ..
        } = &mut *pending;
        let configuration = pending_entry(entries, id)
            .and_then(|entry| entry.configuration.take())
            .unwrap_or_else(|| panic!("component call {} is no longer Pending", id.serial));
        let props = props.slab::<P>().lend(configuration.props.index);
        Self {
            calls,
            id,
            epoch: *epoch,
            configuration: Some(configuration),
            props: Some(props),
        }
    }
}

impl<P: Prop> Drop for ConfigurationGuard<'_, P> {
    fn drop(&mut self) {
        let mut pending = self.calls.borrow_mut();
        let PendingCalls {
            calls: entries,
            props,
            epoch,
            ..
        } = &mut *pending;
        let configuration = self.configuration.take().expect("active configuration");
        let index = configuration.props.index;
        if let Some(entry) = pending_entry(entries, self.id) {
            entry.configuration = Some(configuration);
            props
                .slab::<P>()
                .restore(index, self.props.take().expect("active configuration"));
        } else if *epoch == self.epoch {
            // The call was committed or abandoned while detached.
            props.slab::<P>().release(index);
        }
        drop(pending);
    }
}

/// Register a component use in the active build fragment.
#[track_caller]
pub fn register_component_call<P: Prop>(
    name: &'static str,
    component_type_id: u64,
    props: P,
) -> ComponentCall<P> {
    let session = RuntimeSession::current().expect("a runtime session must be active");
    let declaration = declare_call(&session, name, component_type_id);
    let mut pending = session
        .runtime_globals()
        .component_calls
        .pending
        .borrow_mut();
    let index = pending.props.slab::<P>().insert(props);
    let id = pending.push(
        declaration,
        PropsSlot {
            index,
            execute: execute_pending::<P>,
            discard: discard_pending::<P>,
        },
    );
    ComponentCall {
        session: session.identity(),
        id,
        marker: PhantomData,
    }
}

#[track_caller]
fn declare_call(
    session: &RuntimeSession,
    name: &'static str,
    component_type_id: u64,
) -> PendingDeclaration {
    let mut context = session.execution_context().borrow_mut();
    ensure_build_phase_at(context.phase_stack.last().copied());
    let globals = session.runtime_globals();
    assert!(
        !globals.component_calls.pending.borrow().scopes.is_empty(),
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
        abandoned.into_iter().for_each(PendingCall::discard);
    }
}

/// Build a fragment and expand all uses registered by that fragment.
pub fn with_pending_scope<R>(build: impl FnOnce() -> R) -> R {
    let _guard = begin_pending_scope();
    let result = build();
    commit_pending();
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

pub(crate) fn commit_pending() {
    with_pending_calls(|calls| {
        while let Some((_, call)) = take_next_call(calls) {
            if let Some(call) = call {
                call.execute();
            }
        }
    });
}

fn take_next_call(calls: &RefCell<PendingCalls>) -> Option<(PendingToken, Option<PendingCall>)> {
    let mut calls = calls.borrow_mut();
    let id = calls
        .scopes
        .last_mut()
        .expect("active build fragment")
        .pop_front()?;
    Some((id, calls.remove(id)))
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

fn execute_call(
    declaration: PendingDeclaration,
    updates: PropUpdates,
    props: &mut dyn PendingProps,
) {
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
    } = declaration;
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
            for (property, occurrence, mut update) in updates {
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
                update(props.as_any_mut());
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
        calls.epoch = calls.epoch.wrapping_add(1);
        (
            std::mem::take(&mut calls.calls),
            std::mem::take(&mut calls.props),
        )
    });
    drop(removed);
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::{
        RenderSlot,
        execution_context::reset_execution_context,
        prop::{ComponentProps, Prop},
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

        fn run_component(props: ComponentProps<Self>) {
            props
                .recorder
                .observed
                .with_mut(|values| values.push((props.value, props.slot)));
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
    fn direct_value_assignment_supersedes_a_deferred_property_update() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        recorder.build(|| {
            let call = recorder.declare(1);
            call.configure_deferred("value", |_| panic!("superseded update ran"));
            call.configure_value("value", |props| std::mem::replace(&mut props.value, 9));
        });
        assert_eq!(recorder.values(), [9]);
    }

    #[test]
    fn generated_value_assignment_drops_replaced_props_after_registry_borrow() {
        #[derive(Clone)]
        struct Value(Recorder, u32);
        impl Drop for Value {
            fn drop(&mut self) {
                if self.1 == 1 {
                    self.0.declare(2);
                }
            }
        }
        #[derive(Clone)]
        struct ValueProps(Option<Value>);
        impl Prop for ValueProps {
            fn prop_eq(&self, _other: &Self) -> bool {
                true
            }
            fn run_component(_: ComponentProps<Self>) {}
        }
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorder = Recorder::default();
        recorder.build(|| {
            let call = register_component_call("drop_probe", 345, ValueProps(None));
            call.configure_value("value", |props| {
                std::mem::replace(&mut props.0, Some(Value(recorder.clone(), 1)))
            });
            call.configure_value("value", |props| {
                std::mem::replace(&mut props.0, Some(Value(recorder.clone(), 0)))
            });
        });
        assert_eq!(recorder.values(), [2]);
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
                        .expect("test fixture provides abandoned")
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
