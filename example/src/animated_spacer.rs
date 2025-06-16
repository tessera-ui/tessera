use std::{
    sync::{Arc, atomic},
    time::Instant,
};
use tessera::{DimensionValue, Px};
use tessera_basic_components::spacer::{SpacerArgsBuilder, spacer};
use tessera_macros::tessera;

pub struct AnimSpacerState {
    pub height: atomic::AtomicI32,
    pub max_height: atomic::AtomicI32,
    pub start_time: Instant,
}

impl AnimSpacerState {
    pub fn new() -> Self {
        Self {
            height: atomic::AtomicI32::new(0),
            max_height: atomic::AtomicI32::new(100),
            start_time: Instant::now(),
        }
    }
}

/// Easing function for smooth animation
fn ease_in_out_sine(x: f32) -> f32 {
    -(0.5 * (std::f32::consts::PI * x).cos()) + 0.5
}

/// Animated spacer component with smooth height animation
#[tessera]
pub fn anim_spacer(state: Arc<AnimSpacerState>) {
    spacer(
        SpacerArgsBuilder::default()
            .height(DimensionValue::Fixed(Px(state
                .height
                .load(atomic::Ordering::SeqCst))))
            .build()
            .unwrap(),
    );

    state_handler(Box::new(move |_| {
        let now = Instant::now();
        let elapsed = now.duration_since(state.start_time).as_secs_f32();

        let max_height = state.max_height.load(atomic::Ordering::SeqCst) as f32;
        let speed = 200.0;
        let period = 2.0 * max_height / speed;
        let t = (elapsed % period) / period;
        let triangle = if t < 0.5 { 2.0 * t } else { 2.0 * (1.0 - t) };
        let eased = ease_in_out_sine(triangle);
        let new_height = (eased * max_height).round() as i32;
        state.height.store(new_height, atomic::Ordering::SeqCst);
    }));
}
