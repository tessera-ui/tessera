//! Time picker components for selecting a clock time.
//!
//! ## Usage
//!
//! Use to let users choose a time for alarms, reminders, or schedules.
use std::time::{SystemTime, UNIX_EPOCH};

use derive_setters::Setters;
use tessera_ui::{
    Callback, DimensionValue, Dp, Modifier, RenderSlot, State, provide_context, remember, tessera,
    use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{ContentColor, MaterialTheme},
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

/// Configuration options for [`time_picker`].
///
/// Initial-state fields are applied only when `time_picker` owns the state.
#[derive(PartialEq, Clone, Setters)]
pub struct TimePickerArgs {
    /// Optional modifier chain applied to the time picker.
    pub modifier: Modifier,
    /// Initial hour for the internal state.
    pub initial_hour: u8,
    /// Initial minute for the internal state.
    pub initial_minute: u8,
    /// Whether the internal state uses 24-hour mode.
    pub is_24_hour: bool,
    /// Initial display mode for the internal state.
    pub display_mode: TimePickerDisplayMode,
    /// Step size for hour changes.
    pub hour_step: u8,
    /// Step size for minute changes.
    pub minute_step: u8,
    /// Optional external state for selected time and display mode.
    ///
    /// When this is `None`, `time_picker` creates and owns an internal state.
    #[setters(skip)]
    pub state: Option<State<TimePickerState>>,
}

impl Default for TimePickerArgs {
    fn default() -> Self {
        let (hour, minute) = current_time_utc();
        Self {
            modifier: Modifier::new()
                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP)),
            initial_hour: hour,
            initial_minute: minute,
            is_24_hour: false,
            display_mode: TimePickerDisplayMode::Picker,
            hour_step: 1,
            minute_step: 1,
            state: None,
        }
    }
}

impl TimePickerArgs {
    /// Sets an external time picker state.
    pub fn state(mut self, state: State<TimePickerState>) -> Self {
        self.state = Some(state);
        self
    }
}

/// Configuration for [`time_picker_dialog`].
#[derive(Clone, PartialEq, Setters)]
pub struct TimePickerDialogArgs {
    /// State handle used by the embedded time picker.
    #[setters(skip)]
    pub state: State<TimePickerState>,
    /// Optional override for the dialog title.
    #[setters(strip_option, into)]
    pub title: Option<String>,
    /// Optional confirm button content.
    #[setters(skip)]
    pub confirm_button: Option<RenderSlot>,
    /// Optional dismiss button content.
    #[setters(skip)]
    pub dismiss_button: Option<RenderSlot>,
    /// Whether the display mode toggle is shown.
    pub show_mode_toggle: bool,
    /// Picker configuration forwarded to [`time_picker`].
    pub picker: TimePickerArgs,
}

impl TimePickerDialogArgs {
    /// Creates dialog args with the required time picker state.
    pub fn new(state: State<TimePickerState>) -> Self {
        Self {
            state,
            title: None,
            confirm_button: None,
            dismiss_button: None,
            show_mode_toggle: false,
            picker: TimePickerArgs::default(),
        }
    }

    /// Sets the confirm button content.
    pub fn confirm_button<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.confirm_button = Some(RenderSlot::new(f));
        self
    }

    /// Sets the confirm button content using a shared callback.
    pub fn confirm_button_shared(mut self, f: impl Into<RenderSlot>) -> Self {
        self.confirm_button = Some(f.into());
        self
    }

    /// Sets the dismiss button content.
    pub fn dismiss_button<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.dismiss_button = Some(RenderSlot::new(f));
        self
    }

    /// Sets the dismiss button content using a shared callback.
    pub fn dismiss_button_shared(mut self, f: impl Into<RenderSlot>) -> Self {
        self.dismiss_button = Some(f.into());
        self
    }
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
/// - `args` — configuration for the picker layout and internal state defaults;
///   see [`TimePickerArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::time_picker::{TimePickerArgs, TimePickerState, time_picker};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     || {
/// time_picker(&TimePickerArgs::default());
///
/// let state = TimePickerState::default();
/// assert!(state.hour() <= 23);
/// #     },
/// # );
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn time_picker(args: &TimePickerArgs) {
    let mut args: TimePickerArgs = args.clone();
    let initial_hour = args.initial_hour;
    let initial_minute = args.initial_minute;
    let is_24_hour = args.is_24_hour;
    let display_mode = args.display_mode;

    let state = args.state.unwrap_or_else(|| {
        remember(|| TimePickerState::new(initial_hour, initial_minute, is_24_hour, display_mode))
    });
    args.state = Some(state);
    time_picker_node(&args);
}

#[tessera]
fn time_picker_node(args: &TimePickerArgs) {
    let state = args
        .state
        .expect("time_picker_node requires state to be set");
    let args = args.clone();
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

    column(ColumnArgs::default().modifier(modifier), move |scope| {
        scope.child(move || {
            let hour_display = hour_display.clone();
            let minute_display = minute_display.clone();
            row(
                RowArgs::default()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::Center),
                move |row_scope| {
                    let hour_display = hour_display.clone();
                    row_scope.child(move || {
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
                    });

                    row_scope.child(|| {
                        spacer(&crate::spacer::SpacerArgs::new(
                            Modifier::new().width(Dp(6.0)),
                        ))
                    });
                    row_scope.child(move || {
                        text(&crate::text::TextArgs::from(
                            &TextArgs::default()
                                .text(":")
                                .size(typography.headline_small.font_size)
                                .color(scheme.on_surface_variant),
                        ));
                    });
                    row_scope.child(|| {
                        spacer(&crate::spacer::SpacerArgs::new(
                            Modifier::new().width(Dp(6.0)),
                        ))
                    });

                    row_scope.child(move || {
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
                    });
                },
            );
        });

        if !snapshot.is_24_hour {
            scope.child(|| {
                spacer(&crate::spacer::SpacerArgs::new(
                    Modifier::new().height(TIME_ROW_GAP),
                ))
            });
            let is_pm = snapshot.hour >= 12;
            scope.child(move || {
                period_toggle(is_pm, state);
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
/// - `args` — dialog layout and action configuration; see
///   [`TimePickerDialogArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::remember;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_components::time_picker::{
///     TimePickerDialogArgs, TimePickerState, time_picker_dialog,
/// };
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     || {
/// let state = remember(TimePickerState::default);
/// time_picker_dialog(&TimePickerDialogArgs::new(state));
/// #     },
/// # );
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn time_picker_dialog(args: &TimePickerDialogArgs) {
    let args: TimePickerDialogArgs = args.clone();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let title = args.title;
    let picker = args.picker;
    let state = args.state;
    let confirm_button = args.confirm_button;
    let dismiss_button = args.dismiss_button;
    let show_mode_toggle = args.show_mode_toggle;
    let has_confirm = confirm_button.is_some();
    let has_dismiss = dismiss_button.is_some();

    column(
        ColumnArgs::default().modifier(Modifier::new().constrain(
            Some(DimensionValue::Wrap {
                min: Some(Dp(280.0).into()),
                max: Some(Dp(520.0).into()),
            }),
            Some(DimensionValue::WRAP),
        )),
        move |scope| {
            scope.child(move || {
                let title = title.clone();
                row(
                    RowArgs::default()
                        .modifier(Modifier::new().fill_max_width())
                        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .cross_axis_alignment(CrossAxisAlignment::Center),
                    move |row_scope| {
                        row_scope.child(move || {
                            let title_text = title.as_deref().unwrap_or("Select time");
                            text(&crate::text::TextArgs::from(
                                &TextArgs::default()
                                    .text(title_text)
                                    .size(
                                        use_context::<MaterialTheme>()
                                            .expect("MaterialTheme must be provided")
                                            .get()
                                            .typography
                                            .title_medium
                                            .font_size,
                                    )
                                    .color(scheme.on_surface),
                            ));
                        });
                        if show_mode_toggle {
                            row_scope.child(move || {
                                time_display_mode_toggle(state);
                            });
                        }
                    },
                );
            });

            scope.child(|| {
                spacer(&crate::spacer::SpacerArgs::new(
                    Modifier::new().height(Dp(12.0)),
                ))
            });

            scope.child(move || {
                time_picker(&picker.clone().state(state));
            });

            if has_confirm || has_dismiss {
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(Dp(16.0)),
                    ))
                });
                let action_color = scheme.primary;
                scope.child(move || {
                    let dismiss_button = dismiss_button.clone();
                    let confirm_button = confirm_button.clone();
                    provide_context(
                        || ContentColor {
                            current: action_color,
                        },
                        || {
                            row(
                                RowArgs::default()
                                    .modifier(Modifier::new().fill_max_width())
                                    .main_axis_alignment(MainAxisAlignment::End)
                                    .cross_axis_alignment(CrossAxisAlignment::Center),
                                move |row_scope| {
                                    if let Some(dismiss) = dismiss_button.clone() {
                                        row_scope.child(move || dismiss.render());
                                    }
                                    if has_confirm && has_dismiss {
                                        row_scope.child(|| {
                                            spacer(&crate::spacer::SpacerArgs::new(
                                                Modifier::new().width(Dp(8.0)),
                                            ))
                                        });
                                    }
                                    if let Some(confirm) = confirm_button.clone() {
                                        row_scope.child(move || confirm.render());
                                    }
                                },
                            );
                        },
                    );
                });
            }
        },
    );
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

    column(
        ColumnArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
        move |scope| {
            let on_increment = on_increment.clone();
            scope.child(move || {
                let on_increment = on_increment.clone();
                step_button("+", move || on_increment.call());
            });
            scope.child(|| {
                spacer(&crate::spacer::SpacerArgs::new(
                    Modifier::new().height(Dp(6.0)),
                ))
            });
            let value_text = value.clone();
            scope.child(move || {
                time_value_cell(value_text.clone());
            });
            scope.child(|| {
                spacer(&crate::spacer::SpacerArgs::new(
                    Modifier::new().height(Dp(6.0)),
                ))
            });
            let on_decrement = on_decrement.clone();
            scope.child(move || {
                let on_decrement = on_decrement.clone();
                step_button("-", move || on_decrement.call());
            });
            if show_label {
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(Dp(6.0)),
                    ))
                });
                scope.child(move || {
                    text(&crate::text::TextArgs::from(
                        &TextArgs::default()
                            .text(label)
                            .size(typography.label_small.font_size)
                            .color(scheme.on_surface_variant),
                    ));
                });
            }
        },
    );
}

fn time_value_cell(value: String) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(
                Modifier::new()
                    .width(TIME_CELL_WIDTH)
                    .height(TIME_CELL_HEIGHT),
            )
            .style(SurfaceStyle::Filled {
                color: scheme.surface_container_high,
            })
            .shape(Shape::rounded_rectangle(TIME_CELL_RADIUS))
            .content_alignment(Alignment::Center),
        move || {
            let value = value.clone();
            text(&crate::text::TextArgs::from(
                &TextArgs::default()
                    .text(value)
                    .size(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .typography
                            .headline_small
                            .font_size,
                    )
                    .color(scheme.on_surface),
            ));
        },
    ));
}

fn step_button(label: &'static str, on_click: impl Fn() + Send + Sync + 'static) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().size(TIME_STEP_BUTTON_SIZE, TIME_STEP_BUTTON_SIZE))
            .style(SurfaceStyle::Filled {
                color: scheme.surface_container_low,
            })
            .shape(Shape::capsule())
            .content_alignment(Alignment::Center)
            .on_click(on_click),
        move || {
            text(&crate::text::TextArgs::from(
                &TextArgs::default()
                    .text(label)
                    .size(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .typography
                            .body_medium
                            .font_size,
                    )
                    .color(scheme.on_surface),
            ));
        },
    ));
}

fn period_toggle(is_pm: bool, state: State<TimePickerState>) {
    row(
        RowArgs::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center),
        move |scope| {
            scope.child(move || {
                period_button("AM", !is_pm, DayPeriod::Am, state);
            });
            scope.child(|| {
                spacer(&crate::spacer::SpacerArgs::new(
                    Modifier::new().width(Dp(8.0)),
                ))
            });
            scope.child(move || {
                period_button("PM", is_pm, DayPeriod::Pm, state);
            });
        },
    );
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
    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(
                Modifier::new()
                    .width(PERIOD_BUTTON_WIDTH)
                    .height(PERIOD_BUTTON_HEIGHT),
            )
            .style(style)
            .shape(Shape::capsule())
            .content_alignment(Alignment::Center)
            .on_click(move || {
                state.with_mut(|s| s.set_period(period));
            }),
        move || {
            text(&crate::text::TextArgs::from(
                &TextArgs::default()
                    .text(label)
                    .size(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .typography
                            .label_medium
                            .font_size,
                    )
                    .color(text_color),
            ));
        },
    ));
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
    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().padding_all(Dp(4.0)))
            .style(SurfaceStyle::Filled {
                color: scheme.surface_container_high,
            })
            .shape(Shape::capsule())
            .content_alignment(Alignment::Center)
            .on_click(move || {
                state.with_mut(|s| s.toggle_display_mode());
            }),
        move || {
            text(&crate::text::TextArgs::from(
                &TextArgs::default()
                    .text(label)
                    .size(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .typography
                            .label_small
                            .font_size,
                    )
                    .color(scheme.primary),
            ));
        },
    ));
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
