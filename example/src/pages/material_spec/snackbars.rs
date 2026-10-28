//! Snackbar specification figures and queued notifications.
//!
//! ## Usage
//!
//! Show brief process updates with optional undo and dismiss actions.

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "snackbars_tests.rs"]
mod regression_tests;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    snackbar::{
        SnackbarData, SnackbarDuration, SnackbarHostState, SnackbarRequest, snackbar, snackbar_host,
    },
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn snackbars_spec() {
    article(
        "Snackbar",
        "Show brief updates about app processes.",
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
        ],
    )
    .intro(anatomy)
    .page_heading_levels(std::collections::BTreeMap::from([("Accessibility", 3)]))
    .pages(vec![
        ArticlePage{title:"Overview",intro:Some(RenderSlot::new(|| {
            snackbar_points(vec![
                "Snackbars inform people about a process the app has performed or will perform.",
                "They appear near the bottom of the screen and do not interrupt the current task.",
                "Only one snackbar is visible at a time; it can offer a single optional action.",
                "A snackbar without actions can dismiss automatically, while an actionable one stays until used or dismissed.",
            ]);
            gap(Dp(24.0)); snackbar_figure(false,true,true,false);
        })),sections:vec![SpecSection{title:"Differences from M2",content:RenderSlot::new(|| {
            prose("Material 3 keeps the compact, elevated notification but maps its colors through inverse theme roles. Rich guidance such as inline feedback and persistent actions now also applies to web.");
            gap(Dp(24.0));
            spec_table(vec!["Element".into(),"Material 3 guidance".into()],vec![
                vec!["Container".into(),"Fully opaque inverse surface with level 3 elevation".into()],
                vec!["Text and action".into(),"Inverse on surface and inverse primary for clear contrast".into()],
                vec!["Duration".into(),"Message-only snackbars may dismiss after 4–10 seconds; actionable snackbars persist".into()],
                vec!["Frequency".into(),"One at a time; never stack or place side by side".into()],
            ]);
            gap(Dp(24.0)); snackbar_figure(false,true,true,false);
        })}]},
        ArticlePage{title:"Guidelines",intro:Some(RenderSlot::new(|| {
            snackbar_points(vec![
                "Use a snackbar for a low-priority process update, such as a save, archive, or retry.",
                "Keep the message short and self-contained; people should understand it without leaving the page.",
                "Snackbars never replace dialogs for decisions that block progress or require a response.",
            ]);
            gap(Dp(24.0)); snackbar_figure(false,true,false,false);
        })),sections:vec![
            SpecSection{title:"Usage",content:RenderSlot::new(snackbar_usage)},
            SpecSection{title:"Anatomy",content:RenderSlot::new(snackbar_guideline_anatomy)},
            SpecSection{title:"Placement",content:RenderSlot::new(|| {subheading("At the bottom of a UI".into());prose("Place the snackbar in front of main content near the bottom, clear of navigation, floating actions and focused controls.");gap(Dp(24.0));snackbar_placement_cautions();gap(Dp(32.0));prose("In wide layouts, choose consistent leading or center alignment and keep the container away from the screen edges.");gap(Dp(24.0));snackbar_placement(false);})},
            SpecSection{title:"Responsive layout",content:RenderSlot::new(snackbar_responsive)},
            SpecSection{title:"Behavior",content:RenderSlot::new(|| {subheading("Appearing and disappearing".into());prose("Do not block page interaction. Actionable snackbars remain until acted on; temporary web notifications also need nearby inline feedback.");gap(Dp(24.0));snackbar_behavior_table();gap(Dp(24.0));inline_feedback();gap(Dp(32.0));subheading("Consecutive snackbars".into());prose("Show queued notifications one at a time. Do not stack them or place them side by side.");gap(Dp(24.0));queued();})},
        ]},
        ArticlePage{title:"Accessibility",intro:None,sections:vec![
            SpecSection{title:"Use cases",content:RenderSlot::new(|| {snackbar_points(vec![
                "Be alerted, but not disrupted, when a process update appears.",
                "Move focus to an actionable snackbar and read it at a comfortable pace.",
                "Activate the snackbar action or dismiss control using assistive technology.",
            ]);gap(Dp(24.0));snackbar_figure(false,true,true,false);})},
            SpecSection{title:"Interaction & style",content:RenderSlot::new(|| {
                prose("Snackbars with actions do not auto-dismiss, so people can read and respond at their own pace. Messages without actions may dismiss after 4–10 seconds; on web, pair that timing with inline feedback.");
                gap(Dp(24.0));
                snackbar_points(vec![
                    "Use the default inverse color mapping so the notification stands out without creating a color conflict.",
                    "Keep the message and actions legible in both light and dark themes.",
                    "Do not make a snackbar the only way to reach a core use case.",
                ]);
                gap(Dp(24.0)); inline_feedback();
            })},
            SpecSection{title:"Accessibility requirements on web",content:RenderSlot::new(snackbar_web_feedback)},
            SpecSection{title:"Focus",content:RenderSlot::new(|| {
                prose("Announce the message without moving focus away from the current control by default. Snackbar actions participate in ordinary focus traversal and must not trap focus.");
                gap(Dp(24.0));
                spec_table(vec!["Focus step".into(),"Expected behavior".into()],vec![
                    vec!["Page control".into(),"Keeps focus while a message-only snackbar is announced".into()],
                    vec!["Snackbar action".into(),"Reachable with Tab; the focused action stays visible".into()],
                    vec!["After the action".into(),"Focus returns to, or continues from, the triggering control".into()],
                ]);
            })},
            SpecSection{title:"Keyboard navigation",content:RenderSlot::new(|| {spec_table(vec!["Key".into(),"Implemented action".into()],vec![vec!["Tab".into(),"Move between page controls and snackbar actions".into()],vec!["Shift + Tab".into(),"Move to the previous interactive element".into()],vec!["Enter or Space".into(),"Activate the focused action or dismiss control".into()]]);gap(Dp(16.0));prose("A message-only snackbar adds no Tab stop. Do not obscure the element that currently has focus.");})},
            SpecSection{title:"Labeling elements",content:RenderSlot::new(|| {prose("Publish the supporting text as a polite status update. Action labels describe the outcome, such as Undo; the optional close button is named Dismiss.");gap(Dp(24.0));spec_table(vec!["Element".into(),"Accessible information in this example".into()],vec![
                vec!["Supporting text".into(),"Status role with the complete message; announcements do not repeat".into()],
                vec!["Undo".into(),"Named text button stating the action".into()],
                vec!["Dismiss".into(),"Close button named for its purpose, not its icon".into()],
            ]);})},
        ]},
    ]);
}

#[tessera]
fn sample(
    #[prop(into)] name: String,
    two_lines: Option<bool>,
    action: Option<bool>,
    long_action: Option<bool>,
    dismiss: Option<bool>,
    stacked: Option<bool>,
) {
    let visible = remember(|| true);
    let label = remember(|| name.clone());
    boxed()
        .modifier(Modifier::new().width(Dp(360.0)).semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(name),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if visible.get() {
                let mut bar = snackbar()
                    .modifier(Modifier::new().fill_max_width())
                    .message(if two_lines.unwrap_or(false) {
                        "Your photo was saved.\nIt is ready to share."
                    } else {
                        "Photo saved"
                    })
                    .action_on_new_line(stacked.unwrap_or(false));
                if action.unwrap_or(false) {
                    bar = bar
                        .action_label(if long_action.unwrap_or(false) {
                            "Open photo collection"
                        } else {
                            "Undo"
                        })
                        .on_action(move || visible.set(false));
                }
                if dismiss.unwrap_or(false) {
                    bar = bar
                        .with_dismiss_action(true)
                        .dismiss_action_label(format!("Dismiss {}", label.get()))
                        .on_dismiss(move || visible.set(false));
                }
                drop(bar);
            } else {
                button()
                    .text()
                    .accessibility_label(format!("Restore {}", label.get()))
                    .on_click(move || visible.set(true))
                    .child(|| {
                        text().content("Show snackbar");
                    });
            }
        });
}

#[tessera]
fn anatomy() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            panel(RenderSlot::new(|| {
                sample("Snackbar anatomy").action(true).dismiss(true);
            }));
            gap(Dp(24.0));
            for label in [
                "1  Container",
                "2  Icon (optional close affordance)",
                "3  Action (optional)",
                "4  Supporting text",
            ] {
                text().content(label).style(type_style(16.0, 24.0));
                gap(Dp(16.0));
            }
        });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Inverse surface".into()],
            vec![
                "Supporting text and close icon".into(),
                "Inverse on surface".into(),
            ],
            vec!["Action".into(), "Inverse primary".into()],
            vec!["Supporting text style".into(), "Body medium".into()],
            vec!["Shape".into(), "Extra small (4dp)".into()],
            vec!["Elevation".into(), "Level 3 (6dp)".into()],
        ],
    );
}

#[tessera]
fn colors() {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            for dark in [false, true] {
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        subheading(if dark { "Dark theme" } else { "Light theme" }.into());
                        gap(Dp(16.0));
                        material_theme()
                            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                            .child(move || {
                                panel(RenderSlot::new(move || {
                                    sample(if dark {
                                        "Dark snackbar"
                                    } else {
                                        "Light snackbar"
                                    })
                                    .action(true)
                                    .dismiss(true);
                                }));
                            });
                    });
            }
        });
    gap(Dp(24.0));
    text().content("1  Inverse surface    2  Inverse on surface    3  Inverse primary    4  Inverse on surface").style(type_style(16.0,24.0));
    gap(Dp(8.0));
    prose(
        "Both themes use the same inverse roles. The container stays fully opaque and elevated so the message remains legible over content.",
    );
}

#[tessera]
fn measurements() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            panel(RenderSlot::new(|| {
                sample("Snackbar measurements").action(true);
            }));
            gap(Dp(24.0));
            spec_table(
                vec!["Attribute".into(), "Value".into()],
                vec![
                    vec!["Official single-line minimum".into(), "48dp".into()],
                    vec!["Official two-line minimum".into(), "64dp".into()],
                    vec!["Current Tessera two-line minimum".into(), "68dp (native reference)".into()],
                    vec!["Message horizontal padding".into(), "16dp".into()],
                    vec!["Message vertical padding".into(), "14dp".into()],
                    vec!["Action spacing".into(), "8dp".into()],
                    vec!["Maximum width".into(), "600dp".into()],
                ],
            );
            prose("Official Material 3 reference: 48dp expands to 64dp for two lines at compact widths. The current Tessera native snackbar reserves 68dp for a two-line message so 14dp vertical padding plus two 20dp lines fit without clipping. This example states both values rather than presenting the native height as the official specification.");
        });
}

#[tessera]
fn snackbar_points(items: Vec<&'static str>) {
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
fn snackbar_writing_examples() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    flow_row().item_spacing(Dp(24.0)).line_spacing(Dp(24.0)).children(move || {
        for recommended in [true,false] {
            column().modifier(Modifier::new().width(Dp(360.0))).children(move || {
                boxed().alignment(Alignment::Center).modifier(
                    Modifier::new().fill_max_width().height(Dp(184.0))
                        .background_with_shape(scheme.surface_container_low, Shape::rounded_rectangle(Dp(24.0)))
                        .semantics(SemanticsArgs {role:Some(Role::Note),label:Some(format!("{} snackbar text label",if recommended{"Recommended"}else{"Not recommended"})),merge_descendants:true,..Default::default()}),
                ).children(move || {
                    if recommended {
                        snackbar().modifier(Modifier::new().width(Dp(320.0))).message("Photo archived");
                    } else {
                        snackbar().modifier(Modifier::new().width(Dp(320.0))).message("Your photo has been archived. Open the archive collection to find it again later.");
                    }
                });
                gap(Dp(12.0));
                text().content(if recommended {"Do"} else {"Don't"}).style(type_style(16.0,24.0)).color(if recommended{scheme.primary}else{scheme.error});
                gap(Dp(8.0));
                let guidance = if recommended {
                    "Keep the update to one short line where possible."
                } else {
                    "Avoid long instructions, inline links, icons and styled text in the label."
                };
                prose(guidance);
            });
        }
    });
}

#[tessera]
fn snackbar_placement_cautions() {
    spec_table(
        vec!["Placement".into(), "Guidance".into()],
        vec![
            vec![
                "Main content".into(),
                "In front of page content near the bottom, clear of frequently used controls"
                    .into(),
            ],
            vec![
                "Navigation and FABs".into(),
                "Never cover navigation; appear above a FAB rather than in front of or behind it"
                    .into(),
            ],
            vec![
                "Focused controls (web)".into(),
                "Do not fully obscure an element that currently has keyboard focus".into(),
            ],
            vec![
                "Screen edges".into(),
                "Keep consistent leading or center alignment; do not sit flush against one edge"
                    .into(),
            ],
        ],
    );
    gap(Dp(24.0));
    snackbar_figure(false, true, true, false);
}

#[tessera]
fn snackbar_behavior_table() {
    spec_table(
        vec!["Kind".into(), "Timing and interaction".into()],
        vec![
            vec![
                "Message only".into(),
                "May dismiss automatically after 4–10 seconds".into(),
            ],
            vec![
                "With an action".into(),
                "Stays on screen until the action or dismiss control is used".into(),
            ],
            vec![
                "Consecutive updates".into(),
                "Show one at a time; an updated message may replace an outdated one".into(),
            ],
            vec![
                "Web".into(),
                "Pair an auto-dismissing message with inline feedback near its trigger".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            panel(RenderSlot::new(|| {
                sample("Single line");
                gap(Dp(32.0));
                sample("Single line with action").action(true);
                gap(Dp(32.0));
                sample("Two lines").two_lines(true);
                gap(Dp(32.0));
                sample("Two lines with action").two_lines(true).action(true);
                gap(Dp(32.0));
                sample("Two lines with longer action")
                    .two_lines(true)
                    .action(true)
                    .long_action(true)
                    .stacked(true);
            }));
            gap(Dp(112.0));
            subheading("Queued notifications".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(queued));
        });
}

fn request(two_lines: bool, action: bool, dismiss: bool) -> SnackbarRequest {
    SnackbarRequest {
        message: if two_lines {
            "Your photo was saved.\nIt is ready to share."
        } else {
            "Photo saved"
        }
        .to_owned(),
        action_label: action.then(|| "Undo".to_owned()),
        with_dismiss_action: dismiss,
        duration: None,
    }
}

#[tessera]
fn queued() {
    let state = remember(|| {
        let mut state = SnackbarHostState::new();
        state.show_snackbar(request(false, true, true));
        state
    });
    let two_lines = remember(|| false);
    let action = remember(|| true);
    let dismiss = remember(|| true);
    let stacked = remember(|| false);
    let result = remember(|| "Waiting".to_owned());
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    button()
                        .tonal()
                        .selected(two_lines.get())
                        .accessibility_label("Next snackbar two lines")
                        .on_click(move || two_lines.set(!two_lines.get()))
                        .child(|| {
                            text().content("Two lines");
                        });
                    button()
                        .tonal()
                        .selected(action.get())
                        .accessibility_label("Next snackbar action")
                        .on_click(move || action.set(!action.get()))
                        .child(|| {
                            text().content("Action");
                        });
                    button()
                        .tonal()
                        .selected(dismiss.get())
                        .accessibility_label("Next snackbar dismiss")
                        .on_click(move || dismiss.set(!dismiss.get()))
                        .child(|| {
                            text().content("Close icon");
                        });
                    button()
                        .tonal()
                        .selected(stacked.get())
                        .accessibility_label("Stack snackbar action")
                        .on_click(move || stacked.set(!stacked.get()))
                        .child(|| {
                            text().content("Action on new line");
                        });
                    button()
                        .filled()
                        .accessibility_label("Enqueue snackbar")
                        .on_click(move || {
                            state.with_mut(|state| {
                                state.show_snackbar(request(
                                    two_lines.get(),
                                    action.get(),
                                    dismiss.get(),
                                ))
                            });
                            result.set("Waiting".to_owned());
                        })
                        .child(|| {
                            text().content("Enqueue");
                        });
                });
            gap(Dp(24.0));
            snackbar_host()
                .state(state)
                .snackbar(move |data: SnackbarData| {
                    boxed()
                        .modifier(Modifier::new().width(Dp(360.0)).semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some("Queued snackbar surface".to_owned()),
                            merge_descendants: true,
                            ..Default::default()
                        }))
                        .children(move || {
                            let mut bar = snackbar()
                                .modifier(Modifier::new().fill_max_width())
                                .message(data.message().to_owned())
                                .action_label_optional(data.action_label().map(str::to_owned))
                                .action_on_new_line(stacked.get())
                                .on_action({
                                    let data = data.clone();
                                    move || {
                                        data.perform_action();
                                        result.set("Action performed".to_owned());
                                    }
                                });
                            if data.with_dismiss_action() {
                                let data = data.clone();
                                bar = bar
                                    .with_dismiss_action(true)
                                    .dismiss_action_label("Dismiss queued snackbar")
                                    .on_dismiss(move || {
                                        data.dismiss();
                                        result.set("Dismissed".to_owned());
                                    });
                            }
                            drop(bar);
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!("Snackbar result: {}", result.get()))
                .style(type_style(14.0, 20.0));
            text()
                .content(format!(
                    "Snackbar visible: {}",
                    state.with(SnackbarHostState::is_showing)
                ))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn snackbar_figure(two_lines: bool, action: bool, dismiss: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let result = remember(|| "Ready".to_owned());
    let visible = remember(|| true);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(272.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            if visible.get() {
                let mut bar = snackbar()
                    .modifier(watch(geometry, "bar").width(Dp(360.0)))
                    .message(if two_lines {
                        "This is a two-line snackbar
with a longer message."
                    } else {
                        "This is a snackbar message"
                    });
                if action {
                    bar = bar.action_label("Action").on_action(move || {
                        result.set("Action performed".into());
                        visible.set(false);
                    });
                }
                if dismiss {
                    bar = bar
                        .with_dismiss_action(true)
                        .dismiss_action_label("Dismiss illustrated snackbar")
                        .on_dismiss(move || {
                            result.set("Dismissed".into());
                            visible.set(false);
                        });
                }
                drop(bar);
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        snackbar_annotations(geometry, action, dismiss, measurements);
                    });
            } else {
                button()
                    .text()
                    .accessibility_label("Restore illustrated snackbar")
                    .on_click(move || visible.set(true))
                    .child(|| {
                        text().content("Show snackbar");
                    });
            }
        });
    gap(Dp(12.0));
    text().content(result.get()).style(type_style(14.0, 20.0));
}

#[tessera]
fn snackbar_annotations(
    geometry: State<Geometry>,
    action: bool,
    dismiss: bool,
    measurements: bool,
) {
    let values = geometry.get();
    let (Some(root), Some(bar)) = (values.get("figure"), relative(&values, "bar")) else {
        return;
    };
    let mut svg = String::new();
    annotations::outline(&mut svg, bar);
    if measurements {
        annotations::dimension(
            format!("{:.0}", bar.h),
            Dp(bar.x - 40.0),
            Dp(bar.y + bar.h / 2.0 - 8.0),
        );
        let x = bar.x - 16.0;
        annotations::dotted(&mut svg, x, bar.y, bar.h, true, "#49454f");
        annotations::dotted(&mut svg, x - 4.0, bar.y, 8.0, false, "#49454f");
        annotations::dotted(&mut svg, x - 4.0, bar.y + bar.h, 8.0, false, "#49454f");
        annotations::dimension("16".into(), Dp(bar.x + 2.0), Dp(bar.y - 32.0));
        annotations::dotted(&mut svg, bar.x, bar.y - 8.0, 16.0, false, "#49454f");
        annotations::dotted(&mut svg, bar.x, bar.y - 12.0, 8.0, true, "#49454f");
        annotations::dotted(&mut svg, bar.x + 16.0, bar.y - 12.0, 8.0, true, "#49454f");
        svg.push_str(&format!(
            "<rect x='{}' y='{}' width='16' height='{}' fill='#ee7da4' fill-opacity='.35'/>",
            bar.x, bar.y, bar.h
        ));
    } else {
        let message = Rect {
            x: bar.x + 16.0,
            y: bar.y + 14.0,
            w: bar.w
                - if dismiss {
                    160.0
                } else if action {
                    112.0
                } else {
                    32.0
                },
            h: bar.h - 28.0,
        };
        annotations::outline(&mut svg, message);
        let caption_y = bar.y + bar.h + 32.0;
        let label_x = message.x + message.w / 2.0;
        let container_x = bar.x + 28.0;
        annotations::marker(1, Dp(container_x - 12.0), Dp(bar.y - 64.0));
        annotations::dotted(&mut svg, container_x, bar.y - 32.0, 32.0, true, "#49454f");
        svg.push_str(&format!("<circle cx='{container_x}' cy='{}' r='3.5' fill='none' stroke='#49454f' stroke-width='.7'/>",bar.y));
        annotations::marker(2, Dp(label_x - 12.0), Dp(caption_y));
        annotations::dotted(
            &mut svg,
            label_x,
            message.y + message.h,
            caption_y - message.y - message.h,
            true,
            "#49454f",
        );
        if dismiss {
            let icon = Rect {
                x: bar.x + bar.w - 36.0,
                y: bar.y + (bar.h - 24.0) / 2.0,
                w: 24.0,
                h: 24.0,
            };
            annotations::outline(&mut svg, icon);
            let x = bar.x + bar.w + 36.0;
            let y = bar.y + bar.h / 2.0;
            annotations::marker(4, Dp(x), Dp(y - 16.0));
            annotations::dotted(
                &mut svg,
                icon.x + icon.w,
                y,
                x - icon.x - icon.w,
                false,
                "#49454f",
            );
        }
        if action {
            let label = Rect {
                x: bar.x + bar.w - if dismiss { 104.0 } else { 56.0 },
                y: bar.y + (bar.h - 24.0) / 2.0,
                w: 48.0,
                h: 24.0,
            };
            annotations::outline(&mut svg, label);
            let x = label.x + label.w / 2.0;
            annotations::marker(3, Dp(x - 12.0), Dp(caption_y));
            annotations::dotted(
                &mut svg,
                x,
                label.y + label.h,
                caption_y - label.y - label.h,
                true,
                "#49454f",
            );
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn snackbar_variants() {
    prose(
        "Snackbars briefly describe app processes without interrupting work. They can offer a single optional action.",
    );
    gap(Dp(24.0));
    snackbar_figure(false, true, false, false);
}

#[tessera]
fn snackbar_usage() {
    prose(
        "Use snackbars for low-priority process updates. Show one at a time, with at most one optional action and an optional dismiss button.",
    );
    gap(Dp(24.0));
    snackbar_figure(false, true, false, false);
    gap(Dp(24.0));
    subheading("Similar components".into());
    spec_table(
        vec!["Component".into(), "Priority".into(), "Action".into()],
        vec![
            vec!["Snackbar".into(), "Low".into(), "Optional".into()],
            vec!["Dialog".into(), "High".into(), "Required".into()],
        ],
    );
    gap(Dp(24.0));
    subheading("Accessibility requirements for web".into());
    prose(
        "Keep actionable messages visible, or communicate temporary notifications next to their trigger as well.",
    );
    gap(Dp(24.0));
    inline_feedback();
}

#[tessera]
fn snackbar_guideline_anatomy() {
    snackbar_figure(false, true, true, false);
    prose("1 Container    2 Supporting text    3 Action (optional)    4 Close button (optional)");
    gap(Dp(24.0));
    subheading("Text label".into());
    prose(
        "Use a short, direct process update. Keep the label to one line where possible; compact widths can use two lines. Avoid inline links, decorative icons and styled text.",
    );
    gap(Dp(24.0));
    snackbar_writing_examples();
    gap(Dp(24.0));
    snackbar_figure(true, false, false, false);
    gap(Dp(24.0));
    subheading("Container".into());
    prose(
        "Keep the solid rectangular container and its subtle elevation so the update remains legible over content. Do not use a transparent or heavily altered shape.",
    );
    gap(Dp(24.0));
    snackbar_figure(false, false, false, true);
    gap(Dp(24.0));
    subheading("Action".into());
    prose("Use one text button, such as Undo. A longer action may move onto a separate line.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Guideline long action")
            .two_lines(true)
            .action(true)
            .long_action(true)
            .stacked(true);
    }));
}

#[tessera]
fn snackbar_responsive() {
    subheading("Compact breakpoint".into());
    prose(
        "Keep leading, trailing and bottom spacing while allowing the message to wrap vertically.",
    );
    gap(Dp(24.0));
    snackbar_placement(false);
    gap(Dp(24.0));
    subheading("Medium & expanded breakpoints".into());
    prose(
        "Prefer a single readable line, expanding horizontally as needed. Use consistent leading or center alignment with space around the edges.",
    );
    gap(Dp(24.0));
    snackbar_placement(true);
}

#[tessera]
fn snackbar_placement(wide: bool) {
    let visible = remember(|| true);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::BottomCenter)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(if wide { 320.0 } else { 480.0 }))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            if visible.get() {
                snackbar()
                    .modifier(Modifier::new().width(Dp(if wide { 560.0 } else { 312.0 })))
                    .message(if wide {
                        "Your changes have been saved and are ready to share."
                    } else {
                        "Your changes have been saved."
                    })
                    .action_label("Undo")
                    .on_action(move || visible.set(false));
            } else {
                button()
                    .text()
                    .accessibility_label("Restore placement snackbar")
                    .on_click(move || visible.set(true))
                    .child(|| {
                        text().content("Show snackbar");
                    });
            }
        });
}

#[tessera]
fn snackbar_web_feedback() {
    subheading("1. Add inline feedback".into()).level(4);
    prose(
        "Repeat the result near the trigger so the information remains available after a temporary notification disappears.",
    );
    gap(Dp(24.0));
    inline_feedback();
    gap(Dp(24.0));
    subheading("2. Make the snackbar actionable".into()).level(4);
    prose("An actionable snackbar stays visible until the person acts on it or dismisses it.");
    gap(Dp(24.0));
    panel(RenderSlot::new(queued));
}

#[tessera]
fn inline_feedback() {
    let saved = remember(|| false);
    let host = remember(SnackbarHostState::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .filled()
                .accessibility_label(if saved.get() { "Saved" } else { "Save" })
                .on_click(move || {
                    saved.set(true);
                    host.with_mut(|value| {
                        value.show_snackbar(SnackbarRequest {
                            message: "Changes saved".into(),
                            action_label: Some("Undo".into()),
                            with_dismiss_action: true,
                            duration: Some(SnackbarDuration::Indefinite),
                        });
                    });
                })
                .child(move || {
                    text().content(if saved.get() { "Saved" } else { "Save" });
                });
            gap(Dp(16.0));
            text()
                .content(if saved.get() {
                    "All changes saved"
                } else {
                    "Unsaved changes"
                })
                .style(type_style(16.0, 24.0));
            gap(Dp(24.0));
            snackbar_host()
                .state(host)
                .snackbar(move |data: SnackbarData| {
                    let dismiss = data.clone();
                    snackbar()
                        .message(data.message().to_owned())
                        .action_label("Undo")
                        .on_action(move || {
                            data.perform_action();
                            saved.set(false);
                        })
                        .with_dismiss_action(true)
                        .on_dismiss(move || dismiss.dismiss());
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod anatomy_tests {
    use super::*;
    use std::time::Duration;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
    };
    struct Pipelines;
    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }
    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            column().children(|| {
                snackbar_figure(false, true, true, false);
            });
        });
    }
    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label} {role:?}"))
    }
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn snackbar_anatomy_places_four_callouts_around_native_text_and_actions() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 900,
                height: 400,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(5);
        let message = node(&renderer, "This is a snackbar message", Role::Status)
            .1
            .bounds()
            .unwrap();
        let (action_id, action) = node(&renderer, "Action", Role::Button);
        let action = action.bounds().unwrap();
        let close = node(&renderer, "Dismiss illustrated snackbar", Role::Button)
            .1
            .bounds()
            .unwrap();
        let container_marker = node(&renderer, "Anatomy marker 1", Role::Note)
            .1
            .bounds()
            .unwrap();
        let text_marker = node(&renderer, "Anatomy marker 2", Role::Note)
            .1
            .bounds()
            .unwrap();
        let action_marker = node(&renderer, "Anatomy marker 3", Role::Note)
            .1
            .bounds()
            .unwrap();
        let close_marker = node(&renderer, "Anatomy marker 4", Role::Note)
            .1
            .bounds()
            .unwrap();
        assert!(container_marker.y1 < message.y0);
        assert!(text_marker.y0 > message.y1);
        let text_center = (text_marker.x0 + text_marker.x1) / 2.0;
        assert!(text_center >= message.x0 && text_center <= message.x1);
        assert!(action_marker.y0 > action.y1);
        let action_center = (action_marker.x0 + action_marker.x1) / 2.0;
        assert!(action_center >= action.x0 && action_center <= action.x1);
        assert!(close_marker.x0 > close.x1);
        let close_center = (close_marker.y0 + close_marker.y1) / 2.0;
        assert!(close_center >= close.y0 && close_center <= close.y1);
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: action_id,
            data: None
        }));
        renderer.render_frames(4);
        node(&renderer, "Action performed", Role::Label);
        node(&renderer, "Restore illustrated snackbar", Role::Button);
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Status)
        );
    }
}
