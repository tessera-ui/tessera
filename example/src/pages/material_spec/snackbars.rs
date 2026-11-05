//! Snackbar specification figures and queued notifications.
//!
//! ## Usage
//!
//! Show brief process updates with optional undo and dismiss actions.

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "snackbars_tests.rs"]
mod regression_tests;

use std::fmt::Write as _;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::{RoundedCorner, Shape},
    snackbar::{
        SnackbarData, SnackbarDuration, SnackbarHostState, SnackbarRequest, snackbar, snackbar_host,
    },
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, TextStyle, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, srgb, subheading, type_style,
};

const ANATOMY: u8 = 0;
const GUIDE: u8 = 1;
const COLOR: u8 = 2;
const MEASURE: u8 = 3;
const PLAIN: u8 = 4;

const DO: u8 = 0;
const CAUTION: u8 = 1;
const DONT: u8 = 2;
const NOTE: u8 = 3;

const HERO: u8 = 0;
const ARCHIVED: u8 = 1;
const DIALOG: u8 = 2;
const CONTENT_DO: u8 = 3;
const NAV_DONT: u8 = 4;
const FULL: u8 = 5;
const FAB_DO: u8 = 6;
const FAB_FRONT: u8 = 7;
const FAB_BEHIND: u8 = 8;
const FAB_MOVE: u8 = 9;
const STACK: u8 = 10;
const COMPACT: u8 = 11;
const TEXT_LABEL: u8 = 12;
const TEXT_ONE: u8 = 13;
const TEXT_TWO: u8 = 14;
const ICON: u8 = 15;
const STYLED: u8 = 16;
const SOLID: u8 = 17;
const SAME_COLOR: u8 = 18;
const ELEVATED: u8 = 19;
const TRANSLUCENT: u8 = 20;
const CAPSULE: u8 = 21;
const LONG_ACTION: u8 = 22;
const UNDO: u8 = 23;
const DISMISS: u8 = 24;
const FOCUS_DO: u8 = 25;
const FOCUS_DONT: u8 = 26;
const FOCUS_RETURN: u8 = 27;
const FOCUS_NEXT: u8 = 28;
const A11Y_DARK: u8 = 29;
const RETRY: u8 = 30;
const TABLET_MEDIUM: u8 = 31;
const TABLET_LEFT: u8 = 32;
const TABLET_CENTER: u8 = 33;
const TABLET_FLUSH: u8 = 34;
const TABLET_SIDE: u8 = 35;
const TABLET_EXTENDED: u8 = 36;

const PHONE_W: f64 = 240.0;
const PHONE_H: f64 = 420.0;
const TABLET_W: f64 = 520.0;
const TABLET_H: f64 = 300.0;
const NAV_H: f64 = 56.0;
const NAV_Y: f64 = PHONE_H - NAV_H;
const FRAME: f64 = 6.0;

const LAVENDER: Color = Color::from_rgb(0.7454042, 0.7304607, 0.9386857);
const SLATE: Color = Color::from_rgb(0.0703601, 0.0666259, 0.0953075);
const DOT_STAGE: Color = Color::from_rgb(0.9911021, 0.9646862, 1.0);
const DARK_STAGE: Color = Color::from_rgb(0.0080003, 0.0071827, 0.0103978);

#[tessera]
pub(super) fn snackbars_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "Snackbar",
        "Snackbars show short updates about app processes at the bottom of the screen",
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
    .route_optional(route)
    .intro(anatomy)
    .page_heading_levels(std::collections::BTreeMap::from([("Accessibility", 3)]))
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(resources),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: Some(RenderSlot::new(|| {
                phone_guide(HERO, NOTE, "");
            })),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(guideline_anatomy),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(responsive),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Accessibility requirements on web",
                    content: RenderSlot::new(web_requirements),
                },
                SpecSection {
                    title: "Focus",
                    content: RenderSlot::new(focus),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(scheme.on_surface_variant)
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
}

#[tessera]
fn bullets(items: Vec<&'static str>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for &item in &items {
                row().children(move || {
                    text()
                        .content("•")
                        .style(type_style(16.0, 24.0))
                        .modifier(Modifier::new().width(Dp(20.0)));
                    text()
                        .content(item)
                        .style(type_style(16.0, 24.0))
                        .modifier(Modifier::new().weight(1.0));
                });
                gap(Dp(12.0));
            }
        });
}

#[tessera]
fn numbered(items: Vec<&'static str>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, &item) in items.iter().enumerate() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .size(Dp(24.0), Dp(24.0))
                                    .background_with_shape(scheme.on_surface, Shape::CAPSULE),
                            )
                            .children(move || {
                                text()
                                    .content((index + 1).to_string())
                                    .style(type_style(12.0, 16.0))
                                    .color(scheme.surface);
                            });
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        text()
                            .content(item)
                            .style(type_style(16.0, 24.0))
                            .color(scheme.on_surface_variant);
                    });
                gap(Dp(12.0));
            }
        });
}

fn top_rounded() -> Shape {
    let round = RoundedCorner::manual(Dp(24.0), 3.0);
    let square = RoundedCorner::manual(Dp::ZERO, 3.0);
    Shape::RoundedRectangle {
        top_left: round,
        top_right: round,
        bottom_right: square,
        bottom_left: square,
    }
}

fn bottom_rounded() -> Shape {
    let round = RoundedCorner::manual(Dp(24.0), 3.0);
    let square = RoundedCorner::manual(Dp::ZERO, 3.0);
    Shape::RoundedRectangle {
        top_left: square,
        top_right: square,
        bottom_right: round,
        bottom_left: round,
    }
}

/// Draws a tinted figure area; static figures merge into one image description.
#[tessera]
fn stage(
    height: f64,
    tint: Color,
    #[prop(into)] label: String,
    square_bottom: Option<bool>,
    live: Option<bool>,
    content: Option<RenderSlot>,
) {
    let shape = if square_bottom.unwrap_or(false) {
        top_rounded()
    } else {
        Shape::rounded_rectangle(Dp(24.0))
    };
    let modifier = Modifier::new()
        .fill_max_width()
        .height(Dp(height))
        .clip_shape(shape)
        .background_with_shape(tint, shape);
    let modifier = if label.is_empty() {
        modifier
    } else if live.unwrap_or(false) {
        modifier.semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(label),
            ..Default::default()
        })
    } else {
        modifier.semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some(label),
            merge_descendants: true,
            ..Default::default()
        })
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            if let Some(content) = content {
                content.render();
            }
        });
}

/// Shows a figure with an optional Do, Caution or Don't strip and its note.
#[tessera]
fn guide(
    tone: u8,
    height: f64,
    tint: Color,
    #[prop(into)] label: String,
    #[prop(into)] note: String,
    figure: Option<RenderSlot>,
) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            stage(height, tint, label.clone())
                .square_bottom(tone != NOTE)
                .content_optional(figure);
            if tone != NOTE {
                advice(tone);
            }
            if !note.is_empty() {
                gap(Dp(16.0));
                caption(note.clone());
            }
        });
}

#[tessera]
fn advice(tone: u8) {
    let (background, ink, label) = match tone {
        DO => (srgb(52, 190, 82), srgb(0, 33, 10), "Do"),
        CAUTION => (srgb(255, 207, 36), srgb(36, 26, 0), "Caution"),
        _ => (srgb(255, 99, 71), srgb(48, 6, 0), "Don’t"),
    };
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(34.0))
                .background_with_shape(background, bottom_rounded())
                .padding_symmetric(Dp(12.0), Dp::ZERO),
        )
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            match tone {
                DO => {
                    icon().vector(filled::CHECK_SVG).size(Dp(18.0)).tint(ink);
                }
                CAUTION => {
                    icon()
                        .vector(filled::PRIORITY_HIGH_SVG)
                        .size(Dp(18.0))
                        .tint(ink);
                }
                _ => {
                    icon().vector(filled::CLOSE_SVG).size(Dp(18.0)).tint(ink);
                }
            }
            spacer().modifier(Modifier::new().width(Dp(8.0)));
            text()
                .content(label)
                .style(type_style(14.0, 20.0))
                .color(ink);
        });
}

/// Places two guidance figures side by side, wrapping on narrow layouts.
#[tessera]
fn pair(first: Option<RenderSlot>, second: Option<RenderSlot>) {
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(32.0))
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for slot in [first, second].into_iter().flatten() {
                column()
                    .modifier(Modifier::new().width(Dp(360.0)))
                    .children_shared(slot);
            }
        });
}

/// One snackbar in a device mock, positioned in screen coordinates.
#[derive(Clone, Copy, Default, PartialEq)]
struct Bar {
    x: f64,
    y: f64,
    width: f64,
    message: &'static str,
    action: Option<&'static str>,
    close: bool,
    icon: bool,
    styled: bool,
    capsule: bool,
    square: bool,
    translucent: bool,
    same_color: bool,
    elevated: bool,
    stacked: bool,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Phone {
    wide: bool,
    dark: bool,
    photos: bool,
    nav: bool,
    toolbar: bool,
    fab: Option<(f64, f64)>,
    fab_front: bool,
    ghost: Option<(f64, f64)>,
    dialog: bool,
    ring: Option<(f64, f64, f64, f64)>,
    pins: [Option<(f64, f64)>; 2],
    bars: [Option<Bar>; 2],
}

fn bar(y: f64, message: &'static str, action: Option<&'static str>) -> Bar {
    Bar {
        x: 12.0,
        y,
        width: PHONE_W - 24.0,
        message,
        action,
        ..Default::default()
    }
}

fn one(bar: Bar) -> [Option<Bar>; 2] {
    [Some(bar), None]
}

fn phone_spec(scene: u8) -> Phone {
    let above_nav = NAV_Y - 52.0;
    let list = Phone {
        nav: true,
        ..Default::default()
    };
    let photos = Phone {
        nav: true,
        photos: true,
        ..Default::default()
    };
    let tablet = Phone {
        wide: true,
        ..Default::default()
    };
    let tablet_bar = |x: f64, width: f64, message: &'static str| Bar {
        x,
        y: TABLET_H - 60.0,
        width,
        message,
        action: Some("Undo"),
        ..Default::default()
    };
    match scene {
        HERO => Phone {
            bars: one(Bar {
                close: true,
                ..bar(above_nav, "Email archived", Some("Undo"))
            }),
            ..list
        },
        ARCHIVED | UNDO | TEXT_ONE => Phone {
            bars: one(bar(above_nav, "Email archived", Some("Undo"))),
            ..list
        },
        DIALOG => Phone {
            dialog: true,
            ..list
        },
        CONTENT_DO | SOLID => Phone {
            bars: one(bar(above_nav, "Photo saved", Some("Share"))),
            ..photos
        },
        NAV_DONT => Phone {
            bars: one(bar(NAV_Y + 4.0, "Photo saved", Some("Share"))),
            ..photos
        },
        FULL => Phone {
            fab: Some((PHONE_W - 72.0, NAV_Y - 72.0)),
            bars: one(Bar {
                x: 0.0,
                width: PHONE_W,
                square: true,
                ..bar(NAV_Y - 10.0, "Email archived", Some("Undo"))
            }),
            ..list
        },
        FAB_DO => Phone {
            fab: Some((PHONE_W - 72.0, NAV_Y - 72.0)),
            bars: one(bar(NAV_Y - 132.0, "Email archived", Some("Undo"))),
            ..list
        },
        FAB_FRONT | FAB_BEHIND => Phone {
            fab: Some((PHONE_W - 72.0, NAV_Y - 72.0)),
            fab_front: scene == FAB_BEHIND,
            bars: one(bar(NAV_Y - 64.0, "Email archived", Some("Undo"))),
            ..list
        },
        FAB_MOVE => Phone {
            fab: Some((PHONE_W - 72.0, NAV_Y - 132.0)),
            ghost: Some((PHONE_W - 72.0, NAV_Y - 72.0)),
            bars: one(bar(NAV_Y - 56.0, "Email archived", Some("Undo"))),
            ..list
        },
        STACK => Phone {
            bars: [
                Some(bar(above_nav - 52.0, "Email archived", Some("Undo"))),
                Some(bar(above_nav, "Photo saved", Some("Undo"))),
            ],
            ..list
        },
        COMPACT => Phone {
            nav: false,
            bars: one(bar(
                PHONE_H - 76.0,
                "Your photo was saved to\nthe Vacation album.",
                Some("Undo"),
            )),
            ..list
        },
        TEXT_LABEL => Phone {
            nav: false,
            bars: one(bar(PHONE_H - 60.0, "Saved in Vacation album", None)),
            ..photos
        },
        TEXT_TWO => Phone {
            bars: one(bar(
                NAV_Y - 72.0,
                "Email archived.\nFind it in Archive.",
                Some("Undo"),
            )),
            ..list
        },
        ICON => Phone {
            bars: one(Bar {
                icon: true,
                ..bar(above_nav, "Connection lost", Some("Retry"))
            }),
            ..list
        },
        STYLED => Phone {
            bars: one(Bar {
                styled: true,
                ..bar(above_nav, "Saved in Vacation album", None)
            }),
            ..photos
        },
        SAME_COLOR => Phone {
            bars: one(Bar {
                same_color: true,
                ..bar(above_nav, "Email archived", Some("Undo"))
            }),
            ..list
        },
        ELEVATED => Phone {
            bars: one(Bar {
                elevated: true,
                ..bar(above_nav, "Email archived", Some("Undo"))
            }),
            ..list
        },
        TRANSLUCENT => Phone {
            bars: one(Bar {
                translucent: true,
                ..bar(above_nav, "Photo saved", Some("Share"))
            }),
            ..photos
        },
        CAPSULE => Phone {
            bars: one(Bar {
                capsule: true,
                ..bar(above_nav, "Email archived", Some("Undo"))
            }),
            ..list
        },
        LONG_ACTION => Phone {
            bars: one(Bar {
                stacked: true,
                ..bar(
                    NAV_Y - 94.0,
                    "Your photo was saved to\nthe Vacation album.",
                    Some("Open photo collection"),
                )
            }),
            ..list
        },
        DISMISS => Phone {
            bars: one(bar(above_nav, "Email archived", Some("Dismiss"))),
            ..list
        },
        FOCUS_DO => Phone {
            nav: false,
            ring: Some((4.0, 228.0, PHONE_W - 8.0, 56.0)),
            bars: one(bar(PHONE_H - 60.0, "Email archived", Some("Undo"))),
            ..list
        },
        FOCUS_DONT => Phone {
            nav: false,
            ring: Some((4.0, 284.0, PHONE_W - 8.0, 56.0)),
            bars: one(Bar {
                stacked: true,
                ..bar(
                    PHONE_H - 120.0,
                    "Your email was archived.\nFind it in Archive.",
                    Some("Open archive"),
                )
            }),
            ..list
        },
        FOCUS_RETURN => Phone {
            dark: true,
            nav: false,
            photos: true,
            toolbar: true,
            ring: Some((6.0, NAV_Y + 8.0, 40.0, 40.0)),
            pins: [Some((200.0, NAV_Y - 80.0)), Some((50.0, NAV_Y + 2.0))],
            bars: one(bar(NAV_Y - 52.0, "Photo deleted", Some("Undo"))),
            ..Default::default()
        },
        FOCUS_NEXT => Phone {
            dark: true,
            nav: false,
            photos: true,
            toolbar: true,
            ring: Some((2.0, 30.0, 78.0, 78.0)),
            pins: [Some((200.0, NAV_Y - 80.0)), Some((84.0, 34.0))],
            bars: one(bar(NAV_Y - 52.0, "Photo deleted", Some("Undo"))),
            ..Default::default()
        },
        A11Y_DARK => Phone {
            dark: true,
            bars: one(bar(above_nav, "Email sent", Some("Undo"))),
            ..list
        },
        RETRY => Phone {
            bars: one(bar(
                NAV_Y - 72.0,
                "Can’t send photo.\nRetry in 5 seconds.",
                Some("Retry"),
            )),
            ..photos
        },
        TABLET_MEDIUM => Phone {
            bars: one(tablet_bar(
                16.0,
                TABLET_W - 120.0,
                "Your photo was saved to the Vacation album",
            )),
            ..tablet
        },
        TABLET_LEFT => Phone {
            bars: one(tablet_bar(16.0, 300.0, "Email archived")),
            ..tablet
        },
        TABLET_CENTER => Phone {
            bars: one(tablet_bar(
                (TABLET_W - 300.0) / 2.0,
                300.0,
                "Email archived",
            )),
            ..tablet
        },
        TABLET_FLUSH => Phone {
            bars: one(Bar {
                square: true,
                ..tablet_bar(0.0, 300.0, "Email archived")
            }),
            ..tablet
        },
        TABLET_SIDE => Phone {
            bars: [
                Some(tablet_bar(16.0, 236.0, "Email archived")),
                Some(tablet_bar(268.0, 236.0, "Photo saved")),
            ],
            ..tablet
        },
        TABLET_EXTENDED => Phone {
            bars: one(tablet_bar(
                16.0,
                TABLET_W - 32.0,
                "Your photo was saved to the Vacation album and is ready to share",
            )),
            ..tablet
        },
        _ => list,
    }
}

/// Renders a snackbar guideline scene inside a phone or tablet frame.
#[tessera]
fn phone_guide(scene: u8, tone: u8, #[prop(into)] note: String) {
    let spec = phone_spec(scene);
    let height = if spec.wide { 360.0 } else { 480.0 };
    let tint = if spec.dark { SLATE } else { LAVENDER };
    let label = if note.is_empty() {
        "Snackbar at the bottom of a mobile device".to_owned()
    } else {
        note.clone()
    };
    guide(tone, height, tint, label, note).figure(move || {
        phone(scene);
    });
}

#[tessera]
fn phone(scene: u8) {
    if phone_spec(scene).dark {
        material_theme()
            .theme(|| MaterialTheme::from_seed(ACCENT, true))
            .child(move || {
                phone_body(scene);
            });
    } else {
        phone_body(scene);
    }
}

#[tessera]
fn phone_body(scene: u8) {
    let spec = phone_spec(scene);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (width, height) = if spec.wide {
        (TABLET_W, TABLET_H)
    } else {
        (PHONE_W, PHONE_H)
    };
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(width + FRAME * 2.0), Dp(height + FRAME * 2.0))
                .background_with_shape(srgb(176, 172, 186), Shape::rounded_rectangle(Dp(28.0)))
                .padding_all(Dp(FRAME)),
        )
        .children(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .clip_shape(Shape::rounded_rectangle(Dp(22.0)))
                        .background(scheme.surface),
                )
                .children(move || {
                    if spec.photos {
                        photo_grid(spec.wide);
                    } else {
                        message_list();
                    }
                    if spec.dialog {
                        dialog_mock();
                    }
                    if spec.nav {
                        nav_bar();
                    }
                    if spec.toolbar {
                        photo_toolbar();
                    }
                    if let Some((x, y)) = spec.ghost {
                        boxed().modifier(
                            Modifier::new()
                                .offset(Dp(x), Dp(y))
                                .size(Dp(56.0), Dp(56.0))
                                .border_with_shape(
                                    Dp(1.0),
                                    scheme.outline,
                                    Shape::rounded_rectangle(Dp(16.0)),
                                ),
                        );
                    }
                    if let Some(rect) = spec.ring {
                        ring(rect);
                    }
                    if let Some(position) = spec.fab
                        && !spec.fab_front
                    {
                        fab(position);
                    }
                    for placed in spec.bars.into_iter().flatten() {
                        mock_bar(placed);
                    }
                    if let Some(position) = spec.fab
                        && spec.fab_front
                    {
                        fab(position);
                    }
                    for (index, position) in spec.pins.into_iter().enumerate() {
                        if let Some(position) = position {
                            pin(index + 1, position);
                        }
                    }
                });
        });
}

#[tessera]
fn message_list() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            text()
                .content("9:30")
                .style(type_style(11.0, 16.0))
                .color(scheme.on_surface)
                .modifier(Modifier::new().padding(Padding::new(
                    Dp(20.0),
                    Dp(8.0),
                    Dp(20.0),
                    Dp(4.0),
                )));
            for (index, (name, line)) in [
                ("Ali Connors", "Brunch this weekend?"),
                ("Sofia Sacchi", "Do you remember that cafe"),
                ("Carmen Villanueva", "Graduación de Inés"),
                ("Alejandro Ortega", "Pre-sale concert tickets"),
                ("Anna Russo", "Photos from the trip"),
                ("Kai Tanaka", "Lunch on Friday"),
            ]
            .into_iter()
            .enumerate()
            {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(56.0))
                            .padding_symmetric(Dp(14.0), Dp::ZERO),
                    )
                    .children(move || {
                        let tint = [
                            scheme.primary_container,
                            scheme.tertiary_container,
                            scheme.secondary_container,
                        ][index % 3];
                        boxed().modifier(
                            Modifier::new()
                                .size(Dp(32.0), Dp(32.0))
                                .background_with_shape(tint, Shape::Ellipse),
                        );
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        column().children(move || {
                            text()
                                .content(name)
                                .style(type_style(13.0, 18.0))
                                .color(scheme.on_surface);
                            text()
                                .content(line)
                                .style(type_style(11.0, 16.0))
                                .color(scheme.on_surface_variant);
                        });
                    });
            }
        });
}

#[tessera]
fn photo_grid(wide: bool) {
    let columns = if wide { 6 } else { 3 };
    column()
        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
            Dp(2.0),
            Dp(28.0),
            Dp(2.0),
            Dp::ZERO,
        )))
        .children(move || {
            for line in 0..5 {
                row()
                    .modifier(Modifier::new().fill_max_width().height(Dp(80.0)))
                    .children(move || {
                        for cell in 0..columns {
                            let shade = [
                                srgb(214, 160, 172),
                                srgb(126, 148, 112),
                                srgb(188, 170, 142),
                                srgb(98, 118, 156),
                                srgb(172, 120, 98),
                            ][(line * columns + cell) % 5];
                            boxed().modifier(
                                Modifier::new()
                                    .weight(1.0)
                                    .fill_max_height()
                                    .padding_all(Dp(1.0))
                                    .background(shade),
                            );
                        }
                    });
            }
        });
}

#[tessera]
fn nav_bar() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .main_axis_alignment(MainAxisAlignment::SpaceAround)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp::ZERO, Dp(NAV_Y))
                .size(Dp(PHONE_W), Dp(NAV_H))
                .background(scheme.surface_container),
        )
        .children(move || {
            for (index, label) in ["Inbox", "Chat", "Video", "Groups"].into_iter().enumerate() {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .size(Dp(48.0), Dp(26.0))
                                    .background_with_shape(
                                        if index == 0 {
                                            scheme.secondary_container
                                        } else {
                                            scheme.surface_container
                                        },
                                        Shape::CAPSULE,
                                    ),
                            )
                            .children(move || match index {
                                0 => {
                                    icon().vector(filled::INBOX_SVG).size(Dp(18.0));
                                }
                                1 => {
                                    icon().vector(filled::EMAIL_SVG).size(Dp(18.0));
                                }
                                2 => {
                                    icon().vector(filled::VIDEOCAM_SVG).size(Dp(18.0));
                                }
                                _ => {
                                    icon().vector(filled::ACCOUNT_CIRCLE_SVG).size(Dp(18.0));
                                }
                            });
                        text()
                            .content(label)
                            .style(type_style(10.0, 14.0))
                            .color(scheme.on_surface_variant);
                    });
            }
        });
}

#[tessera]
fn photo_toolbar() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp::ZERO, Dp(NAV_Y))
                .size(Dp(PHONE_W), Dp(NAV_H))
                .background(scheme.surface_container)
                .padding_symmetric(Dp(8.0), Dp::ZERO),
        )
        .children(move || {
            for index in 0..4 {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().size(Dp(36.0), Dp(36.0)))
                    .children(move || match index {
                        0 => {
                            icon().vector(filled::MORE_VERT_SVG).size(Dp(18.0));
                        }
                        1 => {
                            icon().vector(filled::EDIT_SVG).size(Dp(18.0));
                        }
                        2 => {
                            icon().vector(filled::MIC_SVG).size(Dp(18.0));
                        }
                        _ => {
                            icon().vector(filled::IMAGE_SVG).size(Dp(18.0));
                        }
                    });
            }
            spacer().modifier(Modifier::new().weight(1.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .background_with_shape(
                            scheme.primary_container,
                            Shape::rounded_rectangle(Dp(12.0)),
                        ),
                )
                .children(|| {
                    icon().vector(filled::ADD_SVG).size(Dp(18.0));
                });
        });
}

#[tessera]
fn fab(position: (f64, f64)) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .modifier(
            Modifier::new()
                .offset(Dp(position.0), Dp(position.1))
                .size(Dp(56.0), Dp(56.0)),
        )
        .style(scheme.primary_container.into())
        .shape(Shape::rounded_rectangle(Dp(16.0)))
        .elevation(Dp(3.0))
        .content_alignment(Alignment::Center)
        .child(move || {
            icon()
                .vector(filled::ADD_SVG)
                .size(Dp(24.0))
                .tint(scheme.on_primary_container);
        });
}

#[tessera]
fn ring(rect: (f64, f64, f64, f64)) {
    boxed().modifier(
        Modifier::new()
            .offset(Dp(rect.0), Dp(rect.1))
            .size(Dp(rect.2), Dp(rect.3))
            .border_with_shape(
                Dp(3.0),
                srgb(232, 140, 230),
                Shape::rounded_rectangle(Dp(12.0)),
            ),
    );
}

#[tessera]
fn pin(number: usize, position: (f64, f64)) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(position.0), Dp(position.1))
                .size(Dp(22.0), Dp(22.0))
                .background_with_shape(srgb(232, 140, 230), Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(12.0, 16.0))
                .color(srgb(29, 27, 32));
        });
}

#[tessera]
fn dialog_mock() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().modifier(
        Modifier::new()
            .size(Dp(PHONE_W), Dp(PHONE_H))
            .background(scheme.scrim.with_alpha(0.32)),
    );
    surface()
        .modifier(
            Modifier::new()
                .offset(Dp(20.0), Dp(130.0))
                .width(Dp(PHONE_W - 40.0)),
        )
        .style(scheme.surface_container_high.into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_width().padding_all(Dp(20.0)))
                .children(move || {
                    text()
                        .content("Sign in to continue?")
                        .style(type_style(18.0, 24.0))
                        .color(scheme.on_surface);
                    gap(Dp(12.0));
                    text()
                        .content("Sign in to sync your mail. Dismissing keeps you on this screen.")
                        .style(type_style(12.0, 16.0))
                        .color(scheme.on_surface_variant);
                    gap(Dp(20.0));
                    row()
                        .main_axis_alignment(MainAxisAlignment::End)
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            for label in ["Cancel", "Sign in"] {
                                text()
                                    .content(label)
                                    .style(type_style(13.0, 18.0))
                                    .color(scheme.primary)
                                    .modifier(
                                        Modifier::new().padding_symmetric(Dp(10.0), Dp::ZERO),
                                    );
                            }
                        });
                });
        });
}

/// Draws a static snackbar without focusable controls for guideline scenes.
#[tessera]
fn mock_bar(placed: Bar) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let shape = if placed.capsule {
        Shape::CAPSULE
    } else if placed.square {
        Shape::rounded_rectangle(Dp::ZERO)
    } else {
        Shape::rounded_rectangle(Dp(4.0))
    };
    let container = if placed.translucent {
        scheme.inverse_surface.with_alpha(0.85)
    } else {
        scheme.inverse_surface
    };
    let action_color = if placed.same_color {
        scheme.inverse_on_surface
    } else {
        scheme.inverse_primary
    };
    surface()
        .modifier(
            Modifier::new()
                .offset(Dp(placed.x), Dp(placed.y))
                .width(Dp(placed.width)),
        )
        .style(container.into())
        .content_color(scheme.inverse_on_surface)
        .shape(shape)
        .elevation(Dp(6.0))
        .child(move || {
            let padding = Padding::new(
                Dp(16.0),
                Dp(10.0),
                Dp(if placed.close { 4.0 } else { 8.0 }),
                Dp(10.0),
            );
            if placed.stacked {
                column()
                    .modifier(Modifier::new().fill_max_width().padding(padding))
                    .children(move || {
                        bar_text(placed);
                        gap(Dp(8.0));
                        row()
                            .main_axis_alignment(MainAxisAlignment::End)
                            .modifier(Modifier::new().fill_max_width())
                            .children(move || {
                                if let Some(label) = placed.action {
                                    bar_action(label, action_color, placed.elevated);
                                }
                            });
                    });
            } else {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .size_in(None, None, Some(Dp(36.0)), None)
                            .padding(padding),
                    )
                    .children(move || {
                        bar_text(placed);
                        spacer().modifier(Modifier::new().weight(1.0));
                        if let Some(label) = placed.action {
                            bar_action(label, action_color, placed.elevated);
                        }
                        if placed.close {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().size(Dp(32.0), Dp(32.0)))
                                .children(move || {
                                    icon()
                                        .vector(filled::CLOSE_SVG)
                                        .size(Dp(18.0))
                                        .tint(scheme.inverse_on_surface);
                                });
                        }
                    });
            }
        });
}

#[tessera]
fn bar_text(placed: Bar) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let ink = scheme.inverse_on_surface;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            if placed.icon {
                icon()
                    .vector(filled::PRIORITY_HIGH_SVG)
                    .size(Dp(18.0))
                    .tint(ink);
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            if placed.styled {
                text()
                    .content("Saved in ")
                    .style(type_style(12.0, 16.0))
                    .color(ink);
                text()
                    .content("Vacation album")
                    .style(TextStyle {
                        font_weight: 700,
                        ..type_style(12.0, 16.0)
                    })
                    .color(scheme.inverse_primary);
            } else {
                text()
                    .content(placed.message)
                    .style(type_style(12.0, 16.0))
                    .color(ink);
            }
        });
}

#[tessera]
fn bar_action(#[prop(into)] label: String, color: Color, elevated: bool) {
    if elevated {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        surface()
            .style(scheme.surface_container_low.into())
            .shape(Shape::CAPSULE)
            .elevation(Dp(1.0))
            .child(move || {
                text()
                    .content(label.clone())
                    .style(type_style(12.0, 16.0))
                    .color(scheme.primary)
                    .modifier(Modifier::new().padding_symmetric(Dp(14.0), Dp(6.0)));
            });
    } else {
        text()
            .content(label)
            .style(TextStyle {
                font_weight: 500,
                ..type_style(12.0, 16.0)
            })
            .color(color)
            .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp(6.0)));
    }
}

/// Shows a live snackbar on a dot grid, annotated from its measured geometry.
#[tessera]
fn annotated(kind: u8, #[prop(into)] label: String, dark: Option<bool>, shape: Option<Shape>) {
    let dark = dark.unwrap_or(false);
    let geometry = remember(Geometry::new);
    let height = match kind {
        ANATOMY | GUIDE => 240.0,
        COLOR => 200.0,
        MEASURE => 260.0,
        _ => 220.0,
    };
    let shape = shape.unwrap_or(Shape::rounded_rectangle(Dp(24.0)));
    let tint = if dark { DARK_STAGE } else { DOT_STAGE };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(height))
                .clip_shape(shape)
                .background_with_shape(tint, shape)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(label.clone()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            dots(geometry, dark);
            let dismiss = format!("Dismiss {label}");
            match kind {
                MEASURE => {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            snackbar()
                                .modifier(watch(geometry, "bar").width(Dp(360.0)))
                                .message("Single-line snackbar with action")
                                .action_label("Action")
                                .on_action(|| {});
                            gap(Dp(56.0));
                            snackbar()
                                .modifier(watch(geometry, "bar2").width(Dp(360.0)))
                                .message("Single-line snackbar with icon")
                                .action_label("Action")
                                .with_dismiss_action(true)
                                .dismiss_action_label("Dismiss measured snackbar")
                                .on_dismiss(|| {});
                        });
                }
                PLAIN => {
                    snackbar()
                        .modifier(watch(geometry, "bar").width(Dp(360.0)))
                        .message("Single-line snackbar with action")
                        .action_label("Action")
                        .on_action(|| {});
                }
                _ => {
                    snackbar()
                        .modifier(watch(geometry, "bar").width(Dp(400.0)))
                        .message("Single-line snackbar with action")
                        .action_label("Action")
                        .on_action(|| {})
                        .with_dismiss_action(true)
                        .dismiss_action_label(dismiss)
                        .on_dismiss(|| {});
                }
            }
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    figure_annotations(geometry, kind, dark);
                });
        });
}

#[tessera]
fn dots(geometry: State<Geometry>, dark: bool) {
    let values = geometry.get();
    let Some(root) = values.get("figure") else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32()).0;
    let height = Dp::from_pixels_f32(root.height.to_f32()).0;
    let mut path = String::new();
    let mut y = 6.0;
    while y < height {
        let mut x = 6.0;
        while x < width {
            write!(path, "M{x} {y}h1v1h-1z").unwrap();
            x += 10.0;
        }
        y += 10.0;
    }
    let color = if dark { "#4a4458" } else { "#e8b8de" };
    annotations::lines(
        Dp(width),
        Dp(height),
        format!("<path d='{path}' fill='{color}' fill-opacity='.7'/>"),
    );
}

fn span(svg: &mut String, x: f64, y: f64, length: f64, vertical: bool) {
    let color = "#d0475f";
    if vertical {
        write!(
            svg,
            "<rect x='{x}' y='{y}' width='1' height='{length}' fill='{color}'/>\
             <rect x='{}' y='{y}' width='7' height='1' fill='{color}'/>\
             <rect x='{}' y='{}' width='7' height='1' fill='{color}'/>",
            x - 3.0,
            x - 3.0,
            y + length - 1.0
        )
        .unwrap();
    } else {
        write!(
            svg,
            "<rect x='{x}' y='{y}' width='{length}' height='1' fill='{color}'/>\
             <rect x='{x}' y='{}' width='1' height='7' fill='{color}'/>\
             <rect x='{}' y='{}' width='1' height='7' fill='{color}'/>",
            y - 3.0,
            x + length - 1.0,
            y - 3.0
        )
        .unwrap();
    }
}

fn frame(svg: &mut String, rect: Rect, color: &str) {
    annotations::dotted(svg, rect.x, rect.y, rect.w, false, color);
    annotations::dotted(svg, rect.x, rect.y + rect.h, rect.w, false, color);
    annotations::dotted(svg, rect.x, rect.y, rect.h, true, color);
    annotations::dotted(svg, rect.x + rect.w, rect.y, rect.h, true, color);
}

#[tessera]
fn figure_annotations(geometry: State<Geometry>, kind: u8, dark: bool) {
    let values = geometry.get();
    let (Some(root), Some(bar)) = (values.get("figure"), relative(&values, "bar")) else {
        return;
    };
    let ink = if dark { "#cac4d0" } else { "#49454f" };
    let mut svg = String::new();
    if kind == MEASURE {
        let Some(second) = relative(&values, "bar2") else {
            return;
        };
        write!(
            svg,
            "<rect x='{}' y='{}' width='16' height='{}' fill='#ee7da4' fill-opacity='.45'/>\
             <rect x='{}' y='{}' width='8' height='{}' fill='#ee7da4' fill-opacity='.45'/>\
             <rect x='{}' y='{}' width='12' height='{}' fill='#ee7da4' fill-opacity='.45'/>\
             <rect x='{}' y='{}' width='12' height='{}' fill='#ee7da4' fill-opacity='.45'/>",
            bar.x,
            bar.y,
            bar.h,
            bar.x + bar.w - 8.0,
            bar.y,
            bar.h,
            second.x + second.w - 48.0,
            second.y,
            second.h,
            second.x + second.w - 12.0,
            second.y,
            second.h,
        )
        .unwrap();
        span(&mut svg, bar.x - 10.0, bar.y, bar.h, true);
        span(&mut svg, bar.x, bar.y + bar.h + 6.0, 16.0, false);
        span(
            &mut svg,
            bar.x + bar.w - 8.0,
            bar.y + bar.h + 6.0,
            8.0,
            false,
        );
        span(
            &mut svg,
            second.x + second.w - 36.0,
            second.y - 8.0,
            24.0,
            false,
        );
        span(
            &mut svg,
            second.x + second.w - 48.0,
            second.y + second.h + 6.0,
            12.0,
            false,
        );
        span(
            &mut svg,
            second.x + second.w - 12.0,
            second.y + second.h + 6.0,
            12.0,
            false,
        );
        annotations::dimension(
            "48".into(),
            Dp(bar.x - 40.0),
            Dp(bar.y + bar.h / 2.0 - 10.0),
        );
        annotations::dimension("16".into(), Dp(bar.x), Dp(bar.y + bar.h + 12.0));
        annotations::dimension(
            "8".into(),
            Dp(bar.x + bar.w - 6.0),
            Dp(bar.y + bar.h + 12.0),
        );
        annotations::dimension(
            "24".into(),
            Dp(second.x + second.w - 32.0),
            Dp(second.y - 32.0),
        );
        annotations::dimension(
            "12".into(),
            Dp(second.x + second.w - 52.0),
            Dp(second.y + second.h + 12.0),
        );
        annotations::dimension(
            "12".into(),
            Dp(second.x + second.w - 14.0),
            Dp(second.y + second.h + 12.0),
        );
    } else if kind != PLAIN {
        let close = Rect {
            x: bar.x + bar.w - 36.0,
            y: bar.y + (bar.h - 24.0) / 2.0,
            w: 24.0,
            h: 24.0,
        };
        let action = Rect {
            x: bar.x + bar.w - 108.0,
            y: bar.y + (bar.h - 20.0) / 2.0,
            w: 48.0,
            h: 20.0,
        };
        let message = Rect {
            x: bar.x + 16.0,
            y: bar.y + 14.0,
            w: 208.0,
            h: bar.h - 28.0,
        };
        if kind != COLOR {
            frame(&mut svg, message, ink);
            frame(&mut svg, action, ink);
            frame(&mut svg, close, ink);
        }
        let numbers: [(usize, u8); 4] = match kind {
            ANATOMY => [(1, b'c'), (2, b'x'), (3, b'a'), (4, b't')],
            _ => [(1, b'c'), (2, b't'), (3, b'a'), (4, b'x')],
        };
        for (number, part) in numbers {
            let target = match part {
                b'c' => Rect {
                    x: bar.x + 28.0,
                    y: bar.y,
                    w: 0.0,
                    h: 0.0,
                },
                b't' => message,
                b'a' => action,
                _ => close,
            };
            let center = target.x + target.w / 2.0;
            if kind == COLOR {
                let top = bar.y - 40.0;
                annotations::marker(number, Dp(center - 12.0), Dp(top));
                annotations::dotted(
                    &mut svg,
                    center,
                    top + 32.0,
                    target.y - top - 32.0,
                    true,
                    ink,
                );
            } else if part == b'c' {
                annotations::marker(number, Dp(center - 12.0), Dp(bar.y - 64.0));
                annotations::dotted(&mut svg, center, bar.y - 32.0, 32.0, true, ink);
                write!(
                    svg,
                    "<circle cx='{center}' cy='{}' r='3.5' fill='none' stroke='{ink}' stroke-width='.7'/>",
                    bar.y
                )
                .unwrap();
            } else if part == b'x' {
                let x = bar.x + bar.w + 36.0;
                let y = target.y + target.h / 2.0;
                annotations::marker(number, Dp(x), Dp(y - 16.0));
                annotations::dotted(
                    &mut svg,
                    target.x + target.w,
                    y,
                    x - target.x - target.w,
                    false,
                    ink,
                );
            } else {
                let y = bar.y + bar.h + 32.0;
                annotations::marker(number, Dp(center - 12.0), Dp(y));
                annotations::dotted(
                    &mut svg,
                    center,
                    target.y + target.h,
                    y - target.y - target.h,
                    true,
                    ink,
                );
            }
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn anatomy() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            annotated(ANATOMY, "Snackbar anatomy");
            gap(Dp(24.0));
            numbered(vec![
                "Container",
                "Icon (optional close affordance)",
                "Action (optional)",
                "Supporting text",
            ]);
        });
}

const TOKENS: [(&str, &str, &str); 28] = [
    (
        "Enabled",
        "Snackbar container color",
        "Inverse surface · #322F35",
    ),
    (
        "Enabled",
        "Snackbar container shadow color",
        "Shadow · #000000",
    ),
    ("Enabled", "Snackbar container elevation", "Level 3 · 6dp"),
    ("Enabled", "Snackbar container shape", "Extra small · 4dp"),
    (
        "Enabled",
        "Snackbar container with single line height",
        "48dp",
    ),
    (
        "Enabled",
        "Snackbar container with two lines height",
        "68dp",
    ),
    ("Enabled", "Snackbar label text", "Label large"),
    (
        "Enabled",
        "Snackbar label text color",
        "Inverse primary · #D0BCFF",
    ),
    ("Enabled", "Snackbar label text font", "Roboto"),
    ("Enabled", "Snackbar label text line height", "20pt"),
    ("Enabled", "Snackbar label text size", "14pt"),
    ("Enabled", "Snackbar label text tracking", "0.1pt"),
    ("Enabled", "Snackbar label text weight", "500"),
    (
        "Enabled",
        "Snackbar icon color",
        "Inverse on surface · #F5EFF7",
    ),
    ("Enabled", "Snackbar icon size", "24dp"),
    ("Enabled", "Snackbar supporting text", "Body medium"),
    (
        "Enabled",
        "Snackbar supporting text color",
        "Inverse on surface · #F5EFF7",
    ),
    ("Enabled", "Snackbar supporting text font", "Roboto"),
    ("Enabled", "Snackbar supporting text line height", "20pt"),
    ("Enabled", "Snackbar supporting text size", "14pt"),
    ("Enabled", "Snackbar supporting text tracking", "0.25pt"),
    ("Enabled", "Snackbar supporting text weight", "400"),
    (
        "Hovered",
        "Snackbar hover label text color",
        "Inverse primary · #D0BCFF",
    ),
    ("Hovered", "Snackbar hover state layer opacity", "0.08"),
    (
        "Focused",
        "Snackbar focus label text color",
        "Inverse primary · #D0BCFF",
    ),
    ("Focused", "Snackbar focus state layer opacity", "0.1"),
    (
        "Pressed (ripple)",
        "Snackbar pressed label text color",
        "Inverse primary · #D0BCFF",
    ),
    (
        "Pressed (ripple)",
        "Snackbar pressed state layer opacity",
        "0.1",
    ),
];

#[tessera]
fn tokens() {
    prose(
        "Browse the component elements, attributes, tokens, and their values. Values shown are for the default light scheme; Tessera resolves the same roles from the active theme.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["State".into(), "Token".into(), "Value".into()],
        TOKENS
            .iter()
            .map(|(state, token, value)| vec![(*state).into(), (*token).into(), (*value).into()])
            .collect(),
    );
    gap(Dp(16.0));
    caption(
        "Icon state layers use Inverse on surface with the same hover, focus and pressed opacities.",
    );
}

#[tessera]
fn colors() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.",
    );
    gap(Dp(24.0));
    annotated(COLOR, "Light snackbar color roles").shape(top_rounded());
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            annotated(COLOR, "Dark snackbar color roles")
                .dark(true)
                .shape(bottom_rounded());
        });
    gap(Dp(24.0));
    caption("Snackbar color roles used for light and dark schemes:");
    gap(Dp(12.0));
    numbered(vec![
        "Inverse surface",
        "Inverse on surface",
        "Inverse primary",
        "Inverse on surface",
    ]);
}

#[tessera]
fn measurements() {
    annotated(MEASURE, "Snackbar measurements");
    gap(Dp(16.0));
    caption("Snackbar padding and size measurements");
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container height, single line".into(), "48dp".into()],
            vec!["Container height, two lines".into(), "68dp".into()],
            vec!["Leading padding".into(), "16dp".into()],
            vec!["Trailing padding with action".into(), "8dp".into()],
            vec!["Close icon size".into(), "24dp".into()],
            vec!["Close icon padding".into(), "12dp".into()],
            vec!["Supporting text vertical padding".into(), "14dp".into()],
            vec!["Maximum width (Tessera default)".into(), "600dp".into()],
        ],
    );
    gap(Dp(16.0));
    caption(
        "The two-line height follows the 68dp token used by the native Tessera snackbar. The compact guidance in Responsive layout describes the same growth as 48dp to 64dp.",
    );
}

#[tessera]
fn sample(
    #[prop(into)] name: String,
    #[prop(into)] message: String,
    #[prop(into)] action: Option<String>,
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
                snackbar()
                    .modifier(Modifier::new().fill_max_width())
                    .message(message.clone())
                    .action_label_optional(action.clone())
                    .action_on_new_line(stacked.unwrap_or(false))
                    .on_action(move || visible.set(false));
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
fn config_badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().size(Dp(20.0), Dp(20.0)).border_with_shape(
            Dp(1.0),
            srgb(202, 196, 208),
            Shape::Ellipse,
        ))
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(11.0, 16.0))
                .color(srgb(230, 224, 233));
        });
}

#[tessera]
fn configurations() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            material_theme()
                .theme(|| MaterialTheme::from_seed(ACCENT, true))
                .child(|| {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .background_with_shape(
                                    DARK_STAGE,
                                    Shape::rounded_rectangle(Dp(24.0)),
                                )
                                .padding_all(Dp(40.0)),
                        )
                        .children(|| {
                            flow_row()
                                .item_spacing(Dp(40.0))
                                .line_spacing(Dp(32.0))
                                .children(|| {
                                    for (index, (name, message, action, stacked)) in [
                                        ("Single line", "Single-line snackbar", None, false),
                                        (
                                            "Single line with action",
                                            "Single-line snackbar with action",
                                            Some("Undo"),
                                            false,
                                        ),
                                        (
                                            "Two lines",
                                            "Two-line snackbar\nwithout action",
                                            None,
                                            false,
                                        ),
                                        (
                                            "Two lines with action",
                                            "Two-line snackbar\nwith action",
                                            Some("Action"),
                                            false,
                                        ),
                                        (
                                            "Two lines with longer action",
                                            "Two-line snackbar\nwith longer action",
                                            Some("Longer action"),
                                            true,
                                        ),
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    {
                                        column().children(move || {
                                            config_badge(index + 1);
                                            gap(Dp(12.0));
                                            sample(name, message)
                                                .action_optional(action.map(str::to_owned))
                                                .stacked(stacked);
                                        });
                                    }
                                });
                        });
                });
            gap(Dp(24.0));
            numbered(vec![
                "Single line",
                "Single line with action",
                "Two lines",
                "Two lines with action",
                "Two lines with longer action",
            ]);
            gap(Dp(48.0));
            subheading("Queued notifications".to_owned());
            gap(Dp(16.0));
            prose(
                "This live Tessera example shows queued notifications one at a time. The next message appears only after the current one is acted on, dismissed or times out.",
            );
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
                            let message = data.message().to_owned();
                            let action_label = data.action_label().map(str::to_owned);
                            let on_new_line = stacked.get();
                            let action_data = data.clone();
                            let snackbar = snackbar()
                                .modifier(Modifier::new().fill_max_width())
                                .message(message)
                                .action_label_optional(action_label)
                                .action_on_new_line(on_new_line)
                                .on_action(move || {
                                    action_data.perform_action();
                                    result.set("Action performed".to_owned());
                                });
                            if data.with_dismiss_action() {
                                let dismiss_data = data.clone();
                                snackbar
                                    .with_dismiss_action(true)
                                    .dismiss_action_label("Dismiss queued snackbar")
                                    .on_dismiss(move || {
                                        dismiss_data.dismiss();
                                        result.set("Dismissed".to_owned());
                                    });
                            }
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
fn overview() {
    bullets(vec![
        "Snackbars shouldn’t interrupt the user’s experience",
        "Usually appear at the bottom of the UI",
        "Can disappear on their own or remain on screen until the user takes action",
    ]);
    gap(Dp(32.0));
    annotated(PLAIN, "Diagram of snackbar placement");
}

#[tessera]
fn resources() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Live examples on this page use the native Tessera snackbar and snackbar_host. Device scenes in Guidelines and Accessibility are static illustrations without focusable controls.",
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color",
        "Behavior: Clarified that snackbars can either appear temporarily (dismissive) or persist until the user takes an action (non-dismissive)",
    ]);
    gap(Dp(24.0));
    phone_guide(RETRY, NOTE, "Snackbars have new color mappings");
}

#[tessera]
fn usage() {
    prose(
        "Snackbars inform users of a process that an app has performed or will perform. They appear temporarily, towards the bottom of the screen.",
    );
    gap(Dp(16.0));
    prose(
        "They shouldn’t interrupt the user experience. People can browse the page content without being required to interact with the snackbar.",
    );
    gap(Dp(24.0));
    subheading("Frequency".into()).level(4);
    prose("Only one snackbar may be displayed at a time.");
    gap(Dp(24.0));
    subheading("Actions".into()).level(4);
    prose("A snackbar can contain a single action. “Dismiss” or “cancel” actions are optional.");
    gap(Dp(24.0));
    phone_guide(
        ARCHIVED,
        NOTE,
        "Snackbar showing ‘Email archived’ text with an ‘Undo’ text button.",
    );
    gap(Dp(40.0));
    subheading("Similar components".into());
    prose(
        "Dialogs are also designed to show important messages. Choose the right component based on the importance of the message. This component messaging strategy can help avoid overusing snackbars.",
    );
    gap(Dp(24.0));
    phone_guide(DIALOG, NOTE, "Dialogs require immediate action");
    gap(Dp(40.0));
    subheading("When to use snackbars".into());
    prose(
        "Snackbars communicate messages that are minimally interruptive and don’t require user action.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Component".into(), "Priority".into(), "User action".into()],
        vec![
            vec![
                "Snackbar".into(),
                "Low priority".into(),
                "Optional: Snackbars disappear automatically".into(),
            ],
            vec![
                "Dialog".into(),
                "High priority".into(),
                "Required: Dialogs block app usage until the user takes a dialog action or exits the dialog (if available)".into(),
            ],
        ],
    );
    gap(Dp(40.0));
    subheading("Accessibility requirements for web".into());
    prose(
        "On web, auto-dismissing snackbars are inaccessible for people with low vision or who require additional time to perceive information. This can be solved in 2 ways:",
    );
    gap(Dp(24.0));
    subheading("1. Add inline feedback".into()).level(4);
    prose(
        "Information in auto-dismissing snackbars must also be communicated using another accessible method inline or near the action that triggered the snackbar.",
    );
    gap(Dp(12.0));
    prose(
        "For example, update the label on a “Save” button to “Saved”, and trigger an auto-dismissing snackbar that communicates the same message.",
    );
    gap(Dp(24.0));
    subheading("2. Make the snackbar actionable".into()).level(4);
    prose("Alternatively, add actions to the snackbar so it doesn’t dismiss until acted on.");
    gap(Dp(24.0));
    web_feedback_figure();
    gap(Dp(16.0));
    caption("Also communicate snackbar information near the action that triggered the snackbar");
}

#[tessera]
fn web_feedback_figure() {
    stage(
        300.0,
        LAVENDER,
        "A button labelled Save changes to Saved after a moment. A snackbar confirms all changes are saved.",
    )
    .content(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                web_row(false);
                gap(Dp(16.0));
                web_row(true);
                gap(Dp(48.0));
                boxed().modifier(Modifier::new().size(Dp(180.0), Dp(40.0))).children(|| {
                    mock_bar(Bar {
                        width: 180.0,
                        message: "All changes saved",
                        ..Default::default()
                    });
                });
            });
    });
}

#[tessera]
fn web_row(saved: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .width(Dp(460.0))
                .height(Dp(48.0))
                .background(scheme.surface_container_lowest)
                .padding_symmetric(Dp(8.0), Dp::ZERO),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .width(Dp(220.0))
                        .height(Dp(36.0))
                        .background_with_shape(scheme.surface_container_high, Shape::CAPSULE)
                        .padding_symmetric(Dp(12.0), Dp::ZERO),
                )
                .children(move || {
                    text()
                        .content("Search")
                        .style(type_style(12.0, 16.0))
                        .color(scheme.on_surface_variant);
                    spacer().modifier(Modifier::new().weight(1.0));
                    icon().vector(filled::SEARCH_SVG).size(Dp(16.0));
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .height(Dp(32.0))
                        .border_with_shape(Dp(1.0), scheme.outline, Shape::CAPSULE)
                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                )
                .children(move || {
                    if saved {
                        icon()
                            .vector(filled::CHECK_SVG)
                            .size(Dp(14.0))
                            .tint(scheme.primary);
                        spacer().modifier(Modifier::new().width(Dp(6.0)));
                    }
                    text()
                        .content(if saved { "Saved" } else { "Save" })
                        .style(type_style(12.0, 16.0))
                        .color(scheme.primary);
                });
            spacer().modifier(Modifier::new().weight(1.0));
            for index in 0..3 {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().size(Dp(32.0), Dp(32.0)))
                    .children(move || match index {
                        0 => {
                            icon().vector(filled::HELP_OUTLINE_SVG).size(Dp(18.0));
                        }
                        1 => {
                            icon().vector(filled::SETTINGS_SVG).size(Dp(18.0));
                        }
                        _ => {
                            icon().vector(filled::ACCOUNT_CIRCLE_SVG).size(Dp(18.0));
                        }
                    });
            }
        });
}

#[tessera]
fn guideline_anatomy() {
    annotated(GUIDE, "4 elements of a snackbar.");
    gap(Dp(24.0));
    numbered(vec![
        "Container",
        "Supporting text",
        "Action (optional)",
        "Close button (optional)",
    ]);
    gap(Dp(40.0));
    subheading("Text label".into());
    prose(
        "Snackbars contain a text label that directly relates to the process being performed. In compact breakpoints, the text label can contain up to two lines of text.",
    );
    gap(Dp(24.0));
    phone_guide(
        TEXT_LABEL,
        NOTE,
        "Text labels are short, clear updates on processes that have been performed",
    );
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(
                TEXT_ONE,
                DO,
                "Keep the snackbar text label to one line long when possible",
            );
        })
        .second(|| {
            phone_guide(
                TEXT_TWO,
                DO,
                "On mobile, the text label can be up to two lines long",
            );
        });
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(
                ICON,
                CAUTION,
                "Avoid adding icons to snackbars. If your message needs an icon, consider using a different component such as a dialog.",
            );
        })
        .second(|| {
            phone_guide(
                STYLED,
                DONT,
                "Avoid using stylized text or inline links in snackbars; they can add unwanted complexity. If your message needs a link, add a button instead, or use a different component.",
            );
        });
    gap(Dp(40.0));
    subheading("Container".into());
    prose(
        "Snackbars are displayed in rectangular containers with a grey background. Containers should be completely opaque, so that text labels remain legible.",
    );
    gap(Dp(24.0));
    phone_guide(
        SOLID,
        NOTE,
        "Snackbar containers use a solid background color with a shadow to stand out against content",
    );
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(
                SAME_COLOR,
                DONT,
                "The text label shouldn’t share the same color as the text button",
            );
        })
        .second(|| {
            phone_guide(
                ELEVATED,
                DONT,
                "Don’t use a filled or elevated button in a snackbar, as it draws too much attention",
            );
        });
    gap(Dp(32.0));
    phone_guide(
        TABLET_EXTENDED,
        DO,
        "In wide layouts, extend the container width to accommodate longer text labels",
    );
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(
                TRANSLUCENT,
                CAUTION,
                "An app can apply slight transparency to the container background, as long as text remains clearly legible",
            );
        })
        .second(|| {
            phone_guide(
                CAPSULE,
                DONT,
                "Avoid significantly altering the shape of a snackbar container",
            );
        });
    gap(Dp(40.0));
    subheading("Action".into());
    prose(
        "Snackbars can display a single text button that lets users take action on a process performed by the app. Snackbars shouldn’t be the only way to access a core use case, to make an app usable.",
    );
    gap(Dp(24.0));
    phone_guide(
        ARCHIVED,
        NOTE,
        "To distinguish the action from the text label, text buttons should display colored text",
    );
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(
                LONG_ACTION,
                DO,
                "If an action is long, it can be displayed on a third line",
            );
        })
        .second(|| {
            phone_guide(
                UNDO,
                DO,
                "To allow users to amend choices, display an “Undo” action",
            );
        });
    gap(Dp(32.0));
    pair().first(|| {
        phone_guide(
            DISMISS,
            CAUTION,
            "A dismiss action is unnecessary, as snackbar disappears on their own by default",
        );
    });
}

#[tessera]
fn placement() {
    subheading("At the bottom of a UI".into());
    prose(
        "Snackbars should be placed at the bottom of a UI, in front of the main content. In some cases, snackbars can be nudged upwards to avoid overlapping with other UI elements near the bottom, such as FABs or docked toolbars.",
    );
    gap(Dp(12.0));
    prose("Avoid placing a snackbar in front of frequently used touch targets or navigation.");
    gap(Dp(24.0));
    pair()
        .first(|| {
            phone_guide(
                CONTENT_DO,
                DO,
                "Place a snackbar in front of the main content",
            );
        })
        .second(|| {
            phone_guide(
                NAV_DONT,
                DONT,
                "Avoid placing snackbars in front of navigation components",
            );
        });
    gap(Dp(32.0));
    prose(
        "To ensure accessibility for keyboard users on the web, avoid positioning the snackbar in a way that completely obscures actionable elements. Blocking elements makes it difficult to know what is being focused and selected.",
    );
    gap(Dp(24.0));
    pair()
        .first(|| {
            phone_guide(
                FOCUS_DO,
                DO,
                "Adjust the size of the snackbar to avoid blocking elements in focus",
            );
        })
        .second(|| {
            phone_guide(
                FOCUS_DONT,
                DONT,
                "Don’t let the snackbar fully cover elements in focus",
            );
        });
    gap(Dp(32.0));
    prose(
        "Snackbars can span the entire width of the screen only when a UI does not use persistent navigation components like app bars or navigation bars. Snackbars that span the entire width of a UI can push up FABs when they appear.",
    );
    gap(Dp(24.0));
    phone_guide(
        FULL,
        CAUTION,
        "Snackbars can span the entire width of a UI. However, they should not appear in front of navigation or other important UI elements like floating action buttons.",
    );
    gap(Dp(40.0));
    subheading("Snackbars and floating action buttons (FABs)".into());
    prose("Snackbars should appear above FABs.");
    gap(Dp(24.0));
    phone_guide(FAB_DO, NOTE, "Snackbar above a FAB");
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(FAB_FRONT, DONT, "Don’t place a snackbar in front of a FAB");
        })
        .second(|| {
            phone_guide(FAB_BEHIND, DONT, "Don’t place a snackbar behind a FAB");
        });
}

#[tessera]
fn responsive() {
    subheading("Compact breakpoint".into());
    prose(
        "In compact breakpoints, snackbars should expand vertically from 48dp to 64dp to accommodate one or two lines of text, while maintaining a fixed distance from the leading, trailing, and bottom edges of the screen.",
    );
    gap(Dp(24.0));
    phone_guide(
        COMPACT,
        NOTE,
        "Snackbar with its label text extending to the second line and maintaining fixed distance from the edges of a mobile device.",
    );
    gap(Dp(40.0));
    subheading("Medium & expanded breakpoints".into());
    prose(
        "On medium and expanded breakpoints, like tablet and desktop, snackbars should scale horizontally to accommodate longer text strings, keeping in mind that the ideal line length for text is typically between 40-60 characters.",
    );
    gap(Dp(12.0));
    prose(
        "Snackbars use a flexible distance from the trailing edge of the screen. Whenever possible, snackbars on medium and large displays should aim for a single line of text with an optional button.",
    );
    gap(Dp(24.0));
    phone_guide(
        TABLET_MEDIUM,
        NOTE,
        "A horizontally expanded snackbar placed at the bottom of screen on a medium-size device.",
    );
    gap(Dp(32.0));
    prose(
        "In wider layouts, snackbars can be left-aligned or center-aligned if they are consistently placed on the same spot at the bottom of the screen.",
    );
    gap(Dp(24.0));
    phone_guide(TABLET_LEFT, NOTE, "Left-aligned snackbar");
    gap(Dp(24.0));
    phone_guide(TABLET_CENTER, NOTE, "Center-aligned snackbar");
    gap(Dp(32.0));
    phone_guide(
        TABLET_FLUSH,
        DONT,
        "Don’t place snackbars flush to one edge of the layout",
    );
    gap(Dp(32.0));
    phone_guide(
        TABLET_SIDE,
        DONT,
        "Don’t place consecutive snackbars side by side or next to one another",
    );
}

#[tessera]
fn behavior() {
    subheading("Appearing and disappearing".into());
    prose(
        "Snackbars appear without warning, but they don’t block users from interacting with page content.",
    );
    gap(Dp(12.0));
    prose(
        "Snackbars without actions can auto-dismiss after 4–10 seconds, depending on platform. Avoid using auto-dismissing snackbars on web unless there’s also inline feedback.",
    );
    gap(Dp(12.0));
    prose(
        "Snackbars with actions should remain on the screen until the user takes an action on the snackbar, or dismisses it.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(timing_demo));
    gap(Dp(12.0));
    caption(
        "Live Tessera example: the message-only snackbar closes after 4 seconds; the snackbar with an action stays until Undo or the close button is used.",
    );
    gap(Dp(40.0));
    subheading("Consecutive snackbars".into());
    prose("Consecutive snackbars must appear one at a time.");
    gap(Dp(12.0));
    prose(
        "Snackbars without actions appear and disappear automatically, while those with actions remain on screen until dismissed. However, a snackbar with updated information can immediately replace an outdated snackbar.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(queued));
    gap(Dp(32.0));
    pair()
        .first(|| {
            phone_guide(STACK, DONT, "Don’t stack snackbars on top of one another");
        })
        .second(|| {
            phone_guide(
                FAB_MOVE,
                DONT,
                "Don’t animate other components along with snackbar animations, such as the floating action button",
            );
        });
}

#[tessera]
fn timing_demo() {
    let host = remember(SnackbarHostState::new);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    button()
                        .tonal()
                        .accessibility_label("Show message-only snackbar")
                        .on_click(move || {
                            host.with_mut(|state| {
                                state.show_snackbar(SnackbarRequest {
                                    message: "Message archived".into(),
                                    action_label: None,
                                    with_dismiss_action: false,
                                    duration: Some(SnackbarDuration::Short),
                                });
                            });
                        })
                        .child(|| {
                            text().content("Message only");
                        });
                    button()
                        .tonal()
                        .accessibility_label("Show snackbar with action")
                        .on_click(move || {
                            host.with_mut(|state| {
                                state.show_snackbar(SnackbarRequest {
                                    message: "Message archived".into(),
                                    action_label: Some("Undo".into()),
                                    with_dismiss_action: true,
                                    duration: None,
                                });
                            });
                        })
                        .child(|| {
                            text().content("With action");
                        });
                });
            gap(Dp(16.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().size(Dp(380.0), Dp(80.0)))
                .children(move || {
                    snackbar_host()
                        .state(host)
                        .modifier(Modifier::new().width(Dp(360.0)));
                });
        });
}

#[tessera]
fn use_cases() {
    prose("Users should be able to:");
    gap(Dp(12.0));
    bullets(vec![
        "Be alerted, but not disrupted, when a snackbar appears",
        "Move focus to an actionable snackbar",
        "Take action on a snackbar using assistive technology",
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "Snackbars with actions shouldn’t auto-dismiss. This way, users can read and interact with it at their own pace.",
    );
    gap(Dp(12.0));
    prose(
        "Snackbars without actions can auto-dismiss after a sufficient amount of time, however this can still present difficulties on web without additional feedback.",
    );
    gap(Dp(12.0));
    prose(
        "Each platform has its own requirements for auto-dismiss durations, however common acceptable durations are 4–10 seconds.",
    );
    gap(Dp(24.0));
    phone_guide(
        A11Y_DARK,
        NOTE,
        "Auto-dismissing snackbars should remain on screen long enough to read the information",
    );
    gap(Dp(32.0));
    prose(
        "Snackbars use a color intended to stand out against UI elements. Use the default color mapping to avoid color conflict issues.",
    );
    gap(Dp(24.0));
    phone_guide(ARCHIVED, NOTE, "Snackbar should visually stand out");
}

#[tessera]
fn web_requirements() {
    prose(
        "On web, auto-dismissing snackbars can be difficult to navigate for people with low vision or who require additional time to perceive information. This information can be made clearer for all users in two ways:",
    );
    gap(Dp(24.0));
    subheading("1. Add inline feedback".into()).level(4);
    prose(
        "Information in auto-dismissing snackbars must also be communicated inline or near the action that triggered the snackbar.",
    );
    gap(Dp(12.0));
    prose(
        "For example, update the label on a “Save” button to “Saved”, and trigger an auto-dismissing snackbar that communicates the same message.",
    );
    gap(Dp(24.0));
    web_feedback_figure();
    gap(Dp(16.0));
    caption("Communicate snackbar information near the action that triggered the snackbar");
    gap(Dp(24.0));
    panel(RenderSlot::new(inline_feedback));
    gap(Dp(12.0));
    caption(
        "Live Tessera example: Save changes its own label to Saved before the 4 second snackbar closes.",
    );
    gap(Dp(32.0));
    subheading("2. Make the snackbar actionable".into()).level(4);
    prose(
        "Alternatively, add actions to the snackbar so it doesn’t dismiss until acted on. Actionable snackbars shouldn’t auto-dismiss.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(actionable_demo));
    gap(Dp(24.0));
    text()
        .content(
            "Note: Material Web doesn’t yet include the snackbar component. This guidance still applies to custom-made snackbars.",
        )
        .style(TextStyle {
            font_weight: 700,
            ..type_style(16.0, 24.0)
        });
}

#[tessera]
fn inline_feedback() {
    let saved = remember(|| false);
    let host = remember(SnackbarHostState::new);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            button()
                .outlined()
                .accessibility_label(if saved.get() { "Saved" } else { "Save" })
                .on_click(move || {
                    if saved.get() {
                        saved.set(false);
                    } else {
                        saved.set(true);
                        host.with_mut(|state| {
                            state.show_snackbar(SnackbarRequest {
                                message: "All changes saved".into(),
                                action_label: None,
                                with_dismiss_action: false,
                                duration: Some(SnackbarDuration::Short),
                            });
                        });
                    }
                })
                .child(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            if saved.get() {
                                icon().vector(filled::CHECK_SVG).size(Dp(18.0));
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                            }
                            text().content(if saved.get() { "Saved" } else { "Save" });
                        });
                });
            gap(Dp(24.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().size(Dp(380.0), Dp(80.0)))
                .children(move || {
                    snackbar_host()
                        .state(host)
                        .modifier(Modifier::new().width(Dp(360.0)));
                });
        });
}

#[tessera]
fn actionable_demo() {
    let kept = remember(|| true);
    let host = remember(SnackbarHostState::new);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            button()
                .outlined()
                .accessibility_label("Delete photo")
                .on_click(move || {
                    if kept.get() {
                        kept.set(false);
                        host.with_mut(|state| {
                            state.show_snackbar(SnackbarRequest {
                                message: "Photo deleted".into(),
                                action_label: Some("Undo".into()),
                                with_dismiss_action: true,
                                duration: None,
                            });
                        });
                    }
                })
                .child(|| {
                    text().content("Delete photo");
                });
            gap(Dp(12.0));
            text()
                .content(if kept.get() {
                    "Photo in album: yes"
                } else {
                    "Photo in album: no"
                })
                .style(type_style(14.0, 20.0));
            gap(Dp(16.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().size(Dp(380.0), Dp(80.0)))
                .children(move || {
                    snackbar_host()
                        .state(host)
                        .modifier(Modifier::new().width(Dp(360.0)))
                        .snackbar(move |data: SnackbarData| {
                            let dismiss = data.clone();
                            snackbar()
                                .message(data.message().to_owned())
                                .action_label("Undo")
                                .with_dismiss_action(true)
                                .on_dismiss(move || dismiss.dismiss())
                                .on_action(move || {
                                    data.perform_action();
                                    kept.set(true);
                                });
                        });
                });
        });
}

#[tessera]
fn focus() {
    prose("Snackbars have the following focus requirements:");
    gap(Dp(12.0));
    bullets(vec![
        "When a snackbar appears, announce the message but don’t move focus.",
        "Don’t automatically move focus.",
        "Don’t trap focus in the snackbar. Users should be able to freely navigate in and out.",
        "On web, a shortcut should exist for users to move focus to snackbars with actions (like Alt+G). Ensure that this shortcut is clearly documented, like in a help article.",
    ]);
    gap(Dp(24.0));
    phone_guide(
        FOCUS_RETURN,
        NOTE,
        "Focus returns from the snackbar (1) to the previously focused element (2)",
    );
    gap(Dp(32.0));
    prose("Focus exits the snackbar differently per platform:");
    gap(Dp(12.0));
    bullets(vec![
        "Ideally, focus should either return to the element that triggered the snackbar, or go to the next most logical element on the page.",
        "On Android Compose, focus may move to the nearest visible element, or to the first actionable item on the page.",
    ]);
    gap(Dp(24.0));
    phone_guide(
        FOCUS_NEXT,
        NOTE,
        "If the previously focused element is no longer on the page, focus should move from the snackbar (1) to the next most logical element (2)",
    );
    gap(Dp(24.0));
    prose(
        "In Tessera, snackbar_host keeps the current focus when a notification appears and restores the previous target when the last notification closes while its controls own focus. A shortcut that moves focus to an actionable snackbar is app-defined; this example does not register one.",
    );
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Tab".into(),
                "Moves focus between interactive elements".into(),
            ],
            vec!["Esc".into(), "Dismisses the snackbar when in focus".into()],
        ],
    );
}

#[tessera]
fn labeling() {
    prose(
        "Snackbars should be announced once they appear on the screen, but shouldn’t grab focus or prevent people from completing their current task.",
    );
    gap(Dp(12.0));
    bullets(vec![
        "On Android and web, use a live region with a polite (queued) announcement instead of an assertive announcement.",
        "On iOS 17+, snackbars use polite announcements by default.",
    ]);
    prose(
        "If a snackbar appears when the app is launched, it should be announced after the page’s title, but not receive focus.",
    );
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            stage(
                220.0,
                SLATE,
                "Snackbar labeled Photo deleted with role Status, and an Undo action with role Button",
            )
            .content(|| {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(|| {
                        callout("Photo deleted", "Status");
                        connector();
                        boxed()
                            .modifier(Modifier::new().size(Dp(240.0), Dp(48.0)))
                            .children(|| {
                                mock_bar(Bar {
                                    width: 240.0,
                                    message: "Photo deleted",
                                    action: Some("Undo"),
                                    ..Default::default()
                                });
                            });
                        connector();
                        callout("Undo", "Button");
                    });
            });
        });
    gap(Dp(16.0));
    caption("Snackbars are announced when they appear, but don’t trap focus");
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Accessible information in Tessera".into()],
        vec![
            vec![
                "Supporting text".into(),
                "Status role with a polite live region; the message is the label".into(),
            ],
            vec![
                "Action".into(),
                "Button named with its visible text, such as Undo".into(),
            ],
            vec![
                "Close button".into(),
                "Button named Dismiss unless the app provides a more specific name".into(),
            ],
        ],
    );
}

#[tessera]
fn callout(#[prop(into)] name: String, #[prop(into)] role: String) {
    let accent = srgb(232, 140, 230);
    column()
        .modifier(
            Modifier::new()
                .width(Dp(110.0))
                .background_with_shape(srgb(29, 27, 32), Shape::rounded_rectangle(Dp(8.0)))
                .border_with_shape(Dp(2.0), accent, Shape::rounded_rectangle(Dp(8.0)))
                .padding_all(Dp(12.0)),
        )
        .children(move || {
            text()
                .content("Label")
                .style(type_style(11.0, 16.0))
                .color(srgb(230, 224, 233));
            text()
                .content(name.clone())
                .style(TextStyle {
                    font_weight: 700,
                    ..type_style(12.0, 16.0)
                })
                .color(accent);
            gap(Dp(8.0));
            text()
                .content("Role")
                .style(type_style(11.0, 16.0))
                .color(srgb(230, 224, 233));
            text()
                .content(role.clone())
                .style(TextStyle {
                    font_weight: 700,
                    ..type_style(12.0, 16.0)
                })
                .color(accent);
        });
}

#[tessera]
fn connector() {
    boxed().modifier(
        Modifier::new()
            .size(Dp(16.0), Dp(2.0))
            .background(srgb(232, 140, 230)),
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod anatomy_tests {
    use super::*;
    use std::time::Duration;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        accesskit::{Node, NodeId},
    };
    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            column().children(|| {
                annotated(ANATOMY, "Snackbar anatomy");
            });
        });
    }
    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
        let snapshot = renderer
            .inspect_accessibility()
            .expect("accessibility tree");
        let (id, node) = snapshot
            .find(label, role)
            .unwrap_or_else(|| panic!("missing {label} {role:?}"));
        (id, node.clone())
    }
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn snackbar_spec_anatomy_numbers_container_close_action_and_text_like_material() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
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
        let message = node(&renderer, "Single-line snackbar with action", Role::Status)
            .1
            .bounds()
            .unwrap();
        let action = node(&renderer, "Action", Role::Button).1.bounds().unwrap();
        let close = node(&renderer, "Dismiss Snackbar anatomy", Role::Button)
            .1
            .bounds()
            .unwrap();
        let marker = |number: usize| {
            node(&renderer, &format!("Anatomy marker {number}"), Role::Note)
                .1
                .bounds()
                .unwrap()
        };
        let container = marker(1);
        let icon = marker(2);
        let action_marker = marker(3);
        let text_marker = marker(4);
        assert!(container.y1 < message.y0);
        assert!(icon.x0 > close.x1);
        let icon_center = (icon.y0 + icon.y1) / 2.0;
        assert!(icon_center >= close.y0 && icon_center <= close.y1);
        assert!(action_marker.y0 > action.y1);
        let action_center = (action_marker.x0 + action_marker.x1) / 2.0;
        assert!(action_center >= action.x0 && action_center <= action.x1);
        assert!(text_marker.y0 > message.y1);
        let text_center = (text_marker.x0 + text_marker.x1) / 2.0;
        assert!(text_center >= message.x0 && text_center <= message.x1);
    }
}
