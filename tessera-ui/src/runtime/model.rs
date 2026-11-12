//! Typed view models with an owned lifetime.
//!
//! ## Usage
//!
//! Inject screen or feature models into components with `Model<T>` parameters.

use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use super::{SlotKey, State, resolve_state_key, with_slot_table_mut};

struct OwnerLease {
    id: u128,
    session: u128,
    closed: Cell<bool>,
}

impl OwnerLease {
    fn new() -> Self {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        Self {
            id: u128::from(NEXT_OWNER.fetch_add(1, Ordering::Relaxed)),
            session: crate::session::current_session_identity()
                .expect("models require an active UI session"),
            closed: Cell::new(false),
        }
    }

    fn close(&self) {
        if self.closed.replace(true) {
            return;
        }
        let owner = self.id;
        if crate::session::current_session_identity() == Some(self.session) {
            release_slots(owner);
        } else {
            // A closed session has already released every slot.
            let _ = crate::session::with_inactive_session(self.session, || release_slots(owner));
        }
    }
}

impl Drop for OwnerLease {
    fn drop(&mut self) {
        self.close();
    }
}

/// Owner of states that outlive a single composition slot.
#[derive(Clone)]
pub(crate) struct ModelScope(Rc<OwnerLease>);

impl ModelScope {
    pub(crate) fn new() -> Self {
        Self(Rc::new(OwnerLease::new()))
    }

    #[cfg(test)]
    pub(crate) fn is_alive(&self) -> bool {
        !self.0.closed.get() && crate::session::is_open(self.0.session)
    }

    #[cfg(test)]
    pub(crate) fn close(&self) {
        self.0.close();
    }

    pub(crate) fn own_state<T: 'static>(&self, value: T) -> State<T> {
        let _guard = ConstructionGuard::enter(self.0.id);
        create_model_state(value)
    }
}

thread_local! {
    static CONSTRUCTION_OWNER: Cell<Option<u128>> = const { Cell::new(None) };
}

struct ConstructionGuard(Option<u128>);

impl ConstructionGuard {
    fn enter(owner: u128) -> Self {
        Self(CONSTRUCTION_OWNER.with(|current| current.replace(Some(owner))))
    }
}

impl Drop for ConstructionGuard {
    fn drop(&mut self) {
        CONSTRUCTION_OWNER.with(|current| current.set(self.0));
    }
}

pub(crate) fn create_model_state<T: 'static>(value: T) -> State<T> {
    static NEXT_STATE: AtomicU64 = AtomicU64::new(1);
    let owner = CONSTRUCTION_OWNER.with(Cell::get).expect(
        "State::new must run while a model is being initialized; use remember for component state",
    );
    let key = SlotKey {
        kind: super::SlotKind::State,
        owned_owner: Some(owner),
        instance_logic_id: 0,
        slot_hash: NEXT_STATE.fetch_add(1, Ordering::Relaxed),
        type_id: TypeId::of::<T>(),
    };
    resolve_state_key(key, true, || value)
}

/// Declares a model type and creates its value.
///
/// States created with [`State::new`] inside `init` belong to the model and are
/// released when it closes.
pub trait ModelInit: 'static {
    /// Creates the model value.
    fn init() -> Self;
}

/// A copyable handle to a model value.
///
/// Declare a `Model<T>` component parameter to receive one. An explicit argument
/// shares an existing model. Otherwise the nearest provided [`ModelStore`] owns
/// a model created by [`ModelInit::init`], or the component owns it until it
/// leaves the composition.
///
/// Access through [`Model::with`] and [`Model::with_mut`] is not tracked: put
/// data the UI reads in `State` fields, which invalidate their readers, and
/// keep other data in ordinary fields changed by `&mut self` methods. Like
/// [`State`], the handle works on every thread and is `Send` and `Sync` when
/// `T: Send`. Asynchronous model operations take the handle, for example
/// `async fn load(model: Model<Self>)`, and access the value between awaits.
///
/// ```
/// use tessera_ui::{ModelInit, ModelStore, RuntimeSession, State};
///
/// struct Counter {
///     count: State<u32>,
///     clicks: u32,
/// }
///
/// impl ModelInit for Counter {
///     fn init() -> Self {
///         Self {
///             count: State::new(0),
///             clicks: 0,
///         }
///     }
/// }
///
/// impl Counter {
///     fn click(&mut self) {
///         self.clicks += 1;
///         self.count.set(self.clicks);
///     }
/// }
///
/// RuntimeSession::new().run(|| {
///     let store = ModelStore::new();
///     let model = store.model(Counter::init);
///     model.with_mut(Counter::click);
///     assert_eq!(model.with(|counter| counter.count.get()), 1);
/// });
/// ```
pub struct Model<T: 'static>(State<T>);

impl<T> Clone for Model<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Model<T> {}

impl<T> PartialEq for Model<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Model<T> {}

impl<T: 'static> Model<T> {
    fn create(owner: &OwnerLease, init: impl FnOnce() -> T) -> Self {
        let _guard = ConstructionGuard::enter(owner.id);
        let value = init();
        Self(create_model_state(value))
    }

    /// Runs `read` with a shared reference to the model value.
    #[track_caller]
    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        self.0.read_untracked(read)
    }

    /// Runs `update` with a mutable reference to the model value.
    #[track_caller]
    pub fn with_mut<R>(&self, update: impl FnOnce(&mut T) -> R) -> R {
        self.0.write_untracked(update)
    }

    /// Returns whether the model is still open.
    pub fn is_alive(&self) -> bool {
        self.0.is_alive()
    }
}

/// Owns models for one navigation entry or other explicit lifetime.
///
/// Provide its [`ModelStoreScope`] as context; components below then receive
/// `Model<T>` parameters from this store instead of owning them. The store
/// holds one model per type and closes them when dropped.
pub struct ModelStore(Rc<StoreInner>);

/// Context value that routes `Model<T>` injection to a [`ModelStore`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ModelStoreScope(u64);

struct StoreInner {
    scope: ModelStoreScope,
    models: RefCell<HashMap<TypeId, StoredModel>>,
}

struct StoredModel {
    model: Box<dyn Any>,
    _owner: OwnerLease,
}

#[derive(Default)]
struct ModelStores(RefCell<HashMap<ModelStoreScope, Weak<StoreInner>>>);

impl Default for ModelStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelStore {
    /// Creates an empty store in the current UI session.
    pub fn new() -> Self {
        static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);
        let inner = Rc::new(StoreInner {
            scope: ModelStoreScope(NEXT_SCOPE.fetch_add(1, Ordering::Relaxed)),
            models: RefCell::default(),
        });
        let stores = crate::local_runtime_resource::<ModelStores>();
        let mut stores = stores.0.borrow_mut();
        stores.retain(|_, store| store.strong_count() > 0);
        stores.insert(inner.scope, Rc::downgrade(&inner));
        drop(stores);
        Self(inner)
    }

    /// Returns the context value that routes injection to this store.
    pub fn scope(&self) -> ModelStoreScope {
        self.0.scope
    }

    /// Returns this store's model of type `T`, creating it with `init` once.
    pub fn model<T: 'static>(&self, init: impl FnOnce() -> T) -> Model<T> {
        self.0.model(init)
    }
}

impl StoreInner {
    fn model<T: 'static>(&self, init: impl FnOnce() -> T) -> Model<T> {
        let existing = self
            .models
            .borrow()
            .get(&TypeId::of::<T>())
            .and_then(|stored| stored.model.downcast_ref::<Model<T>>().copied())
            .filter(Model::is_alive);
        existing.unwrap_or_else(|| {
            let owner = OwnerLease::new();
            let model = Model::create(&owner, init);
            let previous = self.models.borrow_mut().insert(
                TypeId::of::<T>(),
                StoredModel {
                    model: Box::new(model),
                    _owner: owner,
                },
            );
            drop(previous);
            model
        })
    }
}

impl Drop for ModelStore {
    fn drop(&mut self) {
        let models = std::mem::take(&mut *self.0.models.borrow_mut());
        drop(models);
    }
}

/// Resolves a `Model<T>` component parameter that the caller did not supply.
///
/// The nearest provided [`ModelStoreScope`] owns the model; without one the
/// component's own slot does.
pub(crate) fn inject_model<T: ModelInit>(slot: u64) -> Model<T> {
    // A fixed group keeps the fallback slot independent of whether callers
    // override the model on a given frame.
    let _group = super::GroupGuard::new(u64::MAX - slot);
    let scope = crate::use_context::<ModelStoreScope>().map(|scope| scope.get());
    // The slot caches the resolved model, so stable frames skip the store
    // lookup and only compare the provided scope.
    let resolved = super::remember(|| None::<ResolvedModel<T>>);
    if let Some(model) = resolved.with_untracked(|resolved| {
        resolved
            .as_ref()
            .filter(|resolved| resolved.scope == scope && resolved.model.is_alive())
            .map(|resolved| resolved.model)
    }) {
        return model;
    }
    let store = scope.and_then(|scope| {
        crate::local_runtime_resource::<ModelStores>()
            .0
            .borrow()
            .get(&scope)
            .and_then(Weak::upgrade)
    });
    let (model, owner) = match store {
        Some(store) => (store.model(T::init), None),
        None => {
            let owner = OwnerLease::new();
            (Model::create(&owner, T::init), Some(owner))
        }
    };
    let previous = resolved.with_mut(|resolved| {
        resolved.replace(ResolvedModel {
            scope,
            model,
            _owner: owner,
        })
    });
    drop(previous);
    model
}

struct ResolvedModel<T: 'static> {
    scope: Option<ModelStoreScope>,
    model: Model<T>,
    // Set when the component owns its fallback model.
    _owner: Option<OwnerLease>,
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

#[cfg(test)]
mod tests {
    use super::{ModelInit, ModelStore};
    use crate::State;

    struct Counter {
        value: State<usize>,
    }

    impl ModelInit for Counter {
        fn init() -> Self {
            Self {
                value: State::new(0),
            }
        }
    }

    #[test]
    fn states_created_during_initialization_close_with_the_store() {
        crate::RuntimeSession::new().run(|| {
            let store = ModelStore::new();
            let other = ModelStore::new();
            let model = store.model(Counter::init);
            let value = model.with(|model| model.value);
            let other_value = other.model(Counter::init).with(|model| model.value);
            value.set(3);
            drop(store);
            assert!(!model.is_alive());
            assert!(!value.is_alive());
            assert!(other_value.is_alive());
        });
    }

    #[test]
    fn state_new_outside_model_initialization_is_rejected() {
        crate::RuntimeSession::new().run(|| {
            let result = std::panic::catch_unwind(|| State::new(1u8));
            assert!(result.is_err());
        });
    }
}
