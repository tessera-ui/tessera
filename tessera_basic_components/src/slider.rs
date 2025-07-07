use crate::{
    pipelines::{ShadowProps, ShapeCommand},
    pos_misc::is_position_in_component,
    surface::{SurfaceArgsBuilder, surface},
};
use derive_builder::Builder;
use parking_lot::Mutex;
use std::sync::Arc;
use tessera::{
    ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Px, PxPosition,
    focus_state::Focus, place_node,
};
use tessera_macros::tessera;

/// State for the `slider` component.
pub struct SliderState {
    /// True if the user is currently dragging the thumb.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
}

impl Default for SliderState {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderState {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
        }
    }
}

/// Arguments for the `slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SliderArgs {
    /// The current value of the slider, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// Callback function triggered when the slider's value changes.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,

    /// The width of the slider track.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

    /// The height of the slider track.
    #[builder(default = "Dp(4.0)")]
    pub track_height: Dp,

    /// The color of the active part of the track (from start to thumb).
    #[builder(default = "[0.2, 0.5, 0.8, 1.0]")]
    pub active_track_color: [f32; 4],

    /// The color of the inactive part of the track (from thumb to end).
    #[builder(default = "[0.8, 0.8, 0.8, 1.0]")]
    pub inactive_track_color: [f32; 4],

    /// The color of the draggable thumb.
    #[builder(default = "[1.0, 1.0, 1.0, 1.0]")]
    pub thumb_color: [f32; 4],

    /// The diameter of the draggable thumb.
    #[builder(default = "Dp(16.0)")]
    pub thumb_size: Dp,

    /// Shadow for the thumb.
    #[builder(default)]
    pub thumb_shadow: Option<ShadowProps>,
}

impl std::fmt::Debug for SliderArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SliderArgs")
            .field("value", &self.value)
            .field("on_change", &"<callback>")
            .field("width", &self.width)
            .field("track_height", &self.track_height)
            .field("active_track_color", &self.active_track_color)
            .field("inactive_track_color", &self.inactive_track_color)
            .field("thumb_color", &self.thumb_color)
            .field("thumb_size", &self.thumb_size)
            .finish()
    }
}

#[tessera]
pub fn slider(args: impl Into<SliderArgs>, state: Arc<Mutex<SliderState>>) {
    let args: SliderArgs = args.into();

    // Active track
    surface(
        SurfaceArgsBuilder::default()
            .color(args.active_track_color)
            .corner_radius(args.track_height.0 as f32 / 2.0f32)
            .build()
            .unwrap(),
        None,
        || {},
    );

    // The thumb component
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.thumb_size.to_px()))
            .height(DimensionValue::Fixed(args.thumb_size.to_px()))
            .color(args.thumb_color)
            .corner_radius(args.thumb_size.0 as f32 / 2.0f32)
            .shadow(args.thumb_shadow)
            .build()
            .unwrap(),
        None,
        || {},
    );

    let on_change = args.on_change.clone();
    let state_handler_state = state.clone();
    state_handler(Box::new(move |input| {
        let mut state = state_handler_state.lock();

        let is_in_component = input
            .cursor_position
            .is_some_and(|cursor_pos| is_position_in_component(input.computed_data, cursor_pos));

        if !is_in_component && !state.is_dragging {
            return;
        }

        let mut new_value = None;

        for event in input.cursor_events.iter() {
            match &event.content {
                CursorEventContent::Pressed(_) => {
                    state.focus.request_focus();
                    state.is_dragging = true;

                    if let Some(pos) = input.cursor_position {
                        let thumb_half_width = args.thumb_size.to_px().0 as f32 / 2.0;
                        let effective_width =
                            input.computed_data.width.0 as f32 - thumb_half_width * 2.0;
                        let v =
                            ((pos.x.0 as f32 - thumb_half_width) / effective_width).clamp(0.0, 1.0);
                        new_value = Some(v);
                    }
                }
                CursorEventContent::Released(_) => {
                    state.is_dragging = false;
                }
                _ => {}
            }
        }

        if state.is_dragging {
            if let Some(pos) = input.cursor_position {
                let thumb_half_width = args.thumb_size.to_px().0 as f32 / 2.0;
                let effective_width = input.computed_data.width.0 as f32 - thumb_half_width * 2.0;
                let v = ((pos.x.0 as f32 - thumb_half_width) / effective_width).clamp(0.0, 1.0);
                new_value = Some(v);
            }
        }

        if let Some(v) = new_value {
            if (v - args.value).abs() > f32::EPSILON {
                on_change(v);
            }
        }
    }));

    measure(Box::new(move |input| {
        let self_width = args.width.to_px();
        let self_height = args.thumb_size.to_px();
        let track_height = args.track_height.to_px();
        let track_y = (self_height - track_height) / 2;

        let active_track_id = input.children_ids[0];
        let thumb_id = input.children_ids[1];

        // Measure active track
        let active_track_width = Px((self_width.to_f32() * args.value) as i32);
        let active_track_constraint = Constraint::new(
            DimensionValue::Fixed(active_track_width),
            DimensionValue::Fixed(track_height),
        );
        tessera::measure_node(
            active_track_id,
            &active_track_constraint,
            input.tree,
            input.metadatas,
        )?;
        place_node(
            active_track_id,
            PxPosition::new(Px(0), track_y),
            input.metadatas,
        );

        // Measure thumb
        let thumb_constraint = Constraint::new(
            DimensionValue::Fixed(args.thumb_size.to_px()),
            DimensionValue::Fixed(args.thumb_size.to_px()),
        );
        let thumb_size =
            tessera::measure_node(thumb_id, &thumb_constraint, input.tree, input.metadatas)?;

        // Calculate thumb position
        let thumb_x = (self_width - thumb_size.width).to_f32() * args.value;
        let thumb_y = (self_height - thumb_size.height) / 2;
        place_node(
            thumb_id,
            PxPosition::new(Px(thumb_x as i32), thumb_y),
            input.metadatas,
        );

        // Draw inactive track
        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            let inactive_track_command = ShapeCommand::Rect {
                color: args.inactive_track_color,
                corner_radius: track_height.0 as f32 / 2.0,
                shadow: None,
            };
            metadata.basic_drawable = Some(Box::new(inactive_track_command));
        }

        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }));
}
