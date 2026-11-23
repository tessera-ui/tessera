//! Side sheets present supporting content from a window edge.
//!
//! ## Usage
//!
//! Show filters, details and secondary tasks beside the main content.

mod input;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Callback, CallbackWith, ComputedData, Constraint, Dp, FocusProperties, FocusRestorationTarget,
    FocusScopeNode, FocusTraversalPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition,
    PxSize, RenderSlot, State, focus, launch,
    layout::{MeasureScope, PlacementResult, PlacementScope, Stack, layout},
    modifier::FocusModifierExt as _,
    popup::{WindowOrigin, popup},
    remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    animation::{
        Motion, run_frames,
        sheet_drag::SheetDrag,
        visibility_motion::{VisibilityMotion, VisibilitySample},
    },
    md3::{modal_scrim::modal_scrim, surface::surface, theme::MaterialTheme},
    modifier::with_keyboard_input,
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    },
    overlay_semantics::dialog_semantics,
    shape_def::{RoundedCorner, Shape},
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

/// Controller for side sheets, managing open/closed state.
#[derive(Clone, PartialEq)]
pub struct SideSheetController {
    is_open: bool,
    motion: VisibilityMotion,
    drag: SheetDrag,
    requests: u32,
}

impl SideSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            motion: VisibilityMotion::new(initial_open, true),
            drag: SheetDrag::new(),
            requests: 0,
        }
    }

    /// Initiates the animation to open the side sheet.
    ///
    /// No-op if already open; reverses an in-progress close.
    pub fn open(&mut self) {
        if !self.is_open {
            self.motion.request(true);
            self.is_open = true;
            self.drag.settle();
            self.requests = self.requests.wrapping_add(1);
        }
    }

    /// Initiates the animation to close the side sheet.
    ///
    /// No-op if already closed; reverses an in-progress open.
    pub fn close(&mut self) {
        if self.is_open {
            self.motion.request(false);
            self.is_open = false;
            self.drag.settle();
            self.requests = self.requests.wrapping_add(1);
        }
    }

    /// Returns whether the side sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the side sheet is animating or is about to start
    /// animating on the next frame.
    pub fn is_animating(&self) -> bool {
        self.motion.sample().active || (self.is_open && self.drag.is_animating())
    }

    /// Advances the sheet's motions to `now` and returns whether they are
    /// still running.
    fn advance(&mut self, now: Instant) -> bool {
        self.motion.advance(now);
        self.drag.advance(now);
        self.is_animating()
    }

    fn snapshot(&self) -> (bool, VisibilitySample, f32) {
        (self.is_open, self.motion.sample(), self.drag_offset())
    }

    fn drag_offset(&self) -> f32 {
        self.drag.offset()
    }

    fn grab(&mut self) {
        self.drag.grab();
    }

    fn settle_drag(&mut self) {
        self.drag.settle();
        self.requests = self.requests.wrapping_add(1);
    }

    fn apply_drag_delta(&mut self, delta_x: f32, position: SideSheetPosition) -> f32 {
        let direction = if position == SideSheetPosition::Start {
            -1.0
        } else {
            1.0
        };
        self.drag.apply_delta(delta_x * direction) * direction
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
        self.requests = self.requests.wrapping_add(1);
        self.drag.release(velocity * direction, extent * 0.5, 500.0)
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
                controller.with_mut(|c| c.drag.set_extent(extent));
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
    let Some(&side_sheet) = children.get(1) else {
        return;
    };
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

/// # modal_side_sheet
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
/// - `on_close_request` — called on a scrim tap, unhandled Escape, system back,
///   or a dismissing drag. Defaults to closing the controller.
/// - `position` — side sheet edge position.
/// - `max_width` — maximum surface width, constrained by the window; defaults to
///   400dp.
/// - `content_padding` — inset inside the surface; defaults to 24dp. Negative
///   dimensions become zero; non-finite dimensions use their defaults.
/// - `controller` — controls visibility. Keep the component mounted when
///   closing so its exit animation can finish.
/// - `side_sheet_content` — optional side sheet content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::{side_sheet::{SideSheetController, modal_side_sheet}, text::text};
/// use tessera_ui::remember;
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         let controller = remember(|| SideSheetController::new(true));
///         modal_side_sheet(controller)
///             .accessibility_label("Filters")
///             .max_width(tessera_ui::Dp(360.0))
///             .content_padding(tessera_ui::Dp::ZERO)
///             .side_sheet_content(|| text("Filter options"));
///     });
/// # }
/// ```
#[tessera]
pub fn modal_side_sheet(
    controller: State<SideSheetController>,
    #[prop(into)] accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    max_width: Option<Dp>,
    content_padding: Option<Dp>,
    side_sheet_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request
        .unwrap_or_else(|| Callback::new(move || controller.with_mut(SideSheetController::close)));
    side_sheet_container(controller)
        .accessibility_label_optional(accessibility_label)
        .sheet_type(SideSheetType::Modal)
        .on_close_request_shared(on_close_request)
        .position_optional(position)
        .max_width_optional(max_width)
        .content_padding_optional(content_padding)
        .side_sheet_content_optional(side_sheet_content);
}

/// # standard_side_sheet
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
/// - `controller` — optional external controller.
/// - `main_content` — optional main content rendered behind the sheet.
/// - `side_sheet_content` — optional side sheet content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::{side_sheet::{SideSheetController, standard_side_sheet}, text::text};
/// use tessera_ui::remember;
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         let controller = remember(|| SideSheetController::new(true));
///         standard_side_sheet()
///             .accessibility_label("Details")
///             .controller(controller)
///             .main_content(|| text("Main content"))
///             .side_sheet_content(|| text("Details"));
///     });
/// # }
/// ```
#[tessera]
pub fn standard_side_sheet(
    #[prop(into)] accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    controller: Option<State<SideSheetController>>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let controller = match controller {
        Some(controller) => controller,
        None => remember(|| SideSheetController::new(false)),
    };
    side_sheet_container(controller)
        .accessibility_label_optional(accessibility_label)
        .sheet_type(SideSheetType::Standard)
        .on_close_request_optional(on_close_request)
        .position_optional(position)
        .main_content_optional(main_content)
        .side_sheet_content_optional(side_sheet_content);
}

#[tessera]
fn side_sheet_container(
    controller: State<SideSheetController>,
    accessibility_label: Option<String>,
    max_width: Option<Dp>,
    content_padding: Option<Dp>,
    sheet_type: Option<SideSheetType>,
    on_close_request: Option<Callback>,
    position: Option<SideSheetPosition>,
    main_content: Option<RenderSlot>,
    side_sheet_content: Option<RenderSlot>,
) {
    let max_width = side_sheet_dimension(max_width, MAX_SHEET_WIDTH);
    let content_padding = side_sheet_dimension(content_padding, Dp(24.0));
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let position = position.unwrap_or_default();
    let on_close_request = on_close_request.unwrap_or_default();
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let side_sheet_content = side_sheet_content.unwrap_or_else(RenderSlot::empty);

    let current_position = remember(|| position);
    if current_position.get() != position {
        current_position.set(position);
    }
    let is_modal = sheet_type == SideSheetType::Modal;
    let parent_nested_scroll = if is_modal {
        None
    } else {
        NestedScrollConnection::nearest()
    };
    let coordinate_scale = remember(|| 1.0_f32);
    let nested_scroll_connection = build_side_sheet_nested_scroll_connection(
        controller,
        on_close_request,
        current_position,
        parent_nested_scroll,
    )
    .with_coordinate_scale(coordinate_scale);
    let (is_open, visibility, drag_offset) = controller.with(SideSheetController::snapshot);
    let is_animating = controller.with(SideSheetController::is_animating);
    if is_animating {
        launch(
            controller.with(|c| c.requests),
            run_frames(controller, SideSheetController::advance),
        );
    }

    let show_side_sheet = is_open || is_animating;
    let drag_offset = if position == SideSheetPosition::Start {
        -drag_offset
    } else {
        drag_offset
    };
    let mut modifier =
        Modifier::new().on_local_to_window(move |transform: tessera_ui::LocalToWindow| {
            if coordinate_scale.get() != transform.scale() {
                coordinate_scale.set(transform.scale());
            }
        });
    let focus_scope = if is_modal {
        None
    } else {
        let scope = remember(FocusScopeNode::new).get();
        let focus = remember(StandardSheetFocus::default);
        modifier = standard_sheet_focus(modifier, focus, show_side_sheet, scope);
        Some(scope)
    };
    if is_modal && !show_side_sheet {
        return;
    }
    let scrim = if is_modal {
        Some(
            use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .scrim
                .with_alpha(visibility.opacity * SCRIM_ALPHA),
        )
    } else {
        None
    };

    let progress = if is_open {
        visibility.position
    } else {
        1.0 - visibility.position
    };

    let layer = RenderSlot::new(move || {
        let nested_scroll_connection = nested_scroll_connection.clone();
        let accessibility_label = accessibility_label.clone();
        layout(SideSheetLayout {
            max_width,
            progress,
            is_open,
            position,
            drag_offset,
        })
        .modifier(modifier.clone())
        .child(move || {
            if let Some(scrim) = scrim {
                modal_scrim(scrim).on_dismiss_optional(is_open.then_some(on_close_request));
            } else {
                side_sheet_main_content(main_content);
            }
            if show_side_sheet {
                side_sheet_content_wrapper()
                    .content_padding(content_padding)
                    .nested_scroll_connection(nested_scroll_connection.clone())
                    .accessibility_label_optional(accessibility_label.clone())
                    .focus_scope_optional(focus_scope)
                    .sheet_type(sheet_type)
                    .position(current_position)
                    .controller(controller)
                    .on_close_request_shared(on_close_request)
                    .content_shared(side_sheet_content);
            }
        });
    });
    if is_modal {
        popup(WindowOrigin)
            .modal(true)
            .dismiss_on_outside_press(false)
            .dismiss_on_escape(is_open)
            .dismiss_on_back(is_open)
            .on_dismiss_request(on_close_request)
            .child(layer);
    } else {
        layer.render();
    }
}

#[derive(Default)]
struct StandardSheetFocus {
    visible: bool,
    origin: Option<FocusRestorationTarget>,
    pending: Option<FocusRestorationTarget>,
}

fn standard_sheet_focus(
    modifier: Modifier,
    state: State<StandardSheetFocus>,
    visible: bool,
    scope: FocusScopeNode,
) -> Modifier {
    if visible != state.with(|state| state.visible) {
        focus::defer_keyboard_until_layout();
        if visible {
            state.set(StandardSheetFocus {
                visible,
                origin: focus::save_focus(),
                pending: None,
            });
            scope.restore_focus();
        } else {
            let restore = scope.has_focus();
            state.with_mut(|state| {
                state.visible = false;
                state.pending = state.origin.take().filter(|_| restore);
            });
        }
    }
    if state.with(|state| state.pending.is_some()) {
        modifier.on_globally_positioned(move |_| {
            if let Some(target) = state.with_mut(|state| state.pending.take()) {
                target.restore();
            }
        })
    } else {
        modifier
    }
}

#[tessera]
fn side_sheet_main_content(content: RenderSlot) {
    layout(Stack).child(content);
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
    content: Option<RenderSlot>,
) {
    let content_padding = side_sheet_dimension(content_padding, Dp(24.0));
    let sheet_type = sheet_type.unwrap_or(SideSheetType::Modal);
    let position = position.expect("sheet wrapper requires its edge state");
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
    let scope = if let Some(scope) = focus_scope {
        scope
    } else {
        remember(FocusScopeNode::new).get()
    };
    let mut modifier = Modifier::new()
        .focus_scope(Some(scope))
        .focus_properties(FocusProperties::new().skip_traversal(true))
        .focus_traversal_policy(
            FocusTraversalPolicy::linear()
                .wrap(is_modal)
                .tab_navigation(true),
        );
    if !is_modal {
        modifier = with_keyboard_input(
            modifier,
            Box::new(move |mut input: tessera_ui::KeyboardInput<'_>| {
                let mut handled = false;
                input.keyboard_events.retain(|event| {
                    if event.state == tessera_ui::winit::event::ElementState::Pressed
                        && event.physical_key
                            == tessera_ui::winit::keyboard::PhysicalKey::Code(
                                tessera_ui::winit::keyboard::KeyCode::Escape,
                            )
                    {
                        on_close_request.call();
                        handled = true;
                        false
                    } else {
                        true
                    }
                });
                if handled {
                    input.block_keyboard();
                }
            }),
        );
    }
    let modifier = dialog_semantics(modifier, accessibility_label, is_modal);
    layout(Stack).modifier(modifier).child(move || {
        let content = content;
        let nested_scroll_connection = nested_scroll_connection.clone();
        let edge = position.get();
        surface()
            .elevation(if is_modal { MODAL_ELEVATION } else { Dp::ZERO })
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

impl tessera_ui::LayoutPolicy for SideSheetLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.max_width == other.max_width
    }

    fn place(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let children = input.children();
        let main_content = children.first()?;
        let mut result = LayoutResult::new(input.size());
        result.place_child(*main_content, PxPosition::ZERO);
        if let Some(sheet) = children.get(1) {
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
