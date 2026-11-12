//! Native switch interaction state comparisons.
//!
//! ## Usage
//!
//! Compare independent switches across interaction states and themes.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    switch::switch,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, RenderSlot, accesskit::Role, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

use super::super::{ACCENT, annotations, gap, spec_table, type_style};

const LABELS: [&str; 5] = [
    "ui-enabled",
    "ui-hovered",
    "ui-focused",
    "ui-pressed",
    "ui-disabled",
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
        vec!["ui-state".into(), "ui-interaction".into()],
        LABELS
            .iter()
            .enumerate()
            .map(|(index, label)| vec![(index + 1).to_string(), (*label).into()])
            .collect(),
    );
}

#[tessera]
fn panel(dark: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let theme_name = if dark { "ui-dark" } else { "ui-light" };
    boxed()
        .modifier(
            Modifier::new()
                .required_size(Dp(1024.0), Dp(300.0))
                .background(scheme.surface)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(I18N.format(
                        "ui-switch-state-theme",
                        [("theme", I18N.text(theme_name).into())],
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            for column in 0..5 {
                let marker_x = if dark { 161.0 } else { 158.0 } + column as f64 * 176.0;
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(marker_x - 12.0), Dp(if dark { 56.0 } else { 51.0 }))
                            .size(Dp(24.0), Dp(24.0))
                            .background_with_shape(scheme.on_surface, Shape::Ellipse)
                            .semantics(SemanticsArgs {
                                hidden: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        text((column + 1).to_string())
                            .style(type_style(14.0, 20.0))
                            .color(scheme.surface);
                    });
                for selected in [true, false] {
                    cell(dark, selected, column);
                }
            }
            hover_pointers(dark);
        });
}

#[tessera]
fn hover_pointers(dark: bool) {
    let positions = if dark {
        [(327.0, 151.0), (329.0, 227.0)]
    } else {
        [(326.0, 142.0), (326.0, 227.0)]
    };
    let svg = super::pointer::hand_paths(&positions, 0.5);
    boxed()
        .modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }))
        .children(move || {
            annotations::lines(Dp(1024.0), Dp(300.0), svg.clone());
        });
}

#[tessera]
fn cell(dark: bool, selected: bool, column: usize) {
    let checked = remember(move || selected);
    let left = if dark && selected {
        [135.0, 313.0, 494.0, 665.0, 839.0][column]
    } else if dark {
        [134.0, 313.0, 494.0, 665.0, 839.0][column]
    } else {
        [132.0, 310.0, 491.0, 660.0, 840.0][column]
    };
    let track_y = match (dark, selected, column) {
        (false, true, _) => 117.0,
        (false, false, 4) => 197.0,
        (false, false, _) => 196.0,
        (true, true, _) => 121.0,
        (true, false, _) => 200.0,
    };
    let presentation = [
        InteractionVisualState::Rest,
        InteractionVisualState::Hover,
        InteractionVisualState::Focus,
        InteractionVisualState::Pressed,
        InteractionVisualState::Rest,
    ][column];
    let label = I18N.format(
        "ui-value-value-value-switch-state",
        [
            (
                "v0",
                I18N.text(&(if dark { "ui-dark" } else { "ui-light" }))
                    .into(),
            ),
            ("v1", I18N.text(&(LABELS[column])).into()),
            (
                "v2",
                I18N.text(
                    &(if selected {
                        "ui-selected-7075"
                    } else {
                        "ui-unselected-7076"
                    }),
                )
                .into(),
            ),
        ],
    );
    switch(checked.get())
        .enabled(column != 4)
        .presentation_state(presentation)
        .accessibility_label(I18N.text(label))
        .modifier(Modifier::new().offset(Dp(left), Dp(track_y - 8.0)))
        .on_toggle(move |value| checked.set(value))
        .child_optional(if checked.get() || (!dark && column != 4) {
            Some(RenderSlot::new(move || {
                icon().painter(if checked.get() {
                    filled::CHECK_SVG
                } else {
                    filled::CLOSE_SVG
                });
            }))
        } else {
            None
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "switch_states_tests.rs"]
mod tests;
