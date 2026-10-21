//! A component that displays content sliding up from the bottom of the screen.
//!
//! ## Usage
//!
//! Used to show contextual information or actions in a modal sheet.

use tessera_foundation::gesture::DragRecognizer;
use tessera_ui::{
    AxisConstraint, Callback, CallbackWith, Constraint, Dp, FocusProperties, FocusScopeNode,
    FocusTraversalPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot,
    State, current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    animation::visibility_motion::{VisibilityMotion, VisibilitySample},
    column::column,
    modal_focus::{ModalFocusState, modal_focus_modifier},
    modifier::{ModifierExt, SemanticsArgs, with_keyboard_input, with_pointer_input},
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    },
    overlay_semantics::{dialog_semantics, overlay_background},
    pos_misc::is_position_inside_bounds,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    theme::MaterialTheme,
};

/// Controller for [`bottom_sheet_provider`], managing open/closed state.
///
/// This controller can be created by the application and passed through
/// `bottom_sheet_provider().controller(...)`. It is used to control the
/// visibility of the sheet programmatically.
#[derive(Clone, PartialEq)]
pub struct BottomSheetController {
    is_open: bool,
    motion: VisibilityMotion,
    is_dragging: bool,
    drag_offset: f32,
}

impl BottomSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            motion: VisibilityMotion::new(initial_open, true),
            is_dragging: false,
            drag_offset: 0.0,
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
            self.drag_offset = 0.0;
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
        }
    }

    /// Returns whether the sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the sheet is currently animating in either direction.
    pub fn is_animating(&self) -> bool {
        self.motion.sample(current_frame_nanos()).active
    }

    fn snapshot(&self) -> (bool, VisibilitySample, f32) {
        (
            self.is_open,
            self.motion.sample(current_frame_nanos()),
            self.drag_offset,
        )
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    fn apply_drag_delta(&mut self, delta_y: f32) -> f32 {
        let current_offset = self.drag_offset;
        let new_offset = (current_offset + delta_y).max(0.0);
        self.drag_offset = new_offset;
        new_offset - current_offset
    }

    fn drag_offset(&self) -> f32 {
        self.drag_offset
    }

    fn complete_drag(&mut self) -> bool {
        self.is_dragging = false;
        let should_close = self.drag_offset > 100.0;
        if !should_close {
            self.drag_offset = 0.0;
        }
        should_close
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
    input: &mut tessera_ui::PointerInput<'_>,
    on_close: &Callback,
) {
    let within_bounds = input
        .cursor_position_rel
        .map(|pos| is_position_inside_bounds(input.computed_data, pos))
        .unwrap_or(false);
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            within_bounds,
        )
    });

    if drag_result.started {
        controller.with_mut(|c| c.set_dragging(true));
    }

    if drag_result.updated {
        let consumed = controller.with_mut(|c| c.apply_drag_delta(drag_result.delta_y.to_f32()));
        if consumed.abs() > f32::EPSILON {
            controller.with_mut(|c| c.set_dragging(true));
        }
    }

    if drag_result.ended {
        let should_close = controller.with_mut(|c| c.complete_drag());
        if should_close {
            on_close.call();
        }
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

    let constraint = Constraint::new(sheet_width, AxisConstraint::new(Px::ZERO, Some(max_height)));

    let child_size = match bottom_sheet.measure(input, &constraint) {
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
    let modifier = with_pointer_input(Modifier::new(), move |mut input| {
        handle_drag_gestures(controller, drag_recognizer, &mut input, &on_close);
    });

    layout().modifier(modifier).child(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                spacer().modifier(Modifier::new().height(Dp(22.0)));
                surface()
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .on_surface_variant
                            .with_alpha(0.4)
                            .into(),
                    )
                    .shape(Shape::CAPSULE)
                    .modifier(Modifier::new().size(Dp(32.0), Dp(4.0)))
                    .child(|| {});
                spacer().modifier(Modifier::new().height(Dp(22.0)));
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
                    || controller.with(|c| c.drag_offset()) <= 0.0
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
                if controller.with(|c| c.drag_offset()) <= 0.0 {
                    return ScrollVelocity::ZERO;
                }

                let should_close = controller.with_mut(|c| c.complete_drag());
                if should_close {
                    on_close.call();
                }

                ScrollVelocity::new(0.0, input.available.y.max(0.0))
            }
        }))
        .with_parent(parent)
}

#[tessera]
fn bottom_sheet_content_wrapper(
    accessibility_label: Option<String>,
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
    let parent_nested_scroll = use_context::<NestedScrollConnection>().map(|context| context.get());
    let nested_scroll_connection =
        build_bottom_sheet_nested_scroll_connection(controller, on_close, parent_nested_scroll);
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
            .modifier(Modifier::new().fill_max_width())
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
    main_content: Option<RenderSlot>,
    bottom_sheet_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request.unwrap_or_default();
    let is_open = is_open.unwrap_or(false);
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let bottom_sheet_content = bottom_sheet_content.unwrap_or_else(RenderSlot::empty);
    let external_controller = controller;
    let controller =
        external_controller.unwrap_or_else(|| remember(|| BottomSheetController::new(is_open)));

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
    let (is_open, visibility, drag_offset) = controller.with(|c| c.snapshot());
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
                controller.with(|controller| controller.motion.sample(frame_nanos).active);
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
        .modifier(modifier)
        .policy(BottomSheetLayout {
            progress,
            is_open,
            drag_offset,
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
}

impl tessera_ui::LayoutModifierNode for BottomSheetLayout {
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
