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
    Color, ComputedData, Constraint, CursorEventContent, Dp, KeyboardInput,
    KeyboardInputModifierNode, LayoutResult, MeasurementError, Modifier, PointerInput,
    PointerInputModifierNode, PressKeyEventType, PxPosition, RenderSlot, State,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera,
    time::{SystemTime, UNIX_EPOCH},
    use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    divider::horizontal_divider,
    icon::icon,
    icon_button::icon_button,
    image_vector::{ImageVectorData, TintMode},
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    pipelines::image_vector::command::ImageVectorVertex,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    text_input::{TextInputController, text_input},
    theme::{ContentColor, MaterialTheme},
};

const CLOCK_SIZE: Dp = Dp(256.0);
const CLOCK_OUTER_RADIUS: f32 = 101.0;
const CLOCK_INNER_RADIUS: f32 = 69.0;

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
/// - `hour_step` — step size for dial and keyboard hour changes.
/// - `minute_step` — step size for dial and keyboard minute changes.
/// - `state` — optional external state for selected time and display mode.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::time_picker::{TimePickerState, time_picker};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// time_picker();
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
) {
    let modifier = modifier.unwrap_or_default();
    let initial_hour = initial_hour.unwrap_or(0);
    let initial_minute = initial_minute.unwrap_or(0);
    let is_24_hour = is_24_hour.unwrap_or(false);
    let display_mode = display_mode.unwrap_or_default();
    let hour_step = hour_step.unwrap_or(1);
    let minute_step = minute_step.unwrap_or(1);
    let state = state.unwrap_or_else(|| {
        remember(|| TimePickerState::new(initial_hour, initial_minute, is_24_hour, display_mode))
    });
    time_picker_content(state, modifier, hour_step, minute_step);
}

#[tessera]
fn time_picker_content(
    state: State<TimePickerState>,
    modifier: Modifier,
    hour_step: u8,
    minute_step: u8,
) {
    let snapshot = state.with(TimePickerState::snapshot);
    let input = snapshot.display_mode == TimePickerDisplayMode::Input;
    let width = if snapshot.is_24_hour { 256.0 } else { 280.0 };
    column()
        .modifier(modifier.width(Dp(width)))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(move || {
                    time_number(state, TimeField::Hour, input);
                    let theme = use_context::<MaterialTheme>().unwrap().get();
                    surface()
                        .modifier(
                            Modifier::new().size(Dp(24.0), Dp(if input { 72.0 } else { 80.0 })),
                        )
                        .style(Color::TRANSPARENT.into())
                        .content_alignment(Alignment::Center)
                        .child(move || {
                            text().content(":").style(theme.typography.display_large);
                        });
                    time_number(state, TimeField::Minute, input);
                    if !snapshot.is_24_hour {
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        time_period_selector(state, input);
                    }
                });
            if !input {
                spacer().modifier(Modifier::new().height(Dp(36.0)));
                time_clock(state, hour_step, minute_step);
            }
        });
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
                .modifier(Modifier::new().size(Dp(width), Dp(height)))
                .style(background.into())
                .shape(theme.shapes.small)
                .content_alignment(Alignment::Center)
                .accessibility_label(label)
                .on_click(move || state.with_mut(|s| s.selection = field))
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
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let height = if input { 72.0 } else { 80.0 };
    let period = state.with(TimePickerState::period);
    boxed().children(move || {
        column().children(move || {
            for (label, value) in [("AM", DayPeriod::Am), ("PM", DayPeriod::Pm)] {
                let selected = value == period;
                let corner = RoundedCorner::manual(Dp(8.0), 2.0);
                let shape = if value == DayPeriod::Am {
                    Shape::RoundedRectangle {
                        top_left: corner,
                        top_right: corner,
                        bottom_left: RoundedCorner::ZERO,
                        bottom_right: RoundedCorner::ZERO,
                    }
                } else {
                    Shape::RoundedRectangle {
                        top_left: RoundedCorner::ZERO,
                        top_right: RoundedCorner::ZERO,
                        bottom_left: corner,
                        bottom_right: corner,
                    }
                };
                surface()
                    .modifier(Modifier::new().size(Dp(52.0), Dp(height / 2.0)))
                    .style(
                        if selected {
                            scheme.tertiary_container
                        } else {
                            Color::TRANSPARENT
                        }
                        .into(),
                    )
                    .shape(shape)
                    .content_alignment(Alignment::Center)
                    .accessibility_label(label)
                    .on_click(move || state.with_mut(|s| s.set_period(value)))
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
        });
        surface()
            .modifier(Modifier::new().size(Dp(52.0), Dp(height)))
            .shape(theme.shapes.small)
            .block_input(false)
            .style(SurfaceStyle::Outlined {
                color: scheme.outline,
                width: Dp(1.0),
            });
        column()
            .modifier(Modifier::new().size(Dp(52.0), Dp(height)))
            .main_axis_alignment(MainAxisAlignment::Center)
            .children(move || {
                horizontal_divider().color(scheme.outline);
            });
    });
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
}

impl PointerInputModifierNode for ClockInput {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
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
        if released {
            self.dragging.set(false);
        }
        input.consume_pointer_changes();
    }
}

impl KeyboardInputModifierNode for ClockInput {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        let mut handled = false;
        for event in input.keyboard_events.iter() {
            if event.state != winit::event::ElementState::Pressed {
                continue;
            }
            let delta = match event.logical_key {
                winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::ArrowRight | winit::keyboard::NamedKey::ArrowUp,
                ) => 1,
                winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::ArrowLeft | winit::keyboard::NamedKey::ArrowDown,
                ) => -1,
                _ => 0,
            };
            if delta != 0 {
                self.state.with_mut(|s| match (s.selection, delta > 0) {
                    (TimeField::Hour, true) => s.increment_hour(self.hour_step),
                    (TimeField::Hour, false) => s.decrement_hour(self.hour_step),
                    (TimeField::Minute, true) => s.increment_minute(self.minute_step),
                    (TimeField::Minute, false) => s.decrement_minute(self.minute_step),
                });
                handled = true;
            }
        }
        if handled {
            input.block_keyboard();
        }
    }
}

#[tessera]
fn time_clock(state: State<TimePickerState>, hour_step: u8, minute_step: u8) {
    let snapshot = state.with(TimePickerState::snapshot);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let dragging = remember(|| false);
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
    };
    let count = if snapshot.is_24_hour && snapshot.selection == TimeField::Hour {
        24
    } else {
        12
    };
    layout()
        .modifier(
            Modifier::new()
                .size(CLOCK_SIZE, CLOCK_SIZE)
                .semantics(SemanticsArgs {
                    test_tag: cfg!(test).then(|| "time-clock".to_owned()),
                    label: Some("Time dial".to_owned()),
                    ..Default::default()
                })
                .focusable()
                .push_pointer_input(interaction)
                .push_keyboard_input(interaction),
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
                let number = if snapshot.selection == TimeField::Minute {
                    index * 5
                } else if index == 0 {
                    12
                } else if index == 12 {
                    0
                } else {
                    index
                };
                let selected = if snapshot.selection == TimeField::Minute {
                    number == ((snapshot.minute as f32 / 5.0).round() as u8 * 5) % 60
                } else if snapshot.is_24_hour {
                    number == snapshot.hour
                } else {
                    number == hour_for_display(snapshot.hour, false)
                };
                surface()
                    .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                    .style(Color::TRANSPARENT.into())
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
                                scheme.on_primary
                            } else {
                                scheme.on_surface
                            });
                    });
            }
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
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::remember;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_components::time_picker::{TimePickerState, time_picker_dialog};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let state = remember(TimePickerState::default);
/// time_picker_dialog().state(state);
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
) {
    let show_mode_toggle = show_mode_toggle.unwrap_or(true);
    let picker_modifier = picker_modifier.unwrap_or_default();
    let picker_initial_hour = picker_initial_hour.unwrap_or(0);
    let picker_initial_minute = picker_initial_minute.unwrap_or(0);
    let picker_is_24_hour = picker_is_24_hour.unwrap_or(false);
    let picker_display_mode = picker_display_mode.unwrap_or_default();
    let picker_hour_step = picker_hour_step.unwrap_or(1);
    let picker_minute_step = picker_minute_step.unwrap_or(1);
    let state = state.unwrap_or_else(|| {
        remember(|| {
            TimePickerState::new(
                picker_initial_hour,
                picker_initial_minute,
                picker_is_24_hour,
                picker_display_mode,
            )
        })
    });
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let width = if state.with(|s| s.is_24_hour) {
        256.0
    } else {
        280.0
    };
    column()
        .modifier(Modifier::new().width(Dp(width)))
        .children(move || {
            text()
                .content(title.as_deref().unwrap_or("Select time"))
                .style(theme.typography.label_medium)
                .color(theme.color_scheme.on_surface_variant);
            spacer().modifier(Modifier::new().height(Dp(20.0)));
            time_picker()
                .modifier(picker_modifier.clone())
                .state(state)
                .hour_step(picker_hour_step)
                .minute_step(picker_minute_step);
            spacer().modifier(Modifier::new().height(Dp(24.0)));
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
        .on_click(move || state.with_mut(TimePickerState::toggle_display_mode));
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
        CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition, State,
        remember, tessera,
        testing::{
            layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
            start_layout_test_session,
        },
    };

    use crate::theme::material_theme;

    use super::{TimeField, TimePickerState, time_picker};

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
