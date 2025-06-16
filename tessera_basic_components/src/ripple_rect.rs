use derive_builder::Builder;
use std::sync::{Arc, atomic};
use std::time::Instant;
use tessera::{
    BasicDrawable, ComputedData, Constraint, CursorEventContent, DimensionValue,
    PressKeyEventType, Px, RippleProps, ShadowProps,
};
use tessera_macros::tessera;

use crate::pos_misc::is_position_in_component;

/// State for managing ripple animation
pub struct RippleState {
    pub is_animating: atomic::AtomicBool,
    pub start_time: atomic::AtomicU64, // Store as u64 millis since epoch
    pub click_pos_x: atomic::AtomicI32, // Store as fixed-point * 1000
    pub click_pos_y: atomic::AtomicI32, // Store as fixed-point * 1000
}

impl RippleState {
    pub fn new() -> Self {
        Self {
            is_animating: atomic::AtomicBool::new(false),
            start_time: atomic::AtomicU64::new(0),
            click_pos_x: atomic::AtomicI32::new(0),
            click_pos_y: atomic::AtomicI32::new(0),
        }
    }
    
    pub fn start_animation(&self, click_pos: [f32; 2]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        self.start_time.store(now, atomic::Ordering::SeqCst);
        self.click_pos_x.store((click_pos[0] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.click_pos_y.store((click_pos[1] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.is_animating.store(true, atomic::Ordering::SeqCst);
    }
    
    pub fn get_animation_progress(&self) -> Option<(f32, [f32; 2])> {
        let is_animating = self.is_animating.load(atomic::Ordering::SeqCst);
        
        if !is_animating {
            return None;
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let start = self.start_time.load(atomic::Ordering::SeqCst);
        let elapsed_ms = now.saturating_sub(start);
        let progress = (elapsed_ms as f32) / 600.0; // 600ms animation
        
        if progress >= 1.0 {
            self.is_animating.store(false, atomic::Ordering::SeqCst);
            return None;
        }
        
        let click_pos = [
            self.click_pos_x.load(atomic::Ordering::SeqCst) as f32 / 1000.0,
            self.click_pos_y.load(atomic::Ordering::SeqCst) as f32 / 1000.0,
        ];
        
        Some((progress, click_pos))
    }
}

/// Arguments for the `ripple_rect` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct RippleRectArgs {
    /// The fill color of the rectangle (RGBA).
    #[builder(default = "[0.2, 0.5, 0.8, 1.0]")]
    pub color: [f32; 4],
    /// The corner radius of the rectangle.
    #[builder(default = "8.0")]
    pub corner_radius: f32,
    /// Shadow properties of the rectangle.
    #[builder(default)]
    pub shadow: Option<ShadowProps>,
    /// Optional explicit width behavior for the rectangle.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the rectangle.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
    /// The ripple color (RGB).
    #[builder(default = "[1.0, 1.0, 1.0]")]
    pub ripple_color: [f32; 3],
    /// The click callback function
    #[builder(default = "Arc::new(|| {})")]
    pub on_click: Arc<dyn Fn() + Send + Sync>,
}

impl std::fmt::Debug for RippleRectArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RippleRectArgs")
            .field("color", &self.color)
            .field("corner_radius", &self.corner_radius)
            .field("shadow", &self.shadow)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("ripple_color", &self.ripple_color)
            .field("on_click", &"<callback>")
            .finish()
    }
}

impl Default for RippleRectArgs {
    fn default() -> Self {
        RippleRectArgsBuilder::default()
            .build()
            .unwrap()
    }
}

/// Interactive filled rectangle component with ripple effect animation.
#[tessera]
pub fn ripple_rect(args: impl Into<RippleRectArgs>, ripple_state: Arc<RippleState>) {
    let rect_args: RippleRectArgs = args.into();

    // Measure logic with ripple drawable
    {
        let args_for_measure = rect_args.clone();
        let state_for_measure = Arc::clone(&ripple_state);
        measure(Box::new(move |input| {
            // Calculate ripple animation parameters
            let ripple_props = if let Some((progress, click_pos)) = state_for_measure.get_animation_progress() {
                let radius = progress; // Expand from 0 to 1
                let alpha = (1.0 - progress) * 0.3; // Fade out
                
                
                RippleProps {
                    center: click_pos,
                    radius,
                    alpha,
                    color: args_for_measure.ripple_color,
                }
            } else {
                RippleProps::default()
            };

            // Create the ripple drawable
            let drawable = BasicDrawable::RippleRect {
                color: args_for_measure.color,
                corner_radius: args_for_measure.corner_radius,
                shadow: args_for_measure.shadow,
                ripple: ripple_props,
            };

            // Set the drawable for this component
            if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
                metadata.basic_drawable = Some(drawable);
            }

            // Calculate size based on width/height constraints
            let (computed_width, computed_height) = calculate_rect_size(
                &args_for_measure.width,
                &args_for_measure.height,
                &input.effective_constraint,
            );

            Ok(ComputedData {
                width: computed_width,
                height: computed_height,
            })
        }));
    }

    // Event handling for ripple interactions
    {
        let args_for_handler = rect_args.clone();
        let state_for_handler = Arc::clone(&ripple_state);
        state_handler(Box::new(move |input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position;
            let is_cursor_in_rect = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            // Handle mouse events
            if is_cursor_in_rect {
                // Check for mouse press events to start ripple
                let press_events: Vec<_> = input
                    .cursor_events
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.content,
                            CursorEventContent::Pressed(PressKeyEventType::Left)
                        )
                    })
                    .collect();

                // Check for mouse release events (click)
                let release_events: Vec<_> = input
                    .cursor_events
                    .iter()
                    .filter(|event| {
                        matches!(
                            event.content,
                            CursorEventContent::Released(PressKeyEventType::Left)
                        )
                    })
                    .collect();

                if !press_events.is_empty() {
                    if let Some(cursor_pos) = cursor_pos_option {
                        // Convert cursor position to normalized coordinates [-0.5, 0.5]
                        let normalized_x = (cursor_pos.x.to_f32() / size.width.to_f32()) - 0.5;
                        let normalized_y = (cursor_pos.y.to_f32() / size.height.to_f32()) - 0.5;
                        
                        // Start ripple animation
                        state_for_handler.start_animation([normalized_x, normalized_y]);
                    }
                }

                if !release_events.is_empty() {
                    // Trigger click callback
                    (args_for_handler.on_click)();
                }

                // Consume cursor events if we're handling relevant mouse events
                if !press_events.is_empty() || !release_events.is_empty() {
                    input.cursor_events.clear();
                }
            }
        }));
    }
}

/// Helper function to calculate rectangle size based on constraints
fn calculate_rect_size(
    width_behavior: &Option<DimensionValue>,
    height_behavior: &Option<DimensionValue>,
    constraint: &Constraint,
) -> (Px, Px) {
    let computed_width = match width_behavior {
        Some(DimensionValue::Fixed(px)) => *px,
        Some(DimensionValue::Fill { max, .. }) => {
            match &constraint.width {
                DimensionValue::Fixed(w) => *w,
                DimensionValue::Fill { max: constraint_max, .. } => {
                    max.or(*constraint_max).unwrap_or(Px(100))
                }
                DimensionValue::Wrap { max: constraint_max, .. } => {
                    max.or(*constraint_max).unwrap_or(Px(100))
                }
            }
        }
        Some(DimensionValue::Wrap { max, .. }) => {
            max.unwrap_or(Px(100))
        }
        None => Px(100),
    };

    let computed_height = match height_behavior {
        Some(DimensionValue::Fixed(px)) => *px,
        Some(DimensionValue::Fill { max, .. }) => {
            match &constraint.height {
                DimensionValue::Fixed(h) => *h,
                DimensionValue::Fill { max: constraint_max, .. } => {
                    max.or(*constraint_max).unwrap_or(Px(100))
                }
                DimensionValue::Wrap { max: constraint_max, .. } => {
                    max.or(*constraint_max).unwrap_or(Px(100))
                }
            }
        }
        Some(DimensionValue::Wrap { max, .. }) => {
            max.unwrap_or(Px(100))
        }
        None => Px(100),
    };

    (computed_width, computed_height)
}

/// Convenience constructors for common ripple rect styles
impl RippleRectArgs {
    /// Create a primary ripple rect with default blue styling
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        RippleRectArgsBuilder::default()
            .color([0.2, 0.5, 0.8, 1.0]) // Blue
            .ripple_color([1.0, 1.0, 1.0]) // White ripple
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a success ripple rect with green styling
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        RippleRectArgsBuilder::default()
            .color([0.1, 0.7, 0.3, 1.0]) // Green
            .ripple_color([1.0, 1.0, 1.0]) // White ripple
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a danger ripple rect with red styling
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        RippleRectArgsBuilder::default()
            .color([0.8, 0.2, 0.2, 1.0]) // Red
            .ripple_color([1.0, 1.0, 1.0]) // White ripple
            .on_click(on_click)
            .build()
            .unwrap()
    }
}