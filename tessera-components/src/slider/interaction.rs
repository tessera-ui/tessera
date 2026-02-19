use tessera_ui::{
    CallbackWith, ComputedData, CursorEventContent, Focus, InputHandlerInput, Px, PxPosition,
    State,
    accesskit::{Action, Role},
    winit::window::CursorIcon,
};

use super::{ACCESSIBILITY_STEP, SliderArgs, SliderController, SliderLayout};

pub(super) fn snap_fraction(value: f32, steps: usize) -> f32 {
    if steps == 0 {
        return value.clamp(0.0, 1.0);
    }
    let denom = steps as f32 + 1.0;
    let step = 1.0 / denom;
    (value / step).round().mul_add(step, 0.0).clamp(0.0, 1.0)
}

/// Helper: check if a cursor position is within the bounds of a component.
pub(super) fn cursor_within_bounds(
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
    if layout.track_total_width.0 <= 0 {
        return None;
    }
    cursor_pos.map(|pos| {
        let cursor_x = pos.x.to_f32();
        let half_handle = layout.handle_width.to_f32() / 2.0;
        let start_x = layout.handle_gap.to_f32() + half_handle;
        let fraction = (cursor_x - start_x) / layout.track_total_width.to_f32();
        fraction.clamp(0.0, 1.0)
    })
}

fn range_cursor_progress(
    cursor_pos: Option<PxPosition>,
    layout: &SliderLayout,
    start_handle_width: Px,
    end_handle_width: Px,
) -> Option<f32> {
    let cursor_pos = cursor_pos?;
    let component_width = layout.component_width.to_f32();
    let gap = layout.handle_gap.to_f32();
    let start_half = start_handle_width.to_f32() / 2.0;
    let end_half = end_handle_width.to_f32() / 2.0;
    let track_total = (component_width - start_half - end_half - gap * 2.0).max(0.0);
    if track_total <= 0.0 {
        return None;
    }
    let start_x = gap + start_half;
    let fraction = (cursor_pos.x.to_f32() - start_x) / track_total;
    Some(fraction.clamp(0.0, 1.0))
}

fn range_handle_center_x(
    layout: &SliderLayout,
    value: f32,
    start_handle_width: Px,
    end_handle_width: Px,
) -> f32 {
    let component_width = layout.component_width.to_f32();
    let gap = layout.handle_gap.to_f32();
    let start_half = start_handle_width.to_f32() / 2.0;
    let end_half = end_handle_width.to_f32() / 2.0;
    let track_total = (component_width - start_half - end_half - gap * 2.0).max(0.0);
    let start_x = gap + start_half;
    let raw = start_x + value.clamp(0.0, 1.0) * track_total;
    raw.clamp(start_x, (component_width - gap - end_half).max(start_x))
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

    let is_in_component = cursor_within_bounds(input.cursor_position_rel, &input.computed_data);

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

    handle_cursor_events(input, state, &mut new_value, layout, args.steps);
    update_value_on_drag(input, state, &mut new_value, layout, args.steps);
    notify_on_change(new_value, args);
}

fn handle_cursor_events(
    input: &mut InputHandlerInput,
    state: State<SliderController>,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
    steps: usize,
) {
    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                state.with_mut(|inner| {
                    inner.focus.request_focus();
                    inner.is_dragging = true;
                });
                if let Some(v) = cursor_progress(input.cursor_position_rel, layout) {
                    *new_value = Some(snap_fraction(v, steps));
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
    steps: usize,
) {
    if state.with(|s| s.is_dragging)
        && let Some(v) = cursor_progress(input.cursor_position_rel, layout)
    {
        *new_value = Some(snap_fraction(v, steps));
    }
}

fn notify_on_change(new_value: Option<f32>, args: &SliderArgs) {
    if let Some(v) = new_value
        && (v - args.value).abs() > f32::EPSILON
    {
        args.on_change.call(v);
    }
}

pub(super) fn apply_slider_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &SliderArgs,
    current_value: f32,
    on_change: &CallbackWith<f32>,
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

    let on_change = on_change.clone();
    let steps = args.steps;
    input.set_accessibility_action_handler(move |action| {
        let delta = if steps == 0 {
            ACCESSIBILITY_STEP
        } else {
            1.0 / (steps as f32 + 1.0)
        };
        let new_value = match action {
            Action::Increment => Some(snap_fraction(current_value + delta, steps)),
            Action::Decrement => Some(snap_fraction(current_value - delta, steps)),
            _ => None,
        };

        if let Some(new_value) = new_value
            && (new_value - current_value).abs() > f32::EPSILON
        {
            on_change.call(new_value);
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
    start_handle_width: Px,
    end_handle_width: Px,
) {
    if args.disabled {
        state.with_mut(|inner| {
            inner.is_hovered = false;
            inner.is_dragging_start = false;
            inner.is_dragging_end = false;
        });
        return;
    }

    let is_in_component = cursor_within_bounds(input.cursor_position_rel, &input.computed_data);

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
                if let Some(progress) = range_cursor_progress(
                    input.cursor_position_rel,
                    layout,
                    start_handle_width,
                    end_handle_width,
                ) {
                    let progress = snap_fraction(progress, args.steps);
                    let start_value = args.value.0.clamp(0.0, 1.0);
                    let end_value = args.value.1.clamp(start_value, 1.0);
                    let cursor_x = input.cursor_position_rel.map(|pos| pos.x.to_f32());
                    let start_center_x = range_handle_center_x(
                        layout,
                        start_value,
                        start_handle_width,
                        end_handle_width,
                    );
                    let end_center_x = range_handle_center_x(
                        layout,
                        end_value,
                        start_handle_width,
                        end_handle_width,
                    );
                    let dist_start = cursor_x.map(|x| (x - start_center_x).abs());
                    let dist_end = cursor_x.map(|x| (x - end_center_x).abs());
                    let drag_start =
                        dist_start.unwrap_or(f32::INFINITY) <= dist_end.unwrap_or(f32::INFINITY);

                    state.with_mut(|inner| {
                        if drag_start {
                            inner.is_dragging_start = true;
                            inner.focus_start.request_focus();
                        } else {
                            inner.is_dragging_end = true;
                            inner.focus_end.request_focus();
                        }
                    });

                    if drag_start {
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

    if let Some(progress) = range_cursor_progress(
        input.cursor_position_rel,
        layout,
        start_handle_width,
        end_handle_width,
    ) {
        let progress = snap_fraction(progress, args.steps);
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
        args.on_change.call((ns, args.value.1));
    }
    if let Some(ne) = new_end
        && (ne - args.value.1).abs() > f32::EPSILON
    {
        args.on_change.call((args.value.0, ne));
    }
}

pub(super) fn apply_range_slider_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &super::RangeSliderArgs,
    _current_start: f32,
    _current_end: f32,
    _on_change: &CallbackWith<(f32, f32)>,
) {
    let mut builder = input.accessibility().hidden();
    if args.disabled {
        builder = builder.disabled();
    }
    builder.commit();
}
