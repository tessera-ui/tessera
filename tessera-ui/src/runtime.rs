//! This module provides the global runtime state management for tessera.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

use parking_lot::RwLock;

use crate::{NodeId, component_tree::ComponentTree};

thread_local! {
    /// Stack of currently executing component node ids for the current thread.
    static NODE_CONTEXT_STACK: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    /// Control-flow grouping path for the current thread.
    static GROUP_PATH_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// Component logic identifier stack (one per component invocation).
    static LOGIC_ID_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// Current execution phase stack for the thread.
    static PHASE_STACK: RefCell<Vec<RuntimePhase>> = const { RefCell::new(Vec::new()) };
    /// Call counter stack: tracks sequential remember calls within each group.
    /// Each entry corresponds to a group depth level.
    static CALL_COUNTER_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct SlotKey {
    logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

impl Default for SlotKey {
    fn default() -> Self {
        Self {
            logic_id: 0,
            slot_hash: 0,
            type_id: TypeId::of::<()>(),
        }
    }
}

#[derive(Default)]
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

    fn reset(&mut self) {
        self.entries.clear();
        self.free_list.clear();
        self.key_to_slot.clear();
        self.epoch = 0;
    }
}

static SLOT_TABLE: OnceLock<RwLock<SlotTable>> = OnceLock::new();

fn slot_table() -> &'static RwLock<SlotTable> {
    SLOT_TABLE.get_or_init(|| RwLock::new(SlotTable::default()))
}

/// Handle to memoized state created by [`remember`] and [`remember_with_key`].
///
/// `State<T>` is `Copy + Send + Sync` and provides `with`, `with_mut`, `get`,
/// `set`, and `cloned` to read or update the stored value.
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
    slot: u32,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Copy for State<T> {}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> State<T> {
    fn new(slot: u32, generation: u64) -> Self {
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
    fn load_entry(&self) -> Arc<dyn Any + Send + Sync> {
        let table = slot_table().read();
        let entry = table
            .entries
            .get(self.slot as usize)
            .unwrap_or_else(|| panic!("State points to freed slot: {}", self.slot));

        if entry.generation != self.generation {
            panic!(
                "State is stale (slot {}, generation {}, current generation {})",
                self.slot, self.generation, entry.generation
            );
        }

        if entry.key.type_id != TypeId::of::<T>() {
            panic!(
                "State type mismatch for slot {}: expected {}, stored {:?}",
                self.slot,
                std::any::type_name::<T>(),
                entry.key.type_id
            );
        }

        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("State slot {} has been cleared", self.slot))
            .clone()
    }

    fn load_lock(&self) -> Arc<RwLock<T>> {
        self.load_entry()
            .downcast::<RwLock<T>>()
            .unwrap_or_else(|_| panic!("State slot {} downcast failed", self.slot))
    }

    /// Execute a closure with a shared reference to the stored value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let lock = self.load_lock();
        let guard = lock.read();
        f(&guard)
    }

    /// Execute a closure with a mutable reference to the stored value.
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

    /// Replace the stored value.
    pub fn set(&self, value: T) {
        self.with_mut(|slot| *slot = value);
    }
}

/// Global singleton instance of the [`TesseraRuntime`].
static TESSERA_RUNTIME: OnceLock<RwLock<TesseraRuntime>> = OnceLock::new();

/// Runtime state container.
#[derive(Default)]
pub struct TesseraRuntime {
    /// Hierarchical structure of all UI components in the application.
    pub component_tree: ComponentTree,
    /// Current window dimensions in physical pixels.
    pub(crate) window_size: [u32; 2],
    /// Cursor icon change request from UI components.
    pub cursor_icon_request: Option<winit::window::CursorIcon>,
    /// Called when the window minimize state changes.
    on_minimize_callbacks: Vec<Box<dyn Fn(bool) + Send + Sync>>,
    /// Called when the window close event is triggered.
    on_close_callbacks: Vec<Box<dyn Fn() + Send + Sync>>,
    /// Whether the window is currently minimized.
    pub(crate) window_minimized: bool,
}

impl TesseraRuntime {
    /// Executes a closure with a shared, read-only reference to the runtime.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        f(&TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .read())
    }

    /// Executes a closure with an exclusive, mutable reference to the runtime.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        f(&mut TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .write())
    }

    /// Get the current window size in physical pixels.
    pub fn window_size(&self) -> [u32; 2] {
        self.window_size
    }

    /// Registers a per-frame callback for minimize state changes.
    /// Components should call this every frame they wish to be notified.
    pub fn on_minimize(&mut self, callback: impl Fn(bool) + Send + Sync + 'static) {
        self.on_minimize_callbacks.push(Box::new(callback));
    }

    /// Registers a per-frame callback for window close event.
    /// Components should call this every frame they wish to be notified.
    pub fn on_close(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.on_close_callbacks.push(Box::new(callback));
    }

    /// Clears all per-frame registered callbacks.
    /// Must be called by the event loop at the beginning of each frame.
    pub fn clear_frame_callbacks(&mut self) {
        self.on_minimize_callbacks.clear();
        self.on_close_callbacks.clear();
    }

    /// Triggers all registered callbacks (global and per-frame).
    /// Called by the event loop when a minimize event is detected.
    pub fn trigger_minimize_callbacks(&self, minimized: bool) {
        for callback in &self.on_minimize_callbacks {
            callback(minimized);
        }
    }

    /// Triggers all registered callbacks (global and per-frame) for window
    /// close event. Called by the event loop when a close event is
    /// detected.
    pub fn trigger_close_callbacks(&self) {
        for callback in &self.on_close_callbacks {
            callback();
        }
    }
}

/// Guard that records the current component node id for the calling thread.
/// Nested components push their id and pop on drop, forming a stack.
pub struct NodeContextGuard {
    popped: bool,
    logic_id_popped: bool,
}

/// Execution phase for `remember` usage checks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuntimePhase {
    /// Component render/build phase (allowed for `remember`).
    Build,
    /// Measurement phase (disallowed for `remember`).
    Measure,
    /// Input handling phase (disallowed for `remember`).
    Input,
}

/// Guard for execution phase stack.
pub struct PhaseGuard {
    popped: bool,
}

impl PhaseGuard {
    /// Pop the current phase immediately.
    pub fn pop(mut self) {
        if !self.popped {
            pop_phase();
            self.popped = true;
        }
    }
}

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_phase();
            self.popped = true;
        }
    }
}

impl NodeContextGuard {
    /// Pop the current node id immediately. Usually you rely on `Drop` instead.
    pub fn pop(mut self) {
        if !self.popped {
            pop_current_node();
            self.popped = true;
        }
        if !self.logic_id_popped {
            pop_logic_id();
            self.logic_id_popped = true;
        }
    }
}

impl Drop for NodeContextGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_current_node();
            self.popped = true;
        }
        if !self.logic_id_popped {
            pop_logic_id();
            self.logic_id_popped = true;
        }
    }
}

/// Push the given node id as the current executing component for this thread.
pub fn push_current_node(node_id: NodeId, base_logic_id: u64) -> NodeContextGuard {
    NODE_CONTEXT_STACK.with(|stack| stack.borrow_mut().push(node_id));

    // Get the parent's call index and increment it
    // This distinguishes multiple calls to the same component (e.g., foo(1);
    // foo(2);)
    let (parent_call_index, parent_logic_id) = CALL_COUNTER_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let index = stack.last().copied().unwrap_or(0);
        if let Some(last) = stack.last_mut() {
            *last += 1;
        }
        let parent_id = LOGIC_ID_STACK.with(|s| s.borrow().last().copied().unwrap_or(0));
        (index, parent_id)
    });

    // Combine base_logic_id with parent_logic_id and parent_call_index to create a
    // unique instance ID This ensures:
    // 1. foo(1) and foo(2) get different logic_ids (via parent_call_index)
    // 2. Components in different container instances get different logic_ids (via
    //    parent_logic_id)
    let instance_logic_id = if parent_call_index == 0 && parent_logic_id == 0 {
        base_logic_id
    } else {
        hash_components(&[&base_logic_id, &parent_logic_id, &parent_call_index])
    };

    LOGIC_ID_STACK.with(|stack| stack.borrow_mut().push(instance_logic_id));

    // Push a new call counter layer for this component's internal remember calls
    CALL_COUNTER_STACK.with(|stack| stack.borrow_mut().push(0));

    NodeContextGuard {
        popped: false,
        logic_id_popped: false,
    }
}

fn pop_current_node() {
    NODE_CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current node from an empty stack"
        );
    });
    // Pop this component's call counter layer
    CALL_COUNTER_STACK.with(|stack| {
        let popped = stack.borrow_mut().pop();
        debug_assert!(
            popped.is_some(),
            "CALL_COUNTER_STACK underflow: attempted to pop from empty stack"
        );
    });
}

/// Get the node id at the top of the thread-local component stack.
pub fn current_node_id() -> Option<NodeId> {
    NODE_CONTEXT_STACK.with(|stack| stack.borrow().last().copied())
}

fn current_logic_id() -> Option<u64> {
    LOGIC_ID_STACK.with(|stack| stack.borrow().last().copied())
}

fn pop_logic_id() {
    LOGIC_ID_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let _ = stack.pop();
    });
}

/// Push an execution phase for the current thread.
pub fn push_phase(phase: RuntimePhase) -> PhaseGuard {
    PHASE_STACK.with(|stack| stack.borrow_mut().push(phase));
    PhaseGuard { popped: false }
}

fn pop_phase() {
    PHASE_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop execution phase from an empty stack"
        );
    });
}

pub(crate) fn current_phase() -> Option<RuntimePhase> {
    PHASE_STACK.with(|stack| stack.borrow().last().copied())
}

/// Push a group id onto the thread-local control-flow stack.
pub(crate) fn push_group_id(group_id: u64) {
    GROUP_PATH_STACK.with(|stack| stack.borrow_mut().push(group_id));
}

/// Pop a group id from the thread-local control-flow stack.
pub(crate) fn pop_group_id(expected_group_id: u64) {
    GROUP_PATH_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if let Some(popped) = stack.pop() {
            debug_assert_eq!(
                popped, expected_group_id,
                "Unbalanced GroupGuard stack: expected {}, got {}",
                expected_group_id, popped
            );
        } else {
            debug_assert!(false, "Attempted to pop GroupGuard from an empty stack");
        }
    });
}

/// Get a clone of the current control-flow path.
fn current_group_path() -> Vec<u64> {
    GROUP_PATH_STACK.with(|stack| stack.borrow().clone())
}

/// RAII guard that tracks control-flow grouping for the current component node.
///
/// A guard pushes the provided group id when constructed and pops it when
/// dropped, ensuring grouping stays balanced even with early returns or panics.
pub struct GroupGuard {
    group_id: u64,
}

impl GroupGuard {
    /// Push a group id onto the current component's group stack.
    pub fn new(group_id: u64) -> Self {
        push_group_id(group_id);
        Self { group_id }
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        pop_group_id(self.group_id);
    }
}

fn hash_components<H: Hash>(parts: &[&H]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}

fn compute_slot_key<K: Hash>(key: &K) -> (u64, u64) {
    let logic_id = current_logic_id().expect("remember must be called inside a tessera component");
    let group_path = current_group_path();
    let group_path_hash = hash_components(&[&group_path]);
    let key_hash = hash_components(&[key]);

    // Get the call counter to distinguish multiple remember calls within the same
    // component Note: logic_id already distinguishes different component
    // instances (foo(1) vs foo(2)) and group_path_hash handles nested control
    // flow (if/loop)
    let call_counter = CALL_COUNTER_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        debug_assert!(
            !stack.is_empty(),
            "CALL_COUNTER_STACK is empty; remember must be called inside a component"
        );
        let counter = *stack
            .last()
            .expect("CALL_COUNTER_STACK should not be empty");
        *stack
            .last_mut()
            .expect("CALL_COUNTER_STACK should not be empty") += 1;
        counter
    });

    let slot_hash = hash_components(&[&group_path_hash, &key_hash, &call_counter]);
    (logic_id, slot_hash)
}

pub(crate) fn ensure_build_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) => {}
        Some(RuntimePhase::Measure) => {
            panic!("remember must not be called inside measure; move state to component render")
        }
        Some(RuntimePhase::Input) => {
            panic!(
                "remember must not be called inside input_handler; move state to component render"
            )
        }
        None => panic!(
            "remember must be called inside a tessera component. Ensure you're calling this from within a function annotated with #[tessera]."
        ),
    }
}

/// Swap slot buffers at the beginning of a frame.
pub fn begin_frame_slots() {
    slot_table().write().begin_frame();
}

/// Reset all slot buffers (used on suspension).
pub fn reset_slots() {
    slot_table().write().reset();
}

/// Recycle state slots that were not touched in the current frame.
pub fn recycle_frame_slots() {
    let mut table = slot_table().write();
    let epoch = table.epoch;
    let mut freed: Vec<(u32, SlotKey)> = Vec::new();

    for (slot, entry) in table.entries.iter_mut().enumerate() {
        if entry.last_alive_epoch == epoch || entry.value.is_none() {
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

/// Remember a value across frames with an explicit key.
///
/// This function allows a component to "remember" state between frames, using
/// a user-provided key to identify the state. This is particularly useful for
/// state generated inside loops or dynamic collections where the execution
/// order might change.
///
/// The `init` closure is executed only once — when the key is first
/// encountered. On subsequent updates with the same key, the stored value is
/// returned and `init` is not called.
///
/// # Interior mutability
///
/// This function returns a `State<T>` handle that internally uses an
/// `Arc<RwLock<T>>`. Use `with`, `with_mut`, `get`, or `set` to read or update
/// the value without handling synchronization primitives directly.
///
/// # Comparison with [`remember`]
///
/// Use this function when the state is generated inside a loop or dynamic
/// collection where the execution order might change. In other cases,
/// [`remember`] is sufficient.
///
/// # Panics
///
/// This function must be called during a component's build/render phase.
/// Calling it during the measure or input handling phases will panic.
pub fn remember_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    ensure_build_phase();
    let (logic_id, slot_hash) = compute_slot_key(&key);
    let type_id = TypeId::of::<T>();
    let slot_key = SlotKey {
        logic_id,
        slot_hash,
        type_id,
    };

    let mut table = slot_table().write();
    let mut init_opt = Some(init);

    if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot as usize)
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
        let slot = if let Some(slot) = table.free_list.pop() {
            slot
        } else {
            table.entries.push(SlotEntry {
                key: slot_key,
                generation: 0,
                value: None,
                last_alive_epoch: 0,
            });
            (table.entries.len() - 1) as u32
        };

        let epoch = table.epoch;
        let generation = {
            let entry = table
                .entries
                .get_mut(slot as usize)
                .expect("slot entry should exist");
            entry.key = slot_key;
            entry.generation = entry.generation.wrapping_add(1);
            let init_fn = init_opt
                .take()
                .expect("remember_with_key init called more than once");
            entry.value = Some(Arc::new(RwLock::new(init_fn())));
            entry.last_alive_epoch = epoch;
            entry.generation
        };

        table.key_to_slot.insert(slot_key, slot);
        State::new(slot, generation)
    }
}

/// Remember a value across frames.
///
/// This function allows a component to "remember" state between frames.
/// The `init` closure is executed only once — when the component first runs.
/// On subsequent updates, the stored value is returned and `init` is not
/// called.
///
/// # Interior mutability
///
/// This function returns a `State<T>` handle that internally uses an
/// `Arc<RwLock<T>>`. Use `with`, `with_mut`, `get`, or `set` to read or update
/// the value without handling synchronization primitives directly.
///
/// # Comparison with [`remember_with_key`]
///
/// `remember` identifies stored state based on the component's call order and
/// control-flow path. It associates state by position within a component, but
/// this does not work reliably for dynamically generated state inside loops.
/// For state that is allocated dynamically in loops, consider using
/// [`remember_with_key`] to explicitly provide a unique key.
///
/// # Panics
///
/// This function must be called during a component's build/render phase.
/// Calling it during the measure or input handling phases will panic.
pub fn remember<F, T>(init: F) -> State<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    remember_with_key((), init)
}

/// Groups the execution of a block of code with a stable key.
///
/// This is useful for maintaining state identity in dynamic lists or loops
/// where the order of items might change.
///
/// # Examples
///
/// ```
/// use tessera_ui::{key, remember, tessera};
///
/// #[tessera]
/// fn my_list(items: Vec<String>) {
///     for item in items {
///         key(item.clone(), || {
///             let state = remember(|| 0);
///         });
///     }
/// }
/// ```
pub fn key<K, F, R>(key: K, block: F) -> R
where
    K: Hash,
    F: FnOnce() -> R,
{
    let key_hash = hash_components(&[&key]);
    let _guard = GroupGuard::new(key_hash);
    block()
}
