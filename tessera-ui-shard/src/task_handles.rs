use std::future::Future;

use parking_lot::Mutex;
use tokio::{sync::oneshot, task::JoinHandle};

pub struct TaskHandle {
    handle: JoinHandle<()>,
    cancel: oneshot::Sender<()>,
}

pub struct TaskHandles {
    tasks: Mutex<Vec<TaskHandle>>,
}

impl Default for TaskHandles {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskHandles {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
        }
    }

    pub fn spawn<F>(&self, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let wrapped = async move {
            tokio::select! {
                _ = fut => {},
                _ = rx => {},
            }
        };
        let handle = crate::tokio_runtime::get().spawn(wrapped);
        self.tasks.lock().push(TaskHandle { handle, cancel: tx });
    }

    pub fn cancel_all(&self) {
        let mut tasks = self.tasks.lock();
        for task in tasks.drain(..) {
            let _ = task.cancel.send(());
        }
    }
}
