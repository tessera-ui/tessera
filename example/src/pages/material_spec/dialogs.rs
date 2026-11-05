//! Dialog specification figures and modal interactions.
//!
//! ## Usage
//!
//! Inspect dialog anatomy, spacing and focus behavior.

use std::collections::BTreeMap;

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
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    text_field::text_field,
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Color, Dp, LayoutCoordinates, Modifier, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role, modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

use super::{
    INK,
    annotations::{self, PreciseGeometry as Geometry, Rect, dotted, watch_precise as watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[path = "dialogs_accessibility.rs"]
mod accessibility;

#[path = "dialogs_tokens.rs"]
mod token_sets;

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
            from: "ui-mon-may-18".into(),
            to: "ui-mon-may-18".into(),
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
pub(super) fn dialogs_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    let open = remember(|| false);
    let full = remember(|| false);
    let saved = remember(|| false);
    let draft = remember(EventDraft::default);
    let discard = remember(|| false);
    let close = Callback::new(move || request_dialog_close(open, full, discard, draft));
    dialog_provider()
        .role(if full.get() {
            DialogRole::Dialog
        } else {
            DialogRole::AlertDialog
        })
        .is_open(open.get())
        .full_screen(full.get())
        .accessibility_label(I18N.text(if full.get() {
            "ui-new-event"
        } else {
            "ui-reset-settings"
        }))
        .on_close_request_shared(close)
        .main_content(move || {
            article(
                "ui-dialogs",
                "ui-dialogs-provide-important-prompts-in-a-user-flow",
                vec![
                    SpecSection {
                        title: "ui-tokens-specs",
                        content: RenderSlot::new(token_sets::tokens),
                    },
                    SpecSection {
                        title: "ui-basic-dialogs",
                        content: RenderSlot::new(basic_specs),
                    },
                    SpecSection {
                        title: "ui-full-screen-dialogs",
                        content: RenderSlot::new(full_specs),
                    },
                ],
            ).route_optional(route.clone())
            .pages(vec![
                ArticlePage {
                    title: "ui-overview",
                    intro: Some(RenderSlot::new(move || {
                        overview_intro(open, full, saved);
                    })),
                    sections: vec![
                        SpecSection {
                            title: "ui-availability-resources",
                            content: RenderSlot::new(availability),
                        },
                        SpecSection {
                            title: "ui-differences-from-m2",
                            content: RenderSlot::new(differences),
                        },
                    ],
                },
                ArticlePage {
                    title: "ui-guidelines",
                    intro: Some(RenderSlot::new(guideline_hero)),
                    sections: vec![
                        SpecSection {
                            title: "ui-usage",
                            content: RenderSlot::new(usage_section),
                        },
                        SpecSection {
                            title: "ui-anatomy",
                            content: RenderSlot::new(anatomy_section),
                        },
                        SpecSection {
                            title: "ui-basic-dialog",
                            content: RenderSlot::new(basic_guidance),
                        },
                        SpecSection {
                            title: "ui-full-screen-dialog",
                            content: RenderSlot::new(full_guidance),
                        },
                        SpecSection {
                            title: "ui-adaptive-design",
                            content: RenderSlot::new(adaptive_section),
                        },
                        SpecSection {
                            title: "ui-behavior",
                            content: RenderSlot::new(move || {
                                behavior_section(open, full, saved);
                            }),
                        },
                    ],
                },
                ArticlePage {
                    title: "ui-accessibility",
                    intro: None,
                    sections: vec![
                        SpecSection {
                            title: "ui-use-cases",
                            content: RenderSlot::new(move || {
                                use_cases(open, full, saved);
                            }),
                        },
                        SpecSection {
                            title: "ui-interaction-style",
                            content: RenderSlot::new(interaction_style_section),
                        },
                        SpecSection {
                            title: "ui-initial-focus",
                            content: RenderSlot::new(initial_focus_section),
                        },
                        SpecSection {
                            title: "ui-keyboard-navigation",
                            content: RenderSlot::new(keyboard_section),
                        },
                        SpecSection {
                            title: "ui-labeling-elements",
                            content: RenderSlot::new(labeling_section),
                        },
                    ],
                },
            ]);
        })
        .dialog_content(move |dialog_scope| {
            if full.get() {
                event_dialog(draft, open, saved, discard, close).dialog(dialog_scope);
            } else {
                basic_dialog().dialog(dialog_scope)
                    .headline(I18N.text("ui-reset-settings"))
                    .supporting_text(
                        I18N.text("ui-this-will-reset-your-app-preferences-back-to-their-default-settings-the-following-accounts"),
                    )
                    .dismiss_button(move || {
                        action("ui-cancel".into(), Callback::new(move || open.set(false)));
                    })
                    .confirm_button(move || {
                        action(
                            "ui-accept".into(),
                            Callback::new(move || {
                                saved.set(true);
                                open.set(false);
                            }),
                        );
                    });
            }
        });
}

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const CAUTION_COLOR: Color = Color::from_rgb(0.659, 0.373, 0.0);

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text()
                .content(format!("•  {}", I18N.text(item)))
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn guidance_line(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .padding_symmetric(Dp(12.0), Dp(4.0))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text()
                        .content(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(I18N.text(&(note)))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0))
                .modifier(Modifier::new().weight(1.0));
        });
}

#[tessera]
fn legend(items: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().children(move || {
        for (index, item) in items.clone().into_iter().enumerate() {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .size(Dp(24.0), Dp(32.0))
                                .background_with_shape(
                                    scheme.on_surface,
                                    Shape::rounded_rectangle(Dp(12.0)),
                                ),
                        )
                        .children(move || {
                            text()
                                .content((index + 1).to_string())
                                .style(type_style(12.0, 16.0))
                                .color(scheme.surface);
                        });
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    text()
                        .content(I18N.text(&(item)))
                        .style(type_style(14.0, 20.0))
                        .color(scheme.on_surface_variant);
                });
            gap(Dp(6.0));
        }
    });
}

#[tessera]
fn do_dont(#[prop(into)] good: String, #[prop(into)] bad: String) {
    column().children(move || {
        guidance_line("Do", good.clone(), DO_COLOR);
        gap(Dp(12.0));
        guidance_line("Don't", bad.clone(), DONT_COLOR);
    });
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Verdict {
    #[default]
    Note,
    Do,
    Dont,
    Caution,
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Backdrop {
    #[default]
    Photos,
    Snackbar,
    Product,
    Surface,
}

fn labels(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn product_data() -> ImageData {
    include_bytes!("../../../assets/material_spec/dialog-guideline-product.png")
        .as_slice()
        .try_into_image_data()
        .expect("dialog product example")
}

#[tessera]
fn verdict_band(verdict: Verdict) {
    let (color, vector, label) = match verdict {
        Verdict::Do => (DO_COLOR, filled::CHECK_CIRCLE_SVG, "Do"),
        Verdict::Dont => (DONT_COLOR, filled::CANCEL_SVG, "Don't"),
        Verdict::Caution => (CAUTION_COLOR, filled::WARNING_SVG, "ui-caution"),
        Verdict::Note => (INK, filled::INFO_SVG, "ui-note"),
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(4.0))
                    .background_with_shape(color, Shape::rounded_rectangle(Dp(2.0))),
            );
            gap(Dp(12.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    icon().vector(vector).tint(color).size(Dp(20.0));
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    text()
                        .content(I18N.text(label))
                        .color(color)
                        .style(type_style(16.0, 24.0));
                });
        });
}

#[tessera]
fn guidance_card(
    verdict: Verdict,
    #[prop(into)] note: String,
    scene: RenderSlot,
    wide: Option<bool>,
) {
    let canonical_width = if wide.unwrap_or(false) { 720.0 } else { 400.0 };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let scene = scene.clone();
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
                .children(move || {
                    dialog_canvas(canonical_width, 320.0, scene.clone());
                });
            if verdict != Verdict::Note {
                gap(Dp(4.0));
                verdict_band(verdict);
            }
            gap(Dp(16.0));
            caption(note.clone());
        });
}

#[tessera]
fn guidance_pair(
    first_verdict: Verdict,
    #[prop(into)] first_note: String,
    first: RenderSlot,
    second_verdict: Verdict,
    #[prop(into)] second_note: String,
    second: RenderSlot,
) {
    let width = remember(|| Dp(1024.0));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            let (first, second) = (first.clone(), second.clone());
            let (first_note, second_note) = (first_note.clone(), second_note.clone());
            if width.get().0 < 640.0 {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        guidance_card(first_verdict, first_note.clone(), first.clone());
                        gap(Dp(32.0));
                        guidance_card(second_verdict, second_note.clone(), second.clone());
                    });
            } else {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        let (first, second) = (first.clone(), second.clone());
                        let (first_note, second_note) = (first_note.clone(), second_note.clone());
                        boxed()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                guidance_card(first_verdict, first_note.clone(), first.clone());
                            });
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        boxed()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                guidance_card(second_verdict, second_note.clone(), second.clone());
                            });
                    });
            }
        });
}

#[tessera]
fn guide_marker() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(20.0), Dp(20.0))
                .background_with_shape(scheme.on_surface, Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(I18N.text("ui-marker-1")),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content("1")
                .style(type_style(12.0, 16.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn mock_action(label: String, disabled: bool, marked: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let color = if disabled {
        scheme.on_surface.with_alpha(0.38)
    } else {
        scheme.primary
    };
    let outline = if marked {
        scheme.on_surface
    } else {
        Color::TRANSPARENT
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .height(Dp(40.0))
                .border_with_shape(Dp(1.0), outline, Shape::rounded_rectangle(Dp(20.0)))
                .padding_symmetric(Dp(12.0), Dp(0.0)),
        )
        .children(move || {
            if marked {
                guide_marker();
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            text()
                .content(I18N.text(&(label)))
                .style(type_style(14.0, 20.0))
                .color(color);
        });
}

#[tessera]
fn mock_dialog(
    #[prop(into)] headline: String,
    width: f64,
    actions: Vec<String>,
    #[prop(into)] body: Option<String>,
    disabled: Option<usize>,
    marked: Option<usize>,
    stacked: Option<bool>,
    content: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let stacked = stacked.unwrap_or(false);
    let label = I18N.format(
        "ui-example-dialog-value",
        [("v0", I18N.text(&headline).into())],
    );
    column()
        .modifier(
            Modifier::new()
                .width(Dp(width))
                .background_with_shape(
                    scheme.surface_container_high,
                    Shape::rounded_rectangle(Dp(28.0)),
                )
                .padding_all(Dp(24.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(I18N.text(label)),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            let (actions, content) = (actions.clone(), content.clone());
            text()
                .content(I18N.text(&(headline)))
                .style(type_style(24.0, 32.0))
                .color(scheme.on_surface);
            if let Some(body) = body.clone() {
                gap(Dp(16.0));
                text()
                    .content(I18N.text(body))
                    .style(type_style(14.0, 20.0))
                    .color(scheme.on_surface_variant);
            }
            if let Some(content) = content {
                gap(Dp(16.0));
                content.render();
            }
            gap(Dp(24.0));
            if stacked {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::End)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        for (index, action) in actions.clone().into_iter().enumerate() {
                            if index > 0 {
                                gap(Dp(8.0));
                            }
                            mock_action(action, disabled == Some(index), marked == Some(index));
                        }
                    });
            } else {
                row()
                    .main_axis_alignment(MainAxisAlignment::End)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        for (index, action) in actions.clone().into_iter().enumerate() {
                            if index > 0 {
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                            }
                            mock_action(action, disabled == Some(index), marked == Some(index));
                        }
                    });
            }
        });
}

#[tessera]
fn scrimmed(dialog: RenderSlot) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            spacer().modifier(
                Modifier::new()
                    .fill_max_size()
                    .background(scheme.scrim.with_alpha(0.32)),
            );
            dialog.render();
        });
}

#[tessera]
fn placeholder_lines() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.surface)
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            for width in [200.0, 320.0, 280.0, 300.0, 240.0, 320.0, 180.0] {
                spacer().modifier(
                    Modifier::new()
                        .size(Dp(width), Dp(12.0))
                        .background_with_shape(
                            scheme.outline_variant,
                            Shape::rounded_rectangle(Dp(6.0)),
                        ),
                );
                gap(Dp(20.0));
            }
        });
}

#[tessera]
fn backdrop_scene(backdrop: Backdrop, dialog: Option<RenderSlot>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let dialog = dialog.unwrap_or_else(RenderSlot::empty);
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_size()
                .clip_to_bounds()
                .background(scheme.surface_container_high),
        )
        .children(move || {
            let dialog = dialog.clone();
            if backdrop == Backdrop::Product {
                image()
                    .raster(remember(product_data).get())
                    .modifier(Modifier::new().required_size(Dp(400.0), Dp(325.0)));
            } else if backdrop == Backdrop::Surface {
                placeholder_lines();
                scrimmed(dialog);
            } else {
                boxed()
                    .alignment(if backdrop == Backdrop::Snackbar {
                        Alignment::BottomCenter
                    } else {
                        Alignment::Center
                    })
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        image()
                            .raster(remember(accessibility::photo_data).get())
                            .modifier(Modifier::new().required_size(Dp(720.0), Dp(639.0)));
                    });
                if backdrop == Backdrop::Photos {
                    scrimmed(dialog);
                }
            }
        });
}

#[tessera]
fn radio_options(selected: Option<usize>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, label) in ["ui-callisto", "ui-ganymede", "ui-luna"]
                .into_iter()
                .enumerate()
            {
                let chosen = selected == Some(index);
                let ring = if chosen {
                    scheme.primary
                } else {
                    scheme.on_surface_variant
                };
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(40.0)))
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().size(Dp(20.0), Dp(20.0)).border_with_shape(
                                Dp(2.0),
                                ring,
                                Shape::Ellipse,
                            ))
                            .children(move || {
                                if chosen {
                                    spacer().modifier(
                                        Modifier::new()
                                            .size(Dp(10.0), Dp(10.0))
                                            .background_with_shape(scheme.primary, Shape::Ellipse),
                                    );
                                }
                            });
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text()
                            .content(I18N.text(label))
                            .style(type_style(16.0, 24.0))
                            .color(scheme.on_surface);
                    });
            }
        });
}

#[tessera]
fn full_screen_scene(
    #[prop(into)] title: String,
    #[prop(into)] action: String,
    #[prop(into)] headline: Option<String>,
    marked: Option<bool>,
    overlay: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let marked = marked.unwrap_or(false);
    let mark_title = marked && headline.is_none();
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .clip_to_bounds()
                .background(scheme.surface),
        )
        .children(move || {
            let (title, action, headline) = (title.clone(), action.clone(), headline.clone());
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    let (title, action, headline) =
                        (title.clone(), action.clone(), headline.clone());
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(56.0))
                                .padding_symmetric(Dp(16.0), Dp(0.0)),
                        )
                        .children(move || {
                            icon()
                                .vector(filled::CLOSE_SVG)
                                .size(Dp(24.0))
                                .tint(scheme.on_surface);
                            spacer().modifier(Modifier::new().width(Dp(16.0)));
                            text()
                                .content(I18N.text(&(title)))
                                .style(type_style(22.0, 28.0))
                                .color(scheme.on_surface);
                            if mark_title {
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                                guide_marker();
                            }
                            spacer().modifier(Modifier::new().weight(1.0));
                            text()
                                .content(I18N.text(&(action)))
                                .style(type_style(14.0, 20.0))
                                .color(scheme.primary);
                        });
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            if let Some(headline) = headline.clone() {
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .modifier(Modifier::new().fill_max_width())
                                    .children(move || {
                                        text()
                                            .content(I18N.text(&(headline)))
                                            .style(type_style(24.0, 32.0))
                                            .color(scheme.on_surface)
                                            .modifier(Modifier::new().weight(1.0));
                                        if marked {
                                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                                            guide_marker();
                                        }
                                    });
                                gap(Dp(24.0));
                            }
                            for (caption, value) in [
                                ("ui-location", "ui-mountain-view"),
                                ("ui-date", "ui-fri-aug-8-7-00-pm"),
                            ] {
                                text()
                                    .content(I18N.text(caption))
                                    .style(type_style(12.0, 16.0))
                                    .color(scheme.on_surface_variant);
                                gap(Dp(4.0));
                                text()
                                    .content(I18N.text(value))
                                    .style(type_style(16.0, 24.0))
                                    .color(scheme.on_surface);
                                gap(Dp(8.0));
                                spacer().modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(Dp(1.0))
                                        .background(scheme.outline_variant),
                                );
                                gap(Dp(16.0));
                            }
                        });
                });
            if let Some(overlay) = overlay.clone() {
                scrimmed(overlay);
            }
        });
}

#[tessera]
fn calendar_scene() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.surface)
                .padding_all(Dp(16.0)),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text()
                        .content(I18N.text("ui-august-2025"))
                        .style(type_style(22.0, 28.0))
                        .color(scheme.on_surface);
                    gap(Dp(12.0));
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            for day in ["S", "M", "T", "W", "T", "F", "S"] {
                                boxed()
                                    .alignment(Alignment::Center)
                                    .modifier(Modifier::new().weight(1.0).height(Dp(24.0)))
                                    .children(move || {
                                        text()
                                            .content(I18N.text(day))
                                            .style(type_style(12.0, 16.0))
                                            .color(scheme.on_surface_variant);
                                    });
                            }
                        });
                    for week in 0..6_i32 {
                        row()
                            .modifier(Modifier::new().fill_max_width())
                            .children(move || {
                                for weekday in 0..7_i32 {
                                    // August 1, 2025 falls on a Friday.
                                    let date = week * 7 + weekday - 4;
                                    boxed()
                                        .alignment(Alignment::Center)
                                        .modifier(Modifier::new().weight(1.0).height(Dp(30.0)))
                                        .children(move || {
                                            if (1..=31).contains(&date) {
                                                text()
                                                    .content((date).to_string())
                                                    .style(type_style(14.0, 20.0))
                                                    .color(scheme.on_surface);
                                            }
                                        });
                                }
                            });
                    }
                });
            boxed()
                .alignment(Alignment::BottomEnd)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .size(Dp(56.0), Dp(56.0))
                                .background_with_shape(
                                    scheme.primary_container,
                                    Shape::rounded_rectangle(Dp(16.0)),
                                )
                                .semantics(SemanticsArgs {
                                    role: Some(Role::Image),
                                    label: Some(I18N.text("ui-floating-action-button")),
                                    ..Default::default()
                                }),
                        )
                        .children(move || {
                            icon()
                                .vector(filled::ADD_SVG)
                                .size(Dp(24.0))
                                .tint(scheme.on_primary_container);
                        });
                });
        });
}

#[tessera]
fn saving_scene() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.surface_container_high)
                .padding_all(Dp(8.0)),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .weight(1.0)
                        .fill_max_height()
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
                )
                .children(|| {
                    calendar_scene();
                });
            spacer().modifier(Modifier::new().width(Dp(16.0)));
            boxed()
                .modifier(
                    Modifier::new()
                        .weight(1.0)
                        .fill_max_height()
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
                )
                .children(|| {
                    full_screen_scene("ui-new-event", "ui-save").overlay(|| {
                        mock_dialog(
                            "ui-discard-changes",
                            280.0,
                            labels(&["ui-keep-editing", "ui-discard"]),
                        )
                        .body(I18N.text("ui-discard-the-changes-you-ve-made-to-this-event"));
                    });
                });
        });
}

#[tessera]
fn variant_pair() {
    dialog_overview_pair();
    gap(Dp(16.0));
    caption("ui-basic-and-full-screen-dialog");
    gap(Dp(12.0));
    legend(vec![
        "ui-basic-dialog".into(),
        "ui-full-screen-dialog".into(),
    ]);
}

#[tessera]
fn guideline_hero() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let notice = remember(String::new);
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(24.0), Dp(32.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(
                    Modifier::new()
                        .width(Dp(312.0))
                        .background_with_shape(
                            scheme.surface_container_high,
                            Shape::rounded_rectangle(Dp(28.0)),
                        )
                        .padding_all(Dp(24.0))
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(I18N.text("ui-basic-dialog-in-isolation")),
                            ..Default::default()
                        }),
                )
                .children(move || {
                    text()
                        .content(I18N.text("ui-basic-dialog-title"))
                        .style(type_style(24.0, 32.0))
                        .color(scheme.on_surface);
                    gap(Dp(16.0));
                    text()
                        .content(I18N.text("ui-a-dialog-is-a-type-of-modal-window-that-appears-in-front-of-app-content-to-provide-critica"))
                        .style(type_style(14.0, 20.0))
                        .color(scheme.on_surface_variant);
                    gap(Dp(24.0));
                    row()
                        .main_axis_alignment(MainAxisAlignment::End)
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            action(
                                "ui-action-2".into(),
                                Callback::new(move || notice.set("ui-action-2-selected".into())),
                            );
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            action(
                                "ui-action-1".into(),
                                Callback::new(move || notice.set("ui-action-1-selected".into())),
                            );
                        });
                });
        });
    gap(Dp(16.0));
    caption(if notice.with(String::is_empty) {
        "ui-a-basic-dialog".to_owned()
    } else {
        notice.get()
    });
}

#[tessera]
fn overview_intro(open: State<bool>, full: State<bool>, saved: State<bool>) {
    prose("ui-use-dialogs-to-make-sure-users-act-on-information");
    gap(Dp(16.0));
    bullets(vec![
        "ui-two-variants-basic-dialogs-interrupt-users-with-urgent-information-details-or-actions-they".into(),
        "ui-should-be-dedicated-to-completing-a-single-task".into(),
        "ui-can-also-display-information-relevant-to-the-task".into(),
        "ui-commonly-used-to-confirm-high-risk-actions-like-deleting-progress".into(),
    ]);
    gap(Dp(32.0));
    variant_pair();
    gap(Dp(32.0));
    launch(open, full, saved);
}

#[tessera]
fn availability() {
    spec_table(
        vec!["ui-type".into(), "ui-resource".into(), "ui-status".into()],
        vec![
            vec![
                "ui-design".into(),
                "ui-design-kit-figma".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Flutter".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web".into(),
                "ui-available".into(),
            ],
        ],
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "ui-color-new-color-mappings-and-compatibility-with-dynamic-color-2123".into(),
        "ui-layout-greater-padding-to-account-for-the-increased-corner-radius-and-title-size"
            .into(),
        "ui-position-option-for-custom-basic-dialog-positioning".into(),
        "ui-shape-increased-corner-radius".into(),
        "ui-typography-larger-and-darker-headline".into(),
    ]);
    gap(Dp(24.0));
    basic_figure(false);
    gap(Dp(16.0));
    caption("ui-new-updates-to-color-layout-position-shape-and-typography");
}

#[tessera]
fn usage_section() {
    prose(
        "ui-a-dialog-is-a-modal-window-that-appears-in-front-of-app-content-to-provide-critical-inform",
    );
    gap(Dp(16.0));
    prose(
        "ui-dialogs-are-purposefully-interruptive-so-they-should-be-used-sparingly-a-less-disruptive-a",
    );
    gap(Dp(24.0));
    dialog_overview_pair();
    gap(Dp(16.0));
    caption("ui-there-are-two-variants-of-dialogs");
    gap(Dp(12.0));
    legend(vec![
        "ui-basic-dialog".into(),
        "ui-full-screen-dialog".into(),
    ]);
    gap(Dp(32.0));
    guidance_pair(
        Verdict::Do,
        "ui-use-dialogs-for-prompts-that-block-an-app-s-normal-operation-and-for-critical-information",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Photos).dialog(|| {
                mock_dialog(
                    "ui-delete-selected-images",
                    312.0,
                    labels(&["ui-cancel", "ui-delete"]),
                );
            });
        }),
        Verdict::Dont,
        "ui-don-t-use-dialogs-for-low-or-medium-priority-information-instead-use-a-snackbar-which-can",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Photos).dialog(|| {
                mock_dialog("ui-upload-completed", 312.0, labels(&["ui-ok"]));
            });
        }),
    );
    gap(Dp(32.0));
    subheading("ui-similar-components".into());
    gap(Dp(12.0));
    prose("ui-snackbars-are-also-designed-to-show-important-messages");
    gap(Dp(12.0));
    prose(
        "ui-choose-the-right-component-based-on-the-importance-of-the-message-this-component-messaging",
    );
    gap(Dp(16.0));
    guidance_card(
        Verdict::Note,
        "ui-snackbars-can-disappear-automatically",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Snackbar);
        }),
    )
    .wide(true);
    gap(Dp(16.0));
    spec_table(
        vec![
            "ui-component".into(),
            "ui-importance".into(),
            "ui-action-needed".into(),
        ],
        vec![
            vec![
                "ui-snackbar".into(),
                "ui-low-importance".into(),
                "ui-optional-snackbars-may-not-have-a-button-and-can-disappear-automatically"
                    .into(),
            ],
            vec![
                "ui-dialog".into(),
                "ui-high-importance".into(),
                "ui-required-dialogs-block-the-main-content-until-an-action-is-confirmed".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy_section() {
    subheading("ui-basic-dialog".into());
    gap(Dp(16.0));
    basic_figure(false);
    gap(Dp(16.0));
    legend(
        [
            "ui-container",
            "ui-icon-optional",
            "ui-headline-optional",
            "ui-supporting-text",
            "ui-divider-optional",
            "ui-buttons-label-text",
            "Scrim",
        ]
        .map(String::from)
        .to_vec(),
    );
    gap(Dp(32.0));
    subheading("ui-full-screen-dialog".into());
    gap(Dp(16.0));
    full_spec_figure(0);
    gap(Dp(16.0));
    legend(
        [
            "ui-container",
            "ui-header-region",
            "ui-icon-close-affordance",
            "ui-headline-optional",
            "ui-button-label-text",
            "ui-divider-optional",
        ]
        .map(String::from)
        .to_vec(),
    );
    gap(Dp(32.0));
    subheading("ui-container-and-scrim".into());
    gap(Dp(12.0));
    prose(
        "ui-dialog-containers-appear-above-other-screen-elements-and-hold-the-dialog-s-headline-text-b",
    );
    gap(Dp(12.0));
    prose(
        "ui-to-focus-attention-on-the-dialog-surfaces-behind-the-container-are-scrimmed-with-a-tempora",
    );
    gap(Dp(12.0));
    guidance_card(
        Verdict::Note,
        "ui-basic-dialogs-appear-over-a-background-scrim",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Photos).dialog(|| {
                mock_dialog(
                    "ui-permanently-delete",
                    312.0,
                    labels(&["ui-cancel", "ui-delete"]),
                );
            });
        }),
    )
    .wide(true);
    gap(Dp(32.0));
    subheading("ui-headline-optional".into());
    gap(Dp(12.0));
    prose(
        "ui-a-dialog-s-purpose-should-be-communicated-by-its-headline-and-buttons-or-actionable-items",
    );
    gap(Dp(12.0));
    prose("ui-headlines-should");
    gap(Dp(8.0));
    bullets(vec![
        "ui-contain-a-brief-clear-statement-or-question".into(),
        "ui-avoid-apologies-sorry-for-the-interruption-alarm-warning-or-ambiguity-are-you-sure"
            .into(),
    ]);
    gap(Dp(16.0));
    guidance_pair(
        Verdict::Do,
        "ui-this-dialog-title-poses-a-specific-question-concisely-explains-what-s-involved-in-the-requ",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog("ui-use-location-service", 312.0, labels(&["ui-disagree", "ui-agree"])).body(
                    I18N.text("ui-let-google-help-apps-determine-location-this-means-sending-anonymous-location-data-to-goog"),
                );
            });
        }),
        Verdict::Dont,
        "ui-don-t-use-dialog-titles-that-pose-an-ambiguous-question",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog("ui-are-you-sure", 312.0, labels(&["ui-no", "ui-yes"]));
            });
        }),
    );
    gap(Dp(24.0));
    prose(
        "ui-headlines-should-always-be-succinct-they-can-wrap-to-a-second-line-if-necessary-and-be-tru",
    );
    gap(Dp(12.0));
    prose(
        "ui-in-full-screen-dialogs-long-headlines-or-headlines-of-variable-lengths-such-as-translation",
    );
    gap(Dp(16.0));
    guidance_pair(
        Verdict::Caution,
        "ui-avoid-placing-long-headlines-in-a-full-screen-dialog-s-app-bar-1-as-the-truncated-text-may",
        RenderSlot::new(|| {
            full_screen_scene("ui-reserve-a-table-fo", "ui-save").marked(true);
        }),
        Verdict::Do,
        "ui-find-ways-to-shorten-app-bar-text-and-place-longer-headlines-into-the-content-area-1-of-a",
        RenderSlot::new(move || {
            full_screen_scene("ui-new-booking", "ui-save")
                .headline(I18N.text("ui-reserve-a-table-for-four-at-mountain-view-bistro"))
                .marked(true);
        }),
    );
    gap(Dp(32.0));
    subheading("ui-buttons".into());
    gap(Dp(12.0));
    prose(
        "ui-dialog-actions-are-most-often-represented-as-buttons-and-allow-users-to-confirm-dismiss-or",
    );
    gap(Dp(12.0));
    prose(
        "ui-buttons-are-aligned-to-the-trailing-edge-of-the-dialog-for-easier-interaction-the-confirma",
    );
    gap(Dp(12.0));
    prose(
        "ui-button-alignment-responds-automatically-for-right-to-left-languages-where-the-confirmation",
    );
    gap(Dp(16.0));
    guidance_pair(
        Verdict::Do,
        "ui-disable-confirming-actions-1-until-a-choice-is-made-dismissive-actions-are-never-disabled",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog("ui-phone-ringtone", 312.0, labels(&["ui-cancel", "ui-ok"]))
                    .disabled(1)
                    .marked(1)
                    .content(|| {
                        radio_options();
                    });
            });
        }),
        Verdict::Dont,
        "ui-don-t-place-dismissive-actions-1-to-the-right-of-confirming-actions-instead-place-them-to",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog("ui-phone-ringtone", 312.0, labels(&["ui-ok", "ui-cancel"]))
                    .marked(1)
                    .content(|| {
                        radio_options().selected(1);
                    });
            });
        }),
    );
    gap(Dp(32.0));
    guidance_pair(
        Verdict::Do,
        "ui-a-single-action-may-be-provided-only-if-it-s-an-acknowledgement",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Product);
        }),
        Verdict::Dont,
        "ui-avoid-presenting-people-with-unclear-choices-cancel-doesn-t-make-sense-here-because-no-cle",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog(
                    "ui-out-of-stock",
                    312.0,
                    labels(&["ui-cancel", "ui-got-it"]),
                )
                .body(I18N.text("ui-the-item-in-your-cart-is-no-longer-available"));
            });
        }),
    );
    gap(Dp(24.0));
    prose("ui-dialogs-should-contain-a-maximum-of-two-actions");
    gap(Dp(8.0));
    bullets(vec![
        "ui-if-a-single-action-is-provided-it-must-be-an-acknowledgement-action".into(),
        "ui-if-two-actions-are-provided-one-must-be-a-confirming-action-and-the-other-a-dismissing-act".into(),
    ]);
    gap(Dp(16.0));
    guidance_pair(
        Verdict::Do,
        "ui-display-two-text-buttons-next-to-one-another",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog(
                    "ui-use-location-service",
                    312.0,
                    labels(&["ui-disagree", "ui-agree"]),
                );
            });
        }),
        Verdict::Caution,
        "ui-stacked-buttons-accommodate-longer-button-text-but-take-up-more-room-confirming-actions-ap",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog(
                    "ui-turn-on-speed-boost",
                    312.0,
                    labels(&["ui-turn-on-speed-boost-2194", "ui-no-thanks"]),
                )
                .stacked(true);
            });
        }),
    );
    gap(Dp(24.0));
    prose(
        "ui-providing-a-third-action-such-as-learn-more-is-not-recommended-as-it-navigates-the-user-aw",
    );
    gap(Dp(12.0));
    prose(
        "ui-rather-than-adding-a-third-action-an-inline-expansion-can-display-more-information-if-more",
    );
    gap(Dp(16.0));
    guidance_card(
        Verdict::Caution,
        "ui-the-learn-more-action-1-navigates-away-from-this-dialog-potentially-leaving-it-in-an-indet",
        RenderSlot::new(|| {
            backdrop_scene(Backdrop::Surface).dialog(|| {
                mock_dialog(
                    "ui-use-location-service",
                    360.0,
                    labels(&["ui-learn-more", "ui-disagree", "ui-agree"]),
                )
                .marked(0);
            });
        }),
    )
    .wide(true);
}

#[tessera]
fn basic_guidance() {
    prose(
        "ui-basic-dialogs-interrupt-users-with-urgent-information-details-or-actions-common-use-cases",
    );
    gap(Dp(16.0));
    caption("ui-basic-dialogs-require-a-person-to-take-action-before-it-will-close");
    gap(Dp(8.0));
    caption(
        "ui-basic-dialogs-can-give-people-the-ability-to-provide-confirmation-of-a-choice-before-commi",
    );
    gap(Dp(16.0));
    prose(
        "ui-basic-dialogs-most-often-appear-as-alerts-or-lists-but-can-have-a-variety-of-layouts-and-c",
    );
}

#[tessera]
fn full_guidance() {
    prose(
        "ui-full-screen-dialogs-fill-the-entire-screen-containing-actions-that-require-a-series-of-tas",
    );
    gap(Dp(16.0));
    prose(
        "ui-because-they-take-up-the-entire-screen-full-screen-dialogs-are-the-only-dialogs-over-which",
    );
    gap(Dp(24.0));
    subheading("ui-when-to-use".into());
    gap(Dp(12.0));
    bullets(vec![
        "ui-dialogs-that-include-components-which-require-keyboard-input-such-as-form-fields"
            .into(),
        "ui-when-changes-aren-t-saved-instantly".into(),
        "ui-when-components-within-the-dialog-open-additional-dialogs".into(),
    ]);
    gap(Dp(16.0));
    prose(
        "ui-full-screen-dialogs-are-for-compact-breakpoints-only-such-as-mobile-devices-for-medium-and",
    );
    gap(Dp(24.0));
    subheading("ui-saving-selections".into());
    gap(Dp(12.0));
    prose(
        "ui-to-save-a-selection-in-a-full-screen-dialog-use-save-the-close-icon-dismisses-the-dialog-w",
    );
    gap(Dp(16.0));
    guidance_card(
        Verdict::Note,
        "ui-when-a-full-screen-dialog-is-closed-without-saving-a-basic-dialog-confirms-that-changes-sh",
        RenderSlot::new(|| {
            saving_scene();
        }),
    )
    .wide(true);
    gap(Dp(32.0));
    guidance_pair(
        Verdict::Do,
        "ui-a-clearly-named-action-such-as-create-makes-it-clear-that-the-event-will-be-created",
        RenderSlot::new(|| {
            full_screen_scene("ui-new-event", "ui-create");
        }),
        Verdict::Dont,
        "ui-don-t-trigger-a-basic-dialog-when-the-confirming-action-is-selected-and-don-t-use-the-conf",
        RenderSlot::new(|| {
            full_screen_scene("ui-new-event", "ui-save").overlay(|| {
                mock_dialog("ui-save-event", 280.0, labels(&["ui-cancel", "ui-save"]));
            });
        }),
    );
    gap(Dp(24.0));
    subheading("ui-error-messages".into());
    gap(Dp(12.0));
    prose(
        "ui-errors-about-the-dialog-fields-should-always-appear-inline-where-they-occur-general-errors",
    );
    gap(Dp(24.0));
    full_figure();
}

#[tessera]
fn adaptive_section() {
    subheading("ui-medium-breakpoint".into());
    gap(Dp(12.0));
    prose(
        "ui-basic-dialogs-appear-in-a-center-position-by-default-their-position-can-be-overridden-to-p",
    );
    gap(Dp(24.0));
    subheading("ui-expanded-breakpoint".into());
    gap(Dp(12.0));
    prose(
        "ui-dialogs-on-expanded-breakpoints-like-desktop-are-modal-windows-above-a-scrim-this-puts-the",
    );
    gap(Dp(16.0));
    prose(
        "ui-basic-dialogs-can-be-custom-positioned-anywhere-on-larger-screens-respecting-margins-to-pr",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-default-position".into(), "ui-center".into()],
            vec![
                "ui-custom-placement-margin".into(),
                "ui-56dp-from-screen-edges".into(),
            ],
        ],
    );
    gap(Dp(16.0));
    caption(
        "ui-custom-placement-area-for-basic-dialogs-that-respects-a-56dp-margin-from-the-edges-of-the",
    );
}

#[tessera]
fn behavior_section(open: State<bool>, full: State<bool>, saved: State<bool>) {
    subheading("ui-appearing".into());
    gap(Dp(12.0));
    prose(
        "ui-dialogs-appear-without-warning-requiring-users-to-stop-their-current-task-they-should-be-u",
    );
    gap(Dp(24.0));
    subheading("ui-position".into());
    gap(Dp(12.0));
    prose(
        "ui-dialogs-retain-focus-until-dismissed-or-an-action-has-been-taken-such-as-choosing-a-settin",
    );
    gap(Dp(24.0));
    subheading("ui-scrolling".into());
    gap(Dp(12.0));
    prose(
        "ui-most-dialog-content-should-avoid-scrolling-even-when-scrolling-is-required-the-dialog-titl",
    );
    gap(Dp(24.0));
    launch(open, full, saved);
}

#[tessera]
fn use_cases(open: State<bool>, full: State<bool>, saved: State<bool>) {
    prose("ui-people-should-be-able-to-use-assistive-technology-to");
    gap(Dp(12.0));
    bullets(vec![
        "ui-open-and-close-a-dialog".into(),
        "ui-provide-and-submit-other-inputs-if-the-dialog-is-interactive-such-as-a-text-field-or-selec".into(),
        "ui-scroll-the-dialog-to-access-all-of-its-contents-if-that-content-extends-beyond-the-contain".into(),
    ]);
    gap(Dp(24.0));
    launch(open, full, saved);
}

#[tessera]
fn interaction_style_section() {
    subheading("ui-use-sparingly".into());
    gap(Dp(12.0));
    prose(
        "ui-dialogs-are-purposefully-interruptive-this-means-they-appear-in-front-of-app-content-and-d",
    );
    gap(Dp(16.0));
    prose(
        "ui-as-such-dialogs-should-be-used-sparingly-and-only-to-provide-critical-information-less-cri",
    );
    gap(Dp(16.0));
    do_dont(
        "ui-present-non-critical-information-using-other-ui-within-the-flow-of-app-content",
        "ui-avoid-putting-non-critical-information-in-a-dialog",
    );
    gap(Dp(32.0));
    subheading("ui-200-text-size".into());
    gap(Dp(12.0));
    prose(
        "ui-avoid-excessive-text-wrapping-or-truncation-by-choosing-concise-strings-on-android-headlin",
    );
    gap(Dp(32.0));
    subheading("ui-elements-within-dialogs".into());
    gap(Dp(12.0));
    prose(
        "ui-because-dialogs-can-contain-various-elements-within-them-refer-to-the-relevant-accessibili",
    );
    gap(Dp(24.0));
    accessibility::interaction_style();
}

#[tessera]
fn initial_focus_section() {
    prose(
        "ui-when-a-dialog-appears-focus-should-automatically-land-on-the-first-interactive-element-wit",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-tab-key-moves-focus-through-the-next-interactive-elements-in-a-cycle-and-the-shift-and",
    );
    gap(Dp(24.0));
    accessibility::initial_focus();
}

#[tessera]
fn keyboard_section() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-focus-lands-on-the-next-interactive-element-contained-in-the-dialog-or-the-first-element-i".into(),
            ],
            vec![
                "ui-shift-tab".into(),
                "ui-focus-lands-on-the-previous-interactive-element-contained-in-the-dialog-or-the-last-elemen".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-triggers-or-commits-the-action-of-the-focused-element".into(),
            ],
            vec!["ui-escape".into(), "ui-closes-the-dialog".into()],
        ],
    );
}

#[tessera]
fn labeling_section() {
    prose(
        "ui-the-accessibility-label-for-a-dialog-is-typically-the-same-as-the-dialog-s-title-or-headli",
    );
    gap(Dp(24.0));
    accessibility::labeling();
}

#[tessera]
fn event_dialog(
    draft: State<EventDraft>,
    open: State<bool>,
    saved: State<bool>,
    discard: State<bool>,
    close: Callback,
    dialog: Option<tessera_components::dialog::DialogScope>,
) {
    dialog_provider()
        .role(DialogRole::AlertDialog)
        .is_open(discard.get())
        .accessibility_label(I18N.text("ui-discard-changes"))
        .on_close_request(move || discard.set(false))
        .main_content(move || {
            full_screen_dialog(close)
                .dialog_optional(dialog)
                .title(I18N.text("ui-new-event"))
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
        .dialog_content(move |dialog_scope| {
            basic_dialog()
                .dialog(dialog_scope)
                .headline(I18N.text("ui-discard-changes"))
                .supporting_text(I18N.text("ui-discard-the-changes-you-ve-made-to-this-event"))
                .dismiss_button(move || {
                    action(
                        "ui-keep-editing".into(),
                        Callback::new(move || discard.set(false)),
                    );
                })
                .confirm_button(move || {
                    action(
                        "ui-discard".into(),
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
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn launch(open: State<bool>, full: State<bool>, saved: State<bool>) {
    column().children(move || {
        button()
            .outlined()
            .accessibility_label(I18N.text("ui-open-basic-dialog"))
            .on_click(move || {
                full.set(false);
                saved.set(false);
                open.set(true);
            })
            .child(move || {
                text().content(I18N.text("ui-open-basic-dialog"));
            });
        gap(Dp(12.0));
        button()
            .outlined()
            .accessibility_label(I18N.text("ui-open-full-screen-dialog"))
            .on_click(move || {
                full.set(true);
                saved.set(false);
                open.set(true);
            })
            .child(move || {
                text().content(I18N.text("ui-open-full-screen-dialog"));
            });
        if saved.get() {
            gap(Dp(12.0));
            prose("ui-changes-saved");
        }
    });
}

#[tessera]
fn action(label: String, callback: Callback) {
    button()
        .text()
        .accessibility_label(I18N.text(&(label)))
        .on_click_shared(callback)
        .child(move || {
            text().content(I18N.text(&(label)));
        });
}

#[tessera]
fn basic_specs() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            basic_figure(false);
            gap(Dp(24.0));
            legend(
                [
                    "ui-container",
                    "ui-icon-optional",
                    "ui-headline-optional",
                    "ui-supporting-text",
                    "ui-divider-optional",
                    "ui-button-label-text",
                    "Scrim",
                ]
                .map(String::from)
                .to_vec(),
            );
            gap(Dp(56.0));
            subheading("ui-basic-dialog-color".into());
            gap(Dp(24.0));
            basic_color_figure();
            gap(Dp(56.0));
            subheading("ui-basic-dialog-measurements".into());
            gap(Dp(32.0));
            basic_figure(true);
            gap(Dp(24.0));
            prose("ui-basic-dialog-padding-and-size-measurements");
            gap(Dp(32.0));
            spec_table(
                vec!["ui-attribute".into(), "ui-value".into()],
                vec![
                    vec!["ui-container-shape".into(), "ui-28dp-corner-radius".into()],
                    vec!["ui-container-height".into(), "ui-dynamic".into()],
                    vec!["ui-container-width".into(), "ui-min-280dp-max-560dp".into()],
                    vec!["ui-divider-height".into(), "1dp".into()],
                    vec!["ui-icon-size".into(), "24dp".into()],
                    vec!["ui-alignment-with-icon".into(), "ui-center-aligned".into()],
                    vec![
                        "ui-alignment-without-icon".into(),
                        "ui-start-aligned".into(),
                    ],
                    vec!["ui-top-left-right-bottom-padding".into(), "24dp".into()],
                    vec!["ui-padding-between-buttons".into(), "8dp".into()],
                    vec!["ui-padding-between-title-and-body".into(), "16dp".into()],
                    vec!["ui-padding-between-icon-and-title".into(), "16dp".into()],
                    vec!["ui-padding-between-body-and-actions".into(), "24dp".into()],
                ],
            );
        });
}

#[tessera]
fn basic_figure(measurements: bool) {
    let geometry = remember(Geometry::new);
    let scale = 1.0_f64;
    let notice = remember(String::new);
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
                                    test_tag: Some("dialog-figure-figure".into()),
                                    role: Some(Role::Group),
                                    merge_descendants: true,
                                    label: Some(I18N.text(if measurements {
                                        "ui-basic-dialog-measurement-figure"
                                    } else {
                                        "ui-basic-dialog-anatomy-figure"
                                    })),
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
                                            test_tag: Some("dialog-figure-container".into()),
                                            role: Some(Role::Group),
                                            merge_descendants: true,
                                            label: Some(I18N.text(if measurements {
                                                "ui-basic-dialog-measurement-container"
                                            } else {
                                                "ui-basic-dialog-anatomy-container"
                                            })),
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
                                                                .modifier(
                                                                    watch(geometry, "icon")
                                                                        .semantics(SemanticsArgs {
                                                                        test_tag: Some(
                                                                            "dialog-figure-icon"
                                                                                .into(),
                                                                        ),
                                                                        ..Default::default()
                                                                    }),
                                                                )
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
                                                    .content(I18N.text(if measurements {
                                                        "ui-basic-dialog-title"
                                                    } else {
                                                        "ui-reset-settings"
                                                    }))
                                                    .style(dialog_type_style(24.0, 32.0, scale))
                                                    .color(scheme.on_surface)
                                                    .modifier(if measurements {
                                                        watch(geometry, "headline").semantics(
                                                            SemanticsArgs {
                                                                test_tag: Some(
                                                                    "dialog-figure-headline".into(),
                                                                ),
                                                                ..Default::default()
                                                            },
                                                        )
                                                    } else {
                                                        watch(geometry, "headline")
                                                            .semantics(SemanticsArgs {
                                                                test_tag: Some(
                                                                    "dialog-figure-headline".into(),
                                                                ),
                                                                ..Default::default()
                                                            })
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
                                                                "ui-action-2"
                                                            } else {
                                                                "ui-cancel"
                                                            }
                                                            .into(),
                                                            geometry,
                                                            false,
                                                            scale,
                                                            Callback::new(move || {
                                                                notice.set("ui-dismissed".into())
                                                            }),
                                                        );
                                                        spacer().modifier(
                                                            Modifier::new().width(Dp(8.0 * scale)),
                                                        );
                                                        dialog_figure_action(
                                                            if measurements {
                                                                "ui-action-1"
                                                            } else {
                                                                "ui-accept"
                                                            }
                                                            .into(),
                                                            geometry,
                                                            true,
                                                            scale,
                                                            Callback::new(move || {
                                                                notice.set("ui-accepted".into())
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
                .width(Dp(if label.starts_with("ui-action") {
                    84.0
                } else if confirm {
                    80.0
                } else {
                    77.0
                } * scale)),
        )
        .content_padding(Padding::symmetric(Dp(16.0 * scale), Dp::ZERO))
        .accessibility_label(I18N.text(&(label)))
        .on_click_shared(callback)
        .child(move || {
            let mut style = dialog_type_style(14.0, 20.0, scale);
            style.font_weight = 500;
            text().content(I18N.text(&(label))).style(style);
        });
}

fn dialog_account_photo() -> ImageData {
    include_bytes!("../../../assets/material_spec/dialog-basic-anatomy.png")
        .as_slice()
        .try_into_image_data()
        .expect("official dialog account photographs")
}

#[tessera]
fn dialog_account_avatar(index: usize, scale: f64) {
    let photo = remember(dialog_account_photo).get();
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(40.0 * scale), Dp(40.0 * scale))
                .clip_shape(Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(
                        I18N.format("ui-account-photo-value", [("v0", (index + 1).into())]),
                    ),
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
        text().content(I18N.text(if measurements {
            "ui-a-dialog-is-a-modal-window-that-appears-in-front-of-app-content-to-provide-critical-inform-2275"
        } else {
            "ui-this-will-reset-your-app-preferences-back-to-their-default-settings-the-following-accounts-2276"
        })).style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
            .modifier(watch(geometry, "body").semantics(SemanticsArgs { test_tag: Some("dialog-figure-body".into()), merge_descendants: true,
..Default::default() }).fill_max_width());
        if !measurements {
            gap(Dp(16.0 * scale));
            spacer().modifier(watch(geometry, "divider").semantics(SemanticsArgs { test_tag: Some("dialog-figure-divider".into()), merge_descendants: true,
..Default::default() }).fill_max_width().height(Dp(scale)).background(scheme.outline_variant));
            gap(Dp(8.0 * scale));
            for (index, email) in ["leevillanuevanotes@google.com", "alloalejandro@google.com", "oliortega@google.com"].into_iter().enumerate() {
                row().cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(56.0 * scale))).children(move || {
                        dialog_account_avatar(index, scale);
                        spacer().modifier(Modifier::new().width(Dp(16.0 * scale)));
                        text().content(I18N.text(email)).style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface)
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
                    label: Some(I18N.format("ui-anatomy-marker-value", [("v0", (number).into())])),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content((number).to_string())
                .style(dialog_type_style(14.0, 20.0, scale))
                .color(INK);
        });
}

#[tessera]
fn dialog_dimension(label: String, x: f64, y: f64, scale: f64) {
    text()
        .content(I18N.text(&(label)))
        .style(dialog_type_style(14.0, 20.0, scale))
        .color(INK)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(
                        I18N.format("ui-dimension-value", [("v0", I18N.text(&label).into())]),
                    ),
                    ..Default::default()
                }),
        );
}

#[tessera]
fn dialog_touch_targets(child: RenderSlot) {
    child.render();
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
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width()).children(move || {
dialog_canvas(1024.0, 622.0, RenderSlot::new(move || {
        boxed().modifier(Modifier::new().fill_max_width().height(Dp(622.0 * scale))
            .semantics(SemanticsArgs {
                role: Some(Role::Group),
                label: Some(I18N.text("ui-dialogs-overview-figure")),
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
                                    text().content(I18N.text("ui-basic-dialog-title")).style(dialog_type_style(24.0, 32.0, scale)).color(scheme.on_surface);
                                    text().content(I18N.text("ui-a-dialog-is-a-modal-window-that-appears-in-front-of-app-content-to-provide-critical-inform-2278"))
                                        .style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
                                        .modifier(Modifier::new().offset(Dp::ZERO, Dp(52.0 * scale)).width(Dp(264.0 * scale)));
                                    dialog_scene_action("ui-text-button".into(), "ui-overview-basic-dismiss".into(), 40.0, 132.0, 112.0, scale,
                                        Callback::new(move || notice.set("ui-basic-dialog-dismissed".into())));
                                    dialog_scene_action("ui-text-button".into(), "ui-overview-basic-confirm".into(), 156.0, 132.0, 112.0, scale,
                                        Callback::new(move || notice.set("ui-basic-dialog-accepted".into())));
                                });
                        }));
                        dialog_phone_frame(542.0, 88.0, 908.0, scale, RenderSlot::new(move || {
                            dialog_phone_status(scale);
                            dialog_phone_header("ui-full-screen-dialog-title".into(), scale,
                                Callback::new(move || notice.set("ui-full-screen-dialog-closed".into())),
                                Callback::new(move || notice.set("ui-event-saved".into())));
                            dialog_scene_input(24.0, 144.0, 364.0, "ui-label".into(), "ui-input".into(), "ui-overview-dialog-category".into(), true, scale);
                            dialog_plain_input(24.0, 216.0, 364.0, "ui-event-name".into(), "".into(), "ui-overview-event-name".into(), scale);
                            spacer().modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(295.0 * scale))
                                .size(Dp(364.0 * scale), Dp(scale)).background(scheme.outline_variant));
                            for (index, caption) in ["ui-from", "To"].into_iter().enumerate() {
                                let y = 318.0 + index as f64 * 104.0;
                                text().content(I18N.text(caption)).style(dialog_type_style(12.0, 16.0, scale)).color(scheme.on_surface_variant)
                                    .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(y * scale)));
                                for (column, kind) in ["date", "time"].into_iter().enumerate() {
                                    dialog_scene_input(24.0 + column as f64 * 185.0, y + 25.0, 177.0,
                                        "ui-label".into(), "ui-input".into(), I18N.format("ui-overview-value-value", [("v0", I18N.text(&caption).into()), ("v1", I18N.text(&kind).into())]), true, scale);
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
                    label: Some(I18N.format("ui-dialog-variant-value", [("v0", (number).into())])),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content((number).to_string())
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
        .accessibility_label(I18N.text("ui-close-overview-dialog"))
        .on_click_shared(close)
        .child(move || {
            icon()
                .vector(filled::CLOSE_SVG)
                .size(Dp(24.0 * scale))
                .tint(scheme.on_surface);
        });
    text()
        .content(I18N.text(title))
        .style(dialog_type_style(22.0, 28.0, scale))
        .color(scheme.on_surface)
        .modifier(Modifier::new().offset(Dp(56.0 * scale), Dp(74.0 * scale)));
    dialog_scene_action(
        "ui-save".into(),
        "ui-save-overview-event".into(),
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
        .accessibility_label(I18N.text(name))
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
            text().content(I18N.text(&(label))).style(style);
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
        controller.set_text(&I18N.text(&initial));
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
            let content =
                RenderSlot::new(move || {
                    let field = text_field()
                        .outlined()
                        .controller(controller)
                        .label(I18N.text(label.get()))
                        .accessibility_label(I18N.text(name.get()))
                        .min_width(Dp::ZERO)
                        .min_height(Dp(56.0 * scale))
                        .padding(Dp(16.0 * scale))
                        .font_size(Dp(16.0 * scale))
                        .line_height(Dp(24.0 * scale))
                        .modifier(Modifier::new().size(Dp(width * scale), Dp(56.0 * scale)));
                    if dropdown {
                        field.trailing_icon(move || {
                            button()
                                .text()
                                .accessibility_label(I18N.format(
                                    "ui-choose-value",
                                    [("v0", I18N.text(&name.get()).into())],
                                ))
                                .modifier(Modifier::new().size(Dp(32.0 * scale), Dp(40.0 * scale)))
                                .content_padding(Padding::all(Dp::ZERO))
                                .on_click(move || menu.with_mut(MenuController::toggle))
                                .child(move || {
                                    icon()
                                        .vector(filled::ARROW_DROP_DOWN_SVG)
                                        .size(Dp(24.0 * scale));
                                });
                        });
                    }
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
                    .menu_content(move |menu| {
                        for value in ["ui-input", "ui-option-2", "ui-option-3"] {
                            menu_item()
                                .menu(menu)
                                .label(I18N.text(value))
                                .on_click(move || {
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
    let notice = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width()).children(move || {
dialog_canvas(1024.0, 486.0, RenderSlot::new(move || {
        boxed().modifier(watch(geometry, "figure").semantics(SemanticsArgs { test_tag: Some("dialog-figure-figure".into()), merge_descendants: true,
..Default::default() }).fill_max_width().height(Dp(486.0 * scale))
            .background_with_shape(scheme.surface_container_low, Shape::rounded_rectangle(Dp(24.0))))
            .children(move || {
                spacer().modifier(Modifier::new().fill_max_size().background_with_shape(scheme.scrim.with_alpha(0.32), Shape::rounded_rectangle(Dp(24.0))));
                boxed().modifier(watch(geometry, "container").semantics(SemanticsArgs { test_tag: Some("dialog-figure-container".into()), merge_descendants: true,
..Default::default() }).offset(Dp(341.0 * scale), Dp(114.0 * scale))
                    .size(Dp(340.0 * scale), Dp(264.0 * scale))
                    .background_with_shape(scheme.surface_container_high, Shape::rounded_rectangle(Dp(28.0 * scale)))
                    .padding_all(Dp(24.0 * scale)))
                    .children(move || {
                        boxed().modifier(watch(geometry, "icon").semantics(SemanticsArgs { test_tag: Some("dialog-figure-icon".into()), merge_descendants: true,
..Default::default() }).offset(Dp(134.0 * scale), Dp::ZERO).size(Dp(24.0 * scale), Dp(24.0 * scale)))
                            .children(move || {
                                icon().vector(filled::PHONE_ANDROID_SVG).size(Dp(24.0 * scale)).tint(scheme.secondary);
                                boxed().modifier(Modifier::new().offset(Dp(6.0 * scale), Dp(6.0 * scale))).children(move || {
                                    icon().vector(filled::DONE_SVG).size(Dp(12.0 * scale)).tint(scheme.secondary);
                                });
                            });
                        text().content(I18N.text("ui-dialog-with-hero-icon")).style(dialog_type_style(24.0, 32.0, scale)).color(scheme.on_surface)
                            .modifier(watch(geometry, "headline").semantics(SemanticsArgs { test_tag: Some("dialog-figure-headline".into()), merge_descendants: true,
..Default::default() }).offset(Dp::ZERO, Dp(40.0 * scale)).align(Alignment::TopCenter));
                        text().content(I18N.text("ui-a-dialog-is-a-modal-window-that-appears-in-front-of-app-content-to-provide-critical-inform-2295"))
                            .style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
                            .modifier(watch(geometry, "body").semantics(SemanticsArgs { test_tag: Some("dialog-figure-body".into()), merge_descendants: true,
..Default::default() }).offset(Dp::ZERO, Dp(88.0 * scale)).width(Dp(292.0 * scale)));
                        dialog_touch_targets(RenderSlot::new(move || {
                            boxed().modifier(watch(geometry, "actions").semantics(SemanticsArgs { test_tag: Some("dialog-figure-actions".into()), merge_descendants: true,
..Default::default() }).offset(Dp(116.0 * scale), Dp(176.0 * scale)).size(Dp(176.0 * scale), Dp(40.0 * scale)))
                                .children(move || {
                                    dialog_scene_action("ui-cancel".into(), "ui-cancel-color-dialog".into(), 0.0, 0.0, 80.0, scale,
                                        Callback::new(move || notice.set("ui-color-dialog-dismissed".into())));
                                    dialog_scene_action("ui-accept".into(), "ui-accept-color-dialog".into(), 88.0, 0.0, 88.0, scale,
                                        Callback::new(move || notice.set("ui-color-dialog-accepted".into())));
                                });
                        }));
                    });
                basic_color_annotations(geometry, scale);
            });
            }));
        if !notice.with(String::is_empty) {gap(Dp(12.0)); prose(notice.get());}
        gap(Dp(24.0));
        for (index, role) in ["Surface container high", "Secondary", "On surface", "On surface variant", "Primary", "Scrim"].into_iter().enumerate() {
            prose(I18N.format("ui-value-value", [("v0", (index + 1).into()), ("v1", I18N.text(&role).into())]));
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
                    label: Some(I18N.format(
                        "ui-color-role-value-value",
                        [("v0", (number).into()), ("v1", I18N.text(&role).into())],
                    )),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content((number).to_string())
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
        controller.set_text(&I18N.text(&initial));
        controller
    });
    text_field()
        .outlined()
        .controller(controller)
        .placeholder(I18N.text(placeholder))
        .accessibility_label(I18N.text(name))
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
    let notice = remember(String::new);
    let all_day = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let y = [103.0, 98.0, 67.0][mode];
    dialog_canvas(
        1024.0,
        [463.0, 508.0, 1030.0][mode],
        RenderSlot::new(move || {
            boxed().modifier(watch(geometry, "figure").semantics(SemanticsArgs { test_tag: Some("dialog-figure-figure".into()), merge_descendants: true,
..Default::default() }).fill_max_width()
        .height(Dp([463.0, 508.0, 1030.0][mode] * scale)).clip_to_bounds()
        .background_with_shape(scheme.secondary_container, Shape::rounded_rectangle(Dp(24.0))))
        .children(move || {
            dialog_touch_targets(RenderSlot::new(move || {
                dialog_diagram_theme(scale, RenderSlot::new(move || {
                    boxed().modifier(watch(geometry, "container").semantics(SemanticsArgs { test_tag: Some("dialog-figure-container".into()), merge_descendants: true,
..Default::default() }).offset(Dp(298.0 * scale), Dp(y * scale))
                        .size(Dp(428.0 * scale), Dp(909.0 * scale)))
                        .children(move || {
                            dialog_phone_frame(0.0, 0.0, 909.0, scale, RenderSlot::new(move || {
                                spacer().modifier(Modifier::new().fill_max_size().background(scheme.surface_container_high));
                                dialog_phone_status(scale);
                                let header_y = if mode == 2 {52.0} else {60.0};
                                full_spec_header(if mode == 0 {"ui-new-event"} else {"ui-full-screen-dialog-title"}.into(),
                                    geometry, header_y, if mode == 2 {64.0} else {56.0}, scale,
                                    Callback::new(move || notice.set("ui-full-screen-dialog-closed".into())),
                                    Callback::new(move || notice.set("ui-event-saved".into())));
                                let first_y = if mode == 2 {144.0} else {134.0};
                                let x = if mode == 0 {23.0} else {24.0};
                                dialog_scene_input(x, first_y, 364.0, "ui-label".into(), "ui-input".into(), I18N.format("ui-full-spec-value-category", [("v0", (mode).into())]), true, scale);
                                dialog_plain_input(x, first_y + 72.0, 364.0, "ui-event-name".into(),
                                    match mode {0 => "ui-carmen-s-birthday-surprise-party", 1 => "ui-mam-s-birthday", _ => ""}.into(),
                                    I18N.format("ui-full-spec-value-event-name", [("v0", (mode).into())]), scale);
                                if mode == 1 {
                                    text().content(I18N.text("ui-with-mam-s-birthday-around-the-corner-i-d-like-to-start-planning-location-and-food-i-can-h"))
                                        .style(dialog_type_style(16.0, 24.0, scale)).color(scheme.on_surface_variant)
                                        .modifier(watch(geometry, "body").semantics(SemanticsArgs { test_tag: Some("dialog-figure-body".into()), merge_descendants: true,
..Default::default() }).offset(Dp(24.0 * scale), Dp(280.0 * scale)).width(Dp(364.0 * scale)));
                                } else {
                                    spacer().modifier(watch(geometry, "divider").semantics(SemanticsArgs { test_tag: Some("dialog-figure-divider".into()), merge_descendants: true,
..Default::default() }).offset(Dp(if mode == 0 {27.0} else {24.0} * scale), Dp(if mode == 0 {290.0} else {295.0} * scale))
                                        .size(Dp(if mode == 0 {358.0} else {364.0} * scale), Dp(scale)).background(scheme.outline_variant));
                                    for (row, caption) in ["ui-from", "To"].into_iter().enumerate() {
                                        if mode == 0 && row == 1 {continue;}
                                        let top = if mode == 0 {318.0} else {316.0 + row as f64 * 104.0};
                                        text().content(I18N.text(caption)).style(dialog_type_style(12.0, 16.0, scale)).color(scheme.on_surface_variant)
                                            .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(top * scale)));
                                        for column in 0..2 {
                                            dialog_scene_input(24.0 + column as f64 * 185.0,
                                                if mode == 0 {343.0} else if row == 0 {344.0} else {447.0},
                                                177.0, "ui-label".into(),
                                                if row == 0 && column == 0 {"ui-input"} else {"ui-all-day"}.into(),
                                                I18N.format("ui-full-spec-value-value-value", [("v0", (mode).into()), ("v1", I18N.text(&caption).into()), ("v2", I18N.text(&(if column == 0 {"ui-date-7089"} else {"ui-time"})).into())]), true, scale);
                                        }
                                    }
                                    if mode == 2 {
                                        checkbox().checked(all_day.get()).size(Dp(18.0 * scale))
                                            .on_toggle(move |next| all_day.set(next)).accessibility_label(I18N.text("ui-measurement-all-day"))
                                            .modifier(Modifier::new().offset(Dp(28.0 * scale), Dp(513.0 * scale)).size(Dp(48.0 * scale), Dp(48.0 * scale)));
                                        text().content(I18N.text("ui-all-day")).style(dialog_type_style(16.0, 24.0, scale)).color(scheme.on_surface_variant)
                                            .modifier(Modifier::new().offset(Dp(80.0 * scale), Dp(525.0 * scale)));
                                        text().content(I18N.text("ui-timezone")).style(dialog_type_style(12.0, 16.0, scale)).color(scheme.on_surface_variant)
                                            .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(572.0 * scale)));
                                        dialog_scene_input(24.0, 599.0, 364.0, "ui-label".into(), "ui-input".into(), "ui-measurement-timezone".into(), true, scale);
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
                .semantics(SemanticsArgs {
                    test_tag: Some("dialog-figure-header".into()),
                    merge_descendants: true,
                    ..Default::default()
                })
                .offset(Dp::ZERO, Dp(y * scale))
                .size(Dp(412.0 * scale), Dp(height * scale)),
        )
        .children(move || {
            button()
                .text()
                .accessibility_label(I18N.text("ui-close-spec-dialog"))
                .on_click_shared(close)
                .modifier(
                    watch(geometry, "close")
                        .semantics(SemanticsArgs {
                            test_tag: Some("dialog-figure-close".into()),
                            merge_descendants: true,
                            ..Default::default()
                        })
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
                .content(I18N.text(&(title)))
                .style(dialog_type_style(22.0, 28.0, scale))
                .color(scheme.on_surface)
                .modifier(
                    watch(geometry, "headline")
                        .semantics(SemanticsArgs {
                            test_tag: Some("dialog-figure-headline".into()),
                            merge_descendants: true,
                            ..Default::default()
                        })
                        .offset(Dp(56.0 * scale), Dp((center - 14.0) * scale))
                        .width(Dp(270.0 * scale)),
                );
            boxed()
                .modifier(
                    watch(geometry, "confirm")
                        .semantics(SemanticsArgs {
                            test_tag: Some("dialog-figure-confirm".into()),
                            merge_descendants: true,
                            ..Default::default()
                        })
                        .offset(Dp(336.0 * scale), Dp((center - 20.0) * scale))
                        .size(Dp(64.0 * scale), Dp(40.0 * scale)),
                )
                .children(move || {
                    dialog_scene_action(
                        "ui-save".into(),
                        "ui-save-spec-event".into(),
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
                    "ui-container",
                    244.0,
                    254.0,
                    container.x + 8.0 * scale,
                    270.0 * scale,
                ),
                (
                    2,
                    "ui-header",
                    244.0,
                    181.0,
                    header.x,
                    header.y + header.h / 2.0,
                ),
                (
                    3,
                    "ui-close-affordance",
                    322.0,
                    44.0,
                    close.x + close.w / 2.0,
                    close.y,
                ),
                (
                    4,
                    "ui-headline",
                    455.0,
                    44.0,
                    headline.x + 105.0 * scale,
                    headline.y,
                ),
                (
                    5,
                    "ui-text-button",
                    662.0,
                    44.0,
                    confirm.x + confirm.w / 2.0,
                    confirm.y,
                ),
                (6, "ui-divider", 747.0, 385.0, 705.0 * scale, 401.0 * scale),
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
        legend(
            [
                "ui-container",
                "ui-header",
                "ui-icon-close-affordance",
                "ui-headline-optional",
                "ui-text-button",
                "ui-divider-optional",
            ]
            .map(String::from)
            .to_vec(),
        );
        gap(Dp(56.0));
        subheading("ui-full-screen-dialog-color".into());
        gap(Dp(24.0));
        full_spec_figure(1);
        gap(Dp(24.0));
        caption("ui-full-screen-dialog-color-roles-used-for-light-and-dark-themes");
        gap(Dp(12.0));
        legend(
            ["Surface container high", "On surface", "On surface", "Primary", "On surface variant"]
                .map(String::from)
                .to_vec(),
        );
        gap(Dp(44.0));
        subheading("ui-full-screen-dialog-measurements".into());
        gap(Dp(24.0));
        full_spec_figure(2);
        gap(Dp(24.0));
        prose("ui-full-screen-dialog-padding-and-size-measurements");
        gap(Dp(24.0));
        spec_table(vec!["ui-attribute".into(), "ui-value".into()], vec![
            vec!["ui-container-shape".into(), "ui-0dp-corner-radius".into()],
            vec!["ui-container-height".into(), "ui-dynamic".into()],
            vec!["ui-container-width".into(), "ui-container-width-max-560dp".into()],
            vec!["ui-header-height".into(), "56dp".into()],
            vec!["ui-header-width".into(), "ui-container-width".into()],
            vec!["ui-headline-text-alignment".into(), "ui-start-aligned".into()],
            vec!["ui-divider-height".into(), "1dp".into()],
            vec!["ui-icon-close-affordance-size".into(), "24dp".into()],
            vec!["ui-bottom-action-bar-height".into(), "56dp".into()],
            vec!["ui-bottom-action-bar-width".into(), "ui-container-width".into()],
            vec!["ui-top-left-right-padding".into(), "24dp".into()],
            vec!["ui-padding-between-elements".into(), "8dp".into()],
        ]);
        gap(Dp(24.0));
        caption("ui-the-official-measurement-diagram-labels-the-header-region-as-64dp-while-the-official-table");
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
                                label: Some(I18N.text("ui-full-screen-dialog-figure")),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        full_screen_dialog(Callback::new(move || visible.set(false)))
                            .title(I18N.text("ui-new-event"))
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
                        "ui-event-saved"
                    } else {
                        "ui-dialog-closed"
                    });
                    gap(Dp(16.0));
                    button()
                        .outlined()
                        .on_click(move || {
                            saved.set(false);
                            visible.set(true);
                        })
                        .child(move || {
                            text().content(I18N.text("ui-show-full-screen-dialog"));
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
                .label(I18N.text("ui-event-name"))
                .initial_text(draft.with(|value| value.name.clone()))
                .accessibility_label(I18N.text("ui-event-name"))
                .on_change(move |next: String| {
                    draft.with_mut(|value| value.name = next.clone());
                    next
                })
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(24.0));
            text_field()
                .label(I18N.text("ui-from"))
                .initial_text(draft.with(|value| value.from.clone()))
                .accessibility_label(I18N.text("ui-from"))
                .on_change(move |next: String| {
                    draft.with_mut(|value| value.from = next.clone());
                    next
                })
                .modifier(Modifier::new().fill_max_width());
            gap(Dp(16.0));
            text_field()
                .label(I18N.text("ui-to"))
                .initial_text(draft.with(|value| value.to.clone()))
                .accessibility_label(I18N.text("ui-to"))
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
                        .accessibility_label(I18N.text("ui-all-day-7091"));
                    text().content(I18N.text("ui-all-day-7091"));
                });
            gap(Dp(16.0));
            prose("ui-time-zone");
            text_field()
                .label(I18N.text("ui-time-zone"))
                .initial_text(draft.with(|value| value.time_zone.clone()))
                .accessibility_label(I18N.text("ui-time-zone"))
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
