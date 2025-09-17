//! A component that displays content sliding in from the left side of the screen.
//!
//! The `side_bar_provider` manages the presentation and dismissal of a "side bar" — a common UI
//! pattern for showing navigation or contextual controls.
//!
//! # Key Components
//!
//! * **[`side_bar_provider`]**: The main component function that orchestrates the main content,
//!   the scrim (background overlay), and the side bar content itself.
//! * **[`SideBarProviderState`]**: A state object that applications create and manipulate to
//!   control the side bar visibility. Call its [`open()`] and [`close()`] methods to change state.
//! * **[`SideBarProviderArgs`]**: Configuration for the provider, including the visual [`style`]
//!   and the mandatory `on_close_request` callback.
//! * **[`SideBarStyle`]**: Visual variants for scrim and container (Material or Glass).
//!
//! # Behavior
//!
//! - The side bar animates smoothly in and out from the left edge.
//! - A scrim blocks interaction with the main content while the side bar is visible.
//! - Clicking the scrim or pressing the `Escape` key will invoke the provided `on_close_request`
//!   callback.
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//! use tessera_ui::{tessera, Renderer};
//! use tessera_ui_basic_components::{
//!     side_bar::{side_bar_provider, SideBarProviderArgsBuilder, SideBarProviderState, SideBarStyle},
//!     button::{button, ButtonArgsBuilder},
//!     ripple_state::RippleState,
//!     text::{text, TextArgsBuilder},
//! };
//!
//! // 1. Define an application state to hold the side bar's state.
//! #[derive(Default)]
//! struct AppState {
//!     bar_state: Arc<RwLock<SideBarProviderState>>,
//!     ripple_state: Arc<RippleState>,
//! }
//!
//! #[tessera]
//! fn app(state: Arc<RwLock<AppState>>) {
//!     let bar_state = state.read().bar_state.clone();
//!     // 2. Use the side_bar_provider.
//!     side_bar_provider(
//!         SideBarProviderArgsBuilder::default()
//!             // 3. Provide a callback to handle close requests.
//!             .on_close_request(Arc::new({
//!                 let bar_state = bar_state.clone();
//!                 move || bar_state.write().close()
//!             }))
//!             .build()
//!             .unwrap(),
//!         bar_state.clone(),
//!         // 4. Define the main content that is always visible.
//!         move || {
//!             button(
//!                 ButtonArgsBuilder::default()
//!                     .on_click(Arc::new({
//!                         let bar_state = bar_state.clone();
//!                         move || bar_state.write().open()
//!                     }))
//!                     .build()
//!                     .unwrap(),
//!                 state.read().ripple_state.clone(),
//!                 || text(TextArgsBuilder::default().text("Open Side Bar".to_string()).build().unwrap())
//!             );
//!         },
//!         // 5. Define the content of the side bar itself.
//!         || {
//!             text(TextArgsBuilder::default().text("This is the side bar!").build().unwrap());
//!         }
//!     );
//! }
//! ```
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, Px, PxPosition, tessera, winit};

use crate::{
    animation,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIM_TIME: Duration = Duration::from_millis(300);

/// Defines the visual style of the side bar and its scrim.
///
/// The scrim is the overlay that appears behind the side bar, covering the main content.
#[derive(Default, Clone, Copy)]
pub enum SideBarStyle {
    /// A translucent glass effect that blurs the content behind it.
    /// This style may be more costly and is suitable when a blurred backdrop is desired.
    Glass,
    /// A simple, semi-transparent dark overlay. This is the default style.
    #[default]
    Material,
}

#[derive(Builder)]
pub struct SideBarProviderArgs {
    /// A callback that is invoked when the user requests to close the side bar.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape` key.
    /// The callback is expected to call [`SideBarProviderState::close()`].
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// The visual style used by the provider. See [`SideBarStyle`].
    #[builder(default)]
    pub style: SideBarStyle,
}

/// Manages the open/closed state of a [`side_bar_provider`].
///
/// This state object must be created by the application and passed to the
/// [`side_bar_provider`]. It is used to control the visibility of the side bar
/// programmatically. Wrap in `Arc<RwLock<...>>` for shared access from multiple UI parts.
#[derive(Default)]
pub struct SideBarProviderState {
    is_open: bool,
    timer: Option<Instant>,
}

impl SideBarProviderState {
    /// Initiates the animation to open the side bar.
    ///
    /// If the side bar is already open this has no effect. If it is currently
    /// closing, the animation will reverse direction and start opening from the
    /// current animated position.
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

    /// Initiates the animation to close the side bar.
    ///
    /// If the side bar is already closed this has no effect. If it is currently
    /// opening, the animation will reverse direction and start closing from the
    /// current animated position.
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
    let max_blur_radius = 5.0;
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
                top_left: Dp(0.0),
                top_right: Dp(0.0),
                bottom_right: Dp(0.0),
                bottom_left: Dp(0.0),
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
            .style(Color::BLACK.with_alpha(scrim_alpha).into())
            .on_click(args.on_close_request.clone())
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
) -> Box<dyn Fn(tessera_ui::InputHandlerInput<'_>) + Send + Sync> {
    Box::new(move |input: tessera_ui::InputHandlerInput<'_>| {
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

/// Renders a side bar UI group, managing its animation, scrim, keyboard handling and content.
///
/// This is the primary function to create a side bar. Call it inside a component that owns or
/// has access to the application's shared state. The provider renders the main content first,
/// then the scrim (overlay) and the side bar when the `state` indicates visibility.
///
/// # Arguments
///
/// - `args`: Configuration options including the `on_close_request` callback and visual `style`.
///   See [`SideBarProviderArgs`].
/// - `state`: Shared state controlling open/closed animation timing. See [`SideBarProviderState`].
/// - `main_content`: Closure rendering the always-visible main UI (background for the side bar).
/// - `side_bar_content`: Closure rendering the side bar's content.
///
/// The provider registers a keyboard handler that invokes the provided `on_close_request` when
/// the `Escape` key is pressed. Clicking the scrim also triggers the same callback.
#[tessera]
pub fn side_bar_provider(
    args: SideBarProviderArgs,
    state: Arc<RwLock<SideBarProviderState>>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_bar_content: impl FnOnce() + Send + Sync + 'static,
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
    input_handler(keyboard_closure);

    // Render side bar content with computed alpha.
    side_bar_content_wrapper(args.style, side_bar_content);

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

#[tessera]
fn side_bar_content_wrapper(style: SideBarStyle, content: impl FnOnce() + Send + Sync + 'static) {
    match style {
        SideBarStyle::Glass => {
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .shape(Shape::RoundedRectangle {
                        top_left: Dp(0.0),
                        top_right: Dp(25.0),
                        bottom_right: Dp(25.0),
                        bottom_left: Dp(0.0),
                        g2_k_value: 3.0,
                    })
                    .tint_color(Color::new(0.6, 0.8, 1.0, 0.3))
                    .width(DimensionValue::from(Dp(250.0)))
                    .height(tessera_ui::DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .blur_radius(10.0)
                    .padding(Dp(16.0))
                    .block_input(true)
                    .build()
                    .unwrap(),
                None,
                content,
            );
        }
        SideBarStyle::Material => {
            surface(
                SurfaceArgsBuilder::default()
                    .style(Color::new(0.9, 0.9, 0.9, 1.0).into())
                    .width(DimensionValue::from(Dp(250.0)))
                    .height(tessera_ui::DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .padding(Dp(16.0))
                    .shape(Shape::RoundedRectangle {
                        top_left: Dp(0.0),
                        top_right: Dp(25.0),
                        bottom_right: Dp(25.0),
                        bottom_left: Dp(0.0),
                        g2_k_value: 3.0,
                    })
                    .block_input(true)
                    .build()
                    .unwrap(),
                None,
                content,
            );
        }
    }
}
