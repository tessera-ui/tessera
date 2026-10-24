//! Native calendar day and year state comparisons.
//!
//! ## Usage
//!
//! Compare calendar interaction states in light and dark themes.

use std::fmt::Write as _;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    date_picker::{CalendarDate, CalendarRangePosition, date_picker_day, date_picker_year},
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera, use_context};

use super::super::{ACCENT, annotations, gap, spec_table, type_style};

const COLUMNS: [&str; 5] = ["Enabled", "Disabled", "Hovered", "Focused", "Pressed"];
const KINDS: [&str; 6] = [
    "date",
    "today",
    "selected date",
    "range date",
    "year",
    "selected year",
];

#[tessera]
pub(super) fn states() {
    super::date_canvas(
        1024.0,
        1246.0,
        RenderSlot::new(|| {
            column().children(|| {
                for dark in [false, true] {
                    material_theme()
                        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                        .child(move || {
                            panel(dark);
                        });
                }
            });
        }),
    );
    gap(Dp(24.0));
    spec_table(
        vec!["State".into(), "Interaction".into()],
        COLUMNS
            .iter()
            .enumerate()
            .map(|(index, label)| vec![(index + 1).to_string(), (*label).into()])
            .collect(),
    );
}

#[tessera]
fn panel(dark: bool) {
    let height = if dark { 622.0 } else { 624.0 };
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(1024.0), Dp(height))
                .background(scheme.surface)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    merge_descendants: true,
                    label: Some(format!(
                        "{} calendar states",
                        if dark { "Dark" } else { "Light" }
                    )),
                    ..Default::default()
                }),
        )
        .children(move || {
            dot_grid(height);
            for column in 0..5 {
                let x = 122.0 + column as f64 * 196.0;
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size(Dp(24.0), Dp(24.0))
                            .offset(Dp(x - 12.0), Dp(33.0))
                            .background_with_shape(scheme.on_surface, Shape::CAPSULE)
                            .semantics(SemanticsArgs {
                                role: Some(Role::Note),
                                label: Some(format!(
                                    "{} calendar state {}: {}",
                                    if dark { "Dark" } else { "Light" },
                                    column + 1,
                                    COLUMNS[column]
                                )),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        text()
                            .content((column + 1).to_string())
                            .style(type_style(14.0, 20.0))
                            .color(scheme.surface);
                    });
                for row in 0..6 {
                    if column == 1 && !matches!(row, 0 | 4) {
                        continue;
                    }
                    state_cell(dark, column, row);
                }
            }
        });
}

#[tessera]
fn dot_grid(height: f64) {
    let path = remember(move || {
        let mut path = String::new();
        for y in (9..height as usize).step_by(8) {
            for x in (10..1024).step_by(8) {
                write!(&mut path, "M{x} {y}h1v1h-1z").unwrap();
            }
        }
        path
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let color = scheme.outline;
    let channel = |value: f32| {
        let value = value.clamp(0.0, 1.0);
        ((if value <= 0.0031308 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }) * 255.0)
            .round() as u8
    };
    annotations::lines(
        Dp(1024.0),
        Dp(height),
        format!(
            "<path d='{}' fill='#{:02x}{:02x}{:02x}' fill-opacity='.5'/>",
            path.get(),
            channel(color.r),
            channel(color.g),
            channel(color.b)
        ),
    );
}

#[tessera]
fn state_cell(dark: bool, column: usize, row: usize) {
    let selected = remember(move || matches!(row, 2 | 5));
    let enabled = column != 1;
    let presentation = match column {
        2 => Some(InteractionVisualState::Hover),
        3 => Some(InteractionVisualState::Focus),
        4 => Some(InteractionVisualState::Pressed),
        _ => None,
    };
    let x = 122.0 + column as f64 * 196.0;
    let y = [115.0, 204.0, 291.0, 380.0, 468.0, 561.0][row];
    let (width, height) = if row == 3 {
        (78.0, 58.0)
    } else if row < 4 {
        (48.0, 48.0)
    } else {
        (112.0, 52.0)
    };
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(width), Dp(height))
                .offset(Dp(x - width / 2.0), Dp(y - height / 2.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    merge_descendants: true,
                    label: Some(cell_name(dark, column, row)),
                    ..Default::default()
                }),
        )
        .children(move || {
            if row == 3 {
                range_context(dark, column).presentation_optional(presentation);
            } else if row < 4 {
                date_picker_day(CalendarDate::new(2025, 8, 17).unwrap())
                    .selected(selected.get())
                    .is_today(row == 1)
                    .in_range(row == 3)
                    .range_position_optional((row == 3).then_some(CalendarRangePosition::Middle))
                    .enabled(enabled)
                    .presentation_state_optional(presentation)
                    .label_formatter(|_| "00".into())
                    .accessibility_description(cell_name(dark, column, row))
                    .on_select(move |_| selected.set(true));
            } else {
                date_picker_year(2025)
                    .selected(selected.get())
                    .enabled(enabled)
                    .presentation_state_optional(presentation)
                    .accessibility_description(cell_name(dark, column, row))
                    .on_select(move |_| selected.set(true));
            }
        });
}

#[tessera]
fn range_context(dark: bool, column: usize, presentation: Option<InteractionVisualState>) {
    let selected = remember(|| None::<CalendarDate>);
    boxed()
        .modifier(Modifier::new().size(Dp(78.0), Dp(58.0)).clip_to_bounds())
        .children(move || {
            row()
                .modifier(
                    Modifier::new()
                        .required_width(Dp(144.0))
                        .offset(Dp(-33.0), Dp(5.0)),
                )
                .children(move || {
                    for day in [16, 17, 18] {
                        let date = CalendarDate::new(2025, 8, day).unwrap();
                        date_picker_day(date)
                            .selected(selected.get() == Some(date))
                            .in_range(true)
                            .range_position(CalendarRangePosition::Middle)
                            .presentation_state_optional(if day == 17 {
                                presentation
                            } else {
                                None
                            })
                            .label_formatter(|date| {
                                if date.day() == 17 {
                                    "00".into()
                                } else {
                                    date.day().to_string()
                                }
                            })
                            .accessibility_description(cell_name(dark, column, 3))
                            .on_select(move |date| selected.set(Some(date)));
                    }
                });
        });
}

fn cell_name(dark: bool, column: usize, row: usize) -> String {
    format!(
        "{} {} {} state",
        if dark { "Dark" } else { "Light" },
        COLUMNS[column],
        KINDS[row]
    )
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "date_states_tests.rs"]
mod tests;
