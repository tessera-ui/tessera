//! Runtime sessions with explicit resource ownership.
//!
//! ## Usage
//!
//! Own independent UI roots and release their resources when the session ends.

use std::{
    any::{Any, TypeId},
    cell::{OnceCell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
    sync::Arc,
};

use rustc_hash::FxHashMap as HashMap;

use crate::execution_context::ExecutionContext;

#[derive(Default)]
struct Resources {
    shared: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    local: HashMap<TypeId, Rc<dyn Any>>,
    order: Vec<(TypeId, bool)>,
}

pub(crate) struct SessionState {
    identity: u128,
    root_claimed: std::cell::Cell<bool>,
    closing: std::cell::Cell<bool>,
    execution_context: OnceCell<RefCell<ExecutionContext>>,
    resources: RefCell<Resources>,
    previous: RefCell<Weak<SessionState>>,
}

thread_local! {
    static ACTIVE_SESSION: RefCell<Weak<SessionState>> = const { RefCell::new(Weak::new()) };
}

fn active_session_state() -> Option<Rc<SessionState>> {
    ACTIVE_SESSION
        .try_with(|active| active.borrow().upgrade())
        .ok()
        .flatten()
}

/// An owner for the resources of one UI session.
#[derive(Clone)]
pub struct RuntimeSession(Rc<SessionState>);

/// A non-owning reference used to deliver results to a live UI session.
#[derive(Clone)]
pub struct WeakRuntimeSession(Weak<SessionState>);

impl WeakRuntimeSession {
    /// Runs work on the owning UI thread while the session is still alive.
    pub fn run<R>(&self, work: impl FnOnce() -> R) -> Option<R> {
        self.0
            .upgrade()
            .filter(|owner| !owner.closing.get())
            .map(|owner| RuntimeSession(owner).run(work))
    }
}

/// The identity shared by observations from one runtime session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeSessionId(u128);

impl Default for RuntimeSession {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSession {
    /// Creates a reactive state lifetime owned by this UI session.
    pub fn state_owner(&self) -> crate::StateOwner {
        self.run(crate::StateOwner::new)
    }

    /// Captures result delivery to this session's UI thread.
    pub fn dispatcher(&self) -> crate::UiDispatcher {
        self.run(crate::dispatcher::current)
    }
    /// Returns a reference that does not keep this session alive.
    pub fn downgrade(&self) -> WeakRuntimeSession {
        WeakRuntimeSession(Rc::downgrade(&self.0))
    }
    /// Returns a shared resource owned by this session.
    pub fn resource<T: Default + Send + Sync + 'static>(&self) -> Arc<T> {
        self.run(runtime_resource::<T>)
    }

    /// Returns a resource that stays on this session's UI thread.
    pub fn local_resource<T: Default + 'static>(&self) -> Rc<T> {
        self.run(|| self.local_resource_in_session())
    }

    fn local_resource_in_session<T: Default + 'static>(&self) -> Rc<T> {
        let key = TypeId::of::<T>();
        if let Some(resource) = self.0.resources.borrow().local.get(&key).cloned() {
            return Rc::downcast(resource).unwrap_or_else(|_| unreachable!());
        }
        let resource = Rc::new(T::default());
        let stored = {
            let mut resources = self.0.resources.borrow_mut();
            if let Some(stored) = resources.local.get(&key) {
                stored.clone()
            } else {
                resources.local.insert(key, resource.clone());
                resources.order.push((key, false));
                resource.clone()
            }
        };
        Rc::downcast(stored).unwrap_or_else(|_| unreachable!())
    }

    /// Returns this session's identity for correlating observations.
    pub fn identity(&self) -> RuntimeSessionId {
        RuntimeSessionId(self.0.identity)
    }
    /// Creates an independent session.
    pub fn new() -> Self {
        Self(Rc::new(SessionState {
            identity: uuid::Uuid::new_v4().as_u128(),
            root_claimed: std::cell::Cell::new(false),
            closing: std::cell::Cell::new(false),
            execution_context: OnceCell::new(),
            resources: RefCell::new(Resources::default()),
            previous: RefCell::new(Weak::new()),
        }))
    }

    /// Runs work with this session's resources available.
    pub fn run<R>(&self, work: impl FnOnce() -> R) -> R {
        let _guard = self.enter();
        work()
    }

    /// Selects this session until the returned guard is dropped.
    pub fn enter(&self) -> RuntimeSessionGuard {
        let previous = ACTIVE_SESSION.with(|active| active.replace(Rc::downgrade(&self.0)));
        RuntimeSessionGuard {
            previous,
            _owner: self.clone(),
        }
    }

    pub(crate) fn activate(&self) {
        ACTIVE_SESSION.with(|active| {
            let mut active = active.borrow_mut();
            if !active.ptr_eq(&Rc::downgrade(&self.0)) {
                *self.0.previous.borrow_mut() = active.clone();
                *active = Rc::downgrade(&self.0);
            }
        });
    }

    pub(crate) fn current() -> Option<Self> {
        active_session_state().map(Self)
    }

    pub(crate) fn accepts_work(&self) -> bool {
        !self.0.closing.get()
    }

    pub(crate) fn for_root() -> Self {
        let session = Self::current()
            .filter(|session| !session.0.root_claimed.get())
            .unwrap_or_default();
        session.0.root_claimed.set(true);
        session
    }
}

pub(crate) fn current_session_identity() -> Option<u128> {
    active_session_state().map(|session| session.identity)
}

pub(crate) fn with_active_execution_context<R>(
    read: impl FnOnce(&RefCell<ExecutionContext>) -> R,
) -> R {
    let session = RuntimeSession::current().expect("a runtime session must be active");
    read(session.0.execution_context.get_or_init(RefCell::default))
}

impl Drop for RuntimeSession {
    #[inline]
    fn drop(&mut self) {
        if Rc::strong_count(&self.0) == 1 {
            self.shutdown();
        }
    }
}

impl RuntimeSession {
    #[cold]
    fn shutdown(&self) {
        self.0.closing.set(true);
        let previous = self.0.previous.borrow().clone();
        let was_active = ACTIVE_SESSION
            .try_with(|active| active.borrow().ptr_eq(&Rc::downgrade(&self.0)))
            .unwrap_or(false);
        {
            let _guard = self.enter();
            crate::dispatcher::close();
            crate::runtime::shutdown_runtime();
            loop {
                let removed = {
                    let mut resources = self.0.resources.borrow_mut();
                    resources.order.pop().map(|(key, shared)| {
                        if shared {
                            (resources.shared.remove(&key), None)
                        } else {
                            (None, resources.local.remove(&key))
                        }
                    })
                };
                let Some(removed) = removed else { break };
                drop(removed);
            }
        }
        if was_active {
            let _ = ACTIVE_SESSION.try_with(|active| active.replace(previous));
        }
    }
}

/// Restores the previous active session when a scoped operation finishes.
pub struct RuntimeSessionGuard {
    previous: Weak<SessionState>,
    _owner: RuntimeSession,
}

pub(crate) struct SessionLocal<T>(PhantomData<fn() -> T>);

impl<T: Default + 'static> SessionLocal<T> {
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }

    pub(crate) fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        self.try_with(read)
            .expect("a runtime session must be active")
    }

    pub(crate) fn try_with<R>(&self, read: impl FnOnce(&T) -> R) -> Result<R, ()> {
        let session = RuntimeSession::current().ok_or(())?;
        let resource = session.local_resource_in_session::<T>();
        Ok(read(&resource))
    }
}

impl<T: Default + 'static> SessionLocal<RefCell<T>> {
    pub(crate) fn with_borrow<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        self.with(|value| read(&value.borrow()))
    }

    pub(crate) fn with_borrow_mut<R>(&self, update: impl FnOnce(&mut T) -> R) -> R {
        self.with(|value| update(&mut value.borrow_mut()))
    }
}

impl<T: Copy + Default + 'static> SessionLocal<std::cell::Cell<T>> {
    pub(crate) fn get(&self) -> T {
        self.with(std::cell::Cell::get)
    }
    pub(crate) fn set(&self, value: T) {
        self.with(|slot| slot.set(value));
    }
    pub(crate) fn replace(&self, value: T) -> T {
        self.with(|slot| slot.replace(value))
    }
}

impl Drop for RuntimeSessionGuard {
    fn drop(&mut self) {
        let _ = ACTIVE_SESSION.try_with(|active| active.replace(self.previous.clone()));
    }
}

/// Returns a shared resource owned by the active UI session.
///
/// Without an active session, the returned resource is owned only by its caller.
pub fn runtime_resource<T: Default + Send + Sync + 'static>() -> Arc<T> {
    let Some(session) = RuntimeSession::current() else {
        return Arc::new(T::default());
    };
    let key = TypeId::of::<T>();
    if let Some(resource) = session.0.resources.borrow().shared.get(&key).cloned() {
        return Arc::downcast(resource).unwrap_or_else(|_| unreachable!());
    }
    let resource = Arc::new(T::default());
    let stored = {
        let mut resources = session.0.resources.borrow_mut();
        if let Some(stored) = resources.shared.get(&key) {
            stored.clone()
        } else {
            resources.shared.insert(key, resource.clone());
            resources.order.push((key, true));
            resource.clone()
        }
    };
    Arc::downcast(stored).unwrap_or_else(|_| unreachable!())
}

/// Returns a thread-local resource owned by the active UI session.
///
/// Use this for platform objects that must stay on their creating thread.
/// Without an active session, the returned resource is owned only by its caller.
pub fn local_runtime_resource<T: Default + 'static>() -> Rc<T> {
    let Some(session) = RuntimeSession::current() else {
        return Rc::new(T::default());
    };
    session.local_resource_in_session()
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use std::{cell::Cell, panic::AssertUnwindSafe, rc::Rc};

    use crate::{
        State, remember,
        testing::{TestProbe, layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use super::{RuntimeSession, local_runtime_resource};

    #[crate::tessera(crate)]
    fn observed_state(output: TestProbe<Option<State<usize>>>) {
        let state = remember(|| 7usize);
        let _value = state.get();
        output.set(Some(state));
    }

    #[test]
    fn independent_sessions_reject_foreign_states_and_release_their_resources() {
        let output = TestProbe::default();
        let mut first = start_layout_test_session(layout_test({
            let output = output.clone();
            move || observed_state(output.clone())
        }));
        pump_layout_test_session(&mut first, 0);
        let state = output.get().unwrap();
        let resource = first.run(local_runtime_resource::<Cell<usize>>);
        let weak = Rc::downgrade(&resource);
        drop(resource);

        let second = RuntimeSession::new();
        second.run(|| {
            assert!(!state.is_alive());
            assert!(std::panic::catch_unwind(AssertUnwindSafe(|| state.set(12))).is_err());
            assert_eq!(local_runtime_resource::<Cell<usize>>().get(), 0);
        });
        first.run(|| {
            assert_eq!(state.get(), 7);
            state.set(9);
        });
        pump_layout_test_session(&mut first, 1);
        assert_eq!(
            first
                .work()
                .count("observed_state", crate::WorkPhase::Build),
            1
        );
        let _selected = second.enter();
        drop(first);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            RuntimeSession::current().unwrap().identity(),
            second.identity()
        );
        second.run(|| assert!(!state.is_alive()));
    }

    #[derive(Default)]
    struct CleanupDependency(Cell<usize>);

    #[derive(Default)]
    struct DependentResource(std::cell::RefCell<Option<TestProbe<Vec<usize>>>>);

    impl Drop for DependentResource {
        fn drop(&mut self) {
            if let Some(output) = self.0.get_mut().take() {
                output.with_mut(|values| {
                    values.push(local_runtime_resource::<CleanupDependency>().0.get())
                });
            }
        }
    }

    #[test]
    fn session_resources_drop_after_dependents_with_owner_selected_and_registry_unborrowed() {
        let session = RuntimeSession::new();
        let output = TestProbe::default();
        session.run(|| {
            local_runtime_resource::<CleanupDependency>().0.set(42);
            local_runtime_resource::<DependentResource>()
                .0
                .replace(Some(output.clone()));
        });
        drop(session);
        assert_eq!(output.get(), [42]);
    }

    struct ObserveClockOnDrop(TestProbe<Vec<u64>>);

    impl Drop for ObserveClockOnDrop {
        fn drop(&mut self) {
            self.0
                .with_mut(|values| values.push(crate::current_frame_nanos()));
        }
    }

    #[crate::tessera(crate)]
    fn clock_probe(output: TestProbe<Vec<u64>>) {
        let active = remember(|| true);
        if active.get() {
            let capture = ObserveClockOnDrop(output.clone());
            crate::receive_frame_nanos(move |nanos| {
                assert_eq!(crate::current_frame_nanos(), nanos);
                let _ = &capture;
                output.with_mut(|values| values.push(nanos));
                active.set(false);
                crate::FrameNanosControl::Stop
            });
        }
    }

    #[test]
    fn frame_callbacks_and_stopped_captures_read_the_clock_without_borrowing_the_registry() {
        let output = TestProbe::default();
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || clock_probe(output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        assert_eq!(session.frame_clock_diagnostics().pending_subscriptions, 1);
        pump_layout_test_session(&mut session, 10);
        assert_eq!(output.get(), [10, 10]);
        assert_eq!(session.frame_clock_diagnostics().callbacks_executed, 1);
        assert_eq!(session.frame_clock_diagnostics().pending_subscriptions, 0);
        pump_layout_test_session(&mut session, 20);
        assert_eq!(session.frame_clock_diagnostics().callbacks_executed, 0);
    }

    #[crate::tessera(crate)]
    fn persistent_clock_probe(output: TestProbe<Vec<u64>>) {
        let capture = ObserveClockOnDrop(output);
        crate::receive_frame_nanos(move |_| {
            let _ = &capture;
            crate::FrameNanosControl::Continue
        });
    }

    #[test]
    fn shutdown_drops_clock_captures_while_the_owning_clock_remains_available() {
        let output = TestProbe::default();
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || persistent_clock_probe(output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        pump_layout_test_session(&mut session, 17);
        output.take();
        drop(session);
        assert_eq!(output.get(), [17]);
    }

    #[test]
    fn conditional_state_writes_preserve_idle_frames_and_replay_changed_readers() {
        let output = TestProbe::default();
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || observed_state(output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        let state = output.get().unwrap();
        assert!(!state.set_if_changed(7));
        assert!(!crate::runtime::has_pending_build_invalidations());
        pump_layout_test_session(&mut session, 1);
        assert_eq!(
            session
                .work()
                .count("observed_state", crate::WorkPhase::Build),
            0
        );
        assert!(state.set_if_changed(9));
        assert!(crate::runtime::has_pending_build_invalidations());
        pump_layout_test_session(&mut session, 2);
        assert_eq!(state.get(), 9);
        assert_eq!(
            session
                .work()
                .count("observed_state", crate::WorkPhase::Build),
            1
        );
    }

    #[crate::tessera(crate)]
    fn owned_state_reader(value: State<usize>, output: TestProbe<usize>) {
        output.set(value.get());
    }

    #[test]
    fn controller_owned_state_replays_readers_and_dispatches_worker_results_on_its_ui_session() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::StateOwner::new();
        let value = owner.state_with_key("route", || 7usize);
        assert!(
            owner.state_with_key("route", || panic!("existing state was reinitialized")) == value
        );
        let output = TestProbe::new(0usize);
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || owned_state_reader(value, output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        assert_eq!(output.get(), 7);
        let dispatcher = owner.dispatcher();
        assert!(
            std::thread::spawn(move || dispatcher.dispatch(move || value.set(11)))
                .join()
                .unwrap()
        );
        assert_eq!(output.get(), 7);
        pump_layout_test_session(&mut session, 1);
        assert_eq!(output.get(), 11);
        assert_eq!(
            session
                .work()
                .count("owned_state_reader", crate::WorkPhase::Build),
            1
        );
        drop(owner);
        assert!(!value.is_alive());
        let next = crate::StateOwner::new();
        let replacement = next.state_with_key("route", || 19usize);
        assert_eq!(replacement.get(), 19);
        assert!(!value.is_alive());
        assert!(std::panic::catch_unwind(AssertUnwindSafe(|| value.set(99))).is_err());
        assert_eq!(replacement.get(), 19);
    }

    struct StatefulModifier {
        extra: bool,
        output: TestProbe<Vec<State<usize>>>,
    }

    impl crate::BuildModifierNode for StatefulModifier {
        fn materialize(
            &self,
            context: &mut crate::ModifierBuildContext,
        ) -> Option<crate::Modifier> {
            if self.extra {
                let _ = context.state(|| false);
            }
            let state = context.state(|| 3usize);
            let _ = state.get();
            self.output.with_mut(|values| values.push(state));
            Some(crate::Modifier::new())
        }
    }

    #[crate::tessera(crate)]
    fn modifier_owner_fixture(
        visible: State<bool>,
        extra: State<bool>,
        output: TestProbe<Vec<State<usize>>>,
    ) {
        use crate::FocusModifierExt as _;
        let requester = remember(crate::FocusRequester::new).get();
        let mut modifier = crate::Modifier::new();
        let extra = extra.get();
        if extra {
            modifier = modifier
                .focus_requester(requester)
                .on_layout_coordinates(|_| {});
        }
        if visible.get() {
            modifier = modifier
                .push_build(StatefulModifier {
                    extra,
                    output: output.clone(),
                })
                .push_build(StatefulModifier { extra, output });
        }
        crate::layout::layout().modifier(modifier);
    }

    #[test]
    fn capability_state_survives_unrelated_modifiers_and_dies_with_its_attachment() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::StateOwner::new();
        let visible = owner.state_with_key("visible", || true);
        let extra = owner.state_with_key("extra", || false);
        let output = TestProbe::default();
        let unused = crate::Modifier::new().push_build(StatefulModifier {
            extra: false,
            output: output.clone(),
        });
        assert!(output.with(Vec::is_empty));
        drop(unused);
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || modifier_owner_fixture(visible, extra, output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        let original = output.take();
        assert_eq!(original.len(), 2);
        assert!(original[0] != original[1]);
        original[0].set(8);
        for (frame, enabled) in [(1, false), (2, true), (3, false), (4, true)] {
            extra.set(enabled);
            pump_layout_test_session(&mut session, frame);
            assert!(output.take() == original);
            assert_eq!(original[0].get(), 8);
            assert_eq!(original[1].get(), 3);
        }
        visible.set(false);
        pump_layout_test_session(&mut session, 5);
        assert!(original.iter().all(|state| !state.is_alive()));
        assert!(output.with(Vec::is_empty));
        visible.set(true);
        pump_layout_test_session(&mut session, 6);
        let replacement = output.take();
        assert_eq!(replacement.len(), 2);
        assert!(replacement[0] != original[0]);
        assert_eq!(replacement[0].get(), 3);
    }

    struct ClockModifier(TestProbe<Vec<u64>>);

    impl crate::BuildModifierNode for ClockModifier {
        fn materialize(
            &self,
            context: &mut crate::ModifierBuildContext,
        ) -> Option<crate::Modifier> {
            let output = self.0.clone();
            context.receive_frame_nanos(move |nanos| {
                output.with_mut(|values| values.push(nanos));
                crate::FrameNanosControl::Continue
            });
            Some(crate::Modifier::new())
        }
    }

    #[crate::tessera(crate)]
    fn clock_modifier_fixture(visible: State<bool>, output: TestProbe<Vec<u64>>) {
        let modifier = if visible.get() {
            crate::Modifier::new().push_build(ClockModifier(output))
        } else {
            crate::Modifier::new()
        };
        crate::layout::layout().modifier(modifier);
    }

    #[test]
    fn removing_a_stateful_modifier_unsubscribes_its_clock_before_the_next_frame() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::StateOwner::new();
        let visible = owner.state_with_key("visible", || true);
        let output = TestProbe::default();
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || clock_modifier_fixture(visible, output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        assert_eq!(session.frame_clock_diagnostics().pending_subscriptions, 1);
        pump_layout_test_session(&mut session, 1);
        assert_eq!(output.get(), [1]);
        assert_eq!(session.frame_clock_diagnostics().pending_subscriptions, 1);
        visible.set(false);
        pump_layout_test_session(&mut session, 2);
        assert_eq!(output.get(), [1, 2]);
        assert_eq!(session.frame_clock_diagnostics().pending_subscriptions, 0);
        pump_layout_test_session(&mut session, 3);
        assert_eq!(session.frame_clock_diagnostics().callbacks_executed, 0);
        assert_eq!(output.get(), [1, 2]);
    }

    struct DropIntoState(State<usize>);

    impl Drop for DropIntoState {
        fn drop(&mut self) {
            if self.0.is_alive() {
                self.0.with_mut(|value| *value += 1);
            }
        }
    }

    #[test]
    fn closing_an_owner_during_state_mutation_drops_its_taken_value_outside_the_slot_borrow() {
        let runtime = RuntimeSession::new();
        runtime.run(|| {
            let observer = crate::StateOwner::new();
            let observed = observer.state_with_key("drops", || 0usize);
            let owner = crate::StateOwner::new();
            let value = owner.state_with_key("resource", || DropIntoState(observed));
            value.with_mut(|_| owner.close());
            assert!(!value.is_alive());
            assert_eq!(observed.get(), 1);
        });
    }

    #[test]
    fn closing_a_state_owner_discards_pending_results_before_they_can_touch_freed_state() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::StateOwner::new();
        let value = owner.state_with_key("route", || 5usize);
        let dispatcher = owner.dispatcher();
        assert!(dispatcher.dispatch(move || value.set(9)));
        owner.close();
        assert!(!dispatcher.is_alive());
        assert!(!dispatcher.dispatch(|| panic!("closed owner accepted a result")));
        crate::dispatcher::flush();
        assert!(!value.is_alive());
        let next = crate::StateOwner::new();
        assert_eq!(next.state_with_key("route", || 13usize).get(), 13);
    }

    #[derive(Default)]
    struct DispatcherOnDrop(std::cell::RefCell<Option<TestProbe<Option<crate::UiDispatcher>>>>);

    impl Drop for DispatcherOnDrop {
        fn drop(&mut self) {
            if let Some(output) = self.0.get_mut().take() {
                output.set(Some(crate::dispatcher::current()));
            }
        }
    }

    #[test]
    fn services_created_during_resource_cleanup_cannot_reopen_a_closed_session() {
        let session = RuntimeSession::new();
        let output = TestProbe::default();
        let weak = session.downgrade();
        session
            .local_resource::<DispatcherOnDrop>()
            .0
            .replace(Some(output.clone()));
        drop(session);
        let dispatcher = output
            .take()
            .expect("resource destructor captured its session service");
        assert!(!dispatcher.is_alive());
        assert!(!dispatcher.dispatch(|| panic!("closed session accepted destructor work")));
        assert!(weak.run(|| ()).is_none());
    }

    #[test]
    fn session_shutdown_rejects_dispatched_work_and_closes_external_state_owners() {
        let runtime = RuntimeSession::new();
        let owner = runtime.state_owner();
        let dispatcher = runtime.dispatcher();
        let value = runtime.run(|| owner.state_with_key("value", || 7usize));
        drop(runtime);
        assert!(!owner.is_alive());
        assert!(!dispatcher.is_alive());
        assert!(!dispatcher.dispatch(|| panic!("shutdown work executed")));
        let other = RuntimeSession::new();
        other.run(|| assert!(!value.is_alive()));
    }
}
