//! Historical baseline icon-button token comparison.
//!
//! ## Usage
//!
//! Compare deprecated baseline geometry with current Expressive controls.

use tessera_components::{
    button::button,
    flow_row::flow_row,
    icon_button::{IconButtonVariant, icon_button},
    material_icons::filled,
    text::text,
};
use tessera_ui::{Dp, remember, tessera};

use crate::res::I18N;

use super::super::{gap, spec_table, type_style};

#[tessera]
pub(super) fn figure() {
    text()
        .content(I18N.text("ui-filled-tonal-and-outlined-baseline-icon-button-tokens-are-deprecated-in-favor-of-the-expre"))
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    let variant = remember(|| IconButtonVariant::Filled);
    let actions = remember(|| 0_u32);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for (value, label) in [
                (IconButtonVariant::Filled, "ui-deprecated-filled"),
                (IconButtonVariant::FilledTonal, "ui-deprecated-tonal"),
                (IconButtonVariant::Outlined, "ui-deprecated-outlined"),
            ] {
                button()
                    .text()
                    .accessibility_label(I18N.format(
                        "ui-value-baseline-tokens",
                        [("v0", I18N.text(&label).into())],
                    ))
                    .on_click(move || variant.set(value))
                    .child(move || {
                        text().content(I18N.text(label));
                    });
            }
        });
    gap(Dp(24.0));
    let (prefix, container, icon, outline) = match variant.get() {
        IconButtonVariant::Filled => (
            "md.comp.filled-icon-button",
            "Primary",
            "On primary",
            "ui-none",
        ),
        IconButtonVariant::FilledTonal => (
            "md.comp.filled-tonal-icon-button",
            "Secondary container",
            "On secondary container",
            "ui-none",
        ),
        _ => (
            "md.comp.outlined-icon-button",
            "ui-transparent",
            "On surface variant",
            "Outline",
        ),
    };
    icon_button()
        .variant(variant.get())
        .icon(filled::EDIT_SVG)
        .accessibility_label(I18N.text("ui-baseline-icon-button-preview"))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
        )
        .on_click(move || actions.set(actions.get() + 1));
    gap(Dp(16.0));
    text()
        .content(I18N.format(
            "ui-value-default-state",
            [("v0", I18N.text(&prefix).into())],
        ))
        .style(type_style(16.0, 24.0));
    gap(Dp(16.0));
    spec_table(
        vec!["ui-token".into(), "ui-baseline-value".into()],
        vec![
            vec!["ui-container-size".into(), "40dp".into()],
            vec!["ui-container-width".into(), "40dp".into()],
            vec!["ui-container-shape".into(), "ui-full".into()],
            vec!["ui-icon-size".into(), "24dp".into()],
            vec!["ui-container-color".into(), container.into()],
            vec!["ui-icon-color".into(), icon.into()],
            vec!["ui-outline-color".into(), outline.into()],
        ],
    );
}
