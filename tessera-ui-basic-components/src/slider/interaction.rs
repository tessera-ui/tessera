use std::sync::Arc;

use tessera_ui::{
    ComputedData, CursorEventContent, InputHandlerInput, PxPosition,
    accesskit::{Action, Role},
    winit::window::CursorIcon,
};

use super::{ACCESSIBILITY_STEP, SliderArgs, SliderLayout, SliderState};

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

/// Helper: compute normalized progress (0.0..1.0) from cursor X and overall width.
/// Returns None when cursor is not available.
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
    state: &SliderState,
    args: &SliderArgs,
    layout: &SliderLayout,
) {
    if args.disabled {
        let mut inner = state.write();
        inner.is_hovered = false;
        inner.is_dragging = false;
        return;
    }

    let is_in_component = cursor_within_component(input.cursor_position_rel, &input.computed_data);

    {
        let mut inner = state.write();
        inner.is_hovered = is_in_component;
    }

    if is_in_component {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    if !is_in_component && !state.read().is_dragging {
        return;
    }

    let mut new_value: Option<f32> = None;

    handle_cursor_events(input, state, &mut new_value, layout);
    update_value_on_drag(input, state, &mut new_value, layout);
    notify_on_change(new_value, args);
}

fn handle_cursor_events(
    input: &mut InputHandlerInput,
    state: &SliderState,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
) {
    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                {
                    let mut inner = state.write();
                    inner.focus.request_focus();
                    inner.is_dragging = true;
                }
                if let Some(v) = cursor_progress(input.cursor_position_rel, layout) {
                    *new_value = Some(v);
                }
            }
            CursorEventContent::Released(_) => {
                state.write().is_dragging = false;
            }
            _ => {}
        }
    }
}

fn update_value_on_drag(
    input: &InputHandlerInput,
    state: &SliderState,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
) {
    if state.read().is_dragging
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
