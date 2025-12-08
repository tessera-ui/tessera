//! This module provides the global runtime state management for tessera.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, VecDeque},
    hash::{Hash, Hasher},
    sync::{Arc, OnceLock},
};

use parking_lot::{Mutex, RwLock};

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
}

#[derive(Clone)]
struct SlotEntry {
    logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
    value: Arc<dyn Any + Send + Sync>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct SlotKey {
    logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

#[derive(Default)]
struct SlotTable {
    prev: Vec<SlotEntry>,
    curr: Vec<SlotEntry>,
    read_cursor: usize,
    stash: HashMap<SlotKey, VecDeque<SlotEntry>>,
}

impl SlotTable {
    fn begin_frame(&mut self) {
        std::mem::swap(&mut self.prev, &mut self.curr);
        self.curr.clear();
        self.read_cursor = 0;
        self.stash.clear();
    }

    fn reset(&mut self) {
        self.prev.clear();
        self.curr.clear();
        self.read_cursor = 0;
        self.stash.clear();
    }

    fn take_from_stash<T>(&mut self, slot_key: &SlotKey) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let entries = self.stash.get_mut(slot_key)?;
        let entry = entries.pop_front()?;
        if entries.is_empty() {
            self.stash.remove(slot_key);
        }
        let value = entry
            .value
            .clone()
            .downcast::<T>()
            .expect("remember_with_key slot type mismatch");
        self.curr.push(entry);
        Some(value)
    }

    fn scan_prev_for<T>(&mut self, slot_key: &SlotKey) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        while self.read_cursor < self.prev.len() {
            let entry = self.prev[self.read_cursor].clone();
            self.read_cursor += 1;

            let entry_key = SlotKey {
                logic_id: entry.logic_id,
                slot_hash: entry.slot_hash,
                type_id: entry.type_id,
            };

            if &entry_key == slot_key {
                let value = entry
                    .value
                    .clone()
                    .downcast::<T>()
                    .expect("remember_with_key slot type mismatch");
                self.curr.push(entry);
                return Some(value);
            }

            self.stash.entry(entry_key).or_default().push_back(entry);
        }

        None
    }

    fn allocate_slot<T, F>(&mut self, slot_key: SlotKey, init: F) -> Arc<T>
    where
        F: FnOnce() -> T,
        T: Send + Sync + 'static,
    {
        let value = Arc::new(init());
        self.curr.push(SlotEntry {
            logic_id: slot_key.logic_id,
            slot_hash: slot_key.slot_hash,
            type_id: slot_key.type_id,
            value: value.clone(),
        });
        value
    }
}

static SLOT_TABLE: OnceLock<Mutex<SlotTable>> = OnceLock::new();

fn slot_table() -> &'static Mutex<SlotTable> {
    SLOT_TABLE.get_or_init(|| Mutex::new(SlotTable::default()))
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

    /// Triggers all registered callbacks (global and per-frame) for window close event.
    /// Called by the event loop when a close event is detected.
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
pub fn push_current_node(node_id: NodeId, logic_id: u64) -> NodeContextGuard {
    NODE_CONTEXT_STACK.with(|stack| stack.borrow_mut().push(node_id));
    LOGIC_ID_STACK.with(|stack| stack.borrow_mut().push(logic_id));
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

fn current_phase() -> Option<RuntimePhase> {
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
/// A guard pushes the provided group id when constructed and pops it when dropped,
/// ensuring grouping stays balanced even with early returns or panics.
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
    (logic_id, group_path_hash ^ key_hash)
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
    slot_table().lock().begin_frame();
}

/// Reset all slot buffers (used on suspension).
pub fn reset_slots() {
    slot_table().lock().reset();
}

/// Remember a value across frames with an explicit key.
///
/// This function allows a component to "remember" state between frames, using
/// a user-provided key to identify the state. This is particularly useful for
/// state generated inside loops or dynamic collections where the execution
/// order might change.
///
/// The `init` closure is executed only once — when the key is first encountered.
/// On subsequent updates with the same key, the stored value is returned and
/// `init` is not called.
///
/// # Interior mutability
///
/// This function returns an `Arc<T>`, which is shared and immutable by default.
/// This design supports multi-threaded measurement. If you need a value that can
/// be modified across frames (for example a counter or input buffer), use a
/// type that provides interior mutability (e.g., `Mutex`, `RwLock`, or atomic
/// types). If you need to mutate the value during measurement or input
/// handling, it must also be `Send + Sync`.
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
pub fn remember_with_key<K, F, T>(key: K, init: F) -> Arc<T>
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

    let mut table = slot_table().lock();
    if let Some(value) = table.take_from_stash::<T>(&slot_key) {
        return value;
    }

    if let Some(value) = table.scan_prev_for::<T>(&slot_key) {
        return value;
    }

    table.allocate_slot(slot_key, init)
}

/// Remember a value across frames.
///
/// This function allows a component to "remember" state between frames.
/// The `init` closure is executed only once — when the component first runs.
/// On subsequent updates, the stored value is returned and `init` is not called.
///
/// # Interior mutability
///
/// This function returns an `Arc<T>`, which is shared and immutable by default.
/// This design supports multi-threaded measurement. If you need a value that can
/// be modified across frames (for example a counter or input buffer), use a
/// type that provides interior mutability (e.g., `Mutex`, `RwLock`, or atomic
/// types). If you need to mutate the value during measurement or input
/// handling, it must also be `Send + Sync`.
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
pub fn remember<F, T>(init: F) -> Arc<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    remember_with_key((), init)
}
