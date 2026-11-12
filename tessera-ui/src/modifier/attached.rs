use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem,
    rc::Rc,
};

use rustc_hash::{FxHashMap, FxHashSet};
use slotmap::{Key, SlotMap, new_key_type};
use smallvec::SmallVec;

use super::{
    Modifier, ModifierAction,
    diff::{Edit, Workspace},
    modifier_action_eq,
};

new_key_type! { struct Slot; }

/// Identity of one attached modifier use within a component instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModifierNodeId {
    session: u128,
    owner: u64,
    slot: u64,
}

/// The stable attachment identity passed to modifier lifecycle callbacks.
#[derive(Clone, Copy, Debug)]
pub struct ModifierNodeContext {
    /// Identity retained while compatible modifier inputs are updated.
    pub id: ModifierNodeId,
    /// Component instance that owns this attachment.
    pub component_instance: u64,
}

#[derive(Default)]
struct NodeResources {
    live: FxHashSet<ModifierNodeId>,
    in_use: FxHashSet<(ModifierNodeId, TypeId)>,
    values: FxHashMap<ModifierNodeId, FxHashMap<TypeId, Box<dyn Any>>>,
}

#[derive(Default)]
struct Resources(RefCell<NodeResources>);

impl ModifierNodeContext {
    /// Accesses state owned by this attachment, initializing it once per type.
    /// The value survives compatible updates and is released after `on_detach`.
    pub fn with_state<T: 'static, R>(
        &self,
        init: impl FnOnce() -> T,
        use_state: impl FnOnce(&mut T) -> R,
    ) -> R {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.id.session),
            "modifier context belongs to another session"
        );
        let resources = crate::local_runtime_resource::<Resources>();
        let key = (self.id, TypeId::of::<T>());
        let value = {
            let mut store = resources.0.borrow_mut();
            assert!(store.live.contains(&self.id), "modifier node is detached");
            assert!(
                store.in_use.insert(key),
                "modifier state is already borrowed"
            );
            store
                .values
                .get_mut(&self.id)
                .and_then(|values| values.remove(&key.1))
        };
        struct Restore {
            resources: Rc<Resources>,
            key: (ModifierNodeId, TypeId),
            value: Option<Box<dyn Any>>,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                let discarded = {
                    let mut store = self.resources.0.borrow_mut();
                    store.in_use.remove(&self.key);
                    if store.live.contains(&self.key.0) {
                        self.value.take().and_then(|value| {
                            store
                                .values
                                .entry(self.key.0)
                                .or_default()
                                .insert(self.key.1, value)
                        })
                    } else {
                        self.value.take()
                    }
                };
                drop(discarded);
            }
        }
        let mut restore = Restore {
            resources,
            key,
            value,
        };
        if restore.value.is_none() {
            restore.value = Some(Box::new(init()));
        }
        use_state(
            restore
                .value
                .as_mut()
                .expect("modifier state")
                .downcast_mut::<T>()
                .expect("modifier state type"),
        )
    }
}

#[derive(Clone, Copy)]
enum Event {
    Attach,
    Update,
    Detach,
}

struct Invocation {
    index: usize,
    context: ModifierNodeContext,
    event: Event,
}

struct Batch {
    modifier: Modifier,
    invocations: SmallVec<[Invocation; 4]>,
}

impl Batch {
    fn new(modifier: Modifier) -> Self {
        Self {
            modifier,
            invocations: SmallVec::new(),
        }
    }

    fn run(self) {
        let actions: SmallVec<[&ModifierAction; 8]> = self.modifier.element.actions().collect();
        let resources = crate::local_runtime_resource::<Resources>();
        for invocation in self.invocations {
            if matches!(invocation.event, Event::Attach) {
                resources.0.borrow_mut().live.insert(invocation.context.id);
            }
            struct Detach<'a> {
                resources: &'a Resources,
                id: ModifierNodeId,
                remove: bool,
            }
            impl Drop for Detach<'_> {
                fn drop(&mut self) {
                    if !self.remove {
                        return;
                    }
                    let removed = {
                        let mut store = self.resources.0.borrow_mut();
                        store.live.remove(&self.id);
                        store.values.remove(&self.id)
                    };
                    drop(removed);
                }
            }
            let _detach = Detach {
                resources: &resources,
                id: invocation.context.id,
                remove: matches!(invocation.event, Event::Detach),
            };
            actions[invocation.index].lifecycle(invocation.event, invocation.context);
        }
    }

    fn enqueue(self) {
        if !self.invocations.is_empty() {
            let _ = EVENTS.try_with(|events| events.pending.borrow_mut().push_back(self));
        }
    }
}

#[derive(Default)]
struct Events {
    pending: RefCell<VecDeque<Batch>>,
    flushing: Cell<bool>,
}

static EVENTS: crate::session::SessionLocal<Events> = crate::session::SessionLocal::new();

pub(crate) fn flush_lifecycle() {
    let _ = EVENTS.try_with(|events| {
        if events.flushing.replace(true) {
            return;
        }
        struct Guard<'a>(&'a Cell<bool>);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _guard = Guard(&events.flushing);
        loop {
            let batch = events.pending.borrow_mut().pop_front();
            let Some(batch) = batch else { break };
            batch.run();
        }
    });
}

struct Link {
    previous: Option<Slot>,
    next: Option<Slot>,
    index: usize,
}

#[derive(Default)]
pub(crate) struct AttachedModifierChain {
    current: Modifier,
    owner: u64,
    session: u128,
    slots: SlotMap<Slot, Link>,
    order: SmallVec<[Slot; 4]>,
    spare_order: SmallVec<[Slot; 4]>,
    head: Option<Slot>,
    tail: Option<Slot>,
    workspace: Workspace,
}

impl AttachedModifierChain {
    fn context(&self, slot: Slot) -> ModifierNodeContext {
        ModifierNodeContext {
            id: ModifierNodeId {
                session: self.session,
                owner: self.owner,
                slot: slot.data().as_ffi(),
            },
            component_instance: self.owner,
        }
    }

    pub(crate) fn update(
        &mut self,
        owner: u64,
        modifier: Modifier,
        draw: crate::layout::DrawModifierHandle,
    ) {
        let modifier = modifier.with_terminal_draw(draw);
        if self.current.element.is_same(&modifier.element) && self.owner == owner {
            return;
        }
        assert!(
            self.owner == owner || self.order.is_empty(),
            "modifier chain changed owner"
        );
        self.owner = owner;
        self.session = crate::session::current_session_identity().unwrap_or(0);
        let before: SmallVec<[&ModifierAction; 8]> = self.current.element.actions().collect();
        let after: SmallVec<[&ModifierAction; 8]> = modifier.element.actions().collect();
        let mut removed = Batch::new(self.current.clone());
        let mut changed = Batch::new(modifier.clone());
        let mut workspace = mem::take(&mut self.workspace);
        let edits = if before.len() == after.len()
            && before.iter().zip(&after).all(|(a, b)| a.same_type(b))
        {
            workspace.linear(before.len())
        } else {
            workspace.calculate(before.len(), after.len(), |a, b| {
                before[a].same_type(after[b])
            })
        };
        let mut old_order = mem::replace(&mut self.order, mem::take(&mut self.spare_order));
        self.order.clear();
        let mut cursor = None;
        for &edit in edits {
            match edit {
                Edit::Remove(index) => {
                    let key = old_order[index];
                    let context = self.context(key);
                    let link = self.slots.remove(key).expect("live modifier node");
                    if let Some(previous) = link.previous {
                        self.slots[previous].next = link.next;
                    } else {
                        self.head = link.next;
                    }
                    if let Some(next) = link.next {
                        self.slots[next].previous = link.previous;
                    } else {
                        self.tail = link.previous;
                    }
                    removed.invocations.push(Invocation {
                        index,
                        context,
                        event: Event::Detach,
                    });
                }
                Edit::Insert(index) => {
                    let next = cursor.map_or(self.head, |key| self.slots[key].next);
                    let key = self.slots.insert(Link {
                        previous: cursor,
                        next,
                        index,
                    });
                    if let Some(previous) = cursor {
                        self.slots[previous].next = Some(key);
                    } else {
                        self.head = Some(key);
                    }
                    if let Some(next) = next {
                        self.slots[next].previous = Some(key);
                    } else {
                        self.tail = Some(key);
                    }
                    cursor = Some(key);
                    self.order.push(key);
                    changed.invocations.push(Invocation {
                        index,
                        context: self.context(key),
                        event: Event::Attach,
                    });
                }
                Edit::Same(old, new) => {
                    let key = old_order[old];
                    self.slots[key].index = new;
                    cursor = Some(key);
                    self.order.push(key);
                    if !modifier_action_eq(before[old], after[new]) {
                        changed.invocations.push(Invocation {
                            index: new,
                            context: self.context(key),
                            event: Event::Update,
                        });
                    }
                }
            }
        }
        old_order.clear();
        self.spare_order = old_order;
        self.workspace = workspace;
        drop(before);
        drop(after);
        self.current = modifier;
        removed.enqueue();
        changed.enqueue();
    }
}

impl Drop for AttachedModifierChain {
    fn drop(&mut self) {
        let mut batch = Batch::new(mem::take(&mut self.current));
        for &key in self.order.iter().rev() {
            batch.invocations.push(Invocation {
                index: self.slots[key].index,
                context: self.context(key),
                event: Event::Detach,
            });
        }
        batch.enqueue();
    }
}

impl ModifierAction {
    fn same_type(&self, other: &Self) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }
        let kind = |action: &Self| match action {
            Self::Layout(node) => node.as_any().type_id(),
            Self::Placement(node) => node.as_any().type_id(),
            Self::Draw(node) => node.as_ref().type_id(),
            Self::ParentData(node) => node.as_ref().type_id(),
            Self::Build(node) => node.as_ref().type_id(),
            Self::BuildOwner(owner) => {
                let Some(Self::Build(node)) = owner.element.actions().next() else {
                    unreachable!("build owner")
                };
                node.as_ref().type_id()
            }
            Self::Semantics(node) => node.as_ref().type_id(),
            Self::Cursor(node) => node.as_ref().type_id(),
            Self::PointerPreviewInput(node)
            | Self::PointerInput(node)
            | Self::PointerFinalInput(node) => node.as_ref().type_id(),
            Self::KeyboardPreviewInput(node) | Self::KeyboardInput(node) => node.as_ref().type_id(),
            Self::ClipboardInput(node) => node.as_ref().type_id(),
            Self::ImePreviewInput(node) | Self::ImeInput(node) => node.as_ref().type_id(),
            Self::Focus(_) => TypeId::of::<super::FocusModifierOp>(),
            Self::GloballyPositioned(_) => {
                TypeId::of::<crate::component_tree::GloballyPositionedHandler>()
            }
            Self::TerminalDraw(node) => node.concrete_type_id(),
        };
        kind(self) == kind(other)
    }

    fn lifecycle(&self, event: Event, context: ModifierNodeContext) {
        macro_rules! call {
            ($node:expr) => {
                match event {
                    Event::Attach => $node.on_attach(context),
                    Event::Update => $node.on_update(context),
                    Event::Detach => $node.on_detach(context),
                }
            };
        }
        match self {
            Self::TerminalDraw(node) => call!(node.as_ref()),
            Self::Layout(node) => call!(node.node()),
            Self::Placement(node) => call!(node.node()),
            Self::Draw(node) => call!(node),
            Self::ParentData(node) => call!(node),
            Self::Build(node) => call!(node),
            Self::BuildOwner(owner) => owner
                .element
                .actions()
                .next()
                .expect("build owner")
                .lifecycle(event, context),
            Self::Semantics(node) => call!(node),
            Self::Cursor(node) => call!(node),
            Self::PointerPreviewInput(node)
            | Self::PointerInput(node)
            | Self::PointerFinalInput(node) => call!(node),
            Self::KeyboardPreviewInput(node) | Self::KeyboardInput(node) => call!(node),
            Self::ClipboardInput(node) => call!(node),
            Self::ImePreviewInput(node) | Self::ImeInput(node) => call!(node),
            Self::Focus(_) | Self::GloballyPositioned(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use crate::{
        RuntimeSession, State,
        layout::DrawModifierHandle,
        modifier::{DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier},
        remember,
        runtime::TesseraRuntime,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use super::{AttachedModifierChain, ModifierNodeContext, ModifierNodeId, flush_lifecycle};

    type Record = (u8, u8, ModifierNodeId, usize);

    struct Persistent {
        updates: usize,
        drops: Rc<Cell<usize>>,
    }

    impl Drop for Persistent {
        fn drop(&mut self) {
            TesseraRuntime::with_mut(|_| {});
            self.drops.set(self.drops.get() + 1);
        }
    }

    #[derive(Clone)]
    struct Spy<const KIND: u8> {
        label: u8,
        records: Rc<RefCell<Vec<Record>>>,
        drops: Rc<Cell<usize>>,
    }

    impl<const KIND: u8> PartialEq for Spy<KIND> {
        fn eq(&self, other: &Self) -> bool {
            self.label == other.label && Rc::ptr_eq(&self.records, &other.records)
        }
    }

    impl<const KIND: u8> Spy<KIND> {
        fn event(&self, context: ModifierNodeContext, event: u8) {
            context.with_state(
                || {
                    assert_eq!(
                        event, 0,
                        "updates must keep their original attachment state"
                    );
                    Persistent {
                        updates: 0,
                        drops: self.drops.clone(),
                    }
                },
                |state| {
                    if event == 1 {
                        state.updates += 1;
                    }
                    self.records
                        .borrow_mut()
                        .push((self.label, event, context.id, state.updates));
                },
            );
            TesseraRuntime::with_mut(|_| {});
        }
    }

    impl<const KIND: u8> DrawModifierNode for Spy<KIND> {
        fn on_attach(&self, context: ModifierNodeContext) {
            self.event(context, 0);
        }
        fn on_update(&self, context: ModifierNodeContext) {
            self.event(context, 1);
        }
        fn on_detach(&self, context: ModifierNodeContext) {
            self.event(context, 2);
        }
        fn draw(&self, _: &mut DrawModifierContext<'_, '_>, _: &mut dyn DrawModifierContent) {}
    }

    fn update(chain: &mut AttachedModifierChain, modifier: Modifier) {
        chain.update(1, modifier, DrawModifierHandle::default());
        flush_lifecycle();
    }

    #[crate::tessera]
    fn lifecycle_fixture(
        records: Rc<RefCell<Vec<Record>>>,
        drops: Rc<Cell<usize>>,
        state: Rc<Cell<Option<State<u8>>>>,
    ) {
        let value = remember(|| 0u8);
        state.set(Some(value));
        let value = value.get();
        if value < 2 {
            crate::layout::layout(crate::layout::Stack)
                .draw(Spy::<2> {
                    label: 30 + value,
                    records: records.clone(),
                    drops: drops.clone(),
                })
                .modifier(Modifier::new().push_draw(Spy::<1> {
                    label: 10 + value,
                    records,
                    drops,
                }));
        }
    }

    fn check_links(chain: &AttachedModifierChain) {
        let forward: Vec<_> = std::iter::successors(chain.head, |key| chain.slots[*key].next)
            .take(chain.order.len() + 1)
            .collect();
        let backward: Vec<_> = std::iter::successors(chain.tail, |key| chain.slots[*key].previous)
            .take(chain.order.len() + 1)
            .collect();
        assert_eq!(forward.as_slice(), chain.order.as_slice());
        assert_eq!(backward, forward.iter().rev().copied().collect::<Vec<_>>());
        assert_eq!(chain.slots.len(), chain.order.len());
        for (index, key) in chain.order.iter().enumerate() {
            assert_eq!(chain.slots[*key].index, index);
        }
    }

    #[test]
    fn compatible_updates_skip_diff_keep_state_and_reused_slots_get_new_identities() {
        RuntimeSession::new().run(|| {
            let records = Rc::new(RefCell::new(Vec::new()));
            let drops = Rc::new(Cell::new(0));
            let make = |label| {
                Modifier::new().push_draw(Spy::<1> {
                    label,
                    records: records.clone(),
                    drops: drops.clone(),
                })
            };
            let mut chain = AttachedModifierChain::default();
            update(&mut chain, make(1));
            let first = records.borrow()[0].2;
            assert!(chain.workspace.has_search_storage());
            let changed = make(2);
            assert!(!chain.current.element.is_same(&changed.element));
            chain.workspace = super::Workspace::default();
            update(&mut chain, changed);
            assert!(!chain.workspace.has_search_storage());
            assert_eq!(records.borrow()[1], (2, 1, first, 1));
            update(&mut chain, Modifier::new());
            assert_eq!(records.borrow()[2], (2, 2, first, 1));
            assert_eq!(drops.get(), 1);
            update(&mut chain, make(3));
            assert_ne!(records.borrow()[3].2, first);
            check_links(&chain);
            drop(chain);
            flush_lifecycle();
            assert_eq!(drops.get(), 2);
        });
    }

    #[test]
    fn queued_lifecycles_and_attachment_state_are_isolated_between_sessions() {
        let first_session = RuntimeSession::new();
        let second_session = RuntimeSession::new();
        let records = Rc::new(RefCell::new(Vec::new()));
        let drops = Rc::new(Cell::new(0));
        let shared = Modifier::new().push_draw(Spy::<1> {
            label: 1,
            records: records.clone(),
            drops: drops.clone(),
        });
        let mut first = AttachedModifierChain::default();
        let mut second = AttachedModifierChain::default();
        first_session.run(|| {
            first.update(1, shared.clone(), DrawModifierHandle::default());
        });
        second_session.run(|| {
            update(&mut second, shared);
            assert_eq!(records.borrow().len(), 1);
        });
        first_session.run(flush_lifecycle);
        assert_eq!(records.borrow().len(), 2);
        assert_ne!(records.borrow()[0].2, records.borrow()[1].2);
        first_session.run(|| {
            drop(first);
            flush_lifecycle();
        });
        second_session.run(|| {
            drop(second);
            flush_lifecycle();
        });
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn attachment_state_is_restored_after_a_panicking_access() {
        RuntimeSession::new().run(|| {
            let records = Rc::new(RefCell::new(Vec::new()));
            let drops = Rc::new(Cell::new(0));
            let mut chain = AttachedModifierChain::default();
            update(
                &mut chain,
                Modifier::new().push_draw(Spy::<1> {
                    label: 1,
                    records,
                    drops,
                }),
            );
            let context = chain.context(chain.order[0]);
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                context.with_state(
                    || 7usize,
                    |value| {
                        *value = 8;
                        panic!("state access failed");
                    },
                );
            }));
            assert!(panic.is_err());
            context.with_state(
                || panic!("state must already exist"),
                |value: &mut usize| {
                    assert_eq!(*value, 8);
                },
            );
            drop(chain);
            flush_lifecycle();
        });
    }

    struct Expanding(Spy<7>);

    impl crate::modifier::BuildModifierNode for Expanding {
        fn on_attach(&self, context: ModifierNodeContext) {
            self.0.event(context, 0);
        }
        fn on_update(&self, context: ModifierNodeContext) {
            self.0.event(context, 1);
        }
        fn on_detach(&self, context: ModifierNodeContext) {
            self.0.event(context, 2);
        }
        fn materialize(&self, _: &mut crate::modifier::ModifierBuildContext) -> Option<Modifier> {
            Some(Modifier::new().push_draw(crate::layout::NoopDrawModifier))
        }
    }

    #[test]
    fn materialization_preserves_the_build_owners_lifecycle_without_render_slots() {
        RuntimeSession::new().run(|| {
            let records = Rc::new(RefCell::new(Vec::new()));
            let drops = Rc::new(Cell::new(0));
            let source = Modifier::new().push_build(Expanding(Spy {
                label: 7,
                records: records.clone(),
                drops: drops.clone(),
            }));
            let materialized = source.materialize(1, 1, &[]);
            assert_eq!(materialized.ordered_actions().len(), 1);
            let mut chain = AttachedModifierChain::default();
            update(&mut chain, materialized);
            assert_eq!(records.borrow().len(), 1);
            assert_eq!(records.borrow()[0].1, 0);
            drop(chain);
            flush_lifecycle();
            assert_eq!(records.borrow()[1].1, 2);
            assert_eq!(drops.get(), 1);
        });
    }

    #[test]
    fn structural_diff_splices_around_repeated_types_without_remounting_survivors() {
        RuntimeSession::new().run(|| {
            let records = Rc::new(RefCell::new(Vec::new()));
            let drops = Rc::new(Cell::new(0));
            let a = Modifier::new().push_draw(Spy::<1> {
                label: 1,
                records: records.clone(),
                drops: drops.clone(),
            });
            let b = Modifier::new().push_draw(Spy::<2> {
                label: 2,
                records: records.clone(),
                drops: drops.clone(),
            });
            let c = Modifier::new().push_draw(Spy::<1> {
                label: 3,
                records: records.clone(),
                drops: drops.clone(),
            });
            let inserted = Modifier::new().push_draw(Spy::<3> {
                label: 4,
                records: records.clone(),
                drops: drops.clone(),
            });
            let mut chain = AttachedModifierChain::default();
            update(&mut chain, a.clone().then(b.clone()).then(c.clone()));
            let ids = records.borrow().iter().map(|r| r.2).collect::<Vec<_>>();
            update(
                &mut chain,
                a.then(inserted.clone()).then(b.clone()).then(c.clone()),
            );
            check_links(&chain);
            assert_eq!(records.borrow().len(), 4);
            assert_eq!((records.borrow()[3].0, records.borrow()[3].1), (4, 0));
            update(&mut chain, inserted.then(b).then(c));
            check_links(&chain);
            assert_eq!(records.borrow().len(), 5);
            assert_eq!(records.borrow()[4], (1, 2, ids[0], 0));
            assert_eq!(chain.context(chain.order[1]).id, ids[1]);
            assert_eq!(chain.context(chain.order[2]).id, ids[2]);
            drop(chain);
            flush_lifecycle();
            assert_eq!(drops.get(), 4);
        });
    }

    #[test]
    fn draw_and_chain_nodes_share_the_runtime_attachment_lifecycle() {
        let records = Rc::new(RefCell::new(Vec::new()));
        let drops = Rc::new(Cell::new(0));
        let state = Rc::new(Cell::new(None::<State<u8>>));
        let harness = {
            let records = records.clone();
            let drops = drops.clone();
            let state = state.clone();
            layout_test(move || {
                lifecycle_fixture(records.clone(), drops.clone(), state.clone());
            })
        };
        let mut session = start_layout_test_session(harness);
        pump_layout_test_session(&mut session, 0);
        let first = records.borrow().clone();
        assert_eq!(first.len(), 2);
        assert!(first.iter().all(|event| event.1 == 0));
        pump_layout_test_session(&mut session, 1);
        assert_eq!(records.borrow().len(), 2);
        session.run(|| state.get().unwrap().set(1));
        pump_layout_test_session(&mut session, 2);
        let second = records.borrow().clone();
        assert_eq!(second.len(), 4);
        for (attached, updated) in first.iter().zip(&second[2..]) {
            assert_eq!((updated.1, updated.2, updated.3), (1, attached.2, 1));
        }
        session.run(|| state.get().unwrap().set(2));
        pump_layout_test_session(&mut session, 3);
        assert_eq!(records.borrow().len(), 6);
        assert!(records.borrow()[4..].iter().all(|event| event.1 == 2));
        assert_eq!(drops.get(), 2);
        drop(session);
    }
}
