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

use crate::res::I18N;

use super::{
    super::{gap, spec_table, subheading, type_style},
    SIZES, STYLES, name,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "icon_button_tokens_tests.rs"]
mod tests;

const SETS: [&str; 9] = [
    "ui-filled-color",
    "ui-tonal-color",
    "ui-outlined-color",
    "ui-standard-color",
    "ui-extra-small-size",
    "ui-small-size",
    "ui-medium-size",
    "ui-large-size",
    "ui-extra-large-size",
];

#[tessera]
pub(super) fn figure() {
    let selected = remember(|| 0_usize);
    let shared = remember(|| false);
    text()
        .content(I18N.text("ui-icon-button-tokens-are-organized-by-common-behavior-color-style-and-size-choose-a-token-se"))
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    subheading("ui-color".into());
    gap(Dp(8.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for index in 0..4 {
                button()
                    .tonal()
                    .selected(selected.get() == index)
                    .accessibility_label(I18N.format(
                        "ui-show-value-tokens",
                        [("v0", I18N.text(&(SETS[index])).into())],
                    ))
                    .on_click(move || selected.set(index))
                    .child(move || {
                        text().content(I18N.text(name(STYLES[index])));
                    });
            }
        });
    gap(Dp(16.0));
    subheading("ui-size".into());
    gap(Dp(8.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for index in 0..5 {
                button()
                    .tonal()
                    .selected(selected.get() == index + 4)
                    .accessibility_label(I18N.format(
                        "ui-show-value-tokens",
                        [("v0", I18N.text(&(SETS[index + 4])).into())],
                    ))
                    .on_click(move || selected.set(index + 4))
                    .child(move || {
                        text().content(I18N.text(["ui-xs", "S", "M", "L", "ui-xl"][index]));
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
    spec_table(vec!["ui-token".into(), "ui-value".into()], rows);
    gap(Dp(24.0));
    preview(selected.get());
    gap(Dp(24.0));
    button()
        .text()
        .accessibility_label(I18N.text(if shared.get() {
            "ui-hide-shared-icon-button-values"
        } else {
            "ui-show-shared-icon-button-values"
        }))
        .on_click(move || shared.set(!shared.get()))
        .child(move || {
            text().content(I18N.text(if shared.get() {
                "ui-hide-shared-behavior-summary"
            } else {
                "ui-show-shared-behavior-summary"
            }));
        });
    if shared.get() {
        gap(Dp(16.0));
        spec_table(
            vec!["ui-shared-value".into(), "ui-value".into()],
            common_rows(),
        );
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
        ("ui-minimum-target-width", "48dp".into()),
        ("ui-minimum-target-height", "48dp".into()),
        ("ui-round-container-shape", "ui-full".into()),
        (
            "ui-selected-shape",
            "ui-opposite-of-the-resting-round-or-square-shape".into(),
        ),
        ("ui-hovered-state-layer-opacity", "8%".into()),
        ("ui-focused-state-layer-opacity", "10%".into()),
        ("ui-pressed-state-layer-opacity", "10%".into()),
        ("ui-disabled-icon-opacity", "38%".into()),
        ("ui-disabled-filled-tonal-container-opacity", "10%".into()),
        ("ui-disabled-outline-opacity", "38%".into()),
        ("ui-disabled-content-role", "ui-on-surface".into()),
    ])
}

fn size_rows(size: IconButtonSize) -> Vec<Vec<String>> {
    use tessera_components::icon_button::IconButtonWidth;
    rows(vec![
        ("ui-container-height", format!("{}dp", size.height().0)),
        ("ui-icon-size", format!("{}dp", size.icon_size().0)),
        (
            "ui-narrow-container-width",
            format!("{}dp", size.width(IconButtonWidth::Narrow).0),
        ),
        (
            "ui-default-container-width",
            format!("{}dp", size.width(IconButtonWidth::Uniform).0),
        ),
        (
            "ui-wide-container-width",
            format!("{}dp", size.width(IconButtonWidth::Wide).0),
        ),
        (
            "ui-narrow-leading-trailing-space",
            I18N.format(
                "ui-valuedp-valuedp",
                [
                    (
                        "v0",
                        ((size.width(IconButtonWidth::Narrow).0 - size.icon_size().0) * 0.5).into(),
                    ),
                    (
                        "v1",
                        ((size.width(IconButtonWidth::Narrow).0 - size.icon_size().0) * 0.5).into(),
                    ),
                ],
            ),
        ),
        (
            "ui-default-leading-trailing-space",
            I18N.format(
                "ui-valuedp-valuedp",
                [
                    (
                        "v0",
                        ((size.width(IconButtonWidth::Uniform).0 - size.icon_size().0) * 0.5)
                            .into(),
                    ),
                    (
                        "v1",
                        ((size.width(IconButtonWidth::Uniform).0 - size.icon_size().0) * 0.5)
                            .into(),
                    ),
                ],
            ),
        ),
        (
            "ui-wide-leading-trailing-space",
            I18N.format(
                "ui-valuedp-valuedp",
                [
                    (
                        "v0",
                        ((size.width(IconButtonWidth::Wide).0 - size.icon_size().0) * 0.5).into(),
                    ),
                    (
                        "v1",
                        ((size.width(IconButtonWidth::Wide).0 - size.icon_size().0) * 0.5).into(),
                    ),
                ],
            ),
        ),
        ("ui-round-container-shape", "ui-full".into()),
        ("ui-selected-round-resting-shape", "ui-square".into()),
        ("ui-selected-square-resting-shape", "ui-round".into()),
        ("ui-md3-shape-spring-damping-ratio", "0.9".into()),
        ("ui-md3-shape-spring-stiffness", "1400".into()),
        (
            "ui-square-container-corner-radius",
            format!("{}dp", size.square_radius().0),
        ),
        (
            "ui-pressed-container-corner-radius",
            format!("{}dp", size.pressed_radius().0),
        ),
        (
            "ui-outlined-container-outline-width",
            format!("{}dp", size.outline_width().0),
        ),
    ])
}

fn color_rows(style: IconButtonVariant) -> Vec<Vec<String>> {
    let (container, icon, off_container, off_icon, on_container, on_icon) = match style {
        IconButtonVariant::Filled => (
            "ui-primary",
            "ui-on-primary",
            "ui-surface-container",
            "ui-on-surface-variant",
            "ui-primary",
            "ui-on-primary",
        ),
        IconButtonVariant::FilledTonal => (
            "ui-secondary-container",
            "ui-on-secondary-container",
            "ui-secondary-container",
            "ui-on-secondary-container",
            "ui-secondary",
            "ui-on-secondary",
        ),
        IconButtonVariant::Outlined => (
            "ui-transparent",
            "ui-on-surface-variant",
            "ui-transparent",
            "ui-on-surface-variant",
            "ui-inverse-surface",
            "ui-inverse-on-surface",
        ),
        IconButtonVariant::Standard => (
            "ui-transparent",
            "ui-on-surface-variant",
            "ui-transparent",
            "ui-on-surface-variant",
            "ui-transparent",
            "ui-primary",
        ),
    };
    let mut values = vec![
        ("ui-default-container-color", container.into()),
        ("ui-default-icon-color", icon.into()),
        ("ui-unselected-container-color", off_container.into()),
        ("ui-unselected-icon-color", off_icon.into()),
        ("ui-selected-container-color", on_container.into()),
        ("ui-selected-icon-color", on_icon.into()),
        (
            "ui-default-hovered-focused-pressed-state-layer-color",
            icon.into(),
        ),
        (
            "ui-unselected-hovered-focused-pressed-state-layer-color",
            off_icon.into(),
        ),
        (
            "ui-selected-hovered-focused-pressed-state-layer-color",
            on_icon.into(),
        ),
        ("ui-disabled-icon-color", "ui-on-surface-38".into()),
    ];
    if matches!(
        style,
        IconButtonVariant::Filled | IconButtonVariant::FilledTonal
    ) {
        values.push(("ui-disabled-container-color", "ui-on-surface-10".into()));
    }
    if style == IconButtonVariant::Outlined {
        values.extend([
            (
                "ui-default-unselected-outline-color",
                "ui-outline-variant".into(),
            ),
            ("ui-selected-outline-color", "ui-none".into()),
            ("ui-disabled-outline-color", "ui-outline-variant-38".into()),
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
                .accessibility_label(I18N.text("ui-token-preview-favorite"))
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
                .accessibility_label(I18N.text("ui-token-preview-disabled-favorite"));
            button()
                .text()
                .accessibility_label(I18N.text(if shape.get() == IconButtonShape::Round {
                    "ui-use-square-resting-shape"
                } else {
                    "ui-use-round-resting-shape"
                }))
                .on_click(move || {
                    shape.set(if shape.get() == IconButtonShape::Round {
                        IconButtonShape::Square
                    } else {
                        IconButtonShape::Round
                    });
                })
                .child(move || {
                    text().content(I18N.text(if shape.get() == IconButtonShape::Round {
                        "ui-use-square-resting-shape"
                    } else {
                        "ui-use-round-resting-shape"
                    }));
                });
        });
    gap(Dp(16.0));
    text()
        .content(
            I18N.format(
                "ui-value-preview-value-activations-value",
                [
                    ("v0", I18N.text(&name(style)).into()),
                    (
                        "v1",
                        I18N.text(
                            &(if selected.get() {
                                "ui-selected"
                            } else {
                                "ui-unselected"
                            }),
                        )
                        .into(),
                    ),
                    ("v2", (actions.get()).into()),
                ],
            ),
        )
        .style(type_style(16.0, 24.0));
}
