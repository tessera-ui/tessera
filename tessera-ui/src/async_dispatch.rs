use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

use futures_util::future::{AbortHandle, Abortable};

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
pub struct Task(Option<AbortHandle>);

impl Task {
    /// Lets the task run to completion without a handle.
    pub fn detach(mut self) {
        self.0 = None;
    }

    /// Returns whether the task has finished or was cancelled.
    pub fn is_finished(&self) -> bool {
        self.0.as_ref().is_none_or(AbortHandle::is_aborted)
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
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
    Task(Some(abort))
}

/// Runs a `!Send` future on the current UI session, starting after the current
/// frame's work.
pub(crate) fn spawn_local(future: impl Future<Output = ()> + 'static) -> Task {
    let session = crate::dispatcher::register_current();
    let (abort, future) = abortable(future);
    let task = crate::local_runtime_resource::<MainTasks>()
        .tasks
        .borrow_mut()
        .insert(Some(Box::pin(future)));
    // The first poll waits for the next frame boundary, so spawning during a
    // build never runs task code inside that build.
    crate::dispatcher::dispatch_to_session(session, move || poll_main(task));
    Task(Some(abort))
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

type MainFuture = Pin<Box<dyn Future<Output = ()>>>;

#[derive(Default)]
struct MainTasks {
    tasks: RefCell<slotmap::SlotMap<slotmap::DefaultKey, Option<MainFuture>>>,
}

struct MainWaker {
    session: u128,
    task: slotmap::DefaultKey,
}

impl Wake for MainWaker {
    fn wake(self: Arc<Self>) {
        let task = self.task;
        crate::dispatcher::dispatch_to_session(self.session, move || poll_main(task));
    }
}

fn spawn_main(session: u128, future: impl Future<Output = ()> + Send + 'static) {
    // The first poll waits for the next frame boundary, so spawning during a
    // build never runs task code inside that build.
    crate::dispatcher::dispatch_to_session(session, move || {
        let task = crate::local_runtime_resource::<MainTasks>()
            .tasks
            .borrow_mut()
            .insert(Some(Box::pin(future)));
        poll_main(task);
    });
}

fn poll_main(task: slotmap::DefaultKey) {
    let tasks = crate::local_runtime_resource::<MainTasks>();
    let Some(mut future) = tasks
        .tasks
        .borrow_mut()
        .get_mut(task)
        .and_then(Option::take)
    else {
        return;
    };
    let session = crate::session::current_session_identity().expect("UI tasks run in a session");
    let waker = Waker::from(Arc::new(MainWaker { session, task }));
    // Tokio-based futures register with the shared runtime's reactor, whose
    // driver threads wake this task back onto the UI thread.
    #[cfg(not(target_family = "wasm"))]
    let _runtime = executor(Dispatcher::Default).enter();
    let poll = future.as_mut().poll(&mut Context::from_waker(&waker));
    let mut slots = tasks.tasks.borrow_mut();
    match poll {
        Poll::Ready(()) => {
            slots.remove(task);
        }
        Poll::Pending => {
            if let Some(slot) = slots.get_mut(task) {
                *slot = Some(future);
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
