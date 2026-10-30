//! A component that displays content sliding up from the bottom of the screen.
//!
//! ## Usage
//!
//! Used to show contextual information or actions in a modal sheet.

use std::time::Duration;

use tessera_foundation::{
    gesture::{DragRecognizer, TapRecognizer},
    scroll_physics::ScrollVelocityTracker,
};
use tessera_ui::{
    AxisConstraint, Callback, CallbackWith, Constraint, Dp, FocusProperties, FocusScopeNode,
    FocusTraversalPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot,
    State, current_frame_nanos, current_frame_time,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, remember_with_key, tessera,
    time::Instant,
    use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    animation::{
        material_motion::{Scalar, Spec},
        sheet_drag::SheetDrag,
        visibility_motion::{VisibilityMotion, VisibilitySample},
    },
    column::column,
    modal_focus::{ModalFocusState, modal_focus_modifier},
    modifier::{
        ClickableArgs, InteractionState, ModifierExt, SemanticsArgs, with_keyboard_input,
        with_pointer_input,
    },
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    },
    overlay_semantics::{dialog_semantics, overlay_background},
    pos_misc::is_position_inside_bounds,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    theme::MaterialTheme,
};

/// Height anchor of an open bottom sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BottomSheetExpansion {
    /// Use the partial height, or the content's natural height when
    /// unspecified.
    #[default]
    PartiallyExpanded,
    /// Fill the height available below the sheet's top margin.
    Expanded,
}

/// Controller for [`bottom_sheet_provider`], managing visibility and height.
///
/// This controller can be created by the application and passed through
/// `bottom_sheet_provider().controller(...)`. It is used to control the
/// visibility and selected height of the sheet programmatically.
#[derive(Clone, PartialEq)]
pub struct BottomSheetController {
    is_open: bool,
    motion: VisibilityMotion,
    is_dragging: bool,
    drag: SheetDrag,
    expansion: BottomSheetExpansion,
    resize: Scalar,
    resize_origin: BottomSheetExpansion,
    resize_range: Dp,
    can_resize: bool,
}

impl BottomSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            motion: VisibilityMotion::new(initial_open, true),
            is_dragging: false,
            drag: SheetDrag::new(),
            expansion: BottomSheetExpansion::PartiallyExpanded,
            resize: Scalar::new(0.0),
            resize_origin: BottomSheetExpansion::PartiallyExpanded,
            resize_range: Dp::ZERO,
            can_resize: true,
        }
    }

    /// Initiates the animation to open the bottom sheet.
    ///
    /// If the sheet is already open, this has no effect. If the sheet is
    /// currently closing, it will reverse direction and start opening from
    /// its current position.
    pub fn open(&mut self) {
        if !self.is_open {
            self.motion.retarget(true, current_frame_nanos());
            self.is_open = true;
            self.drag.settle(current_frame_nanos());
        }
    }

    /// Initiates the animation to close the bottom sheet.
    ///
    /// If the sheet is already closed, this has no effect. If the sheet is
    /// currently opening, it will reverse direction and start closing from
    /// its current position.
    pub fn close(&mut self) {
        if self.is_open {
            self.motion.retarget(false, current_frame_nanos());
            self.is_open = false;
            self.is_dragging = false;
            self.drag.settle(current_frame_nanos());
            self.resize.snap(self.expansion_fraction());
        }
    }

    /// Returns whether the sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the selected height anchor, including while the sheet is closed.
    pub fn expansion(&self) -> BottomSheetExpansion {
        self.expansion
    }

    /// Selects the expanded height without opening a closed sheet.
    pub fn expand(&mut self) {
        self.expansion = BottomSheetExpansion::Expanded;
        self.resize.snap(1.0);
    }

    /// Selects the partial height without opening a closed sheet.
    pub fn partially_expand(&mut self) {
        self.expansion = BottomSheetExpansion::PartiallyExpanded;
        self.resize.snap(0.0);
    }

    /// Switches between partial and expanded heights without changing
    /// visibility.
    pub fn toggle_expansion(&mut self) {
        match self.expansion {
            BottomSheetExpansion::PartiallyExpanded => self.expand(),
            BottomSheetExpansion::Expanded => self.partially_expand(),
        }
    }

    /// Returns whether the sheet is currently animating in either direction.
    pub fn is_animating(&self) -> bool {
        self.is_animating_at(current_frame_nanos())
    }

    fn is_animating_at(&self, frame: u64) -> bool {
        let mut resize = self.resize;
        resize.advance(frame);
        self.motion.sample(frame).active
            || (self.is_open && (self.drag.sample(frame).1 || resize.is_animating()))
    }

    fn snapshot(&self) -> (bool, VisibilitySample, f32, f64) {
        let frame = current_frame_nanos();
        let mut resize = self.resize;
        resize.advance(frame);
        (
            self.is_open,
            self.motion.sample(frame),
            self.drag_offset(),
            resize.value.clamp(0.0, 1.0),
        )
    }

    fn resize_fraction(&self) -> f64 {
        let mut resize = self.resize;
        resize.advance(current_frame_nanos());
        resize.value.clamp(0.0, 1.0)
    }

    fn expansion_fraction(&self) -> f64 {
        match self.expansion {
            BottomSheetExpansion::PartiallyExpanded => 0.0,
            BottomSheetExpansion::Expanded => 1.0,
        }
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
        if dragging {
            let frame = current_frame_nanos();
            self.drag.grab(frame);
            self.resize.advance(frame);
            self.resize.snap(self.resize.value.clamp(0.0, 1.0));
            self.resize_origin = self.expansion;
        }
    }

    fn apply_drag_delta(&mut self, delta_y: f32) -> f32 {
        if !self.is_dragging {
            self.set_dragging(true);
        }
        let frame = current_frame_nanos();
        let mut remaining = delta_y;
        let mut consumed = 0.0;
        if remaining < 0.0 && self.drag_offset() > 0.0 {
            let delta = self.drag.apply_delta(remaining, frame);
            consumed += delta;
            remaining -= delta;
        }
        if self.resize_range.0 > 0.0 {
            self.resize.advance(frame);
            let before = self.resize.value.clamp(0.0, 1.0);
            let after =
                (before - Dp::from_pixels_f32(remaining).0 / self.resize_range.0).clamp(0.0, 1.0);
            self.resize.snap(after);
            let delta = Dp((before - after) * self.resize_range.0).to_pixels_f32();
            consumed += delta;
            remaining -= delta;
        }
        if remaining > 0.0 {
            consumed += self.drag.apply_delta(remaining, frame);
        }
        consumed
    }

    fn drag_offset(&self) -> f32 {
        self.drag.sample(current_frame_nanos()).0
    }

    fn complete_drag(&mut self, velocity: f32) -> bool {
        self.is_dragging = false;
        let frame = current_frame_nanos();
        self.resize.advance(frame);
        if self.resize_range.0 > 0.0 && self.resize.value > 0.0 {
            let velocity = Dp::from_pixels_f32(velocity).0;
            let expanded = if velocity.abs() >= 125.0 {
                velocity < 0.0
            } else {
                self.resize.value >= 0.5
            };
            self.expansion = if expanded {
                BottomSheetExpansion::Expanded
            } else {
                BottomSheetExpansion::PartiallyExpanded
            };
            self.resize.velocity = -velocity / self.resize_range.0;
            self.resize
                .start_to(self.expansion_fraction(), Spec::Spatial, frame);
            self.drag.settle(frame);
            false
        } else {
            if self.resize_range.0 > 0.0 {
                self.partially_expand();
            }
            self.drag.release(frame, velocity, 56.0, 125.0)
        }
    }

    fn cancel_drag(&mut self) {
        self.is_dragging = false;
        self.expansion = self.resize_origin;
        let frame = current_frame_nanos();
        self.resize
            .retarget(self.expansion_fraction(), Spec::Spatial, frame);
        self.drag.settle(frame);
    }
}

impl Default for BottomSheetController {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Compute Y position for bottom sheet placement.
fn compute_bottom_sheet_y(
    parent_height: Px,
    child_height: Px,
    progress: f32,
    is_open: bool,
    drag_offset: f32,
) -> f32 {
    let parent = parent_height.0 as f32;
    let child = child_height.0 as f32;
    let y = if is_open {
        parent - child * progress
    } else {
        parent - child * (1.0 - progress)
    };
    y + drag_offset
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

/// Handle drag gestures on the bottom sheet.
fn handle_drag_gestures(
    controller: State<BottomSheetController>,
    drag_recognizer: State<DragRecognizer>,
    velocity_tracker: State<Option<ScrollVelocityTracker>>,
    tap_recognizer: State<TapRecognizer>,
    interaction: State<InteractionState>,
    input: &mut tessera_ui::PointerInput<'_>,
    on_close: &Callback,
) {
    let within_bounds = input
        .cursor_position_rel
        .map(|pos| is_position_inside_bounds(input.computed_data, pos))
        .unwrap_or(false);
    let now = current_frame_time().unwrap_or_else(Instant::now);
    if within_bounds
        && input.pointer_changes.iter().any(|change| {
            !change.is_consumed()
                && matches!(
                    change.content,
                    tessera_ui::CursorEventContent::Pressed(tessera_ui::PressKeyEventType::Left)
                )
        })
    {
        velocity_tracker.set(Some(ScrollVelocityTracker::new(
            now,
            Duration::from_millis(90),
            Duration::from_millis(65),
        )));
    }
    let (drag_result, cancelled) = drag_recognizer.with_mut(|recognizer| {
        let was_dragging = recognizer.is_dragging();
        let result = recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            within_bounds,
        );
        (
            result,
            was_dragging && !recognizer.is_dragging() && !result.ended,
        )
    });

    if drag_result.started {
        tap_recognizer.with_mut(TapRecognizer::cancel);
        interaction.with_mut(InteractionState::release);
        controller.with_mut(|c| c.set_dragging(true));
    }

    if drag_result.updated {
        let consumed = controller.with_mut(|c| {
            c.apply_drag_delta(input.local_delta(0.0, drag_result.delta_y.to_f32()).1)
        });
        velocity_tracker.with_mut(|tracker| {
            if let Some(tracker) = tracker {
                let window = input
                    .local_to_window
                    .map_vector(PxPosition::from_f32_xy(0.0, consumed));
                tracker.push_delta(now, window.x, window.y);
            }
        });
    }

    if drag_result.ended {
        let velocity = velocity_tracker.with_mut(|tracker| {
            tracker
                .take()
                .and_then(|mut tracker| tracker.resolve(now))
                .map(|v| v.y)
                .unwrap_or(0.0)
        });
        let velocity = input.local_delta(0.0, velocity).1;
        let should_close = controller.with_mut(|c| c.complete_drag(velocity));
        if should_close {
            on_close.call();
        }
    }
    if cancelled {
        velocity_tracker.set(None);
        controller.with_mut(BottomSheetController::cancel_drag);
    }
}

/// Place bottom sheet if present. Extracted to reduce complexity of the parent
/// function.
fn place_bottom_sheet_if_present(
    input: &mut MeasureScope<'_>,
    result: &mut LayoutResult,
    is_open: bool,
    drag_offset: f32,
    progress: f32,
    resize_fraction: f64,
    partial_height: Option<Dp>,
    controller: State<BottomSheetController>,
    resize_range: State<Option<Dp>>,
) {
    let children = input.children();
    if children.len() <= 2 {
        return;
    }

    let bottom_sheet = children[2];

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

    let max_width_px = Dp(640.0).to_px();
    let is_large_screen = parent_width > max_width_px;

    let sheet_width = if is_large_screen {
        max_width_px.min((parent_width - Dp(112.0).to_px()).max(Px::ZERO))
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

    let constraint = Constraint::new(sheet_width, AxisConstraint::new(Px::ZERO, Some(max_height)));

    let natural_size = match bottom_sheet.measure(input, &constraint) {
        Ok(s) => s,
        Err(_) => return,
    };
    let partial_height = partial_height
        .map(|height| height.to_px().max(Dp(48.0).to_px()).min(max_height))
        .unwrap_or(natural_size.height);
    let range = Dp::from(max_height - partial_height);
    if resize_range.get() != Some(range) {
        resize_range.set(Some(range));
        controller.with_mut(|controller| {
            controller.resize_range = range;
            controller.can_resize = range.0 > 0.0;
        });
    }
    let height = partial_height
        + Px::from_f32(((max_height - partial_height).to_f32() * resize_fraction as f32).round());
    let child_size = if height == natural_size.height {
        natural_size
    } else {
        match bottom_sheet.measure(
            input,
            &Constraint::new(sheet_width, AxisConstraint::exact(height)),
        ) {
            Ok(size) => size,
            Err(_) => return,
        }
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

    result.place_child(bottom_sheet, PxPosition::from_f32_xy(x.0 as f32, y));
}
#[tessera]
fn bottom_sheet_drag_handle(
    controller: Option<State<BottomSheetController>>,
    on_close: Option<Callback>,
) {
    let on_close = on_close.unwrap_or_default();
    let controller = controller.expect("bottom_sheet_drag_handle requires controller");
    let drag_recognizer = remember(DragRecognizer::default);
    let velocity_tracker = remember(|| None::<ScrollVelocityTracker>);
    let tap_recognizer = remember(TapRecognizer::default);
    let interaction = remember(InteractionState::new);
    let modifier = with_pointer_input(
        Modifier::new().clickable_with(ClickableArgs {
            enabled: controller.with(|controller| controller.can_resize),
            label: Some("Drag handle".to_string()),
            block_input: false,
            tap_recognizer: Some(tap_recognizer),
            interaction_state: Some(interaction),
            on_click: Callback::new(move || {
                controller.with_mut(BottomSheetController::toggle_expansion)
            }),
            ..Default::default()
        }),
        move |mut input| {
            handle_drag_gestures(
                controller,
                drag_recognizer,
                velocity_tracker,
                tap_recognizer,
                interaction,
                &mut input,
                &on_close,
            );
        },
    );

    layout().modifier(modifier).child(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(10.0)));
                let scheme = use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme;
                surface()
                    .style(if interaction.with(InteractionState::is_focused) {
                        SurfaceStyle::Outlined {
                            color: scheme.secondary,
                            width: Dp(2.0),
                        }
                    } else {
                        tessera_ui::Color::TRANSPARENT.into()
                    })
                    .shape(Shape::CAPSULE)
                    .modifier(Modifier::new().size(Dp(56.0), Dp(28.0)))
                    .block_input(false)
                    .content_alignment(crate::alignment::Alignment::Center)
                    .child(move || {
                        surface()
                            .style(scheme.on_surface_variant.with_alpha(0.4).into())
                            .shape(Shape::CAPSULE)
                            .modifier(Modifier::new().size(Dp(32.0), Dp(4.0)))
                            .block_input(false)
                            .child(|| {});
                    });
                spacer().modifier(Modifier::new().height(Dp(10.0)));
            });
    });
}

fn build_bottom_sheet_nested_scroll_connection(
    controller: State<BottomSheetController>,
    on_close: Callback,
    parent: Option<NestedScrollConnection>,
) -> NestedScrollConnection {
    NestedScrollConnection::new()
        .with_pre_scroll_handler(CallbackWith::new({
            move |input: PreScrollInput| {
                if input.source != tessera_ui::ScrollEventSource::Touch
                    || input.available.y >= 0.0
                    || controller.with(|c| {
                        c.drag_offset() <= 0.0
                            && (c.resize_fraction() >= 1.0 || c.resize_range.0 <= 0.0)
                    })
                {
                    return ScrollDelta::ZERO;
                }

                let consumed_y = controller.with_mut(|c| c.apply_drag_delta(input.available.y));
                ScrollDelta::new(0.0, consumed_y)
            }
        }))
        .with_post_scroll_handler(CallbackWith::new({
            move |input: PostScrollInput| {
                if input.source != tessera_ui::ScrollEventSource::Touch || input.available.y <= 0.0
                {
                    return ScrollDelta::ZERO;
                }

                let consumed_y = controller.with_mut(|c| c.apply_drag_delta(input.available.y));
                ScrollDelta::new(0.0, consumed_y)
            }
        }))
        .with_pre_fling_handler(CallbackWith::new({
            move |input: PreFlingInput| {
                if controller.with(|c| !c.is_dragging && c.drag_offset() <= 0.0) {
                    return ScrollVelocity::ZERO;
                }

                let resizing =
                    controller.with(|c| c.resize_range.0 > 0.0 && c.resize_fraction() > 0.0);
                let should_close = controller.with_mut(|c| c.complete_drag(input.available.y));
                if should_close {
                    on_close.call();
                }

                ScrollVelocity::new(
                    0.0,
                    if resizing {
                        input.available.y
                    } else {
                        input.available.y.max(0.0)
                    },
                )
            }
        }))
        .with_parent(parent)
}

#[tessera]
fn bottom_sheet_content_wrapper(
    accessibility_label: Option<String>,
    nested_scroll_connection: Option<NestedScrollConnection>,
    bottom_sheet_content: Option<RenderSlot>,
    controller: Option<State<BottomSheetController>>,
    on_close: Option<Callback>,
    just_opened: Option<bool>,
    focus_scope: Option<FocusScopeNode>,
) {
    let on_close = on_close.unwrap_or_default();
    let just_opened = just_opened.unwrap_or(false);
    let controller = controller.expect("bottom_sheet_content_wrapper requires controller");
    let bottom_sheet_content =
        bottom_sheet_content.expect("bottom_sheet_content_wrapper requires sheet content");
    let nested_scroll_connection =
        nested_scroll_connection.expect("sheet content requires its nested scroll connection");
    let focus_scope = focus_scope.expect("modal content requires its focus scope");
    let modifier = dialog_semantics(
        with_keyboard_input(
            Modifier::new()
                .focus_scope_with(focus_scope)
                .focus_properties(FocusProperties::new().skip_traversal(true))
                .focus_traversal_policy(
                    FocusTraversalPolicy::linear()
                        .wrap(true)
                        .tab_navigation(true),
                ),
            make_keyboard_closure(on_close),
        ),
        accessibility_label,
        true,
    );
    if just_opened {
        focus_scope.request_focus();
    }
    layout().modifier(modifier).child(move || {
        let bottom_sheet_content = bottom_sheet_content;
        let nested_scroll_connection = nested_scroll_connection.clone();
        let content_wrapper = move || {
            let bottom_sheet_content = bottom_sheet_content;
            let nested_scroll_connection = nested_scroll_connection.clone();
            column()
                .modifier(Modifier::new().fill_max_width())
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    bottom_sheet_drag_handle()
                        .controller(controller)
                        .on_close_shared(on_close);

                    let bottom_sheet_content = bottom_sheet_content;
                    let nested_scroll_connection = nested_scroll_connection.clone();
                    provide_context(
                        || nested_scroll_connection.clone(),
                        move || {
                            bottom_sheet_content.render();
                        },
                    );
                });
        };
        surface()
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
            .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
                move |coordinates: tessera_ui::LayoutCoordinates| {
                    let extent = Dp::from_pixels_f64(f64::from(coordinates.local_size.height.0)).0;
                    if controller.with(|c| c.drag.extent() != extent) {
                        controller.with_mut(|c| c.drag.set_extent(extent, current_frame_nanos()));
                    }
                },
            ))
            .block_input(true)
            .child(content_wrapper);
    });
}

/// # bottom_sheet_provider
///
/// Provides a modal bottom sheet for contextual actions or information.
///
/// ## Usage
///
/// Show contextual menus, supplemental information, or simple forms without
/// navigating away from the main screen.
///
/// ## Parameters
///
/// - `accessibility_label` — accessible name describing the dialog or sheet.
/// - `on_close_request` — optional callback invoked when the sheet requests
///   closing.
/// - `is_open` — declarative open state.
/// - `controller` — optional external controller for programmatic open/close.
/// - `partial_height` — optional partial height, including the drag handle;
///   defaults to the content's natural height. It is limited to the available
///   height and is at least 48dp when that space is available.
/// - `main_content` — optional main content rendered behind the sheet.
/// - `bottom_sheet_content` — optional content rendered inside the sheet.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::bottom_sheet::bottom_sheet_provider;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// bottom_sheet_provider()
///     .accessibility_label("Additional options")
///     .on_close_request(|| {})
///     .is_open(true)
///     .partial_height(tessera_ui::Dp(240.0))
///     .main_content(|| { /* main content */ })
///     .bottom_sheet_content(|| { /* bottom sheet content */ });
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn bottom_sheet_provider(
    #[prop(into)] accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    is_open: Option<bool>,
    controller: Option<State<BottomSheetController>>,
    partial_height: Option<Dp>,
    main_content: Option<RenderSlot>,
    bottom_sheet_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request.unwrap_or_default();
    let is_open = is_open.unwrap_or(false);
    let partial_height = partial_height.filter(|height| height.0.is_finite());
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let bottom_sheet_content = bottom_sheet_content.unwrap_or_else(RenderSlot::empty);
    let external_controller = controller;
    let controller =
        external_controller.unwrap_or_else(|| remember(|| BottomSheetController::new(is_open)));
    let parent_nested_scroll = use_context::<NestedScrollConnection>().map(|context| context.get());
    let coordinate_scale = remember(|| 1.0_f32);
    let nested_scroll_connection = build_bottom_sheet_nested_scroll_connection(
        controller,
        on_close_request,
        parent_nested_scroll,
    )
    .with_coordinate_scale(coordinate_scale);

    // In controlled mode (external controller provided), do not override
    // controller state from `is_open`.
    if external_controller.is_none() {
        let current_open = controller.with(|c| c.is_open());
        if is_open != current_open {
            if is_open {
                controller.with_mut(|c| c.open());
            } else {
                controller.with_mut(|c| c.close());
            }
        }
    }

    // Snapshot state to minimize locking overhead.
    let (is_open, visibility, drag_offset, resize_fraction) = controller.with(|c| c.snapshot());
    // Separate anchor feedback keeps dismissal state out of measurement
    // dependencies.
    let resize_range = remember_with_key(controller, || None::<Dp>);
    let is_animating = controller.with(|c| c.is_animating());
    let bottom_sheet_open_state = remember(|| false);
    let mut just_opened = false;
    bottom_sheet_open_state.with_mut(|was_open| {
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

    // The sheet overlay is gated by open state, but `main_content` must keep
    // rendering: providers host the rest of the page inside it.
    let show_bottom_sheet = is_open || is_animating;
    let focus_scope = remember(FocusScopeNode::new).get();
    let focus_state = remember(ModalFocusState::default);
    let modifier =
        modal_focus_modifier(Modifier::new(), focus_state, show_bottom_sheet, focus_scope);

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
        .policy(BottomSheetLayout {
            progress,
            is_open,
            drag_offset,
            resize_fraction,
            partial_height,
            controller,
            resize_range,
        })
        .child(move || {
            let bottom_sheet_content = bottom_sheet_content;
            overlay_background(main_content, show_bottom_sheet);

            if !show_bottom_sheet {
                return;
            }

            let scrim_alpha = visibility.opacity * 0.32;
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

            bottom_sheet_content_wrapper()
                .nested_scroll_connection(nested_scroll_connection.clone())
                .accessibility_label_optional(accessibility_label.clone())
                .focus_scope(focus_scope)
                .bottom_sheet_content_shared(bottom_sheet_content)
                .controller(controller)
                .on_close_shared(on_close_request)
                .just_opened(just_opened);
        });
}

#[derive(Clone, PartialEq)]
struct BottomSheetLayout {
    progress: f32,
    is_open: bool,
    drag_offset: f32,
    resize_fraction: f64,
    partial_height: Option<Dp>,
    controller: State<BottomSheetController>,
    resize_range: State<Option<Dp>>,
}

impl tessera_ui::LayoutModifierNode for BottomSheetLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.resize_fraction == other.resize_fraction
            && self.partial_height == other.partial_height
            && self.resize_range == other.resize_range
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
            let parent_width = input
                .parent_constraint()
                .width()
                .resolve_max()
                .unwrap_or(Px::ZERO);
            let parent_height = input
                .parent_constraint()
                .height()
                .resolve_max()
                .unwrap_or(Px::ZERO);
            let size = sheet.size();
            let x = if parent_width >= Dp(640.0).to_px() {
                (parent_width - size.width) / 2
            } else {
                Px::ZERO
            };
            let y = compute_bottom_sheet_y(
                parent_height,
                size.height,
                self.progress,
                self.is_open,
                self.drag_offset,
            );
            result.place_child(*sheet, PxPosition::from_f32_xy(x.0 as f32, y));
        }
        Some(result.into_placement_result())
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_constraint = input.parent_constraint().without_min();
        let main_content = children[0];
        let main_content_size = main_content.measure(input, &child_constraint)?;
        result.place_child(main_content, PxPosition::new(Px(0), Px(0)));

        if children.len() > 1 {
            let scrim = children[1];
            scrim.measure(input, &child_constraint)?;
            result.place_child(scrim, PxPosition::new(Px(0), Px(0)));
        }

        place_bottom_sheet_if_present(
            input,
            &mut result,
            self.is_open,
            self.drag_offset,
            self.progress,
            self.resize_fraction,
            self.partial_height,
            self.controller,
            self.resize_range,
        );

        Ok(result.with_size(main_content_size.size()))
    }
}

#[cfg(test)]
mod placement_tests {
    use tessera_ui::Px;

    use super::compute_bottom_sheet_y;

    #[test]
    fn reversing_directions_keeps_fractional_sheet_placement_and_drag_offsets() {
        for (open, progress) in [(true, 0.3125), (false, 0.6875)] {
            assert_eq!(
                compute_bottom_sheet_y(Px(600), Px(120), progress, open, 4.25),
                566.75
            );
        }
    }
}

#[cfg(test)]
#[path = "bottom_sheet_tests.rs"]
mod accessibility_tests;
