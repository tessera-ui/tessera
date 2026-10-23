//! Dialog specification figures and modal interactions.
//!
//! ## Usage
//!
//! Inspect dialog anatomy, spacing and focus behavior.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    checkbox::checkbox,
    column::column,
    dialog::{basic_dialog, dialog_provider, full_screen_dialog},
    divider::horizontal_divider,
    icon::icon,
    material_icons::filled,
    modifier::{MinimumInteractiveComponentEnforcement, ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    text_field::text_field,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Dp, Modifier, RenderSlot, State, accesskit::Role, provide_context, remember, tessera,
    use_context,
};

use super::{
    INK,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn dialogs_spec() {
    let open = remember(|| false);
    let full = remember(|| false);
    let saved = remember(|| false);
    dialog_provider()
        .is_open(open.get())
        .full_screen(full.get())
        .accessibility_label(if full.get() { "New event" } else { "Reset settings?" })
        .on_close_request(move || open.set(false))
        .main_content(move || {
            article("Dialogs", "Dialogs provide important prompts in a user flow", vec![
                SpecSection { title: "Tokens & specs", content: RenderSlot::new(tokens) },
                SpecSection { title: "Basic dialogs", content: RenderSlot::new(basic_specs) },
                SpecSection { title: "Full-screen dialogs", content: RenderSlot::new(full_specs) },
            ])
            .pages(vec![
                ArticlePage { title: "Overview", intro: Some(RenderSlot::new(move || {
                    prose("Dialogs interrupt a flow for one important decision or task. Basic dialogs support brief confirmations; full-screen dialogs provide room for a longer task.");
                    gap(Dp(32.0));
                    launch(open, full, saved);
                    gap(Dp(32.0));
                    basic_figure(false);
                })), sections: vec![] },
                ArticlePage { title: "Guidelines", intro: None, sections: vec![
                    SpecSection { title: "Usage", content: RenderSlot::new(|| { prose("Reserve dialogs for critical information or decisions that must interrupt the current task. Use a snackbar for lower-priority feedback."); }) },
                    SpecSection { title: "Anatomy", content: RenderSlot::new(|| { basic_figure(false); }) },
                    SpecSection { title: "Basic dialog", content: RenderSlot::new(|| { prose("Use a clear headline that describes the decision. Place the dismissive action before the confirming action at the trailing edge; provide no more than two actions."); }) },
                    SpecSection { title: "Full-screen dialog", content: RenderSlot::new(|| { prose("Use a full-screen dialog for a sequence of related tasks. The close icon dismisses the task; a clearly named action commits it. Confirm before discarding unsaved changes."); gap(Dp(24.0)); full_figure(); }) },
                    SpecSection { title: "Adaptive design", content: RenderSlot::new(|| { prose("At compact widths, a task can occupy the full screen. At larger widths, use a centered basic dialog and retain margins around the container."); }) },
                    SpecSection { title: "Behavior", content: RenderSlot::new(move || { prose("Keep focus within an open dialog. Closing it returns focus to its trigger. When content scrolls, keep the title and actions available."); gap(Dp(24.0)); launch(open, full, saved); }) },
                ] },
                ArticlePage { title: "Accessibility", intro: None, sections: vec![
                    SpecSection { title: "Use cases", content: RenderSlot::new(move || { prose("People using assistive technology must be able to open and close the dialog, edit its inputs, submit the task and reach any scrolling content."); gap(Dp(24.0)); launch(open, full, saved); }) },
                    SpecSection { title: "Interaction & style", content: RenderSlot::new(|| { subheading("Use sparingly".into()); prose("An interruptive modal changes the reading flow. Keep non-critical information in the page instead."); gap(Dp(24.0)); subheading("200% text size".into()); prose("Use concise labels and allow content to grow without hiding the meaning. Follow the accessibility requirements of the buttons and fields inside the dialog."); }) },
                    SpecSection { title: "Initial focus", content: RenderSlot::new(|| { prose("Opening a dialog moves focus to its first interactive control. Tab and Shift + Tab cycle within the dialog until it closes."); }) },
                    SpecSection { title: "Keyboard navigation", content: RenderSlot::new(|| { spec_table(vec!["Keys".into(),"Action".into()], vec![vec!["Tab".into(),"Next dialog control; wrap at the end".into()],vec!["Shift + Tab".into(),"Previous dialog control; wrap at the start".into()],vec!["Space or Enter".into(),"Activate the focused control".into()],vec!["Escape".into(),"Close the dialog".into()]]); }) },
                    SpecSection { title: "Labeling elements", content: RenderSlot::new(|| { prose("Use the dialog headline as its accessible label. Label every contained control according to its purpose, including the close affordance and editable fields."); }) },
                ] },
            ]);
        })
        .dialog_content(move || {
            if full.get() {
                full_screen_dialog(Callback::new(move || open.set(false)))
                    .title("New event")
                    .on_confirm(move || { saved.set(true); open.set(false); })
                    .content(event_form);
            } else {
                basic_dialog().headline("Reset settings?")
                    .supporting_text("This will reset your app preferences back to their default settings.")
                    .dismiss_button(move || { action("Cancel".into(), Callback::new(move || open.set(false))); })
                    .confirm_button(move || { action("Accept".into(), Callback::new(move || { saved.set(true); open.set(false); })); });
            }
        });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn launch(open: State<bool>, full: State<bool>, saved: State<bool>) {
    column().children(move || {
        button()
            .outlined()
            .accessibility_label("Open basic dialog")
            .on_click(move || {
                full.set(false);
                saved.set(false);
                open.set(true);
            })
            .child(|| {
                text().content("Open basic dialog");
            });
        gap(Dp(12.0));
        button()
            .outlined()
            .accessibility_label("Open full-screen dialog")
            .on_click(move || {
                full.set(true);
                saved.set(false);
                open.set(true);
            })
            .child(|| {
                text().content("Open full-screen dialog");
            });
        if saved.get() {
            gap(Dp(12.0));
            prose("Changes saved");
        }
    });
}

#[tessera]
fn action(label: String, callback: Callback) {
    button()
        .text()
        .accessibility_label(label.clone())
        .on_click_shared(callback)
        .child(move || {
            text().content(label.clone());
        });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Surface container high".into()],
            vec!["Icon".into(), "Secondary · 24dp".into()],
            vec!["Headline".into(), "On surface · Headline small".into()],
            vec![
                "Supporting text".into(),
                "On surface variant · Body medium".into(),
            ],
            vec!["Actions".into(), "Primary · Label large".into()],
            vec!["Basic shape".into(), "28dp corner radius".into()],
        ],
    );
}

#[tessera]
fn basic_specs() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            basic_figure(false);
            gap(Dp(24.0));
            for (number, label) in [
                (1, "Container"),
                (2, "Icon (optional)"),
                (3, "Headline (optional)"),
                (4, "Supporting text"),
                (5, "Divider (optional)"),
                (6, "Button label text"),
                (7, "Scrim"),
            ] {
                prose(format!("{number}  {label}"));
                gap(Dp(12.0));
            }
            gap(Dp(56.0));
            subheading("Basic dialog color".into());
            gap(Dp(24.0));
            tokens();
            gap(Dp(56.0));
            subheading("Basic dialog measurements".into());
            gap(Dp(32.0));
            basic_figure(true);
            gap(Dp(24.0));
            prose("Basic dialog padding and size measurements");
            gap(Dp(32.0));
            spec_table(
                vec!["Attribute".into(), "Value".into()],
                vec![
                    vec!["Container shape".into(), "28dp corner radius".into()],
                    vec!["Container height".into(), "Dynamic".into()],
                    vec!["Container width".into(), "Min 280dp; Max 560dp".into()],
                    vec!["Divider height".into(), "1dp".into()],
                    vec!["Icon size".into(), "24dp".into()],
                    vec!["Alignment with icon".into(), "Center-aligned".into()],
                    vec!["Alignment without icon".into(), "Start-aligned".into()],
                    vec!["Top/left/right/bottom padding".into(), "24dp".into()],
                    vec!["Padding between buttons".into(), "8dp".into()],
                    vec!["Padding between title and body".into(), "16dp".into()],
                    vec!["Padding between icon and title".into(), "16dp".into()],
                    vec!["Padding between body and actions".into(), "24dp".into()],
                ],
            );
        });
}

#[tessera]
fn basic_figure(measurements: bool) {
    let geometry = remember(Geometry::new);
    let notice = remember(String::new);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch(geometry, "figure")
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            merge_descendants: true,
                            label: Some(
                                if measurements {
                                    "Basic dialog measurement figure"
                                } else {
                                    "Basic dialog anatomy figure"
                                }
                                .into(),
                            ),
                            ..Default::default()
                        })
                        .fill_max_width()
                        .height(Dp(if measurements { 420.0 } else { 696.0 }))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    if !measurements {
                        spacer().modifier(Modifier::new().fill_max_size().background_with_shape(
                            scheme.scrim.with_alpha(0.32),
                            Shape::rounded_rectangle(Dp(24.0)),
                        ));
                    }
                    boxed()
                        .modifier(
                            watch(geometry, "container")
                                .semantics(SemanticsArgs {
                                    role: Some(Role::Group),
                                    merge_descendants: true,
                                    label: Some(
                                        if measurements {
                                            "Basic dialog measurement container"
                                        } else {
                                            "Basic dialog anatomy container"
                                        }
                                        .into(),
                                    ),
                                    ..Default::default()
                                })
                                .width(Dp(312.0))
                                .background_with_shape(
                                    scheme.surface_container_high,
                                    Shape::rounded_rectangle(Dp(28.0)),
                                )
                                .padding_all(Dp(24.0)),
                        )
                        .children(move || {
                            provide_context(
                                || MinimumInteractiveComponentEnforcement { enabled: false },
                                move || {
                                    let mut dialog = basic_dialog()
                                        .headline(if measurements {
                                            "Basic dialog title"
                                        } else {
                                            "Reset settings?"
                                        })
                                        .content(move || {
                                            dialog_body(geometry, measurements);
                                        })
                                        .dismiss_button(move || {
                                            boxed().modifier(watch(geometry, "dismiss")).children(
                                                move || {
                                                    action(
                                                        if measurements {
                                                            "Action 2"
                                                        } else {
                                                            "Cancel"
                                                        }
                                                        .into(),
                                                        Callback::new(move || {
                                                            notice.set("Dismissed".into())
                                                        }),
                                                    );
                                                },
                                            );
                                        })
                                        .confirm_button(move || {
                                            boxed().modifier(watch(geometry, "confirm")).children(
                                                move || {
                                                    action(
                                                        if measurements {
                                                            "Action 1"
                                                        } else {
                                                            "Accept"
                                                        }
                                                        .into(),
                                                        Callback::new(move || {
                                                            notice.set("Accepted".into())
                                                        }),
                                                    );
                                                },
                                            );
                                        });
                                    if !measurements {
                                        dialog = dialog.icon(move || {
                                            boxed().modifier(watch(geometry, "icon")).children(
                                                || {
                                                    icon()
                                                        .vector(filled::RESTART_ALT_SVG)
                                                        .size(Dp(24.0));
                                                },
                                            );
                                        });
                                    }
                                    drop(dialog);
                                },
                            );
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            basic_annotations(geometry, measurements);
                        });
                });
            if !notice.with(String::is_empty) {
                gap(Dp(12.0));
                prose(notice.get());
            }
        });
}

#[tessera]
fn dialog_body(geometry: State<Geometry>, measurements: bool) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    column().modifier(Modifier::new().fill_max_width()).children(move || {
        boxed().modifier(watch(geometry,"body").fill_max_width()).children(move || {
            text().content(if measurements {
                "A dialog is a modal window that appears in front of app content to provide critical information or prompt for a decision to be made."
            } else { "This will reset your app preferences back to their default settings. The following accounts will also be signed out:" })
                .style(theme.typography.body_medium).color(theme.color_scheme.on_surface_variant);
        });
        if !measurements {
            gap(Dp(16.0));
            boxed().modifier(watch(geometry,"divider").fill_max_width()).children(|| { horizontal_divider(); });
            gap(Dp(8.0));
            for (initials,email) in [("LN","leevillanuevanotes@google.com"),("AA","alloalejandro@google.com"),("LO","oliortega@google.com")] {
                row().cross_axis_alignment(CrossAxisAlignment::Center).modifier(Modifier::new().fill_max_width().height(Dp(56.0))).children(move || {
                    boxed().alignment(Alignment::Center).modifier(Modifier::new().size(Dp(40.0),Dp(40.0)).background_with_shape(theme.color_scheme.secondary_container,Shape::Ellipse)).children(move || { text().content(initials).color(theme.color_scheme.on_secondary_container); });
                    spacer().modifier(Modifier::new().width(Dp(16.0)));
                    text().content(email).style(theme.typography.body_medium).modifier(Modifier::new().weight(1.0));
                });
            }
            gap(Dp(8.0)); horizontal_divider();
        }
    });
}

#[tessera]
fn basic_annotations(geometry: State<Geometry>, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(body), Some(dismiss), Some(confirm)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "body"),
        relative(&values, "dismiss"),
        relative(&values, "confirm"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let actions = Rect {
        x: dismiss.x,
        y: dismiss.y.min(confirm.y),
        w: confirm.x + confirm.w - dismiss.x,
        h: dismiss.h.max(confirm.h),
    };
    let mut svg = String::new();
    if measurements {
        let bottom_body = body.y + body.h;
        for rect in [
            Rect {
                x: container.x,
                y: container.y,
                w: 24.0,
                h: container.h,
            },
            Rect {
                x: container.x + container.w - 24.0,
                y: container.y,
                w: 24.0,
                h: container.h,
            },
            Rect {
                x: container.x,
                y: container.y,
                w: container.w,
                h: 24.0,
            },
            Rect {
                x: container.x,
                y: body.y - 16.0,
                w: container.w,
                h: 16.0,
            },
            Rect {
                x: container.x,
                y: bottom_body,
                w: container.w,
                h: 24.0,
            },
            Rect {
                x: container.x,
                y: container.y + container.h - 24.0,
                w: container.w,
                h: 24.0,
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#e75f96' fill-opacity='.28'/>",
                rect.x, rect.y, rect.w, rect.h
            ));
        }
        for (y, label) in [
            (body.y - 8.0, "16"),
            (bottom_body + 12.0, "24"),
            (container.y + container.h - 12.0, "24"),
        ] {
            dotted(
                &mut svg,
                container.x + container.w,
                y,
                24.0,
                false,
                "#ed7099",
            );
            annotations::dimension(
                label.into(),
                Dp(container.x + container.w + 28.0),
                Dp(y - 10.0),
            );
        }
        dotted(
            &mut svg,
            container.x,
            container.y + 12.0,
            -32.0,
            false,
            "#ed7099",
        );
        annotations::dimension("24".into(), Dp(container.x - 50.0), Dp(container.y + 2.0));
        for x in [container.x + 12.0, container.x + container.w - 12.0] {
            dotted(
                &mut svg,
                x,
                container.y + container.h,
                28.0,
                true,
                "#ed7099",
            );
            annotations::dimension(
                "24".into(),
                Dp(x - 8.0),
                Dp(container.y + container.h + 32.0),
            );
        }
    } else {
        let Some(icon) = relative(&values, "icon") else {
            return;
        };
        let Some(divider) = relative(&values, "divider") else {
            return;
        };
        let right = (container.x + container.w + 64.0).min(width.0 - 28.0);
        for (number, x, y, target_x, target_y) in [
            (
                1,
                (container.x - 52.0).max(4.0),
                container.y + 24.0,
                container.x + 8.0,
                container.y + 40.0,
            ),
            (2, icon.x, container.y - 76.0, icon.x + 12.0, icon.y),
            (
                3,
                right,
                icon.y + 40.0,
                container.x + container.w * 0.75,
                icon.y + 56.0,
            ),
            (4, right, body.y + 8.0, body.x + body.w, body.y + 24.0),
            (5, right, divider.y - 16.0, divider.x + divider.w, divider.y),
            (
                6,
                right,
                actions.y + 4.0,
                actions.x + actions.w,
                actions.y + 20.0,
            ),
            (
                7,
                width.0 * 0.5 - 12.0,
                container.y + container.h + 40.0,
                width.0 * 0.5,
                container.y + container.h + 40.0,
            ),
        ] {
            let cx = x + 12.0;
            let cy = y + 16.0;
            if number == 2 {
                dotted(
                    &mut svg,
                    cx,
                    cy + 16.0,
                    target_y - cy - 16.0,
                    true,
                    "#49454f",
                );
            } else if number != 7 {
                let start = if target_x < cx { x } else { x + 24.0 };
                dotted(
                    &mut svg,
                    start,
                    target_y,
                    target_x - start,
                    false,
                    "#49454f",
                );
            }
            annotations::marker(number, Dp(x), Dp(y));
        }
        annotations::outline(&mut svg, icon);
        annotations::outline(&mut svg, actions);
    }
    if !svg.is_empty() {
        annotations::lines(width, height, svg);
    }
}

#[tessera]
fn full_specs() {
    column().modifier(Modifier::new().fill_max_width()).children(|| {
        full_figure();
        gap(Dp(24.0)); prose("1  Container   2  Header   3  Close affordance   4  Headline   5  Text button   6  Divider");
        gap(Dp(56.0)); subheading("Full-screen dialog color".into());
        gap(Dp(24.0)); prose("Surface container high · On surface · Primary · On surface variant");
        gap(Dp(56.0)); subheading("Full-screen dialog measurements".into());
        gap(Dp(24.0)); spec_table(vec!["Attribute".into(),"Value".into()],vec![
            vec!["Container shape".into(),"0dp corner radius".into()],vec!["Container width".into(),"Container width; Max 560dp".into()],
            vec!["Header height".into(),"64dp".into()],vec!["Close icon".into(),"24dp".into()],
            vec!["Content padding".into(),"24dp".into()],vec!["Padding between elements".into(),"8dp".into()],
        ]);
    });
}

#[tessera]
fn full_figure() {
    let visible = remember(|| true);
    let saved = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(640.0))
                .padding_all(Dp(24.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            if visible.get() {
                boxed()
                    .modifier(
                        Modifier::new()
                            .width(Dp(360.0))
                            .height(Dp(544.0))
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                merge_descendants: true,
                                label: Some("Full-screen dialog figure".into()),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        full_screen_dialog(Callback::new(move || visible.set(false)))
                            .title("New event")
                            .on_confirm(move || {
                                saved.set(true);
                                visible.set(false);
                            })
                            .content(event_form);
                    });
            } else {
                column().children(move || {
                    prose(if saved.get() {
                        "Event saved"
                    } else {
                        "Dialog closed"
                    });
                    gap(Dp(16.0));
                    button()
                        .outlined()
                        .on_click(move || {
                            saved.set(false);
                            visible.set(true);
                        })
                        .child(|| {
                            text().content("Show full-screen dialog");
                        });
                });
            }
        });
}

#[tessera]
fn event_form() {
    let all_day = remember(|| false);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            text_field()
                .label("Event name")
                .accessibility_label("Event name")
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(24.0));
            text_field()
                .label("From")
                .initial_text("Mon, May 18")
                .accessibility_label("From")
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(16.0));
            text_field()
                .label("To")
                .initial_text("Mon, May 18")
                .accessibility_label("To")
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(16.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    checkbox()
                        .checked(all_day.get())
                        .on_toggle(move |value| all_day.set(value))
                        .accessibility_label("All-day");
                    text().content("All-day");
                });
            gap(Dp(16.0));
            prose("Time zone");
            text_field()
                .label("Time zone")
                .initial_text("GMT-07:00")
                .accessibility_label("Time zone")
                .modifier(Modifier::new().fill_max_width());
        });
}
