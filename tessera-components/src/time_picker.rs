//! Time picker components for selecting a clock time.
//!
//! ## Usage
//!
//! Use to let users choose a time for alarms, reminders, or schedules.

use std::{
    f32::consts::TAU,
    sync::{Arc, OnceLock},
};

use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, CursorEventContent, Dp, FocusManager,
    FocusProperties, FocusRequester, KeyboardInput, KeyboardInputModifierNode, LayoutResult,
    MeasurementError, Modifier, PointerInput, PointerInputModifierNode, PressKeyEventType, Px,
    PxPosition, RenderSlot, State,
    accesskit::{Role, Toggled},
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera,
    time::{SystemTime, UNIX_EPOCH},
    use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    column::column,
    icon::icon,
    icon_button::icon_button,
    image_vector::{ImageVectorData, TintMode},
    material_icons::filled,
    modifier::{InteractionState, ModifierExt as _, SemanticsArgs},
    pipelines::image_vector::command::ImageVectorVertex,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    text_input::{TextInputController, text_input},
    theme::{ContentColor, MaterialTheme},
};

const CLOCK_SIZE: Dp = Dp(256.0);
const CLOCK_OUTER_RADIUS: f32 = 101.0;
const CLOCK_INNER_RADIUS: f32 = 69.0;

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "time_picker/accessibility_tests.rs"]
mod accessibility_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TimeField {
    #[default]
    Hour,
    Minute,
}

/// Display modes supported by [`time_picker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimePickerDisplayMode {
    /// Touch-friendly picker layout.
    #[default]
    Picker,
    /// Input-focused layout with labels.
    Input,
}

/// Preferred arrangement of the clock dial and time controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimePickerLayout {
    /// Stack the time controls above the clock dial.
    #[default]
    Vertical,
    /// Place the time controls beside the dial when at least 524dp is
    /// available. Narrower containers fall back to the vertical
    /// arrangement.
    Horizontal,
}

/// Indicates whether the selected time is in AM or PM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayPeriod {
    /// Ante meridiem (before noon).
    Am,
    /// Post meridiem (after noon).
    Pm,
}

/// Holds the current selection for a time picker.
pub struct TimePickerState {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    display_mode: TimePickerDisplayMode,
    selection: TimeField,
}

impl TimePickerState {
    /// Creates a time picker state with the provided initial values.
    pub fn new(
        initial_hour: u8,
        initial_minute: u8,
        is_24_hour: bool,
        display_mode: TimePickerDisplayMode,
    ) -> Self {
        Self {
            hour: clamp_hour(initial_hour),
            minute: clamp_minute(initial_minute),
            is_24_hour,
            display_mode,
            selection: TimeField::Hour,
        }
    }

    /// Returns the selected hour in 24-hour form (0-23).
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// Returns the selected minute (0-59).
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// Returns whether the picker uses 24-hour mode.
    pub fn is_24_hour(&self) -> bool {
        self.is_24_hour
    }

    /// Returns the display mode.
    pub fn display_mode(&self) -> TimePickerDisplayMode {
        self.display_mode
    }

    /// Returns the period for 12-hour mode.
    pub fn period(&self) -> DayPeriod {
        if self.hour >= 12 {
            DayPeriod::Pm
        } else {
            DayPeriod::Am
        }
    }

    /// Returns the hour to display in the UI.
    pub fn hour_for_display(&self) -> u8 {
        if self.is_24_hour {
            self.hour
        } else {
            let hour = self.hour % 12;
            if hour == 0 { 12 } else { hour }
        }
    }

    /// Sets the hour, clamped to 0-23.
    pub fn set_hour(&mut self, hour: u8) {
        self.hour = clamp_hour(hour);
    }

    /// Sets the minute, clamped to 0-59.
    pub fn set_minute(&mut self, minute: u8) {
        self.minute = clamp_minute(minute);
    }

    /// Sets 24-hour mode on or off.
    pub fn set_is_24_hour(&mut self, is_24_hour: bool) {
        self.is_24_hour = is_24_hour;
    }

    /// Sets the display mode.
    pub fn set_display_mode(&mut self, mode: TimePickerDisplayMode) {
        self.display_mode = mode;
    }

    /// Toggles between picker and input modes.
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            TimePickerDisplayMode::Picker => TimePickerDisplayMode::Input,
            TimePickerDisplayMode::Input => TimePickerDisplayMode::Picker,
        };
    }

    /// Updates the period when in 12-hour mode.
    pub fn set_period(&mut self, period: DayPeriod) {
        if self.is_24_hour {
            return;
        }
        let is_pm = self.hour >= 12;
        match (period, is_pm) {
            (DayPeriod::Am, true) => self.hour = self.hour.saturating_sub(12),
            (DayPeriod::Pm, false) => self.hour = (self.hour + 12).min(23),
            _ => {}
        }
    }

    /// Increments the hour by the given step, wrapping around.
    pub fn increment_hour(&mut self, step: u8) {
        let step = normalize_step(step, 23);
        self.hour = ((self.hour as u16 + step as u16) % 24) as u8;
    }

    /// Decrements the hour by the given step, wrapping around.
    pub fn decrement_hour(&mut self, step: u8) {
        let step = normalize_step(step, 23) as i16;
        let value = (self.hour as i16 - step).rem_euclid(24);
        self.hour = value as u8;
    }

    /// Increments the minute by the given step, wrapping around.
    pub fn increment_minute(&mut self, step: u8) {
        let step = normalize_step(step, 59);
        self.minute = ((self.minute as u16 + step as u16) % 60) as u8;
    }

    /// Decrements the minute by the given step, wrapping around.
    pub fn decrement_minute(&mut self, step: u8) {
        let step = normalize_step(step, 59) as i16;
        let value = (self.minute as i16 - step).rem_euclid(60);
        self.minute = value as u8;
    }

    fn snapshot(&self) -> TimePickerSnapshot {
        TimePickerSnapshot {
            hour: self.hour,
            minute: self.minute,
            is_24_hour: self.is_24_hour,
            display_mode: self.display_mode,
            selection: self.selection,
        }
    }
}

impl Default for TimePickerState {
    fn default() -> Self {
        let (hour, minute) = current_time_utc();
        TimePickerState::new(hour, minute, false, TimePickerDisplayMode::Picker)
    }
}

#[derive(Clone, PartialEq)]
struct TimePickerSnapshot {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    display_mode: TimePickerDisplayMode,
    selection: TimeField,
}

/// # time_picker
///
/// Render a time picker for selecting an hour and minute value.
///
/// ## Usage
///
/// Use when users need to choose a time for reminders or scheduling.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the time picker.
/// - `initial_hour` — initial hour for the internal state.
/// - `initial_minute` — initial minute for the internal state.
/// - `is_24_hour` — whether the internal state uses 24-hour mode.
/// - `display_mode` — initial display mode for the internal state.
/// - `hour_step` — pointer snapping interval for hour changes.
/// - `minute_step` — pointer snapping interval for minute changes.
/// - `state` — optional external state for selected time and display mode.
/// - `layout` — preferred dial arrangement; defaults to vertical.
///
/// Labeled clock values and manual entry select their exact value. Pointer
/// snapping steps apply to selecting a position on the dial.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::time_picker::{TimePickerLayout, TimePickerState, time_picker};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// time_picker().layout(TimePickerLayout::Horizontal);
///
/// let state = TimePickerState::default();
/// assert!(state.hour() <= 23);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn time_picker(
    modifier: Option<Modifier>,
    initial_hour: Option<u8>,
    initial_minute: Option<u8>,
    is_24_hour: Option<bool>,
    display_mode: Option<TimePickerDisplayMode>,
    hour_step: Option<u8>,
    minute_step: Option<u8>,
    state: Option<State<TimePickerState>>,
    layout: Option<TimePickerLayout>,
) {
    let modifier = modifier.unwrap_or_default();
    let initial_hour = initial_hour.unwrap_or(0);
    let initial_minute = initial_minute.unwrap_or(0);
    let is_24_hour = is_24_hour.unwrap_or(false);
    let display_mode = display_mode.unwrap_or_default();
    let hour_step = hour_step.unwrap_or(1);
    let minute_step = minute_step.unwrap_or(1);
    let state = if let Some(state) = state {
        state
    } else {
        remember(|| TimePickerState::new(initial_hour, initial_minute, is_24_hour, display_mode))
    };
    time_picker_content(
        state,
        modifier,
        hour_step,
        minute_step,
        layout.unwrap_or_default(),
    );
}

#[tessera]
fn time_picker_content(
    state: State<TimePickerState>,
    modifier: Modifier,
    hour_step: u8,
    minute_step: u8,
    arrangement: TimePickerLayout,
) {
    let snapshot = state.with(TimePickerState::snapshot);
    let input = snapshot.display_mode == TimePickerDisplayMode::Input;
    #[cfg(test)]
    let modifier = modifier.push_semantics(TimePickerTag("time-picker"));
    layout()
        .modifier(modifier)
        .policy(TimePickerMeasure {
            arrangement,
            input,
            is_24_hour: snapshot.is_24_hour,
        })
        .child(move || {
            time_number(state, TimeField::Hour, input);
            let theme = use_context::<MaterialTheme>().unwrap().get();
            surface()
                .modifier(Modifier::new().size(Dp(24.0), Dp(if input { 72.0 } else { 80.0 })))
                .style(Color::TRANSPARENT.into())
                .content_alignment(Alignment::Center)
                .child(move || {
                    text().content(":").style(theme.typography.display_large);
                });
            time_number(state, TimeField::Minute, input);
            if !snapshot.is_24_hour {
                time_period_selector(state, input);
            }
            if !input {
                time_clock(state, hour_step, minute_step);
            }
        });
}

#[cfg(test)]
#[derive(Clone, PartialEq)]
struct TimePickerTag(&'static str);

#[cfg(test)]
impl tessera_ui::SemanticsModifierNode for TimePickerTag {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.key = Some(self.0.to_owned());
    }
}

#[derive(Clone, PartialEq)]
struct TimePickerMeasure {
    arrangement: TimePickerLayout,
    input: bool,
    is_24_hour: bool,
}

impl tessera_ui::LayoutModifierNode for TimePickerMeasure {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let horizontal = !self.input
            && self.arrangement != TimePickerLayout::Vertical
            && parent
                .width
                .max
                .is_none_or(|width| width >= Dp(524.0).to_px());
        let width = if horizontal {
            524.0
        } else if self.is_24_hour {
            256.0
        } else {
            280.0
        };
        let width = parent.width.clamp(Dp(width).to_px());
        let field_width = Dp(if self.is_24_hour && !horizontal && !self.input {
            114.0
        } else {
            96.0
        })
        .to_px();
        let field_height = Dp(if self.input { 72.0 } else { 80.0 }).to_px();
        let fields_width = field_width * 2 + Dp(24.0).to_px();
        let natural_height = if self.input {
            Dp(92.0).to_px()
        } else if horizontal {
            CLOCK_SIZE.to_px()
        } else {
            Dp(372.0).to_px()
        };
        let height = parent.height.clamp(natural_height);
        let mut result = LayoutResult::new(ComputedData { width, height });
        let children = scope.children();
        let controls_width = fields_width
            + if !horizontal && !self.is_24_hour {
                Dp(64.0).to_px()
            } else {
                Px::ZERO
            };
        let controls_x = if horizontal {
            Px::ZERO
        } else {
            (width - controls_width).max(Px::ZERO) / 2
        };
        let controls_y = if horizontal {
            (height - CLOCK_SIZE.to_px()).max(Px::ZERO) / 2
                + Dp(if self.is_24_hour { 88.0 } else { 72.0 }).to_px()
        } else {
            Px::ZERO
        };
        for (index, child) in children.iter().copied().take(3).enumerate() {
            let child_width = if index == 1 {
                Dp(24.0).to_px()
            } else {
                field_width
            };
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(child_width), AxisConstraint::NONE),
            )?;
            let x = match index {
                0 => Px::ZERO,
                1 => field_width,
                _ => field_width + Dp(24.0).to_px(),
            };
            result.place_child(child, PxPosition::new(controls_x + x, controls_y));
        }
        if !self.is_24_hour {
            let period = children[3];
            let (period_width, period_height, x, y) = if horizontal {
                (
                    Dp(216.0).to_px(),
                    Dp(38.0).to_px(),
                    controls_x,
                    controls_y + field_height + Dp(16.0).to_px(),
                )
            } else {
                (
                    Dp(52.0).to_px(),
                    field_height,
                    controls_x + fields_width + Dp(12.0).to_px(),
                    controls_y,
                )
            };
            period.measure(
                scope,
                &Constraint::new(
                    AxisConstraint::exact(period_width),
                    AxisConstraint::exact(period_height),
                ),
            )?;
            result.place_child(period, PxPosition::new(x, y));
        }
        if !self.input {
            let clock = *children.last().expect("clock dial");
            clock.measure(scope, &Constraint::NONE)?;
            let (x, y) = if horizontal {
                (
                    Dp(268.0).to_px(),
                    (height - CLOCK_SIZE.to_px()).max(Px::ZERO) / 2,
                )
            } else {
                (
                    (width - CLOCK_SIZE.to_px()).max(Px::ZERO) / 2,
                    Dp(116.0).to_px(),
                )
            };
            result.place_child(clock, PxPosition::new(x, y));
        }
        Ok(result)
    }
}

#[tessera]
fn time_number(state: State<TimePickerState>, field: TimeField, input: bool) {
    let snapshot = state.with(TimePickerState::snapshot);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let selected = snapshot.selection == field;
    let value = if field == TimeField::Hour {
        hour_for_display(snapshot.hour, snapshot.is_24_hour)
    } else {
        snapshot.minute
    };
    let width = if snapshot.is_24_hour && !input {
        114.0
    } else {
        96.0
    };
    let height = if input { 72.0 } else { 80.0 };
    let label = if field == TimeField::Hour {
        "Hour"
    } else {
        "Minute"
    };
    let background = if selected && !input {
        scheme.primary_container
    } else {
        scheme.surface_container_highest
    };
    let foreground = if selected && !input {
        scheme.on_primary_container
    } else {
        scheme.on_surface
    };
    column().children(move || {
        if input {
            let controller = remember(|| {
                let mut controller = TextInputController::new(Dp(45.0), Some(Dp(52.0)));
                controller.set_text(&format_two_digit(value));
                controller.with_editor_mut(|editor| {
                    use glyphon::Edit;
                    editor.with_buffer_mut(|buffer| {
                        for line in &mut buffer.lines {
                            line.set_align(Some(glyphon::cosmic_text::Align::Center));
                        }
                    });
                });
                controller
            });
            let previous = remember(|| format_two_digit(value));
            let was_focused = remember(|| false);
            let focused = controller.with(|c| c.focus_handler().is_focused());
            if focused != was_focused.get() {
                was_focused.set(focused);
                if focused {
                    state.with_mut(|s| s.selection = field);
                    controller.with_mut(|c| c.select_all());
                }
            }
            if !focused && controller.with(|c| c.text()) != format_two_digit(value) {
                controller.with_mut(|c| c.set_text(&format_two_digit(value)));
                previous.set(format_two_digit(value));
            }
            text_input()
                .modifier(Modifier::new().size(Dp(width), Dp(height)))
                .controller(controller)
                .min_width(Dp(width))
                .min_height(Dp(height))
                .padding(Dp(10.0))
                .style(theme.typography.display_medium)
                .font_size(Dp(45.0))
                .line_height(Dp(52.0))
                .background_color(background)
                .focus_background_color(scheme.primary_container)
                .border_width(Dp(0.0))
                .focus_border_width(Dp(2.0))
                .focus_border_color(scheme.primary)
                .shape(theme.shapes.small)
                .text_color(foreground)
                .accessibility_label(label)
                .single_line(true)
                .on_change(move |text| {
                    if text.len() > 2 || !text.bytes().all(|ch| ch.is_ascii_digit()) {
                        return previous.get();
                    }
                    if let Ok(number) = text.parse::<u8>() {
                        let is_24_hour = state.with(TimePickerState::is_24_hour);
                        let valid = match field {
                            TimeField::Hour => {
                                if is_24_hour {
                                    number <= 23
                                } else {
                                    (1..=12).contains(&number)
                                }
                            }
                            TimeField::Minute => number <= 59,
                        };
                        if !valid {
                            return previous.get();
                        }
                        state.with_mut(|s| match field {
                            TimeField::Hour => s.set_hour(if is_24_hour {
                                number
                            } else {
                                number % 12 + if s.period() == DayPeriod::Pm { 12 } else { 0 }
                            }),
                            TimeField::Minute => s.set_minute(number),
                        });
                    }
                    previous.set(text.clone());
                    text
                });
            spacer().modifier(Modifier::new().height(Dp(4.0)));
            text()
                .content(label)
                .style(theme.typography.body_small)
                .color(scheme.on_surface_variant);
        } else {
            surface()
                .modifier(
                    Modifier::new()
                        .size(Dp(width), Dp(height))
                        .semantics(SemanticsArgs {
                            value: Some(format_two_digit(value)),
                            selected: Some(selected),
                            ..Default::default()
                        }),
                )
                .style(background.into())
                .shape(theme.shapes.small)
                .content_alignment(Alignment::Center)
                .accessibility_label(label)
                .on_click(move || {
                    state.with_mut(|s| s.selection = field);
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text()
                        .content(format_two_digit(value))
                        .style(theme.typography.display_large)
                        .color(foreground);
                });
        }
    });
}

#[tessera]
fn time_period_selector(state: State<TimePickerState>, input: bool) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let period = state.with(TimePickerState::period);
    let requesters = remember(|| [FocusRequester::new(), FocusRequester::new()]).get();
    let interactions = [
        remember(InteractionState::new),
        remember(InteractionState::new),
    ];
    layout()
        .policy(PeriodMeasure { input })
        .modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: cfg!(test).then(|| "time-period".to_owned()),
            role: Some(Role::RadioGroup),
            label: Some("AM/PM".to_owned()),
            merge_descendants: true,
            ..Default::default()
        }))
        .child(move || {
            layout()
                .policy(PeriodVisualMeasure { input })
                .modifier(
                    Modifier::new()
                        .clip_shape(theme.shapes.small)
                        .semantics(SemanticsArgs::clear()),
                )
                .child(move || {
                    for (index, (label, value)) in [("AM", DayPeriod::Am), ("PM", DayPeriod::Pm)]
                        .into_iter()
                        .enumerate()
                    {
                        let selected = value == period;
                        surface()
                            .style(
                                if selected {
                                    scheme.tertiary_container
                                } else {
                                    Color::TRANSPARENT
                                }
                                .into(),
                            )
                            .shape(Shape::RECTANGLE)
                            .interaction_state(interactions[index])
                            .show_ripple(false)
                            .block_input(false)
                            .content_alignment(Alignment::Center)
                            .child(move || {
                                text()
                                    .content(label)
                                    .style(theme.typography.title_medium)
                                    .color(if selected {
                                        scheme.on_tertiary_container
                                    } else {
                                        scheme.on_surface_variant
                                    });
                            });
                    }
                    surface()
                        .shape(theme.shapes.small)
                        .block_input(false)
                        .style(SurfaceStyle::Outlined {
                            color: scheme.outline,
                            width: Dp(1.0),
                        });
                    surface()
                        .shape(Shape::RECTANGLE)
                        .block_input(false)
                        .style(scheme.outline.into());
                });
            for (index, (label, value)) in [("AM", DayPeriod::Am), ("PM", DayPeriod::Pm)]
                .into_iter()
                .enumerate()
            {
                surface()
                    .modifier(
                        Modifier::new()
                            .semantics(SemanticsArgs::clear())
                            .push_keyboard_input(PeriodNavigation {
                                state,
                                requesters,
                                index,
                            }),
                    )
                    .style(Color::TRANSPARENT.into())
                    .shape(Shape::RECTANGLE)
                    .interaction_state(interactions[index])
                    .show_state_layer(false)
                    .show_ripple(false)
                    .accessibility_label(label)
                    .focus_requester(requesters[index])
                    .focus_properties(FocusProperties::new().skip_traversal(value != period))
                    .accessibility_role(Role::RadioButton)
                    .accessibility_toggled(if value == period {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .on_click(move || {
                        state.with_mut(|state| state.set_period(value));
                        FocusManager::current().defer_keyboard_until_layout();
                    });
            }
        });
}

#[derive(Clone, PartialEq)]
struct PeriodMeasure {
    input: bool,
}

impl tessera_ui::LayoutModifierNode for PeriodMeasure {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(52.0).to_px());
        let height = parent
            .height
            .clamp(Dp(if self.input { 72.0 } else { 80.0 }).to_px());
        let horizontal = width > height;
        let mut result = LayoutResult::new(ComputedData { width, height });
        let children = scope.children();
        children[0].measure(
            scope,
            &Constraint::new(AxisConstraint::exact(width), AxisConstraint::exact(height)),
        )?;
        result.place_child(children[0], PxPosition::ZERO);
        let target_height = Dp(48.0).to_px();
        // Targets remain outside the visual clip so compact toggles keep 48dp
        // hit areas.
        for (index, child) in children.iter().copied().skip(1).enumerate() {
            let first = if horizontal { width / 2 } else { height / 2 };
            let (w, h, x, y) = if horizontal {
                (
                    if index == 0 { first } else { width - first },
                    target_height,
                    if index == 0 { Px::ZERO } else { first },
                    (height - target_height) / 2,
                )
            } else {
                (
                    width,
                    target_height,
                    Px::ZERO,
                    if index == 0 {
                        first - target_height
                    } else {
                        first
                    },
                )
            };
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(w), AxisConstraint::exact(h)),
            )?;
            result.place_child(child, PxPosition::new(x, y));
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct PeriodVisualMeasure {
    input: bool,
}

impl tessera_ui::LayoutModifierNode for PeriodVisualMeasure {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(52.0).to_px());
        let height = parent
            .height
            .clamp(Dp(if self.input { 72.0 } else { 80.0 }).to_px());
        let horizontal = width > height;
        let mut result = LayoutResult::new(ComputedData { width, height });
        let children = scope.children();
        let first = if horizontal { width / 2 } else { height / 2 };
        for (index, child) in children.iter().copied().take(2).enumerate() {
            let (w, h, x, y) = if horizontal {
                (
                    if index == 0 { first } else { width - first },
                    height,
                    if index == 0 { Px::ZERO } else { first },
                    Px::ZERO,
                )
            } else {
                (
                    width,
                    if index == 0 { first } else { height - first },
                    Px::ZERO,
                    if index == 0 { Px::ZERO } else { first },
                )
            };
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(w), AxisConstraint::exact(h)),
            )?;
            result.place_child(child, PxPosition::new(x, y));
        }
        children[2].measure(
            scope,
            &Constraint::new(AxisConstraint::exact(width), AxisConstraint::exact(height)),
        )?;
        result.place_child(children[2], PxPosition::ZERO);
        let thickness = Dp(1.0).to_px();
        let (w, h, x, y) = if horizontal {
            (thickness, height, first, Px::ZERO)
        } else {
            (width, thickness, Px::ZERO, first)
        };
        children[3].measure(
            scope,
            &Constraint::new(AxisConstraint::exact(w), AxisConstraint::exact(h)),
        )?;
        result.place_child(children[3], PxPosition::new(x, y));
        Ok(result)
    }
}

#[derive(Clone, Copy)]
struct PeriodNavigation {
    state: State<TimePickerState>,
    requesters: [FocusRequester; 2],
    index: usize,
}

impl KeyboardInputModifierNode for PeriodNavigation {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
            || input.key_modifiers.shift_key()
        {
            return;
        }
        for event in input.keyboard_events.iter() {
            if event.state != winit::event::ElementState::Pressed {
                continue;
            }
            let index = match event.logical_key {
                winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::ArrowLeft
                    | winit::keyboard::NamedKey::ArrowRight
                    | winit::keyboard::NamedKey::ArrowUp
                    | winit::keyboard::NamedKey::ArrowDown,
                ) => 1 - self.index,
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home) => 0,
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::End) => 1,
                _ => continue,
            };
            self.state.with_mut(|state| {
                state.set_period(if index == 0 {
                    DayPeriod::Am
                } else {
                    DayPeriod::Pm
                });
            });
            self.requesters[index].request_focus();
            FocusManager::current().defer_keyboard_until_layout();
            input.block_keyboard();
            return;
        }
    }
}

fn clock_track(inner: bool) -> ImageVectorData {
    static TRACKS: OnceLock<[ImageVectorData; 2]> = OnceLock::new();
    TRACKS.get_or_init(|| {
        [CLOCK_OUTER_RADIUS, CLOCK_INNER_RADIUS].map(|radius| {
            let vertices = [
                [127.0, 128.0 - radius],
                [129.0, 128.0 - radius],
                [129.0, 128.0],
                [127.0, 128.0],
            ]
            .map(|position| ImageVectorVertex {
                position: [position[0] / 256.0, position[1] / 256.0],
                color: Color::WHITE,
            });
            ImageVectorData::new(
                256.0,
                256.0,
                Arc::new(vertices.to_vec()),
                Arc::new(vec![0, 1, 2, 0, 2, 3]),
            )
        })
    })[usize::from(inner)]
    .clone()
}

#[derive(Clone, PartialEq)]
struct ClockLayout {
    hour: u8,
    minute: u8,
    is_24_hour: bool,
    selection: TimeField,
}

impl ClockLayout {
    fn inner(&self) -> bool {
        self.selection == TimeField::Hour && self.is_24_hour && (self.hour == 0 || self.hour > 12)
    }
    fn angle(&self) -> f32 {
        if self.selection == TimeField::Hour {
            (self.hour % 12) as f32 * TAU / 12.0
        } else {
            self.minute as f32 * TAU / 60.0
        }
    }
}

impl tessera_ui::LayoutModifierNode for ClockLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let size = CLOCK_SIZE.to_px();
        let mut result = LayoutResult::new(ComputedData {
            width: size,
            height: size,
        });
        let children = scope.children();
        let factor = CLOCK_SIZE.to_pixels_f32() / 256.0;
        for (index, child) in children.iter().copied().enumerate() {
            let measured = child.measure(scope, &Constraint::NONE)?;
            let (x, y) = match index {
                0 | 1 => (128.0, 128.0),
                2 => {
                    let radius = if self.inner() {
                        CLOCK_INNER_RADIUS
                    } else {
                        CLOCK_OUTER_RADIUS
                    };
                    (
                        128.0 + radius * self.angle().sin(),
                        128.0 - radius * self.angle().cos(),
                    )
                }
                3 => (128.0, 128.0),
                _ => {
                    let slot = index - 4;
                    let radius = if slot >= 12 {
                        CLOCK_INNER_RADIUS
                    } else {
                        CLOCK_OUTER_RADIUS
                    };
                    let angle = (slot % 12) as f32 * TAU / 12.0;
                    (128.0 + radius * angle.sin(), 128.0 - radius * angle.cos())
                }
            };
            result.place_child(
                child,
                PxPosition::from_f32_xy(
                    x * factor - measured.width.to_f32() / 2.0,
                    y * factor - measured.height.to_f32() / 2.0,
                ),
            );
        }
        Ok(result)
    }
}

#[derive(Clone, Copy)]
struct ClockInput {
    state: State<TimePickerState>,
    dragging: State<bool>,
    hour_step: u8,
    minute_step: u8,
    focus: State<ClockFocus>,
}

impl PointerInputModifierNode for ClockInput {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        if input.pointer_changes.is_empty() {
            return;
        }
        if input
            .pointer_changes
            .iter()
            .any(|c| matches!(c.content, CursorEventContent::Cancelled(_)))
        {
            self.dragging.set(false);
            return;
        }
        let Some(position) = input.cursor_position_rel else {
            if input.has_unconsumed_release() {
                self.dragging.set(false);
            }
            return;
        };
        let pressed = input.pointer_changes.iter().any(|c| {
            !c.is_consumed()
                && matches!(
                    c.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
        });
        let released = input.has_unconsumed_release();
        if pressed {
            let width = input.computed_data.width.to_f32();
            let height = input.computed_data.height.to_f32();
            if position.x < 0.0 || position.y < 0.0 || position.x >= width || position.y >= height {
                return;
            }
            self.dragging.set(true);
        }
        if !self.dragging.get() {
            return;
        }
        let center = input.computed_data.width.to_f32() / 2.0;
        let dx = position.x - center;
        let dy = position.y - center;
        let angle = dx.atan2(-dy).rem_euclid(TAU);
        self.state.with_mut(|s| match s.selection {
            TimeField::Hour => {
                let index = (angle * 12.0 / TAU).round() as u8 % 12;
                let hour = if s.is_24_hour {
                    if dx.hypot(dy) < center * 85.0 / 128.0 {
                        if index == 0 { 0 } else { index + 12 }
                    } else if index == 0 {
                        12
                    } else {
                        index
                    }
                } else {
                    index + if s.period() == DayPeriod::Pm { 12 } else { 0 }
                };
                s.set_hour(
                    (hour / normalize_step(self.hour_step, 23))
                        * normalize_step(self.hour_step, 23),
                );
                if released {
                    s.selection = TimeField::Minute;
                }
            }
            TimeField::Minute => {
                let step = normalize_step(self.minute_step, 59);
                let minute = (angle * 60.0 / TAU).round() as u8 % 60;
                s.set_minute((minute / step) * step);
            }
        });
        let slot = self
            .state
            .with(|state| selected_clock_slot(&state.snapshot()));
        let requester = self.focus.with_mut(|focus| {
            focus.anchor = slot;
            focus.requesters[slot]
        });
        requester.request_focus();
        if released {
            self.dragging.set(false);
            FocusManager::current().defer_keyboard_until_layout();
        }
        input.consume_pointer_changes();
    }
}

#[derive(Clone, Copy)]
struct ClockFocus {
    requesters: [FocusRequester; 24],
    anchor: usize,
    field: TimeField,
}

impl ClockFocus {
    fn new(snapshot: &TimePickerSnapshot) -> Self {
        Self {
            requesters: std::array::from_fn(|_| FocusRequester::new()),
            anchor: selected_clock_slot(snapshot),
            field: snapshot.selection,
        }
    }
}

fn selected_clock_slot(snapshot: &TimePickerSnapshot) -> usize {
    match snapshot.selection {
        TimeField::Hour if snapshot.is_24_hour && snapshot.hour == 0 => 12,
        TimeField::Hour if snapshot.is_24_hour && snapshot.hour > 12 => snapshot.hour as usize,
        TimeField::Hour => (snapshot.hour % 12) as usize,
        TimeField::Minute => ((snapshot.minute as f32 / 5.0).round() as usize) % 12,
    }
}

fn clock_value_number(field: TimeField, index: usize) -> u8 {
    match (field, index) {
        (TimeField::Minute, _) => index as u8 * 5,
        (TimeField::Hour, 0) => 12,
        (TimeField::Hour, 12) => 0,
        (TimeField::Hour, _) => index as u8,
    }
}

fn select_clock_slot(state: State<TimePickerState>, index: usize) {
    state.with_mut(|state| {
        let number = clock_value_number(state.selection, index);
        match state.selection {
            TimeField::Hour => {
                let hour = if state.is_24_hour {
                    number
                } else {
                    number % 12
                        + if state.period() == DayPeriod::Pm {
                            12
                        } else {
                            0
                        }
                };
                state.set_hour(hour);
            }
            TimeField::Minute => {
                state.set_minute(number);
            }
        }
    });
}

#[derive(Clone, Copy)]
struct ClockNavigation {
    focus: State<ClockFocus>,
    index: usize,
    count: usize,
}

impl KeyboardInputModifierNode for ClockNavigation {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
            || input.key_modifiers.shift_key()
        {
            return;
        }
        for event in input.keyboard_events.iter() {
            if event.state != winit::event::ElementState::Pressed {
                continue;
            }
            let target = match event.logical_key {
                winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::ArrowRight | winit::keyboard::NamedKey::ArrowDown,
                ) => (self.index + 1) % self.count,
                winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::ArrowLeft | winit::keyboard::NamedKey::ArrowUp,
                ) => (self.index + self.count - 1) % self.count,
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home) => 0,
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::End) => self.count - 1,
                _ => continue,
            };
            let requester = self.focus.with_mut(|focus| {
                focus.anchor = target;
                focus.requesters[target]
            });
            requester.request_focus();
            FocusManager::current().defer_keyboard_until_layout();
            input.block_keyboard();
            return;
        }
    }
}

#[tessera]
fn time_clock(state: State<TimePickerState>, hour_step: u8, minute_step: u8) {
    let snapshot = state.with(TimePickerState::snapshot);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let dragging = remember(|| false);
    let focus = remember(|| ClockFocus::new(&snapshot));
    let count = if snapshot.is_24_hour && snapshot.selection == TimeField::Hour {
        24
    } else {
        12
    };
    let current = focus.get();
    if current.field != snapshot.selection
        || current.anchor >= count
        || !current.requesters.iter().any(FocusRequester::is_focused)
    {
        let anchor = selected_clock_slot(&snapshot);
        if current.field != snapshot.selection || current.anchor != anchor {
            focus.with_mut(|focus| {
                focus.anchor = anchor;
                focus.field = snapshot.selection;
            });
        }
    }
    let policy = ClockLayout {
        hour: snapshot.hour,
        minute: snapshot.minute,
        is_24_hour: snapshot.is_24_hour,
        selection: snapshot.selection,
    };
    let interaction = ClockInput {
        state,
        dragging,
        hour_step,
        minute_step,
        focus,
    };
    layout()
        .modifier(
            Modifier::new()
                .size(CLOCK_SIZE, CLOCK_SIZE)
                .semantics(SemanticsArgs {
                    test_tag: cfg!(test).then(|| "time-clock".to_owned()),
                    role: Some(Role::Group),
                    label: Some(
                        if snapshot.selection == TimeField::Hour {
                            "Hours"
                        } else {
                            "Minutes"
                        }
                        .to_owned(),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                })
                .focus_restorer(Some(focus.with(|focus| focus.requesters[focus.anchor])))
                .push_pointer_preview_input(interaction),
        )
        .policy(policy.clone())
        .child(move || {
            surface()
                .modifier(Modifier::new().size(CLOCK_SIZE, CLOCK_SIZE))
                .style(scheme.surface_container_highest.into())
                .shape(Shape::Ellipse)
                .child(|| {});
            icon()
                .vector(clock_track(policy.inner()))
                .size(CLOCK_SIZE)
                .rotation(policy.angle().to_degrees())
                .tint(scheme.primary)
                .tint_mode(TintMode::Solid);
            surface()
                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                .style(scheme.primary.into())
                .shape(Shape::Ellipse)
                .content_alignment(Alignment::Center)
                .child(move || {
                    if snapshot.selection == TimeField::Minute && snapshot.minute % 5 != 0 {
                        surface()
                            .modifier(Modifier::new().size(Dp(4.0), Dp(4.0)))
                            .style(scheme.on_primary.into())
                            .shape(Shape::Ellipse)
                            .child(|| {});
                    }
                });
            surface()
                .modifier(Modifier::new().size(Dp(8.0), Dp(8.0)))
                .style(scheme.primary.into())
                .shape(Shape::Ellipse)
                .child(|| {});
            for index in 0..count {
                clock_value(state, focus, index, count);
            }
        });
}

#[tessera]
fn clock_value(
    state: State<TimePickerState>,
    focus: State<ClockFocus>,
    index: usize,
    count: usize,
) {
    let snapshot = state.with(TimePickerState::snapshot);
    let theme = use_context::<MaterialTheme>().expect("theme").get();
    let number = clock_value_number(snapshot.selection, index);
    let selected = index == selected_clock_slot(&snapshot);
    let (requester, anchor) = focus.with(|focus| (focus.requesters[index], focus.anchor));
    let field = if snapshot.selection == TimeField::Hour {
        "Hour"
    } else {
        "Minute"
    };
    let total = if snapshot.selection == TimeField::Minute {
        60
    } else if snapshot.is_24_hour {
        24
    } else {
        12
    };
    surface()
        .modifier(
            Modifier::new()
                .size(Dp(48.0), Dp(48.0))
                .semantics(SemanticsArgs {
                    selected: Some(selected),
                    ..Default::default()
                })
                .on_focus_event(move |state: tessera_ui::FocusState| {
                    if state.is_focused() && focus.with(|focus| focus.anchor != index) {
                        focus.with_mut(|focus| focus.anchor = index);
                    }
                })
                .push_keyboard_input(ClockNavigation {
                    focus,
                    index,
                    count,
                }),
        )
        .style(Color::TRANSPARENT.into())
        .shape(Shape::Ellipse)
        .show_ripple(false)
        .focus_requester(requester)
        .focus_properties(FocusProperties::new().skip_traversal(index != anchor))
        .accessibility_label(format!("{field} {number} of {total}"))
        .on_click(move || select_clock_slot(state, index))
        .content_alignment(Alignment::Center)
        .child(move || {
            text()
                .content(if snapshot.selection == TimeField::Minute {
                    format_two_digit(number)
                } else {
                    number.to_string()
                })
                .style(theme.typography.body_large)
                .color(if selected {
                    theme.color_scheme.on_primary
                } else {
                    theme.color_scheme.on_surface
                });
        });
}

/// # time_picker_dialog
///
/// Render a time picker dialog body with optional action buttons.
///
/// ## Usage
///
/// Use inside `dialog_provider` when presenting a modal time selection flow.
///
/// ## Parameters
///
/// - `state` — state handle used by the embedded time picker.
/// - `title` — optional override for the dialog title.
/// - `confirm_button` — optional confirm button content.
/// - `dismiss_button` — optional dismiss button content.
/// - `show_mode_toggle` — whether the display mode toggle is shown.
/// - `picker_modifier` — modifier chain applied to the embedded picker.
/// - `picker_initial_hour` — initial picker hour.
/// - `picker_initial_minute` — initial picker minute.
/// - `picker_is_24_hour` — whether the picker uses 24-hour mode.
/// - `picker_display_mode` — initial picker display mode.
/// - `picker_hour_step` — hour step size for the picker.
/// - `picker_minute_step` — minute step size for the picker.
/// - `picker_layout` — preferred dial arrangement; defaults to vertical.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::remember;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_components::time_picker::{TimePickerLayout, TimePickerState, time_picker_dialog};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let state = remember(TimePickerState::default);
/// time_picker_dialog()
///     .state(state)
///     .picker_layout(TimePickerLayout::Horizontal);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn time_picker_dialog(
    state: Option<State<TimePickerState>>,
    #[prop(into)] title: Option<String>,
    confirm_button: Option<RenderSlot>,
    dismiss_button: Option<RenderSlot>,
    show_mode_toggle: Option<bool>,
    picker_modifier: Option<Modifier>,
    picker_initial_hour: Option<u8>,
    picker_initial_minute: Option<u8>,
    picker_is_24_hour: Option<bool>,
    picker_display_mode: Option<TimePickerDisplayMode>,
    picker_hour_step: Option<u8>,
    picker_minute_step: Option<u8>,
    picker_layout: Option<TimePickerLayout>,
) {
    let show_mode_toggle = show_mode_toggle.unwrap_or(true);
    let picker_modifier = picker_modifier.unwrap_or_default();
    let picker_initial_hour = picker_initial_hour.unwrap_or(0);
    let picker_initial_minute = picker_initial_minute.unwrap_or(0);
    let picker_is_24_hour = picker_is_24_hour.unwrap_or(false);
    let picker_display_mode = picker_display_mode.unwrap_or_default();
    let picker_hour_step = picker_hour_step.unwrap_or(1);
    let picker_minute_step = picker_minute_step.unwrap_or(1);
    let state = if let Some(state) = state {
        state
    } else {
        remember(|| {
            TimePickerState::new(
                picker_initial_hour,
                picker_initial_minute,
                picker_is_24_hour,
                picker_display_mode,
            )
        })
    };
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let modifier = Modifier::new();
    #[cfg(test)]
    let modifier = modifier.push_semantics(TimePickerTag("time-picker-dialog"));
    layout()
        .modifier(modifier)
        .policy(TimeDialogMeasure {
            arrangement: picker_layout.unwrap_or_default(),
            input: state.with(|state| state.display_mode() == TimePickerDisplayMode::Input),
        })
        .child(move || {
            text()
                .content(title.as_deref().unwrap_or("Select time"))
                .style(theme.typography.label_medium)
                .color(theme.color_scheme.on_surface_variant);
            time_picker()
                .modifier(picker_modifier.clone())
                .state(state)
                .hour_step(picker_hour_step)
                .minute_step(picker_minute_step)
                .layout(picker_layout.unwrap_or_default());
            row()
                .modifier(Modifier::new().fill_max_width().height(Dp(40.0)))
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    if show_mode_toggle {
                        time_display_mode_toggle(state);
                    }
                    spacer().modifier(Modifier::new().weight(1.0));
                    provide_context(
                        || ContentColor {
                            current: theme.color_scheme.primary,
                        },
                        || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    if let Some(dismiss) = dismiss_button {
                                        dismiss.render();
                                    }
                                    if dismiss_button.is_some() && confirm_button.is_some() {
                                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    }
                                    if let Some(confirm) = confirm_button {
                                        confirm.render();
                                    }
                                });
                        },
                    );
                });
        });
}

#[derive(Clone, PartialEq)]
struct TimeDialogMeasure {
    arrangement: TimePickerLayout,
    input: bool,
}

impl tessera_ui::LayoutModifierNode for TimeDialogMeasure {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let children = scope.children();
        let gap = Dp(
            if !self.input
                && self.arrangement == TimePickerLayout::Horizontal
                && parent
                    .width
                    .max
                    .is_none_or(|width| width >= Dp(524.0).to_px())
            {
                0.0
            } else {
                20.0
            },
        )
        .to_px();
        let title = children[0].probe(
            scope,
            &Constraint::new(parent.width.without_min(), AxisConstraint::NONE),
        )?;
        let mut body = children[1].measure(
            scope,
            &Constraint::new(
                parent.width.without_min(),
                (parent.height - title.height - gap - Dp(64.0).to_px()).without_min(),
            ),
        )?;
        let resolved_gap = Dp(
            if !self.input
                && self.arrangement == TimePickerLayout::Horizontal
                && body.width >= Dp(524.0).to_px()
            {
                0.0
            } else {
                20.0
            },
        )
        .to_px();
        if resolved_gap != gap {
            body = children[1].measure(
                scope,
                &Constraint::new(
                    parent.width.without_min(),
                    (parent.height - title.height - resolved_gap - Dp(64.0).to_px()).without_min(),
                ),
            )?;
        }
        let title = children[0].measure(
            scope,
            &Constraint::new(AxisConstraint::exact(body.width), AxisConstraint::NONE),
        )?;
        let actions = children[2].measure(
            scope,
            &Constraint::new(
                AxisConstraint::exact(body.width),
                AxisConstraint::exact(Dp(40.0).to_px()),
            ),
        )?;
        let body_y = title.height + resolved_gap;
        let actions_y = body_y + body.height + Dp(24.0).to_px();
        let mut result = LayoutResult::new(ComputedData {
            width: parent.width.clamp(body.width),
            height: parent.height.clamp(actions_y + actions.height),
        });
        result.place_child(children[0], PxPosition::ZERO);
        result.place_child(children[1], PxPosition::new(Px::ZERO, body_y));
        result.place_child(children[2], PxPosition::new(Px::ZERO, actions_y));
        Ok(result)
    }
}

#[tessera]
fn time_display_mode_toggle(state: State<TimePickerState>) {
    let input = state.with(|s| s.display_mode == TimePickerDisplayMode::Input);
    icon_button()
        .standard()
        .icon(if input {
            filled::SCHEDULE_SVG
        } else {
            filled::KEYBOARD_SVG
        })
        .accessibility_label(if input {
            "Switch to clock"
        } else {
            "Switch to time input"
        })
        .on_click(move || {
            state.with_mut(TimePickerState::toggle_display_mode);
            FocusManager::current().defer_keyboard_until_layout();
        });
}

fn format_two_digit(value: u8) -> String {
    format!("{value:02}")
}

fn hour_for_display(hour: u8, is_24_hour: bool) -> u8 {
    if is_24_hour {
        hour
    } else {
        let hour = hour % 12;
        if hour == 0 { 12 } else { hour }
    }
}

fn normalize_step(step: u8, max: u8) -> u8 {
    if step == 0 { 1 } else { step.min(max) }
}

fn clamp_hour(hour: u8) -> u8 {
    hour.min(23)
}

fn clamp_minute(minute: u8) -> u8 {
    minute.min(59)
}

fn current_time_utc() -> (u8, u8) {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hour = ((secs / 3_600) % 24) as u8;
    let minute = ((secs / 60) % 60) as u8;
    (hour, minute)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        CursorEventContent, Dp, GestureState, Modifier, PointerChange, PressKeyEventType,
        PxPosition, State, remember, tessera,
        testing::{
            layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
            start_layout_test_session,
        },
    };

    use crate::{modifier::ModifierExt as _, theme::material_theme};

    use super::{TimeField, TimePickerLayout, TimePickerState, time_picker, time_picker_dialog};

    thread_local! {
        static STATE: Cell<Option<State<TimePickerState>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn clock_fixture(is_24_hour: bool) {
        let state = remember(|| TimePickerState::new(10, 8, is_24_hour, Default::default()));
        STATE.set(Some(state));
        material_theme().child(move || {
            time_picker().state(state);
        });
    }

    #[tessera]
    fn arrangement_fixture(arrangement: TimePickerLayout, is_24_hour: bool) {
        material_theme().child(move || {
            crate::boxed::boxed().children(move || {
                time_picker().layout(arrangement).is_24_hour(is_24_hour);
            });
        });
    }

    #[test]
    fn dial_arrangement_uses_available_space_and_horizontal_period_geometry() {
        for (arrangement, is_24_hour, viewport, size, clock_x, clock_y) in [
            (
                TimePickerLayout::Vertical,
                false,
                (800, 600),
                (280, 372),
                12,
                116,
            ),
            (
                TimePickerLayout::Horizontal,
                false,
                (800, 600),
                (524, 256),
                268,
                0,
            ),
            (
                TimePickerLayout::Horizontal,
                true,
                (800, 600),
                (524, 256),
                268,
                0,
            ),
            (
                TimePickerLayout::Horizontal,
                false,
                (400, 600),
                (280, 372),
                12,
                116,
            ),
            (
                TimePickerLayout::Horizontal,
                false,
                (524, 300),
                (524, 256),
                268,
                0,
            ),
            (
                TimePickerLayout::Horizontal,
                false,
                (523, 600),
                (280, 372),
                12,
                116,
            ),
        ] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    arrangement_fixture(arrangement, is_24_hour);
                })
                .viewport_px(viewport.0, viewport.1),
            );
            let snapshot = pump_layout_test_session(&mut session, 0);
            snapshot.node("time-picker").size(size.0, size.1);
            let clock = snapshot.node("time-clock").bounds();
            assert_eq!((clock.x.raw(), clock.y.raw()), (clock_x, clock_y));
            if !is_24_hour {
                let period = snapshot.node("time-period").bounds();
                if size.0 == 524 {
                    assert_eq!(
                        (
                            period.x.raw(),
                            period.y.raw(),
                            period.width.raw(),
                            period.height.raw()
                        ),
                        (0, 168, 216, 38)
                    );
                } else {
                    assert_eq!(
                        (
                            period.x.raw(),
                            period.y.raw(),
                            period.width.raw(),
                            period.height.raw()
                        ),
                        (228, 0, 52, 80)
                    );
                }
            }
        }
    }

    #[tessera]
    fn constrained_dialog_fixture() {
        material_theme().child(|| {
            crate::boxed::boxed().children(|| {
                time_picker_dialog()
                    .picker_layout(TimePickerLayout::Horizontal)
                    .picker_modifier(Modifier::new().width(Dp(400.0)));
            });
        });
    }

    #[test]
    fn constrained_horizontal_dialog_restores_vertical_header_spacing() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                constrained_dialog_fixture();
            })
            .viewport_px(800, 600),
        );
        let snapshot = pump_layout_test_session(&mut session, 0);
        snapshot.node("time-picker-dialog").size(400, 472);
        let clock = snapshot.node("time-clock").bounds();
        assert_eq!((clock.x.raw(), clock.y.raw()), (72, 152));
    }

    #[test]
    fn dial_pointer_selection_distinguishes_inner_hours_and_advances_to_minutes() {
        for (is_24_hour, radius, expected_hour) in [(false, 101.0, 3), (true, 69.0, 15)] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    clock_fixture(is_24_hour);
                })
                .viewport_px(400, 500),
            );
            let snapshot = pump_layout_test_session(&mut session, 0);
            snapshot.node("time-clock").size(256, 256);
            let bounds = snapshot.node("time-clock").bounds();
            let cursor = Some(PxPosition::from_f32_xy(
                bounds.x.to_f32() + bounds.width.to_f32() / 2.0 + radius,
                bounds.y.to_f32() + bounds.height.to_f32() / 2.0,
            ));
            for (index, content) in [
                CursorEventContent::Pressed(PressKeyEventType::Left),
                CursorEventContent::Released(PressKeyEventType::Left),
            ]
            .into_iter()
            .enumerate()
            {
                pump_layout_test_session_with_input(
                    &mut session,
                    (index as u64 + 1) * 16_666_667,
                    vec![PointerChange::new(
                        tessera_ui::time::Instant::now(),
                        1,
                        content,
                        GestureState::TapCandidate,
                    )],
                    cursor,
                );
            }
            let state = STATE.get().unwrap();
            assert_eq!(state.with(TimePickerState::hour), expected_hour);
            assert_eq!(state.with(|s| s.selection), TimeField::Minute);
            for (index, content) in [
                CursorEventContent::Pressed(PressKeyEventType::Left),
                CursorEventContent::Released(PressKeyEventType::Left),
            ]
            .into_iter()
            .enumerate()
            {
                pump_layout_test_session_with_input(
                    &mut session,
                    (index as u64 + 3) * 16_666_667,
                    vec![PointerChange::new(
                        tessera_ui::time::Instant::now(),
                        1,
                        content,
                        GestureState::TapCandidate,
                    )],
                    cursor,
                );
            }
            assert_eq!(state.with(TimePickerState::minute), 15);
        }
    }
    #[tessera]
    fn time_input_fixture() {
        let state = remember(|| {
            let mut state = TimePickerState::new(10, 8, false, super::TimePickerDisplayMode::Input);
            state.selection = TimeField::Minute;
            state
        });
        STATE.set(Some(state));
        material_theme().child(move || {
            crate::column::column().children(move || {
                time_picker().state(state);
            });
        });
    }

    #[test]
    fn focusing_an_input_updates_the_active_clock_field() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                time_input_fixture();
            })
            .viewport_px(400, 500),
        );
        pump_layout_test_session(&mut session, 0);
        for (index, content) in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Released(PressKeyEventType::Left),
        ]
        .into_iter()
        .enumerate()
        {
            pump_layout_test_session_with_input(
                &mut session,
                (index as u64 + 1) * 16_666_667,
                vec![PointerChange::new(
                    tessera_ui::time::Instant::now(),
                    1,
                    content,
                    GestureState::TapCandidate,
                )],
                Some(PxPosition::from_f32_xy(48.0, 36.0)),
            );
        }
        for frame in 3..6 {
            pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(STATE.get().unwrap().with(|s| s.selection), TimeField::Hour);
    }
}
