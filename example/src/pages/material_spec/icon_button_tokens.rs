//! Interactive icon-button token sets.
//!
//! ## Usage
//!
//! Inspect common, color and size tokens beside a live themed control.

use tessera_components::{
    boxed::boxed,
    button::button,
    flow_row::flow_row,
    icon_button::{IconButtonShape, IconButtonSize, IconButtonVariant, icon_button},
    material_icons::filled,
    menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::{ModifierExt as _, SemanticsArgs},
    text::text,
};
use tessera_ui::{
    Dp, Modifier, PxPosition, PxRect, PxSize, accesskit::HasPopup, modifier::FocusModifierExt as _,
    remember, tessera,
};

use super::{
    super::{gap, spec_table, subheading, type_style},
    SIZES, STYLES, name,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "icon_button_tokens_tests.rs"]
mod tests;

const SETS: [&str; 10] = [
    "Common",
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
    let menu = remember(MenuController::new);
    let anchor = remember(PxRect::default);
    let origin = remember(PxRect::default);
    text()
        .content("Icon button tokens are organized by common behavior, color style, and size. Choose a token set to inspect its values. Color roles resolve through the current theme.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .on_globally_positioned(move |rect| {
                    if origin.get() != rect {
                        origin.set(rect);
                    }
                }),
        )
        .children(move || {
            let a = anchor.get();
            let o = origin.get();
            menu_provider()
                .controller(menu)
                .placement(MenuPlacement::BelowStart)
                .anchor(MenuAnchor::new(
                    PxPosition::new(a.x - o.x, a.y - o.y),
                    PxSize::new(a.width, a.height),
                ))
                .main_content(move || {
                    button()
                        .outlined()
                        .accessibility_label(format!(
                            "Icon button token set: {}",
                            SETS[selected.get()]
                        ))
                        .modifier(
                            Modifier::new()
                                .on_globally_positioned(move |rect| {
                                    if anchor.get() != rect {
                                        anchor.set(rect);
                                    }
                                })
                                .semantics(SemanticsArgs {
                                    expanded: Some(menu.with(MenuController::is_open)),
                                    has_popup: Some(HasPopup::Menu),
                                    ..Default::default()
                                }),
                        )
                        .on_click(move || menu.with_mut(MenuController::toggle))
                        .child(move || {
                            text().content(format!("{}  ▾", SETS[selected.get()]));
                        });
                })
                .menu_content(move || {
                    for (index, label) in SETS.into_iter().enumerate() {
                        menu_item()
                            .label(label)
                            .on_click(move || selected.set(index));
                    }
                });
        });
    gap(Dp(24.0));
    subheading(SETS[selected.get()].into());
    gap(Dp(16.0));
    let rows = match selected.get() {
        0 => common_rows(),
        1..=4 => color_rows(STYLES[selected.get() - 1]),
        index => size_rows(SIZES[index - 5]),
    };
    spec_table(vec!["Token".into(), "Value".into()], rows);
    gap(Dp(24.0));
    preview(selected.get());
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
        ("Round container shape", "Full".into()),
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
    let style = if (1..=4).contains(&set) {
        STYLES[set - 1]
    } else {
        IconButtonVariant::Filled
    };
    let size = if set >= 5 {
        SIZES[set - 5]
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
