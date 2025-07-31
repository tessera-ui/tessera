use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tessera_ui_basic_components::{
    checkbox::CheckboxState as UiCheckboxState, ripple_state::RippleState,
    scrollable::ScrollableState, switch::SwitchState as BasicSwitchState,
};

use crate::{performance_display::PerformanceMetrics, text_editors::TextEditorsState};

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
    pub checkbox_state: CheckboxState,
    pub switch_state: SwitchState,

    // --- Ripple States for Demo Components ---
    // Each interactive component in the demo has its own ripple state
    // to ensure animations are independent.

    // States for `interactive_demo.rs`
    pub primary_button_ripple: Arc<RippleState>,
    pub success_button_ripple: Arc<RippleState>,
    pub danger_button_ripple: Arc<RippleState>,
    pub primary_glass_button_ripple: Arc<RippleState>,
    pub secondary_glass_button_ripple: Arc<RippleState>,
    pub success_glass_button_ripple: Arc<RippleState>,
    pub danger_glass_button_ripple: Arc<RippleState>,

    // States for `component_showcase.rs`
    pub surface_ripple_1: Arc<RippleState>,
    pub surface_ripple_2: Arc<RippleState>,
    pub surface_ripple_3: Arc<RippleState>,
    pub fluid_glass_ripple_1: Arc<RippleState>,
    pub fluid_glass_ripple_2: Arc<RippleState>,
    pub fluid_glass_ripple_3: Arc<RippleState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
            text_editors_state: TextEditorsState::new(),
            scrollable_state: Arc::new(ScrollableState::new()),
            checkbox_state: CheckboxState::new(),
            switch_state: SwitchState::new(),

            // Ripple states for interactive_demo
            primary_button_ripple: Arc::new(RippleState::new()),
            success_button_ripple: Arc::new(RippleState::new()),
            danger_button_ripple: Arc::new(RippleState::new()),
            primary_glass_button_ripple: Arc::new(RippleState::new()),
            secondary_glass_button_ripple: Arc::new(RippleState::new()),
            success_glass_button_ripple: Arc::new(RippleState::new()),
            danger_glass_button_ripple: Arc::new(RippleState::new()),

            // Ripple states for component_showcase
            surface_ripple_1: Arc::new(RippleState::new()),
            surface_ripple_2: Arc::new(RippleState::new()),
            surface_ripple_3: Arc::new(RippleState::new()),
            fluid_glass_ripple_1: Arc::new(RippleState::new()),
            fluid_glass_ripple_2: Arc::new(RippleState::new()),
            fluid_glass_ripple_3: Arc::new(RippleState::new()),
        }
    }
}
