//! A component that displays content sliding up from the bottom of the screen.
//!
//! ## Usage
//!
//! Used to show contextual information or actions in a modal sheet.
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp,
    MeasurementError, Modifier, PressKeyEventType, Px, PxPosition, RenderSlot, State,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    animation,
    column::{ColumnArgs, column},
    fluid_glass::{FluidGlassArgs, fluid_glass},
    modifier::ModifierExt,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    theme::MaterialTheme,
};

const ANIM_TIME: Duration = Duration::from_millis(300);

/// Defines the visual style of the bottom sheet's scrim.
///
/// The scrim is the overlay that appears behind the bottom sheet, covering the
/// main content.
#[derive(Default, Clone, PartialEq, Copy)]
pub enum BottomSheetStyle {
    /// A translucent glass effect that blurs the content behind it.
    /// This style is more resource-intensive and may not be suitable for all
    /// targets.
    Glass,
    /// A simple, semi-transparent dark overlay. This is the default style.
    #[default]
    Material,
}

/// Configuration arguments for the [`bottom_sheet_provider`].
#[derive(Clone, PartialEq, Setters)]
pub struct BottomSheetProviderArgs {
    /// A callback that is invoked when the user requests to close the sheet.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape`
    /// key. The callback is responsible for closing the sheet.
    #[setters(skip)]
    pub on_close_request: Callback,
    /// The visual style of the scrim. See [`BottomSheetStyle`].
    pub style: BottomSheetStyle,
    /// Whether the sheet is initially open (for declarative usage).
    pub is_open: bool,
    /// Optional external controller for programmatic open/close.
    #[setters(skip)]
    pub controller: Option<State<BottomSheetController>>,
    /// Optional main content rendered behind the sheet.
    #[setters(skip)]
    pub main_content: Option<RenderSlot>,
    /// Optional content rendered inside the bottom sheet.
    #[setters(skip)]
    pub bottom_sheet_content: Option<RenderSlot>,
}

impl BottomSheetProviderArgs {
    /// Create args with a required close-request callback.
    pub fn new(on_close_request: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            on_close_request: Callback::new(on_close_request),
            style: BottomSheetStyle::default(),
            is_open: false,
            controller: None,
            main_content: None,
            bottom_sheet_content: None,
        }
    }

    /// Set the close-request callback.
    pub fn on_close_request<F>(mut self, on_close_request: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close_request = Callback::new(on_close_request);
        self
    }

    /// Set the close-request callback using a shared callback.
    pub fn on_close_request_shared(mut self, on_close_request: impl Into<Callback>) -> Self {
        self.on_close_request = on_close_request.into();
        self
    }

    /// Sets an external bottom sheet controller.
    pub fn controller(mut self, controller: State<BottomSheetController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Sets the main content slot.
    pub fn main_content<F>(mut self, main_content: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.main_content = Some(RenderSlot::new(main_content));
        self
    }

    /// Sets the main content slot using a shared render slot.
    pub fn main_content_shared(mut self, main_content: impl Into<RenderSlot>) -> Self {
        self.main_content = Some(main_content.into());
        self
    }

    /// Sets the bottom sheet content slot.
    pub fn bottom_sheet_content<F>(mut self, bottom_sheet_content: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.bottom_sheet_content = Some(RenderSlot::new(bottom_sheet_content));
        self
    }

    /// Sets the bottom sheet content slot using a shared render slot.
    pub fn bottom_sheet_content_shared(
        mut self,
        bottom_sheet_content: impl Into<RenderSlot>,
    ) -> Self {
        self.bottom_sheet_content = Some(bottom_sheet_content.into());
        self
    }
}

/// Controller for [`bottom_sheet_provider`], managing open/closed state.
///
/// This controller can be created by the application and passed through
/// [`BottomSheetProviderArgs::controller`]. It is used to control the
/// visibility of the sheet programmatically.
#[derive(Clone, PartialEq)]
pub struct BottomSheetController {
    is_open: bool,
    timer: Option<Instant>,
    is_dragging: bool,
    drag_offset: f32,
    drag_start_y: f32,
}

impl BottomSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            timer: None,
            is_dragging: false,
            drag_offset: 0.0,
            drag_start_y: 0.0,
        }
    }

    /// Initiates the animation to open the bottom sheet.
    ///
    /// If the sheet is already open, this has no effect. If the sheet is
    /// currently closing, it will reverse direction and start opening from
    /// its current position.
    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            self.drag_offset = 0.0;
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
    /// If the sheet is already closed, this has no effect. If the sheet is
    /// currently opening, it will reverse direction and start closing from
    /// its current position.
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

    /// Returns whether the sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the sheet is currently animating in either direction.
    pub fn is_animating(&self) -> bool {
        self.timer.is_some_and(|t| t.elapsed() < ANIM_TIME)
    }

    fn snapshot(&self) -> (bool, Option<Instant>, f32) {
        (self.is_open, self.timer, self.drag_offset)
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    fn update_drag_offset(&mut self, offset: f32) {
        self.drag_offset = offset;
    }

    fn get_drag_offset(&self) -> f32 {
        self.drag_offset
    }

    fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    fn set_drag_start_y(&mut self, y: f32) {
        self.drag_start_y = y;
    }

    fn get_drag_start_y(&self) -> f32 {
        self.drag_start_y
    }
}

impl Default for BottomSheetController {
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
        progress * 0.32
    } else {
        0.32 * (1.0 - progress)
    }
}

/// Compute Y position for bottom sheet placement.
fn compute_bottom_sheet_y(
    parent_height: Px,
    child_height: Px,
    progress: f32,
    is_open: bool,
    drag_offset: f32,
) -> i32 {
    let parent = parent_height.0 as f32;
    let child = child_height.0 as f32;
    let y = if is_open {
        parent - child * progress
    } else {
        parent - child * (1.0 - progress)
    };
    (y + drag_offset) as i32
}

fn render_glass_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
    // Glass scrim: compute blur radius and render using fluid_glass.
    let max_blur_radius = 5.0;
    let blur_radius = blur_radius_for(progress, is_open, max_blur_radius);
    fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
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
    ));
}

fn render_material_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
    // Material scrim: compute alpha and render a simple dark surface.
    let scrim_alpha = scrim_alpha_for(progress, is_open);
    let scrim_color = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .scrim;
    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .style(scrim_color.with_alpha(scrim_alpha).into())
            .on_click_shared(args.on_close_request.clone())
            .modifier(Modifier::new().fill_max_size())
            .block_input(true),
        || {},
    ));
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

/// Create the keyboard handler closure used to close the sheet on Escape.
fn make_keyboard_closure(
    on_close: Callback,
) -> Box<dyn Fn(tessera_ui::InputHandlerInput<'_>) + Send + Sync> {
    Box::new(move |input: tessera_ui::InputHandlerInput<'_>| {
        for event in input.keyboard_events.drain(..) {
            if event.state == winit::event::ElementState::Pressed
                && let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                    event.physical_key
            {
                on_close.call();
            }
        }
    })
}

/// Handle drag gestures on the bottom sheet.
fn handle_drag_gestures(
    controller: State<BottomSheetController>,
    input: &mut tessera_ui::InputHandlerInput<'_>,
    on_close: &Callback,
) {
    let mut is_dragging = controller.with(|c| c.is_dragging());
    let drag_offset = controller.with(|c| c.get_drag_offset());

    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(PressKeyEventType::Left) => {
                if let Some(pos) = input.cursor_position_rel {
                    is_dragging = true;
                    controller.with_mut(|c| {
                        c.set_dragging(true);
                        c.set_drag_start_y(pos.y.0 as f32);
                    });
                }
            }
            CursorEventContent::Released(PressKeyEventType::Left) => {
                if is_dragging {
                    is_dragging = false;
                    controller.with_mut(|c| c.set_dragging(false));

                    if drag_offset > 100.0 {
                        on_close.call();
                    } else {
                        controller.with_mut(|c| c.update_drag_offset(0.0));
                    }
                }
            }
            _ => {}
        }
    }

    if is_dragging && let Some(pos) = input.cursor_position_rel {
        let current_y = pos.y.0 as f32;
        let start_y = controller.with(|c| c.get_drag_start_y());
        let delta = current_y - start_y;

        // Accumulate delta since component moves with drag.
        let new_offset = (drag_offset + delta).max(0.0);
        if (new_offset - drag_offset).abs() > 0.001 {
            controller.with_mut(|c| c.update_drag_offset(new_offset));
        }
    }
}

/// Place bottom sheet if present. Extracted to reduce complexity of the parent
/// function.
fn place_bottom_sheet_if_present(
    input: &LayoutInput<'_>,
    output: &mut LayoutOutput<'_>,
    is_open: bool,
    drag_offset: f32,
    progress: f32,
) {
    if input.children_ids().len() <= 2 {
        return;
    }

    let bottom_sheet_id = input.children_ids()[2];

    let parent_width = input.parent_constraint().width().get_max().unwrap_or(Px(0));
    let parent_height = input
        .parent_constraint()
        .height()
        .get_max()
        .unwrap_or(Px(0));

    // M3 Spec: Max width 640dp.
    let max_width_px = Dp(640.0).to_px();
    let is_large_screen = parent_width >= max_width_px;

    let sheet_width = if is_large_screen {
        max_width_px
    } else {
        parent_width
    };

    // M3 Spec: Top margin 56dp or 72dp.
    let top_margin = if is_large_screen {
        Dp(56.0).to_px()
    } else {
        Dp(72.0).to_px()
    };
    let max_height = (parent_height - top_margin).max(Px(0));

    let constraint = Constraint {
        width: DimensionValue::Fixed(sheet_width),
        height: DimensionValue::Wrap {
            min: None,
            max: Some(max_height),
        },
    };

    let child_size = match input.measure_child(bottom_sheet_id, &constraint) {
        Ok(s) => s,
        Err(_) => return,
    };

    let y = compute_bottom_sheet_y(
        parent_height,
        child_size.height,
        progress,
        is_open,
        drag_offset,
    );

    let x = if is_large_screen {
        (parent_width - child_size.width) / 2
    } else {
        Px(0)
    };

    output.place_child(bottom_sheet_id, PxPosition::new(x, Px(y)));
}
#[derive(PartialEq, Clone)]
struct DragHandlerArgs {
    controller: State<BottomSheetController>,
    on_close: Callback,
    child: RenderSlot,
}

#[tessera]
fn drag_handler_node(args: &DragHandlerArgs) {
    let controller = args.controller;
    let on_close = args.on_close.clone();
    let child = args.child.clone();

    input_handler(move |mut input| {
        handle_drag_gestures(controller, &mut input, &on_close);
    });

    child.render();
}

fn render_content(
    style: BottomSheetStyle,
    bottom_sheet_content: RenderSlot,
    controller: State<BottomSheetController>,
    on_close: Callback,
) {
    let bottom_sheet_content = bottom_sheet_content.clone();
    let content_wrapper = move || {
        let bottom_sheet_content = bottom_sheet_content.clone();
        let child_once: Box<dyn FnOnce() + Send + Sync> = Box::new(move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .cross_axis_alignment(CrossAxisAlignment::Center),
                |scope| {
                    scope.child(|| {
                        spacer(&crate::spacer::SpacerArgs::new(
                            Modifier::new().height(Dp(22.0)),
                        ));
                    });
                    scope.child(|| {
                        surface(&crate::surface::SurfaceArgs::with_child(
                            SurfaceArgs::default()
                                .style(
                                    use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .on_surface_variant
                                        .with_alpha(0.4)
                                        .into(),
                                )
                                .shape(Shape::capsule())
                                .modifier(Modifier::new().size(Dp(32.0), Dp(4.0))),
                            || {},
                        ));
                    });
                    scope.child(|| {
                        spacer(&crate::spacer::SpacerArgs::new(
                            Modifier::new().height(Dp(22.0)),
                        ));
                    });

                    let bottom_sheet_content = bottom_sheet_content.clone();
                    scope.child(move || {
                        bottom_sheet_content.render();
                    });
                },
            );
        });
        let child_slot = Arc::new(Mutex::new(Some(child_once)));
        let replayable_child = {
            let child_slot = Arc::clone(&child_slot);
            RenderSlot::new(move || {
                if let Some(child_once) = child_slot
                    .lock()
                    .expect("drag_handler child mutex poisoned")
                    .take()
                {
                    child_once();
                }
            })
        };

        let drag_handler_args = DragHandlerArgs {
            controller,
            on_close: on_close.clone(),
            child: replayable_child,
        };
        drag_handler_node(&drag_handler_args);
    };
    match style {
        BottomSheetStyle::Glass => {
            fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
                FluidGlassArgs::default()
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .tint_color(Color::WHITE.with_alpha(0.4))
                    .modifier(Modifier::new().fill_max_width())
                    .refraction_amount(32.0)
                    .blur_radius(Dp(5.0))
                    .block_input(true),
                content_wrapper,
            ));
        }
        BottomSheetStyle::Material => {
            surface(&crate::surface::SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .surface_container_low
                            .into(),
                    )
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .modifier(Modifier::new().fill_max_width())
                    .block_input(true),
                content_wrapper,
            ));
        }
    }
}

/// # bottom_sheet_provider
///
/// Provides a modal bottom sheet for contextual actions or information.
///
/// # Usage
///
/// Show contextual menus, supplemental information, or simple forms without
/// navigating away from the main screen.
///
/// ## Parameters
///
/// - `args` — configuration for the sheet's appearance and behavior; see
///   [`BottomSheetProviderArgs`].
///
/// # Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::bottom_sheet::{BottomSheetProviderArgs, bottom_sheet_provider};
/// # use tessera_components::theme::{MaterialTheme, MaterialThemeProviderArgs, material_theme};
///
/// # material_theme(&MaterialThemeProviderArgs::new(
/// #     MaterialTheme::default,
/// #     || {
/// bottom_sheet_provider(
///     &BottomSheetProviderArgs::new(|| {})
///         .is_open(true)
///         .main_content(|| { /* main content */ })
///         .bottom_sheet_content(|| { /* bottom sheet content */ }),
/// );
/// #     },
/// # ));
/// # }
/// # component();
/// ```
#[tessera]
pub fn bottom_sheet_provider(args: &BottomSheetProviderArgs) {
    let provider_args = args.clone();
    let main_content = provider_args
        .main_content
        .clone()
        .unwrap_or_else(|| RenderSlot::new(|| {}));
    let bottom_sheet_content = provider_args
        .bottom_sheet_content
        .clone()
        .unwrap_or_else(|| RenderSlot::new(|| {}));
    let controller = provider_args
        .controller
        .unwrap_or_else(|| remember(|| BottomSheetController::new(provider_args.is_open)));

    // In controlled mode (external controller provided), do not override
    // controller state from `is_open`.
    if provider_args.controller.is_none() {
        let current_open = controller.with(|c| c.is_open());
        if provider_args.is_open != current_open {
            if provider_args.is_open {
                controller.with_mut(|c| c.open());
            } else {
                controller.with_mut(|c| c.close());
            }
        }
    }

    main_content.render();

    // Snapshot state to minimize locking overhead.
    let (is_open, timer_opt, drag_offset) = controller.with(|c| c.snapshot());
    let is_animating = timer_opt.is_some_and(|t| t.elapsed() < ANIM_TIME);
    if is_animating {
        let controller_for_frame = controller;
        receive_frame_nanos(move |_| {
            let is_animating = controller_for_frame.with_mut(|controller| {
                let (_, timer_opt, _) = controller.snapshot();
                timer_opt.is_some_and(|timer| timer.elapsed() < ANIM_TIME)
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    if !(is_open || is_animating) {
        return;
    }

    let on_close_for_keyboard = provider_args.on_close_request.clone();
    let progress = calc_progress_from_timer(timer_opt.as_ref());

    render_scrim(&provider_args, progress, is_open);

    let keyboard_closure = make_keyboard_closure(on_close_for_keyboard);
    input_handler(keyboard_closure);

    render_content(
        provider_args.style,
        bottom_sheet_content,
        controller,
        provider_args.on_close_request.clone(),
    );

    layout(BottomSheetLayout {
        progress,
        is_open,
        drag_offset,
    });
}

#[derive(Clone, PartialEq)]
struct BottomSheetLayout {
    progress: f32,
    is_open: bool,
    drag_offset: f32,
}

impl LayoutSpec for BottomSheetLayout {
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

        place_bottom_sheet_if_present(input, output, self.is_open, self.drag_offset, self.progress);

        Ok(main_content_size)
    }
}
