//! Side sheet provider - slide supporting content in from the screen edge.
//!
//! ## Usage
//!
//! Show filters, details, or secondary tasks without leaving the current
//! screen.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px,
    PxPosition, State,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    remember, tessera, use_context, winit,
};

use crate::{
    animation,
    modifier::ModifierExt,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, surface},
    theme::MaterialTheme,
};

const ANIM_TIME: Duration = Duration::from_millis(300);
const SCRIM_ALPHA: f32 = 0.32;
const MAX_SHEET_WIDTH: Dp = Dp(360.0);
const CORNER_RADIUS: Dp = Dp(16.0);
const MODAL_ELEVATION: Dp = Dp(1.0);

/// Defines how the side sheet behaves relative to the main content.
#[derive(Default, Clone, Copy, PartialEq)]
enum SideSheetType {
    /// A modal sheet that blocks interaction with content behind it.
    #[default]
    Modal,
    /// A standard sheet that does not block interaction behind it.
    Standard,
}

/// Defines which edge the sheet is attached to.
#[derive(Default, Clone, Copy, PartialEq)]
pub enum SideSheetPosition {
    /// Attach to the start (left) edge.
    #[default]
    Start,
    /// Attach to the end (right) edge.
    End,
}

/// Configuration arguments for side sheet providers.
#[derive(Setters)]
pub struct SideSheetProviderArgs {
    /// A callback invoked when the user requests to close the sheet.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape`
    /// key. The callback is responsible for closing the sheet.
    #[setters(skip)]
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// Which edge the sheet is attached to. See [`SideSheetPosition`].
    pub position: SideSheetPosition,
    /// Whether the sheet is initially open (for declarative usage).
    pub is_open: bool,
}

impl SideSheetProviderArgs {
    /// Create args with a required close-request callback.
    pub fn new(on_close_request: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            on_close_request: Arc::new(on_close_request),
            position: SideSheetPosition::default(),
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

/// Controller for side sheet providers, managing open/closed state.
///
/// This controller can be created by the application and passed to the
/// [`modal_side_sheet_provider_with_controller`] or
/// [`standard_side_sheet_provider_with_controller`]. It is used to control the
/// visibility of the sheet programmatically.
///
/// # Example
///
/// ```
/// use tessera_components::side_sheet::SideSheetController;
///
/// let mut controller = SideSheetController::new(false);
/// assert!(!controller.is_open());
/// controller.open();
/// assert!(controller.is_open());
/// controller.close();
/// assert!(!controller.is_open());
/// ```
#[derive(Clone)]
pub struct SideSheetController {
    is_open: bool,
    timer: Option<Instant>,
}

impl SideSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            timer: None,
        }
    }

    /// Initiates the animation to open the side sheet.
    ///
    /// If the sheet is already open this has no effect. If it is currently
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

    /// Initiates the animation to close the side sheet.
    ///
    /// If the sheet is already closed this has no effect. If it is currently
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

    /// Returns whether the side sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the side sheet is currently animating.
    pub fn is_animating(&self) -> bool {
        self.timer.is_some_and(|t| t.elapsed() < ANIM_TIME)
    }

    fn snapshot(&self) -> (bool, Option<Instant>) {
        (self.is_open, self.timer)
    }
}

impl Default for SideSheetController {
    fn default() -> Self {
        Self::new(false)
    }
}

fn sheet_shape(position: SideSheetPosition) -> Shape {
    let rounded = RoundedCorner::manual(CORNER_RADIUS, 3.0);
    match position {
        SideSheetPosition::Start => Shape::RoundedRectangle {
            top_left: RoundedCorner::ZERO,
            top_right: rounded,
            bottom_right: rounded,
            bottom_left: RoundedCorner::ZERO,
        },
        SideSheetPosition::End => Shape::RoundedRectangle {
            top_left: rounded,
            top_right: RoundedCorner::ZERO,
            bottom_right: RoundedCorner::ZERO,
            bottom_left: rounded,
        },
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

/// Compute scrim alpha for modal style.
fn scrim_alpha_for(progress: f32, is_open: bool) -> f32 {
    if is_open {
        progress * SCRIM_ALPHA
    } else {
        SCRIM_ALPHA * (1.0 - progress)
    }
}

/// Compute X position for side sheet placement.
fn compute_side_sheet_x(
    parent_width: Px,
    sheet_width: Px,
    progress: f32,
    is_open: bool,
    position: SideSheetPosition,
) -> i32 {
    let parent = parent_width.0 as f32;
    let sheet = sheet_width.0 as f32;
    let (closed_x, open_x) = match position {
        SideSheetPosition::Start => (-sheet, 0.0),
        SideSheetPosition::End => (parent, parent - sheet),
    };
    let x = if is_open {
        closed_x + (open_x - closed_x) * progress
    } else {
        open_x + (closed_x - open_x) * progress
    };
    x as i32
}

fn render_modal_scrim(on_close_request: Arc<dyn Fn() + Send + Sync>, progress: f32, is_open: bool) {
    let scrim_alpha = scrim_alpha_for(progress, is_open);
    let scrim_color = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .scrim;
    surface(
        SurfaceArgs::default()
            .style(scrim_color.with_alpha(scrim_alpha).into())
            .on_click_shared(on_close_request)
            .modifier(Modifier::new().fill_max_size())
            .block_input(true),
        || {},
    );
}

fn render_standard_scrim() {
    surface(
        SurfaceArgs::default()
            .style(Color::TRANSPARENT.into())
            .modifier(Modifier::new().fill_max_size()),
        || {},
    );
}

/// Render scrim according to configured type.
fn render_scrim(
    sheet_type: SideSheetType,
    on_close_request: Arc<dyn Fn() + Send + Sync>,
    progress: f32,
    is_open: bool,
) {
    match sheet_type {
        SideSheetType::Modal => render_modal_scrim(on_close_request, progress, is_open),
        SideSheetType::Standard => render_standard_scrim(),
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

/// Place side sheet if present. Extracted to reduce complexity of the parent
/// function.
fn place_side_sheet_if_present(
    input: &LayoutInput<'_>,
    output: &mut LayoutOutput<'_>,
    is_open: bool,
    progress: f32,
    position: SideSheetPosition,
) {
    if input.children_ids().len() <= 2 {
        return;
    }

    let side_sheet_id = input.children_ids()[2];
    let parent_width = input.parent_constraint().width().get_max().unwrap_or(Px(0));
    let parent_height = input
        .parent_constraint()
        .height()
        .get_max()
        .unwrap_or(Px(0));

    let max_width_px = MAX_SHEET_WIDTH.to_px();
    let sheet_width = if parent_width > max_width_px {
        max_width_px
    } else {
        parent_width
    };

    let constraint = Constraint {
        width: DimensionValue::Fixed(sheet_width),
        height: DimensionValue::Fixed(parent_height),
    };

    let child_size = match input.measure_child(side_sheet_id, &constraint) {
        Ok(s) => s,
        Err(_) => return,
    };

    let x = compute_side_sheet_x(parent_width, child_size.width, progress, is_open, position);
    output.place_child(side_sheet_id, PxPosition::new(Px(x), Px(0)));
}

/// # modal_side_sheet_provider
///
/// Show a modal side sheet that blocks interaction with the main content.
///
/// ## Usage
///
/// Present filters or details that require an explicit dismissal.
///
/// ## Parameters
///
/// - `args` - configuration for the sheet's behavior; see
///   [`SideSheetProviderArgs`].
/// - `main_content` - closure that renders the main UI behind the sheet.
/// - `side_sheet_content` - closure that renders the sheet content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::side_sheet::{SideSheetProviderArgs, modal_side_sheet_provider};
/// let args = SideSheetProviderArgs::new(|| {}).is_open(true);
/// assert!(args.is_open);
/// material_theme(MaterialTheme::default, || {
///     modal_side_sheet_provider(
///         args,
///         || { /* main content */ },
///         || { /* side sheet content */ },
///     );
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn modal_side_sheet_provider(
    args: impl Into<SideSheetProviderArgs>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    side_sheet_provider_inner(SideSheetType::Modal, args, main_content, side_sheet_content);
}

/// # modal_side_sheet_provider_with_controller
///
/// Controlled version of [`modal_side_sheet_provider`] that accepts an
/// external controller.
///
/// ## Usage
///
/// Use when you need to control the modal sheet state programmatically.
///
/// ## Parameters
///
/// - `args` - configuration for the sheet's behavior; see
///   [`SideSheetProviderArgs`].
/// - `controller` - a [`SideSheetController`] used to open and close the sheet.
/// - `main_content` - closure that renders the main UI behind the sheet.
/// - `side_sheet_content` - closure that renders the sheet content.
///
/// ## Examples
///
/// ```
/// use tessera_components::side_sheet::{
///     SideSheetController, SideSheetProviderArgs, modal_side_sheet_provider_with_controller,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let controller = remember(|| SideSheetController::new(false));
///     assert!(!controller.with(|c| c.is_open()));
///     controller.with_mut(|c| c.open());
///     assert!(controller.with(|c| c.is_open()));
///     material_theme(MaterialTheme::default, || {
///         modal_side_sheet_provider_with_controller(
///             SideSheetProviderArgs::new(|| {}),
///             controller,
///             || { /* main content */ },
///             || { /* side sheet content */ },
///         );
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn modal_side_sheet_provider_with_controller(
    args: impl Into<SideSheetProviderArgs>,
    controller: State<SideSheetController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    side_sheet_provider_with_controller_inner(
        SideSheetType::Modal,
        args,
        controller,
        main_content,
        side_sheet_content,
    );
}

/// # standard_side_sheet_provider
///
/// Show a standard side sheet that keeps the main content interactive.
///
/// ## Usage
///
/// Present supplementary tools or information that can remain open while the
/// user continues interacting with the main UI.
///
/// ## Parameters
///
/// - `args` - configuration for the sheet's behavior; see
///   [`SideSheetProviderArgs`].
/// - `main_content` - closure that renders the main UI behind the sheet.
/// - `side_sheet_content` - closure that renders the sheet content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::side_sheet::{SideSheetProviderArgs, standard_side_sheet_provider};
/// let args = SideSheetProviderArgs::new(|| {}).is_open(true);
/// assert!(args.is_open);
/// material_theme(MaterialTheme::default, || {
///     standard_side_sheet_provider(
///         args,
///         || { /* main content */ },
///         || { /* side sheet content */ },
///     );
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn standard_side_sheet_provider(
    args: impl Into<SideSheetProviderArgs>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    side_sheet_provider_inner(
        SideSheetType::Standard,
        args,
        main_content,
        side_sheet_content,
    );
}

/// # standard_side_sheet_provider_with_controller
///
/// Controlled version of [`standard_side_sheet_provider`] that accepts an
/// external controller.
///
/// ## Usage
///
/// Use when you need to control the standard sheet state programmatically.
///
/// ## Parameters
///
/// - `args` - configuration for the sheet's behavior; see
///   [`SideSheetProviderArgs`].
/// - `controller` - a [`SideSheetController`] used to open and close the sheet.
/// - `main_content` - closure that renders the main UI behind the sheet.
/// - `side_sheet_content` - closure that renders the sheet content.
///
/// ## Examples
///
/// ```
/// use tessera_components::side_sheet::{
///     SideSheetController, SideSheetProviderArgs, standard_side_sheet_provider_with_controller,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let controller = remember(|| SideSheetController::new(false));
///     assert!(!controller.with(|c| c.is_open()));
///     controller.with_mut(|c| c.open());
///     assert!(controller.with(|c| c.is_open()));
///     material_theme(MaterialTheme::default, || {
///         standard_side_sheet_provider_with_controller(
///             SideSheetProviderArgs::new(|| {}),
///             controller,
///             || { /* main content */ },
///             || { /* side sheet content */ },
///         );
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn standard_side_sheet_provider_with_controller(
    args: impl Into<SideSheetProviderArgs>,
    controller: State<SideSheetController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    side_sheet_provider_with_controller_inner(
        SideSheetType::Standard,
        args,
        controller,
        main_content,
        side_sheet_content,
    );
}

#[tessera]
fn side_sheet_provider_inner(
    sheet_type: SideSheetType,
    args: impl Into<SideSheetProviderArgs>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SideSheetProviderArgs = args.into();
    let controller = remember(|| SideSheetController::new(args.is_open));

    if args.is_open != controller.with(|c| c.is_open()) {
        if args.is_open {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }

    side_sheet_provider_with_controller_inner(
        sheet_type,
        args,
        controller,
        main_content,
        side_sheet_content,
    );
}

#[tessera]
fn side_sheet_provider_with_controller_inner(
    sheet_type: SideSheetType,
    args: impl Into<SideSheetProviderArgs>,
    controller: State<SideSheetController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    side_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SideSheetProviderArgs = args.into();

    main_content();

    let (is_open, timer_opt) = controller.with(|c| c.snapshot());

    if !(is_open || timer_opt.is_some_and(|t| t.elapsed() < ANIM_TIME)) {
        return;
    }

    let progress = calc_progress_from_timer(timer_opt.as_ref());

    render_scrim(sheet_type, args.on_close_request.clone(), progress, is_open);

    input_handler(make_keyboard_closure(args.on_close_request.clone()));

    side_sheet_content_wrapper(sheet_type, args.position, side_sheet_content);

    layout(SideSheetLayout {
        progress,
        is_open,
        position: args.position,
    });
}

#[tessera]
fn side_sheet_content_wrapper(
    sheet_type: SideSheetType,
    position: SideSheetPosition,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let container_color = match sheet_type {
        SideSheetType::Modal => scheme.surface_container_low,
        SideSheetType::Standard => scheme.surface,
    };
    let surface_args = match sheet_type {
        SideSheetType::Modal => SurfaceArgs::default().elevation(MODAL_ELEVATION),
        SideSheetType::Standard => SurfaceArgs::default(),
    };

    surface(
        surface_args
            .style(container_color.into())
            .shape(sheet_shape(position))
            .modifier(Modifier::new().fill_max_height())
            .block_input(true),
        move || {
            Modifier::new().padding_all(Dp(16.0)).run(content);
        },
    );
}

#[derive(Clone, PartialEq)]
struct SideSheetLayout {
    progress: f32,
    is_open: bool,
    position: SideSheetPosition,
}

impl LayoutSpec for SideSheetLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let main_content_id = input.children_ids()[0];
        let main_content_size = input.measure_child_in_parent_constraint(main_content_id)?;
        output.place_child(main_content_id, PxPosition::new(Px(0), Px(0)));

        if input.children_ids().len() > 1 {
            let scrim_id = input.children_ids()[1];
            input.measure_child_in_parent_constraint(scrim_id)?;
            output.place_child(scrim_id, PxPosition::new(Px(0), Px(0)));
        }

        place_side_sheet_if_present(input, output, self.is_open, self.progress, self.position);

        Ok(main_content_size)
    }
}
