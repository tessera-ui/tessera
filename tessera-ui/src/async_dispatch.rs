use std::future::Future;

use futures_util::future::{AbortHandle, Abortable};

/// Execution context for background work.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Dispatcher {
    /// Run work on the platform's IO-capable executor.
    Io,
    /// Run work on the platform's general asynchronous executor.
    Compute,
}

/// Cancellation handle for one dispatched task.
#[derive(Clone)]
pub struct TaskHandle(AbortHandle);

impl TaskHandle {
    /// Requests cancellation of the task.
    pub fn cancel(&self) {
        self.0.abort();
    }

    /// Returns whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.is_aborted()
    }
}

/// Runs synchronous work on a background executor.
///
/// Use [`State::update`](crate::State::update) to deliver changes to UI-owned
/// state. Cancellation is best-effort once synchronous work has started.
pub fn dispatch(dispatcher: Dispatcher, work: impl FnOnce() + Send + 'static) -> TaskHandle {
    let (abort, registration) = AbortHandle::new_pair();
    let handle = TaskHandle(abort.clone());
    #[cfg(not(target_family = "wasm"))]
    match dispatcher {
        Dispatcher::Io => {
            let _ = executor(dispatcher).spawn_blocking(move || {
                if !abort.is_aborted() {
                    work();
                }
            });
        }
        Dispatcher::Compute => {
            let _ = executor(dispatcher).spawn(Abortable::new(async move { work() }, registration));
        }
    }
    #[cfg(target_family = "wasm")]
    {
        let _ = dispatcher;
        wasm_bindgen_futures::spawn_local(async move {
            let _ = Abortable::new(async move { work() }, registration).await;
        });
    }
    handle
}

pub(crate) fn launch(
    dispatcher: Dispatcher,
    work: impl Future<Output = ()> + Send + 'static,
) -> TaskHandle {
    let (abort, registration) = AbortHandle::new_pair();
    let handle = TaskHandle(abort);
    #[cfg(not(target_family = "wasm"))]
    {
        let _ = executor(dispatcher).spawn(Abortable::new(work, registration));
    }
    #[cfg(target_family = "wasm")]
    {
        let _ = dispatcher;
        wasm_bindgen_futures::spawn_local(async move {
            let _ = Abortable::new(work, registration).await;
        });
    }
    handle
}

#[cfg(not(target_family = "wasm"))]
fn executor(dispatcher: Dispatcher) -> &'static tokio::runtime::Runtime {
    static IO: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("tessera-io")
            .build()
            .expect("IO task executor")
    });
    static COMPUTE: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_time()
            .thread_name("tessera-compute")
            .build()
            .expect("compute task executor")
    });
    match dispatcher {
        Dispatcher::Io => &IO,
        Dispatcher::Compute => &COMPUTE,
    }
}
