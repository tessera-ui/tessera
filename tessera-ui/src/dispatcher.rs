//! Result delivery to an owned UI session.
//!
//! ## Usage
//!
//! Return background results to a live UI owner before its next build.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex as StdMutex, OnceLock, Weak},
};

use parking_lot::Mutex;

type UiWork = Box<dyn FnOnce() + Send + 'static>;
type Wake = Arc<dyn Fn() + Send + Sync>;

struct QueueState {
    open: bool,
    pending: VecDeque<UiWork>,
    wake: Option<Wake>,
}

struct DispatchQueue(Mutex<QueueState>);

fn queues() -> &'static StdMutex<HashMap<u128, Weak<DispatchQueue>>> {
    static QUEUES: OnceLock<StdMutex<HashMap<u128, Weak<DispatchQueue>>>> = OnceLock::new();
    QUEUES.get_or_init(|| StdMutex::new(HashMap::new()))
}

fn register(session: u128, queue: &Arc<DispatchQueue>) {
    let mut queues = queues()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    queues.retain(|_, queue| queue.strong_count() > 0);
    queues.insert(session, Arc::downgrade(queue));
}

pub(crate) fn dispatch_to_session(session: u128, work: impl FnOnce() + Send + 'static) -> bool {
    let queue = queues()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&session)
        .and_then(Weak::upgrade);
    let Some(queue) = queue else { return false };
    let wake = {
        let mut state = queue.0.lock();
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

impl Default for DispatchQueue {
    fn default() -> Self {
        Self(Mutex::new(QueueState {
            open: crate::RuntimeSession::current().is_some_and(|session| session.accepts_work()),
            pending: VecDeque::new(),
            wake: None,
        }))
    }
}

/// Registers the active session's queue and returns its identity for
/// [`dispatch_to_session`].
pub(crate) fn register_current() -> u128 {
    let queue = crate::shared_runtime_resource::<DispatchQueue>();
    let identity = crate::session::current_session_identity().expect("active UI session");
    register(identity, &queue);
    identity
}

pub(crate) fn set_waker(wake: Option<Wake>) {
    let queue = crate::shared_runtime_resource::<DispatchQueue>();
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
        && !crate::shared_runtime_resource::<DispatchQueue>()
            .0
            .lock()
            .pending
            .is_empty()
}

pub(crate) fn flush() {
    crate::session::serve_other_threads();
    let queue = crate::shared_runtime_resource::<DispatchQueue>();
    let work = std::mem::take(&mut queue.0.lock().pending);
    for work in work {
        work();
    }
}

pub(crate) fn close() {
    let queue = crate::shared_runtime_resource::<DispatchQueue>();
    if let Some(session) = crate::session::current_session_identity() {
        queues()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&session);
    }
    let removed = {
        let mut state = queue.0.lock();
        state.open = false;
        (std::mem::take(&mut state.pending), state.wake.take())
    };
    drop(removed);
}
