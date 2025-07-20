use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::Mutex;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Px, PxPosition,
    focus_state::Focus,
};
use tessera_ui_macros::tessera;

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
};

/// State for the `glass_slider` component.
pub struct GlassSliderState {
    /// True if the user is currently dragging the slider.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
}

impl Default for GlassSliderState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlassSliderState {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
        }
    }
}

/// Arguments for the `glass_slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct GlassSliderArgs {
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
    #[builder(default = "Dp(12.0)")]
    pub track_height: Dp,

    /// Glass tint color for the track background.
    #[builder(default = "Color::new(0.3, 0.3, 0.3, 0.15)")]
    pub track_tint_color: Color,

    /// Glass tint color for the progress fill.
    #[builder(default = "Color::new(0.5, 0.7, 1.0, 0.25)")]
    pub progress_tint_color: Color,

    /// Glass blur radius for all components.
    #[builder(default = "8.0")]
    pub blur_radius: f32,

    /// Border color for the track.
    #[builder(default = "Color::new(0.5, 0.5, 0.5, 0.3)")]
    pub track_border_color: Color,

    /// Border width for the track.
    #[builder(default = "Dp(2.0)")]
    pub track_border_width: Dp,

    /// Disable interaction.
    #[builder(default = "false")]
    pub disabled: bool,
}

#[tessera]
pub fn glass_slider(args: impl Into<GlassSliderArgs>, state: Arc<Mutex<GlassSliderState>>) {
    let args: GlassSliderArgs = args.into();

    // External track (background) with border - capsule shape
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.track_height.to_px()))
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape(Shape::RoundedRectangle {
                corner_radius: args.track_height.0 as f32 / 2.0,
                g2_k_value: 2.0, // Capsule shape
            })
            .border(GlassBorder::new(
                args.track_border_width,
                args.track_border_color,
            ))
            .padding(args.track_border_width)
            .build()
            .unwrap(),
        None,
        || {
            row_ui!(
                RowArgsBuilder::default().build().unwrap(),
                move || {
                    // Internal progress fill - capsule shape using surface
                    let progress_width = (args.width.to_px().to_f32() * args.value)
                        - (args.track_border_width.to_px().to_f32() * 2.0);
                    let effective_height = args.track_height.to_px().to_f32()
                        - (args.track_border_width.to_px().to_f32() * 2.0);
                    surface(
                        SurfaceArgsBuilder::default()
                            .width(DimensionValue::Fixed(Px(progress_width as i32)))
                            .height(DimensionValue::Fill {
                                min: None,
                                max: None,
                            })
                            .color(args.progress_tint_color)
                            .shape(Shape::RoundedRectangle {
                                corner_radius: effective_height / 2.0,
                                g2_k_value: 2.0, // Capsule shape
                            })
                            .build()
                            .unwrap(),
                        None,
                        || {},
                    );
                },
                move || spacer(Dp(10.0)),
            );
        },
    );

    let on_change = args.on_change.clone();
    let state_handler_state = state.clone();
    let disabled = args.disabled;

    state_handler(Box::new(move |input| {
        if disabled {
            return;
        }
        let mut state = state_handler_state.lock();

        let is_in_component = input.cursor_position.is_some_and(|cursor_pos| {
            cursor_pos.x.0 >= 0
                && cursor_pos.x.0 < input.computed_data.width.0
                && cursor_pos.y.0 >= 0
                && cursor_pos.y.0 < input.computed_data.height.0
        });

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
                        let v =
                            (pos.x.0 as f32 / input.computed_data.width.0 as f32).clamp(0.0, 1.0);
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
                let v = (pos.x.0 as f32 / input.computed_data.width.0 as f32).clamp(0.0, 1.0);
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
        let self_height = args.track_height.to_px();

        let track_id = input.children_ids[0];

        // Measure track
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self_width),
            DimensionValue::Fixed(self_height),
        );
        tessera_ui::measure_node(
            track_id,
            &track_constraint,
            input.tree,
            input.metadatas,
            input.compute_resource_manager.clone(),
            input.gpu,
        )?;
        tessera_ui::place_node(track_id, PxPosition::new(Px(0), Px(0)), input.metadatas);

        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }));
}
