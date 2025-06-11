//! Focus is a common requirement. As a functional UI framework,
//! `tessera` lacks a stable method for locating specific components.
//! Treating focus as an independent, shared state aligns better with
//! `tessera`'s design philosophy and provides greater flexibility.

use std::sync::OnceLock;

use parking_lot::{RwLock, RwLockReadGuard};
use uuid::Uuid;

static FOCUS_STATE: OnceLock<RwLock<FocusState>> = OnceLock::new();

/// Focus state
#[derive(Default)]
struct FocusState {
    focused: Option<Uuid>,
}

fn read_focus_state() -> RwLockReadGuard<'static, FocusState> {
    FOCUS_STATE
        .get_or_init(|| RwLock::new(FocusState::default()))
        .read()
}

fn write_focus_state() -> parking_lot::RwLockWriteGuard<'static, FocusState> {
    FOCUS_STATE
        .get_or_init(|| RwLock::new(FocusState::default()))
        .write()
}

/// Focus
pub struct Focus {
    id: Uuid,
}

impl Default for Focus {
    fn default() -> Self {
        Self::new()
    }
}

impl Focus {
    /// Creates a new focus
    pub fn new() -> Self {
        Focus { id: Uuid::new_v4() }
    }

    /// Checks if the focus is currently active
    pub fn is_focused(&self) -> bool {
        let focus_state = read_focus_state();
        focus_state.focused == Some(self.id)
    }

    /// Requests focus
    pub fn request_focus(&self) {
        let mut focus_state = write_focus_state();
        focus_state.focused = Some(self.id);
    }

    /// Clears focus
    pub fn unfocus(&self) {
        let mut focus_state = write_focus_state();
        if focus_state.focused == Some(self.id) {
            focus_state.focused = None;
        }
    }
}

impl Drop for Focus {
    fn drop(&mut self) {
        self.unfocus(); // Ensure focus is cleared when the Focus instance is dropped
    }
}
