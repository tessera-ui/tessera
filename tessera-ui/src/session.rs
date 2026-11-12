//! Runtime sessions with explicit resource ownership.
//!
//! ## Usage
//!
//! Own independent UI roots and release their resources when the session ends.

use std::{
    any::{Any, TypeId},
    cell::{Cell, OnceCell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
    sync::{
        Arc, Mutex as StdMutex, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::ThreadId,
};

use parking_lot::lock_api::{RawMutex as _, RawMutexFair as _};
use rustc_hash::FxHashMap as HashMap;

use crate::{execution_context::ExecutionContext, runtime::RuntimeGlobals};

#[derive(Default)]
struct Resources {
    shared: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    local: HashMap<TypeId, Rc<dyn Any>>,
    order: Vec<(TypeId, bool)>,
}

pub(crate) struct SessionState {
    identity: u128,
    root_claimed: Cell<bool>,
    closing: Cell<bool>,
    density: Cell<f64>,
    execution_context: OnceCell<RefCell<ExecutionContext>>,
    runtime_globals: OnceCell<Box<RuntimeGlobals>>,
    resources: RefCell<Resources>,
    previous: RefCell<SessionSelection>,
    gate: Arc<AccessGate>,
    gate_guard: RefCell<Option<GateGuard>>,
}

type GateGuard = <parking_lot::RawMutex as parking_lot::lock_api::RawMutex>::Guard;

/// Serializes other threads' state access with the owning UI thread.
///
/// The owning thread holds the lock while it runs and hands it over only at
/// frame boundaries, so its own access needs no locking and builds never see
/// another thread's write midway.
struct AccessGate {
    raw: parking_lot::RawMutex,
    waiting: AtomicUsize,
    closed: AtomicBool,
    owner: ThreadId,
}

struct GateEntry {
    gate: Arc<AccessGate>,
    state: SessionPtr,
}

#[derive(Clone, Copy)]
struct SessionPtr(*const SessionState);

// SAFETY: the pointer is dereferenced only by a thread holding the session's
// gate, which excludes the owning thread, and only before the session closes.
unsafe impl Send for SessionPtr {}

fn gates() -> &'static StdMutex<HashMap<u128, GateEntry>> {
    static GATES: OnceLock<StdMutex<HashMap<u128, GateEntry>>> = OnceLock::new();
    GATES.get_or_init(Default::default)
}

fn gate_entry(session: u128) -> Option<(Arc<AccessGate>, SessionPtr)> {
    gates()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&session)
        .map(|entry| (entry.gate.clone(), entry.state))
}

/// Runs `access` with `session` active when it is not the current session.
///
/// On the owning thread the session is entered directly. Other threads wait
/// until the owning thread reaches a frame boundary. Returns `None` when the
/// session has closed.
pub(crate) fn with_inactive_session<R>(session: u128, access: impl FnOnce() -> R) -> Option<R> {
    let (gate, state) = gate_entry(session)?;
    if gate.owner == std::thread::current().id() {
        // SAFETY: the registered pointer stays valid until the owning thread
        // closes the session, and this is the owning thread.
        let session = unsafe {
            Rc::increment_strong_count(state.0);
            RuntimeSession(Rc::from_raw(state.0))
        };
        return Some(session.run(access));
    }
    gate.waiting.fetch_add(1, Ordering::AcqRel);
    crate::dispatcher::dispatch_to_session(session, || {});
    let guard = gate.raw.lock();
    gate.waiting.fetch_sub(1, Ordering::AcqRel);
    struct Release<'a>(&'a AccessGate, Option<GateGuard>);
    impl Drop for Release<'_> {
        fn drop(&mut self) {
            if let Some(guard) = self.1.take() {
                // SAFETY: the guard came from locking this mutex.
                unsafe { self.0.raw.unlock_fair(guard) };
            }
        }
    }
    let _release = Release(&gate, Some(guard));
    if gate.closed.load(Ordering::Acquire) {
        return None;
    }
    // SAFETY: holding the open gate keeps the owner parked, so the session is
    // alive and its reference counts are not touched concurrently.
    let session = unsafe {
        Rc::increment_strong_count(state.0);
        RuntimeSession(Rc::from_raw(state.0))
    };
    let output = session.run(access);
    drop(session);
    Some(output)
}

#[cfg(test)]
pub(crate) fn is_open(session: u128) -> bool {
    gate_entry(session).is_some()
}

/// Lets threads waiting on the active session's gate run their access.
pub(crate) fn serve_other_threads() {
    let Some(session) = RuntimeSession::current() else {
        return;
    };
    let gate = &session.0.gate;
    if gate.waiting.load(Ordering::Acquire) == 0 {
        return;
    }
    if let Some(guard) = session.0.gate_guard.borrow_mut().as_mut() {
        // SAFETY: the guard came from locking this mutex on the owning thread.
        unsafe { gate.raw.bump(guard) };
    }
}

#[derive(Clone, Default)]
struct SessionSelection {
    state: Weak<SessionState>,
    identity: u128,
    // Only guards may own the selection; persistent activation remains weak.
    scoped: Option<Rc<SessionState>>,
}

impl SessionSelection {
    fn new(state: &Rc<SessionState>) -> Self {
        Self {
            state: Rc::downgrade(state),
            identity: state.identity,
            scoped: None,
        }
    }

    fn selects(&self, state: &Rc<SessionState>) -> bool {
        self.state.as_ptr() == Rc::as_ptr(state)
    }

    fn scoped(state: &Rc<SessionState>) -> Self {
        let mut selection = Self::new(state);
        selection.scoped = Some(state.clone());
        selection
    }
}

thread_local! {
    static ACTIVE_SESSION: RefCell<SessionSelection> = const { RefCell::new(SessionSelection {
        state: Weak::new(),
        identity: 0,
        scoped: None,
    }) };
}

fn active_session_state() -> Option<Rc<SessionState>> {
    ACTIVE_SESSION
        .try_with(|active| {
            let selected = active.borrow();
            selected.scoped.clone().or_else(|| selected.state.upgrade())
        })
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
    /// Returns a reference that does not keep this session alive.
    pub fn downgrade(&self) -> WeakRuntimeSession {
        WeakRuntimeSession(Rc::downgrade(&self.0))
    }
    /// Returns this session's resource of type `T`, creating it once.
    pub fn resource<T: Default + 'static>(&self) -> Rc<T> {
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
        let gate = Arc::new(AccessGate {
            raw: parking_lot::RawMutex::INIT,
            waiting: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
            owner: std::thread::current().id(),
        });
        let gate_guard = gate.raw.lock();
        let state = Rc::new(SessionState {
            identity: uuid::Uuid::new_v4().as_u128(),
            root_claimed: Cell::new(false),
            closing: Cell::new(false),
            density: Cell::new(1.0),
            execution_context: OnceCell::new(),
            runtime_globals: OnceCell::new(),
            resources: RefCell::new(Resources::default()),
            previous: RefCell::default(),
            gate: gate.clone(),
            gate_guard: RefCell::new(Some(gate_guard)),
        });
        gates()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                state.identity,
                GateEntry {
                    gate,
                    state: SessionPtr(Rc::as_ptr(&state)),
                },
            );
        Self(state)
    }

    /// Runs work with this session's resources available.
    pub fn run<R>(&self, work: impl FnOnce() -> R) -> R {
        let _guard = self.enter();
        work()
    }

    /// Selects this session until the returned guard is dropped.
    pub fn enter(&self) -> RuntimeSessionGuard {
        let previous =
            ACTIVE_SESSION.with(|active| active.replace(SessionSelection::scoped(&self.0)));
        RuntimeSessionGuard {
            previous,
            _owner: self.clone(),
        }
    }

    pub(crate) fn activate(&self) {
        ACTIVE_SESSION.with(|active| {
            let mut active = active.borrow_mut();
            if !active.selects(&self.0) {
                let mut previous = std::mem::replace(&mut *active, SessionSelection::new(&self.0));
                previous.scoped = None;
                *self.0.previous.borrow_mut() = previous;
            }
        });
    }

    /// Returns the active session, if any.
    pub fn current() -> Option<Self> {
        active_session_state().map(Self)
    }

    pub(crate) fn execution_context(&self) -> &RefCell<ExecutionContext> {
        self.0.execution_context.get_or_init(RefCell::default)
    }

    pub(crate) fn runtime_globals(&self) -> &RuntimeGlobals {
        self.0.runtime_globals.get_or_init(Box::default)
    }

    pub(crate) fn density(&self) -> f64 {
        self.0.density.get()
    }

    pub(crate) fn set_density(&self, density: f64) {
        self.0.density.set(density);
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
    ACTIVE_SESSION
        .try_with(|active| {
            let selected = active.borrow();
            (selected.state.strong_count() != 0).then_some(selected.identity)
        })
        .ok()
        .flatten()
}

#[inline]
pub(crate) fn with_active_execution_context<R>(
    read: impl FnOnce(&RefCell<ExecutionContext>) -> R,
) -> R {
    let session = RuntimeSession::current().expect("a runtime session must be active");
    read(session.execution_context())
}

#[inline]
pub(crate) fn with_runtime_globals<R>(read: impl FnOnce(&RuntimeGlobals) -> R) -> R {
    try_with_runtime_globals(read).expect("a runtime session must be active")
}

#[inline]
pub(crate) fn try_with_runtime_globals<R>(
    read: impl FnOnce(&RuntimeGlobals) -> R,
) -> Result<R, ()> {
    let session = RuntimeSession::current().ok_or(())?;
    Ok(read(session.runtime_globals()))
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
            .try_with(|active| active.borrow().selects(&self.0))
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
        gates()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.0.identity);
        self.0.gate.closed.store(true, Ordering::Release);
        if let Some(guard) = self.0.gate_guard.borrow_mut().take() {
            // SAFETY: the owning thread has held this guard since creation;
            // waiting threads now observe the closed session and return.
            unsafe { self.0.gate.raw.unlock(guard) };
        }
    }
}

/// Restores the previous active session when a scoped operation finishes.
pub struct RuntimeSessionGuard {
    previous: SessionSelection,
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
        let _ =
            ACTIVE_SESSION.try_with(|active| active.replace(std::mem::take(&mut self.previous)));
    }
}

/// Returns a shared resource owned by the active UI session.
///
/// Without an active session, the returned resource is owned only by its caller.
pub(crate) fn shared_runtime_resource<T: Default + Send + Sync + 'static>() -> Arc<T> {
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

/// Returns the active UI session's resource of type `T`, creating it once.
///
/// # Panics
///
/// Panics without an active session.
pub fn runtime_resource<T: Default + 'static>() -> Rc<T> {
    RuntimeSession::current()
        .expect("runtime resources require an active UI session")
        .local_resource_in_session()
}

pub(crate) fn local_runtime_resource<T: Default + 'static>() -> Rc<T> {
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

    #[crate::tessera]
    fn observed_state(output: TestProbe<Option<State<usize>>>) {
        let state = remember(|| 7usize);
        let _value = state.get();
        output.set(Some(state));
    }

    #[test]
    fn states_access_their_own_session_and_sessions_release_their_resources() {
        let output = TestProbe::default();
        let mut first = start_layout_test_session(layout_test({
            let output = output.clone();
            move || observed_state(output.clone())
        }));
        pump_layout_test_session(&mut first, 0);
        let state = output.get().expect("test fixture entry exists");
        let resource = first.run(local_runtime_resource::<Cell<usize>>);
        let weak = Rc::downgrade(&resource);
        drop(resource);

        let second = RuntimeSession::new();
        second.run(|| {
            assert_eq!(
                state.get(),
                7,
                "a state reads from the session that owns it"
            );
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
            RuntimeSession::current()
                .expect("test fixture provides current")
                .identity(),
            second.identity()
        );
        second.run(|| assert!(!state.is_alive()));
    }

    #[test]
    fn session_selection_restores_density_and_rejects_an_expired_owner() {
        let first = RuntimeSession::new();
        first.activate();
        crate::dp::set_scale_factor(2.0);
        let second = RuntimeSession::new();
        second.run(|| {
            assert_eq!(crate::dp::scale_factor(), 1.0);
            crate::dp::set_scale_factor(3.0);
        });
        assert_eq!(super::current_session_identity(), Some(first.identity().0));
        assert_eq!(crate::dp::scale_factor(), 2.0);

        let scope = second.enter();
        drop(first);
        assert_eq!(super::current_session_identity(), Some(second.identity().0));
        assert_eq!(crate::dp::scale_factor(), 3.0);
        drop(scope);

        assert_eq!(super::current_session_identity(), None);
        assert_eq!(crate::dp::scale_factor(), 1.0);
    }

    #[test]
    fn activation_during_a_scoped_selection_does_not_retain_the_previous_owner() {
        let first = RuntimeSession::new();
        let first_owner = Rc::downgrade(&first.0);
        let second = RuntimeSession::new();
        {
            let _scope = first.enter();
            second.activate();
            assert_eq!(
                RuntimeSession::current().unwrap().identity(),
                second.identity()
            );
        }
        drop(first);
        assert!(first_owner.upgrade().is_none());
        drop(second);
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

    #[crate::tessera]
    fn clock_probe(output: TestProbe<Vec<u64>>) {
        let active = remember(|| true);
        if active.get() {
            let capture = ObserveClockOnDrop(output.clone());
            crate::launch_once(async move {
                let nanos = crate::next_frame().await;
                assert_eq!(crate::current_frame_nanos(), nanos);
                let _capture = capture;
                output.with_mut(|values| values.push(nanos));
                active.set(false);
            });
        }
    }

    #[test]
    fn frame_tasks_and_finished_captures_read_the_clock_of_their_frame() {
        let output = TestProbe::default();
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || clock_probe(output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        pump_layout_test_session(&mut session, 10);
        assert_eq!(output.get(), [10, 10]);
        assert_eq!(session.frame_clock_diagnostics().resumed_tasks, 1);
        assert_eq!(session.frame_clock_diagnostics().waiting_tasks, 0);
        pump_layout_test_session(&mut session, 20);
        assert_eq!(session.frame_clock_diagnostics().resumed_tasks, 0);
    }

    #[crate::tessera]
    fn persistent_clock_probe(output: TestProbe<Vec<u64>>) {
        let capture = ObserveClockOnDrop(output);
        crate::launch_once(async move {
            let _capture = capture;
            loop {
                crate::next_frame().await;
            }
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
        let state = output.get().expect("test fixture entry exists");
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

    #[crate::tessera]
    fn owned_state_reader(value: State<usize>, output: TestProbe<usize>) {
        output.set(value.get());
    }

    #[test]
    fn controller_owned_state_replays_readers_and_dispatches_worker_results_on_its_ui_session() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::runtime::ModelScope::new();
        let value = owner.own_state(7usize);
        let output = TestProbe::new(0usize);
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || owned_state_reader(value, output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        assert_eq!(output.get(), 7);
        let target = crate::dispatcher::register_current();
        assert!(
            std::thread::spawn(move || {
                crate::dispatcher::dispatch_to_session(target, move || value.set(11))
            })
            .join()
            .expect("join succeeds in the test fixture")
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
        let next = crate::runtime::ModelScope::new();
        let replacement = next.own_state(19usize);
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

    #[crate::tessera]
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
        crate::layout::layout(crate::layout::Stack).modifier(modifier);
    }

    #[test]
    fn capability_state_survives_unrelated_modifiers_and_dies_with_its_attachment() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::runtime::ModelScope::new();
        let visible = owner.own_state(true);
        let extra = owner.own_state(false);
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
            context.launch(async move {
                loop {
                    let nanos = crate::next_frame().await;
                    output.with_mut(|values| values.push(nanos));
                }
            });
            Some(crate::Modifier::new())
        }
    }

    #[crate::tessera]
    fn clock_modifier_fixture(visible: State<bool>, output: TestProbe<Vec<u64>>) {
        let modifier = if visible.get() {
            crate::Modifier::new().push_build(ClockModifier(output))
        } else {
            crate::Modifier::new()
        };
        crate::layout::layout(crate::layout::Stack).modifier(modifier);
    }

    #[test]
    fn removing_a_stateful_modifier_cancels_its_frame_task() {
        let runtime = RuntimeSession::new();
        let _scope = runtime.enter();
        let owner = crate::runtime::ModelScope::new();
        let visible = owner.own_state(true);
        let output = TestProbe::default();
        let mut session = start_layout_test_session(layout_test({
            let output = output.clone();
            move || clock_modifier_fixture(visible, output.clone())
        }));
        pump_layout_test_session(&mut session, 0);
        pump_layout_test_session(&mut session, 1);
        assert_eq!(output.get(), [1]);
        assert_eq!(session.frame_clock_diagnostics().waiting_tasks, 1);
        visible.set(false);
        pump_layout_test_session(&mut session, 2);
        assert_eq!(output.get(), [1, 2]);
        assert_eq!(session.frame_clock_diagnostics().waiting_tasks, 0);
        pump_layout_test_session(&mut session, 3);
        assert_eq!(session.frame_clock_diagnostics().resumed_tasks, 0);
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
            let observer = crate::runtime::ModelScope::new();
            let observed = observer.own_state(0usize);
            let owner = crate::runtime::ModelScope::new();
            let value = owner.own_state(DropIntoState(observed));
            value.with_mut(|_| owner.close());
            assert!(!value.is_alive());
            assert_eq!(observed.get(), 1);
        });
    }

    #[derive(Default)]
    struct DispatcherOnDrop(std::cell::RefCell<Option<TestProbe<Option<u128>>>>);

    impl Drop for DispatcherOnDrop {
        fn drop(&mut self) {
            if let Some(output) = self.0.get_mut().take() {
                output.set(Some(crate::dispatcher::register_current()));
            }
        }
    }

    #[test]
    fn services_created_during_resource_cleanup_cannot_reopen_a_closed_session() {
        let session = RuntimeSession::new();
        let output = TestProbe::default();
        let weak = session.downgrade();
        session
            .resource::<DispatcherOnDrop>()
            .0
            .replace(Some(output.clone()));
        drop(session);
        let target = output
            .take()
            .expect("resource destructor captured its session service");
        assert!(!crate::dispatcher::dispatch_to_session(target, || {
            panic!("closed session accepted destructor work")
        }));
        assert!(weak.run(|| ()).is_none());
    }

    #[test]
    fn session_shutdown_rejects_dispatched_work_and_closes_external_models() {
        let runtime = RuntimeSession::new();
        let model = runtime.run(crate::runtime::ModelScope::new);
        let target = runtime.run(crate::dispatcher::register_current);
        let value = runtime.run(|| model.own_state(7usize));
        drop(runtime);
        assert!(!model.is_alive());
        assert!(!crate::dispatcher::dispatch_to_session(target, || {
            panic!("shutdown work executed")
        }));
        let other = RuntimeSession::new();
        other.run(|| assert!(!value.is_alive()));
    }
}
