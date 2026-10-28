//! Interactive icon-button token sets.
//!
//! ## Usage
//!
//! Inspect common, color and size tokens beside a live themed control.

use tessera_components::{
    button::button,
    flow_row::flow_row,
    icon_button::{IconButtonShape, IconButtonSize, IconButtonVariant, icon_button},
    material_icons::filled,
    text::text,
};
use tessera_ui::{Dp, remember, tessera};

use super::{
    super::{gap, spec_table, subheading, type_style},
    SIZES, STYLES, name,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "icon_button_tokens_tests.rs"]
mod tests;

const SETS: [&str; 9] = [
    "Filled color",
    "Tonal color",
    "Outlined color",
    "Standard color",
    "Extra small size",
    "Small size",
    "Medium size",
    "Large size",
    "Extra large size",
];

#[tessera]
pub(super) fn figure() {
    let selected = remember(|| 0_usize);
    let shared = remember(|| false);
    text()
        .content("Icon button tokens are organized by common behavior, color style, and size. Choose a token set to inspect its values. Color roles resolve through the current theme.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    subheading("Color".into());
    gap(Dp(8.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for index in 0..4 {
                button()
                    .tonal()
                    .selected(selected.get() == index)
                    .accessibility_label(format!("Show {} tokens", SETS[index]))
                    .on_click(move || selected.set(index))
                    .child(move || {
                        text().content(name(STYLES[index]));
                    });
            }
        });
    gap(Dp(16.0));
    subheading("Size".into());
    gap(Dp(8.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for index in 0..5 {
                button()
                    .tonal()
                    .selected(selected.get() == index + 4)
                    .accessibility_label(format!("Show {} tokens", SETS[index + 4]))
                    .on_click(move || selected.set(index + 4))
                    .child(move || {
                        text().content(["XS", "S", "M", "L", "XL"][index]);
                    });
            }
        });
    gap(Dp(24.0));
    subheading(SETS[selected.get()].into());
    gap(Dp(16.0));
    let rows = match selected.get() {
        0..=3 => color_rows(STYLES[selected.get()]),
        index => size_rows(SIZES[index - 4]),
    };
    spec_table(vec!["Token".into(), "Value".into()], rows);
    gap(Dp(24.0));
    preview(selected.get());
    gap(Dp(24.0));
    button()
        .text()
        .accessibility_label(if shared.get() {
            "Hide shared icon button values"
        } else {
            "Show shared icon button values"
        })
        .on_click(move || shared.set(!shared.get()))
        .child(move || {
            text().content(if shared.get() {
                "Hide shared behavior summary"
            } else {
                "Show shared behavior summary"
            });
        });
    if shared.get() {
        gap(Dp(16.0));
        spec_table(vec!["Shared value".into(), "Value".into()], common_rows());
    }
}

fn rows(values: Vec<(&str, String)>) -> Vec<Vec<String>> {
    values
        .into_iter()
        .map(|(key, value)| vec![key.into(), value])
        .collect()
}

fn common_rows() -> Vec<Vec<String>> {
    rows(vec![
        ("Minimum target width", "48dp".into()),
        ("Minimum target height", "48dp".into()),
        ("Round container shape", "Full".into()),
        (
            "Selected shape",
            "Opposite of the resting round or square shape".into(),
        ),
        ("Hovered state layer opacity", "8%".into()),
        ("Focused state layer opacity", "10%".into()),
        ("Pressed state layer opacity", "10%".into()),
        ("Disabled icon opacity", "38%".into()),
        ("Disabled filled / tonal container opacity", "10%".into()),
        ("Disabled outline opacity", "38%".into()),
        ("Disabled content role", "On surface".into()),
    ])
}

fn size_rows(size: IconButtonSize) -> Vec<Vec<String>> {
    use tessera_components::icon_button::IconButtonWidth;
    rows(vec![
        ("Container height", format!("{}dp", size.height().0)),
        ("Icon size", format!("{}dp", size.icon_size().0)),
        (
            "Narrow container width",
            format!("{}dp", size.width(IconButtonWidth::Narrow).0),
        ),
        (
            "Default container width",
            format!("{}dp", size.width(IconButtonWidth::Uniform).0),
        ),
        (
            "Wide container width",
            format!("{}dp", size.width(IconButtonWidth::Wide).0),
        ),
        (
            "Narrow leading / trailing space",
            format!(
                "{}dp / {}dp",
                (size.width(IconButtonWidth::Narrow).0 - size.icon_size().0) * 0.5,
                (size.width(IconButtonWidth::Narrow).0 - size.icon_size().0) * 0.5
            ),
        ),
        (
            "Default leading / trailing space",
            format!(
                "{}dp / {}dp",
                (size.width(IconButtonWidth::Uniform).0 - size.icon_size().0) * 0.5,
                (size.width(IconButtonWidth::Uniform).0 - size.icon_size().0) * 0.5
            ),
        ),
        (
            "Wide leading / trailing space",
            format!(
                "{}dp / {}dp",
                (size.width(IconButtonWidth::Wide).0 - size.icon_size().0) * 0.5,
                (size.width(IconButtonWidth::Wide).0 - size.icon_size().0) * 0.5
            ),
        ),
        ("Round container shape", "Full".into()),
        ("Selected round resting shape", "Square".into()),
        ("Selected square resting shape", "Round".into()),
        ("MD3 shape spring damping ratio", "0.9".into()),
        ("MD3 shape spring stiffness", "1400".into()),
        (
            "Square container corner radius",
            format!("{}dp", size.square_radius().0),
        ),
        (
            "Pressed container corner radius",
            format!("{}dp", size.pressed_radius().0),
        ),
        (
            "Outlined container outline width",
            format!("{}dp", size.outline_width().0),
        ),
    ])
}

fn color_rows(style: IconButtonVariant) -> Vec<Vec<String>> {
    let (container, icon, off_container, off_icon, on_container, on_icon) = match style {
        IconButtonVariant::Filled => (
            "Primary",
            "On primary",
            "Surface container",
            "On surface variant",
            "Primary",
            "On primary",
        ),
        IconButtonVariant::FilledTonal => (
            "Secondary container",
            "On secondary container",
            "Secondary container",
            "On secondary container",
            "Secondary",
            "On secondary",
        ),
        IconButtonVariant::Outlined => (
            "Transparent",
            "On surface variant",
            "Transparent",
            "On surface variant",
            "Inverse surface",
            "Inverse on surface",
        ),
        IconButtonVariant::Standard => (
            "Transparent",
            "On surface variant",
            "Transparent",
            "On surface variant",
            "Transparent",
            "Primary",
        ),
    };
    let mut values = vec![
        ("Default container color", container.into()),
        ("Default icon color", icon.into()),
        ("Unselected container color", off_container.into()),
        ("Unselected icon color", off_icon.into()),
        ("Selected container color", on_container.into()),
        ("Selected icon color", on_icon.into()),
        (
            "Default hovered / focused / pressed state layer color",
            icon.into(),
        ),
        (
            "Unselected hovered / focused / pressed state layer color",
            off_icon.into(),
        ),
        (
            "Selected hovered / focused / pressed state layer color",
            on_icon.into(),
        ),
        ("Disabled icon color", "On surface · 38%".into()),
    ];
    if matches!(
        style,
        IconButtonVariant::Filled | IconButtonVariant::FilledTonal
    ) {
        values.push(("Disabled container color", "On surface · 10%".into()));
    }
    if style == IconButtonVariant::Outlined {
        values.extend([
            (
                "Default / unselected outline color",
                "Outline variant".into(),
            ),
            ("Selected outline color", "None".into()),
            ("Disabled outline color", "Outline variant · 38%".into()),
        ]);
    }
    rows(values)
}

#[tessera]
fn preview(set: usize) {
    let selected = remember(|| false);
    let shape = remember(|| IconButtonShape::Round);
    let actions = remember(|| 0_u32);
    let style = if set < 4 {
        STYLES[set]
    } else {
        IconButtonVariant::Filled
    };
    let size = if set >= 4 {
        SIZES[set - 4]
    } else {
        IconButtonSize::Small
    };
    flow_row()
        .item_spacing(Dp(16.0))
        .line_spacing(Dp(16.0))
        .children(move || {
            icon_button()
                .variant(style)
                .size(size)
                .shape_style(shape.get())
                .selected(selected.get())
                .icon(if selected.get() {
                    filled::FAVORITE_SVG
                } else {
                    filled::FAVORITE_BORDER_SVG
                })
                .accessibility_label("Token preview favorite")
                .on_click(move || {
                    selected.set(!selected.get());
                    actions.set(actions.get() + 1);
                });
            icon_button()
                .variant(style)
                .size(size)
                .shape_style(shape.get())
                .enabled(false)
                .icon(filled::FAVORITE_SVG)
                .accessibility_label("Token preview disabled favorite");
            button()
                .text()
                .accessibility_label(if shape.get() == IconButtonShape::Round {
                    "Use square resting shape"
                } else {
                    "Use round resting shape"
                })
                .on_click(move || {
                    shape.set(if shape.get() == IconButtonShape::Round {
                        IconButtonShape::Square
                    } else {
                        IconButtonShape::Round
                    });
                })
                .child(move || {
                    text().content(if shape.get() == IconButtonShape::Round {
                        "Use square resting shape"
                    } else {
                        "Use round resting shape"
                    });
                });
        });
    gap(Dp(16.0));
    text()
        .content(format!(
            "{} preview · {} · Activations: {}",
            name(style),
            if selected.get() {
                "Selected"
            } else {
                "Unselected"
            },
            actions.get()
        ))
        .style(type_style(16.0, 24.0));
}
