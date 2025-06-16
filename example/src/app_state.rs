use parking_lot::RwLock;
use std::sync::Arc;
use tessera_basic_components::scrollable::ScrollableState;

use crate::{
    animated_spacer::AnimSpacerState, button_demo::ButtonDemoData,
    performance_display::PerformanceMetrics, text_editors::TextEditorsState,
};

pub struct AppState {
    pub metrics: Arc<PerformanceMetrics>,
    pub anim_spacer_state: Arc<AnimSpacerState>,
    pub button_demo_data: Arc<ButtonDemoData>,
    pub text_editors_state: TextEditorsState,
    pub scrollable_state: Arc<RwLock<ScrollableState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
            anim_spacer_state: Arc::new(AnimSpacerState::new()),
            button_demo_data: Arc::new(ButtonDemoData::new()),
            text_editors_state: TextEditorsState::new(),
            scrollable_state: Arc::new(RwLock::new(ScrollableState::new())),
        }
    }
}
