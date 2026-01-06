use tessera_ui::{Dp, Modifier, State, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    button::{ButtonArgs, button},
    date_picker::{
        CalendarDate, DatePickerArgs, DatePickerDialogArgs, DatePickerState, date_picker_dialog,
        date_picker_with_state,
    },
    dialog::{DialogController, DialogProviderArgs, DialogStyle, dialog_provider_with_controller},
    lazy_list::{LazyColumnArgs, lazy_column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
    time_picker::{
        DayPeriod, TimePickerArgs, TimePickerDialogArgs, TimePickerState, time_picker_dialog,
        time_picker_with_state,
    },
};

#[derive(Clone, Copy)]
enum PickerDialog {
    Date,
    Time,
}

#[tessera]
#[shard]
pub fn date_time_picker_showcase() {
    let date_state = remember(DatePickerState::default);
    let time_state = remember(TimePickerState::default);
    let dialog_controller = remember(DialogController::default);
    let active_dialog = remember(|| PickerDialog::Date);

    dialog_provider_with_controller(
        DialogProviderArgs::new(move || {
            dialog_controller.with_mut(|c| c.close());
        })
        .style(DialogStyle::Material),
        dialog_controller,
        move || {
            date_time_picker_content(date_state, time_state, dialog_controller, active_dialog);
        },
        move || match active_dialog.get() {
            PickerDialog::Date => {
                date_picker_dialog(
                    DatePickerDialogArgs::new(date_state)
                        .title("Select date")
                        .confirm_button(move || {
                            button(
                                ButtonArgs::text(move || {
                                    dialog_controller.with_mut(|c| c.close());
                                }),
                                || text("Confirm"),
                            );
                        })
                        .dismiss_button(move || {
                            button(
                                ButtonArgs::text(move || {
                                    dialog_controller.with_mut(|c| c.close());
                                }),
                                || text("Cancel"),
                            );
                        }),
                );
            }
            PickerDialog::Time => {
                time_picker_dialog(
                    TimePickerDialogArgs::new(time_state)
                        .title("Select time")
                        .show_mode_toggle(true)
                        .confirm_button(move || {
                            button(
                                ButtonArgs::text(move || {
                                    dialog_controller.with_mut(|c| c.close());
                                }),
                                || text("Confirm"),
                            );
                        })
                        .dismiss_button(move || {
                            button(
                                ButtonArgs::text(move || {
                                    dialog_controller.with_mut(|c| c.close());
                                }),
                                || text("Cancel"),
                            );
                        }),
                );
            }
        },
    );
}

#[tessera]
fn date_time_picker_content(
    date_state: State<DatePickerState>,
    time_state: State<TimePickerState>,
    dialog_controller: State<DialogController>,
    active_dialog: State<PickerDialog>,
) {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size())
                    .content_padding(Dp(16.0)),
                move |scope| {
                    scope.item(|| {
                        text(
                            TextArgs::default()
                                .text("Date & Time Pickers")
                                .size(Dp(20.0)),
                        );
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(16.0))));

                    scope.item(|| {
                        text(
                            TextArgs::default()
                                .text("Inline Date Picker")
                                .size(Dp(16.0)),
                        );
                    });

                    scope.item(move || {
                        let label = date_state.with(|s| format_date_label(s.selected_date()));
                        text(
                            TextArgs::default()
                                .text(format!("Selected date: {label}"))
                                .size(Dp(14.0))
                                .color(
                                    use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .on_surface_variant,
                                ),
                        );
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(12.0))));

                    scope.item(move || {
                        date_picker_with_state(DatePickerArgs::default(), date_state);
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(24.0))));

                    scope.item(|| {
                        text(
                            TextArgs::default()
                                .text("Inline Time Picker")
                                .size(Dp(16.0)),
                        );
                    });

                    scope.item(move || {
                        let label = time_state.with(format_time_label);
                        text(
                            TextArgs::default()
                                .text(format!("Selected time: {label}"))
                                .size(Dp(14.0))
                                .color(
                                    use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .on_surface_variant,
                                ),
                        );
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(12.0))));

                    scope.item(move || {
                        time_picker_with_state(TimePickerArgs::default(), time_state);
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(12.0))));

                    scope.item(move || {
                        let is_24_hour = time_state.with(|s| s.is_24_hour());
                        let label = if is_24_hour {
                            "Use 12-hour clock"
                        } else {
                            "Use 24-hour clock"
                        };
                        button(
                            ButtonArgs::text(move || {
                                time_state.with_mut(|s| {
                                    s.set_is_24_hour(!s.is_24_hour());
                                });
                            }),
                            move || text(label),
                        );
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(24.0))));

                    scope.item(|| {
                        text(TextArgs::default().text("Dialog Pickers").size(Dp(16.0)));
                    });

                    scope.item(|| spacer(Modifier::new().height(Dp(8.0))));

                    scope.item(move || {
                        row(RowArgs::default(), move |row_scope| {
                            row_scope.child(move || {
                                button(
                                    ButtonArgs::filled(move || {
                                        active_dialog.set(PickerDialog::Date);
                                        dialog_controller.with_mut(|c| c.open());
                                    }),
                                    || text("Open date dialog"),
                                );
                            });

                            row_scope.child(|| {
                                spacer(Modifier::new().width(Dp(12.0)));
                            });

                            row_scope.child(move || {
                                button(
                                    ButtonArgs::filled(move || {
                                        active_dialog.set(PickerDialog::Time);
                                        dialog_controller.with_mut(|c| c.open());
                                    }),
                                    || text("Open time dialog"),
                                );
                            });
                        });
                    });
                },
            );
        },
    );
}

fn format_date_label(date: Option<CalendarDate>) -> String {
    match date {
        Some(date) => format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day()),
        None => "None".to_string(),
    }
}

fn format_time_label(state: &TimePickerState) -> String {
    let hour = state.hour_for_display();
    let minute = state.minute();
    if state.is_24_hour() {
        format!("{hour:02}:{minute:02}")
    } else {
        let period = match state.period() {
            DayPeriod::Am => "AM",
            DayPeriod::Pm => "PM",
        };
        format!("{hour:02}:{minute:02} {period}")
    }
}
