//! A component that displays content sliding up from the bottom of the screen.
//!
//! The `bottom_sheet_provider` is the core of this module. It manages the presentation
//! and dismissal of a "bottom sheet" — a common UI pattern for showing contextual
//! information or actions.
//!
//! # Key Components
//!
//! * **[`bottom_sheet_provider`]**: The main component function that you call to create the UI.
//!   It orchestrates the main content, the scrim (background overlay), and the sheet content itself.
//! * **[`BottomSheetProviderState`]**: A state object that you must create and manage to control
//!   the bottom sheet. Use its [`open()`](BottomSheetProviderState::open) and
//!   [`close()`](BottomSheetProviderState::close) methods to show and hide the sheet.
//! * **[`BottomSheetProviderArgs`]**: Configuration for the provider, including the visual
//!   [`style`](BottomSheetStyle) and the mandatory `on_close_request` callback.
//! * **[`BottomSheetStyle`]**: Defines the appearance of the background scrim, either `Material`
//!   (a simple dark overlay) or `Glass` (a blurred, translucent effect).
//!
//! # Behavior
//!
//! - The sheet animates smoothly into and out of view.
//! - It displays a background scrim that blocks interaction with the main content.
//! - Clicking the scrim or pressing the `Escape` key triggers the `on_close_request` callback.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//! use tessera_ui::{tessera, Renderer};
//! use tessera_ui_basic_components::{
//!     bottom_sheet::{
//!         bottom_sheet_provider, BottomSheetProviderArgsBuilder, BottomSheetProviderState
//!     },
//!     button::{button, ButtonArgsBuilder},
//!     ripple_state::RippleState,
//!     text::{text, TextArgsBuilder},
//! };
//!
//! // 1. Define an application state to hold the bottom sheet's state.
//! #[derive(Default)]
//! struct AppState {
//!     sheet_state: Arc<RwLock<BottomSheetProviderState>>,
//!     ripple_state: Arc<RippleState>,
//! }
//!
//! #[tessera]
//! fn app(state: Arc<RwLock<AppState>>) {
//!     let sheet_state = state.read().sheet_state.clone();
//!
//!     // 2. Use the bottom_sheet_provider.
//!     bottom_sheet_provider(
//!         BottomSheetProviderArgsBuilder::default()
//!             // 3. Provide a callback to handle close requests.
//!             .on_close_request(Arc::new({
//!                 let sheet_state = sheet_state.clone();
//!                 move || sheet_state.write().close()
//!             }))
//!             .build()
//!             .unwrap(),
//!         sheet_state.clone(),
//!         // 4. Define the main content that is always visible.
//!         move || {
//!             button(
//!                 ButtonArgsBuilder::default()
//!                     .on_click(Arc::new({
//!                         let sheet_state = sheet_state.clone();
//!                         move || sheet_state.write().open()
//!                     }))
//!                     .build()
//!                     .unwrap(),
//!                 state.read().ripple_state.clone(),
//!                 || text(TextArgsBuilder::default().text("Show Sheet".to_string()).build().unwrap())
//!             );
//!         },
//!         // 5. Define the content of the bottom sheet itself.
//!         // It receives an `alpha` value for fade animations.
//!         |alpha| {
//!             text(TextArgsBuilder::default().text("This is the bottom sheet!".to_string()).build().unwrap());
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

/// Defines the visual style of the bottom sheet's scrim.
///
/// The scrim is the overlay that appears behind the bottom sheet, covering the main content.
#[derive(Default, Clone, Copy)]
pub enum BottomSheetStyle {
    /// A translucent glass effect that blurs the content behind it.
    /// This style is more resource-intensive and may not be suitable for all targets.
    Glass,
    /// A simple, semi-transparent dark overlay. This is the default style.
    #[default]
    Material,
}

/// Configuration arguments for the [`bottom_sheet_provider`].
#[derive(Builder)]
pub struct BottomSheetProviderArgs {
    /// A callback that is invoked when the user requests to close the sheet.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape` key.
    /// The callback is responsible for calling [`BottomSheetProviderState::close()`].
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// The visual style of the scrim. See [`BottomSheetStyle`].
    #[builder(default)]
    pub style: BottomSheetStyle,
}

/// Manages the open/closed state of a [`bottom_sheet_provider`].
///
/// This state object must be created by the application and passed to the
/// [`bottom_sheet_provider`]. It is used to control the visibility of the sheet
/// programmatically.
///
/// For safe shared access across different parts of your UI (e.g., a button that opens
/// the sheet and the provider itself), this state should be wrapped in an `Arc<RwLock<>>`.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui_basic_components::bottom_sheet::BottomSheetProviderState;
///
/// // Create the state, wrapped for shared access.
/// let sheet_state = Arc::new(RwLock::new(BottomSheetProviderState::default()));
///
/// // Later, in an event handler (e.g., a button click):
/// sheet_state.write().open();
///
/// // Or to close it:
/// sheet_state.write().close();
/// ```
#[derive(Default)]
pub struct BottomSheetProviderState {
    is_open: bool,
    timer: Option<Instant>,
}

impl BottomSheetProviderState {
    /// Initiates the animation to open the bottom sheet.
    ///
    /// If the sheet is already open, this has no effect. If the sheet is currently
    /// closing, it will reverse direction and start opening from its current position.
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

    /// Initiates the animation to close the bottom sheet.
    ///
    /// If the sheet is already closed, this has no effect. If the sheet is currently
    /// opening, it will reverse direction and start closing from its current position.
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

/// Compute Y position for bottom sheet placement.
fn compute_bottom_sheet_y(
    parent_height: Px,
    child_height: Px,
    progress: f32,
    is_open: bool,
) -> i32 {
    let parent = parent_height.0 as f32;
    let child = child_height.0 as f32;
    let y = if is_open {
        parent - child * progress
    } else {
        parent - child * (1.0 - progress)
    };
    y as i32
}

fn render_glass_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
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

fn render_material_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
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
fn render_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
    match args.style {
        BottomSheetStyle::Glass => render_glass_scrim(args, progress, is_open),
        BottomSheetStyle::Material => render_material_scrim(args, progress, is_open),
    }
}

/// Snapshot provider state to reduce lock duration and centralize access.
fn snapshot_state(state: &Arc<RwLock<BottomSheetProviderState>>) -> (bool, Option<Instant>) {
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

/// Place bottom sheet if present. Extracted to reduce complexity of the parent function.
fn place_bottom_sheet_if_present(
    input: &tessera_ui::MeasureInput<'_>,
    state_for_measure: &Arc<RwLock<BottomSheetProviderState>>,
    progress: f32,
) {
    if input.children_ids.len() <= 2 {
        return;
    }

    let bottom_sheet_id = input.children_ids[2];

    let child_size = match input.measure_child(bottom_sheet_id, input.parent_constraint) {
        Ok(s) => s,
        Err(_) => return,
    };

    let parent_height = input.parent_constraint.height.get_max().unwrap_or(Px(0));
    let current_is_open = state_for_measure.read().is_open;
    let y = compute_bottom_sheet_y(parent_height, child_size.height, progress, current_is_open);
    input.place_child(bottom_sheet_id, PxPosition::new(Px(0), Px(y)));
}

fn render_content(
    style: BottomSheetStyle,
    bottom_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    match style {
        BottomSheetStyle::Glass => {
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .shape(Shape::RoundedRectangle {
                        top_left: 50.0,
                        top_right: 50.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                        g2_k_value: 3.0,
                    })
                    .tint_color(Color::new(0.6, 0.8, 1.0, 0.3)) // Give it a slight blue tint
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .refraction_amount(25.0)
                    .padding(Dp(20.0))
                    .blur_radius(10.0)
                    .block_input(true)
                    .build()
                    .unwrap(),
                None,
                bottom_sheet_content,
            );
        }
        BottomSheetStyle::Material => {
            surface(
                SurfaceArgsBuilder::default()
                    .style(Color::new(0.2, 0.2, 0.2, 1.0).into())
                    .shape(Shape::RoundedRectangle {
                        top_left: 25.0,
                        top_right: 25.0,
                        bottom_right: 0.0,
                        bottom_left: 0.0,
                        g2_k_value: 3.0,
                    })
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .padding(Dp(20.0))
                    .block_input(true)
                    .build()
                    .unwrap(),
                None,
                bottom_sheet_content,
            );
        }
    }
}

/// Renders a bottom sheet UI group, managing its animation, scrim, and content.
///
/// This is the main function for creating a bottom sheet. It should be called within a
/// component that manages the application's state.
///
/// # Arguments
///
/// - `args`: Configuration options, including the style and close request handler.
///   See [`BottomSheetProviderArgs`].
/// - `state`: The shared state object that controls whether the sheet is open or closed.
///   See [`BottomSheetProviderState`].
/// - `main_content`: A closure that renders the primary UI content, which is always visible
///   behind the sheet.
/// - `bottom_sheet_content`: A closure that renders the content of the sheet itself. It
///   receives a `f32` argument representing the current animation progress (alpha),
///   which can be used to fade content in and out.
#[tessera]
pub fn bottom_sheet_provider(
    args: BottomSheetProviderArgs,
    state: Arc<RwLock<BottomSheetProviderState>>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    bottom_sheet_content: impl FnOnce() + Send + Sync + 'static,
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

    // Render bottom sheet content.
    render_content(args.style, bottom_sheet_content);

    // Measurement: place main content, scrim and bottom sheet.
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

        // Place bottom sheet (if present) using extracted helper.
        place_bottom_sheet_if_present(input, &state_for_measure, progress);

        // Return the main content size (best-effort; unwrap used above to satisfy closure type).
        Ok(main_content_size)
    });
    measure(measure_closure);
}
