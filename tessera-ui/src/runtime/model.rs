//! State and asynchronous work scoped to a component or route lifetime.
//!
//! ## Usage
//!
//! Create a model scope and share it with components that use its state.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    hash::{Hash, Hasher},
    panic::Location,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use parking_lot::Mutex;
use rustc_hash::FxHasher;

use super::{SlotKey, State, resolve_state_key, with_slot_table_mut};

struct ModelLease {
    id: u128,
    session: u128,
    closed: AtomicBool,
    dispatcher: crate::UiDispatcher,
    tasks: Mutex<HashMap<TaskSite, ModelTask>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TaskSite {
    file: &'static str,
    line: u32,
    column: u32,
    instance: u64,
    group_path: u64,
    ordinal: u64,
}

struct TaskKey {
    value: Box<dyn Any + Send>,
    type_id: TypeId,
    equal: fn(&dyn Any, &dyn Any) -> bool,
}

impl TaskKey {
    fn new<K: Any + Eq + Send>(key: K) -> Self {
        fn equal<K: Any + Eq>(left: &dyn Any, right: &dyn Any) -> bool {
            left.downcast_ref::<K>() == right.downcast_ref::<K>()
        }
        Self {
            value: Box::new(key),
            type_id: TypeId::of::<K>(),
            equal: equal::<K>,
        }
    }

    fn matches<K: Any>(&self, key: &K) -> bool {
        self.type_id == TypeId::of::<K>() && (self.equal)(self.value.as_ref(), key)
    }
}

struct ModelTask {
    key: TaskKey,
    handle: crate::TaskHandle,
    current: Arc<AtomicBool>,
}

/// A task's validity token for delivering results to UI-owned state.
#[derive(Clone)]
pub struct ModelTaskContext {
    current: Arc<AtomicBool>,
}

impl ModelTaskContext {
    /// Returns whether this task still has the active key at its call site.
    pub fn is_current(&self) -> bool {
        self.current.load(Ordering::Acquire)
    }

    /// Applies a mutation only while this task is current and the state is live.
    pub fn update<T: 'static>(
        &self,
        state: State<T>,
        update: impl FnOnce(&mut T) + Send + 'static,
    ) -> bool {
        if !self.is_current() {
            return false;
        }
        let current = self.current.clone();
        state.update(move |value| {
            if current.load(Ordering::Acquire) {
                update(value);
            }
        })
    }
}

impl ModelLease {
    fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let tasks = std::mem::take(&mut *self.tasks.lock());
        for task in tasks.values() {
            task.current.store(false, Ordering::Release);
            task.handle.cancel();
        }
        drop(tasks);
        let owner = self.id;
        if crate::session::current_session_identity() == Some(self.session) {
            release_slots(owner);
        } else {
            self.dispatcher.dispatch(move || release_slots(owner));
        }
    }
}

impl Drop for ModelLease {
    fn drop(&mut self) {
        self.close();
    }
}

/// Owns keyed reactive state and asynchronous work for one UI lifetime.
///
/// Create a model with [`remember_model`] for component-scoped lifetime, or use
/// one as a route-entry scope. Clones share the lifetime. Closing the model, or
/// dropping its final clone, releases its state. Cleanup requested from another
/// thread runs on the owning UI session before its next frame.
///
/// ```
/// use tessera_ui::{Dispatcher, Model, RuntimeSession};
///
/// let session = RuntimeSession::new();
/// session.run(|| {
///     let model = Model::new();
///     let count = model.state_with_key("count", || 0usize);
///     count.with_mut(|value| *value += 1);
///     assert_eq!(count.get(), 1);
///     let launches = (0..2)
///         .map(|_| model.launch(Dispatcher::Compute, "load", |_| async {}))
///         .collect::<Vec<_>>();
///     assert_eq!(launches, [true, false]);
///     model.close();
///     assert!(!count.is_alive());
/// });
/// ```
#[derive(Clone)]
pub struct Model(Arc<ModelLease>);

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Model {}

impl Model {
    /// Creates a model scope in the current UI session.
    pub fn new() -> Self {
        let owner = Arc::new(ModelLease {
            id: uuid::Uuid::new_v4().as_u128(),
            session: crate::session::current_session_identity()
                .expect("models require an active UI session"),
            closed: AtomicBool::new(false),
            dispatcher: crate::dispatcher::current(),
            tasks: Mutex::new(HashMap::new()),
        });
        Self(owner)
    }

    /// Returns whether this model still accepts state and task results.
    pub fn is_alive(&self) -> bool {
        !self.0.closed.load(Ordering::Acquire) && self.0.dispatcher.is_alive()
    }

    /// Closes the model even when other clones still exist.
    pub fn close(&self) {
        self.0.close();
    }

    /// Captures result delivery that stops when this model closes.
    pub fn dispatcher(&self) -> crate::UiDispatcher {
        let owner = Arc::downgrade(&self.0);
        self.0.dispatcher.clone().scoped(move || {
            owner
                .upgrade()
                .is_some_and(|owner| !owner.closed.load(Ordering::Acquire))
        })
    }

    /// Gets or initializes reactive state by a key within this model.
    ///
    /// Reads and writes use the same build and layout invalidation mechanisms
    /// as remembered state. The owning UI thread performs initialization.
    pub fn state_with_key<K, T>(&self, key: K, init: impl FnOnce() -> T) -> State<T>
    where
        K: Hash,
        T: 'static,
    {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.0.session),
            "model belongs to a different UI session"
        );
        assert!(self.is_alive(), "model is closed");
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        let key = SlotKey {
            kind: super::SlotKind::State,
            owned_owner: Some(self.0.id),
            instance_logic_id: 0,
            slot_hash: hasher.finish(),
            type_id: TypeId::of::<T>(),
        };
        resolve_state_key(key, true, || {
            let value = init();
            assert!(self.is_alive(), "model closed during initialization");
            value
        })
    }

    /// Launches a task at this call site and retains it until the model closes.
    /// Recomposition with an equal key leaves the existing task unchanged;
    /// changing the key invalidates and cancels its previous task before
    /// launching the new one. Cancellation is best-effort; use the supplied
    /// context's [`ModelTaskContext::update`] to reject results from an old key.
    #[track_caller]
    pub fn launch<K, Factory, F>(
        &self,
        dispatcher: crate::Dispatcher,
        key: K,
        task: Factory,
    ) -> bool
    where
        K: Any + Eq + Send,
        Factory: FnOnce(ModelTaskContext) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.0.session),
            "model tasks must be launched in their owning UI session"
        );
        if !self.is_alive() {
            return false;
        }
        let location = Location::caller();
        let (instance, group_path, ordinal) =
            if super::current_phase() == Some(super::RuntimePhase::Build) {
                (
                    super::current_instance_logic_id_opt().unwrap_or_default(),
                    super::current_group_path_hash(),
                    super::next_order_counter(
                        super::OrderCounterKind::ModelTask,
                        "model tasks require a component build scope",
                    ),
                )
            } else {
                (0, 0, 0)
            };
        let site = TaskSite {
            file: location.file(),
            line: location.line(),
            column: location.column(),
            instance,
            group_path,
            ordinal,
        };
        let mut tasks = self.0.tasks.lock();
        if tasks
            .get(&site)
            .is_some_and(|existing| existing.key.matches(&key))
        {
            return false;
        }
        if !self.is_alive() {
            return false;
        }
        let previous = tasks.remove(&site);
        if let Some(previous) = previous {
            previous.current.store(false, Ordering::Release);
            previous.handle.cancel();
        }
        let current = Arc::new(AtomicBool::new(true));
        let context = ModelTaskContext {
            current: current.clone(),
        };
        let handle = crate::async_dispatch::launch(dispatcher, task(context));
        tasks.insert(
            site,
            ModelTask {
                key: TaskKey::new(key),
                handle,
                current,
            },
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Model, ModelTaskContext};
    use parking_lot::Mutex;
    use std::sync::Arc;

    fn launch(model: &Model, key: u32, contexts: Arc<Mutex<Vec<ModelTaskContext>>>) -> bool {
        model.launch(crate::Dispatcher::Compute, key, move |context| {
            contexts.lock().push(context);
            std::future::pending()
        })
    }

    #[test]
    fn keyed_model_tasks_reuse_equal_keys_cancel_changes_and_stop_on_close() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let model = Model::new();
            let contexts = Arc::new(Mutex::new(Vec::new()));
            assert!(launch(&model, 1, contexts.clone()));
            let first = model
                .0
                .tasks
                .lock()
                .values()
                .next()
                .expect("first task")
                .handle
                .clone();
            let first_context = contexts.lock()[0].clone();
            assert!(!launch(&model, 1, contexts.clone()));
            assert!(!first.is_cancelled());

            assert!(launch(&model, 2, contexts.clone()));
            assert!(first.is_cancelled());
            assert!(!first_context.is_current());
            let second = model
                .0
                .tasks
                .lock()
                .values()
                .next()
                .expect("replacement task")
                .handle
                .clone();
            let second_context = contexts.lock()[1].clone();
            let state = model.state_with_key("result", || 0usize);
            assert!(!first_context.update(state, |value| *value = 1));
            assert!(second_context.update(state, |value| *value = 2));
            assert_eq!(state.get(), 2);
            model.close();
            assert!(second.is_cancelled());
            assert!(!second_context.is_current());
            assert!(!model.is_alive());
        });
    }

    #[test]
    fn model_task_delivers_state_updates_through_its_latest_key_context() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let model = Model::new();
            let state = model.state_with_key("result", || 0usize);
            let (sender, receiver) = std::sync::mpsc::sync_channel(0);
            assert!(model.launch(
                crate::Dispatcher::Compute,
                "request",
                move |task| async move {
                    sender
                        .send(task.update(state, |value| *value = 42))
                        .expect("test receiver remains connected");
                },
            ));
            assert!(receiver.recv().expect("model task ran"));
            assert_eq!(state.get(), 0, "task results are queued for UI delivery");
            crate::dispatcher::flush();
            assert_eq!(state.get(), 42);
            model.close();
        });
    }
}

fn release_slots(owner: u128) {
    let removed = with_slot_table_mut(|table| {
        let slots = table.owned_slots.remove(&owner).unwrap_or_default();
        slots
            .into_iter()
            .filter_map(|slot| table.remove_slot(slot))
            .collect::<Vec<_>>()
    });
    drop(removed);
}
