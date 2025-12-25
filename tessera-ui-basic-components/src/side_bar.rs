//! A component that displays a side bar sliding in from the left.
//!
//! ## Usage
//!
//! Use to show app navigation or contextual controls.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{Color, Dp, Modifier, Px, PxPosition, State, remember, tessera, winit};

use crate::{
    animation,
    fluid_glass::{FluidGlassArgs, fluid_glass},
    modifier::ModifierExt,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, surface},
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
#[derive(Setters)]
pub struct SideBarProviderArgs {
    /// A callback that is invoked when the user requests to close the side bar.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape`
    /// key. The callback is expected to close the side bar.
    #[setters(skip)]
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// The visual style used by the provider. See [`SideBarStyle`].
    pub style: SideBarStyle,
    /// Whether the side bar is initially open (for declarative usage).
    pub is_open: bool,
}

impl SideBarProviderArgs {
    /// Create args with a required close-request callback.
    pub fn new(on_close_request: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            on_close_request: Arc::new(on_close_request),
            style: SideBarStyle::default(),
            is_open: false,
        }
    }

    /// Set the close-request callback.
    pub fn on_close_request<F>(mut self, on_close_request: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close_request = Arc::new(on_close_request);
        self
    }

    /// Set the close-request callback using a shared callback.
    pub fn on_close_request_shared(
        mut self,
        on_close_request: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        self.on_close_request = on_close_request;
        self
    }
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
/// let mut controller = SideBarController::new(false);
/// assert!(!controller.is_open()); // Initially closed
/// controller.open();
/// assert!(controller.is_open()); // Now open
/// controller.close();
/// assert!(!controller.is_open()); // Closed again
/// ```
#[derive(Clone)]
pub struct SideBarController {
    is_open: bool,
    timer: Option<Instant>,
}

impl SideBarController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            timer: None,
        }
    }

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

    /// Returns whether the side bar is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the side bar is currently animating.
    pub fn is_animating(&self) -> bool {
        self.timer.is_some_and(|t| t.elapsed() < ANIM_TIME)
    }

    fn snapshot(&self) -> (bool, Option<Instant>) {
        (self.is_open, self.timer)
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
        FluidGlassArgs::default()
            .on_click_shared(args.on_close_request.clone())
            .tint_color(Color::TRANSPARENT)
            .modifier(Modifier::new().fill_max_size())
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
            .noise_amount(0.0),
        || {},
    );
}

fn render_material_scrim(args: &SideBarProviderArgs, progress: f32, is_open: bool) {
    // Material scrim: compute alpha and render a simple dark surface.
    let scrim_alpha = scrim_alpha_for(progress, is_open);
    surface(
        SurfaceArgs::default()
            .style(Color::BLACK.with_alpha(scrim_alpha).into())
            .on_click_shared(args.on_close_request.clone())
            .modifier(Modifier::new().fill_max_size())
            .block_input(true),
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
    controller: State<SideBarController>,
    progress: f32,
) {
    if input.children_ids.len() <= 2 {
        return;
    }

    let side_bar_id = input.children_ids[2];

    let child_size = match input.measure_child_in_parent_constraint(side_bar_id) {
        Ok(s) => s,
        Err(_) => return,
    };

    let current_is_open = controller.with(|c| c.is_open());
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
/// use tessera_ui_basic_components::side_bar::{SideBarProviderArgs, side_bar_provider};
///
/// side_bar_provider(
///     SideBarProviderArgs::new(|| {}).is_open(true),
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

    if args.is_open != controller.with(|c| c.is_open()) {
        if args.is_open {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
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
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::side_bar::{
///     SideBarController, SideBarProviderArgs, side_bar_provider_with_controller,
/// };
///
/// #[tessera]
/// fn foo() {
///     let controller = remember(|| SideBarController::new(false));
///     side_bar_provider_with_controller(
///         SideBarProviderArgs::new(|| {}),
///         controller,
///         || { /* main content */ },
///         || { /* side bar content */ },
///     );
/// }
/// ```
#[tessera]
pub fn side_bar_provider_with_controller(
    args: impl Into<SideBarProviderArgs>,
    controller: State<SideBarController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_bar_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SideBarProviderArgs = args.into();

    // Render main content first.
    main_content();

    // Snapshot state once to minimize locking overhead.
    let (is_open, timer_opt) = controller.with(|c| c.snapshot());

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
    let measure_closure = Box::new(move |input: &tessera_ui::MeasureInput<'_>| {
        // Place main content at origin.
        let main_content_id = input.children_ids[0];
        let main_content_size = input.measure_child_in_parent_constraint(main_content_id)?;
        input.place_child(main_content_id, PxPosition::new(Px(0), Px(0)));

        // Place scrim (if present) covering the whole parent.
        if input.children_ids.len() > 1 {
            let scrim_id = input.children_ids[1];
            input.measure_child_in_parent_constraint(scrim_id)?;
            input.place_child(scrim_id, PxPosition::new(Px(0), Px(0)));
        }

        // Place side bar (if present) using extracted helper.
        place_side_bar_if_present(input, controller, progress);

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
                FluidGlassArgs::default()
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .tint_color(Color::new(0.6, 0.8, 1.0, 0.3))
                    .modifier(Modifier::new().width(Dp(250.0)).fill_max_height())
                    .blur_radius(Dp(10.0))
                    .padding(Dp(16.0))
                    .block_input(true),
                content,
            );
        }
        SideBarStyle::Material => {
            surface(
                SurfaceArgs::default()
                    .style(Color::new(0.9, 0.9, 0.9, 1.0).into())
                    .modifier(Modifier::new().width(Dp(250.0)).fill_max_height())
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .block_input(true),
                move || {
                    Modifier::new().padding_all(Dp(16.0)).run(content);
                },
            );
        }
    }
}
