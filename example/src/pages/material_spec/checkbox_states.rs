//! Native checkbox interaction and validation state comparisons.
//!
//! ## Usage
//!
//! Compare independent checkbox states in light and dark themes.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    checkbox::checkbox,
    column::column,
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, accesskit::Role, modifier::FocusModifierExt as _, remember,
    scaled_box, tessera, use_context,
};

use super::{
    super::{ACCENT, annotations, gap, spec_table, type_style},
    pointer,
};

const COLUMNS: [&str; 5] = ["Enabled", "Disabled", "Hovered", "Focused", "Pressed"];
const ROWS: [&str; 6] = [
    "selected",
    "mixed",
    "unselected",
    "error selected",
    "error mixed",
    "error unselected",
];

#[tessera]
pub(super) fn states() {
    let available = remember(|| Dp(1024.0));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if next.0 > 0.0 && next != available.get() {
                    available.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box((available.get().0 / 1024.0) as f32).child(|| {
                column().children(|| {
                    for dark in [false, true] {
                        material_theme()
                            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                            .child(move || {
                                panel(dark);
                            });
                    }
                });
            });
        });
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
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .required_size(Dp(1024.0), Dp(757.0))
                .background(scheme.surface)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!(
                        "{} checkbox states",
                        if dark { "Dark" } else { "Light" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            for column in 0..5 {
                let center = [119.0, 315.0, 510.0, 705.0, 899.0][column];
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(center - 12.0), Dp(84.0))
                            .size(Dp(24.0), Dp(24.0))
                            .background_with_shape(scheme.on_surface, Shape::Ellipse)
                            .semantics(SemanticsArgs {
                                hidden: true,
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
                    cell(dark, column, row);
                }
            }
            hover_pointers();
        });
}

#[tessera]
fn cell(dark: bool, column: usize, row: usize) {
    let checked = remember(move || row % 3 == 0);
    let mixed = remember(move || row % 3 == 1);
    let visual_left = [110.0, 305.0, 501.0, 697.0, 890.0][column];
    let visual_top = match (column, row) {
        (4, 0) => 170.0,
        (4, 1) => 260.0,
        (3, 2) => 352.0,
        (_, 0) => 174.0,
        (_, 1) => 264.0,
        (_, 2) => 354.0,
        (_, 3) => 453.0,
        (_, 4) => 550.0,
        _ => 642.0,
    };
    let presentation = [
        InteractionVisualState::Rest,
        InteractionVisualState::Rest,
        InteractionVisualState::Hover,
        InteractionVisualState::Focus,
        InteractionVisualState::Pressed,
    ][column];
    checkbox()
        .checked(checked.get())
        .indeterminate(mixed.get())
        .is_error(row >= 3)
        .disabled(column == 1)
        .presentation_state(presentation)
        .accessibility_label(format!(
            "{} {} {} checkbox state",
            if dark { "Dark" } else { "Light" },
            COLUMNS[column],
            ROWS[row]
        ))
        .modifier(Modifier::new().offset(Dp(visual_left - 15.0), Dp(visual_top - 15.0)))
        .on_toggle(move |value| {
            checked.set(value);
            mixed.set(false);
        });
}

#[tessera]
fn hover_pointers() {
    let svg = pointer::hand_paths(
        &[
            (511.0, 190.0),
            (511.0, 278.0),
            (511.0, 369.0),
            (511.0, 468.0),
            (511.0, 565.0),
            (511.0, 657.0),
        ],
        0.5,
    );
    boxed()
        .modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }))
        .children(move || {
            annotations::lines(Dp(1024.0), Dp(757.0), svg.clone());
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "checkbox_states_tests.rs"]
mod tests;
