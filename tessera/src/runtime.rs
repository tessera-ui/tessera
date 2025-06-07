use std::sync::OnceLock;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::component_tree::ComponentTree;

static TESSERA_RUNTIME: OnceLock<RwLock<TesseraRuntime>> = OnceLock::new();

/// Contains sideeffects and runtime data(such as component tree)
/// access runtime by static function `TesseraRuntime::get()`
#[derive(Default)]
pub struct TesseraRuntime {
    /// Component tree
    pub component_tree: ComponentTree,
    /// The size of the window, by pixels
    pub window_size: [u32; 2],
}

impl TesseraRuntime {
    /// Locks this Runtime with shared read access, blocking the current thread until it can be acquired.
    pub fn read() -> RwLockReadGuard<'static, Self> {
        TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .read()
    }

    /// Locks this Runtime with exclusive write access, blocking the current thread until it can be acquired.
    pub fn write() -> RwLockWriteGuard<'static, Self> {
        TESSERA_RUNTIME
            .get_or_init(|| RwLock::new(Self::default()))
            .write()
    }

    /// Sets the currently focused node in the component tree.
    pub fn set_focused_node(&mut self, node_id: Option<indextree::NodeId>) {
        self.component_tree.focused_node_id = node_id;
    }

    /// Gets the currently focused node from the component tree.
    pub fn get_focused_node(&self) -> Option<indextree::NodeId> {
        self.component_tree.focused_node_id
    }
}
