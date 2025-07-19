use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::Mutex;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType,
    PxPosition, winit::window::CursorIcon,
};
use tessera_ui_macros::tessera;

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::Shape,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// State for the `glass_switch` component, handling animation.
pub struct GlassSwitchState {
    pub checked: bool,
    progress: Mutex<f32>,
    last_toggle_time: Mutex<Option<Instant>>,
}

impl GlassSwitchState {
    pub fn new(initial_state: bool) -> Self {
        Self {
            checked: initial_state,
            progress: Mutex::new(if initial_state { 1.0 } else { 0.0 }),
            last_toggle_time: Mutex::new(None),
        }
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
        *self.last_toggle_time.lock() = Some(Instant::now());
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct GlassSwitchArgs {
    #[builder(default)]
    pub state: Option<Arc<Mutex<GlassSwitchState>>>,

    #[builder(default = "false")]
    pub checked: bool,

    #[builder(default = "Arc::new(|_| {})")]
    pub on_toggle: Arc<dyn Fn(bool) + Send + Sync>,

    #[builder(default = "Dp(52.0)")]
    pub width: Dp,

    #[builder(default = "Dp(32.0)")]
    pub height: Dp,

    /// Track color when switch is ON
    #[builder(default = "Color::new(0.2, 0.7, 1.0, 0.5)")]
    pub track_on_color: Color,
    /// Track color when switch is OFF
    #[builder(default = "Color::new(0.8, 0.8, 0.8, 0.5)")]
    pub track_off_color: Color,

    /// Thumb alpha when switch is ON (opacity when ON)
    #[builder(default = "0.5")]
    pub thumb_on_alpha: f32,
    /// Thumb alpha when switch is OFF (opacity when OFF)
    #[builder(default = "1.0")]
    pub thumb_off_alpha: f32,

    /// Border for the thumb
    #[builder(
        default = "Some(GlassBorder::new(Dp(2.0), Color::BLUE.with_alpha(0.5)))",
        setter(strip_option)
    )]
    pub thumb_border: Option<GlassBorder>,

    /// Border for the track
    #[builder(
        default = "Some(GlassBorder::new(Dp(2.0), Color::WHITE.with_alpha(0.5)))",
        setter(strip_option)
    )]
    pub track_border: Option<GlassBorder>,

    /// Padding around the thumb
    #[builder(default = "Dp(3.0)")]
    pub thumb_padding: Dp,
}

#[tessera]
pub fn glass_switch(args: impl Into<GlassSwitchArgs>) {
    let args: GlassSwitchArgs = args.into();
    let thumb_size = Dp(args.height.0 - (args.thumb_padding.0 * 2.0));

    // Track (background) as the first child, rendered with fluid_glass
    let progress = args
        .state
        .as_ref()
        .map(|s| *s.lock().progress.lock())
        .unwrap_or(if args.checked { 1.0 } else { 0.0 });
    let track_color = Color {
        r: args.track_off_color.r + (args.track_on_color.r - args.track_off_color.r) * progress,
        g: args.track_off_color.g + (args.track_on_color.g - args.track_off_color.g) * progress,
        b: args.track_off_color.b + (args.track_on_color.b - args.track_off_color.b) * progress,
        a: args.track_off_color.a + (args.track_on_color.a - args.track_off_color.a) * progress,
    };
    let mut arg = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(args.width.to_px()))
        .height(DimensionValue::Fixed(args.height.to_px()))
        .tint_color(track_color)
        .blur_radius(10.0)
        .shape(Shape::RoundedRectangle {
            corner_radius: args.height.to_px().to_f32() / 2.0,
            g2_k_value: 2.0,
        })
        .blur_radius(8.0);
    if let Some(border) = args.track_border {
        arg = arg.border(border);
    }
    let track_glass_arg = arg.build().unwrap();
    fluid_glass(track_glass_arg, None, || {});

    // Thumb (slider) is always white, opacity changes with progress
    let thumb_alpha =
        args.thumb_off_alpha + (args.thumb_on_alpha - args.thumb_off_alpha) * progress;
    let thumb_color = Color::new(1.0, 1.0, 1.0, thumb_alpha);
    let mut thumb_glass_arg = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(thumb_size.to_px()))
        .height(DimensionValue::Fixed(thumb_size.to_px()))
        .tint_color(thumb_color)
        .refraction_height(1.0)
        .shape(Shape::Ellipse);
    if let Some(border) = args.thumb_border {
        thumb_glass_arg = thumb_glass_arg.border(border);
    }
    let thumb_glass_arg = thumb_glass_arg.build().unwrap();
    fluid_glass(thumb_glass_arg, None, || {});

    let on_toggle = args.on_toggle.clone();
    let state = args.state.clone();
    let checked = args.checked;

    state_handler(Box::new(move |input| {
        if let Some(state) = &state {
            let state = state.lock();
            let mut progress = state.progress.lock();
            if let Some(last_toggle_time) = *state.last_toggle_time.lock() {
                let elapsed = last_toggle_time.elapsed();
                let animation_fraction =
                    (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
                *progress = if state.checked {
                    animation_fraction
                } else {
                    1.0 - animation_fraction
                };
            }
        }

        let size = input.computed_data;
        let is_cursor_in = if let Some(pos) = input.cursor_position {
            pos.x.0 >= 0 && pos.x.0 < size.width.0 && pos.y.0 >= 0 && pos.y.0 < size.height.0
        } else {
            false
        };
        if is_cursor_in {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }
        for e in input.cursor_events.iter() {
            if let CursorEventContent::Pressed(PressKeyEventType::Left) = &e.content {
                if is_cursor_in {
                    if let Some(state) = &state {
                        state.lock().toggle();
                    }
                    on_toggle(!checked);
                }
            }
        }
    }));

    measure(Box::new(move |input| {
        let track_id = input.children_ids[0]; // track is the first child
        let thumb_id = input.children_ids[1]; // thumb is the second child
        // Prepare constraints for both children
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(args.width.to_px()),
            DimensionValue::Fixed(args.height.to_px()),
        );
        let thumb_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
        );
        // Measure both children in parallel
        let nodes_constraints = vec![(track_id, track_constraint), (thumb_id, thumb_constraint)];
        let sizes_map = tessera_ui::measure_nodes(
            nodes_constraints,
            input.tree,
            input.metadatas,
            input.compute_resource_manager.clone(),
            input.gpu,
        );
        let _track_size = sizes_map
            .get(&track_id)
            .and_then(|r| r.as_ref().ok())
            .expect("track measurement failed");
        let thumb_size = sizes_map
            .get(&thumb_id)
            .and_then(|r| r.as_ref().ok())
            .expect("thumb measurement failed");
        let self_width_px = args.width.to_px();
        let self_height_px = args.height.to_px();
        let thumb_padding_px = args.thumb_padding.to_px();
        let progress = args
            .state
            .as_ref()
            .map(|s| *s.lock().progress.lock())
            .unwrap_or(if args.checked { 1.0 } else { 0.0 });
        // Place track at origin
        tessera_ui::place_node(
            track_id,
            PxPosition::new(tessera_ui::Px(0), tessera_ui::Px(0)),
            input.metadatas,
        );
        // Place thumb according to progress
        let start_x = thumb_padding_px;
        let end_x = self_width_px - thumb_size.width - thumb_padding_px;
        let thumb_x = start_x.0 as f32 + (end_x.0 - start_x.0) as f32 * progress;
        let thumb_y = (self_height_px - thumb_size.height) / 2;
        tessera_ui::place_node(
            thumb_id,
            PxPosition::new(tessera_ui::Px(thumb_x as i32), thumb_y),
            input.metadatas,
        );
        Ok(ComputedData {
            width: self_width_px,
            height: self_height_px,
        })
    }));
}
