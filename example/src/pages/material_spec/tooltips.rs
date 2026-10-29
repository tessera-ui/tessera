//! Plain and rich tooltip specification examples.
//!
//! ## Usage
//!
//! Explain actions with contextual help and optional follow-up actions.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{MinimumInteractiveComponentEnforcement, ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
    tooltip::{
        TooltipController, TooltipPlacement, TooltipVariant, tooltip_anchor, tooltip_provider,
    },
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, provide_context, remember, tessera,
    use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn tooltips_spec() {
    article(
        "Tooltips",
        "Tooltips display brief labels or messages",
        vec![
            section("Tokens & specs", RenderSlot::new(tokens)),
            section("Plain tooltip", RenderSlot::new(|| { detail(false); })),
            section("Rich tooltip", RenderSlot::new(|| { detail(true); })),
        ],
    ).pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(|| {
                tooltip_points(vec![
                    "Use tooltips to add context to a button or other UI element.",
                    "Two variants: plain and rich.",
                    "Plain tooltips describe elements or actions of icon buttons.",
                    "Rich tooltips explain features in more detail, with an optional subhead and text actions.",
                    "Keep essential information visible without opening a tooltip.",
                ]);
                gap(Dp(32.0)); tooltip_variant_examples();
            })),
            sections: vec![section("Differences from M2", RenderSlot::new(|| {
                prose("Material 3 introduces rich tooltips alongside familiar plain labels. Color roles adapt both variants to light and dark themes.".into());
                gap(Dp(24.0));
                spec_table(vec!["Element".into(), "Material 3".into()], vec![
                    vec!["Plain tooltip".into(), "Brief, non-interactive supporting text on an inverse surface".into()],
                    vec!["Rich tooltip".into(), "A larger container with supporting text, optional subhead, and up to two text buttons".into()],
                    vec!["Color".into(), "Theme-based inverse surface, surface container, and primary roles".into()],
                ]);
            }))],
        },
        ArticlePage {
            title: "Guidelines", intro: None,
            sections: vec![
                section("Usage", RenderSlot::new(|| {
                    prose("Use a plain tooltip to identify an unfamiliar icon or action. Use a rich tooltip to explain a feature, define a term, or describe its value. Tooltips supplement the interface; they do not replace visible instructions, errors, or control labels.".into());
                    gap(Dp(24.0)); tooltip_variant_examples(); gap(Dp(24.0));
                    tooltip_points(vec![
                        "Do not repeat a label that is already visible next to the control.",
                        "Avoid images and long instructions. Keep the explanation relevant to its target.",
                        "Do not place interactive content inside a plain tooltip.",
                    ]);
                })),
                section("Anatomy", RenderSlot::new(|| {
                    subheading("Plain tooltip".into()); gap(Dp(24.0)); tooltip_figure(false, false);
                    gap(Dp(32.0)); subheading("Supporting text".into());
                    prose("Describe the action briefly, ideally on one line. Use sentence case and omit terminal punctuation for a short label.".into());
                    gap(Dp(24.0)); tooltip_writing_examples(false);
                    gap(Dp(48.0)); subheading("Rich tooltip".into()); gap(Dp(24.0)); tooltip_figure(true, false);
                    gap(Dp(32.0)); subheading("Subhead (optional)".into());
                    prose("Summarize the message in a few words, ideally on one line. A subhead is especially helpful when a rich tooltip appears automatically.".into());
                    gap(Dp(24.0)); tooltip_writing_examples(true);
                    gap(Dp(32.0)); subheading("Supporting text".into());
                    prose("Explain the feature in a concise, self-contained message. Supporting text can wrap across lines; a subhead is not a substitute for that explanation.".into());
                    gap(Dp(24.0)); subheading("Text buttons (optional)".into());
                    prose("Use one or two brief, relevant actions. Keep labels short enough to fit side by side. Provide an explicit way to dismiss a persistent explanation.".into());
                })),
                section("Placement", RenderSlot::new(|| {
                    subheading("Plain tooltips".into());
                    prose("Prefer above the target, or below an app-bar action. Leave 4dp from the visible control boundary. Keep the tooltip close to its target without covering it.".into());
                    gap(Dp(24.0)); tooltip_figure(false, true);
                    gap(Dp(32.0)); subheading("Rich tooltips".into());
                    prose("Position the explanation near its target and adjust at window edges. On desktop, a rich tooltip can be centered below the target. Do not obscure the control or information needed to understand the message.".into());
                    gap(Dp(24.0)); placement_examples(); gap(Dp(16.0));
                    prose("Corner examples request above or below placement; the tooltip flips and shifts to remain inside its available space.".into());
                })),
                section("Behavior", RenderSlot::new(|| {
                    subheading("Transient tooltips".into());
                    prose("Hover, keyboard focus, or a touch long press can reveal an explanation. Activating the control still performs its action. Keep the message available while the trigger or tooltip is engaged.".into());
                    gap(Dp(24.0)); live_tooltips();
                    gap(Dp(32.0)); subheading("Persistent rich tooltips".into());
                    prose("A deliberate click can open a persistent rich explanation with actions. Keep it visible until an action, outside interaction, or Escape dismisses it. Use a labeled trigger for the explanation rather than overloading an icon-only control's primary action.".into());
                    gap(Dp(24.0));
                    spec_table(vec!["Interaction".into(), "Example behavior".into()], vec![
                        vec!["Hover or focus".into(), "Reveal contextual help without activating the target".into()],
                        vec!["Long press".into(), "Show help for a touch target without performing its primary action".into()],
                        vec!["About collections".into(), "Open the persistent rich tooltip".into()],
                        vec!["Got it / outside press / Escape".into(), "Dismiss the persistent explanation".into()],
                    ]);
                })),
            ],
        },
        ArticlePage {
            title: "Accessibility", intro: None,
            sections: vec![
                section("Use cases", RenderSlot::new(|| {
                    tooltip_points(vec![
                        "Receive the tooltip information with assistive technology.",
                        "Reveal and dismiss the explanation with keyboard or switch input.",
                        "Activate the trigger and any rich-tooltip actions without a pointer.",
                    ]);
                })),
                section("Interaction & style", RenderSlot::new(|| {
                    prose("Give people enough time to read the message without interrupting their task. Tooltips without required actions are transient; messages with actions must remain available while people move into the popup.".into());
                    gap(Dp(24.0));
                    tooltip_points(vec![
                        "Reveal help on keyboard focus as well as hover; do not rely on hover alone.",
                        "Allow the pointer to enter the tooltip without immediately hiding it.",
                        "Keep important instructions visible in the page rather than only in a tooltip.",
                        "Use theme color pairs and legible supporting text in both light and dark themes.",
                    ]);
                    gap(Dp(24.0)); live_tooltips();
                })),
                section("Focus order", RenderSlot::new(|| {
                    prose("Keep the trigger and rich-tooltip actions in a logical sequence. A plain tooltip is a description, not another Tab stop. A rich tooltip with actions lets focus reach those actions, then continue to the page; it must not trap focus.".into());
                    gap(Dp(24.0)); tooltip_focus_sequence(); gap(Dp(24.0));
                    prose("Escape dismisses an open explanation. When dismissing from inside the popup, focus returns to its trigger. Revealing a plain tooltip must not steal focus from the control.".into());
                })),
                section("Keyboard navigation", RenderSlot::new(|| {
                    spec_table(vec!["Keys".into(), "Actions".into()], vec![
                        vec!["Tab".into(), "Move to the next control or available tooltip action".into()],
                        vec!["Shift + Tab".into(), "Move to the previous interactive element".into()],
                        vec!["Space or Enter".into(), "Activate the focused element".into()],
                        vec!["Escape".into(), "Dismiss the open tooltip; restore its trigger when leaving the popup".into()],
                    ]);
                    gap(Dp(24.0));
                    prose("Try the live example in Interaction & style: focus the favorite icon to reveal its label, then focus About collections, press Enter, and Tab to Got it.".into());
                })),
                section("Labeling elements", RenderSlot::new(|| {
                    prose("Name the trigger by its action, not its appearance. Expose non-interactive help with a Tooltip role and associate the message with its trigger. An interactive rich explanation needs popup semantics and named actions, rather than a non-interactive label containing buttons.".into());
                    gap(Dp(24.0));
                    spec_table(vec!["Element".into(), "Accessible information in this example".into()], vec![
                        vec!["Favorite icon".into(), "Add to favorites / Remove from favorites; the label follows the action".into()],
                        vec!["Plain tooltip".into(), "Tooltip role; the same brief action description".into()],
                        vec!["About collections".into(), "A labeled trigger for the rich explanation".into()],
                        vec!["Interactive rich tooltip".into(), "Dialog role, Collections title, supporting description, and Got it action".into()],
                    ]);
                })),
            ],
        },
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn prose(content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn tooltip_points(items: Vec<&'static str>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in &items {
                prose(format!("•  {item}"));
                gap(Dp(12.0));
            }
        });
}

#[tessera]
fn tooltip_variant_examples() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(move || {
            for rich in [false, true] {
                surface()
                    .style(scheme.secondary_container.into())
                    .shape(Shape::rounded_rectangle(Dp(24.0)))
                    .modifier(Modifier::new().width(Dp(328.0)))
                    .child(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(move || {
                                sample(
                                    if rich {
                                        "Overview rich"
                                    } else {
                                        "Overview plain"
                                    },
                                    rich,
                                );
                                gap(Dp(16.0));
                                subheading(
                                    if rich {
                                        "2  Rich tooltip"
                                    } else {
                                        "1  Plain tooltip"
                                    }
                                    .into(),
                                );
                                gap(Dp(8.0));
                                prose(
                                    if rich {
                                        "More context, with an optional subhead and actions."
                                    } else {
                                        "A brief description of an element or action."
                                    }
                                    .into(),
                                );
                            });
                    });
            }
        });
}

#[tessera]
fn tooltip_writing_examples(rich: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    flow_row().item_spacing(Dp(24.0)).line_spacing(Dp(24.0)).children(move || {
        for recommended in [true, false] {
            column().modifier(Modifier::new().width(Dp(300.0))).children(move || {
                boxed().alignment(Alignment::Center).modifier(
                    Modifier::new().fill_max_width().height(Dp(176.0))
                        .background_with_shape(scheme.surface_container_low, Shape::rounded_rectangle(Dp(24.0)))
                        .semantics(SemanticsArgs {
                            role: Some(Role::Note),
                            label: Some(format!("{} {} wording example", if recommended { "Recommended" } else { "Not recommended" }, if rich { "rich tooltip subhead" } else { "plain tooltip" })),
                            merge_descendants: true, ..Default::default()
                        }),
                ).children(move || {
                    surface().style(if rich { scheme.surface_container } else { scheme.inverse_surface }.into())
                        .shape(Shape::rounded_rectangle(Dp(if rich { 12.0 } else { 4.0 })))
                        .modifier(Modifier::new().width(Dp(244.0)))
                        .child(move || {
                            column().modifier(Modifier::new().padding_all(Dp(if rich { 16.0 } else { 8.0 }))).children(move || {
                                text().content(if rich {
                                    if recommended { "New settings" } else { "New settings are now available for your uploaded images" }
                                } else if recommended { "Upload" } else { "Press this button to upload your images." })
                                    .style(type_style(if rich { 14.0 } else { 12.0 }, if rich { 20.0 } else { 16.0 }))
                                    .color(if rich { scheme.on_surface_variant } else { scheme.inverse_on_surface });
                                if rich {
                                    gap(Dp(4.0));
                                    text().content("Adjust image quality and storage space.").style(type_style(14.0, 20.0)).color(scheme.on_surface_variant);
                                }
                            });
                        });
                });
                gap(Dp(12.0));
                text().content(if recommended { "Do" } else { "Don't" }).style(type_style(16.0, 24.0)).color(if recommended { scheme.primary } else { scheme.error });
                gap(Dp(8.0));
                prose(if rich {
                    if recommended { "Summarize the message in a few words." } else { "Avoid a subhead that wraps across several lines." }
                } else if recommended { "Name the action directly." } else { "Avoid repeating instructions and unnecessary words." }.into());
            });
        }
    });
}

#[tessera]
fn tooltip_focus_sequence() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    flow_row()
        .item_spacing(Dp(12.0))
        .line_spacing(Dp(12.0))
        .children(move || {
            for label in [
                "1  About collections",
                "2  Got it (in popup)",
                "3  Next page control",
            ] {
                surface()
                    .style(scheme.secondary_container.into())
                    .shape(Shape::rounded_rectangle(Dp(16.0)))
                    .child(move || {
                        text()
                            .content(label)
                            .style(type_style(14.0, 20.0))
                            .modifier(Modifier::new().padding_all(Dp(16.0)));
                    });
            }
        });
}

#[tessera]
fn sample(#[prop(into)] name: String, rich: bool, controls: Option<bool>) {
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let calls = remember(|| 0_usize);
    let long = remember(|| false);
    let label = remember(|| name.clone());
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{name} tooltip preview")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if controls.unwrap_or(false) {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button().tonal().accessibility_label(format!("Show {}", label.get()))
                            .on_click(move || controller.with_mut(TooltipController::show))
                            .child(|| { text().content("Show"); });
                        button().text().accessibility_label(format!("Dismiss {}", label.get()))
                            .on_click(move || controller.with_mut(TooltipController::dismiss))
                            .child(|| { text().content("Dismiss"); });
                        button().tonal().selected(long.get())
                            .accessibility_label(format!("{} long message", label.get()))
                            .on_click(move || long.set(!long.get()))
                            .child(|| { text().content("Long message"); });
                    });
                gap(Dp(16.0));
            }
            let message = if long.get() {
                "Add this item to your collection so you can find it again later. You can change the collection at any time."
            } else if rich {
                "Add this item to your collection so you can find it again later."
            } else {
                "Add to favorites"
            };
            if rich {
                tooltip_provider(message)
                    .controller(controller)
                    .variant(TooltipVariant::Rich)
                    .headline("Rich tooltip")
                    .action_label("Learn more")
                    .on_action(move || calls.set(calls.get() + 1))
                    .placement(TooltipPlacement::Above)
                    .persistent(true)
                    .child(move || {
                        boxed()
                            .modifier(Modifier::new().fill_max_width().height(Dp(240.0)))
                            .children(move || {
                                tooltip_anchor()
                                    .controller(controller)
                                    .modifier(Modifier::new().align(Alignment::BottomCenter))
                                    .child(move || {
                                        button()
                                            .text()
                                            .accessibility_label(format!("{} anchor", label.get()))
                                            .on_click(move || controller.with_mut(TooltipController::show))
                                            .child(|| { text().content("More information"); });
                                    });
                            });
                    });
            } else {
                tooltip_provider(message)
                    .controller(controller)
                    .variant(TooltipVariant::Plain)
                    .placement(TooltipPlacement::Above)
                    .child(move || {
                        boxed()
                            .modifier(Modifier::new().fill_max_width().height(Dp(240.0)))
                            .children(move || {
                                tooltip_anchor()
                                    .controller(controller)
                                    .modifier(Modifier::new().align(Alignment::BottomCenter))
                                    .child(move || {
                                        icon_button()
                                            .standard()
                                            .icon(filled::INFO_SVG)
                                            .accessibility_label(format!("{} anchor", label.get()))
                                            .on_click(move || controller.with_mut(TooltipController::show));
                                    });
                            });
                    });
            }
            if controls.unwrap_or(false) {
                gap(Dp(16.0));
                text().content(format!("{} actions: {}", label.get(), calls.get()))
                    .style(type_style(14.0, 20.0));
                text().content(format!(
                    "{} visibility: {}",
                    label.get(),
                    if controller.with(TooltipController::is_open) { "Shown" } else { "Hidden" }
                ))
                .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn tokens() {
    let rich = remember(|| false);
    prose("Select a variant to inspect its color, typography, shape, and elevation roles. The measurements below also show the rendered component's padding.".into());
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for (label, value) in [("Plain tooltip", false), ("Rich tooltip", true)] {
                button()
                    .tonal()
                    .selected(rich.get() == value)
                    .accessibility_label(format!("Show {label} tokens"))
                    .on_click(move || rich.set(value))
                    .child(move || {
                        text().content(label);
                    });
            }
        });
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Token value".into()],
        if rich.get() {
            vec![
                vec!["Container color".into(), "Surface container".into()],
                vec![
                    "Subhead / supporting text color".into(),
                    "On surface variant".into(),
                ],
                vec!["Action color".into(), "Primary".into()],
                vec!["Subhead type".into(), "Title small · 14sp / 20sp".into()],
                vec![
                    "Supporting text type".into(),
                    "Body medium · 14sp / 20sp".into(),
                ],
                vec!["Action type".into(), "Label large · 14sp / 20sp".into()],
                vec!["Container corner size".into(), "12dp".into()],
                vec!["Container elevation".into(), "Level 2 · 3dp".into()],
            ]
        } else {
            vec![
                vec!["Container color".into(), "Inverse surface".into()],
                vec!["Supporting text color".into(), "Inverse on surface".into()],
                vec![
                    "Supporting text type".into(),
                    "Body small · 12sp / 16sp".into(),
                ],
                vec!["Container corner size".into(), "4dp".into()],
                vec!["Container elevation".into(), "Level 0 · 0dp".into()],
            ]
        },
    );
}

#[tessera]
fn detail(rich: bool) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            tooltip_figure(rich, false);
            gap(Dp(32.0));
            let title = if rich {
                "Rich tooltip"
            } else {
                "Plain tooltip"
            };
            panel(RenderSlot::new(move || {
                sample(
                    format!("Configured {}", if rich { "Rich" } else { "Plain" }),
                    rich,
                )
                .controls(true);
            }));
            gap(Dp(24.0));
            text()
        .content(if rich {
            "1  Optional subhead    2  Container    3  Supporting text    4  Optional text actions"
        } else {
            "1  Supporting text    2  Container"
        })
        .style(type_style(16.0, 24.0));
            gap(Dp(112.0));
            subheading(format!("{title} colors"));
            gap(Dp(24.0));
            tooltip_color_pair(rich);
            gap(Dp(88.0));
            subheading(format!("{title} measurements"));
            gap(Dp(24.0));
            tooltip_figure(rich, true);
            gap(Dp(24.0));
            spec_table(
                vec!["Attribute".into(), "Value".into()],
                if rich {
                    vec![
                        vec!["Top padding".into(), "12dp".into()],
                        vec!["Bottom padding without actions".into(), "16dp".into()],
                        vec!["Subhead to supporting text".into(), "4dp".into()],
                        vec!["Supporting text to actions".into(), "12dp".into()],
                        vec!["Bottom padding with actions".into(), "8dp".into()],
                        vec!["Horizontal padding".into(), "16dp".into()],
                        vec!["Maximum width".into(), "320dp".into()],
                    ]
                } else {
                    vec![
                        vec!["Single-line container height".into(), "24dp".into()],
                        vec!["Minimum width".into(), "40dp".into()],
                        vec!["Vertical padding".into(), "4dp".into()],
                        vec!["Horizontal padding".into(), "8dp".into()],
                        vec!["Maximum width".into(), "200dp".into()],
                    ]
                },
            );
            if rich {
                gap(Dp(112.0));
                subheading("Rich tooltip configurations".to_owned());
                gap(Dp(24.0));
                tooltip_configurations();
            }
        });
}

fn tooltip_color_labels(rich: bool) -> Vec<&'static str> {
    if rich {
        vec![
            "On surface variant",
            "Surface container",
            "On surface variant",
            "Primary",
        ]
    } else {
        vec!["Inverse on surface", "Inverse surface"]
    }
}

#[tessera]
fn tooltip_color_pair(rich: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!(
                        "{} tooltip color roles",
                        if rich { "Rich" } else { "Plain" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    for dark in [false, true] {
                        column()
                            .modifier(Modifier::new().weight(1.0).semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(format!(
                                    "{} {} tooltip color example",
                                    if dark { "Dark" } else { "Light" },
                                    if rich { "rich" } else { "plain" }
                                )),
                                merge_descendants: true,
                                ..Default::default()
                            }))
                            .children(move || {
                                material_theme()
                                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                                    .child(move || {
                                        tooltip_figure(rich, false).color_roles(true);
                                    });
                            });
                    }
                });
        });
    gap(Dp(24.0));
    prose(format!(
        "{} tooltip color roles used for light and dark themes:",
        if rich { "Rich" } else { "Plain" }
    ));
    gap(Dp(16.0));
    for (index, label) in tooltip_color_labels(rich).into_iter().enumerate() {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size(Dp(24.0), Dp(32.0))
                            .background_with_shape(scheme.on_surface, Shape::CAPSULE),
                    )
                    .children(move || {
                        text()
                            .content((index + 1).to_string())
                            .style(type_style(14.0, 20.0))
                            .color(scheme.surface);
                    });
                text()
                    .content(label)
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp(4.0)));
            });
        gap(Dp(8.0));
    }
}

#[tessera]
fn tooltip_configurations() {
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(tooltip_configuration_canvas);
    gap(Dp(24.0));
    for (index, label) in [
        "Subhead, supporting text, and two buttons",
        "Subhead, supporting text, and one button",
        "Subhead and supporting text",
        "Supporting text and one button",
        "Supporting text and two buttons",
    ]
    .into_iter()
    .enumerate()
    {
        prose(format!("{}  {label}", index + 1));
        gap(Dp(8.0));
    }
}

#[tessera]
fn tooltip_configuration_canvas() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|rect| Dp::from_pixels_f32(rect.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let scale = width / 1024.0;
    let height = width * 1394.0 / 2048.0;
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(height))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some("Rich tooltip five configurations".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            tooltip_grid(Dp(width), Dp(height));
            for (number, headline, actions, x, y) in [
                (1, true, 2, 122.0, 117.0),
                (2, true, 1, 122.0, 349.0),
                (3, true, 0, 590.0, 117.0),
                (4, false, 1, 590.0, 286.0),
                (5, false, 2, 590.0, 488.0),
            ] {
                tooltip_configuration(number, headline, actions, Dp(x * scale), Dp(y * scale));
            }
        });
}

#[tessera]
fn tooltip_configuration(number: usize, headline: bool, actions: usize, x: Dp, y: Dp) {
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let calls = remember(|| 0_usize);
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let popup_height = geometry
        .with(|values| {
            values
                .get("popup")
                .map(|rect| Dp::from_pixels_f32(rect.height.to_f32()).0)
        })
        .unwrap_or(if actions == 0 {
            112.0
        } else if headline {
            156.0
        } else {
            132.0
        });
    boxed().modifier(Modifier::new().offset(Dp(x.0-8.0),Dp(y.0-48.0)).required_size(Dp(328.0),Dp(popup_height+56.0)).semantics(SemanticsArgs {
        role:Some(Role::Group),label:Some(format!("Rich tooltip configuration {number}")),merge_descendants:true,..Default::default()
    })).children(move || {
        provide_context(||MinimumInteractiveComponentEnforcement {enabled:false},move || {
            if headline && actions > 0 {
                tooltip_provider("Rich tooltips bring attention to a particular element or feature that warrants a person’s focus.")
                    .controller(controller).variant(TooltipVariant::Rich).headline("Rich tooltip")
                    .actions(move || {
                        row().children(move || {
                            for action in 1..=actions {
                                button().text().accessibility_label(format!("Tooltip configuration {number} action {action}")).on_click(move || {
                                    calls.set(calls.get()+1);controller.with_mut(TooltipController::dismiss);
                                }).child(|| {text().content("Action");});
                            }
                        });
                    })
                    .placement(TooltipPlacement::Below).persistent(true).popup_modifier(watch(geometry,"popup").width(Dp(312.0)))
                    .child(move || {
                        boxed().modifier(Modifier::new().required_size(Dp(328.0),Dp(popup_height+56.0))).children(move || {
                            tooltip_anchor().controller(controller).modifier(Modifier::new().offset(Dp(8.0),Dp(8.0)).size(Dp(312.0),Dp(36.0))).child(move || {
                                boxed().modifier(Modifier::new().size(Dp(312.0),Dp(36.0))).children(move || {
                                    surface().style(scheme.surface_container_high.into()).shape(Shape::rounded_rectangle(Dp(12.0))).modifier(Modifier::new().required_size(Dp(24.0),Dp(24.0)).border_with_shape(Dp(1.0),scheme.on_surface,Shape::rounded_rectangle(Dp(12.0))))
                                        .accessibility_label(format!("Show rich tooltip configuration {number}")).on_click(move ||controller.with_mut(TooltipController::show)).child(move || {text().content(number.to_string()).style(type_style(14.0,20.0));});
                                });
                            });
                            if calls.get()>0 && !controller.with(TooltipController::is_open) {
                                text().content(format!("Tooltip configuration {number} actions: {}",calls.get())).style(type_style(14.0,20.0)).modifier(Modifier::new().offset(Dp(8.0),Dp(48.0)));
                            }
                        });
                    });
            } else if headline {
                tooltip_provider("Rich tooltips bring attention to a particular element or feature that warrants a person’s focus.")
                    .controller(controller).variant(TooltipVariant::Rich).headline("Rich tooltip")
                    .placement(TooltipPlacement::Below).persistent(false).popup_modifier(watch(geometry,"popup").width(Dp(312.0)))
                    .child(move || {
                        boxed().modifier(Modifier::new().required_size(Dp(328.0),Dp(popup_height+56.0))).children(move || {
                            tooltip_anchor().controller(controller).modifier(Modifier::new().offset(Dp(8.0),Dp(8.0)).size(Dp(312.0),Dp(36.0))).child(move || {
                                boxed().modifier(Modifier::new().size(Dp(312.0),Dp(36.0))).children(move || {
                                    surface().style(scheme.surface_container_high.into()).shape(Shape::rounded_rectangle(Dp(12.0))).modifier(Modifier::new().required_size(Dp(24.0),Dp(24.0)).border_with_shape(Dp(1.0),scheme.on_surface,Shape::rounded_rectangle(Dp(12.0))))
                                        .accessibility_label(format!("Show rich tooltip configuration {number}")).on_click(move ||controller.with_mut(TooltipController::show)).child(move || {text().content(number.to_string()).style(type_style(14.0,20.0));});
                                });
                            });
                            if calls.get()>0 && !controller.with(TooltipController::is_open) {
                                text().content(format!("Tooltip configuration {number} actions: {}",calls.get())).style(type_style(14.0,20.0)).modifier(Modifier::new().offset(Dp(8.0),Dp(48.0)));
                            }
                        });
                    });
            } else if actions > 0 {
                tooltip_provider("Rich tooltips bring attention to a particular element or feature that warrants a person’s focus.")
                    .controller(controller).variant(TooltipVariant::Rich)
                    .actions(move || {
                        row().children(move || {
                            for action in 1..=actions {
                                button().text().accessibility_label(format!("Tooltip configuration {number} action {action}")).on_click(move || {
                                    calls.set(calls.get()+1);controller.with_mut(TooltipController::dismiss);
                                }).child(|| {text().content("Action");});
                            }
                        });
                    })
                    .placement(TooltipPlacement::Below).persistent(true).popup_modifier(watch(geometry,"popup").width(Dp(312.0)))
                    .child(move || {
                        boxed().modifier(Modifier::new().required_size(Dp(328.0),Dp(popup_height+56.0))).children(move || {
                            tooltip_anchor().controller(controller).modifier(Modifier::new().offset(Dp(8.0),Dp(8.0)).size(Dp(312.0),Dp(36.0))).child(move || {
                                boxed().modifier(Modifier::new().size(Dp(312.0),Dp(36.0))).children(move || {
                                    surface().style(scheme.surface_container_high.into()).shape(Shape::rounded_rectangle(Dp(12.0))).modifier(Modifier::new().required_size(Dp(24.0),Dp(24.0)).border_with_shape(Dp(1.0),scheme.on_surface,Shape::rounded_rectangle(Dp(12.0))))
                                        .accessibility_label(format!("Show rich tooltip configuration {number}")).on_click(move ||controller.with_mut(TooltipController::show)).child(move || {text().content(number.to_string()).style(type_style(14.0,20.0));});
                                });
                            });
                            if calls.get()>0 && !controller.with(TooltipController::is_open) {
                                text().content(format!("Tooltip configuration {number} actions: {}",calls.get())).style(type_style(14.0,20.0)).modifier(Modifier::new().offset(Dp(8.0),Dp(48.0)));
                            }
                        });
                    });
            } else {
                tooltip_provider("Rich tooltips bring attention to a particular element or feature that warrants a person’s focus.")
                    .controller(controller).variant(TooltipVariant::Rich)
                    .placement(TooltipPlacement::Below).persistent(false).popup_modifier(watch(geometry,"popup").width(Dp(312.0)))
                    .child(move || {
                        boxed().modifier(Modifier::new().required_size(Dp(328.0),Dp(popup_height+56.0))).children(move || {
                            tooltip_anchor().controller(controller).modifier(Modifier::new().offset(Dp(8.0),Dp(8.0)).size(Dp(312.0),Dp(36.0))).child(move || {
                                boxed().modifier(Modifier::new().size(Dp(312.0),Dp(36.0))).children(move || {
                                    surface().style(scheme.surface_container_high.into()).shape(Shape::rounded_rectangle(Dp(12.0))).modifier(Modifier::new().required_size(Dp(24.0),Dp(24.0)).border_with_shape(Dp(1.0),scheme.on_surface,Shape::rounded_rectangle(Dp(12.0))))
                                        .accessibility_label(format!("Show rich tooltip configuration {number}")).on_click(move ||controller.with_mut(TooltipController::show)).child(move || {text().content(number.to_string()).style(type_style(14.0,20.0));});
                                });
                            });
                            if calls.get()>0 && !controller.with(TooltipController::is_open) {
                                text().content(format!("Tooltip configuration {number} actions: {}",calls.get())).style(type_style(14.0,20.0)).modifier(Modifier::new().offset(Dp(8.0),Dp(48.0)));
                            }
                        });
                    });
            }
        });
    });
}

#[tessera]
fn tooltip_figure(rich: bool, measurements: bool, color_roles: Option<bool>) {
    let color_roles = color_roles.unwrap_or(false);
    let geometry = remember(Geometry::new);
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let actions = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let figure_width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|rect| Dp::from_pixels_f32(rect.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let figure_height = if color_roles {
        figure_width * 2.0 / (2048.0 / if rich { 676.0 } else { 600.0 })
    } else {
        figure_width / (2048.0 / 600.0)
    };
    let popup_height = geometry
        .with(|values| {
            values
                .get("popup")
                .map(|rect| Dp::from_pixels_f32(rect.height.to_f32()).0)
        })
        .unwrap_or(if rich { 156.0 } else { 24.0 });
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(figure_height))
                .background_with_shape(
                    if color_roles {scheme.surface_container_lowest} else {scheme.surface_container_low},
                    Shape::rounded_rectangle(if color_roles {Dp::ZERO} else {Dp(24.0)}),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!(
                        "{} tooltip {} figure",
                        if rich { "Rich" } else { "Plain" },
                        if color_roles {
                            "color"
                        } else if measurements {
                            "measurement"
                        } else {
                            "anatomy"
                        }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            tooltip_grid(Dp(figure_width),Dp(figure_height));
            provide_context(|| MinimumInteractiveComponentEnforcement { enabled: false }, move || {
            let message = if rich {
                "Rich tooltips bring attention to a particular element or feature that warrants a person's focus."
            } else {
                "Plain tooltip"
            };
            let popup_modifier = if rich { watch(geometry, "popup").width(Dp(312.0)) } else { watch(geometry, "popup") };
            if rich {
                tooltip_provider(message)
                    .controller(controller)
                    .variant(TooltipVariant::Rich)
                    .popup_modifier(popup_modifier)
                    .headline("Rich tooltip")
                    .actions(move || {
                        button()
                            .text()
                            .modifier(watch(geometry, "action"))
                            .accessibility_label("Action")
                            .on_click(move || {
                                controller.with_mut(TooltipController::dismiss);
                                actions.set(actions.get() + 1);
                            })
                            .child(|| {
                                let typography = use_context::<MaterialTheme>()
                                    .unwrap()
                                    .get()
                                    .typography;
                                text().content("Action").style(typography.label_large);
                            });
                    })
                    .placement(TooltipPlacement::Above)
                    .child(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        tooltip_anchor()
                            .controller(controller)
                            .modifier(
                                watch(geometry, "anchor")
                                    .align(Alignment::TopCenter)
                                    .offset(Dp(0.0), Dp((figure_height + popup_height) * 0.5 + 4.0)),
                            )
                            .child(|| { spacer().modifier(Modifier::new().size(Dp(1.0), Dp(1.0))); });
                    });
            });
            } else {
                tooltip_provider(message)
                    .controller(controller)
                    .variant(TooltipVariant::Plain)
                    .popup_modifier(popup_modifier)
                    .placement(TooltipPlacement::Above)
                    .child(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        tooltip_anchor()
                            .controller(controller)
                            .modifier(
                                watch(geometry, "anchor")
                                    .align(Alignment::TopCenter)
                                    .offset(Dp(0.0), Dp((figure_height + popup_height) * 0.5 + 4.0)),
                            )
                            .child(|| { spacer().modifier(Modifier::new().size(Dp(1.0), Dp(1.0))); });
                    });
            });
            }
            });
            tooltip_annotations(geometry, rich, measurements).color_roles(color_roles);
        });
    if !color_roles {
        gap(Dp(16.0));
        prose(if measurements {
            format!(
                "{} tooltip padding and size measurements",
                if rich { "Rich" } else { "Plain" }
            )
        } else if rich {
            "1  Subhead\n2  Container\n3  Supporting text\n4  Text button".into()
        } else {
            "1  Supporting text\n2  Container".into()
        });
    }
    if actions.get() > 0 {
        text()
            .content(format!("Tooltip figure actions: {}", actions.get()))
            .style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn tooltip_grid(width: Dp, height: Dp) {
    let mut path = String::new();
    for row in 1..(height.0 / 8.0).ceil() as usize {
        for column in 1..(width.0 / 8.0).ceil() as usize {
            let x = column as f64 * 8.0 + 2.0;
            let y = row as f64 * 8.0 + 2.0;
            path.push_str(&format!("M{x},{y}h0.6v0.6h-0.6z"));
        }
    }
    annotations::lines(
        width,
        height,
        format!("<path d='{path}' fill='#79747e' fill-opacity='.6'/>"),
    );
}

#[tessera]
fn tooltip_annotations(
    geometry: State<Geometry>,
    rich: bool,
    measurements: bool,
    color_roles: Option<bool>,
) {
    let values = geometry.with(annotations::relative_geometry);
    tooltip_annotation_content(values, rich, measurements, color_roles.unwrap_or(false));
}

#[tessera]
fn tooltip_annotation_content(values: Geometry, rich: bool, measurements: bool, color_roles: bool) {
    let (Some(root), Some(popup), Some(anchor)) = (
        values.get("figure"),
        relative(&values, "popup"),
        relative(&values, "anchor"),
    ) else {
        return;
    };
    let inset = if rich { 16.0 } else { 8.0 };
    let top = if rich { 12.0 } else { 4.0 };
    let bottom = if rich { 8.0 } else { 4.0 };
    let mut svg = String::new();
    if color_roles {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        let ink = if scheme.is_dark { "#cac4d0" } else { "#49454f" };
        let labels = tooltip_color_labels(rich);
        let regions = if rich {
            let headline = Rect {
                x: popup.x + 16.0,
                y: popup.y + 12.0,
                w: 80.0,
                h: 20.0,
            };
            let Some(action) = relative(&values, "action") else {
                return;
            };
            vec![
                (1, headline, headline.x + 40.0, true),
                (2, popup, popup.x + popup.w - 88.0, true),
                (
                    3,
                    Rect {
                        x: headline.x,
                        y: headline.y + 24.0,
                        w: popup.w - 32.0,
                        h: (action.y - headline.y - 36.0).max(0.0),
                    },
                    popup.x + popup.w - 64.0,
                    false,
                ),
                (4, action, action.x + 20.0, false),
            ]
        } else {
            vec![
                (
                    1,
                    Rect {
                        x: popup.x + 8.0,
                        y: popup.y + 4.0,
                        w: popup.w - 16.0,
                        h: popup.h - 8.0,
                    },
                    popup.x + 20.0,
                    false,
                ),
                (2, popup, popup.x + popup.w - 12.0, true),
            ]
        };
        for (number, rect, x, above) in regions {
            let end = if above { rect.y } else { rect.y + rect.h };
            if rich && number == 4 {
                let marker_x = popup.x - 40.0;
                let marker_y = rect.y + rect.h * 0.5 - 16.0;
                dotted(
                    &mut svg,
                    marker_x + 24.0,
                    marker_y + 16.0,
                    rect.x - marker_x - 24.0,
                    false,
                    ink,
                );
                tooltip_color_marker(
                    number,
                    labels[number - 1].into(),
                    Dp(marker_x),
                    Dp(marker_y),
                );
                dotted(&mut svg, rect.x, rect.y, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y + rect.h, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y, rect.h, true, ink);
                dotted(&mut svg, rect.x + rect.w, rect.y, rect.h, true, ink);
                continue;
            }
            let y = if above {
                popup.y
                    - if rich && number == 1 {
                        36.0
                    } else if rich {
                        56.0
                    } else {
                        52.0
                    }
            } else if rich {
                popup.y + popup.h - 24.0
            } else {
                popup.y + popup.h + 36.0
            };
            let start = if above { y + 32.0 } else { end };
            let length = if above { end - start } else { y - end };
            dotted(&mut svg, x, start, length, true, ink);
            if number != 2 {
                dotted(&mut svg, rect.x, rect.y, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y + rect.h, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y, rect.h, true, ink);
                dotted(&mut svg, rect.x + rect.w, rect.y, rect.h, true, ink);
            }
            tooltip_color_marker(number, labels[number - 1].into(), Dp(x - 12.0), Dp(y));
        }
    } else if measurements {
        for rect in [
            Rect {
                x: popup.x,
                y: popup.y,
                w: inset,
                h: popup.h,
            },
            Rect {
                x: popup.x + popup.w - inset,
                y: popup.y,
                w: inset,
                h: popup.h,
            },
            Rect {
                x: popup.x,
                y: popup.y,
                w: popup.w,
                h: top,
            },
            Rect {
                x: popup.x,
                y: popup.y + popup.h - bottom,
                w: popup.w,
                h: bottom,
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#e883aa' fill-opacity='.32'/>",
                rect.x, rect.y, rect.w, rect.h
            ));
        }
        for (label, x, y) in [
            (format!("{top:.0}"), popup.x - 32.0, popup.y),
            (
                format!("{bottom:.0}"),
                popup.x + popup.w + 16.0,
                popup.y + popup.h - bottom,
            ),
            (format!("{inset:.0}"), popup.x, popup.y + popup.h + 16.0),
            (
                format!("{inset:.0}"),
                popup.x + popup.w - inset,
                popup.y + popup.h + 16.0,
            ),
        ] {
            annotations::dimension(label, Dp(x), Dp(y));
        }
        dotted(
            &mut svg,
            popup.x + popup.w + 8.0,
            popup.y,
            popup.h,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{:.0}", popup.h),
            Dp(popup.x + popup.w + 16.0),
            Dp(popup.y + popup.h * 0.5 - 10.0),
        );
        let gap = (anchor.y - popup.y - popup.h).max(0.0);
        dotted(
            &mut svg,
            anchor.x - 16.0,
            popup.y + popup.h,
            gap,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{gap:.0}"),
            Dp(anchor.x - 36.0),
            Dp(popup.y + popup.h),
        );
    } else if rich {
        let headline = Rect {
            x: popup.x + 16.0,
            y: popup.y + 12.0,
            w: popup.w - 32.0,
            h: 20.0,
        };
        let actions = Rect {
            x: popup.x + 16.0,
            y: popup.y + popup.h - 48.0,
            w: popup.w - 32.0,
            h: 40.0,
        };
        let body = Rect {
            x: headline.x,
            y: headline.y + headline.h + 4.0,
            w: headline.w,
            h: (actions.y - 12.0 - headline.y - headline.h - 4.0).max(0.0),
        };
        for (number, rect, x, above) in [
            (1, headline, headline.x + 8.0, true),
            (2, popup, popup.x + popup.w - 12.0, true),
            (3, body, body.x + body.w - 16.0, false),
            (4, actions, actions.x + 20.0, false),
        ] {
            let end = if above { rect.y } else { rect.y + rect.h };
            let start = if above { popup.y - 40.0 } else { end };
            let length = if above {
                end - start
            } else {
                popup.y + popup.h + 40.0 - end
            };
            dotted(&mut svg, x, start, length, true, "#49454f");
            annotations::marker(
                number,
                Dp(x - 12.0),
                Dp(if above { start - 32.0 } else { start + length }),
            );
            annotations::outline(&mut svg, rect);
        }
    } else {
        let text = Rect {
            x: popup.x + 8.0,
            y: popup.y + 4.0,
            w: popup.w - 16.0,
            h: popup.h - 8.0,
        };
        let text_x = text.x + 8.0;
        dotted(&mut svg, text_x, text.y + text.h, 36.0, true, "#49454f");
        annotations::marker(1, Dp(text_x - 12.0), Dp(text.y + text.h + 36.0));
        let container_x = popup.x + popup.w - 8.0;
        dotted(&mut svg, container_x, popup.y - 36.0, 36.0, true, "#49454f");
        annotations::marker(2, Dp(container_x - 12.0), Dp(popup.y - 68.0));
        annotations::outline(&mut svg, text);
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn tooltip_color_marker(number: usize, label: String, x: Dp, y: Dp) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.on_surface, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Tooltip color role {number}: {label}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn live_tooltips() {
    let plain = remember(TooltipController::new);
    let rich = remember(TooltipController::new);
    let favorite = remember(|| false);
    let actions = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    prose("Hover or focus the favorite icon. Select About collections to open a persistent explanation, then try Got it, outside press, or Escape.".into());
    gap(Dp(16.0));
    tooltip_provider(if favorite.get() {
        "Remove from favorites"
    } else {
        "Add to favorites"
    })
    .controller(plain)
    .persistent(false)
    .child(move || {
        tooltip_provider("Keep related items together so you can find them again.")
            .controller(rich)
            .variant(TooltipVariant::Rich)
            .headline("Collections")
            .placement(TooltipPlacement::Below)
            .persistent(true)
            .action_label("Got it")
            .on_action(move || actions.set(actions.get() + 1))
            .child(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(280.0))
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::rounded_rectangle(Dp(24.0)),
                            ),
                    )
                    .children(move || {
                        flow_row()
                            .item_spacing(Dp(24.0))
                            .line_spacing(Dp(16.0))
                            .children(move || {
                                tooltip_anchor().controller(plain).child(move || {
                                    icon_button()
                                        .standard()
                                        .selected(favorite.get())
                                        .icon(if favorite.get() {
                                            filled::FAVORITE_SVG
                                        } else {
                                            filled::FAVORITE_BORDER_SVG
                                        })
                                        .accessibility_label(if favorite.get() {
                                            "Remove from favorites"
                                        } else {
                                            "Add to favorites"
                                        })
                                        .on_click(move || favorite.set(!favorite.get()));
                                });
                                tooltip_anchor().controller(rich).child(move || {
                                    button()
                                        .text()
                                        .accessibility_label("About collections")
                                        .on_click(move || rich.with_mut(TooltipController::show))
                                        .child(|| {
                                            text().content("About collections");
                                        });
                                });
                            });
                    });
            });
    });
    gap(Dp(16.0));
    text()
        .content(format!(
            "Favorite: {}; tooltip actions: {}; rich tooltip: {}",
            if favorite.get() { "on" } else { "off" },
            actions.get(),
            if rich.with(TooltipController::is_open) {
                "shown"
            } else {
                "hidden"
            }
        ))
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn placement_examples() {
    flow_row()
        .item_spacing(Dp(16.0))
        .line_spacing(Dp(16.0))
        .children(|| {
            for (alignment, placement) in [
                (Alignment::TopStart, TooltipPlacement::Above),
                (Alignment::TopEnd, TooltipPlacement::Above),
                (Alignment::BottomStart, TooltipPlacement::Below),
                (Alignment::BottomEnd, TooltipPlacement::Below),
            ] {
                let controller = remember(|| {
                    let mut controller = TooltipController::new();
                    controller.show();
                    controller
                });
                tooltip_provider("Tooltip stays within the available space.")
                    .controller(controller)
                    .variant(TooltipVariant::Rich)
                    .headline("Dynamic placement")
                    .placement(placement)
                    .persistent(false)
                    .child(move || {
                        boxed()
                            .modifier(Modifier::new().size(Dp(264.0), Dp(220.0)))
                            .children(move || {
                                tooltip_anchor()
                                    .controller(controller)
                                    .modifier(Modifier::new().align(alignment))
                                    .child(move || {
                                        button()
                                            .text()
                                            .accessibility_label(format!(
                                                "Corner tooltip {alignment:?}"
                                            ))
                                            .on_click(move || {
                                                controller.with_mut(TooltipController::show)
                                            })
                                            .child(|| {
                                                text().content("Details");
                                            });
                                    });
                            });
                    });
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "tooltips_tests.rs"]
mod tests;
