//! Context provider — share ambient values during component build.
//!
//! ## Usage
//!
//! Provide themes and configuration objects to descendant components.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

use im::HashMap as ImHashMap;
use parking_lot::RwLock;

use crate::runtime::{compute_context_slot_key, ensure_build_phase};

type ContextMap = ImHashMap<TypeId, (u32, u64)>;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextMap>> = RefCell::new(vec![ContextMap::new()]);
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct SlotKey {
    logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

struct SlotEntry {
    key: SlotKey,
    generation: u64,
    value: Option<Arc<dyn Any + Send + Sync>>,
    last_alive_epoch: u64,
}

#[derive(Default)]
struct SlotTable {
    entries: Vec<SlotEntry>,
    free_list: Vec<u32>,
    key_to_slot: HashMap<SlotKey, u32>,
    epoch: u64,
}

impl SlotTable {
    fn begin_frame(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }
}

static SLOT_TABLE: OnceLock<RwLock<SlotTable>> = OnceLock::new();

fn slot_table() -> &'static RwLock<SlotTable> {
    SLOT_TABLE.get_or_init(|| RwLock::new(SlotTable::default()))
}

pub(crate) fn begin_frame_context_slots() {
    slot_table().write().begin_frame();
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(ContextMap::new());
    });
}

pub(crate) fn recycle_frame_context_slots() {
    let mut table = slot_table().write();
    let epoch = table.epoch;

    let mut freed: Vec<(u32, SlotKey)> = Vec::new();
    for (slot, entry) in table.entries.iter_mut().enumerate() {
        if entry.value.is_none() {
            continue;
        }

        if entry.last_alive_epoch == epoch {
            continue;
        }

        freed.push((slot as u32, entry.key));
        entry.value = None;
        entry.generation = entry.generation.wrapping_add(1);
        entry.last_alive_epoch = 0;
    }

    for (slot, key) in freed {
        table.key_to_slot.remove(&key);
        table.free_list.push(slot);
    }
}

/// Handle to a context value created by [`provide_context`] and retrieved by
/// [`use_context`].
///
/// # Examples
///
/// ```
/// use tessera_ui::{provide_context, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     color: String,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         || Theme {
///             color: "blue".into(),
///         },
///         || {
///             child();
///         },
///     );
/// }
///
/// #[tessera]
/// fn child() {
///     let theme = use_context::<Theme>().expect("Theme must be provided");
///     theme.with(|t| println!("Color: {}", t.color));
/// }
/// ```
pub struct Context<T> {
    slot: u32,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Copy for Context<T> {}

impl<T> Clone for Context<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Context<T> {
    fn new(slot: u32, generation: u64) -> Self {
        Self {
            slot,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> Context<T>
where
    T: Send + Sync + 'static,
{
    fn load_entry(&self) -> Arc<dyn Any + Send + Sync> {
        let table = slot_table().read();
        let entry = table
            .entries
            .get(self.slot as usize)
            .unwrap_or_else(|| panic!("Context points to freed slot: {}", self.slot));

        if entry.generation != self.generation {
            panic!(
                "Context is stale (slot {}, generation {}, current generation {})",
                self.slot, self.generation, entry.generation
            );
        }

        if entry.key.type_id != TypeId::of::<T>() {
            panic!(
                "Context type mismatch for slot {}: expected {}, stored {:?}",
                self.slot,
                std::any::type_name::<T>(),
                entry.key.type_id
            );
        }

        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("Context slot {} has been recycled", self.slot))
            .clone()
    }

    fn load_lock(&self) -> Arc<RwLock<T>> {
        self.load_entry()
            .downcast::<RwLock<T>>()
            .unwrap_or_else(|_| panic!("Context slot {} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the context value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let lock = self.load_lock();
        let guard = lock.read();
        f(&guard)
    }

    /// Execute a closure with a mutable reference to the context value.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let lock = self.load_lock();
        let mut guard = lock.write();
        f(&mut guard)
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Set the context value.
    pub fn set(&self, value: T) {
        self.with_mut(|v| *v = value);
    }

    /// Replace the context value and return the old value.
    pub fn replace(&self, value: T) -> T {
        self.with_mut(|v| std::mem::replace(v, value))
    }
}

fn push_context_layer(type_id: TypeId, slot: u32, generation: u64) {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let parent = stack.last().cloned().unwrap_or_else(ContextMap::new);
        let next = parent.update(type_id, (slot, generation));
        stack.push(next);
    });
}

fn pop_context_layer() {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(popped.is_some(), "Context stack underflow");
        if stack.is_empty() {
            stack.push(ContextMap::new());
        }
    });
}

/// Provides a typed context value for the duration of the given closure.
///
/// The `init` closure is evaluated only when the context slot is created (or
/// re-created after being recycled).
///
/// This is intended for use inside component build functions only.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, provide_context, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         || Theme {
///             primary: Color::RED,
///         },
///         || {
///             leaf();
///         },
///     );
/// }
///
/// #[tessera]
/// fn leaf() {
///     let theme = use_context::<Theme>().expect("Theme must be provided");
///     theme.with(|t| assert_eq!(t.primary, Color::RED));
/// }
/// ```
pub fn provide_context<T, I, F, R>(init: I, f: F) -> R
where
    T: Send + Sync + 'static,
    I: FnOnce() -> T,
    F: FnOnce() -> R,
{
    ensure_build_phase();

    let (logic_id, slot_hash) = compute_context_slot_key();
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        logic_id,
        slot_hash,
        type_id,
    };

    let (slot, generation) = {
        let mut table = slot_table().write();
        let epoch = table.epoch;

        if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("context slot entry should exist");
            entry.last_alive_epoch = epoch;

            if entry.value.is_none() {
                entry.value = Some(Arc::new(RwLock::new(init())));
                entry.generation = entry.generation.wrapping_add(1);
            }

            let generation = entry.generation;
            (slot, generation)
        } else if let Some(slot) = table.free_list.pop() {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("context slot entry should exist");

            entry.key = slot_key;
            entry.value = Some(Arc::new(RwLock::new(init())));
            entry.last_alive_epoch = epoch;

            let generation = entry.generation;
            table.key_to_slot.insert(slot_key, slot);
            (slot, generation)
        } else {
            let generation = 0;
            let slot = table.entries.len() as u32;
            table.entries.push(SlotEntry {
                key: slot_key,
                generation,
                value: Some(Arc::new(RwLock::new(init()))),
                last_alive_epoch: epoch,
            });
            table.key_to_slot.insert(slot_key, slot);
            (slot, generation)
        }
    };

    push_context_layer(type_id, slot, generation);

    struct ContextScopeGuard;
    impl Drop for ContextScopeGuard {
        fn drop(&mut self) {
            pop_context_layer();
        }
    }

    let guard = ContextScopeGuard;
    let result = f();
    drop(guard);
    result
}

/// Reads a typed context value from the current scope.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn component() {
///     let theme = use_context::<Theme>();
///     assert!(theme.is_none());
/// }
/// ```
pub fn use_context<T>() -> Option<Context<T>>
where
    T: Send + Sync + 'static,
{
    ensure_build_phase();

    CONTEXT_STACK.with(|stack| {
        let stack = stack.borrow();
        let map = stack
            .last()
            .expect("Context stack must always contain at least one layer");
        map.get(&TypeId::of::<T>())
            .copied()
            .map(|(slot, generation)| Context::new(slot, generation))
    })
}
// (legacy comment removed)
