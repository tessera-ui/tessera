//! Floating action menu specifications and creation actions.
//!
//! ## Usage
//!
//! Explore related primary actions, theme colors and menu dismissal.

use tessera_components::{
    column::column,
    floating_action_button_menu::{
        FloatingActionButtonMenuColor, FloatingActionButtonMenuItem, FloatingActionButtonMenuState,
        floating_action_button_menu,
    },
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn fab_menu_spec() {
    action_article(
        Kind::FabMenu,
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(|| {
                    example("Variant", FloatingActionButtonMenuColor::Primary);
                }),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(anatomy),
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
            example("Overview", FloatingActionButtonMenuColor::Primary);
        }),
        RenderSlot::new(anatomy),
        RenderSlot::new(|| {
            example("Behavior", FloatingActionButtonMenuColor::Primary);
        }),
    );
}

#[tessera]
fn example(#[prop(into)] label: String, color: FloatingActionButtonMenuColor) {
    let state = remember(FloatingActionButtonMenuState::new);
    let status = remember(|| "Choose a creation action".to_owned());
    let theme = use_context::<MaterialTheme>().expect("theme").get();
    let caption = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} FAB menu")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            floating_action_button_menu()
                .state(state)
                .color(color)
                .open_label(format!("Open {label} actions"))
                .close_label(format!("Close {label} actions"))
                .modifier(Modifier::new().fill_max_width().height(Dp(320.0)))
                .items(vec![
                    FloatingActionButtonMenuItem::new(
                        "New document",
                        Some(filled::EDIT_SVG.into()),
                        Callback::new(move || status.set("Document created".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "New folder",
                        Some(filled::FOLDER_SVG.into()),
                        Callback::new(move || status.set("Folder created".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "Upload file",
                        Some(filled::UPLOAD_SVG.into()),
                        Callback::new(move || status.set("File uploaded".into())),
                    ),
                ])
                .main_content(move || {
                    surface()
                        .style(theme.color_scheme.surface_container_low.into())
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            column()
                                .modifier(Modifier::new().padding_all(Dp(24.0)))
                                .children(move || {
                                    text()
                                        .content("Create something")
                                        .style(theme.typography.title_large);
                                    gap(Dp(8.0));
                                    text().content(status.get());
                                });
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!("{caption}: {}", status.get()))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn configurations() {
    for (name, color) in [
        ("Primary", FloatingActionButtonMenuColor::Primary),
        ("Secondary", FloatingActionButtonMenuColor::Secondary),
        ("Tertiary", FloatingActionButtonMenuColor::Tertiary),
    ] {
        subheading(name.into());
        gap(Dp(24.0));
        example(name, color);
        gap(Dp(32.0));
    }
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Close button".into(), "56 × 56dp · fully round".into()],
            vec!["Action item".into(), "56dp height · fully round".into()],
            vec!["Action icon".into(), "24dp".into()],
            vec!["Action label".into(), "Title medium".into()],
            vec!["Window margins".into(), "16dp".into()],
        ],
    );
}

#[tessera]
fn anatomy() {
    example("Anatomy", FloatingActionButtonMenuColor::Primary);
    gap(Dp(24.0));
    text()
        .content("1  Close button    2  Menu actions")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn colors() {
    for dark in [false, true] {
        subheading(if dark {
            "Dark scheme".into()
        } else {
            "Light scheme".into()
        });
        gap(Dp(24.0));
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                example(
                    if dark { "Dark" } else { "Light" },
                    FloatingActionButtonMenuColor::Primary,
                );
            });
        gap(Dp(32.0));
    }
    spec_table(
        vec![
            "Set".into(),
            "Action / content".into(),
            "Close / content".into(),
        ],
        vec![
            vec![
                "Primary".into(),
                "Primary container / on primary container".into(),
                "Primary / on primary".into(),
            ],
            vec![
                "Secondary".into(),
                "Secondary container / on secondary container".into(),
                "Secondary / on secondary".into(),
            ],
            vec![
                "Tertiary".into(),
                "Tertiary container / on tertiary container".into(),
                "Tertiary / on tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    text().content("Open the actions, navigate with arrow keys, and activate a choice. Escape, the close button, and a click outside dismiss the list and return focus to the trigger.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    example("State", FloatingActionButtonMenuColor::Primary);
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["Property".into(), "Value".into()],
        vec![
            vec!["Close button".into(), "56dp".into()],
            vec!["Action height".into(), "56dp · medium button".into()],
            vec!["Maximum action count".into(), "6".into()],
            vec!["FAB margins".into(), "16dp".into()],
            vec![
                "Alignment".into(),
                "Menu and close button share the trailing edge".into(),
            ],
        ],
    );
}
