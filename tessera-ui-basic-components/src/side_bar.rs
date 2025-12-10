//! A component that displays a side bar sliding in from the left.
//!
//! ## Usage
//!
//! Use to show app navigation or contextual controls.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, Px, PxPosition, remember, tessera, winit};

use crate::{
    animation,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
};

const ANIM_TIME: Duration = Duration::from_millis(300);

/// Defines the visual style of the side bar and its scrim.
///
/// The scrim is the overlay that appears behind the side bar, covering the main
/// content.
#[derive(Default, Clone, Copy)]
pub enum SideBarStyle {
    /// A translucent glass effect that blurs the content behind it.
    /// This style may be more costly and is suitable when a blurred backdrop is
    /// desired.
    Glass,
    /// A simple, semi-transparent dark overlay. This is the default style.
    #[default]
    Material,
}

/// Configuration arguments for the [`side_bar_provider`] component.
#[derive(Builder)]
pub struct SideBarProviderArgs {
    /// A callback that is invoked when the user requests to close the side bar.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape`
    /// key. The callback is expected to close the side bar.
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// The visual style used by the provider. See [`SideBarStyle`].
    #[builder(default)]
    pub style: SideBarStyle,
    /// Whether the side bar is initially open (for declarative usage).
    #[builder(default = "false")]
    pub is_open: bool,
}

#[derive(Default)]
struct SideBarStateInner {
    is_open: bool,
    timer: Option<Instant>,
}

/// Controller for [`side_bar_provider`], managing open/closed state.
///
/// This controller can be created by the application and passed to the
/// [`side_bar_provider_with_controller`]. It is used to control the visibility
/// of the side bar programmatically.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::side_bar::SideBarController;
///
/// let controller = SideBarController::new(false);
/// assert!(!controller.is_open()); // Initially closed
/// controller.open();
/// assert!(controller.is_open()); // Now open
/// controller.close();
/// assert!(!controller.is_open()); // Closed again
/// ```
pub struct SideBarController {
    inner: RwLock<SideBarStateInner>,
}

impl SideBarController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            inner: RwLock::new(SideBarStateInner {
                is_open: initial_open,
                timer: None,
            }),
        }
    }

    /// Initiates the animation to open the side bar.
    ///
    /// If the side bar is already open this has no effect. If it is currently
    /// closing, the animation will reverse direction and start opening from the
    /// current animated position.
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

    /// Initiates the animation to close the side bar.
    ///
    /// If the side bar is already closed this has no effect. If it is currently
    /// opening, the animation will reverse direction and start closing from the
    /// current animated position.
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

    /// Returns whether the side bar is currently open.
    pub fn is_open(&self) -> bool {
        self.inner.read().is_open
    }

    /// Returns whether the side bar is currently animating.
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

impl Default for SideBarController {
    fn default() -> Self {
        Self::new(false)
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
            .dispersion_height(Dp(0.0))
            .refraction_height(Dp(0.0))
            .block_input(true)
            .blur_radius(Dp(blur_radius as f64))
            .border(None)
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
            .noise_amount(0.0)
            .build()
            .expect("builder construction failed"),
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
            .expect("builder construction failed"),
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
fn snapshot_state(controller: &SideBarController) -> (bool, Option<Instant>) {
    controller.snapshot()
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

/// Place side bar if present. Extracted to reduce complexity of the parent
/// function.
fn place_side_bar_if_present(
    input: &tessera_ui::MeasureInput<'_>,
    controller_for_measure: &SideBarController,
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

    let current_is_open = controller_for_measure.is_open();
    let x = compute_side_bar_x(child_size.width, progress, current_is_open);
    input.place_child(side_bar_id, PxPosition::new(Px(x), Px(0)));
}

/// # side_bar_provider
///
/// Provides a side bar that slides in from the left, with a scrim overlay.
///
/// # Usage
///
/// Use as a top-level provider to display a navigation drawer or other
/// contextual side content.
///
/// # Parameters
///
/// - `args` — configures the side bar's style and `on_close_request` callback;
///   see [`SideBarProviderArgs`].
/// - `main_content` — a closure that renders the main UI, which is visible
///   behind the side bar.
/// - `side_bar_content` — a closure that renders the content of the side bar
///   itself.
///
/// # Examples
///
/// ```
/// use tessera_ui_basic_components::side_bar::{SideBarProviderArgsBuilder, side_bar_provider};
///
/// side_bar_provider(
///     SideBarProviderArgsBuilder::default()
///         .is_open(true)
///         .on_close_request(std::sync::Arc::new(|| {}))
///         .build()
///         .unwrap(),
///     || { /* main content */ },
///     || { /* side bar content */ },
/// );
/// ```
#[tessera]
pub fn side_bar_provider(
    args: impl Into<SideBarProviderArgs>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_bar_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SideBarProviderArgs = args.into();
    let controller = remember(|| SideBarController::new(args.is_open));

    if args.is_open != controller.is_open() {
        if args.is_open {
            controller.open();
        } else {
            controller.close();
        }
    }

    side_bar_provider_with_controller(args, controller, main_content, side_bar_content);
}

/// # side_bar_provider_with_controller
///
/// Controlled version of [`side_bar_provider`] that accepts an external
/// controller.
///
/// # Usage
///
/// Use when you need to control the side bar's open/closed state
/// programmatically via a controller.
///
/// # Parameters
///
/// - `args` — configures the side bar's style and `on_close_request` callback;
///   see [`SideBarProviderArgs`].
/// - `controller` — a [`SideBarController`] to manage the open/closed state.
/// - `main_content` — a closure that renders the main UI, which is visible
///   behind the side bar.
/// - `side_bar_content` — a closure that renders the content of the side bar
///   itself.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::side_bar::{
///     SideBarController, SideBarProviderArgsBuilder, side_bar_provider_with_controller,
/// };
///
/// #[tessera]
/// fn foo() {
///     let controller = remember(|| SideBarController::new(false));
///     side_bar_provider_with_controller(
///         SideBarProviderArgsBuilder::default()
///             .on_close_request(Arc::new(|| {}))
///             .build()
///             .unwrap(),
///         controller.clone(),
///         || { /* main content */ },
///         || { /* side bar content */ },
///     );
/// }
/// ```
#[tessera]
pub fn side_bar_provider_with_controller(
    args: impl Into<SideBarProviderArgs>,
    controller: Arc<SideBarController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_bar_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SideBarProviderArgs = args.into();

    // Render main content first.
    main_content();

    // Snapshot state once to minimize locking overhead.
    let (is_open, timer_opt) = snapshot_state(&controller);

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
    let controller_for_measure = controller.clone();
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
        place_side_bar_if_present(input, &controller_for_measure, progress);

        // Return the main content size (best-effort; unwrap used above to satisfy
        // closure type).
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
                        top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .tint_color(Color::new(0.6, 0.8, 1.0, 0.3))
                    .width(DimensionValue::from(Dp(250.0)))
                    .height(tessera_ui::DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .blur_radius(Dp(10.0))
                    .padding(Dp(16.0))
                    .block_input(true)
                    .build()
                    .expect("builder construction failed"),
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
                        top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .block_input(true)
                    .build()
                    .expect("builder construction failed"),
                content,
            );
        }
    }
}
