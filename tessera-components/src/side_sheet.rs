//! Side sheet provider - slide supporting content in from the screen edge.
//!
//! ## Usage
//!
//! Show filters, details and secondary tasks beside the main content.

mod input;

use tessera_ui::{
    Callback, CallbackWith, Color, ComputedData, Constraint, Dp, FocusProperties, FocusScopeNode,
    FocusTraversalPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition, PxSize,
    RenderSlot, State, current_frame_nanos,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    modifier::FocusModifierExt as _,
    receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    animation::{
        sheet_drag::SheetDrag,
        visibility_motion::{VisibilityMotion, VisibilitySample},
    },
    modal_focus::{ModalFocusState, modal_focus_modifier},
    modifier::{ModifierExt, SemanticsArgs, with_keyboard_input},
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    },
    overlay_semantics::{dialog_semantics, overlay_background},
    shape_def::{RoundedCorner, Shape},
    surface::surface,
    theme::MaterialTheme,
};

use input::side_sheet_body;

const SCRIM_ALPHA: f32 = 0.32;
const MAX_SHEET_WIDTH: Dp = Dp(400.0);
const CORNER_RADIUS: Dp = Dp(16.0);
const MODAL_ELEVATION: Dp = Dp(1.0);

fn side_sheet_dimension(value: Option<Dp>, default: Dp) -> Dp {
    let value = value.filter(|value| value.0.is_finite()).unwrap_or(default);
    Dp(value.0.max(0.0))
}

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
    motion: VisibilityMotion,
    drag: SheetDrag,
}

impl SideSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            motion: VisibilityMotion::new(initial_open, true),
            drag: SheetDrag::new(),
        }
    }

    /// Initiates the animation to open the side sheet.
    ///
    /// If the sheet is already open this has no effect. If it is currently
    /// closing, the animation will reverse direction and start opening from the
    /// current animated position.
    pub fn open(&mut self) {
        if !self.is_open {
            self.motion.retarget(true, current_frame_nanos());
            self.is_open = true;
            self.drag.settle(current_frame_nanos());
        }
    }

    /// Initiates the animation to close the side sheet.
    ///
    /// If the sheet is already closed this has no effect. If it is currently
    /// opening, the animation will reverse direction and start closing from the
    /// current animated position.
    pub fn close(&mut self) {
        if self.is_open {
            self.motion.retarget(false, current_frame_nanos());
            self.is_open = false;
            self.drag.settle(current_frame_nanos());
        }
    }

    /// Returns whether the side sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the side sheet is currently animating.
    pub fn is_animating(&self) -> bool {
        self.is_animating_at(current_frame_nanos())
    }

    fn is_animating_at(&self, frame: u64) -> bool {
        self.motion.sample(frame).active || (self.is_open && self.drag.sample(frame).1)
    }

    fn snapshot(&self) -> (bool, VisibilitySample, f32) {
        (
            self.is_open,
            self.motion.sample(current_frame_nanos()),
            self.drag_offset(),
        )
    }

    fn drag_offset(&self) -> f32 {
        self.drag.sample(current_frame_nanos()).0
    }

    fn apply_drag_delta(&mut self, delta_x: f32, position: SideSheetPosition) -> f32 {
        let direction = if position == SideSheetPosition::Start {
            -1.0
        } else {
            1.0
        };
        self.drag
            .apply_delta(delta_x * direction, current_frame_nanos())
            * direction
    }

    fn complete_drag(&mut self, velocity: f32, position: SideSheetPosition) -> bool {
        let direction = if position == SideSheetPosition::Start {
            -1.0
        } else {
            1.0
        };
        let extent = if self.drag.extent() > 0.0 {
            self.drag.extent()
        } else {
            MAX_SHEET_WIDTH.0
        };
        self.drag.release(
            current_frame_nanos(),
            velocity * direction,
            extent * 0.5,
            500.0,
        )
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

/// Compute X position for side sheet placement.
fn compute_side_sheet_x(
    parent_width: Px,
    sheet_width: Px,
    progress: f32,
    is_open: bool,
    position: SideSheetPosition,
    drag_offset: f32,
) -> f32 {
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
    x + drag_offset
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

fn side_sheet_extent_modifier(controller: State<SideSheetController>) -> Modifier {
    Modifier::new().fill_max_height().on_layout_coordinates(
        move |coordinates: tessera_ui::LayoutCoordinates| {
            let extent = Dp::from_pixels_f64(f64::from(coordinates.local_size.width.0)).0;
            if controller.with(|c| c.drag.extent() != extent) {
                controller.with_mut(|c| c.drag.set_extent(extent, current_frame_nanos()));
            }
        },
    )
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
    position: State<SideSheetPosition>,
    parent: Option<NestedScrollConnection>,
) -> NestedScrollConnection {
    NestedScrollConnection::new()
        .with_pre_scroll_handler(CallbackWith::new({
            move |input: PreScrollInput| {
                if input.source != tessera_ui::ScrollEventSource::Touch
                    || !is_open_direction(input.available.x, position.get())
                    || controller.with(|c| c.drag_offset()) == 0.0
                {
                    return ScrollDelta::ZERO;
                }

                let consumed_x =
                    controller.with_mut(|c| c.apply_drag_delta(input.available.x, position.get()));
                ScrollDelta::new(consumed_x, 0.0)
            }
        }))
        .with_post_scroll_handler(CallbackWith::new({
            move |input: PostScrollInput| {
                if input.source != tessera_ui::ScrollEventSource::Touch
                    || !is_close_direction(input.available.x, position.get())
                {
                    return ScrollDelta::ZERO;
                }

                let consumed_x =
                    controller.with_mut(|c| c.apply_drag_delta(input.available.x, position.get()));
                ScrollDelta::new(consumed_x, 0.0)
            }
        }))
        .with_pre_fling_handler(CallbackWith::new({
            move |input: PreFlingInput| {
                if controller.with(|c| c.drag_offset()) == 0.0 {
                    return ScrollVelocity::ZERO;
                }

                let should_close =
                    controller.with_mut(|c| c.complete_drag(input.available.x, position.get()));
                if should_close {
                    on_close_request.call();
                }

                ScrollVelocity::new(input.available.x, 0.0)
            }
        }))
        .with_parent(parent)
}

fn place_side_sheet_if_present(
    input: &mut MeasureScope<'_>,
    result: &mut LayoutResult,
    host_size: PxSize,
    max_width: Dp,
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
    let parent_width = host_size.width;
    let parent_height = host_size.height;

    let max_width_px = max_width.to_px();
    let sheet_width = if parent_width > max_width_px {
        max_width_px
    } else {
        parent_width
    };

    let constraint = Constraint::exact(sheet_width, parent_height);

    let child_size = match side_sheet.measure(input, &constraint) {
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
    result.place_child(side_sheet, PxPosition::from_f32_xy(x, 0.0));
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
/// - `accessibility_label` — accessible name describing the dialog or sheet.
/// - `on_close_request` — optional close-request callback.
/// - `position` — side sheet edge position.
/// - `is_open` — declarative open state.
/// - `max_width` — maximum surface width, constrained by the host; defaults to
///   400dp.
/// - `content_padding` — inset inside the surface; defaults to 24dp. Negative
///   dimensions become zero; non-finite dimensions use their defaults.
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
///             .accessibility_label("Filters")
///             .on_close_request(|| {})
///             .is_open(true)
///             .max_width(tessera_ui::Dp(360.0))
///             .content_padding(tessera_ui::Dp::ZERO)
///             .main_content(|| { /* main content */ })
///             .side_sheet_content(|| { /* side sheet content */ });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn modal_side_sheet_provider(
    #[prop(into)] accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    is_open: Option<bool>,
    max_width: Option<Dp>,
    content_padding: Option<Dp>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let position = position.unwrap_or_default();
    let is_open = is_open.unwrap_or(false);
    side_sheet_provider_inner()
        .accessibility_label_optional(accessibility_label)
        .sheet_type(SideSheetType::Modal)
        .on_close_request_shared(on_close_request.unwrap_or_default())
        .position(position)
        .is_open(is_open)
        .max_width_optional(max_width)
        .content_padding_optional(content_padding)
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
/// - `accessibility_label` — accessible name describing the dialog or sheet.
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
///             .accessibility_label("Details")
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
    #[prop(into)] accessibility_label: Option<String>,
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
        .accessibility_label_optional(accessibility_label)
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
    accessibility_label: Option<String>,
    max_width: Option<Dp>,
    content_padding: Option<Dp>,
    sheet_type: Option<SideSheetType>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    is_open: Option<bool>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let max_width = side_sheet_dimension(max_width, MAX_SHEET_WIDTH);
    let content_padding = side_sheet_dimension(content_padding, Dp(24.0));
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
        .accessibility_label_optional(accessibility_label)
        .max_width(max_width)
        .content_padding(content_padding)
        .sheet_type(sheet_type)
        .on_close_request_shared(on_close_request)
        .position(position)
        .controller(controller)
        .main_content_shared(main_content)
        .side_sheet_content_shared(side_sheet_content);
}

#[tessera]
fn side_sheet_provider_render(
    accessibility_label: Option<String>,
    max_width: Option<Dp>,
    content_padding: Option<Dp>,
    sheet_type: Option<SideSheetType>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let max_width = side_sheet_dimension(max_width, MAX_SHEET_WIDTH);
    let content_padding = side_sheet_dimension(content_padding, Dp(24.0));
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let position = position.unwrap_or_default();
    let on_close_request = on_close_request.unwrap_or_default();
    let controller = controller.expect("side_sheet_provider_render requires controller");
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let side_sheet_content = side_sheet_content.unwrap_or_else(RenderSlot::empty);

    let current_position = remember(|| position);
    if current_position.get() != position {
        current_position.set(position);
    }
    let parent_nested_scroll = use_context::<NestedScrollConnection>().map(|context| context.get());
    let coordinate_scale = remember(|| 1.0_f32);
    let nested_scroll_connection = build_side_sheet_nested_scroll_connection(
        controller,
        on_close_request,
        current_position,
        parent_nested_scroll,
    )
    .with_coordinate_scale(coordinate_scale);
    let (is_open, visibility, drag_offset) = controller.get().snapshot();
    let is_animating = controller.get().is_animating();
    let side_sheet_open_state = remember(|| false);
    let mut just_opened = false;
    side_sheet_open_state.with_mut(|was_open| {
        just_opened = !*was_open && is_open;
        *was_open = is_open;
    });
    if is_animating {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating =
                controller.with(|controller| controller.is_animating_at(frame_nanos));
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let show_side_sheet = is_open || is_animating;
    let drag_offset = if position == SideSheetPosition::Start {
        -drag_offset
    } else {
        drag_offset
    };
    let focus_scope = remember(FocusScopeNode::new).get();
    let focus_state = remember(ModalFocusState::default);
    let modifier = modal_focus_modifier(Modifier::new(), focus_state, show_side_sheet, focus_scope);

    let progress = if is_open {
        visibility.position
    } else {
        1.0 - visibility.position
    };

    layout()
        .modifier(
            modifier.on_local_to_window(move |transform: tessera_ui::LocalToWindow| {
                if coordinate_scale.get() != transform.scale() {
                    coordinate_scale.set(transform.scale());
                }
            }),
        )
        .policy(SideSheetLayout {
            max_width,
            progress,
            is_open,
            position,
            drag_offset,
        })
        .child(move || {
            let side_sheet_content = side_sheet_content;
            overlay_background(
                main_content,
                show_side_sheet && sheet_type == SideSheetType::Modal,
            );

            if show_side_sheet {
                match sheet_type {
                    SideSheetType::Modal => {
                        let scrim_alpha = visibility.opacity * SCRIM_ALPHA;
                        let scrim_color = use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .scrim;
                        surface()
                            .style(scrim_color.with_alpha(scrim_alpha).into())
                            .on_click_shared(on_close_request)
                            .show_ripple(false)
                            .show_state_layer(false)
                            .modifier(
                                Modifier::new()
                                    .fill_max_size()
                                    .semantics(SemanticsArgs {
                                        hidden: true,
                                        ..Default::default()
                                    })
                                    .focusable()
                                    .focus_properties(
                                        FocusProperties::new()
                                            .can_focus(false)
                                            .can_request_focus(false)
                                            .skip_traversal(true),
                                    ),
                            )
                            .block_input(true)
                            .child(|| {});
                    }
                    SideSheetType::Standard => {
                        surface()
                            .style(Color::TRANSPARENT.into())
                            .modifier(Modifier::new().fill_max_size())
                            .child(|| {});
                    }
                }

                side_sheet_content_wrapper()
                    .content_padding(content_padding)
                    .nested_scroll_connection(nested_scroll_connection.clone())
                    .accessibility_label_optional(accessibility_label.clone())
                    .focus_scope(focus_scope)
                    .sheet_type(sheet_type)
                    .position(current_position)
                    .controller(controller)
                    .on_close_request_shared(on_close_request)
                    .just_opened(just_opened)
                    .content_shared(side_sheet_content);
            }
        });
}

#[tessera]
fn side_sheet_content_wrapper(
    accessibility_label: Option<String>,
    content_padding: Option<Dp>,
    nested_scroll_connection: Option<NestedScrollConnection>,
    focus_scope: Option<FocusScopeNode>,
    sheet_type: Option<SideSheetType>,
    position: Option<State<SideSheetPosition>>,
    controller: Option<State<SideSheetController>>,
    on_close_request: Option<Callback>,
    just_opened: Option<bool>,
    content: Option<RenderSlot>,
) {
    let content_padding = side_sheet_dimension(content_padding, Dp(24.0));
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let position = position.expect("sheet wrapper requires its edge state");
    let just_opened = just_opened.unwrap_or(false);
    let controller = controller.expect("side_sheet_content_wrapper requires controller");
    let on_close_request = on_close_request.unwrap_or_default();
    let content = content.expect("side_sheet_content_wrapper requires content");
    let nested_scroll_connection =
        nested_scroll_connection.expect("sheet content requires its nested scroll connection");
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

    let focus_scope = focus_scope.expect("side sheet content requires its focus scope");
    let modifier = dialog_semantics(
        with_keyboard_input(
            Modifier::new()
                .focus_scope_with(focus_scope)
                .focus_properties(FocusProperties::new().skip_traversal(true))
                .focus_traversal_policy(
                    FocusTraversalPolicy::linear()
                        .wrap(is_modal)
                        .tab_navigation(true),
                ),
            make_keyboard_closure(on_close_request_for_keyboard),
        ),
        accessibility_label,
        is_modal,
    );
    if just_opened {
        if is_modal {
            focus_scope.request_focus();
        } else {
            focus_scope.restore_focus();
        }
    }
    layout().modifier(modifier).child(move || {
        let content = content;
        let nested_scroll_connection = nested_scroll_connection.clone();
        let edge = position.get();
        if is_modal {
            surface()
                .elevation(MODAL_ELEVATION)
                .style(container_color.into())
                .shape(sheet_shape(edge))
                .modifier(side_sheet_extent_modifier(controller))
                .block_input(true)
                .child(move || {
                    side_sheet_body()
                        .content_padding(content_padding)
                        .controller(controller)
                        .position(position)
                        .on_close_shared(on_close_request)
                        .content_shared(content)
                        .nested_scroll_connection(nested_scroll_connection.clone());
                });
        } else {
            surface()
                .style(container_color.into())
                .shape(sheet_shape(edge))
                .modifier(side_sheet_extent_modifier(controller))
                .block_input(true)
                .child(move || {
                    side_sheet_body()
                        .content_padding(content_padding)
                        .controller(controller)
                        .position(position)
                        .on_close_shared(on_close_request)
                        .content_shared(content)
                        .nested_scroll_connection(nested_scroll_connection.clone());
                });
        }
    });
}

#[derive(Clone, PartialEq)]
struct SideSheetLayout {
    max_width: Dp,
    progress: f32,
    is_open: bool,
    position: SideSheetPosition,
    drag_offset: f32,
}

impl tessera_ui::LayoutModifierNode for SideSheetLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.max_width == other.max_width
    }

    fn place_children(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let children = input.children();
        let main_content = children.first()?;
        let mut result = LayoutResult::new(input.size());
        result.place_child(*main_content, PxPosition::ZERO);
        if let Some(scrim) = children.get(1) {
            result.place_child(*scrim, PxPosition::ZERO);
        }
        if let Some(sheet) = children.get(2) {
            let parent_width = input.size().width;
            let x = compute_side_sheet_x(
                parent_width,
                sheet.size().width,
                self.progress,
                self.is_open,
                self.position,
                self.drag_offset,
            );
            result.place_child(*sheet, PxPosition::from_f32_xy(x, 0.0));
        }
        Some(result.into_placement_result())
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_constraint = input.parent_constraint().without_min();
        let main_content = children[0];
        let main_content_size = main_content.measure(input, &child_constraint)?;
        let host_size = PxSize::new(
            input
                .parent_constraint()
                .width()
                .clamp(main_content_size.width),
            input
                .parent_constraint()
                .height()
                .clamp(main_content_size.height),
        );
        result.place_child(main_content, PxPosition::new(Px(0), Px(0)));

        if children.len() > 1 {
            let scrim = children[1];
            scrim.measure(input, &Constraint::exact(host_size.width, host_size.height))?;
            result.place_child(scrim, PxPosition::new(Px(0), Px(0)));
        }

        place_side_sheet_if_present(
            input,
            &mut result,
            host_size,
            self.max_width,
            self.is_open,
            self.progress,
            self.position,
            self.drag_offset,
        );

        Ok(result.with_size(ComputedData {
            width: host_size.width,
            height: host_size.height,
        }))
    }
}

#[cfg(test)]
mod placement_tests {
    use tessera_ui::Px;

    use super::{SideSheetPosition, compute_side_sheet_x};

    #[test]
    fn both_edges_and_directions_keep_fractional_sheet_and_drag_placement() {
        for (open, progress) in [(true, 0.3125), (false, 0.6875)] {
            assert_eq!(
                compute_side_sheet_x(
                    Px(800),
                    Px(360),
                    progress,
                    open,
                    SideSheetPosition::Start,
                    -2.25
                ),
                -249.75
            );
            assert_eq!(
                compute_side_sheet_x(
                    Px(800),
                    Px(360),
                    progress,
                    open,
                    SideSheetPosition::End,
                    2.25
                ),
                689.75
            );
        }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "side_sheet_tests.rs"]
mod host_tests;
