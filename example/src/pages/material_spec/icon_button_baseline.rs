//! Historical baseline icon-button token comparison.
//!
//! ## Usage
//!
//! Compare deprecated baseline geometry with current Expressive controls.

use tessera_components::{
    button::button,
    flow_row::flow_row,
    icon_button::{IconButtonSize, IconButtonVariant, icon_button},
    material_icons::filled,
    text::text,
};
use tessera_ui::{Dp, remember, tessera};

use super::super::{gap, spec_table, type_style};

#[tessera]
pub(super) fn figure() {
    text()
        .content("Filled, tonal, and outlined baseline icon button tokens are deprecated in favor of the Expressive token sets above. Select a set to compare its baseline geometry and default color roles; the preview uses the current theme.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    let variant = remember(|| IconButtonVariant::Filled);
    let actions = remember(|| 0_u32);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for (value, label) in [
                (IconButtonVariant::Filled, "[Deprecated] Filled"),
                (IconButtonVariant::FilledTonal, "[Deprecated] Tonal"),
                (IconButtonVariant::Outlined, "[Deprecated] Outlined"),
            ] {
                button()
                    .text()
                    .accessibility_label(format!("{label} baseline tokens"))
                    .on_click(move || variant.set(value))
                    .child(move || {
                        text().content(label);
                    });
            }
        });
    gap(Dp(24.0));
    let (prefix, container, icon, outline) = match variant.get() {
        IconButtonVariant::Filled => (
            "md.comp.filled-icon-button",
            "Primary",
            "On primary",
            "None",
        ),
        IconButtonVariant::FilledTonal => (
            "md.comp.filled-tonal-icon-button",
            "Secondary container",
            "On secondary container",
            "None",
        ),
        _ => (
            "md.comp.outlined-icon-button",
            "Transparent",
            "On surface variant",
            "Outline",
        ),
    };
    icon_button()
        .variant(variant.get())
        .size(IconButtonSize::Small)
        .icon(filled::EDIT_SVG)
        .accessibility_label("Baseline icon button preview")
        .accessibility_description(format!("Activations: {}", actions.get()))
        .on_click(move || actions.set(actions.get() + 1));
    gap(Dp(16.0));
    text()
        .content(format!("{prefix} · default state"))
        .style(type_style(16.0, 24.0));
    gap(Dp(16.0));
    spec_table(
        vec!["Token".into(), "Baseline value".into()],
        vec![
            vec!["Container size".into(), "40dp".into()],
            vec!["Container width".into(), "40dp".into()],
            vec!["Container shape".into(), "Full".into()],
            vec!["Icon size".into(), "24dp".into()],
            vec!["Container color".into(), container.into()],
            vec!["Icon color".into(), icon.into()],
            vec!["Outline color".into(), outline.into()],
        ],
    );
}
