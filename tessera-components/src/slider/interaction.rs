use tessera_foundation::gesture::{DragRecognizer, TapRecognizer};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, CallbackWith, ComputedData, FocusRequester,
    PointerInput, Px, PxPosition, State,
    accesskit::{Action, Role},
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
    input: &mut PointerInput,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    state: State<SliderController>,
    args: &SliderArgs,
    layout: &SliderLayout,
) {
    if args.disabled {
        let should_reset = state.with(|inner| inner.is_hovered || inner.is_dragging);
        if should_reset {
            state.with_mut(|inner| {
                inner.is_hovered = false;
                inner.is_dragging = false;
            });
        }
        return;
    }

    let is_in_component = cursor_within_bounds(input.cursor_position_rel, &input.computed_data);

    let hover_changed = state.with(|inner| inner.is_hovered != is_in_component);
    if hover_changed {
        state.with_mut(|inner| {
            inner.is_hovered = is_in_component;
        });
    }

    let tap_result = tap_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            is_in_component,
        )
    });
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            is_in_component,
        )
    });

    let mut new_value: Option<f32> = None;

    handle_press_for_slider(
        input,
        state,
        &mut new_value,
        layout,
        args.steps,
        tap_result.pressed,
    );
    handle_drag_for_slider(
        input,
        state,
        &mut new_value,
        layout,
        args.steps,
        drag_result.started,
        drag_result.updated,
    );
    if tap_result.released || drag_result.ended {
        state.with_mut(|inner| inner.is_dragging = false);
    }
    notify_on_change(new_value, args);
}

fn handle_press_for_slider(
    input: &mut PointerInput,
    state: State<SliderController>,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
    steps: usize,
    pressed: bool,
) {
    if pressed {
        state.with_mut(|inner| {
            inner.focus.request_focus();
        });
        if let Some(v) = cursor_progress(input.cursor_position_rel, layout) {
            *new_value = Some(snap_fraction(v, steps));
        }
    }
}

fn handle_drag_for_slider(
    input: &PointerInput,
    state: State<SliderController>,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
    steps: usize,
    drag_started: bool,
    drag_updated: bool,
) {
    if drag_started {
        state.with_mut(|inner| inner.is_dragging = true);
    }

    if (drag_updated || state.with(|s| s.is_dragging))
        && let Some(v) = cursor_progress(input.cursor_position_rel, layout)
    {
        *new_value = Some(snap_fraction(v, steps));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RangeSliderHandle {
    Start,
    End,
}

fn notify_on_change(new_value: Option<f32>, args: &SliderArgs) {
    if let Some(v) = new_value
        && (v - args.value).abs() > f32::EPSILON
    {
        args.on_change.call(v);
    }
}

pub(super) fn apply_slider_semantics(
    accessibility: &mut AccessibilityNode,
    action_handler: &mut Option<AccessibilityActionHandler>,
    args: &SliderArgs,
    current_value: f32,
    on_change: &CallbackWith<f32>,
) {
    accessibility.role = Some(Role::Slider);
    accessibility.label = args.accessibility_label.clone();
    accessibility.description = args.accessibility_description.clone();
    accessibility.numeric_value = Some(current_value as f64);
    accessibility.min_numeric_value = Some(0.0);
    accessibility.max_numeric_value = Some(1.0);
    accessibility.focusable = !args.disabled;
    accessibility.disabled = args.disabled;
    accessibility.actions.clear();

    if args.disabled {
        *action_handler = None;
        return;
    }

    let on_change = *on_change;
    let steps = args.steps;
    *action_handler = Some(Box::new(move |action| {
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
    }));
    accessibility.actions.push(Action::Increment);
    accessibility.actions.push(Action::Decrement);
}

/// Controller for the `range_slider` component.
pub struct RangeSliderController {
    pub(crate) is_hovered: bool,
    pub(crate) is_dragging_start: bool,
    pub(crate) is_dragging_end: bool,
    active_handle: Option<RangeSliderHandle>,
    pub(crate) focus_start: FocusRequester,
    pub(crate) focus_end: FocusRequester,
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
            active_handle: None,
            focus_start: FocusRequester::new(),
            focus_end: FocusRequester::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct RangeSliderHandleWidths {
    pub start: Px,
    pub end: Px,
}

pub(super) fn handle_range_slider_state(
    input: &mut PointerInput,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    state: &State<RangeSliderController>,
    args: &super::RangeSliderArgs,
    layout: &SliderLayout,
    handle_widths: RangeSliderHandleWidths,
) {
    if args.disabled {
        let should_reset = state
            .with(|inner| inner.is_hovered || inner.is_dragging_start || inner.is_dragging_end);
        if should_reset {
            state.with_mut(|inner| {
                inner.is_hovered = false;
                inner.is_dragging_start = false;
                inner.is_dragging_end = false;
                inner.active_handle = None;
            });
        }
        return;
    }

    let is_in_component = cursor_within_bounds(input.cursor_position_rel, &input.computed_data);

    let hover_changed = state.with(|inner| inner.is_hovered != is_in_component);
    if hover_changed {
        state.with_mut(|inner| {
            inner.is_hovered = is_in_component;
        });
    }

    let tap_result = tap_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            is_in_component,
        )
    });
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            is_in_component,
        )
    });

    let mut new_start: Option<f32> = None;
    let mut new_end: Option<f32> = None;

    if tap_result.pressed
        && let Some(progress) = range_cursor_progress(
            input.cursor_position_rel,
            layout,
            handle_widths.start,
            handle_widths.end,
        )
    {
        let progress = snap_fraction(progress, args.steps);
        let active_handle = choose_range_slider_handle(
            input.cursor_position_rel,
            layout,
            args.value.0.clamp(0.0, 1.0),
            args.value.1.clamp(args.value.0.clamp(0.0, 1.0), 1.0),
            handle_widths.start,
            handle_widths.end,
        );
        state.with_mut(|inner| {
            inner.active_handle = Some(active_handle);
            match active_handle {
                RangeSliderHandle::Start => inner.focus_start.request_focus(),
                RangeSliderHandle::End => inner.focus_end.request_focus(),
            }
        });
        match active_handle {
            RangeSliderHandle::Start => new_start = Some(progress.min(args.value.1)),
            RangeSliderHandle::End => new_end = Some(progress.max(args.value.0)),
        }
    }

    if drag_result.started {
        state.with_mut(|inner| match inner.active_handle {
            Some(RangeSliderHandle::Start) => inner.is_dragging_start = true,
            Some(RangeSliderHandle::End) => inner.is_dragging_end = true,
            None => {}
        });
    }

    if (drag_result.updated || state.with(|s| s.is_dragging_start || s.is_dragging_end))
        && let Some(progress) = range_cursor_progress(
            input.cursor_position_rel,
            layout,
            handle_widths.start,
            handle_widths.end,
        )
    {
        let progress = snap_fraction(progress, args.steps);
        state.with(|s| {
            if s.is_dragging_start {
                new_start = Some(progress.min(args.value.1)); // Don't cross end
            } else if s.is_dragging_end {
                new_end = Some(progress.max(args.value.0)); // Don't cross start
            }
        });
    }

    if tap_result.released || drag_result.ended {
        state.with_mut(|inner| {
            inner.is_dragging_start = false;
            inner.is_dragging_end = false;
            inner.active_handle = None;
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

fn choose_range_slider_handle(
    cursor_pos: Option<PxPosition>,
    layout: &SliderLayout,
    start_value: f32,
    end_value: f32,
    start_handle_width: Px,
    end_handle_width: Px,
) -> RangeSliderHandle {
    let cursor_x = cursor_pos.map(|pos| pos.x.to_f32());
    let start_center_x =
        range_handle_center_x(layout, start_value, start_handle_width, end_handle_width);
    let end_center_x =
        range_handle_center_x(layout, end_value, start_handle_width, end_handle_width);
    let dist_start = cursor_x.map(|x| (x - start_center_x).abs());
    let dist_end = cursor_x.map(|x| (x - end_center_x).abs());
    if dist_start.unwrap_or(f32::INFINITY) <= dist_end.unwrap_or(f32::INFINITY) {
        RangeSliderHandle::Start
    } else {
        RangeSliderHandle::End
    }
}

pub(super) fn apply_range_slider_semantics(
    accessibility: &mut AccessibilityNode,
    args: &super::RangeSliderArgs,
    _current_start: f32,
    _current_end: f32,
    _on_change: &CallbackWith<(f32, f32)>,
) {
    accessibility.hidden = true;
    if args.disabled {
        accessibility.disabled = true;
    }
}
