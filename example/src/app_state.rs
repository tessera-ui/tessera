use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tessera_basic_components::{
    ripple_state::RippleState, scrollable::ScrollableState, switch::SwitchState as BasicSwitchState,
};

use crate::{
    animated_spacer::AnimSpacerState, performance_display::PerformanceMetrics,
    text_editors::TextEditorsState,
};

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

#[derive(Clone, Default)]
pub struct CheckboxState {
    pub checked: Arc<RwLock<bool>>,
}

impl CheckboxState {
    pub fn new() -> Self {
        Self {
            checked: Arc::new(RwLock::new(false)),
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
    pub anim_spacer_state: Arc<AnimSpacerState>,
    pub text_editors_state: TextEditorsState,
    pub scrollable_state: Arc<RwLock<ScrollableState>>,
    pub ripple_states: RippleDemoStates,
    pub checkbox_state: CheckboxState,
    pub switch_state: SwitchState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
            anim_spacer_state: Arc::new(AnimSpacerState::new()),
            text_editors_state: TextEditorsState::new(),
            scrollable_state: Arc::new(RwLock::new(ScrollableState::new())),
            ripple_states: RippleDemoStates::new(),
            checkbox_state: CheckboxState::new(),
            switch_state: SwitchState::new(),
        }
    }
}
