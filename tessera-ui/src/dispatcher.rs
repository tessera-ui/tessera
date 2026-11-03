//! Result delivery to an owned UI session.
//!
//! ## Usage
//!
//! Return background results to a live UI owner before its next build.

use std::{collections::VecDeque, sync::Arc};

use parking_lot::Mutex;

type UiWork = Box<dyn FnOnce() + Send + 'static>;
type Wake = Arc<dyn Fn() + Send + Sync>;

struct QueueState {
    open: bool,
    pending: VecDeque<UiWork>,
    wake: Option<Wake>,
}

struct DispatchQueue(Mutex<QueueState>);

impl Default for DispatchQueue {
    fn default() -> Self {
        Self(Mutex::new(QueueState {
            open: crate::RuntimeSession::current().is_some_and(|session| session.accepts_work()),
            pending: VecDeque::new(),
            wake: None,
        }))
    }
}

/// A captured delivery service for one UI session.
#[derive(Clone)]
pub struct UiDispatcher {
    owner: crate::RuntimeSessionId,
    queue: Arc<DispatchQueue>,
    liveness: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
}

impl UiDispatcher {
    /// Queues work for the next UI frame, returning false after shutdown.
    ///
    /// An owner-scoped dispatcher runs work only while that owner remains alive.
    pub fn dispatch(&self, work: impl FnOnce() + Send + 'static) -> bool {
        if !self.is_alive() {
            return false;
        }
        let liveness = self.liveness.clone();
        let work = move || {
            if liveness.as_ref().is_none_or(|alive| alive()) {
                work();
            }
        };
        let wake = {
            let mut state = self.queue.0.lock();
            if !state.open {
                return false;
            }
            state.pending.push_back(Box::new(work));
            state.wake.clone()
        };
        if let Some(wake) = wake {
            wake();
        }
        true
    }

    /// Returns whether the destination session still accepts results.
    pub fn is_alive(&self) -> bool {
        let open = self.queue.0.lock().open;
        open && self.liveness.as_ref().is_none_or(|alive| alive())
    }

    pub(crate) fn scoped(mut self, alive: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        self.liveness = Some(Arc::new(alive));
        self
    }

    /// Returns the destination session's identity.
    pub fn session_id(&self) -> crate::RuntimeSessionId {
        self.owner
    }
}

pub(crate) fn current() -> UiDispatcher {
    let session =
        crate::RuntimeSession::current().expect("a UI dispatcher requires an active session");
    UiDispatcher {
        owner: session.identity(),
        queue: crate::runtime_resource::<DispatchQueue>(),
        liveness: None,
    }
}

pub(crate) fn set_waker(wake: Option<Wake>) {
    let queue = crate::runtime_resource::<DispatchQueue>();
    let (previous, pending) = {
        let mut state = queue.0.lock();
        let previous = std::mem::replace(&mut state.wake, wake.clone());
        (previous, !state.pending.is_empty())
    };
    drop(previous);
    if pending && let Some(wake) = wake {
        wake();
    }
}

pub(crate) fn has_pending() -> bool {
    crate::RuntimeSession::current().is_some()
        && !crate::runtime_resource::<DispatchQueue>()
            .0
            .lock()
            .pending
            .is_empty()
}

pub(crate) fn flush() {
    let queue = crate::runtime_resource::<DispatchQueue>();
    let work = std::mem::take(&mut queue.0.lock().pending);
    for work in work {
        work();
    }
}

pub(crate) fn close() {
    let queue = crate::runtime_resource::<DispatchQueue>();
    let removed = {
        let mut state = queue.0.lock();
        state.open = false;
        (std::mem::take(&mut state.pending), state.wake.take())
    };
    drop(removed);
}
