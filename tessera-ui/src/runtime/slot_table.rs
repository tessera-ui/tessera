use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
};

use parking_lot::RwLock;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use super::{
    build_scope::{
        compute_functor_slot_key, compute_slot_key, current_component_instance_key_from_scope,
        ensure_build_phase,
    },
    composition::{
        record_component_invalidation_for_instance_key, render_slot_read_subscribers,
        state_read_subscribers, track_state_read_dependency,
    },
    session::with_composition_runtime,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) struct SlotKey {
    pub(crate) instance_logic_id: u64,
    pub(crate) slot_hash: u64,
    pub(crate) type_id: TypeId,
}

impl Default for SlotKey {
    fn default() -> Self {
        Self {
            instance_logic_id: 0,
            slot_hash: 0,
            type_id: TypeId::of::<()>(),
        }
    }
}

new_key_type! {
    pub(crate) struct SlotHandle;
}

#[derive(Default)]
pub(crate) struct SlotEntry {
    pub(crate) key: SlotKey,
    pub(crate) generation: u64,
    pub(crate) value: Option<Arc<dyn Any + Send + Sync>>,
    pub(crate) last_alive_epoch: u64,
    pub(crate) retained: bool,
}

#[derive(Default)]
struct InstanceSlotCursor {
    previous_order: SmallVec<[SlotHandle; 4]>,
    current_order: SmallVec<[SlotHandle; 4]>,
    cursor: usize,
    epoch: u64,
}

impl InstanceSlotCursor {
    fn begin_epoch(&mut self, epoch: u64) {
        if self.epoch == epoch {
            return;
        }
        self.previous_order = std::mem::take(&mut self.current_order);
        self.cursor = 0;
        self.epoch = epoch;
    }

    fn fast_candidate(&self) -> Option<SlotHandle> {
        self.previous_order.get(self.cursor).copied()
    }

    fn record_fast_match(&mut self, slot: SlotHandle) {
        self.cursor = self.cursor.saturating_add(1);
        self.current_order.push(slot);
    }

    fn record_slow_match(&mut self, slot: SlotHandle) {
        if self.cursor < self.previous_order.len()
            && let Some(offset) = self.previous_order[self.cursor..]
                .iter()
                .position(|candidate| *candidate == slot)
        {
            self.cursor += offset + 1;
        }
        self.current_order.push(slot);
    }
}

#[derive(Default)]
pub(crate) struct SlotTable {
    pub(crate) entries: SlotMap<SlotHandle, SlotEntry>,
    pub(crate) key_to_slot: HashMap<SlotKey, SlotHandle>,
    cursors_by_instance_logic_id: HashMap<u64, InstanceSlotCursor>,
    pub(crate) epoch: u64,
}

impl SlotTable {
    fn begin_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }

    fn reset(&mut self) {
        self.entries.clear();
        self.key_to_slot.clear();
        self.cursors_by_instance_logic_id.clear();
        self.epoch = 0;
    }

    fn try_fast_slot_lookup(&mut self, key: SlotKey) -> Option<SlotHandle> {
        let epoch = self.epoch;
        let candidate = {
            let cursor = self
                .cursors_by_instance_logic_id
                .entry(key.instance_logic_id)
                .or_default();
            cursor.begin_epoch(epoch);
            cursor.fast_candidate()
        }?;

        let is_match = self
            .entries
            .get(candidate)
            .is_some_and(|entry| entry.key == key);

        if !is_match {
            return None;
        }

        let cursor = self
            .cursors_by_instance_logic_id
            .get_mut(&key.instance_logic_id)
            .expect("cursor entry should exist");
        cursor.record_fast_match(candidate);
        Some(candidate)
    }

    fn record_slot_usage_slow(&mut self, instance_logic_id: u64, slot: SlotHandle) {
        let epoch = self.epoch;
        let cursor = self
            .cursors_by_instance_logic_id
            .entry(instance_logic_id)
            .or_default();
        cursor.begin_epoch(epoch);
        cursor.record_slow_match(slot);
    }
}

pub(crate) fn slot_table() -> Arc<RwLock<SlotTable>> {
    with_composition_runtime(|runtime| runtime.slot_table())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FunctorHandle {
    slot: SlotHandle,
    generation: u64,
}

impl FunctorHandle {
    fn new(slot: SlotHandle, generation: u64) -> Self {
        Self { slot, generation }
    }
}

struct CallbackCell {
    shared: RwLock<Arc<dyn Fn() + Send + Sync>>,
}

impl CallbackCell {
    fn new(initial: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            shared: RwLock::new(initial),
        }
    }

    fn update(&self, next: Arc<dyn Fn() + Send + Sync>) {
        *self.shared.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn() + Send + Sync> {
        self.shared.read().clone()
    }
}

struct CallbackWithCell<T, R> {
    shared: RwLock<Arc<dyn Fn(T) -> R + Send + Sync>>,
}

impl<T, R> CallbackWithCell<T, R> {
    fn new(initial: Arc<dyn Fn(T) -> R + Send + Sync>) -> Self {
        Self {
            shared: RwLock::new(initial),
        }
    }

    fn update(&self, next: Arc<dyn Fn(T) -> R + Send + Sync>) {
        *self.shared.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn(T) -> R + Send + Sync> {
        self.shared.read().clone()
    }
}

struct RenderSlotCell {
    shared: RwLock<Arc<dyn Fn() + Send + Sync>>,
}

impl RenderSlotCell {
    fn new(initial: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            shared: RwLock::new(initial),
        }
    }

    fn update(&self, next: Arc<dyn Fn() + Send + Sync>) {
        *self.shared.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn() + Send + Sync> {
        self.shared.read().clone()
    }
}

struct RenderSlotWithCell<T, R> {
    shared: RwLock<Arc<dyn Fn(T) -> R + Send + Sync>>,
}

impl<T, R> RenderSlotWithCell<T, R> {
    fn new(initial: Arc<dyn Fn(T) -> R + Send + Sync>) -> Self {
        Self {
            shared: RwLock::new(initial),
        }
    }

    fn update(&self, next: Arc<dyn Fn(T) -> R + Send + Sync>) {
        *self.shared.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn(T) -> R + Send + Sync> {
        self.shared.read().clone()
    }
}

/// Handle to memoized state created by [`remember`] and [`remember_with_key`].
///
/// `State<T>` is `Copy + Send + Sync` and provides `with`, `with_mut`, `get`,
/// `set`, and `cloned` to read or update the stored value.
///
/// Handles are validated with a slot generation token so stale references fail
/// fast if their slot has been recycled.
///
/// # Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn counter() {
///     let count = remember(|| 0usize);
///     count.with_mut(|c| *c += 1);
///     let current = count.get();
///     assert!(current >= 1);
/// }
/// ```
pub struct State<T> {
    slot: SlotHandle,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Copy for State<T> {}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.generation == other.generation
    }
}

impl<T> Eq for State<T> {}

impl<T> Hash for State<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.slot.hash(state);
        self.generation.hash(state);
    }
}

impl<T> State<T> {
    fn new(slot: SlotHandle, generation: u64) -> Self {
        Self {
            slot,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> State<T>
where
    T: Send + Sync + 'static,
{
    pub(crate) fn is_alive(&self) -> bool {
        let slot_table = slot_table();
        let table = slot_table.read();
        let Some(entry) = table.entries.get(self.slot) else {
            return false;
        };

        entry.generation == self.generation
            && entry.key.type_id == TypeId::of::<T>()
            && entry.value.is_some()
    }

    fn load_entry(&self) -> Arc<dyn Any + Send + Sync> {
        let slot_table = slot_table();
        let table = slot_table.read();
        let entry = table
            .entries
            .get(self.slot)
            .unwrap_or_else(|| panic!("State points to freed slot: {:?}", self.slot));

        if entry.generation != self.generation {
            panic!(
                "State is stale (slot {:?}, generation {}, current generation {})",
                self.slot, self.generation, entry.generation
            );
        }

        if entry.key.type_id != TypeId::of::<T>() {
            panic!(
                "State type mismatch for slot {:?}: expected {}, stored {:?}",
                self.slot,
                std::any::type_name::<T>(),
                entry.key.type_id
            );
        }

        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("State slot {:?} has been cleared", self.slot))
            .clone()
    }

    fn load_lock(&self) -> Arc<RwLock<T>> {
        self.load_entry()
            .downcast::<RwLock<T>>()
            .unwrap_or_else(|_| panic!("State slot {:?} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the stored value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        track_state_read_dependency(self.slot, self.generation);
        let lock = self.load_lock();
        let guard = lock.read();
        f(&guard)
    }

    /// Execute a closure with a mutable reference to the stored value.
    #[track_caller]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let lock = self.load_lock();

        let result = {
            let mut guard = lock.write();
            f(&mut guard)
        };

        let subscribers = state_read_subscribers(self.slot, self.generation);
        for instance_key in subscribers {
            record_component_invalidation_for_instance_key(instance_key);
        }
        result
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Replace the stored value.
    #[track_caller]
    pub fn set(&self, value: T) {
        self.with_mut(|slot| *slot = value);
    }
}

pub fn begin_recompose_slot_epoch() {
    let slot_table = slot_table();
    slot_table.write().begin_epoch();
}

pub fn reset_slots() {
    let slot_table = slot_table();
    slot_table.write().reset();
}

pub(crate) fn recycle_recomposed_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let slot_table = slot_table();
    let mut table = slot_table.write();
    let epoch = table.epoch;
    let mut freed: Vec<(SlotHandle, SlotKey)> = Vec::new();

    for (slot, entry) in table.entries.iter() {
        if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
            continue;
        }
        if entry.last_alive_epoch == epoch || entry.retained {
            continue;
        }
        freed.push((slot, entry.key));
    }

    for (slot, key) in freed {
        table.entries.remove(slot);
        table.key_to_slot.remove(&key);
    }
}

pub(crate) fn live_slot_instance_logic_ids() -> HashSet<u64> {
    let slot_table = slot_table();
    let table = slot_table.read();
    table
        .entries
        .iter()
        .map(|(_, entry)| entry.key.instance_logic_id)
        .collect()
}

pub(crate) fn drop_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let slot_table = slot_table();
    let mut table = slot_table.write();
    let mut freed: Vec<(SlotHandle, SlotKey)> = Vec::new();
    for (slot, entry) in table.entries.iter() {
        if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
            continue;
        }
        if entry.retained {
            continue;
        }
        freed.push((slot, entry.key));
    }
    for (slot, key) in freed {
        table.entries.remove(slot);
        table.key_to_slot.remove(&key);
    }
    for instance_logic_id in instance_logic_ids {
        table.cursors_by_instance_logic_id.remove(instance_logic_id);
    }
}

fn remember_functor_cell_with_key<K, T, F>(key: K, init: F) -> (Arc<T>, FunctorHandle)
where
    K: Hash,
    T: Send + Sync + 'static,
    F: FnOnce() -> T,
{
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_functor_slot_key(&key);
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id: TypeId::of::<T>(),
    };

    let slot_table = slot_table();
    let mut table = slot_table.write();
    let mut init_opt = Some(init);
    if let Some(slot) = table.try_fast_slot_lookup(slot_key) {
        let epoch = table.epoch;
        let (generation, value) = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("functor slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "callback slot type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("callback slot init called more than once");
                entry.value = Some(Arc::new(init_fn()));
                entry.generation = entry.generation.wrapping_add(1);
            }

            (
                entry.generation,
                entry
                    .value
                    .as_ref()
                    .expect("callback slot must contain a value")
                    .clone(),
            )
        };

        (
            value
                .downcast::<T>()
                .unwrap_or_else(|_| panic!("callback slot {:?} downcast failed", slot)),
            FunctorHandle::new(slot, generation),
        )
    } else if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
        table.record_slot_usage_slow(instance_logic_id, slot);
        let epoch = table.epoch;
        let (generation, value) = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("functor slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "callback slot type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("callback slot init called more than once");
                entry.value = Some(Arc::new(init_fn()));
                entry.generation = entry.generation.wrapping_add(1);
            }

            (
                entry.generation,
                entry
                    .value
                    .as_ref()
                    .expect("callback slot must contain a value")
                    .clone(),
            )
        };

        (
            value
                .downcast::<T>()
                .unwrap_or_else(|_| panic!("callback slot {:?} downcast failed", slot)),
            FunctorHandle::new(slot, generation),
        )
    } else {
        let epoch = table.epoch;
        let init_fn = init_opt
            .take()
            .expect("callback slot init called more than once");
        let generation = 1u64;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(Arc::new(init_fn())),
            last_alive_epoch: epoch,
            retained: false,
        });

        table.key_to_slot.insert(slot_key, slot);
        table.record_slot_usage_slow(instance_logic_id, slot);

        let value = table
            .entries
            .get(slot)
            .expect("functor slot entry should exist")
            .value
            .as_ref()
            .expect("callback slot must contain a value")
            .clone()
            .downcast::<T>()
            .unwrap_or_else(|_| panic!("callback slot {:?} downcast failed", slot));

        (value, FunctorHandle::new(slot, generation))
    }
}

fn load_functor_cell<T>(handle: FunctorHandle) -> Arc<T>
where
    T: Send + Sync + 'static,
{
    let slot_table = slot_table();
    let table = slot_table.read();
    let entry = table
        .entries
        .get(handle.slot)
        .unwrap_or_else(|| panic!("Callback points to freed slot: {:?}", handle.slot));

    if entry.generation != handle.generation {
        panic!(
            "Callback is stale (slot {:?}, generation {}, current generation {})",
            handle.slot, handle.generation, entry.generation
        );
    }

    if entry.key.type_id != TypeId::of::<T>() {
        panic!(
            "Callback type mismatch for slot {:?}: expected {}, stored {:?}",
            handle.slot,
            std::any::type_name::<T>(),
            entry.key.type_id
        );
    }

    entry
        .value
        .as_ref()
        .unwrap_or_else(|| panic!("Callback slot {:?} has been cleared", handle.slot))
        .clone()
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("Callback slot {:?} downcast failed", handle.slot))
}

pub(crate) fn remember_callback_handle<F>(handler: F) -> FunctorHandle
where
    F: Fn() + Send + Sync + 'static,
{
    let handler = Arc::new(handler) as Arc<dyn Fn() + Send + Sync>;
    let (cell, handle) = remember_functor_cell_with_key((), {
        let handler = Arc::clone(&handler);
        move || CallbackCell::new(handler)
    });
    cell.update(handler);
    handle
}

pub(crate) fn invoke_callback_handle(handle: FunctorHandle) {
    let callback = load_functor_cell::<CallbackCell>(handle).shared();
    callback();
}

pub(crate) fn remember_render_slot_handle<F>(render: F) -> FunctorHandle
where
    F: Fn() + Send + Sync + 'static,
{
    let render = Arc::new(render) as Arc<dyn Fn() + Send + Sync>;
    let creator_instance_key = current_component_instance_key_from_scope()
        .unwrap_or_else(|| panic!("RenderSlot handles must be created during a component build"));
    let (cell, handle) = remember_functor_cell_with_key((), {
        let render = Arc::clone(&render);
        move || RenderSlotCell::new(render)
    });
    cell.update(render);
    for instance_key in render_slot_read_subscribers(handle) {
        if instance_key != creator_instance_key {
            record_component_invalidation_for_instance_key(instance_key);
        }
    }
    handle
}

pub(crate) fn invoke_render_slot_handle(handle: FunctorHandle) {
    let render = load_functor_cell::<RenderSlotCell>(handle).shared();
    render();
}

pub(crate) fn remember_render_slot_with_handle<T, R, F>(render: F) -> FunctorHandle
where
    T: 'static,
    R: 'static,
    F: Fn(T) -> R + Send + Sync + 'static,
{
    let render = Arc::new(render) as Arc<dyn Fn(T) -> R + Send + Sync>;
    let creator_instance_key = current_component_instance_key_from_scope().unwrap_or_else(|| {
        panic!("RenderSlotWith handles must be created during a component build")
    });
    let (cell, handle) = remember_functor_cell_with_key((), {
        let render = Arc::clone(&render);
        move || RenderSlotWithCell::new(render)
    });
    cell.update(render);
    for instance_key in render_slot_read_subscribers(handle) {
        if instance_key != creator_instance_key {
            record_component_invalidation_for_instance_key(instance_key);
        }
    }
    handle
}

pub(crate) fn invoke_render_slot_with_handle<T, R>(handle: FunctorHandle, value: T) -> R
where
    T: 'static,
    R: 'static,
{
    let render = load_functor_cell::<RenderSlotWithCell<T, R>>(handle).shared();
    render(value)
}

pub(crate) fn remember_callback_with_handle<T, R, F>(handler: F) -> FunctorHandle
where
    T: 'static,
    R: 'static,
    F: Fn(T) -> R + Send + Sync + 'static,
{
    let handler = Arc::new(handler) as Arc<dyn Fn(T) -> R + Send + Sync>;
    let (cell, handle) = remember_functor_cell_with_key((), {
        let handler = Arc::clone(&handler);
        move || CallbackWithCell::new(handler)
    });
    cell.update(handler);
    handle
}

pub(crate) fn invoke_callback_with_handle<T, R>(handle: FunctorHandle, value: T) -> R
where
    T: 'static,
    R: 'static,
{
    let callback = load_functor_cell::<CallbackWithCell<T, R>>(handle).shared();
    callback(value)
}

/// Remember a value across frames with an explicit key.
pub fn remember_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_slot_key(&key);
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id,
    };

    let slot_table = slot_table();
    let mut table = slot_table.write();
    let mut init_opt = Some(init);
    if let Some(slot) = table.try_fast_slot_lookup(slot_key) {
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "remember_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("remember_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }
            entry.generation
        };

        State::new(slot, generation)
    } else if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
        table.record_slot_usage_slow(instance_logic_id, slot);
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "remember_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("remember_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }
            entry.generation
        };

        State::new(slot, generation)
    } else {
        let epoch = table.epoch;
        let init_fn = init_opt
            .take()
            .expect("remember_with_key init called more than once");
        let generation = 1u64;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(Arc::new(RwLock::new(init_fn()))),
            last_alive_epoch: epoch,
            retained: false,
        });

        table.key_to_slot.insert(slot_key, slot);
        table.record_slot_usage_slow(instance_logic_id, slot);
        State::new(slot, generation)
    }
}

/// Remember a value across recomposition (build) passes.
pub fn remember<F, T>(init: F) -> State<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    remember_with_key((), init)
}

/// Retain a value across recomposition (build) passes with an explicit key,
/// even if unused.
pub fn retain_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_slot_key(&key);
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id,
    };

    let slot_table = slot_table();
    let mut table = slot_table.write();
    let mut init_opt = Some(init);
    if let Some(slot) = table.try_fast_slot_lookup(slot_key) {
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "retain_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            entry.retained = true;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("retain_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }

            entry.generation
        };

        State::new(slot, generation)
    } else if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
        table.record_slot_usage_slow(instance_logic_id, slot);
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot)
                .expect("slot entry should exist");

            if entry.key.type_id != slot_key.type_id {
                panic!(
                    "retain_with_key type mismatch: expected {}, found {:?}",
                    std::any::type_name::<T>(),
                    entry.key.type_id
                );
            }

            entry.last_alive_epoch = epoch;
            entry.retained = true;
            if entry.value.is_none() {
                let init_fn = init_opt
                    .take()
                    .expect("retain_with_key init called more than once");
                entry.value = Some(Arc::new(RwLock::new(init_fn())));
                entry.generation = entry.generation.wrapping_add(1);
            }

            entry.generation
        };

        State::new(slot, generation)
    } else {
        let epoch = table.epoch;
        let init_fn = init_opt
            .take()
            .expect("retain_with_key init called more than once");
        let generation = 1u64;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(Arc::new(RwLock::new(init_fn()))),
            last_alive_epoch: epoch,
            retained: true,
        });

        table.key_to_slot.insert(slot_key, slot);
        table.record_slot_usage_slow(instance_logic_id, slot);
        State::new(slot, generation)
    }
}

/// Retain a value across recomposition (build) passes, even if unused.
pub fn retain<F, T>(init: F) -> State<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    retain_with_key((), init)
}
