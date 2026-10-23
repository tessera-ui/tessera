//! Interactive radio button specifications.
//!
//! ## Usage
//!
//! Compare exclusive choices, selection feedback and radio button dimensions.

use tessera_components::{
    alignment::CrossAxisAlignment,
    column::column,
    modifier::ModifierExt as _,
    radio_button::{radio_button, radio_group},
    row::row,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlot, remember, tessera, use_context,
};

use super::{
    ACCENT,
    article::SpecSection,
    gap, panel,
    selection_articles::{Kind, option_label, selection_article},
    spec_table, type_style,
};

#[tessera]
pub(super) fn radio_buttons_spec() {
    selection_article(
        Kind::Radio,
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
        RenderSlot::new(|| {
            panel(RenderSlot::new(|| {
                choices("Notification preference", true);
            }));
        }),
        RenderSlot::new(|| {
            panel(RenderSlot::new(|| {
                choices("Radio button anatomy", true);
            }));
        }),
    );
}

#[tessera]
fn choices(#[prop(into)] label: String, enabled: bool) {
    let selected = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    column().children(move || {
        text().content(label.clone()).style(type_style(24.0, 32.0));
        gap(Dp(16.0));
        let label = label.clone();
        radio_group()
            .accessibility_label(label.clone())
            .content(move || {
                let label = label.clone();
                column().children(move || {
                    for (index, option) in ["All notifications", "Important only", "None"]
                        .into_iter()
                        .enumerate()
                    {
                        let focus = remember(FocusRequester::new).get();
                        let label = label.clone();
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                radio_button()
                                    .focus_requester(focus)
                                    .selected(selected.get() == index)
                                    .enabled(enabled)
                                    .on_select(move |_| selected.set(index))
                                    .accessibility_label(format!("{label}: {option}"));
                                option_label(
                                    option.into(),
                                    Callback::new(move || {
                                        selected.set(index);
                                        focus.request_focus();
                                    }),
                                    enabled,
                                )
                                .color(if enabled {
                                    scheme.on_surface
                                } else {
                                    scheme.on_surface.with_alpha(0.38)
                                });
                            });
                    }
                });
            });
    });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Selected".into(), "Unselected".into()],
        vec![
            vec!["Icon".into(), "Primary".into(), "On surface variant".into()],
            vec![
                "Disabled icon".into(),
                "On surface · 38%".into(),
                "On surface · 38%".into(),
            ],
            vec![
                "Hovered state layer".into(),
                "Primary · 8%".into(),
                "On surface · 8%".into(),
            ],
            vec![
                "Focused / pressed state layer".into(),
                "Primary · 10%".into(),
                "On surface · 10%".into(),
            ],
            vec![
                "Adjacent label".into(),
                "On surface".into(),
                "On surface".into(),
            ],
        ],
    );
}

#[tessera]
fn scheme(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let theme = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get();
            surface()
                .style(theme.color_scheme.surface.into())
                .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                .child(move || {
                    choices(if dark { "Dark theme" } else { "Light theme" }, true);
                });
        });
}

#[tessera]
fn colors() {
    scheme(false);
    gap(Dp(24.0));
    scheme(true);
    gap(Dp(24.0));
    text()
        .content("Adjacent labels use On surface regardless of selection.")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn states() {
    text()
        .content(
            "Hover, use Tab and the arrow keys, or hold a choice to inspect actual input feedback.",
        )
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        choices("Enabled choices", true);
    }));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        choices("Disabled choices", false);
    }));
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        radio_button()
            .selected(true)
            .accessibility_label("Radio measurement example");
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Icon".into(), "20dp".into()],
            vec!["State layer".into(), "40dp".into()],
            vec!["Target".into(), "48 × 48dp".into()],
        ],
    );
}
