use tessera_foundation::gesture::{DragRecognizer, TapRecognizer};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, CallbackWith, ComputedData, FocusRequester,
    KeyboardInput, KeyboardInputModifierNode, PointerInput, Px, PxPosition, State,
    accesskit::{Action, Role},
    winit::{
        event::ElementState,
        keyboard::{Key, ModifiersState, NamedKey},
    },
};

use super::{
    ACCESSIBILITY_STEP, SliderArgs, SliderController, SliderLayout, SliderOrientation,
    axis::SliderAxis,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SliderKeyboardModel {
    Single(State<f32>),
    Start(State<(f32, f32)>),
    End(State<(f32, f32)>),
}

impl SliderKeyboardModel {
    fn snapshot(self) -> (f32, f32, f32) {
        match self {
            Self::Single(value) => (value.get(), 0.0, 1.0),
            Self::Start(value) => {
                let (start, end) = value.get();
                (start, 0.0, end)
            }
            Self::End(value) => {
                let (start, end) = value.get();
                (end, start, 1.0)
            }
        }
    }

    fn update(self, next: f32) {
        match self {
            Self::Single(value) => value.set(next),
            Self::Start(value) => value.with_mut(|value| value.0 = next),
            Self::End(value) => value.with_mut(|value| value.1 = next),
        }
    }
}

pub(super) struct SliderKeyboardModifierNode {
    pub axis: SliderAxis,
    pub model: SliderKeyboardModel,
    pub steps: usize,
    pub disabled: bool,
    pub on_change: CallbackWith<f32>,
}

fn key_target(
    axis: SliderAxis,
    key: &Key,
    value: f32,
    min: f32,
    max: f32,
    steps: usize,
) -> Option<f32> {
    let intervals = if steps == 0 {
        100
    } else {
        steps.saturating_add(1)
    };
    let delta = 1.0 / intervals as f32;
    let page = (intervals / 10).clamp(1, 10) as f32;
    let sign = if axis.reversed { -1.0 } else { 1.0 };
    let vertical = axis.orientation == SliderOrientation::Vertical;
    let next = match key {
        Key::Named(NamedKey::ArrowUp) if vertical => value - sign * delta,
        Key::Named(NamedKey::ArrowDown) if vertical => value + sign * delta,
        Key::Named(NamedKey::ArrowRight | NamedKey::ArrowLeft) if vertical => return None,
        Key::Named(NamedKey::ArrowRight) => value + sign * delta,
        Key::Named(NamedKey::ArrowLeft) => value - sign * delta,
        Key::Named(NamedKey::PageUp) if vertical => value - sign * page * delta,
        Key::Named(NamedKey::PageDown) if vertical => value + sign * page * delta,
        Key::Named(NamedKey::ArrowUp) => value + delta,
        Key::Named(NamedKey::ArrowDown) => value - delta,
        Key::Named(NamedKey::PageUp) => value + page * delta,
        Key::Named(NamedKey::PageDown) => value - page * delta,
        Key::Named(NamedKey::Home) => return Some(min),
        Key::Named(NamedKey::End) => return Some(max),
        _ => return None,
    };
    Some(snap_fraction(next, steps).clamp(min, max))
}

impl SliderKeyboardModifierNode {
    fn handle_key(&self, key: &Key, state: ElementState, modifiers: ModifiersState) -> bool {
        if self.disabled || modifiers.control_key() || modifiers.alt_key() || modifiers.super_key()
        {
            return false;
        }
        let (value, min, max) = self.model.snapshot();
        let value = if value.is_nan() {
            min
        } else {
            value.clamp(min, max)
        };
        let Some(next) = key_target(self.axis, key, value, min, max, self.steps) else {
            return false;
        };
        if state == ElementState::Pressed && (next - value).abs() > f32::EPSILON {
            self.model.update(next);
            self.on_change.call(next);
        }
        true
    }
}

impl KeyboardInputModifierNode for SliderKeyboardModifierNode {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        let modifiers = input.key_modifiers;
        input
            .keyboard_events
            .retain(|event| !self.handle_key(&event.logical_key, event.state, modifiers));
    }
}

pub(super) fn snap_fraction(value: f32, steps: usize) -> f32 {
    let value = if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    };
    if steps == 0 {
        return value;
    }
    let denom = steps as f32 + 1.0;
    (value * denom).round() / denom
}

/// Helper: check if a cursor position is within the bounds of a component.
pub(super) fn cursor_within_bounds(
    cursor_pos: Option<PxPosition>,
    computed: &ComputedData,
) -> bool {
    if let Some(pos) = cursor_pos {
        let within_x = pos.x >= 0.0 && pos.x < computed.width.0 as f32;
        let within_y = pos.y >= 0.0 && pos.y < computed.height.0 as f32;
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
        let cursor_x = layout.axis.input_main(pos, layout.component_width);
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
    let fraction =
        (layout.axis.input_main(cursor_pos, layout.component_width) - start_x) / track_total;
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
            is_in_component,
        )
    });
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
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
    accessibility.orientation = Some(args.axis.accessibility_orientation());
    accessibility.label = args.accessibility_label.clone();
    accessibility.description = args.accessibility_description.clone();
    accessibility.numeric_value = Some(current_value as f64);
    accessibility.value = args.value_label.clone().or_else(|| {
        (args.value_indicator != super::SliderValueIndicatorVisibility::Hidden)
            .then(|| format!("{}%", super::default_value_label(current_value)))
    });
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
            is_in_component,
        )
    });
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
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
    let cursor_x = cursor_pos.map(|pos| layout.axis.input_main(pos, layout.component_width));
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
    accessibility.role = Some(Role::Group);
    accessibility.label = args.accessibility_label.clone();
    accessibility.description = args.accessibility_description.clone();
    accessibility.hidden = false;
    accessibility.focusable = false;
    accessibility.merge_descendants = true;
    accessibility.disabled = args.disabled;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use super::*;

    #[test]
    fn snapped_values_match_tick_fractions_and_preserve_exact_endpoints() {
        for (value, steps, expected) in [
            (1.0, 40, 1.0),
            (0.0, 40, 0.0),
            (0.87, 6, 0.85714287),
            (0.72, 6, 0.71428573),
            (0.34, 2, 0.33333334),
            (0.66, 2, 0.6666667),
            (0.625, 3, 0.75),
            (f32::NAN, 0, 0.0),
            (f32::INFINITY, 6, 1.0),
            (f32::NEG_INFINITY, 6, 0.0),
        ] {
            assert_eq!(snap_fraction(value, steps), expected);
        }
    }

    #[test]
    fn continuous_and_discrete_keys_follow_their_step_and_page_sizes() {
        for (key, expected) in [
            (NamedKey::ArrowRight, 0.51),
            (NamedKey::ArrowUp, 0.51),
            (NamedKey::ArrowLeft, 0.49),
            (NamedKey::ArrowDown, 0.49),
            (NamedKey::Home, 0.0),
            (NamedKey::End, 1.0),
            (NamedKey::PageUp, 0.6),
            (NamedKey::PageDown, 0.4),
        ] {
            assert!(
                (key_target(SliderAxis::default(), &Key::Named(key), 0.5, 0.0, 1.0, 0).unwrap()
                    - expected)
                    .abs()
                    < 0.000001
            );
        }
        for (steps, next, page) in [(3, 0.75, 0.75), (19, 0.55, 0.6), (199, 0.505, 0.55)] {
            assert!(
                (key_target(
                    SliderAxis::default(),
                    &Key::Named(NamedKey::ArrowRight),
                    0.5,
                    0.0,
                    1.0,
                    steps
                )
                .unwrap()
                    - next)
                    .abs()
                    < 0.000001
            );
            assert!(
                (key_target(
                    SliderAxis::default(),
                    &Key::Named(NamedKey::PageUp),
                    0.5,
                    0.0,
                    1.0,
                    steps
                )
                .unwrap()
                    - page)
                    .abs()
                    < 0.000001
            );
        }
    }

    #[test]
    fn range_keys_observe_the_neighbor_without_rescaling_the_step() {
        assert_eq!(
            key_target(
                SliderAxis::default(),
                &Key::Named(NamedKey::Home),
                0.5,
                0.2,
                0.8,
                0
            ),
            Some(0.2)
        );
        assert_eq!(
            key_target(
                SliderAxis::default(),
                &Key::Named(NamedKey::End),
                0.5,
                0.2,
                0.8,
                0
            ),
            Some(0.8)
        );
        assert!(
            (key_target(
                SliderAxis::default(),
                &Key::Named(NamedKey::ArrowRight),
                0.5,
                0.2,
                0.8,
                0
            )
            .unwrap()
                - 0.51)
                .abs()
                < 0.000001
        );
        assert_eq!(
            key_target(
                SliderAxis::default(),
                &Key::Named(NamedKey::ArrowRight),
                0.75,
                0.0,
                0.8,
                3
            ),
            Some(0.8)
        );
        assert_eq!(
            key_target(
                SliderAxis::default(),
                &Key::Named(NamedKey::ArrowLeft),
                0.25,
                0.2,
                1.0,
                3
            ),
            Some(0.2)
        );
    }

    #[test]
    fn vertical_keys_follow_screen_direction_without_consuming_horizontal_arrows() {
        for reversed in [false, true] {
            let axis = SliderAxis {
                orientation: SliderOrientation::Vertical,
                reversed,
            };
            for (key, expected) in [
                (NamedKey::ArrowUp, if reversed { 0.75 } else { 0.25 }),
                (NamedKey::ArrowDown, if reversed { 0.25 } else { 0.75 }),
                (NamedKey::PageUp, if reversed { 0.75 } else { 0.25 }),
                (NamedKey::PageDown, if reversed { 0.25 } else { 0.75 }),
                (NamedKey::Home, 0.0),
                (NamedKey::End, 1.0),
            ] {
                assert_eq!(
                    key_target(axis, &Key::Named(key), 0.5, 0.0, 1.0, 3),
                    Some(expected)
                );
            }
            for key in [NamedKey::ArrowLeft, NamedKey::ArrowRight, NamedKey::Tab] {
                assert_eq!(key_target(axis, &Key::Named(key), 0.5, 0.0, 1.0, 3), None);
            }
        }
        let reversed = SliderAxis {
            reversed: true,
            ..Default::default()
        };
        assert_eq!(
            key_target(
                reversed,
                &Key::Named(NamedKey::ArrowRight),
                0.5,
                0.0,
                1.0,
                3
            ),
            Some(0.25)
        );
        assert_eq!(
            key_target(reversed, &Key::Named(NamedKey::ArrowLeft), 0.5, 0.0, 1.0, 3),
            Some(0.75)
        );
    }

    #[tessera_ui::tessera]
    fn key_dispatch_fixture() {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let recorded = changes.clone();
        let value = tessera_ui::remember(|| 0.5);
        let mut node = SliderKeyboardModifierNode {
            axis: SliderAxis::default(),
            model: SliderKeyboardModel::Single(value),
            steps: 0,
            disabled: false,
            on_change: CallbackWith::new(move |value| recorded.lock().push(value)),
        };
        assert!(node.handle_key(
            &Key::Named(NamedKey::ArrowRight),
            ElementState::Released,
            ModifiersState::empty()
        ));
        assert!(changes.lock().is_empty());
        for modifiers in [
            ModifiersState::CONTROL,
            ModifiersState::ALT,
            ModifiersState::SUPER,
        ] {
            assert!(!node.handle_key(&Key::Named(NamedKey::End), ElementState::Pressed, modifiers));
        }
        for key in [Key::Named(NamedKey::Tab), Key::Character("q".into())] {
            assert!(!node.handle_key(&key, ElementState::Pressed, ModifiersState::empty()));
        }
        for _ in 0..3 {
            assert!(node.handle_key(
                &Key::Named(NamedKey::ArrowRight),
                ElementState::Pressed,
                ModifiersState::empty()
            ));
        }
        assert_eq!(changes.lock().len(), 3);
        assert!((value.get() - 0.53).abs() < 0.000001);
        node.handle_key(
            &Key::Named(NamedKey::End),
            ElementState::Pressed,
            ModifiersState::empty(),
        );
        assert!(node.handle_key(
            &Key::Named(NamedKey::ArrowRight),
            ElementState::Pressed,
            ModifiersState::empty()
        ));
        assert_eq!(changes.lock().len(), 4);
        node.disabled = true;
        assert!(!node.handle_key(
            &Key::Named(NamedKey::Home),
            ElementState::Pressed,
            ModifiersState::empty()
        ));
        assert_eq!(value.get(), 1.0);
        let range = tessera_ui::remember(|| (0.2, 0.8));
        let start = SliderKeyboardModifierNode {
            axis: SliderAxis::default(),
            model: SliderKeyboardModel::Start(range),
            steps: 0,
            disabled: false,
            on_change: CallbackWith::new(move |_| recorded_range(range)),
        };
        let end = SliderKeyboardModifierNode {
            axis: SliderAxis::default(),
            model: SliderKeyboardModel::End(range),
            steps: 0,
            disabled: false,
            on_change: CallbackWith::new(move |_| recorded_range(range)),
        };
        for key in [
            NamedKey::ArrowRight,
            NamedKey::ArrowRight,
            NamedKey::ArrowRight,
        ] {
            assert!(start.handle_key(
                &Key::Named(key),
                ElementState::Pressed,
                ModifiersState::empty(),
            ));
        }
        assert!((range.get().0 - 0.23).abs() < 0.000001);
        end.handle_key(
            &Key::Named(NamedKey::Home),
            ElementState::Pressed,
            ModifiersState::empty(),
        );
        assert_eq!(range.get().0, range.get().1);
        start.handle_key(
            &Key::Named(NamedKey::End),
            ElementState::Pressed,
            ModifiersState::empty(),
        );
        assert_eq!(range.get().0, range.get().1);
        end.handle_key(
            &Key::Named(NamedKey::End),
            ElementState::Pressed,
            ModifiersState::empty(),
        );
        start.handle_key(
            &Key::Named(NamedKey::End),
            ElementState::Pressed,
            ModifiersState::empty(),
        );
        assert_eq!(range.get(), (1.0, 1.0));
        crate::spacer::spacer();
    }

    fn recorded_range(range: State<(f32, f32)>) {
        let (start, end) = range.get();
        assert!(start <= end);
    }

    #[test]
    fn key_dispatch_consumes_limits_and_releases_but_preserves_shortcuts_and_tab() {
        tessera_ui::testing::layout_test(|| {
            key_dispatch_fixture();
        })
        .run();
    }
}
