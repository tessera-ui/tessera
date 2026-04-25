//! Side sheet provider - slide supporting content in from the screen edge.
//!
//! ## Usage
//!
//! Show filters, details, or secondary tasks without leaving the current
//! screen.
use std::time::Duration;

use tessera_ui::{
    Callback, CallbackWith, Color, Constraint, Dp, FocusScopeNode, FocusTraversalPolicy,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot, State,
    current_frame_nanos,
    layout::{LayoutPolicy, MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    animation,
    modifier::{ModifierExt, with_keyboard_input},
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    },
    shape_def::{RoundedCorner, Shape},
    surface::surface,
    theme::MaterialTheme,
};

const ANIM_TIME: Duration = Duration::from_millis(300);
const SCRIM_ALPHA: f32 = 0.32;
const MAX_SHEET_WIDTH: Dp = Dp(250.0);
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

/// Controller for side sheet providers, managing open/closed state.
///
/// This controller can be created by the application and passed to the
/// [`modal_side_sheet_provider`] or
/// [`standard_side_sheet_provider`]. It is used to control the
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
#[derive(Clone, PartialEq)]
pub struct SideSheetController {
    is_open: bool,
    animation_start_frame_nanos: Option<u64>,
    drag_offset: f32,
}

impl SideSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            animation_start_frame_nanos: None,
            drag_offset: 0.0,
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
            self.drag_offset = 0.0;
            let now_nanos = current_frame_nanos();
            if let Some(old_start_frame_nanos) = self.animation_start_frame_nanos {
                let elapsed_nanos = now_nanos.saturating_sub(old_start_frame_nanos);
                let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                if elapsed_nanos < animation_nanos {
                    self.animation_start_frame_nanos =
                        Some(now_nanos.saturating_add(animation_nanos - elapsed_nanos));
                    return;
                }
            }
            self.animation_start_frame_nanos = Some(now_nanos);
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
            let now_nanos = current_frame_nanos();
            if let Some(old_start_frame_nanos) = self.animation_start_frame_nanos {
                let elapsed_nanos = now_nanos.saturating_sub(old_start_frame_nanos);
                let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                if elapsed_nanos < animation_nanos {
                    self.animation_start_frame_nanos =
                        Some(now_nanos.saturating_add(animation_nanos - elapsed_nanos));
                    return;
                }
            }
            self.animation_start_frame_nanos = Some(now_nanos);
        }
    }

    /// Returns whether the side sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the side sheet is currently animating.
    pub fn is_animating(&self) -> bool {
        self.animation_start_frame_nanos
            .map(|start| {
                let elapsed_nanos = current_frame_nanos().saturating_sub(start);
                let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                elapsed_nanos < animation_nanos
            })
            .unwrap_or(false)
    }

    fn snapshot(&self) -> (bool, Option<u64>, f32) {
        (
            self.is_open,
            self.animation_start_frame_nanos,
            self.drag_offset,
        )
    }

    fn drag_offset(&self) -> f32 {
        self.drag_offset
    }

    fn apply_drag_delta(&mut self, delta_x: f32, position: SideSheetPosition) -> f32 {
        let current_offset = self.drag_offset;
        let new_offset = match position {
            SideSheetPosition::Start => (current_offset + delta_x).min(0.0),
            SideSheetPosition::End => (current_offset + delta_x).max(0.0),
        };
        self.drag_offset = new_offset;
        new_offset - current_offset
    }

    fn complete_drag(&mut self) -> bool {
        let should_close = self.drag_offset.abs() > 100.0;
        if !should_close {
            self.drag_offset = 0.0;
        }
        should_close
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
fn calc_progress_from_timer(animation_start_frame_nanos: Option<u64>) -> f32 {
    let raw = match animation_start_frame_nanos {
        None => 1.0,
        Some(start_frame_nanos) => {
            let elapsed_nanos = current_frame_nanos().saturating_sub(start_frame_nanos);
            let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
            if elapsed_nanos >= animation_nanos {
                1.0
            } else {
                elapsed_nanos as f32 / animation_nanos as f32
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
    drag_offset: f32,
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
    (x + drag_offset) as i32
}

fn render_modal_scrim(on_close_request: Callback, progress: f32, is_open: bool) {
    let scrim_alpha = scrim_alpha_for(progress, is_open);
    let scrim_color = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .scrim;
    surface()
        .style(scrim_color.with_alpha(scrim_alpha).into())
        .on_click_shared(on_close_request)
        .modifier(Modifier::new().fill_max_size())
        .block_input(true)
        .child(|| {});
}

/// Render scrim according to configured type.
fn render_scrim(
    sheet_type: SideSheetType,
    on_close_request: Callback,
    progress: f32,
    is_open: bool,
) {
    match sheet_type {
        SideSheetType::Modal => render_modal_scrim(on_close_request, progress, is_open),
        SideSheetType::Standard => {
            surface()
                .style(Color::TRANSPARENT.into())
                .modifier(Modifier::new().fill_max_size())
                .child(|| {});
        }
    }
}

/// Create the keyboard handler closure used to close the sheet on Escape.
fn make_keyboard_closure(
    on_close: Callback,
) -> Box<dyn Fn(tessera_ui::KeyboardInput<'_>) + Send + Sync> {
    Box::new(move |mut input: tessera_ui::KeyboardInput<'_>| {
        let mut handled = false;
        input.keyboard_events.retain(|event| {
            if event.state != winit::event::ElementState::Pressed {
                return true;
            }

            if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                event.physical_key
            {
                on_close.call();
                handled = true;
                return false;
            }

            true
        });
        if handled {
            input.block_keyboard();
        }
    })
}

fn is_close_direction(delta_x: f32, position: SideSheetPosition) -> bool {
    match position {
        SideSheetPosition::Start => delta_x < 0.0,
        SideSheetPosition::End => delta_x > 0.0,
    }
}

fn is_open_direction(delta_x: f32, position: SideSheetPosition) -> bool {
    match position {
        SideSheetPosition::Start => delta_x > 0.0,
        SideSheetPosition::End => delta_x < 0.0,
    }
}

fn build_side_sheet_nested_scroll_connection(
    controller: State<SideSheetController>,
    on_close_request: Callback,
    position: SideSheetPosition,
    parent: Option<NestedScrollConnection>,
) -> NestedScrollConnection {
    NestedScrollConnection::new()
        .with_pre_scroll_handler(CallbackWith::new({
            move |input: PreScrollInput| {
                if input.source != tessera_ui::ScrollEventSource::Touch
                    || !is_open_direction(input.available.x, position)
                    || controller.with(|c| c.drag_offset()) == 0.0
                {
                    return ScrollDelta::ZERO;
                }

                let consumed_x =
                    controller.with_mut(|c| c.apply_drag_delta(input.available.x, position));
                ScrollDelta::new(consumed_x, 0.0)
            }
        }))
        .with_post_scroll_handler(CallbackWith::new({
            move |input: PostScrollInput| {
                if input.source != tessera_ui::ScrollEventSource::Touch
                    || !is_close_direction(input.available.x, position)
                {
                    return ScrollDelta::ZERO;
                }

                let consumed_x =
                    controller.with_mut(|c| c.apply_drag_delta(input.available.x, position));
                ScrollDelta::new(consumed_x, 0.0)
            }
        }))
        .with_pre_fling_handler(CallbackWith::new({
            move |input: PreFlingInput| {
                if controller.with(|c| c.drag_offset()) == 0.0 {
                    return ScrollVelocity::ZERO;
                }

                let should_close = controller.with_mut(|c| c.complete_drag());
                if should_close {
                    on_close_request.call();
                }

                ScrollVelocity::new(input.available.x, 0.0)
            }
        }))
        .with_parent(parent)
}

/// Place side sheet if present. Extracted to reduce complexity of the parent
/// function.
fn place_side_sheet_if_present(
    input: &MeasureScope<'_>,
    result: &mut LayoutResult,
    is_open: bool,
    progress: f32,
    position: SideSheetPosition,
    drag_offset: f32,
) {
    let children = input.children();
    if children.len() <= 2 {
        return;
    }

    let side_sheet = children[2];
    let parent_width = input
        .parent_constraint()
        .width()
        .resolve_max()
        .unwrap_or(Px(0));
    let parent_height = input
        .parent_constraint()
        .height()
        .resolve_max()
        .unwrap_or(Px(0));

    let max_width_px = MAX_SHEET_WIDTH.to_px();
    let sheet_width = if parent_width > max_width_px {
        max_width_px
    } else {
        parent_width
    };

    let constraint = Constraint::exact(sheet_width, parent_height);

    let child_size = match side_sheet.measure(&constraint) {
        Ok(s) => s,
        Err(_) => return,
    };

    let x = compute_side_sheet_x(
        parent_width,
        child_size.width,
        progress,
        is_open,
        position,
        drag_offset,
    );
    result.place_child(side_sheet, PxPosition::new(Px(x), Px(0)));
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
/// - `on_close_request` — optional close-request callback.
/// - `position` — side sheet edge position.
/// - `is_open` — declarative open state.
/// - `controller` — optional external controller.
/// - `main_content` — optional main content rendered behind the sheet.
/// - `side_sheet_content` — optional side sheet content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::side_sheet::modal_side_sheet_provider;
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         modal_side_sheet_provider()
///             .on_close_request(|| {})
///             .is_open(true)
///             .main_content(|| { /* main content */ })
///             .side_sheet_content(|| { /* side sheet content */ });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn modal_side_sheet_provider(
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    is_open: Option<bool>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let position = position.unwrap_or_default();
    let is_open = is_open.unwrap_or(false);
    side_sheet_provider_inner()
        .sheet_type(SideSheetType::Modal)
        .on_close_request_shared(on_close_request.unwrap_or_default())
        .position(position)
        .is_open(is_open)
        .main_content_shared(main_content.unwrap_or_else(RenderSlot::empty))
        .side_sheet_content_shared(side_sheet_content.unwrap_or_else(RenderSlot::empty))
        .controller_optional(controller);
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
/// - `on_close_request` — optional close-request callback.
/// - `position` — side sheet edge position.
/// - `is_open` — declarative open state.
/// - `controller` — optional external controller.
/// - `main_content` — optional main content rendered behind the sheet.
/// - `side_sheet_content` — optional side sheet content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::side_sheet::standard_side_sheet_provider;
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         standard_side_sheet_provider()
///             .on_close_request(|| {})
///             .is_open(true)
///             .main_content(|| { /* main content */ })
///             .side_sheet_content(|| { /* side sheet content */ });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn standard_side_sheet_provider(
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    is_open: Option<bool>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let position = position.unwrap_or_default();
    let is_open = is_open.unwrap_or(false);
    side_sheet_provider_inner()
        .sheet_type(SideSheetType::Standard)
        .on_close_request_shared(on_close_request.unwrap_or_default())
        .position(position)
        .is_open(is_open)
        .main_content_shared(main_content.unwrap_or_else(RenderSlot::empty))
        .side_sheet_content_shared(side_sheet_content.unwrap_or_else(RenderSlot::empty))
        .controller_optional(controller);
}

#[tessera]
fn side_sheet_provider_inner(
    sheet_type: Option<SideSheetType>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    is_open: Option<bool>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let on_close_request = on_close_request.unwrap_or_default();
    let position = position.unwrap_or_default();
    let is_open = is_open.unwrap_or(false);
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let side_sheet_content = side_sheet_content.unwrap_or_else(RenderSlot::empty);
    let external_controller = controller;
    let controller =
        external_controller.unwrap_or_else(|| remember(|| SideSheetController::new(is_open)));

    // In controlled mode (external controller provided), do not override
    // controller state from `is_open`.
    if external_controller.is_none() && is_open != controller.with(|c| c.is_open()) {
        if is_open {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }

    side_sheet_provider_render()
        .sheet_type(sheet_type)
        .on_close_request_shared(on_close_request)
        .position(position)
        .controller(controller)
        .main_content_shared(main_content)
        .side_sheet_content_shared(side_sheet_content);
}

#[tessera]
fn side_sheet_provider_render(
    sheet_type: Option<SideSheetType>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let position = position.unwrap_or_default();
    let on_close_request = on_close_request.unwrap_or_default();
    let controller = controller.expect("side_sheet_provider_render requires controller");
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let side_sheet_content = side_sheet_content.unwrap_or_else(RenderSlot::empty);

    let (is_open, timer_opt, drag_offset) = controller.with(|c| c.snapshot());
    let is_animating = controller.with(|c| c.is_animating());
    let side_sheet_open_state = remember(|| false);
    let mut just_opened = false;
    side_sheet_open_state.with_mut(|was_open| {
        just_opened = !*was_open && is_open;
        *was_open = is_open;
    });
    if is_animating {
        let controller_for_frame = controller;
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller_for_frame.with_mut(|controller| {
                let (_, timer_opt, _) = controller.snapshot();
                if let Some(start_frame_nanos) = timer_opt {
                    let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
                    let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                    elapsed_nanos < animation_nanos
                } else {
                    false
                }
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let show_side_sheet = is_open || is_animating;

    let progress = calc_progress_from_timer(timer_opt);

    layout()
        .layout_policy(SideSheetLayout {
            progress,
            is_open,
            position,
            drag_offset,
        })
        .child(move || {
            let side_sheet_content = side_sheet_content;
            main_content.render();

            if show_side_sheet {
                render_scrim(sheet_type, on_close_request, progress, is_open);

                side_sheet_content_wrapper()
                    .sheet_type(sheet_type)
                    .position(position)
                    .controller(controller)
                    .on_close_request_shared(on_close_request)
                    .just_opened(just_opened)
                    .content_shared(side_sheet_content);
            }
        });
}

#[tessera]
fn side_sheet_content_wrapper(
    sheet_type: Option<SideSheetType>,
    position: Option<SideSheetPosition>,
    controller: Option<State<SideSheetController>>,
    on_close_request: Option<Callback>,
    just_opened: Option<bool>,
    content: Option<RenderSlot>,
) {
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let position = position.unwrap_or_default();
    let just_opened = just_opened.unwrap_or(false);
    let controller = controller.expect("side_sheet_content_wrapper requires controller");
    let on_close_request = on_close_request.unwrap_or_default();
    let content = content.expect("side_sheet_content_wrapper requires content");
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let container_color = match sheet_type {
        SideSheetType::Modal => scheme.surface_container_low,
        SideSheetType::Standard => scheme.surface,
    };
    let is_modal = sheet_type == SideSheetType::Modal;
    let on_close_request_for_keyboard = on_close_request;

    let focus_scope = remember(FocusScopeNode::new).get();
    let modifier = with_keyboard_input(
        Modifier::new()
            .focus_scope_with(focus_scope)
            .focus_traversal_policy(
                FocusTraversalPolicy::linear()
                    .wrap(true)
                    .tab_navigation(true),
            ),
        make_keyboard_closure(on_close_request_for_keyboard),
    );
    if just_opened {
        focus_scope.restore_focus();
    }
    layout().modifier(modifier).child(move || {
        let content = content;
        if is_modal {
            surface()
                .elevation(MODAL_ELEVATION)
                .style(container_color.into())
                .shape(sheet_shape(position))
                .modifier(Modifier::new().fill_max_height())
                .block_input(true)
                .child(move || {
                    let content = content;
                    let parent_nested_scroll =
                        use_context::<NestedScrollConnection>().map(|context| context.get());
                    let nested_scroll_connection = build_side_sheet_nested_scroll_connection(
                        controller,
                        on_close_request,
                        position,
                        parent_nested_scroll,
                    );
                    layout()
                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                        .child(move || {
                            let content = content;
                            provide_context(
                                || nested_scroll_connection.clone(),
                                move || {
                                    content.render();
                                },
                            );
                        });
                });
        } else {
            surface()
                .style(container_color.into())
                .shape(sheet_shape(position))
                .modifier(Modifier::new().fill_max_height())
                .block_input(true)
                .child(move || {
                    let content = content;
                    let parent_nested_scroll =
                        use_context::<NestedScrollConnection>().map(|context| context.get());
                    let nested_scroll_connection = build_side_sheet_nested_scroll_connection(
                        controller,
                        on_close_request,
                        position,
                        parent_nested_scroll,
                    );
                    layout()
                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                        .child(move || {
                            let content = content;
                            provide_context(
                                || nested_scroll_connection.clone(),
                                move || {
                                    content.render();
                                },
                            );
                        });
                });
        }
    });
}

#[derive(Clone, PartialEq)]
struct SideSheetLayout {
    progress: f32,
    is_open: bool,
    position: SideSheetPosition,
    drag_offset: f32,
}

impl LayoutPolicy for SideSheetLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_constraint = input.parent_constraint().without_min();
        let main_content = children[0];
        let main_content_size = main_content.measure(&child_constraint)?;
        result.place_child(main_content, PxPosition::new(Px(0), Px(0)));

        if children.len() > 1 {
            let scrim = children[1];
            scrim.measure(&child_constraint)?;
            result.place_child(scrim, PxPosition::new(Px(0), Px(0)));
        }

        place_side_sheet_if_present(
            input,
            &mut result,
            self.is_open,
            self.progress,
            self.position,
            self.drag_offset,
        );

        Ok(result.with_size(main_content_size.size()))
    }
}
