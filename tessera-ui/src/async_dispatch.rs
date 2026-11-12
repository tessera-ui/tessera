use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

use futures_util::future::{AbortHandle, Abortable};

#[cfg(test)]
mod allocation_tests;

/// Where a task runs.
///
/// On native targets every dispatcher runs tasks inside a Tokio 1 runtime
/// context, so Tokio-based libraries work in any task. On the web all
/// dispatchers run on the browser event loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Dispatcher {
    /// The UI session that spawned the task, polled between frames.
    Ui,
    /// A pool for work that blocks on IO.
    Io,
    /// A pool sized to the CPU cores for computation.
    Default,
}

/// A running task, cancelled when dropped.
///
/// Store the task where its lifetime belongs: a model state, a composition
/// slot, or a local variable. Call [`Task::detach`] to let it run to
/// completion on its own.
#[must_use = "dropping a task cancels it; call `detach` to let it run"]
pub struct Task {
    abort: Option<AbortHandle>,
    // A UI-session task, removed immediately when cancelled on its session.
    local: Option<(u128, TaskKey)>,
}

impl Task {
    /// Lets the task run to completion without a handle.
    pub fn detach(mut self) {
        self.abort = None;
        self.local = None;
    }

    /// Returns whether the task has finished or was cancelled.
    pub fn is_finished(&self) -> bool {
        self.abort.as_ref().is_none_or(AbortHandle::is_aborted)
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        let Some(handle) = self.abort.take() else {
            return;
        };
        handle.abort();
        if let Some((session, key)) = self.local
            && crate::session::current_session_identity() == Some(session)
            && crate::RuntimeSession::current().is_some_and(|session| session.accepts_work())
        {
            let removed = crate::local_runtime_resource::<UiTasks>()
                .tasks
                .borrow_mut()
                .remove(key);
            drop(removed);
        }
    }
}

thread_local! {
    // The UI session a task belongs to while one of its futures is polled.
    static TASK_SESSION: Cell<Option<u128>> = const { Cell::new(None) };
}

/// Runs `future` on `dispatcher`; dropping the returned [`Task`] cancels it.
///
/// [`Dispatcher::Ui`] targets the current UI session, or the session of the
/// task that calls `spawn`.
pub fn spawn(dispatcher: Dispatcher, future: impl Future<Output = ()> + Send + 'static) -> Task {
    let session = session_for(dispatcher);
    let (abort, future) = abortable(future);
    let future = SessionScoped { session, future };
    match dispatcher {
        Dispatcher::Ui => spawn_main(session.expect("checked by session_for"), future),
        #[cfg(not(target_family = "wasm"))]
        Dispatcher::Io => {
            // Each IO task owns a blocking-capable thread, so it may block
            // while still driving Tokio IO resources.
            let runtime = executor(dispatcher).handle().clone();
            let _ = runtime
                .clone()
                .spawn_blocking(move || runtime.block_on(future));
        }
        #[cfg(not(target_family = "wasm"))]
        Dispatcher::Default => {
            let _ = executor(dispatcher).spawn(future);
        }
        #[cfg(target_family = "wasm")]
        Dispatcher::Io | Dispatcher::Default => wasm_bindgen_futures::spawn_local(future),
    }
    Task {
        abort: Some(abort),
        local: None,
    }
}

/// Runs a `!Send` future on the current UI session, starting after the current
/// frame's work.
pub(crate) fn spawn_local(future: impl Future<Output = ()> + 'static) -> Task {
    let session = crate::dispatcher::register_current();
    let (abort, future) = abortable(future);
    let schedule = insert_task(Box::pin(future));
    let key = schedule.task;
    // Scheduling defers the first poll to the next frame boundary, so spawning
    // during a build never runs task code inside that build.
    schedule.wake();
    Task {
        abort: Some(abort),
        local: Some((session, key)),
    }
}

fn abortable<F: Future<Output = ()>>(future: F) -> (AbortHandle, impl Future<Output = ()>) {
    let (abort, registration) = AbortHandle::new_pair();
    let handle = abort.clone();
    let future = Abortable::new(future, registration);
    (abort, async move {
        let _ = future.await;
        // Marks the task finished for `Task::is_finished`.
        handle.abort();
    })
}

/// Runs `future` on `dispatcher` and returns its output.
///
/// Dropping the returned future cancels the work.
pub fn on<R: Send + 'static>(
    dispatcher: Dispatcher,
    future: impl Future<Output = R> + Send + 'static,
) -> impl Future<Output = R> + Send {
    let (sender, receiver) = futures_channel::oneshot::channel();
    let task = spawn(dispatcher, async move {
        let _ = sender.send(future.await);
    });
    async move {
        let output = receiver.await;
        drop(task);
        output.expect("a dispatched future completes unless cancelled")
    }
}

fn session_for(dispatcher: Dispatcher) -> Option<u128> {
    let session =
        crate::session::current_session_identity().or_else(|| TASK_SESSION.with(Cell::get));
    if dispatcher == Dispatcher::Ui {
        assert!(
            session.is_some(),
            "Dispatcher::Ui requires a UI session or a task spawned from one"
        );
    }
    session
}

struct SessionScoped<F> {
    session: Option<u128>,
    future: F,
}

impl<F: Future> Future for SessionScoped<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: `future` is structurally pinned and never moved out.
        let this = unsafe { self.get_unchecked_mut() };
        let previous = TASK_SESSION.with(|current| current.replace(this.session));
        let output = unsafe { Pin::new_unchecked(&mut this.future) }.poll(context);
        TASK_SESSION.with(|current| current.set(previous));
        output
    }
}

slotmap::new_key_type! {
    /// A task in a UI session's executor.
    pub(crate) struct TaskKey;
}

type UiFuture = Pin<Box<dyn Future<Output = ()>>>;

// Each task allocates its future and waker once when spawned; scheduling,
// polling, and frame waits reuse buffers so steady-state polling does not
// allocate.
struct UiTask {
    future: UiFuture,
    waker: Waker,
    schedule: Arc<TaskWaker>,
}

#[derive(Default)]
struct UiTasks {
    tasks: RefCell<slotmap::SlotMap<TaskKey, Option<UiTask>>>,
    ready: RefCell<Vec<TaskKey>>,
    frame_waiters: RefCell<Vec<TaskKey>>,
    resuming_frame: RefCell<Vec<TaskKey>>,
    frames_resumed: Cell<u64>,
    resumed_last_frame: Cell<usize>,
}

thread_local! {
    static CURRENT_TASK: Cell<Option<TaskKey>> = const { Cell::new(None) };
}

struct TaskWaker {
    task: TaskKey,
    queue: Arc<crate::dispatcher::DispatchQueue>,
    scheduled: AtomicBool,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if !self.scheduled.swap(true, Ordering::AcqRel) {
            self.queue.schedule(self.task);
        }
    }
}

fn insert_task(future: UiFuture) -> Arc<TaskWaker> {
    let queue = crate::dispatcher::current_queue();
    let tasks = crate::local_runtime_resource::<UiTasks>();
    let mut slots = tasks.tasks.borrow_mut();
    let task = slots.insert(None);
    let schedule = Arc::new(TaskWaker {
        task,
        queue,
        scheduled: AtomicBool::new(false),
    });
    slots[task] = Some(UiTask {
        future,
        waker: Waker::from(schedule.clone()),
        schedule: schedule.clone(),
    });
    schedule
}

fn spawn_main(session: u128, future: impl Future<Output = ()> + Send + 'static) {
    crate::dispatcher::dispatch_to_session(session, move || {
        insert_task(Box::pin(future)).wake();
    });
}

/// Polls one task; returns whether it was still live.
fn poll_task(tasks: &UiTasks, key: TaskKey) -> bool {
    let Some(mut task) = tasks.tasks.borrow_mut().get_mut(key).and_then(Option::take) else {
        return false;
    };
    task.schedule.scheduled.store(false, Ordering::Release);
    let previous = CURRENT_TASK.with(|current| current.replace(Some(key)));
    let poll = task
        .future
        .as_mut()
        .poll(&mut Context::from_waker(&task.waker));
    CURRENT_TASK.with(|current| current.set(previous));
    let mut slots = tasks.tasks.borrow_mut();
    match poll {
        Poll::Ready(()) => {
            slots.remove(key);
            drop(slots);
            drop(task);
        }
        Poll::Pending => {
            if let Some(slot) = slots.get_mut(key) {
                *slot = Some(task);
            }
        }
    }
    true
}

fn poll_batch(
    tasks: &UiTasks,
    batch: &RefCell<Vec<TaskKey>>,
    fill: impl FnOnce(&mut Vec<TaskKey>),
) -> usize {
    let mut keys = std::mem::take(&mut *batch.borrow_mut());
    fill(&mut keys);
    // Tokio-based futures register with the shared runtime's reactor, whose
    // driver threads wake these tasks back onto the UI thread.
    #[cfg(not(target_family = "wasm"))]
    let _runtime = (!keys.is_empty()).then(|| executor(Dispatcher::Default).enter());
    let mut polled = 0;
    for key in keys.drain(..) {
        polled += usize::from(poll_task(tasks, key));
    }
    // Keep the drained buffer's capacity for the next batch.
    *batch.borrow_mut() = keys;
    polled
}

/// Polls the tasks woken since the last call.
pub(crate) fn run_ready(queue: &crate::dispatcher::DispatchQueue) {
    let tasks = crate::local_runtime_resource::<UiTasks>();
    poll_batch(&tasks, &tasks.ready, |keys| queue.take_ready(keys));
}

/// Resumes the tasks waiting in [`with_frame_time`] for the current frame.
pub(crate) fn resume_frame_waiters() {
    let tasks = crate::local_runtime_resource::<UiTasks>();
    tasks.frames_resumed.set(tasks.frames_resumed.get() + 1);
    let resumed = poll_batch(&tasks, &tasks.resuming_frame, |keys| {
        std::mem::swap(keys, &mut *tasks.frame_waiters.borrow_mut());
    });
    tasks.resumed_last_frame.set(resumed);
}

/// Returns whether a live task waits for the next frame.
pub(crate) fn has_frame_waiters() -> bool {
    frame_waiter_count() > 0
}

pub(crate) fn frame_waiter_count() -> usize {
    let tasks = crate::local_runtime_resource::<UiTasks>();
    let slots = tasks.tasks.borrow();
    tasks
        .frame_waiters
        .borrow()
        .iter()
        .filter(|key| slots.contains_key(**key))
        .count()
}

#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
pub(crate) fn frame_waiters_resumed_last_frame() -> usize {
    crate::local_runtime_resource::<UiTasks>()
        .resumed_last_frame
        .get()
}

/// Waits for the next frame and runs `frame` with its time.
///
/// This is the only way UI code observes time: builds, controllers, and input
/// handlers do not read a clock. Await it in a UI task, such as one started by
/// [`crate::launch`]. Each frame resumes waiting tasks after delivering queued
/// results and before the build, so state written by `frame` appears in that
/// frame. Every call waits for a new frame, and all tasks resumed by one frame
/// receive the same time. Animate by awaiting it in a loop that records the
/// first frame as the start time, writes state from each later frame, and
/// breaks when the motion settles.
///
/// # Panics
///
/// Panics when polled outside a UI task.
pub fn with_frame_time<R>(
    frame: impl FnOnce(crate::time::Instant) -> R,
) -> impl Future<Output = R> {
    let next = NextFrame { since: None };
    async move { frame(next.await) }
}

struct NextFrame {
    since: Option<u64>,
}

impl Future for NextFrame {
    type Output = crate::time::Instant;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let tasks = crate::local_runtime_resource::<UiTasks>();
        let resumed = tasks.frames_resumed.get();
        match self.since {
            Some(since) if resumed > since => Poll::Ready(crate::runtime::frame_time()),
            Some(_) => Poll::Pending,
            None => {
                let task = CURRENT_TASK
                    .with(Cell::get)
                    .expect("with_frame_time must be awaited in a UI task");
                self.since = Some(resumed);
                let mut waiters = tasks.frame_waiters.borrow_mut();
                if waiters.is_empty() {
                    crate::runtime::schedule_runtime_redraw();
                }
                waiters.push(task);
                Poll::Pending
            }
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn executor(dispatcher: Dispatcher) -> &'static tokio::runtime::Runtime {
    static IO: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("tessera-io")
            .build()
            .expect("IO task executor")
    });
    static DEFAULT: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("tessera-default")
            .build()
            .expect("default task executor")
    });
    match dispatcher {
        Dispatcher::Io => &IO,
        Dispatcher::Default | Dispatcher::Ui => &DEFAULT,
    }
}
