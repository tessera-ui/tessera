//! Filled and outlined text field token sets grouped by interaction state.
//!
//! ## Usage
//!
//! Compare container, indicator, label and icon tokens across text field
//! states.

use tessera_components::{
    alignment::CrossAxisAlignment, button::button, flow_row::flow_row, modifier::ModifierExt as _,
    row::row, text::text,
};
use tessera_ui::{Dp, Modifier, RenderSlot, remember, tessera};

use super::{gap, prose, spec_table, subheading, type_style};

const VARIANTS: [&str; 2] = ["Text field - Filled", "Text field - Outlined"];

const GROUPS: [&str; 5] = ["Enabled", "Disabled", "Hovered", "Focused", "Error"];

const ROWS: [[&[(&str, &str)]; 5]; 2] = [
    [
        &[
            ("Container color", "Surface container highest"),
            ("Container shape", "Extra small top (4dp)"),
            ("Container height", "56dp"),
            ("Active indicator color", "On surface variant"),
            ("Active indicator height", "1dp"),
            ("Label text color", "On surface variant"),
            ("Label text type", "Body large; Body small when populated"),
            ("Input text color", "On surface"),
            ("Input text type", "Body large"),
            ("Supporting text color", "On surface variant"),
            ("Supporting text type", "Body small"),
            ("Leading and trailing icon color", "On surface variant"),
            ("Leading and trailing icon size", "24dp"),
            ("Caret color", "Primary"),
        ],
        &[
            ("Container color", "On surface, 4% opacity"),
            ("Active indicator color", "On surface, 38% opacity"),
            ("Label text color", "On surface, 38% opacity"),
            ("Input text color", "On surface, 38% opacity"),
            ("Supporting text color", "On surface, 38% opacity"),
            ("Leading and trailing icon color", "On surface, 38% opacity"),
        ],
        &[
            ("State layer color", "On surface, 8% opacity"),
            ("Active indicator color", "On surface"),
            ("Label text color", "On surface variant"),
            ("Supporting text color", "On surface variant"),
            ("Leading and trailing icon color", "On surface variant"),
        ],
        &[
            ("Active indicator color", "Primary"),
            ("Active indicator height", "2dp"),
            ("Label text color", "Primary"),
            ("Input text color", "On surface"),
            ("Supporting text color", "On surface variant"),
            ("Leading and trailing icon color", "On surface variant"),
            ("Caret color", "Primary"),
        ],
        &[
            ("Active indicator color", "Error"),
            ("Hovered active indicator color", "On error container"),
            ("Label text color", "Error"),
            ("Supporting text color", "Error"),
            ("Trailing icon color", "Error"),
            ("Caret color", "Error"),
        ],
    ],
    [
        &[
            ("Container shape", "Extra small (4dp)"),
            ("Container height", "56dp"),
            ("Outline color", "Outline"),
            ("Outline width", "1dp"),
            ("Label text color", "On surface variant"),
            ("Label text type", "Body large; Body small when populated"),
            ("Input text color", "On surface"),
            ("Input text type", "Body large"),
            ("Supporting text color", "On surface variant"),
            ("Supporting text type", "Body small"),
            ("Leading and trailing icon color", "On surface variant"),
            ("Leading and trailing icon size", "24dp"),
            ("Caret color", "Primary"),
        ],
        &[
            ("Outline color", "On surface, 12% opacity"),
            ("Label text color", "On surface, 38% opacity"),
            ("Input text color", "On surface, 38% opacity"),
            ("Supporting text color", "On surface, 38% opacity"),
            ("Leading and trailing icon color", "On surface, 38% opacity"),
        ],
        &[
            ("Outline color", "On surface"),
            ("Label text color", "On surface variant"),
            ("Supporting text color", "On surface variant"),
            ("Leading and trailing icon color", "On surface variant"),
        ],
        &[
            ("Outline color", "Primary"),
            ("Outline width", "2dp"),
            ("Label text color", "Primary"),
            ("Input text color", "On surface"),
            ("Supporting text color", "On surface variant"),
            ("Leading and trailing icon color", "On surface variant"),
            ("Caret color", "Primary"),
        ],
        &[
            ("Outline color", "Error"),
            ("Hovered outline color", "On error container"),
            ("Label text color", "Error"),
            ("Supporting text color", "Error"),
            ("Trailing icon color", "Error"),
            ("Caret color", "Error"),
        ],
    ],
];

#[tessera]
pub(super) fn token_sets() {
    let variant = remember(|| 0_usize);
    let expanded = remember(|| [[false; 5]; 2]);
    prose("Browse the component elements, attributes, tokens, and their values.");
    gap(Dp(8.0));
    text()
        .content(
            "Reference values follow the Material 3 filled and outlined text field token specification. They have not been checked one by one against the current token explorer on m3.material.io.",
        )
        .style(type_style(14.0, 20.0));
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for (index, name) in VARIANTS.into_iter().enumerate() {
                let selected = variant.get() == index;
                let choice = button();
                let choice = if selected {
                    choice.tonal()
                } else {
                    choice.outlined()
                };
                choice
                    .accessibility_label(format!(
                        "{name} token set{}",
                        if selected { ", selected" } else { "" }
                    ))
                    .on_click(move || variant.set(index))
                    .child_shared(RenderSlot::new(move || {
                        text().content(name);
                    }));
            }
            let index = variant.get();
            let all_open = expanded.with(|state| state[index].iter().all(|open| *open));
            let action = if all_open {
                "Collapse all"
            } else {
                "Expand all"
            };
            button()
                .text()
                .accessibility_label(action)
                .on_click(move || {
                    let index = variant.get();
                    expanded.with_mut(|state| {
                        let open = !state[index].iter().all(|open| *open);
                        state[index] = [open; 5];
                    });
                })
                .child(move || {
                    text().content(action);
                });
        });
    gap(Dp(16.0));
    let index = variant.get();
    for (group, title) in GROUPS.into_iter().enumerate() {
        let open = expanded.with(|state| state[index][group]);
        button()
            .text()
            .modifier(Modifier::new().fill_max_width())
            .accessibility_label(format!(
                "{title} tokens, {}",
                if open { "expanded" } else { "collapsed" }
            ))
            .on_click(move || {
                expanded.with_mut(|state| state[index][group] = !state[index][group]);
            })
            .child(move || {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        text()
                            .content(title)
                            .style(type_style(16.0, 24.0))
                            .modifier(Modifier::new().weight(1.0));
                        text()
                            .content(if open { "▾" } else { "▸" })
                            .style(type_style(16.0, 24.0));
                    });
            });
        if open {
            spec_table(
                vec!["Token".into(), "Value".into()],
                ROWS[index][group]
                    .iter()
                    .map(|(name, value)| vec![(*name).into(), (*value).into()])
                    .collect(),
            );
        }
        gap(Dp(8.0));
    }
    gap(Dp(24.0));
    subheading("Color role summary".into());
}
