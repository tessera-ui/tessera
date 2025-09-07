use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Px, PxPosition, tessera, winit};

use crate::{
    animation,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIM_TIME: Duration = Duration::from_millis(300);

/// Defines the visual style of the side bar's scrim.
#[derive(Default, Clone, Copy)]
pub enum SideBarStyle {
    /// A translucent glass effect that blurs the content behind it.
    Glass,
    /// A simple, semi-transparent dark overlay.
    #[default]
    Material,
}

#[derive(Builder)]
pub struct SideBarProviderArgs {
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    #[builder(default)]
    pub style: SideBarStyle,
}

#[derive(Default)]
pub struct SideBarProviderState {
    is_open: bool,
    timer: Option<Instant>,
}

impl SideBarProviderState {
    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }

    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }
}

/// Compute eased progress from an optional timer reference.
fn calc_progress_from_timer(timer: Option<&Instant>) -> f32 {
    let raw = match timer {
        None => 1.0,
        Some(t) => {
            let elapsed = t.elapsed();
            if elapsed >= ANIM_TIME {
                1.0
            } else {
                elapsed.as_secs_f32() / ANIM_TIME.as_secs_f32()
            }
        }
    };
    animation::easing(raw)
}

/// Compute blur radius for glass style.
fn blur_radius_for(progress: f32, is_open: bool, max_blur_radius: f32) -> f32 {
    if is_open {
        progress * max_blur_radius
    } else {
        max_blur_radius * (1.0 - progress)
    }
}

/// Compute scrim alpha for material style.
fn scrim_alpha_for(progress: f32, is_open: bool) -> f32 {
    if is_open {
        progress * 0.5
    } else {
        0.5 * (1.0 - progress)
    }
}

/// Compute X position for side bar placement.
fn compute_side_bar_x(child_width: Px, progress: f32, is_open: bool) -> i32 {
    let child = child_width.0 as f32;
    let x = if is_open {
        -child * (1.0 - progress)
    } else {
        -child * progress
    };
    x as i32
}

fn render_glass_scrim(args: &SideBarProviderArgs, progress: f32, is_open: bool) {
    // Glass scrim: compute blur radius and render using fluid_glass.
    let max_blur_radius = 50.0;
    let blur_radius = blur_radius_for(progress, is_open, max_blur_radius);
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .on_click(args.on_close_request.clone())
            .tint_color(Color::TRANSPARENT)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .dispersion_height(0.0)
            .refraction_height(0.0)
            .block_input(true)
            .blur_radius(blur_radius)
            .border(None)
            .shape(Shape::RoundedRectangle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
                g2_k_value: 3.0,
            })
            .noise_amount(0.0)
            .build()
            .unwrap(),
        None,
        || {},
    );
}

fn render_material_scrim(args: &SideBarProviderArgs, progress: f32, is_open: bool) {
    // Material scrim: compute alpha and render a simple dark surface.
    let scrim_alpha = scrim_alpha_for(progress, is_open);
    surface(
        SurfaceArgsBuilder::default()
            .color(Color::BLACK.with_alpha(scrim_alpha))
            .on_click(Some(args.on_close_request.clone()))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .block_input(true)
            .build()
            .unwrap(),
        None,
        || {},
    );
}

/// Render scrim according to configured style.
/// Delegates actual rendering to small, focused helpers to keep the
/// main API surface concise and improve readability.
fn render_scrim(args: &SideBarProviderArgs, progress: f32, is_open: bool) {
    match args.style {
        SideBarStyle::Glass => render_glass_scrim(args, progress, is_open),
        SideBarStyle::Material => render_material_scrim(args, progress, is_open),
    }
}

/// Snapshot provider state to reduce lock duration and centralize access.
fn snapshot_state(state: &Arc<RwLock<SideBarProviderState>>) -> (bool, Option<Instant>) {
    let s = state.read();
    (s.is_open, s.timer)
}

/// Create the keyboard handler closure used to close the sheet on Escape.
fn make_keyboard_closure(
    on_close: Arc<dyn Fn() + Send + Sync>,
) -> Box<dyn Fn(tessera_ui::StateHandlerInput<'_>) + Send + Sync> {
    Box::new(move |input: tessera_ui::StateHandlerInput<'_>| {
        for event in input.keyboard_events.drain(..) {
            if event.state == winit::event::ElementState::Pressed
                && let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                    event.physical_key
            {
                (on_close)();
            }
        }
    })
}

/// Place side bar if present. Extracted to reduce complexity of the parent function.
fn place_side_bar_if_present(
    input: &tessera_ui::MeasureInput<'_>,
    state_for_measure: &Arc<RwLock<SideBarProviderState>>,
    progress: f32,
) {
    if input.children_ids.len() <= 2 {
        return;
    }

    let side_bar_id = input.children_ids[2];

    let child_size = match input.measure_child(side_bar_id, input.parent_constraint) {
        Ok(s) => s,
        Err(_) => return,
    };

    let current_is_open = state_for_measure.read().is_open;
    let x = compute_side_bar_x(child_size.width, progress, current_is_open);
    input.place_child(side_bar_id, PxPosition::new(Px(x), Px(0)));
}

#[tessera]
pub fn side_bar_provider(
    args: SideBarProviderArgs,
    state: Arc<RwLock<SideBarProviderState>>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_bar_content: impl FnOnce(f32) + Send + Sync + 'static,
) {
    // Render main content first.
    main_content();

    // Snapshot state once to minimize locking overhead.
    let (is_open, timer_opt) = snapshot_state(&state);

    // Fast exit when nothing to render.
    if !(is_open || timer_opt.is_some_and(|t| t.elapsed() < ANIM_TIME)) {
        return;
    }

    // Prepare values used by rendering and placement.
    let on_close_for_keyboard = args.on_close_request.clone();
    let progress = calc_progress_from_timer(timer_opt.as_ref());

    // Render the configured scrim.
    render_scrim(&args, progress, is_open);

    // Register keyboard handler (close on Escape).
    let keyboard_closure = make_keyboard_closure(on_close_for_keyboard);
    state_handler(keyboard_closure);

    // Render side bar content with computed alpha.
    let content_alpha = if is_open { progress } else { 1.0 - progress };
    side_bar_content(content_alpha);

    // Measurement: place main content, scrim and side bar.
    let state_for_measure = state.clone();
    let measure_closure = Box::new(move |input: &tessera_ui::MeasureInput<'_>| {
        // Place main content at origin.
        let main_content_id = input.children_ids[0];
        let main_content_size = input.measure_child(main_content_id, input.parent_constraint)?;
        input.place_child(main_content_id, PxPosition::new(Px(0), Px(0)));

        // Place scrim (if present) covering the whole parent.
        if input.children_ids.len() > 1 {
            let scrim_id = input.children_ids[1];
            input.measure_child(scrim_id, input.parent_constraint)?;
            input.place_child(scrim_id, PxPosition::new(Px(0), Px(0)));
        }

        // Place side bar (if present) using extracted helper.
        place_side_bar_if_present(input, &state_for_measure, progress);

        // Return the main content size (best-effort; unwrap used above to satisfy closure type).
        Ok(main_content_size)
    });
    measure(measure_closure);
}
