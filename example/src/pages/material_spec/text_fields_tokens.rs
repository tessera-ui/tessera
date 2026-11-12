//! Filled and outlined text field token sets grouped by interaction state.
//!
//! ## Usage
//!
//! Compare container, indicator, label and icon tokens across text field
//! states.

use tessera_components::{
    alignment::CrossAxisAlignment, button::button, flow_row::flow_row, row::row, text::text,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{Dp, Modifier, remember, tessera};

use crate::res::I18N;

use super::{gap, prose, spec_table, subheading, type_style};

const VARIANTS: [&str; 2] = ["ui-text-field-filled", "ui-text-field-outlined"];

const GROUPS: [&str; 5] = [
    "ui-enabled",
    "ui-disabled",
    "ui-hovered",
    "ui-focused",
    "ui-error",
];

const ROWS: [[&[(&str, &str)]; 5]; 2] = [
    [
        &[
            ("ui-container-color", "Surface container highest"),
            ("ui-container-shape", "ui-extra-small-top-4dp"),
            ("ui-container-height", "56dp"),
            ("ui-active-indicator-color", "On surface variant"),
            ("ui-active-indicator-height", "1dp"),
            ("ui-label-text-color", "On surface variant"),
            (
                "ui-label-text-type",
                "ui-body-large-body-small-when-populated",
            ),
            ("ui-input-text-color", "On surface"),
            ("ui-input-text-type", "Body large"),
            ("ui-supporting-text-color", "On surface variant"),
            ("ui-supporting-text-type", "Body small"),
            ("ui-leading-and-trailing-icon-color", "On surface variant"),
            ("ui-leading-and-trailing-icon-size", "24dp"),
            ("ui-caret-color", "Primary"),
        ],
        &[
            ("ui-container-color", "ui-on-surface-4-opacity"),
            ("ui-active-indicator-color", "ui-on-surface-38-opacity"),
            ("ui-label-text-color", "ui-on-surface-38-opacity"),
            ("ui-input-text-color", "ui-on-surface-38-opacity"),
            ("ui-supporting-text-color", "ui-on-surface-38-opacity"),
            (
                "ui-leading-and-trailing-icon-color",
                "ui-on-surface-38-opacity",
            ),
        ],
        &[
            ("ui-state-layer-color", "ui-on-surface-8-opacity"),
            ("ui-active-indicator-color", "On surface"),
            ("ui-label-text-color", "On surface variant"),
            ("ui-supporting-text-color", "On surface variant"),
            ("ui-leading-and-trailing-icon-color", "On surface variant"),
        ],
        &[
            ("ui-active-indicator-color", "Primary"),
            ("ui-active-indicator-height", "2dp"),
            ("ui-label-text-color", "Primary"),
            ("ui-input-text-color", "On surface"),
            ("ui-supporting-text-color", "On surface variant"),
            ("ui-leading-and-trailing-icon-color", "On surface variant"),
            ("ui-caret-color", "Primary"),
        ],
        &[
            ("ui-active-indicator-color", "Error"),
            ("ui-hovered-active-indicator-color", "On error container"),
            ("ui-label-text-color", "Error"),
            ("ui-supporting-text-color", "Error"),
            ("ui-trailing-icon-color", "Error"),
            ("ui-caret-color", "Error"),
        ],
    ],
    [
        &[
            ("ui-container-shape", "ui-extra-small-4dp-6024"),
            ("ui-container-height", "56dp"),
            ("ui-outline-color", "Outline"),
            ("ui-outline-width", "1dp"),
            ("ui-label-text-color", "On surface variant"),
            (
                "ui-label-text-type",
                "ui-body-large-body-small-when-populated",
            ),
            ("ui-input-text-color", "On surface"),
            ("ui-input-text-type", "Body large"),
            ("ui-supporting-text-color", "On surface variant"),
            ("ui-supporting-text-type", "Body small"),
            ("ui-leading-and-trailing-icon-color", "On surface variant"),
            ("ui-leading-and-trailing-icon-size", "24dp"),
            ("ui-caret-color", "Primary"),
        ],
        &[
            ("ui-outline-color", "ui-on-surface-12-opacity"),
            ("ui-label-text-color", "ui-on-surface-38-opacity"),
            ("ui-input-text-color", "ui-on-surface-38-opacity"),
            ("ui-supporting-text-color", "ui-on-surface-38-opacity"),
            (
                "ui-leading-and-trailing-icon-color",
                "ui-on-surface-38-opacity",
            ),
        ],
        &[
            ("ui-outline-color", "On surface"),
            ("ui-label-text-color", "On surface variant"),
            ("ui-supporting-text-color", "On surface variant"),
            ("ui-leading-and-trailing-icon-color", "On surface variant"),
        ],
        &[
            ("ui-outline-color", "Primary"),
            ("ui-outline-width", "2dp"),
            ("ui-label-text-color", "Primary"),
            ("ui-input-text-color", "On surface"),
            ("ui-supporting-text-color", "On surface variant"),
            ("ui-leading-and-trailing-icon-color", "On surface variant"),
            ("ui-caret-color", "Primary"),
        ],
        &[
            ("ui-outline-color", "Error"),
            ("ui-hovered-outline-color", "On error container"),
            ("ui-label-text-color", "Error"),
            ("ui-supporting-text-color", "Error"),
            ("ui-trailing-icon-color", "Error"),
            ("ui-caret-color", "Error"),
        ],
    ],
];

#[tessera]
pub(super) fn token_sets() {
    let variant = remember(|| 0_usize);
    let expanded = remember(|| [[false; 5]; 2]);
    prose("ui-browse-the-component-elements-attributes-tokens-and-their-values");
    gap(Dp(8.0));
    text(I18N.text("ui-reference-values-follow-the-material-3-filled-and-outlined-text-field-token-specification"))
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
                    .accessibility_label(
                        I18N.format(
                            "ui-value-token-setvalue",
                            [
                                ("v0", I18N.text(&name).into()),
                                (
                                    "v1",
                                    I18N.text(&(if selected { "ui-selected-7004" } else { "" }))
                                        .into(),
                                ),
                            ],
                        ),
                    )
                    .on_click(move || variant.set(index))
                    .child(move || {
                        text(I18N.text(name));
                    });
            }
            let index = variant.get();
            let all_open = expanded.with(|state| state[index].iter().all(|open| *open));
            let action = if all_open {
                "ui-collapse-all"
            } else {
                "ui-expand-all"
            };
            button()
                .text()
                .accessibility_label(I18N.text(action))
                .on_click(move || {
                    let index = variant.get();
                    expanded.with_mut(|state| {
                        let open = !state[index].iter().all(|open| *open);
                        state[index] = [open; 5];
                    });
                })
                .child(move || {
                    text(I18N.text(action));
                });
        });
    gap(Dp(16.0));
    let index = variant.get();
    for (group, title) in GROUPS.into_iter().enumerate() {
        let open = expanded.with(|state| state[index][group]);
        button()
            .text()
            .modifier(Modifier::new().fill_max_width())
            .accessibility_label(
                I18N.format(
                    "ui-value-tokens-value",
                    [
                        ("v0", I18N.text(&title).into()),
                        (
                            "v1",
                            I18N.text(
                                &(if open {
                                    "ui-expanded-7100"
                                } else {
                                    "ui-collapsed-7101"
                                }),
                            )
                            .into(),
                        ),
                    ],
                ),
            )
            .on_click(move || {
                expanded.with_mut(|state| state[index][group] = !state[index][group]);
            })
            .child(move || {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        text(I18N.text(title))
                            .style(type_style(16.0, 24.0))
                            .modifier(Modifier::new().weight(1.0));
                        text(I18N.text(if open { "▾" } else { "▸" })).style(type_style(16.0, 24.0));
                    });
            });
        if open {
            spec_table(
                vec!["ui-token".into(), "ui-value".into()],
                ROWS[index][group]
                    .iter()
                    .map(|(name, value)| vec![(*name).into(), (*value).into()])
                    .collect(),
            );
        }
        gap(Dp(8.0));
    }
    gap(Dp(24.0));
    subheading("ui-color-role-summary".into());
}
