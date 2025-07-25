use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tessera_ui_basic_components::{
    checkbox::CheckboxState as UiCheckboxState, ripple_state::RippleState,
    scrollable::ScrollableState, switch::SwitchState as BasicSwitchState,
};

use crate::{performance_display::PerformanceMetrics, text_editors::TextEditorsState};

pub struct RippleDemoStates {
    pub primary: Arc<RippleState>,
    pub success: Arc<RippleState>,
    pub danger: Arc<RippleState>,
    pub custom: Arc<RippleState>,
}

impl RippleDemoStates {
    pub fn new() -> Self {
        Self {
            primary: Arc::new(RippleState::new()),
            success: Arc::new(RippleState::new()),
            danger: Arc::new(RippleState::new()),
            custom: Arc::new(RippleState::new()),
        }
    }
}

#[derive(Clone)]
pub struct CheckboxState {
    pub checked: Arc<RwLock<bool>>,  // Separate for demo logic
    pub state: Arc<UiCheckboxState>, // Holds ripple+checkmark state
}

impl CheckboxState {
    pub fn new() -> Self {
        Self {
            checked: Arc::new(RwLock::new(false)),
            state: Arc::new(UiCheckboxState::new(false)),
        }
    }
}

#[derive(Clone)]
pub struct SwitchState {
    pub state: Arc<Mutex<BasicSwitchState>>,
}

impl SwitchState {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(BasicSwitchState::new(false))),
        }
    }
}

pub struct AppState {
    pub metrics: Arc<PerformanceMetrics>,
    pub text_editors_state: TextEditorsState,
    pub scrollable_state: Arc<ScrollableState>,
    pub ripple_states: RippleDemoStates,
    pub checkbox_state: CheckboxState,
    pub switch_state: SwitchState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
            text_editors_state: TextEditorsState::new(),
            scrollable_state: Arc::new(ScrollableState::new()),
            ripple_states: RippleDemoStates::new(),
            checkbox_state: CheckboxState::new(),
            switch_state: SwitchState::new(),
        }
    }
}
