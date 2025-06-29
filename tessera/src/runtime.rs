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
    /// The cursor icon requested by components in the current frame.
    pub cursor_icon_request: Option<winit::window::CursorIcon>,
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
}
