//! Background task handles for shard-owned async work.
//!
//! ## Usage
//!
//! Track spawned jobs in `TaskHandles` so they can be canceled together.

use std::future::Future;

use futures_util::future::{AbortHandle, Abortable};
use parking_lot::Mutex;

#[cfg(not(target_family = "wasm"))]
use tokio::task::JoinHandle;

#[cfg(target_family = "wasm")]
use wasm_bindgen_futures::spawn_local;

#[cfg(not(target_family = "wasm"))]
/// Platform-specific handle returned when spawning an asynchronous task.
pub type TaskRuntimeHandle = JoinHandle<()>;
#[cfg(target_family = "wasm")]
/// Platform-specific handle returned when spawning an asynchronous task.
pub type TaskRuntimeHandle = ();

/// Handle pair for a spawned background task.
pub struct TaskHandle {
    /// Runtime handle returned by the executor.
    pub handle: TaskRuntimeHandle,
    cancel: AbortHandle,
}

/// Tracks multiple background tasks and aborts them on drop.
pub struct TaskHandles {
    tasks: Mutex<Vec<TaskHandle>>,
}

impl Default for TaskHandles {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TaskHandles {
    fn drop(&mut self) {
        self.cancel_all();
    }
}

impl TaskHandles {
    /// Creates an empty task handle collection.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
        }
    }

    /// Spawns a task and stores its handle for later cancellation.
    pub fn spawn<F>(&self, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let (handle, cancel) = spawn_task(fut);
        self.tasks.lock().push(TaskHandle { handle, cancel });
    }

    /// Aborts and removes all tracked tasks.
    pub fn cancel_all(&self) {
        let mut tasks = self.tasks.lock();
        for task in tasks.drain(..) {
            task.cancel.abort();
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn spawn_task<F>(fut: F) -> (TaskRuntimeHandle, AbortHandle)
where
    F: Future<Output = ()> + Send + 'static,
{
    let (cancel, registration) = AbortHandle::new_pair();
    let wrapped = Abortable::new(fut, registration);
    let handle = crate::async_support::tokio_runtime::get().spawn(async move {
        let _ = wrapped.await;
    });
    (handle, cancel)
}

#[cfg(target_family = "wasm")]
fn spawn_task<F>(fut: F) -> (TaskRuntimeHandle, AbortHandle)
where
    F: Future<Output = ()> + Send + 'static,
{
    let (cancel, registration) = AbortHandle::new_pair();
    let wrapped = Abortable::new(fut, registration);
    spawn_local(async move {
        let _ = wrapped.await;
    });
    ((), cancel)
}
