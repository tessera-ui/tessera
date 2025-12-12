use std::sync::Arc;

use tessera_ui::{
    ComputedData, CursorEventContent, Focus, InputHandlerInput, PxPosition, State,
    accesskit::{Action, Role},
    winit::window::CursorIcon,
};

use super::{ACCESSIBILITY_STEP, SliderArgs, SliderController, SliderLayout};

/// Helper: check if a cursor position is within the bounds of a component.
pub(super) fn cursor_within_component(
    cursor_pos: Option<PxPosition>,
    computed: &ComputedData,
) -> bool {
    if let Some(pos) = cursor_pos {
        let within_x = pos.x.0 >= 0 && pos.x.0 < computed.width.0;
        let within_y = pos.y.0 >= 0 && pos.y.0 < computed.height.0;
        within_x && within_y
    } else {
        false
    }
}

/// Helper: compute normalized progress (0.0..1.0) from cursor X and overall
/// width. Returns None when cursor is not available.
pub(super) fn cursor_progress(
    cursor_pos: Option<PxPosition>,
    layout: &SliderLayout,
) -> Option<f32> {
    if layout.component_width.0 <= 0 {
        return None;
    }
    cursor_pos.map(|pos| (pos.x.0 as f32 / layout.component_width.to_f32()).clamp(0.0, 1.0))
}

pub(super) fn handle_slider_state(
    input: &mut InputHandlerInput,
    state: State<SliderController>,
    args: &SliderArgs,
    layout: &SliderLayout,
) {
    if args.disabled {
        state.with_mut(|inner| {
            inner.is_hovered = false;
            inner.is_dragging = false;
        });
        return;
    }

    let is_in_component = cursor_within_component(input.cursor_position_rel, &input.computed_data);

    state.with_mut(|inner| {
        inner.is_hovered = is_in_component;
    });

    if is_in_component {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    if !is_in_component && !state.with(|s| s.is_dragging) {
        return;
    }

    let mut new_value: Option<f32> = None;

    handle_cursor_events(input, state, &mut new_value, layout);
    update_value_on_drag(input, state, &mut new_value, layout);
    notify_on_change(new_value, args);
}

fn handle_cursor_events(
    input: &mut InputHandlerInput,
    state: State<SliderController>,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
) {
    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                state.with_mut(|inner| {
                    inner.focus.request_focus();
                    inner.is_dragging = true;
                });
                if let Some(v) = cursor_progress(input.cursor_position_rel, layout) {
                    *new_value = Some(v);
                }
            }
            CursorEventContent::Released(_) => {
                state.with_mut(|s| s.is_dragging = false);
            }
            _ => {}
        }
    }
}

fn update_value_on_drag(
    input: &InputHandlerInput,
    state: State<SliderController>,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
) {
    if state.with(|s| s.is_dragging)
        && let Some(v) = cursor_progress(input.cursor_position_rel, layout)
    {
        *new_value = Some(v);
    }
}

fn notify_on_change(new_value: Option<f32>, args: &SliderArgs) {
    if let Some(v) = new_value
        && (v - args.value).abs() > f32::EPSILON
    {
        (args.on_change)(v);
    }
}

pub(super) fn apply_slider_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &SliderArgs,
    current_value: f32,
    on_change: &Arc<dyn Fn(f32) + Send + Sync>,
) {
    let mut builder = input.accessibility().role(Role::Slider);

    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }
    if let Some(description) = args.accessibility_description.as_ref() {
        builder = builder.description(description.clone());
    }

    builder = builder
        .numeric_value(current_value as f64)
        .numeric_range(0.0, 1.0);

    if args.disabled {
        builder = builder.disabled();
    } else {
        builder = builder
            .focusable()
            .action(Action::Increment)
            .action(Action::Decrement);
    }

    builder.commit();

    if args.disabled {
        return;
    }

    let value_for_handler = current_value;
    let on_change = on_change.clone();
    input.set_accessibility_action_handler(move |action| {
        let new_value = match action {
            Action::Increment => Some((value_for_handler + ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            Action::Decrement => Some((value_for_handler - ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            _ => None,
        };

        if let Some(new_value) = new_value
            && (new_value - value_for_handler).abs() > f32::EPSILON
        {
            on_change(new_value);
        }
    });
}

/// Controller for the `range_slider` component.
pub struct RangeSliderController {
    pub(crate) is_hovered: bool,
    pub(crate) is_dragging_start: bool,
    pub(crate) is_dragging_end: bool,
    pub(crate) focus_start: Focus,
    pub(crate) focus_end: Focus,
}

impl Default for RangeSliderController {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeSliderController {
    /// Creates a new range slider controller.
    pub fn new() -> Self {
        Self {
            is_hovered: false,
            is_dragging_start: false,
            is_dragging_end: false,
            focus_start: Focus::new(),
            focus_end: Focus::new(),
        }
    }
}

pub(super) fn handle_range_slider_state(
    input: &mut InputHandlerInput,
    state: &State<RangeSliderController>,
    args: &super::RangeSliderArgs,
    layout: &SliderLayout,
) {
    if args.disabled {
        state.with_mut(|inner| {
            inner.is_hovered = false;
            inner.is_dragging_start = false;
            inner.is_dragging_end = false;
        });
        return;
    }

    let is_in_component = cursor_within_component(input.cursor_position_rel, &input.computed_data);

    state.with_mut(|inner| {
        inner.is_hovered = is_in_component;
    });

    if is_in_component {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    let is_dragging = state.with(|s| s.is_dragging_start || s.is_dragging_end);

    if !is_in_component && !is_dragging {
        return;
    }

    let mut new_start: Option<f32> = None;
    let mut new_end: Option<f32> = None;

    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                if let Some(progress) = cursor_progress(input.cursor_position_rel, layout) {
                    let dist_start = (progress - args.value.0).abs();
                    let dist_end = (progress - args.value.1).abs();

                    state.with_mut(|inner| {
                        // Determine which handle to drag based on proximity
                        if dist_start <= dist_end {
                            inner.is_dragging_start = true;
                            inner.focus_start.request_focus();
                        } else {
                            inner.is_dragging_end = true;
                            inner.focus_end.request_focus();
                        }
                    });

                    if dist_start <= dist_end {
                        new_start = Some(progress);
                    } else {
                        new_end = Some(progress);
                    }
                }
            }
            CursorEventContent::Released(_) => {
                state.with_mut(|inner| {
                    inner.is_dragging_start = false;
                    inner.is_dragging_end = false;
                });
            }
            _ => {}
        }
    }

    if let Some(progress) = cursor_progress(input.cursor_position_rel, layout) {
        state.with(|s| {
            if s.is_dragging_start {
                new_start = Some(progress.min(args.value.1)); // Don't cross end
            } else if s.is_dragging_end {
                new_end = Some(progress.max(args.value.0)); // Don't cross start
            }
        });
    }

    if let Some(ns) = new_start
        && (ns - args.value.0).abs() > f32::EPSILON
    {
        (args.on_change)((ns, args.value.1));
    }
    if let Some(ne) = new_end
        && (ne - args.value.1).abs() > f32::EPSILON
    {
        (args.on_change)((args.value.0, ne));
    }
}

pub(super) fn apply_range_slider_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &super::RangeSliderArgs,
    _current_start: f32,
    _current_end: f32,
    _on_change: &Arc<dyn Fn((f32, f32)) + Send + Sync>,
) {
    // For range slider, we ideally need two accessibility nodes.
    // However, given current limitations, we might just expose one node or the
    // "primary" interaction. A better approach for accessibility in range
    // sliders is usually multiple children nodes. For now, let's just make the
    // container focusable but it might be confusing. To do this properly, we
    // should probably split the accessibility into the two handles in the main
    // component code by attaching accessibility info to the handle children
    // instead of the container. But the current structure attaches to the
    // container. TODO: Improve accessibility for range slider (requires
    // structural changes to expose handles as children).

    // Minimal implementation: report range as a string? or just focusable?
    // Let's skip specific numeric value reporting for the container to avoid
    // confusion, or just report the start value for now.
    let mut builder = input.accessibility().role(Role::Slider);
    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }
    if args.disabled {
        builder = builder.disabled();
    }
    builder.commit();
}
