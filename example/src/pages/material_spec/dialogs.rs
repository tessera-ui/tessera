//! Dialog specification figures and modal interactions.
//!
//! ## Usage
//!
//! Inspect dialog anatomy, spacing and focus behavior.

use std::{collections::BTreeMap, sync::OnceLock};

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    checkbox::checkbox,
    column::column,
    dialog::{DialogRole, basic_dialog, dialog_provider, full_screen_dialog},
    icon::icon,
    image::{ImageData, TryIntoImageData as _, image},
    material_icons::filled,
    menus::{MenuAnchor, MenuController, menu_item, menu_provider},
    modifier::{MinimumInteractiveComponentEnforcement, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    text_field::{TextFieldBuilder, text_field},
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, LayoutCoordinates, Modifier, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role, modifier::FocusModifierExt as _, provide_context, remember, scaled_box,
    tessera, use_context,
};

use super::{
    INK,
    annotations::{self, PreciseGeometry as Geometry, Rect, dotted, watch_precise as watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[path = "dialogs_accessibility.rs"]
mod accessibility;

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "dialogs_tests.rs"]
mod regression_tests;

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "dialogs_scaled_tests.rs"]
mod canonical_tests;

#[derive(Clone, PartialEq)]
struct EventDraft {
    name: String,
    from: String,
    to: String,
    time_zone: String,
    all_day: bool,
}

impl Default for EventDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            from: "Mon, May 18".into(),
            to: "Mon, May 18".into(),
            time_zone: "GMT-07:00".into(),
            all_day: false,
        }
    }
}

impl EventDraft {
    fn is_dirty(&self) -> bool {
        self != &Self::default()
    }
}

fn request_dialog_close(
    open: State<bool>,
    full: State<bool>,
    discard: State<bool>,
    draft: State<EventDraft>,
) {
    if full.get() && draft.with(EventDraft::is_dirty) {
        discard.set(true);
    } else {
        open.set(false);
    }
}

#[tessera]
pub(super) fn dialogs_spec() {
    let open = remember(|| false);
    let full = remember(|| false);
    let saved = remember(|| false);
    let draft = remember(EventDraft::default);
    let discard = remember(|| false);
    let close = Callback::new(move || request_dialog_close(open, full, discard, draft));
    dialog_provider()
        .role(if full.get() {DialogRole::Dialog} else {DialogRole::AlertDialog})
        .is_open(open.get())
        .full_screen(full.get())
        .accessibility_label(if full.get() { "New event" } else { "Reset settings?" })
        .on_close_request_shared(close)
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
                    dialog_overview_pair();
                    gap(Dp(32.0));
                    launch(open, full, saved);
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
                    SpecSection { title: "Interaction & style", content: RenderSlot::new(accessibility::interaction_style) },
                    SpecSection { title: "Initial focus", content: RenderSlot::new(accessibility::initial_focus) },
                    SpecSection { title: "Keyboard navigation", content: RenderSlot::new(|| { spec_table(vec!["Keys".into(),"Action".into()], vec![vec!["Tab".into(),"Next dialog control; wrap at the end".into()],vec!["Shift + Tab".into(),"Previous dialog control; wrap at the start".into()],vec!["Space or Enter".into(),"Activate the focused control".into()],vec!["Escape".into(),"Close the dialog".into()]]); }) },
                    SpecSection { title: "Labeling elements", content: RenderSlot::new(accessibility::labeling) },
                ] },
            ]);
        })
        .dialog_content(move || {
            if full.get() {
                event_dialog(draft, open, saved, discard, close);
            } else {
                basic_dialog().headline("Reset settings?")
                    .supporting_text("This will reset your app preferences back to their default settings.")
                    .dismiss_button(move || { action("Cancel".into(), Callback::new(move || open.set(false))); })
                    .confirm_button(move || { action("Accept".into(), Callback::new(move || { saved.set(true); open.set(false); })); });
            }
        });
}

#[tessera]
fn event_dialog(
    draft: State<EventDraft>,
    open: State<bool>,
    saved: State<bool>,
    discard: State<bool>,
    close: Callback,
) {
    dialog_provider()
        .role(DialogRole::AlertDialog)
        .is_open(discard.get())
        .accessibility_label("Discard changes?")
        .on_close_request(move || discard.set(false))
        .main_content(move || {
            full_screen_dialog(close)
                .title("New event")
                .on_confirm(move || {
                    draft.set(EventDraft::default());
                    saved.set(true);
                    discard.set(false);
                    open.set(false);
                })
                .content(move || {
                    event_form(draft);
                });
        })
        .dialog_content(move || {
            basic_dialog()
                .headline("Discard changes?")
                .supporting_text("Discard the changes you've made to this event?")
                .dismiss_button(move || {
                    action(
                        "Keep editing".into(),
                        Callback::new(move || discard.set(false)),
                    );
                })
                .confirm_button(move || {
                    action(
                        "Discard".into(),
                        Callback::new(move || {
                            draft.set(EventDraft::default());
                            discard.set(false);
                            open.set(false);
                        }),
                    );
                });
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
            basic_color_figure();
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
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::GEOMETRY.set(Some(geometry));
    let scale = 1.0_f64;
    let notice = remember(String::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::NOTICE.set(Some(notice));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            dialog_canvas(
                1024.0,
                if measurements { 426.0 } else { 698.0 },
                RenderSlot::new(move || {
                    boxed()
                        .modifier(
                            watch(geometry, "figure")
                                .fill_max_width()
                                .height(Dp(if measurements { 426.0 } else { 698.0 } * scale))
                                .background_with_shape(
                                    scheme.surface_container_low,
                                    Shape::rounded_rectangle(Dp(24.0)),
                                )
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
                                }),
                        )
                        .children(move || {
                            if !measurements {
                                spacer().modifier(
                                    Modifier::new().fill_max_size().background_with_shape(
                                        scheme.scrim.with_alpha(0.32),
                                        Shape::rounded_rectangle(Dp(24.0)),
                                    ),
                                );
                            }
                            boxed()
                                .modifier(
                                    watch(geometry, "container")
                                        .offset(
                                            Dp(if measurements { 360.0 } else { 356.0 } * scale),
                                            Dp(if measurements { 70.0 } else { 123.0 } * scale),
                                        )
                                        .width(Dp(312.0 * scale))
                                        .background_with_shape(
                                            scheme.surface_container_high,
                                            Shape::rounded_rectangle(Dp(28.0 * scale)),
                                        )
                                        .padding_all(Dp(24.0 * scale))
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
                                        }),
                                )
                                .children(move || {
                                    dialog_touch_targets(RenderSlot::new(move || {
                                        column()
                                            .modifier(Modifier::new().fill_max_width())
                                            .children(move || {
                                                if !measurements {
                                                    boxed()
                                                        .alignment(Alignment::Center)
                                                        .modifier(Modifier::new().fill_max_width())
                                                        .children(move || {
                                                            boxed()
                                                                .modifier(watch(geometry, "icon"))
                                                                .children(move || {
                                                                    icon()
                                                                        .vector(
                                                                            filled::RESTART_ALT_SVG,
                                                                        )
                                                                        .size(Dp(24.0 * scale))
                                                                        .tint(scheme.secondary);
                                                                });
                                                        });
                                                    gap(Dp(16.0 * scale));
                                                }
                                                text()
                                                    .content(if measurements {
                                                        "Basic dialog title"
                                                    } else {
                                                        "Reset settings?"
                                                    })
                                                    .style(dialog_type_style(24.0, 32.0, scale))
                                                    .color(scheme.on_surface)
                                                    .modifier(if measurements {
                                                        watch(geometry, "headline")
                                                    } else {
                                                        watch(geometry, "headline")
                                                            .align(Alignment::Center)
                                                    });
                                                gap(Dp(16.0 * scale));
                                                dialog_body(geometry, measurements, scale);
                                                gap(Dp(
                                                    if measurements { 24.0 } else { 16.0 } * scale
                                                ));
                                                row()
                                                    .main_axis_alignment(MainAxisAlignment::End)
                                                    .modifier(Modifier::new().fill_max_width())
                                                    .children(move || {
                                                        dialog_figure_action(
                                                            if measurements {
                                                                "Action 2"
                                                            } else {
                                                                "Cancel"
                                                            }
                                                            .into(),
                                                            geometry,
                                                            false,
                                                            scale,
                                                            Callback::new(move || {
                                                                notice.set("Dismissed".into())
                                                            }),
                                                        );
                                                        spacer().modifier(
                                                            Modifier::new().width(Dp(8.0 * scale)),
                                                        );
                                                        dialog_figure_action(
                                                            if measurements {
                                                                "Action 1"
                                                            } else {
                                                                "Accept"
                                                            }
                                                            .into(),
                                                            geometry,
                                                            true,
                                                            scale,
                                                            Callback::new(move || {
                                                                notice.set("Accepted".into())
                                                            }),
                                                        );
                                                    });
                                            });
                                    }));
                                });
                            boxed()
                                .modifier(Modifier::new().fill_max_size())
                                .children(move || {
                                    basic_annotations(geometry, measurements, scale);
                                });
                        });
                }),
            );
            if !notice.with(String::is_empty) {
                gap(Dp(12.0));
                prose(notice.get());
            }
        });
}

fn dialog_type_style(size: f64, line: f64, scale: f64) -> tessera_components::theme::TextStyle {
    let mut style = type_style(size, line);
    style.font_size = Dp(style.font_size.0 * scale);
    style.line_height = style.line_height.map(|height| Dp(height.0 * scale));
    style.letter_spacing = Dp(style.letter_spacing.0 * scale);
    style
}

#[tessera]
fn dialog_figure_action(
    label: String,
    geometry: State<Geometry>,
    confirm: bool,
    scale: f64,
    callback: Callback,
) {
    button()
        .text()
        .modifier(
            watch(geometry, if confirm { "confirm" } else { "dismiss" })
                .height(Dp(40.0 * scale))
                .width(Dp(if label.starts_with("Action") {
                    84.0
                } else if confirm {
                    80.0
                } else {
                    77.0
                } * scale)),
        )
        .content_padding(Padding::symmetric(Dp(16.0 * scale), Dp::ZERO))
        .accessibility_label(label.clone())
        .on_click_shared(callback)
        .child(move || {
            let mut style = dialog_type_style(14.0, 20.0, scale);
            style.font_weight = 500;
            text().content(label.clone()).style(style);
        });
}

fn dialog_account_photo() -> ImageData {
    static PHOTO: OnceLock<ImageData> = OnceLock::new();
    PHOTO
        .get_or_init(|| {
            include_bytes!("../../../assets/material_spec/dialog-basic-anatomy.png")
                .as_slice()
                .try_into_image_data()
                .expect("official dialog account photographs")
        })
        .clone()
}

#[tessera]
fn dialog_account_avatar(index: usize, scale: f64) {
    let photo = dialog_account_photo();
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(40.0 * scale), Dp(40.0 * scale))
                .clip_shape(Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(format!("Account photo {}", index + 1)),
                    ..Default::default()
                }),
        )
        .children(move || {
            image().raster(photo.clone()).modifier(
                Modifier::new()
                    .offset(
                        Dp(-378.0 * scale),
                        Dp(-(327.0 + index as f64 * 56.0) * scale),
                    )
                    .required_size(Dp(1024.0 * scale), Dp(698.0 * scale)),
            );
        });
}

#[tessera]
fn dialog_body(geometry: State<Geometry>, measurements: bool, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width()).children(move || {
        text().content(if measurements {
            "A dialog is a modal window that appears in front of app content to provide critical information or prompt for a decision to be made."
        } else {
            "This will reset your app preferences back to their default settings. The following accounts will also be signed out:"
        }).style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
            .modifier(watch(geometry, "body").fill_max_width());
        if !measurements {
            gap(Dp(16.0 * scale));
            spacer().modifier(watch(geometry, "divider").fill_max_width().height(Dp(scale)).background(scheme.outline_variant));
            gap(Dp(8.0 * scale));
            for (index, email) in ["leevillanuevanotes@google.com", "alloalejandro@google.com", "oliortega@google.com"].into_iter().enumerate() {
                row().cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(56.0 * scale))).children(move || {
                        dialog_account_avatar(index, scale);
                        spacer().modifier(Modifier::new().width(Dp(16.0 * scale)));
                        text().content(email).style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface)
                            .modifier(Modifier::new().weight(1.0));
                    });
            }
            gap(Dp(8.0 * scale));
            spacer().modifier(Modifier::new().fill_max_width().height(Dp(scale)).background(scheme.outline_variant));
        }
    });
}

#[tessera]
fn basic_annotations(geometry: State<Geometry>, measurements: bool, scale: f64) {
    basic_annotation_content(
        geometry.with(annotations::relative_precise_geometry),
        measurements,
        scale,
    );
}

#[tessera]
fn basic_annotation_content(values: BTreeMap<&'static str, Rect>, measurements: bool, scale: f64) {
    let (Some(root), Some(container), Some(headline), Some(body), Some(dismiss), Some(confirm)) = (
        values.get("figure"),
        values.get("container").copied(),
        values.get("headline").copied(),
        values.get("body").copied(),
        values.get("dismiss").copied(),
        values.get("confirm").copied(),
    ) else {
        return;
    };
    let width = Dp(root.w);
    let height = Dp(root.h);
    let actions = Rect {
        x: dismiss.x,
        y: dismiss.y.min(confirm.y),
        w: confirm.x + confirm.w - dismiss.x,
        h: dismiss.h.max(confirm.h),
    };
    let mut svg = String::new();
    if measurements {
        let horizontal = 24.0 * scale;
        for rect in [
            Rect {
                x: container.x,
                y: container.y,
                w: horizontal,
                h: container.h,
            },
            Rect {
                x: container.x + container.w - horizontal,
                y: container.y,
                w: horizontal,
                h: container.h,
            },
            Rect {
                x: container.x,
                y: container.y,
                w: container.w,
                h: horizontal,
            },
            Rect {
                x: container.x,
                y: headline.y + headline.h,
                w: container.w,
                h: body.y - headline.y - headline.h,
            },
            Rect {
                x: container.x,
                y: body.y + body.h,
                w: container.w,
                h: actions.y - body.y - body.h,
            },
            Rect {
                x: container.x,
                y: actions.y + actions.h,
                w: container.w,
                h: container.y + container.h - actions.y - actions.h,
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#e75f96' fill-opacity='.28'/>",
                rect.x,
                rect.y,
                rect.w,
                rect.h.max(0.0)
            ));
        }
        for (y, label) in [
            (headline.y + headline.h + 8.0 * scale, "16"),
            (body.y + body.h + 12.0 * scale, "24"),
            (container.y + container.h - 12.0 * scale, "24"),
        ] {
            dotted(
                &mut svg,
                container.x + container.w,
                y,
                24.0 * scale,
                false,
                "#ed7099",
            );
            dialog_dimension(
                label.into(),
                container.x + container.w + 28.0 * scale,
                y - 10.0 * scale,
                scale,
            );
        }
        dotted(
            &mut svg,
            container.x,
            container.y + 12.0 * scale,
            -32.0 * scale,
            false,
            "#ed7099",
        );
        dialog_dimension(
            "24".into(),
            container.x - 50.0 * scale,
            container.y + 2.0 * scale,
            scale,
        );
        for x in [
            container.x + 12.0 * scale,
            container.x + container.w - 12.0 * scale,
        ] {
            dotted(
                &mut svg,
                x,
                container.y + container.h,
                28.0 * scale,
                true,
                "#ed7099",
            );
            dialog_dimension(
                "24".into(),
                x - 8.0 * scale,
                container.y + container.h + 32.0 * scale,
                scale,
            );
        }
    } else {
        let (Some(icon), Some(divider)) =
            (values.get("icon").copied(), values.get("divider").copied())
        else {
            return;
        };
        let right = 734.0 * scale;
        for (number, x, y, target_x, target_y) in [
            (
                1,
                304.0 * scale,
                147.0 * scale,
                container.x + 8.0 * scale,
                container.y + 40.0 * scale,
            ),
            (
                2,
                500.0 * scale,
                48.0 * scale,
                icon.x + icon.w / 2.0,
                icon.y,
            ),
            (
                3,
                right,
                headline.y + headline.h / 2.0 - 16.0 * scale,
                headline.x + headline.w + 4.0 * scale,
                headline.y + headline.h / 2.0,
            ),
            (
                4,
                right,
                body.y + 12.0 * scale,
                body.x + body.w - 20.0 * scale,
                body.y + 28.0 * scale,
            ),
            (
                5,
                right,
                divider.y - 16.0 * scale,
                divider.x + divider.w + 4.0 * scale,
                divider.y,
            ),
            (
                6,
                right,
                actions.y + actions.h / 2.0 - 16.0 * scale,
                actions.x + actions.w,
                actions.y + actions.h / 2.0,
            ),
            (
                7,
                500.0 * scale,
                613.0 * scale,
                512.0 * scale,
                629.0 * scale,
            ),
        ] {
            if number == 1 {
                let start = x + 24.0 * scale;
                svg.push_str(&format!("<path d='M{start} {target_y}H{target_x}' fill='none' stroke='#49454f' stroke-width='{}'/><circle cx='{target_x}' cy='{target_y}' r='{}' fill='none' stroke='#49454f' stroke-width='{}'/>", scale, 3.5 * scale, scale));
            } else if number == 2 {
                dotted(
                    &mut svg,
                    target_x,
                    y + 32.0 * scale,
                    target_y - y - 32.0 * scale,
                    true,
                    "#49454f",
                );
            } else if number != 7 {
                dotted(&mut svg, target_x, target_y, x - target_x, false, "#49454f");
            }
            dialog_marker(number, x, y, scale);
        }
        annotations::outline(&mut svg, icon);
        annotations::outline(&mut svg, actions);
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn dialog_marker(number: usize, x: f64, y: f64, scale: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(24.0 * scale), Dp(32.0 * scale))
                .background_with_shape(super::BACKGROUND, Shape::CAPSULE)
                .border_with_shape(Dp(scale), INK, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Anatomy marker {number}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(dialog_type_style(14.0, 20.0, scale))
                .color(INK);
        });
}

#[tessera]
fn dialog_dimension(label: String, x: f64, y: f64, scale: f64) {
    text()
        .content(label.clone())
        .style(dialog_type_style(14.0, 20.0, scale))
        .color(INK)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Dimension {label}")),
                    ..Default::default()
                }),
        );
}

#[tessera]
fn dialog_touch_targets(child: RenderSlot) {
    provide_context(
        || MinimumInteractiveComponentEnforcement { enabled: false },
        move || child.render(),
    );
}

#[tessera]
fn dialog_diagram_theme(scale: f64, child: RenderSlot) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    material_theme()
        .theme(move || {
            let mut theme = theme.clone();
            for style in [
                &mut theme.typography.body_large,
                &mut theme.typography.body_small,
            ] {
                style.font_size = Dp(style.font_size.0 * scale);
                style.line_height = style.line_height.map(|height| Dp(height.0 * scale));
                style.letter_spacing = Dp(style.letter_spacing.0 * scale);
            }
            theme
        })
        .child_shared(child);
}

#[tessera]
fn dialog_overview_pair() {
    let scale = 1.0_f64;
    let notice = remember(String::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::NOTICE.set(Some(notice));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width()).children(move || {
dialog_canvas(1024.0, 622.0, RenderSlot::new(move || {
        boxed().modifier(Modifier::new().fill_max_width().height(Dp(622.0 * scale))
            .semantics(SemanticsArgs {
                role: Some(Role::Group),
                label: Some("Dialogs overview figure".into()),
                merge_descendants: true,
                ..Default::default()
            })
            .clip_to_bounds().background_with_shape(scheme.secondary_container, Shape::rounded_rectangle(Dp(24.0))))
            .children(move || {
                dialog_touch_targets(RenderSlot::new(move || {
                    dialog_diagram_theme(scale, RenderSlot::new(move || {
                        dialog_overview_badge(1, 274.5, scale);
                        dialog_overview_badge(2, 756.5, scale);
                        dialog_phone_frame(61.0, 88.0, 908.0, scale, RenderSlot::new(move || {
                            dialog_phone_status(scale);
                            spacer().modifier(Modifier::new().fill_max_size().background(scheme.scrim.with_alpha(0.32)));
                            boxed().modifier(Modifier::new().offset(Dp(50.0 * scale), Dp(214.0 * scale))
                                .size(Dp(312.0 * scale), Dp(220.0 * scale))
                                .background_with_shape(scheme.surface_container_high, Shape::rounded_rectangle(Dp(28.0 * scale)))
                                .padding_all(Dp(24.0 * scale)))
                                .children(move || {
                                    text().content("Basic dialog title").style(dialog_type_style(24.0, 32.0, scale)).color(scheme.on_surface);
                                    text().content("A dialog is a modal window that appears in front of app content to provide critical information or ask for a decision")
                                        .style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
                                        .modifier(Modifier::new().offset(Dp::ZERO, Dp(52.0 * scale)).width(Dp(264.0 * scale)));
                                    dialog_scene_action("Text button".into(), "Overview basic dismiss".into(), 40.0, 132.0, 112.0, scale,
                                        Callback::new(move || notice.set("Basic dialog dismissed".into())));
                                    dialog_scene_action("Text button".into(), "Overview basic confirm".into(), 156.0, 132.0, 112.0, scale,
                                        Callback::new(move || notice.set("Basic dialog accepted".into())));
                                });
                        }));
                        dialog_phone_frame(542.0, 88.0, 908.0, scale, RenderSlot::new(move || {
                            dialog_phone_status(scale);
                            dialog_phone_header("Full-screen dialog title".into(), scale,
                                Callback::new(move || notice.set("Full-screen dialog closed".into())),
                                Callback::new(move || notice.set("Event saved".into())));
                            dialog_scene_input(24.0, 144.0, 364.0, "Label".into(), "Input".into(), "Overview dialog category".into(), true, scale);
                            dialog_plain_input(24.0, 216.0, 364.0, "Event name".into(), "".into(), "Overview event name".into(), scale);
                            spacer().modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(295.0 * scale))
                                .size(Dp(364.0 * scale), Dp(scale)).background(scheme.outline_variant));
                            for (index, caption) in ["From", "To"].into_iter().enumerate() {
                                let y = 318.0 + index as f64 * 104.0;
                                text().content(caption).style(dialog_type_style(12.0, 16.0, scale)).color(scheme.on_surface_variant)
                                    .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(y * scale)));
                                for (column, kind) in ["date", "time"].into_iter().enumerate() {
                                    dialog_scene_input(24.0 + column as f64 * 185.0, y + 25.0, 177.0,
                                        "Label".into(), "Input".into(), format!("Overview {caption} {kind}"), true, scale);
                                }
                            }
                        }));
                    }));
                }));
            });
            }));
        if !notice.with(String::is_empty) {gap(Dp(12.0)); prose(notice.get());}
    });
}

#[tessera]
fn dialog_overview_badge(number: usize, center: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp((center - 20.0) * scale), Dp(25.0 * scale))
                .size(Dp(40.0 * scale), Dp(40.0 * scale))
                .background_with_shape(scheme.on_surface, Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Dialog variant {number}")),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(dialog_type_style(16.0, 24.0, scale))
                .color(scheme.surface);
        });
}

#[tessera]
fn dialog_phone_frame(x: f64, y: f64, height: f64, scale: f64, child: RenderSlot) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(428.0 * scale), Dp(height * scale))
                .background_with_shape(
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(28.0 * scale)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(8.0 * scale), Dp(8.0 * scale))
                        .size(Dp(412.0 * scale), Dp((height - 16.0) * scale))
                        .clip_shape(Shape::rounded_rectangle(Dp(20.0 * scale)))
                        .background(scheme.surface),
                )
                .children_shared(child);
        });
}

#[tessera]
fn dialog_phone_status(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content("9:30")
        .style(dialog_type_style(14.0, 20.0, scale))
        .color(scheme.on_surface)
        .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(23.0 * scale)));
    for (x, vector) in [
        (334.0, filled::WIFI_SVG),
        (352.0, filled::SIGNAL_CELLULAR_4_BAR_SVG),
        (370.0, filled::BATTERY_FULL_SVG),
    ] {
        boxed()
            .modifier(Modifier::new().offset(Dp(x * scale), Dp(25.0 * scale)))
            .children(move || {
                icon()
                    .vector(vector)
                    .size(Dp(16.0 * scale))
                    .tint(scheme.on_surface);
            });
    }
}

#[tessera]
fn dialog_phone_header(title: String, scale: f64, close: Callback, save: Callback) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    button()
        .text()
        .modifier(
            Modifier::new()
                .offset(Dp(8.0 * scale), Dp(56.0 * scale))
                .size(Dp(48.0 * scale), Dp(64.0 * scale)),
        )
        .content_padding(Padding::all(Dp::ZERO))
        .accessibility_label("Close overview dialog")
        .on_click_shared(close)
        .child(move || {
            icon()
                .vector(filled::CLOSE_SVG)
                .size(Dp(24.0 * scale))
                .tint(scheme.on_surface);
        });
    text()
        .content(title)
        .style(dialog_type_style(22.0, 28.0, scale))
        .color(scheme.on_surface)
        .modifier(Modifier::new().offset(Dp(56.0 * scale), Dp(74.0 * scale)));
    dialog_scene_action(
        "Save".into(),
        "Save overview event".into(),
        334.0,
        68.0,
        64.0,
        scale,
        save,
    );
}

#[tessera]
fn dialog_scene_action(
    label: String,
    name: String,
    x: f64,
    y: f64,
    width: f64,
    scale: f64,
    callback: Callback,
) {
    button()
        .text()
        .accessibility_label(name)
        .on_click_shared(callback)
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(40.0 * scale)),
        )
        .content_padding(Padding::all(Dp::ZERO))
        .child(move || {
            let mut style = dialog_type_style(14.0, 20.0, scale);
            style.font_weight = 500;
            text().content(label.clone()).style(style);
        });
}

#[tessera]
fn dialog_scene_input(
    x: f64,
    y: f64,
    width: f64,
    label: String,
    initial: String,
    name: String,
    dropdown: bool,
    scale: f64,
) {
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(initial.as_str());
        controller
    });
    let menu = remember(MenuController::new);
    let name = remember(|| name);
    let label = remember(|| label);
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(56.0 * scale)),
        )
        .children(move || {
            let content = RenderSlot::new(move || {
                let mut field = TextFieldBuilder::outlined()
                    .controller(controller)
                    .label(label.get())
                    .accessibility_label(name.get())
                    .min_width(Dp::ZERO)
                    .min_height(Dp(56.0 * scale))
                    .padding(Dp(16.0 * scale))
                    .font_size(Dp(16.0 * scale))
                    .line_height(Dp(24.0 * scale))
                    .modifier(Modifier::new().size(Dp(width * scale), Dp(56.0 * scale)));
                if dropdown {
                    field = field.trailing_icon_shared(RenderSlot::new(move || {
                        button()
                            .text()
                            .accessibility_label(format!("Choose {}", name.get()))
                            .modifier(Modifier::new().size(Dp(32.0 * scale), Dp(40.0 * scale)))
                            .content_padding(Padding::all(Dp::ZERO))
                            .on_click(move || menu.with_mut(MenuController::toggle))
                            .child(move || {
                                icon()
                                    .vector(filled::ARROW_DROP_DOWN_SVG)
                                    .size(Dp(24.0 * scale));
                            });
                    }));
                }
                drop(field);
            });
            if dropdown {
                menu_provider()
                    .controller(menu)
                    .anchor(MenuAnchor::new(
                        PxPosition::new(Dp::ZERO.into(), Dp::ZERO.into()),
                        PxSize::new(Dp(width * scale).into(), Dp(56.0 * scale).into()),
                    ))
                    .modifier(Modifier::new().size(Dp(width * scale), Dp(56.0 * scale)))
                    .main_content_shared(content)
                    .menu_content(move || {
                        for value in ["Input", "Option 2", "Option 3"] {
                            menu_item().label(value).on_click(move || {
                                controller.with_mut(|controller| controller.set_text(value))
                            });
                        }
                    });
            } else {
                content.render();
            }
        });
}

#[tessera]
fn basic_color_figure() {
    let scale = 1.0_f64;
    let geometry = remember(Geometry::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::GEOMETRY.set(Some(geometry));
    let notice = remember(String::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::NOTICE.set(Some(notice));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width()).children(move || {
dialog_canvas(1024.0, 486.0, RenderSlot::new(move || {
        boxed().modifier(watch(geometry, "figure").fill_max_width().height(Dp(486.0 * scale))
            .background_with_shape(scheme.surface_container_low, Shape::rounded_rectangle(Dp(24.0))))
            .children(move || {
                spacer().modifier(Modifier::new().fill_max_size().background_with_shape(scheme.scrim.with_alpha(0.32), Shape::rounded_rectangle(Dp(24.0))));
                boxed().modifier(watch(geometry, "container").offset(Dp(341.0 * scale), Dp(114.0 * scale))
                    .size(Dp(340.0 * scale), Dp(264.0 * scale))
                    .background_with_shape(scheme.surface_container_high, Shape::rounded_rectangle(Dp(28.0 * scale)))
                    .padding_all(Dp(24.0 * scale)))
                    .children(move || {
                        boxed().modifier(watch(geometry, "icon").offset(Dp(134.0 * scale), Dp::ZERO).size(Dp(24.0 * scale), Dp(24.0 * scale)))
                            .children(move || {
                                icon().vector(filled::PHONE_ANDROID_SVG).size(Dp(24.0 * scale)).tint(scheme.secondary);
                                boxed().modifier(Modifier::new().offset(Dp(6.0 * scale), Dp(6.0 * scale))).children(move || {
                                    icon().vector(filled::DONE_SVG).size(Dp(12.0 * scale)).tint(scheme.secondary);
                                });
                            });
                        text().content("Dialog with hero icon").style(dialog_type_style(24.0, 32.0, scale)).color(scheme.on_surface)
                            .modifier(watch(geometry, "headline").offset(Dp::ZERO, Dp(40.0 * scale)).align(Alignment::TopCenter));
                        text().content("A dialog is a modal window that appears in front of app content to provide critical information or ask for a decision.")
                            .style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
                            .modifier(watch(geometry, "body").offset(Dp::ZERO, Dp(88.0 * scale)).width(Dp(292.0 * scale)));
                        dialog_touch_targets(RenderSlot::new(move || {
                            boxed().modifier(watch(geometry, "actions").offset(Dp(116.0 * scale), Dp(176.0 * scale)).size(Dp(176.0 * scale), Dp(40.0 * scale)))
                                .children(move || {
                                    dialog_scene_action("Cancel".into(), "Cancel color dialog".into(), 0.0, 0.0, 80.0, scale,
                                        Callback::new(move || notice.set("Color dialog dismissed".into())));
                                    dialog_scene_action("Accept".into(), "Accept color dialog".into(), 88.0, 0.0, 88.0, scale,
                                        Callback::new(move || notice.set("Color dialog accepted".into())));
                                });
                        }));
                    });
                basic_color_annotations(geometry, scale);
            });
            }));
        if !notice.with(String::is_empty) {gap(Dp(12.0)); prose(notice.get());}
        gap(Dp(24.0));
        for (index, role) in ["Surface container high", "Secondary", "On surface", "On surface variant", "Primary", "Scrim"].into_iter().enumerate() {
            prose(format!("{}  {role}", index + 1));
            gap(Dp(12.0));
        }
    });
}

#[tessera]
fn basic_color_annotations(geometry: State<Geometry>, scale: f64) {
    basic_color_annotation_content(geometry.with(annotations::relative_precise_geometry), scale);
}

#[tessera]
fn basic_color_annotation_content(values: BTreeMap<&'static str, Rect>, scale: f64) {
    let (Some(root), Some(container), Some(icon), Some(headline), Some(body), Some(actions)) = (
        values.get("figure"),
        values.get("container").copied(),
        values.get("icon").copied(),
        values.get("headline").copied(),
        values.get("body").copied(),
        values.get("actions").copied(),
    ) else {
        return;
    };
    let mut svg = String::new();
    for (number, role, x, y, tx, ty) in [
        (
            1,
            "Surface container high",
            270.0,
            159.0,
            container.x + 12.0 * scale,
            container.y + 61.0 * scale,
        ),
        (2, "Secondary", 499.0, 42.0, icon.x + icon.w / 2.0, icon.y),
        (
            3,
            "On surface",
            727.0,
            179.0,
            headline.x + headline.w,
            headline.y + headline.h / 2.0,
        ),
        (
            4,
            "On surface variant",
            727.0,
            238.0,
            body.x + body.w,
            body.y + body.h / 2.0,
        ),
        (
            5,
            "Primary",
            727.0,
            315.0,
            actions.x + actions.w,
            actions.y + actions.h / 2.0,
        ),
        (6, "Scrim", 497.0, 407.0, 509.0 * scale, 423.0 * scale),
    ] {
        let x = x * scale;
        let y = y * scale;
        if number == 1 {
            svg.push_str(&format!("<path d='M{} {ty}H{tx}' stroke='#49454f' stroke-width='{scale}'/><circle cx='{tx}' cy='{ty}' r='{}' fill='none' stroke='#49454f' stroke-width='{scale}'/>", x + 24.0 * scale, 3.5 * scale));
        } else if number == 2 {
            dotted(
                &mut svg,
                tx,
                y + 32.0 * scale,
                ty - y - 32.0 * scale,
                true,
                "#49454f",
            );
        } else if number != 6 {
            dotted(&mut svg, tx, ty, x - tx, false, "#49454f");
        }
        dialog_color_marker(number, role.into(), x, y, scale);
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn dialog_color_marker(number: usize, role: String, x: f64, y: f64, scale: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(24.0 * scale), Dp(32.0 * scale))
                .background_with_shape(super::BACKGROUND, Shape::CAPSULE)
                .border_with_shape(Dp(scale), INK, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Color role {number}: {role}")),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(dialog_type_style(14.0, 20.0, scale))
                .color(INK);
        });
}

#[tessera]
fn dialog_plain_input(
    x: f64,
    y: f64,
    width: f64,
    placeholder: String,
    initial: String,
    name: String,
    scale: f64,
) {
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(initial.as_str());
        controller
    });
    TextFieldBuilder::outlined()
        .controller(controller)
        .placeholder(placeholder)
        .accessibility_label(name)
        .min_width(Dp::ZERO)
        .min_height(Dp(56.0 * scale))
        .padding(Dp(16.0 * scale))
        .font_size(Dp(16.0 * scale))
        .line_height(Dp(24.0 * scale))
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(56.0 * scale)),
        );
}

#[tessera]
fn full_spec_figure(mode: usize) {
    let scale = 1.0_f64;
    let geometry = remember(Geometry::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::GEOMETRY.set(Some(geometry));
    let notice = remember(String::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    canonical_tests::NOTICE.set(Some(notice));
    let all_day = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let y = [103.0, 98.0, 67.0][mode];
    dialog_canvas(
        1024.0,
        [463.0, 508.0, 1030.0][mode],
        RenderSlot::new(move || {
            boxed().modifier(watch(geometry, "figure").fill_max_width()
        .height(Dp([463.0, 508.0, 1030.0][mode] * scale)).clip_to_bounds()
        .background_with_shape(scheme.secondary_container, Shape::rounded_rectangle(Dp(24.0))))
        .children(move || {
            dialog_touch_targets(RenderSlot::new(move || {
                dialog_diagram_theme(scale, RenderSlot::new(move || {
                    boxed().modifier(watch(geometry, "container").offset(Dp(298.0 * scale), Dp(y * scale))
                        .size(Dp(428.0 * scale), Dp(909.0 * scale)))
                        .children(move || {
                            dialog_phone_frame(0.0, 0.0, 909.0, scale, RenderSlot::new(move || {
                                spacer().modifier(Modifier::new().fill_max_size().background(scheme.surface_container_high));
                                dialog_phone_status(scale);
                                let header_y = if mode == 2 {52.0} else {60.0};
                                full_spec_header(if mode == 0 {"New event"} else {"Full-screen dialog title"}.into(),
                                    geometry, header_y, if mode == 2 {64.0} else {56.0}, scale,
                                    Callback::new(move || notice.set("Full-screen dialog closed".into())),
                                    Callback::new(move || notice.set("Event saved".into())));
                                let first_y = if mode == 2 {144.0} else {134.0};
                                let x = if mode == 0 {23.0} else {24.0};
                                dialog_scene_input(x, first_y, 364.0, "Label".into(), "Input".into(), format!("Full spec {mode} category"), true, scale);
                                dialog_plain_input(x, first_y + 72.0, 364.0, "Event name".into(),
                                    match mode {0 => "Carmen’s Birthday Surprise Party", 1 => "Mamá’s birthday", _ => ""}.into(),
                                    format!("Full spec {mode} event name"), scale);
                                if mode == 1 {
                                    text().content("With mamá’s birthday around the corner, I’d like to start planning location and food. I can host en mi casa if you all help with the food. Since we all know how much she loves to eat maybe we can cook together the day before? Should")
                                        .style(dialog_type_style(16.0, 24.0, scale)).color(scheme.on_surface_variant)
                                        .modifier(watch(geometry, "body").offset(Dp(24.0 * scale), Dp(280.0 * scale)).width(Dp(364.0 * scale)));
                                } else {
                                    spacer().modifier(watch(geometry, "divider").offset(Dp(if mode == 0 {27.0} else {24.0} * scale), Dp(if mode == 0 {290.0} else {295.0} * scale))
                                        .size(Dp(if mode == 0 {358.0} else {364.0} * scale), Dp(scale)).background(scheme.outline_variant));
                                    for (row, caption) in ["From", "To"].into_iter().enumerate() {
                                        if mode == 0 && row == 1 {continue;}
                                        let top = if mode == 0 {318.0} else {316.0 + row as f64 * 104.0};
                                        text().content(caption).style(dialog_type_style(12.0, 16.0, scale)).color(scheme.on_surface_variant)
                                            .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(top * scale)));
                                        for column in 0..2 {
                                            dialog_scene_input(24.0 + column as f64 * 185.0,
                                                if mode == 0 {343.0} else if row == 0 {344.0} else {447.0},
                                                177.0, "Label".into(),
                                                if row == 0 && column == 0 {"Input"} else {"All day"}.into(),
                                                format!("Full spec {mode} {caption} {}", if column == 0 {"date"} else {"time"}), true, scale);
                                        }
                                    }
                                    if mode == 2 {
                                        checkbox().checked(all_day.get()).size(Dp(18.0 * scale))
                                            .on_toggle(move |next| all_day.set(next)).accessibility_label("Measurement all day")
                                            .modifier(Modifier::new().offset(Dp(28.0 * scale), Dp(513.0 * scale)).size(Dp(48.0 * scale), Dp(48.0 * scale)));
                                        text().content("All day").style(dialog_type_style(16.0, 24.0, scale)).color(scheme.on_surface_variant)
                                            .modifier(Modifier::new().offset(Dp(80.0 * scale), Dp(525.0 * scale)));
                                        text().content("Timezone").style(dialog_type_style(12.0, 16.0, scale)).color(scheme.on_surface_variant)
                                            .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(572.0 * scale)));
                                        dialog_scene_input(24.0, 599.0, 364.0, "Label".into(), "Input".into(), "Measurement timezone".into(), true, scale);
                                        spacer().modifier(Modifier::new().offset(Dp(152.0 * scale), Dp(878.0 * scale)).size(Dp(108.0 * scale), Dp(4.0 * scale))
                                            .background_with_shape(scheme.on_surface_variant, Shape::CAPSULE));
                                    }
                                }
                            }));
                        });
                    full_spec_annotations(geometry, mode, scale);
                }));
            }));
        });
        }),
    );
    if !notice.with(String::is_empty) {
        gap(Dp(12.0));
        prose(notice.get());
    }
}

#[tessera]
fn full_spec_header(
    title: String,
    geometry: State<Geometry>,
    y: f64,
    height: f64,
    scale: f64,
    close: Callback,
    save: Callback,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let center = if height == 56.0 { 24.0 } else { 32.0 };
    boxed()
        .modifier(
            watch(geometry, "header")
                .offset(Dp::ZERO, Dp(y * scale))
                .size(Dp(412.0 * scale), Dp(height * scale)),
        )
        .children(move || {
            button()
                .text()
                .accessibility_label("Close spec dialog")
                .on_click_shared(close)
                .modifier(
                    watch(geometry, "close")
                        .offset(Dp(8.0 * scale), Dp((center - 20.0) * scale))
                        .size(Dp(40.0 * scale), Dp(40.0 * scale)),
                )
                .content_padding(Padding::all(Dp::ZERO))
                .child(move || {
                    icon()
                        .vector(filled::CLOSE_SVG)
                        .size(Dp(24.0 * scale))
                        .tint(scheme.on_surface);
                });
            text()
                .content(title.clone())
                .style(dialog_type_style(22.0, 28.0, scale))
                .color(scheme.on_surface)
                .modifier(
                    watch(geometry, "headline")
                        .offset(Dp(56.0 * scale), Dp((center - 14.0) * scale))
                        .width(Dp(270.0 * scale)),
                );
            boxed()
                .modifier(
                    watch(geometry, "confirm")
                        .offset(Dp(336.0 * scale), Dp((center - 20.0) * scale))
                        .size(Dp(64.0 * scale), Dp(40.0 * scale)),
                )
                .children(move || {
                    dialog_scene_action(
                        "Save".into(),
                        "Save spec event".into(),
                        0.0,
                        0.0,
                        64.0,
                        scale,
                        save,
                    );
                });
        });
}

#[tessera]
fn full_spec_annotations(geometry: State<Geometry>, mode: usize, scale: f64) {
    full_spec_annotation_content(
        geometry.with(annotations::relative_precise_geometry),
        mode,
        scale,
    );
}

#[tessera]
fn full_spec_annotation_content(values: BTreeMap<&'static str, Rect>, mode: usize, scale: f64) {
    let (Some(root), Some(container), Some(header), Some(close), Some(headline), Some(confirm)) = (
        values.get("figure"),
        values.get("container").copied(),
        values.get("header").copied(),
        values.get("close").copied(),
        values.get("headline").copied(),
        values.get("confirm").copied(),
    ) else {
        return;
    };
    let mut svg = String::new();
    if mode == 2 {
        for rect in [
            Rect {
                x: header.x,
                y: header.y,
                w: 16.0 * scale,
                h: header.h,
            },
            Rect {
                x: header.x + 40.0 * scale,
                y: header.y,
                w: 16.0 * scale,
                h: header.h,
            },
            Rect {
                x: header.x,
                y: 435.0 * scale,
                w: 24.0 * scale,
                h: 24.0 * scale,
            },
            Rect {
                x: header.x + header.w - 24.0 * scale,
                y: 435.0 * scale,
                w: 24.0 * scale,
                h: 24.0 * scale,
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#ffcde1' fill-opacity='.85'/>",
                rect.x, rect.y, rect.w, rect.h
            ));
        }
        for (label, x, y, tx, ty) in [
            (
                "16",
                header.x - 48.0 * scale,
                header.y - 16.0 * scale,
                close.x,
                header.y - 8.0 * scale,
            ),
            (
                "24",
                header.x - 48.0 * scale,
                close.y + close.h / 2.0 - 10.0 * scale,
                close.x,
                close.y + close.h / 2.0,
            ),
        ] {
            dotted(
                &mut svg,
                x + 20.0 * scale,
                ty,
                tx - x - 20.0 * scale,
                false,
                "#ed7099",
            );
            dialog_dimension(label.into(), x, y, scale);
        }
        svg.push_str(&format!("<path d='M{} {}h{}M{} {}v{}M{} {}h{}' fill='none' stroke='#ed7099' stroke-width='{}'/>",
            header.x + header.w + 12.0 * scale, header.y, 8.0 * scale,
            header.x + header.w + 16.0 * scale, header.y, header.h,
            header.x + header.w + 12.0 * scale, header.y + header.h, 8.0 * scale, scale));
        dialog_dimension(
            "64".into(),
            header.x + header.w + 24.0 * scale,
            header.y + header.h / 2.0 - 10.0 * scale,
            scale,
        );
        for x in [header.x, header.x + header.w - 24.0 * scale] {
            let y = 475.0 * scale;
            svg.push_str(&format!(
                "<path d='M{x} {y}h{}' stroke='#ed7099' stroke-width='{scale}'/>",
                24.0 * scale
            ));
            dialog_dimension("24".into(), x + 4.0 * scale, y + 8.0 * scale, scale);
        }
    } else {
        let entries = if mode == 0 {
            vec![
                (
                    1,
                    "Container",
                    244.0,
                    254.0,
                    container.x + 8.0 * scale,
                    270.0 * scale,
                ),
                (
                    2,
                    "Header",
                    244.0,
                    181.0,
                    header.x,
                    header.y + header.h / 2.0,
                ),
                (
                    3,
                    "Close affordance",
                    322.0,
                    44.0,
                    close.x + close.w / 2.0,
                    close.y,
                ),
                (
                    4,
                    "Headline",
                    455.0,
                    44.0,
                    headline.x + 105.0 * scale,
                    headline.y,
                ),
                (
                    5,
                    "Text button",
                    662.0,
                    44.0,
                    confirm.x + confirm.w / 2.0,
                    confirm.y,
                ),
                (6, "Divider", 747.0, 385.0, 705.0 * scale, 401.0 * scale),
            ]
        } else {
            vec![
                (
                    1,
                    "Surface container high",
                    236.0,
                    339.0,
                    container.x + 6.0 * scale,
                    355.0 * scale,
                ),
                (
                    2,
                    "On surface",
                    236.0,
                    174.0,
                    close.x,
                    close.y + close.h / 2.0,
                ),
                (
                    3,
                    "On surface",
                    418.0,
                    33.0,
                    headline.x + 68.0 * scale,
                    headline.y,
                ),
                (
                    4,
                    "Primary",
                    658.0,
                    33.0,
                    confirm.x + confirm.w / 2.0,
                    confirm.y,
                ),
                (
                    5,
                    "On surface variant",
                    765.0,
                    419.0,
                    700.0 * scale,
                    435.0 * scale,
                ),
            ]
        };
        for (number, role, x, y, tx, ty) in entries {
            let x = x * scale;
            let y = y * scale;
            if number == 1 {
                svg.push_str(&format!("<path d='M{} {ty}H{tx}' stroke='#49454f' stroke-width='{scale}'/><circle cx='{tx}' cy='{ty}' r='{}' fill='none' stroke='#49454f'/>", x + 24.0 * scale, 3.5 * scale));
            } else if y < ty && x > 300.0 * scale && x < 720.0 * scale {
                dotted(
                    &mut svg,
                    tx,
                    y + 32.0 * scale,
                    ty - y - 32.0 * scale,
                    true,
                    "#49454f",
                );
            } else {
                let start = if tx < x { x } else { x + 24.0 * scale };
                dotted(&mut svg, start, ty, tx - start, false, "#49454f");
            }
            if mode == 0 {
                dialog_marker(number, x, y, scale);
            } else {
                dialog_color_marker(number, role.into(), x, y, scale);
            }
        }
        if mode == 0 {
            for rect in [
                header,
                Rect {
                    x: close.x + 8.0 * scale,
                    y: close.y + 8.0 * scale,
                    w: 24.0 * scale,
                    h: 24.0 * scale,
                },
                headline,
                confirm,
            ] {
                annotations::outline(&mut svg, rect);
            }
        }
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn full_specs() {
    column().modifier(Modifier::new().fill_max_width()).children(|| {
        full_spec_figure(0);
        gap(Dp(24.0));
        prose("1  Container   2  Header   3  Close affordance   4  Headline   5  Text button   6  Divider");
        gap(Dp(56.0));
        subheading("Full-screen dialog color".into());
        gap(Dp(24.0));
        full_spec_figure(1);
        gap(Dp(24.0));
        for (index, role) in ["Surface container high", "On surface", "On surface", "Primary", "On surface variant"].into_iter().enumerate() {
            prose(format!("{}  {role}", index + 1));
            gap(Dp(12.0));
        }
        gap(Dp(44.0));
        subheading("Full-screen dialog measurements".into());
        gap(Dp(24.0));
        full_spec_figure(2);
        gap(Dp(24.0));
        spec_table(vec!["Attribute".into(), "Value".into()], vec![
            vec!["Container shape".into(), "0dp corner radius".into()],
            vec!["Container width".into(), "Container width; Max 560dp".into()],
            vec!["Header height".into(), "64dp".into()],
            vec!["Close icon".into(), "24dp".into()],
            vec!["Content padding".into(), "24dp".into()],
            vec!["Padding between elements".into(), "8dp".into()],
        ]);
    });
}

#[tessera]
fn full_figure() {
    let visible = remember(|| true);
    let saved = remember(|| false);
    let draft = remember(EventDraft::default);
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
                            .content(move || {
                                event_form(draft);
                            });
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
fn event_form(draft: State<EventDraft>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            text_field()
                .label("Event name")
                .initial_text(draft.with(|value| value.name.clone()))
                .accessibility_label("Event name")
                .on_change(move |next: String| {
                    draft.with_mut(|value| value.name = next.clone());
                    next
                })
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(24.0));
            text_field()
                .label("From")
                .initial_text(draft.with(|value| value.from.clone()))
                .accessibility_label("From")
                .on_change(move |next: String| {
                    draft.with_mut(|value| value.from = next.clone());
                    next
                })
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(16.0));
            text_field()
                .label("To")
                .initial_text(draft.with(|value| value.to.clone()))
                .accessibility_label("To")
                .on_change(move |next: String| {
                    draft.with_mut(|value| value.to = next.clone());
                    next
                })
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(16.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    checkbox()
                        .checked(draft.with(|value| value.all_day))
                        .on_toggle(move |next| draft.with_mut(|value| value.all_day = next))
                        .accessibility_label("All-day");
                    text().content("All-day");
                });
            gap(Dp(16.0));
            prose("Time zone");
            text_field()
                .label("Time zone")
                .initial_text(draft.with(|value| value.time_zone.clone()))
                .accessibility_label("Time zone")
                .on_change(move |next: String| {
                    draft.with_mut(|value| value.time_zone = next.clone());
                    next
                })
                .modifier(Modifier::new().fill_max_width());
        });
}

#[tessera]
fn dialog_canvas(canonical_width: f64, canonical_height: f64, child: RenderSlot) {
    let available_width = remember(move || Dp(canonical_width));
    let scale = (available_width.get().0 / canonical_width) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .modifier(
                        Modifier::new().required_size(Dp(canonical_width), Dp(canonical_height)),
                    )
                    .children_shared(child);
            });
        });
}
