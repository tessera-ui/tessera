//! Time picker components for selecting a clock time.
//!
//! ## Usage
//!
//! Use to let users choose a time for alarms, reminders, or schedules.
use std::time::{SystemTime, UNIX_EPOCH};

use tessera_ui::{
    AxisConstraint, Callback, Dp, Modifier, RenderSlot, State, provide_context, remember, tessera,
    use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    column::column,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialTheme, TextStyle},
};

const TIME_CELL_WIDTH: Dp = Dp(72.0);
const TIME_CELL_HEIGHT: Dp = Dp(56.0);
const TIME_CELL_RADIUS: Dp = Dp(12.0);
const TIME_STEP_BUTTON_SIZE: Dp = Dp(28.0);
const PERIOD_BUTTON_WIDTH: Dp = Dp(56.0);
const PERIOD_BUTTON_HEIGHT: Dp = Dp(32.0);
const TIME_ROW_GAP: Dp = Dp(12.0);

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
}

#[derive(Clone)]
struct TimePickerConfig {
    modifier: Modifier,
    hour_step: u8,
    minute_step: u8,
    state: Option<State<TimePickerState>>,
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
/// - `hour_step` — step size for hour changes.
/// - `minute_step` — step size for minute changes.
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
    modifier: Modifier,
    initial_hour: u8,
    initial_minute: u8,
    is_24_hour: bool,
    display_mode: TimePickerDisplayMode,
    hour_step: u8,
    minute_step: u8,
    state: Option<State<TimePickerState>>,
) {
    let state = state.unwrap_or_else(|| {
        remember(|| TimePickerState::new(initial_hour, initial_minute, is_24_hour, display_mode))
    });
    time_picker_inner(TimePickerConfig {
        modifier,
        hour_step,
        minute_step,
        state: Some(state),
    });
}

fn time_picker_inner(args: TimePickerConfig) {
    let state = args
        .state
        .expect("time_picker_inner requires state to be set");
    let snapshot = state.with(|s| s.snapshot());
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let modifier = args.modifier;
    let hour_step = normalize_step(args.hour_step, 23);
    let minute_step = normalize_step(args.minute_step, 59);

    let hour_display = format_two_digit(hour_for_display(snapshot.hour, snapshot.is_24_hour));
    let minute_display = format_two_digit(snapshot.minute);
    let show_labels = snapshot.display_mode == TimePickerDisplayMode::Input;

    column().modifier(modifier).children(move || {
        {
            let hour_display = hour_display.clone();
            let minute_display = minute_display.clone();
            row()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    let hour_display = hour_display.clone();
                    {
                        time_stepper_column(
                            "Hour",
                            hour_display.clone(),
                            show_labels,
                            Callback::new(move || {
                                state.with_mut(|s| s.increment_hour(hour_step));
                            }),
                            Callback::new(move || {
                                state.with_mut(|s| s.decrement_hour(hour_step));
                            }),
                        );
                    };

                    {
                        spacer().modifier(Modifier::new().width(Dp(6.0)));
                    };
                    {
                        text()
                            .content(":")
                            .style(TextStyle {
                                font_size: typography.headline_small.font_size,
                                line_height: typography.headline_small.line_height,
                            })
                            .color(scheme.on_surface_variant);
                    };
                    {
                        spacer().modifier(Modifier::new().width(Dp(6.0)));
                    };

                    {
                        let minute_display = minute_display.clone();
                        time_stepper_column(
                            "Minute",
                            minute_display,
                            show_labels,
                            Callback::new(move || {
                                state.with_mut(|s| s.increment_minute(minute_step));
                            }),
                            Callback::new(move || {
                                state.with_mut(|s| s.decrement_minute(minute_step));
                            }),
                        );
                    };
                });
        };

        if !snapshot.is_24_hour {
            {
                spacer().modifier(Modifier::new().height(TIME_ROW_GAP));
            };
            let is_pm = snapshot.hour >= 12;
            {
                period_toggle(is_pm, state);
            };
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
    show_mode_toggle: bool,
    picker_modifier: Modifier,
    picker_initial_hour: u8,
    picker_initial_minute: u8,
    picker_is_24_hour: bool,
    picker_display_mode: TimePickerDisplayMode,
    picker_hour_step: u8,
    picker_minute_step: u8,
) {
    let state = state.expect("time_picker_dialog requires state to be set");
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let has_confirm = confirm_button.is_some();
    let has_dismiss = dismiss_button.is_some();

    column()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::new(
                Dp(280.0).into(),
                Some(Dp(520.0).into()),
            )),
            Some(AxisConstraint::NONE),
        ))
        .children(move || {
            {
                let title = title.clone();
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        {
                            let title_text = title.as_deref().unwrap_or("Select time");
                            text()
                                .content(title_text)
                                .size(
                                    use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .typography
                                        .title_medium
                                        .font_size,
                                )
                                .color(scheme.on_surface);
                        };
                        if show_mode_toggle {
                            {
                                time_display_mode_toggle(state);
                            };
                        }
                    });
            };

            {
                spacer().modifier(Modifier::new().height(Dp(12.0)));
            };

            {
                time_picker()
                    .modifier(picker_modifier.clone())
                    .initial_hour(picker_initial_hour)
                    .initial_minute(picker_initial_minute)
                    .is_24_hour(picker_is_24_hour)
                    .display_mode(picker_display_mode)
                    .hour_step(picker_hour_step)
                    .minute_step(picker_minute_step)
                    .state(state);
            };

            if has_confirm || has_dismiss {
                {
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                };
                let action_color = scheme.primary;
                {
                    let dismiss_button = dismiss_button;
                    let confirm_button = confirm_button;
                    provide_context(
                        || ContentColor {
                            current: action_color,
                        },
                        || {
                            row()
                                .modifier(Modifier::new().fill_max_width())
                                .main_axis_alignment(MainAxisAlignment::End)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    if let Some(dismiss) = dismiss_button {
                                        dismiss.render();
                                    }
                                    if has_confirm && has_dismiss {
                                        {
                                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                                        };
                                    }
                                    if let Some(confirm) = confirm_button {
                                        confirm.render();
                                    }
                                });
                        },
                    );
                };
            }
        });
}

fn time_stepper_column(
    label: &'static str,
    value: String,
    show_label: bool,
    on_increment: Callback,
    on_decrement: Callback,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            {
                step_button("+", on_increment);
            };
            {
                spacer().modifier(Modifier::new().height(Dp(6.0)));
            };
            let value_text = value.clone();
            {
                time_value_cell(value_text.clone());
            };
            {
                spacer().modifier(Modifier::new().height(Dp(6.0)));
            };
            {
                step_button("-", on_decrement);
            };
            if show_label {
                {
                    spacer().modifier(Modifier::new().height(Dp(6.0)));
                };
                {
                    text()
                        .content(label)
                        .style(TextStyle {
                            font_size: typography.label_small.font_size,
                            line_height: typography.label_small.line_height,
                        })
                        .color(scheme.on_surface_variant);
                };
            }
        });
}

fn time_value_cell(value: String) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface()
        .modifier(
            Modifier::new()
                .width(TIME_CELL_WIDTH)
                .height(TIME_CELL_HEIGHT),
        )
        .style(SurfaceStyle::Filled {
            color: scheme.surface_container_high,
        })
        .shape(Shape::rounded_rectangle(TIME_CELL_RADIUS))
        .content_alignment(Alignment::Center)
        .child(move || {
            text()
                .content(value.clone())
                .size(
                    use_context::<MaterialTheme>()
                        .expect("MaterialTheme must be provided")
                        .get()
                        .typography
                        .headline_small
                        .font_size,
                )
                .color(scheme.on_surface);
        });
}

fn step_button(label: &'static str, on_click: Callback) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface()
        .modifier(Modifier::new().size(TIME_STEP_BUTTON_SIZE, TIME_STEP_BUTTON_SIZE))
        .style(SurfaceStyle::Filled {
            color: scheme.surface_container_low,
        })
        .shape(Shape::CAPSULE)
        .content_alignment(Alignment::Center)
        .on_click_shared(on_click)
        .child(move || {
            text()
                .content(label)
                .size(
                    use_context::<MaterialTheme>()
                        .expect("MaterialTheme must be provided")
                        .get()
                        .typography
                        .body_medium
                        .font_size,
                )
                .color(scheme.on_surface);
        });
}

fn period_toggle(is_pm: bool, state: State<TimePickerState>) {
    row()
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            {
                period_button("AM", !is_pm, DayPeriod::Am, state);
            };
            {
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            };
            {
                period_button("PM", is_pm, DayPeriod::Pm, state);
            };
        });
}

fn period_button(
    label: &'static str,
    selected: bool,
    period: DayPeriod,
    state: State<TimePickerState>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let text_color = if selected {
        scheme.on_primary
    } else {
        scheme.on_surface
    };
    let style = if selected {
        SurfaceStyle::Filled {
            color: scheme.primary,
        }
    } else {
        SurfaceStyle::Filled {
            color: scheme.surface_container_low,
        }
    };
    surface()
        .modifier(
            Modifier::new()
                .width(PERIOD_BUTTON_WIDTH)
                .height(PERIOD_BUTTON_HEIGHT),
        )
        .style(style)
        .shape(Shape::CAPSULE)
        .content_alignment(Alignment::Center)
        .on_click(move || {
            state.with_mut(|s| s.set_period(period));
        })
        .child(move || {
            text()
                .content(label)
                .size(
                    use_context::<MaterialTheme>()
                        .expect("MaterialTheme must be provided")
                        .get()
                        .typography
                        .label_medium
                        .font_size,
                )
                .color(text_color);
        });
}

fn time_display_mode_toggle(state: State<TimePickerState>) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let label = state.with(|s| match s.display_mode() {
        TimePickerDisplayMode::Picker => "Input",
        TimePickerDisplayMode::Input => "Picker",
    });
    surface()
        .modifier(Modifier::new().padding_all(Dp(4.0)))
        .style(SurfaceStyle::Filled {
            color: scheme.surface_container_high,
        })
        .shape(Shape::CAPSULE)
        .content_alignment(Alignment::Center)
        .on_click(move || {
            state.with_mut(|s| s.toggle_display_mode());
        })
        .child(move || {
            text()
                .content(label)
                .size(
                    use_context::<MaterialTheme>()
                        .expect("MaterialTheme must be provided")
                        .get()
                        .typography
                        .label_small
                        .font_size,
                )
                .color(scheme.primary);
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
