//! A component that displays content sliding up from the bottom of the screen.
//!
//! ## Usage
//!
//! Used to show contextual information or actions in a modal sheet.
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
/// the sheet and the provider itself), clone the handle freely—the locking is handled
/// internally so clones stay lightweight.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui_basic_components::bottom_sheet::BottomSheetProviderState;
///
/// // Create the state handle (cheap to clone and share).
/// let sheet_state = BottomSheetProviderState::new();
///
/// // Later, in an event handler (e.g., a button click):
/// let state = sheet_state.clone();
/// state.open();
///
/// // Or to close it:
/// sheet_state.close();
/// ```
#[derive(Default)]
struct BottomSheetProviderStateInner {
    is_open: bool,
    timer: Option<Instant>,
}

#[derive(Clone, Default)]
pub struct BottomSheetProviderState {
    inner: Arc<RwLock<BottomSheetProviderStateInner>>,
}

impl BottomSheetProviderState {
    /// Creates a new provider state handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initiates the animation to open the bottom sheet.
    ///
    /// If the sheet is already open, this has no effect. If the sheet is currently
    /// closing, it will reverse direction and start opening from its current position.
    pub fn open(&self) {
        let mut inner = self.inner.write();
        if !inner.is_open {
            inner.is_open = true;
            let mut timer = Instant::now();
            if let Some(old_timer) = inner.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            inner.timer = Some(timer);
        }
    }

    /// Initiates the animation to close the bottom sheet.
    ///
    /// If the sheet is already closed, this has no effect. If the sheet is currently
    /// opening, it will reverse direction and start closing from its current position.
    pub fn close(&self) {
        let mut inner = self.inner.write();
        if inner.is_open {
            inner.is_open = false;
            let mut timer = Instant::now();
            if let Some(old_timer) = inner.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            inner.timer = Some(timer);
        }
    }

    /// Returns whether the sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.inner.read().is_open
    }

    /// Returns whether the sheet is currently animating in either direction.
    pub fn is_animating(&self) -> bool {
        self.inner
            .read()
            .timer
            .is_some_and(|t| t.elapsed() < ANIM_TIME)
    }

    fn snapshot(&self) -> (bool, Option<Instant>) {
        let inner = self.inner.read();
        (inner.is_open, inner.timer)
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
            .dispersion_height(Dp(0.0))
            .refraction_height(Dp(0.0))
            .block_input(true)
            .blur_radius(Dp(blur_radius as f64))
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
            .expect("FluidGlassArgsBuilder failed with required fields set"),
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
            .expect("SurfaceArgsBuilder failed with required fields set"),
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
fn snapshot_state(state: &BottomSheetProviderState) -> (bool, Option<Instant>) {
    state.snapshot()
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

/// Place bottom sheet if present. Extracted to reduce complexity of the parent function.
fn place_bottom_sheet_if_present(
    input: &tessera_ui::MeasureInput<'_>,
    state_for_measure: &BottomSheetProviderState,
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
    let current_is_open = state_for_measure.is_open();
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
                        top_left: Dp(50.0),
                        top_right: Dp(50.0),
                        bottom_right: Dp(0.0),
                        bottom_left: Dp(0.0),
                        g2_k_value: 3.0,
                    })
                    .tint_color(Color::new(0.6, 0.8, 1.0, 0.3)) // Give it a slight blue tint
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .refraction_amount(25.0)
                    .padding(Dp(20.0))
                    .blur_radius(Dp(10.0))
                    .block_input(true)
                    .build()
                    .expect("FluidGlassArgsBuilder failed with required fields set"),
                None,
                bottom_sheet_content,
            );
        }
        BottomSheetStyle::Material => {
            surface(
                SurfaceArgsBuilder::default()
                    .style(Color::new(0.2, 0.2, 0.2, 1.0).into())
                    .shape(Shape::RoundedRectangle {
                        top_left: Dp(25.0),
                        top_right: Dp(25.0),
                        bottom_right: Dp(0.0),
                        bottom_left: Dp(0.0),
                        g2_k_value: 3.0,
                    })
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .padding(Dp(20.0))
                    .block_input(true)
                    .build()
                    .expect("SurfaceArgsBuilder failed with required fields set"),
                None,
                bottom_sheet_content,
            );
        }
    }
}

/// # bottom_sheet_provider
///
/// Provides a modal bottom sheet for contextual actions or information.
///
/// ## Usage
///
/// Show contextual menus, supplemental information, or simple forms without navigating away from the main screen.
///
/// ## Parameters
///
/// - `args` — configuration for the sheet's appearance and behavior; see [`BottomSheetProviderArgs`].
/// - `state` — a clonable [`BottomSheetProviderState`] used to open and close the sheet.
/// - `main_content` — closure that renders the always-visible base UI.
/// - `bottom_sheet_content` — closure that renders the content of the sheet itself.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::bottom_sheet::BottomSheetProviderState;
///
/// let state = BottomSheetProviderState::new();
/// assert!(!state.is_open());
///
/// state.open();
/// assert!(state.is_open());
///
/// state.close();
/// assert!(!state.is_open());
/// ```
#[tessera]
pub fn bottom_sheet_provider(
    args: BottomSheetProviderArgs,
    state: BottomSheetProviderState,
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
    input_handler(keyboard_closure);

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

