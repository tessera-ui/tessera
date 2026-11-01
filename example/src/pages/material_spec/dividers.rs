//! Divider specification article with full-width, inset, and vertical figures.
//!
//! ## Usage
//!
//! Compare full-width, inset, and vertical dividers in grouped content.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    divider::{horizontal_divider, vertical_divider},
    icon::icon,
    material_icons::filled,
    modifier::{ClickableArgs, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, TextStyle, material_theme},
};
use tessera_ui::{
    Callback, Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    annotations::{self, SpaceGeometry, space_geometry, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, srgb, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "dividers_tests.rs"]
mod figures_tests;

const INBOX: u8 = 0;
const MAIL: u8 = 1;
const PLAYLIST: u8 = 2;
const PODCAST: u8 = 3;
const CONTACTS: u8 = 4;

const FULL: u8 = 0;
const INSET: u8 = 1;
const OPEN: u8 = 2;

const NONE: u8 = 0;
const DO: u8 = 1;
const CAUTION: u8 = 2;

const OUTLINED: u8 = 0;
const DARK: u8 = 1;
const LIGHT: u8 = 2;

/// Inset applied to both ends of an inset divider.
const INSET_MARGIN: f64 = 16.0;

const DESIGN_TOKENS_URL: &str =
    "https://m3.material.io/m3/pages/design-tokens/overview/825906c9-6eed-47d1-8812-450910c1356e";

// Article links leave the example app, so they are handed to the platform
// browser instead of the in-app router.
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function divider_open_external(url) { window.open(url, '_blank', 'noopener'); }"
)]
extern "C" {
    fn divider_open_external(url: &str);
}

#[cfg(target_family = "wasm")]
fn open_external(url: &str) {
    divider_open_external(url);
}

#[cfg(all(not(target_family = "wasm"), target_os = "windows"))]
fn open_external(url: &str) {
    let _ = std::process::Command::new("explorer").arg(url).spawn();
}

#[cfg(all(not(target_family = "wasm"), target_os = "macos"))]
fn open_external(url: &str) {
    let _ = std::process::Command::new("open").arg(url).spawn();
}

#[cfg(all(
    not(target_family = "wasm"),
    not(target_os = "windows"),
    not(target_os = "macos"),
    not(target_os = "android")
))]
fn open_external(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

#[cfg(target_os = "android")]
fn open_external(_url: &str) {}

/// Shows link text that opens an external Material Design page.
#[tessera]
fn external_link(#[prop(into)] label: String, #[prop(into)] url: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(label.clone())
        .style(type_style(16.0, 24.0))
        .color(scheme.primary)
        .modifier(Modifier::new().clickable_with(ClickableArgs {
            role: Some(Role::Link),
            label: Some(label.clone()),
            on_click: Callback::new(move || open_external(&url)),
            ..Default::default()
        }));
}

/// Paints the dotted canvas used behind some official figures.
#[tessera]
fn dot_grid(#[prop(into)] color: String) {
    let size = remember(|| (0.0_f64, 0.0_f64));
    let (width, height) = size.get();
    boxed()
        .modifier(Modifier::new().fill_max_size().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = (
                    Dp::from_pixels_f32(coordinates.local_size.width.to_f32()).0,
                    Dp::from_pixels_f32(coordinates.local_size.height.to_f32()).0,
                );
                if size.get() != next {
                    size.set(next);
                }
            },
        ))
        .children(move || {
            if width < 1.0 || height < 1.0 {
                return;
            }
            let mut svg = String::new();
            let mut y = 4.0;
            while y < height {
                let mut x = 4.0;
                while x < width {
                    svg.push_str(&format!(
                        "<rect x='{x}' y='{y}' width='1' height='1' fill='{color}'/>"
                    ));
                    x += 8.0;
                }
                y += 8.0;
            }
            annotations::lines(Dp(width), Dp(height), svg);
        });
}

fn rose() -> Color {
    srgb(247, 242, 247)
}

fn lavender() -> Color {
    srgb(226, 224, 249)
}

fn blush() -> Color {
    srgb(252, 241, 250)
}

fn frame() -> Color {
    srgb(176, 172, 186)
}

#[tessera]
pub(super) fn divider_spec() {
    article(
        "Divider",
        "Dividers are thin lines that group content in lists or other containers",
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
        ],
    )
    .intro(divider_anatomy)
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
            intro: Some(RenderSlot::new(guidelines_intro)),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(|| {
                        prose("A divider is a simple line.");
                        gap(Dp(48.0));
                        divider_anatomy();
                    }),
                },
                SpecSection {
                    title: "Full-width dividers",
                    content: RenderSlot::new(full_width_guidelines),
                },
                SpecSection {
                    title: "Inset dividers",
                    content: RenderSlot::new(inset_guidelines),
                },
                SpecSection {
                    title: "Vertical divider",
                    content: RenderSlot::new(vertical_guidelines),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: Some(RenderSlot::new(accessibility)),
            sections: vec![],
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
        .color(scheme.on_surface_variant)
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    for item in items {
        row().children(move || {
            text()
                .content("•")
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().width(Dp(20.0)));
            text()
                .content(item.clone())
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().weight(1.0));
        });
        gap(Dp(14.0));
    }
}

#[tessera]
fn badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(srgb(43, 41, 48), Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(12.0, 16.0))
                .color(srgb(255, 255, 255));
        });
}

#[tessera]
fn numbered(items: Vec<String>) {
    for (index, item) in items.into_iter().enumerate() {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(move || {
                badge(index + 1);
                spacer().modifier(Modifier::new().width(Dp(12.0)));
                text()
                    .content(item.clone())
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().weight(1.0));
            });
        gap(Dp(16.0));
    }
}

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO))
        .children(move || {
            badge(number);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(label.clone())
                .style(type_style(14.0, 20.0))
                .color(scheme.on_surface_variant);
        });
}

/// Places explanatory text beside its figure, stacking them in narrow layouts.
#[tessera]
fn split(lead: Option<RenderSlot>, figure: Option<RenderSlot>) {
    let width = remember(|| Dp(750.0));
    let wide = width.get().0 >= 640.0;
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
            if wide {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                if let Some(lead) = lead {
                                    lead.render();
                                }
                            });
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        column()
                            .modifier(Modifier::new().weight(2.0))
                            .children(move || {
                                if let Some(figure) = figure {
                                    figure.render();
                                }
                            });
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        if let Some(lead) = lead {
                            lead.render();
                        }
                        gap(Dp(32.0));
                        if let Some(figure) = figure {
                            figure.render();
                        }
                    });
            }
        });
}

/// Places two figures in equal columns, stacking them below `narrow` width.
#[tessera]
fn pair(narrow: Option<f64>, first: Option<RenderSlot>, second: Option<RenderSlot>) {
    let width = remember(|| Dp(750.0));
    let wide = width.get().0 >= narrow.unwrap_or(600.0);
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
            if wide {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                if let Some(first) = first {
                                    first.render();
                                }
                            });
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                if let Some(second) = second {
                                    second.render();
                                }
                            });
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        if let Some(first) = first {
                            first.render();
                        }
                        gap(Dp(32.0));
                        if let Some(second) = second {
                            second.render();
                        }
                    });
            }
        });
}

fn top_rounded() -> Shape {
    let round = RoundedCorner::manual(Dp(16.0), 3.0);
    let square = RoundedCorner::manual(Dp::ZERO, 3.0);
    Shape::RoundedRectangle {
        top_left: round,
        top_right: round,
        bottom_right: square,
        bottom_left: square,
    }
}

fn bottom_rounded() -> Shape {
    let round = RoundedCorner::manual(Dp(16.0), 3.0);
    let square = RoundedCorner::manual(Dp::ZERO, 3.0);
    Shape::RoundedRectangle {
        top_left: square,
        top_right: square,
        bottom_right: round,
        bottom_left: round,
    }
}

/// Draws a clipped, tinted figure area.
///
/// A non-empty `label` publishes the whole figure as one described image.
#[tessera]
fn stage(
    height: f64,
    tint: Color,
    #[prop(into)] label: String,
    square_bottom: Option<bool>,
    modifier: Option<Modifier>,
    content: Option<RenderSlot>,
) {
    let shape = if square_bottom.unwrap_or(false) {
        top_rounded()
    } else {
        Shape::rounded_rectangle(Dp(16.0))
    };
    let modifier = modifier.unwrap_or_default();
    let modifier = if label.is_empty() {
        modifier
    } else {
        modifier.semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some(label),
            merge_descendants: true,
            ..Default::default()
        })
    };
    boxed()
        .alignment(Alignment::TopCenter)
        .modifier(
            modifier
                .fill_max_width()
                .height(Dp(height))
                .clip_shape(shape)
                .background_with_shape(tint, shape),
        )
        .children(move || {
            if let Some(content) = content {
                content.render();
            }
        });
}

/// Draws a device frame whose top `crop` dp lie above the visible figure.
#[tessera]
fn phone(width: f64, crop: f64, height: f64, content: Option<RenderSlot>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp::ZERO, Dp(-crop))
                .required_size(Dp(width), Dp(height))
                .background_with_shape(frame(), Shape::rounded_rectangle(Dp(28.0)))
                .padding_all(Dp(6.0)),
        )
        .children(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .clip_shape(Shape::rounded_rectangle(Dp(22.0)))
                        .background(scheme.surface),
                )
                .children(move || {
                    if let Some(content) = content {
                        content.render();
                    }
                });
        });
}

#[tessera]
fn header(#[prop(into)] title: String, search: Option<bool>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background(scheme.surface_container),
        )
        .children(move || {
            text()
                .content("9:30")
                .style(type_style(11.0, 16.0))
                .modifier(Modifier::new().padding(Padding::new(
                    Dp(20.0),
                    Dp(10.0),
                    Dp(20.0),
                    Dp(6.0),
                )));
            let title = title.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(44.0))
                        .padding_symmetric(Dp(14.0), Dp::ZERO),
                )
                .children(move || {
                    icon().vector(filled::MENU_SVG).size(Dp(18.0));
                    spacer().modifier(Modifier::new().width(Dp(14.0)));
                    text()
                        .content(title.clone())
                        .style(type_style(16.0, 24.0))
                        .modifier(Modifier::new().weight(1.0));
                    if search.unwrap_or(true) {
                        icon().vector(filled::SEARCH_SVG).size(Dp(18.0));
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                    }
                    icon().vector(filled::MORE_VERT_SVG).size(Dp(18.0));
                });
        });
}

fn entry(kind: u8, index: usize) -> (&'static str, &'static str, &'static str) {
    const INBOX_ROWS: [(&str, &str, &str); 4] = [
        ("Cumpleaños de mamá", "Hola, en mi casa if you…", ""),
        ("Graduación de Inés", "Aquí tienes unas fotos…", ""),
        ("Pre-sale concert tickets", "I just saw there are a…", ""),
        ("Console ideas", "Do you remember that cafe…", ""),
    ];
    const MAIL_ROWS: [(&str, &str, &str); 4] = [
        ("Brunch this weekend?", "I’ll be in your area…", ""),
        ("Oui Oui", "Do you have Paris recs?", ""),
        ("Graduación de Inés", "Aquí tienes unas fotos…", ""),
        ("Recipe to try", "We should eat this…", ""),
    ];
    const PLAYLIST_ROWS: [(&str, &str, &str); 4] = [
        ("Glass & Grace", "Sound Check", ""),
        ("The Fortnightly", "Live sets", ""),
        ("The Breakdown", "Approaches", ""),
        ("Song Histories", "38 episodes", ""),
    ];
    const PODCAST_ROWS: [(&str, &str, &str); 6] = [
        ("A Year in Ecuador", "10 episodes", "68 min"),
        ("Wizards & Winona", "56 episodes", "109 min"),
        ("Gamer Course", "79 episodes", "57 min"),
        ("Song Histories", "38 episodes", "46 min"),
        ("The Breakdown", "58 episodes", "73 min"),
        ("Approaches", "10 episodes", "68 min"),
    ];
    const CONTACT_ROWS: [(&str, &str, &str); 4] = [
        ("Sofia Saachi", "she/her, coworker", ""),
        ("Alejandro Ortega", "he/him, partner", ""),
        ("Ines Villaneuva", "she/her, child", ""),
        ("Oli Ortega", "he/him, child", ""),
    ];
    match kind {
        INBOX => INBOX_ROWS[index % 4],
        MAIL => MAIL_ROWS[index % 4],
        PLAYLIST => PLAYLIST_ROWS[index % 4],
        PODCAST => PODCAST_ROWS[index % 6],
        _ => CONTACT_ROWS[index % 4],
    }
}

fn swatch(index: usize) -> Color {
    [
        srgb(128, 128, 128),
        srgb(226, 206, 172),
        srgb(140, 152, 132),
        srgb(24, 122, 142),
        srgb(240, 186, 222),
        srgb(96, 44, 86),
    ][index % 6]
}

#[tessera]
fn media(width: Option<f64>, height: f64, tint: Color) {
    let modifier = Modifier::new().height(Dp(height));
    let modifier = match width {
        Some(width) => modifier.width(Dp(width)),
        None => modifier.fill_max_width(),
    };
    boxed().modifier(modifier.background_with_shape(tint, Shape::rounded_rectangle(Dp(8.0))));
}

#[tessera]
fn row_item(kind: u8, index: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (title, body, trailing) = entry(kind, index);
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(16.0), Dp(8.0)),
        )
        .children(move || {
            match kind {
                MAIL | PLAYLIST | CONTACTS => {
                    boxed().modifier(
                        Modifier::new()
                            .size(Dp(36.0), Dp(36.0))
                            .background_with_shape(swatch(index + 1), Shape::Ellipse),
                    );
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                }
                PODCAST => {
                    media(36.0, swatch(index)).width(62.0);
                    spacer().modifier(Modifier::new().width(Dp(18.0)));
                }
                _ => {}
            }
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    text().content(title).style(type_style(14.0, 20.0));
                    text()
                        .content(body)
                        .style(type_style(12.0, 16.0))
                        .color(scheme.on_surface_variant);
                });
            if !trailing.is_empty() {
                text()
                    .content(trailing)
                    .style(type_style(11.0, 16.0))
                    .color(scheme.on_surface_variant);
            }
            if kind == CONTACTS {
                icon()
                    .vector(filled::INFO_SVG)
                    .size(Dp(18.0))
                    .tint(scheme.on_surface_variant);
            }
        });
}

/// Draws a divider whose start and end are inset by its surrounding layout.
#[tessera]
fn rule(left: f64, right: f64, modifier: Option<Modifier>) {
    boxed()
        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
            Dp(left),
            Dp::ZERO,
            Dp(right),
            Dp::ZERO,
        )))
        .children(move || {
            horizontal_divider().modifier(modifier.clone().unwrap_or_default());
        });
}

/// Lists rows separated by full-width dividers, inset dividers, or open space.
#[tessera]
fn list_screen(kind: u8, rules: u8, count: Option<usize>) {
    let count = count.unwrap_or(4);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for index in 0..count {
                row_item(kind, index);
                if index + 1 < count {
                    match rules {
                        FULL => {
                            rule(0.0, 0.0);
                        }
                        INSET => {
                            rule(INSET_MARGIN, INSET_MARGIN);
                        }
                        _ => {
                            gap(Dp(4.0));
                        }
                    }
                }
            }
        });
}

#[tessera]
fn section_title(#[prop(into)] label: String) {
    text()
        .content(label)
        .style(type_style(18.0, 24.0))
        .modifier(Modifier::new().padding(Padding::new(Dp(16.0), Dp(14.0), Dp(16.0), Dp(6.0))));
}

#[tessera]
fn advice(tone: u8) {
    let (background, ink, asset, label) = if tone == DO {
        (srgb(52, 190, 82), srgb(0, 33, 10), filled::CHECK_SVG, "Do")
    } else {
        (
            srgb(255, 207, 36),
            srgb(36, 26, 0),
            filled::PRIORITY_HIGH_SVG,
            "Caution",
        )
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
            icon().vector(asset).size(Dp(18.0)).tint(ink);
            spacer().modifier(Modifier::new().width(Dp(8.0)));
            text()
                .content(label)
                .style(type_style(14.0, 20.0))
                .color(ink);
        });
}

/// Shows a figure with an optional Do or Caution strip and its note.
#[tessera]
fn guidance(
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
                .square_bottom(tone != NONE)
                .content_optional(figure);
            if tone != NONE {
                advice(tone);
            }
            gap(Dp(16.0));
            text()
                .content(note.clone())
                .style(type_style(16.0, 24.0))
                .color(
                    use_context::<MaterialTheme>()
                        .unwrap()
                        .get()
                        .color_scheme
                        .on_surface_variant,
                )
                .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
        });
}

/// Draws an annotation pin above the divider it identifies.
#[tessera]
fn pin(tone: u8, number: usize, #[prop(into)] label: String, background: Color) {
    let (fill, ink, line) = match tone {
        DARK => (srgb(43, 41, 48), srgb(255, 255, 255), srgb(73, 69, 79)),
        LIGHT => (srgb(230, 224, 233), srgb(29, 27, 32), srgb(202, 196, 208)),
        _ => (srgb(247, 242, 250), INK, srgb(73, 69, 79)),
    };
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            let pill = Modifier::new()
                .size(Dp(18.0), Dp(24.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(label.clone()),
                    merge_descendants: true,
                    ..Default::default()
                })
                .background_with_shape(fill, Shape::CAPSULE);
            let pill = if tone == OUTLINED {
                pill.border_with_shape(Dp(1.0), line, Shape::CAPSULE)
            } else {
                pill
            };
            boxed()
                .alignment(Alignment::Center)
                .modifier(pill)
                .children(move || {
                    text()
                        .content(number.to_string())
                        .style(type_style(11.0, 16.0))
                        .color(ink);
                });
            spacer().modifier(Modifier::new().size(Dp(1.0), Dp(20.0)).background(line));
            boxed().modifier(
                Modifier::new()
                    .size(Dp(7.0), Dp(7.0))
                    .background_with_shape(background, Shape::Ellipse)
                    .border_with_shape(Dp(1.0), line, Shape::Ellipse),
            );
        });
}

/// Centers a short divider below a pin, with the pin's dot on the line.
#[tessera]
fn pinned_line(tone: u8, number: usize, #[prop(into)] label: String, width: f64, surface: Color) {
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(24.0), Dp::ZERO),
        )
        .children(move || {
            pin(tone, number, label.clone(), surface);
            boxed()
                .modifier(Modifier::new().offset(Dp::ZERO, Dp(-4.0)).width(Dp(width)))
                .children(|| {
                    horizontal_divider();
                });
        });
}

#[tessera]
fn divider_anatomy() {
    stage(224.0, rose(), "").content(|| {
        column()
            .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                Dp::ZERO,
                Dp(66.0),
                Dp::ZERO,
                Dp::ZERO,
            )))
            .children(|| {
                pinned_line(OUTLINED, 1, "Anatomy marker 1", 224.0, rose());
            });
    });
    gap(Dp(12.0));
    legend(1, "Divider");
}

#[tessera]
fn tokens() {
    prose("Browse the component elements, attributes, tokens, and their values.");
    gap(Dp(48.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec!["Divider thickness".into(), "1dp".into()],
            vec!["Divider color".into(), "Outline variant".into()],
        ],
    );
}

#[tessera]
fn scheme_half(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(Modifier::new().fill_max_size().background(scheme.surface))
                .children(move || {
                    dot_grid(if dark { "#2e2b33" } else { "#ead7e8" });
                    column()
                        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                            Dp::ZERO,
                            Dp(50.0),
                            Dp::ZERO,
                            Dp::ZERO,
                        )))
                        .children(move || {
                            pinned_line(
                                if dark { LIGHT } else { DARK },
                                1,
                                if dark {
                                    "Dark theme outline variant"
                                } else {
                                    "Light theme outline variant"
                                },
                                276.0,
                                scheme.surface,
                            );
                        });
                });
        });
}

#[tessera]
fn color_figure() {
    let width = remember(|| Dp(750.0));
    let wide = width.get().0 >= 560.0;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .clip_shape(Shape::rounded_rectangle(Dp(16.0)))
                .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                    let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                    if width.get() != next {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            if wide {
                row()
                    .modifier(Modifier::new().fill_max_width().height(Dp(218.0)))
                    .children(|| {
                        boxed()
                            .modifier(Modifier::new().weight(1.0).fill_max_height())
                            .children(|| {
                                scheme_half(false);
                            });
                        boxed()
                            .modifier(Modifier::new().weight(1.0).fill_max_height())
                            .children(|| {
                                scheme_half(true);
                            });
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        for dark in [false, true] {
                            boxed()
                                .modifier(Modifier::new().fill_max_width().height(Dp(180.0)))
                                .children(move || {
                                    scheme_half(dark);
                                });
                        }
                    });
            }
        });
}

#[tessera]
fn colors() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.",
    );
    external_link("Learn more about design tokens", DESIGN_TOKENS_URL);
    gap(Dp(48.0));
    color_figure();
    gap(Dp(16.0));
    caption("Divider color roles used for light and dark schemes:");
    gap(Dp(12.0));
    legend(1, "Outline variant");
}

#[tessera]
fn measure_figure() {
    stage(218.0, rose(), "").content(|| {
        boxed()
            .alignment(Alignment::Center)
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                column().children(|| {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(|| {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .modifier(Modifier::new().width(Dp(24.0)).semantics(
                                    SemanticsArgs {
                                        role: Some(Role::Note),
                                        label: Some("Dimension 1dp thickness".into()),
                                        merge_descendants: true,
                                        ..Default::default()
                                    },
                                ))
                                .children(|| {
                                    text()
                                        .content("1")
                                        .style(type_style(12.0, 16.0))
                                        .color(INK);
                                    spacer().modifier(Modifier::new().width(Dp(5.0)));
                                    annotations::lines(
                                        Dp(6.0),
                                        Dp(4.0),
                                        "<path d='M0 0.5H6M0 3.5H6' stroke='#d8565a' stroke-width='1'/>"
                                            .into(),
                                    );
                                });
                            boxed()
                                .modifier(Modifier::new().width(Dp(236.0)))
                                .children(|| {
                                    horizontal_divider();
                                });
                        });
                    gap(Dp(12.0));
                    row().children(|| {
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        column()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .modifier(Modifier::new().width(Dp(236.0)).semantics(SemanticsArgs {
                                role: Some(Role::Note),
                                label: Some("Dimension available width".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }))
                            .children(|| {
                                annotations::lines(
                                    Dp(236.0),
                                    Dp(7.0),
                                    "<path d='M0.5 0V7M0.5 3.5H235.5M235.5 0V7' stroke='#d8565a' stroke-width='1' fill='none'/>"
                                        .into(),
                                );
                                gap(Dp(4.0));
                                annotations::lines(
                                    Dp(16.0),
                                    Dp(10.0),
                                    "<path d='M8 5C6 2 2 2 2 5s4 3 6 0 4-3 6 0-4 3-6 0' fill='none' stroke='#1d1b20' stroke-width='1.2'/>"
                                        .into(),
                                );
                            });
                    });
                });
            });
    });
}

#[tessera]
fn measurements() {
    measure_figure();
    gap(Dp(12.0));
    text()
        .content("Measurements")
        .style(type_style(14.0, 20.0))
        .color(
            use_context::<MaterialTheme>()
                .unwrap()
                .get()
                .color_scheme
                .on_surface_variant,
        )
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
    gap(Dp(48.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Divider full-width".into(), "100%".into()],
            vec!["Divider inset left margin".into(), "16dp".into()],
            vec!["Divider inset right margin".into(), "0dp".into()],
            vec!["Divider middle-inset left margin".into(), "16dp".into()],
            vec!["Divider middle-inset right margin".into(), "16dp".into()],
            vec![
                "Space between divider & supporting-text".into(),
                "4dp".into(),
            ],
            vec!["Divider right margin".into(), "8dp".into()],
            vec!["Divider bottom margin".into(), "8dp".into()],
        ],
    );
}

#[tessera]
fn overview() {
    bullets(vec![
        "Make dividers visible but not bold".into(),
        "Only use dividers if items can’t be grouped with open space".into(),
        "Use dividers to group things, not separate individual items".into(),
    ]);
    gap(Dp(48.0));
    stage(
        240.0,
        blush(),
        "Podcast list with inset dividers between items",
    )
    .content(|| {
        phone(312.0, 18.0, 360.0).content(|| {
            gap(Dp(24.0));
            list_screen(PODCAST, INSET).count(6);
        });
    });
    gap(Dp(16.0));
    caption("Dividers separating items in a list");
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
            vec!["".into(), "Jetpack Compose".into(), "Available".into()],
            vec![
                "".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["".into(), "Web".into(), "Available".into()],
        ],
    );
}

#[tessera]
fn differences() {
    split()
        .lead(|| {
            bullets(vec![
                "Color: New color mappings and compatibility with dynamic color".into(),
                "Configurations: Ability to have vertical dividers".into(),
            ]);
        })
        .figure(|| {
            stage(
                218.0,
                blush(),
                "Inbox using inset dividers with new color mappings",
            )
            .content(|| {
                phone(312.0, 58.0, 360.0).content(|| {
                    header("Inbox");
                    section_title("Today");
                    for index in 0..3 {
                        row_item(MAIL, index);
                        rule(64.0, 16.0);
                    }
                });
            });
            gap(Dp(16.0));
            caption("Dividers have new color mappings");
        });
}

#[tessera]
fn inbox_phone(width: f64, crop: f64, rules: u8) {
    phone(width, crop, 420.0).content(move || {
        header("Inbox");
        section_title("Today");
        list_screen(INBOX, rules);
    });
}

#[tessera]
fn guidelines_intro() {
    stage(
        292.0,
        lavender(),
        "Inbox app using full-width dividers to separate messages",
    )
    .content(|| {
        boxed()
            .modifier(Modifier::new().padding(Padding::new(Dp::ZERO, Dp(26.0), Dp::ZERO, Dp::ZERO)))
            .children(|| {
                inbox_phone(312.0, 0.0, FULL);
            });
    });
    gap(Dp(16.0));
    caption("Full-width dividers");
}

#[tessera]
fn usage() {
    split()
        .lead(|| {
            prose(
                "Dividers are one way to visually group components and create hierarchy. They can also be used to imply nested parent/child relationships.",
            );
            gap(Dp(20.0));
            prose("The divider can be used in two ways:");
            gap(Dp(16.0));
            numbered(vec!["Full width".into(), "Inset".into()]);
        })
        .figure(|| {
            pair()
                .narrow(420.0)
                .first(|| {
                    guidance(
                        NONE,
                        216.0,
                        lavender(),
                        "Inbox app using full-width dividers to separate messages",
                        "Full-width divider",
                    )
                    .figure(|| {
                        boxed()
                            .modifier(Modifier::new().padding(Padding::new(
                                Dp::ZERO,
                                Dp(20.0),
                                Dp::ZERO,
                                Dp::ZERO,
                            )))
                            .children(|| {
                                phone(212.0, 0.0, 360.0).content(|| {
                                    header("Inbox").search(false);
                                    list_screen(INBOX, FULL);
                                });
                            });
                    });
                })
                .second(|| {
                    guidance(
                        NONE,
                        216.0,
                        lavender(),
                        "Music app using inset dividers to separate songs in a playlist",
                        "Inset divider",
                    )
                    .figure(|| {
                        boxed()
                            .modifier(Modifier::new().padding(Padding::new(
                                Dp::ZERO,
                                Dp(20.0),
                                Dp::ZERO,
                                Dp::ZERO,
                            )))
                            .children(|| {
                                phone(212.0, 0.0, 360.0).content(|| {
                                    header("Playlist").search(false);
                                    list_screen(PLAYLIST, INSET);
                                });
                            });
                    });
                });
        });
}

#[tessera]
fn order_row(index: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (title, body) = [
        ("Succulents", "Nursery"),
        ("Watering can", "Medium, brass finish"),
        ("Subtotal", "incl tax"),
    ][index];
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(16.0), Dp(8.0)),
        )
        .children(move || {
            if index < 2 {
                media(36.0, swatch(index + 2)).width(40.0);
                spacer().modifier(Modifier::new().width(Dp(12.0)));
            }
            column().children(move || {
                text().content(title).style(type_style(13.0, 18.0));
                text()
                    .content(body)
                    .style(type_style(11.0, 16.0))
                    .color(scheme.on_surface_variant);
            });
        });
}

#[tessera]
fn article_card() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .width(Dp(250.0))
                .offset(Dp::ZERO, Dp(40.0))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(12.0)))
                .border_with_shape(Dp(1.0), scheme.outline, Shape::rounded_rectangle(Dp(12.0))),
        )
        .children(move || {
            row()
                .modifier(Modifier::new().padding_all(Dp(10.0)))
                .children(move || {
                    media(64.0, swatch(2)).width(64.0);
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    column()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            text()
                                .content("The fortnightly")
                                .style(type_style(18.0, 24.0));
                            gap(Dp(4.0));
                            text()
                                .content("How to farm with less water: lessons from…")
                                .style(type_style(11.0, 16.0))
                                .color(scheme.on_surface_variant);
                        });
                });
            horizontal_divider();
            text()
                .content("Read more")
                .style(type_style(11.0, 16.0))
                .color(scheme.primary)
                .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp(12.0)));
        });
}

#[tessera]
fn order_list() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            boxed().modifier(
                Modifier::new()
                    .width(Dp(60.0))
                    .fill_max_height()
                    .background(srgb(68, 69, 88)),
            );
            boxed().modifier(
                Modifier::new()
                    .width(Dp(5.0))
                    .fill_max_height()
                    .background(frame()),
            );
            column()
                .modifier(
                    Modifier::new()
                        .weight(1.0)
                        .fill_max_height()
                        .background(scheme.surface),
                )
                .children(|| {
                    text()
                        .content("Your order")
                        .style(type_style(18.0, 24.0))
                        .modifier(Modifier::new().padding(Padding::new(
                            Dp(16.0),
                            Dp(14.0),
                            Dp(16.0),
                            Dp(10.0),
                        )));
                    for index in 0..3 {
                        horizontal_divider();
                        order_row(index);
                    }
                    horizontal_divider();
                });
        });
}

#[tessera]
fn full_width_guidelines() {
    split()
        .lead(|| {
            prose(
                "Use full-width dividers to separate larger sections of unrelated content. They can be used directly on a surface or inside other components like cards or lists.",
            );
            gap(Dp(20.0));
            prose("Full-width dividers can also separate interactive areas from non-interactive areas.");
            gap(Dp(20.0));
            prose(
                "They are used to group visual elements together, and indicate when elements are related to each other from an interaction perspective.",
            );
        })
        .figure(|| {
            stage(
                232.0,
                lavender(),
                "Inbox messages visually separated by full-width dividers for improved readability",
            )
            .content(|| {
                inbox_phone(312.0, 92.0, FULL);
            });
            gap(Dp(16.0));
            caption("Full-width dividers to indicate separation of content");
        });
    gap(Dp(64.0));
    pair()
        .first(|| {
            guidance(
                DO,
                216.0,
                blush(),
                "Non-interactive information on a card and a clickable read more link separated by a full-width divider",
                "Use full-width divider lines to separate interactive and non-interactive areas of a container such as a card",
            )
            .figure(|| {
                dot_grid("#ead7e8");
                article_card();
            });
        })
        .second(|| {
            guidance(
                CAUTION,
                216.0,
                lavender(),
                "Full-width dividers used after every piece of content on a page",
                "Use full-width dividers sparingly. Too many divider lines will make an interface look cluttered.",
            )
            .figure(|| {
                order_list();
            });
        });
}

#[tessera]
fn chip(#[prop(into)] label: String, selected: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let modifier = Modifier::new().height(Dp(24.0));
    let modifier = if selected {
        modifier.background_with_shape(
            scheme.secondary_container,
            Shape::rounded_rectangle(Dp(6.0)),
        )
    } else {
        modifier.border_with_shape(Dp(1.0), scheme.outline, Shape::rounded_rectangle(Dp(6.0)))
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(modifier.padding_symmetric(Dp(10.0), Dp::ZERO))
        .children(move || {
            if selected {
                icon()
                    .vector(filled::CHECK_SVG)
                    .size(Dp(12.0))
                    .tint(scheme.on_secondary_container);
                spacer().modifier(Modifier::new().width(Dp(6.0)));
            }
            text()
                .content(label.clone())
                .style(type_style(11.0, 16.0))
                .color(if selected {
                    scheme.on_secondary_container
                } else {
                    scheme.on_surface_variant
                });
        });
}

#[tessera]
fn chips(labels: Vec<String>, selected: usize) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO))
        .children(move || {
            for (index, label) in labels.iter().enumerate() {
                if index > 0 {
                    spacer().modifier(Modifier::new().width(Dp(6.0)));
                }
                chip(label.clone(), index == selected);
            }
        });
}

#[tessera]
fn plant_copy() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(
            "Hand selected from a variety of rare and exotic desert plants presented in a locally crafted ceramics. Choose your preference between succulent or cacti only or a combination arrangement of both.",
        )
        .style(type_style(11.0, 17.0))
        .color(scheme.on_surface)
        .modifier(Modifier::new().padding(Padding::new(
            Dp(16.0),
            Dp(12.0),
            Dp(16.0),
            Dp(16.0),
        )));
}

#[tessera]
fn product_detail() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .weight(1.0)
                        .fill_max_height()
                        .background(srgb(204, 196, 182)),
                )
                .children(|| {});
            column()
                .modifier(
                    Modifier::new()
                        .weight(1.7)
                        .fill_max_height()
                        .background(scheme.surface),
                )
                .children(move || {
                    plant_copy();
                    rule(INSET_MARGIN, INSET_MARGIN);
                    text()
                        .content("Select Size")
                        .style(type_style(12.0, 16.0))
                        .modifier(Modifier::new().padding(Padding::new(
                            Dp(16.0),
                            Dp(14.0),
                            Dp(16.0),
                            Dp(10.0),
                        )));
                    chips(vec!["Small".into(), "Medium".into(), "Large".into()], 1);
                    gap(Dp(14.0));
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .padding_symmetric(Dp(16.0), Dp::ZERO)
                                .height(Dp(28.0))
                                .border_with_shape(Dp(1.0), scheme.outline, Shape::CAPSULE),
                        )
                        .children(move || {
                            text()
                                .content("Add to cart")
                                .style(type_style(11.0, 16.0))
                                .color(scheme.primary);
                        });
                });
            boxed().modifier(
                Modifier::new()
                    .width(Dp(6.0))
                    .fill_max_height()
                    .background(frame()),
            );
            spacer().modifier(Modifier::new().width(Dp(60.0)));
        });
}

#[tessera]
fn inset_guidelines() {
    split()
        .lead(|| {
            prose("Use inset dividers to separate related content within a section.");
            gap(Dp(20.0));
            prose("Inset dividers are equally indented from both sides of the screen by default.");
        })
        .figure(|| {
            stage(240.0, lavender(), "2 different content sections separated by an inset divider")
                .content(|| {
                    product_detail();
                });
            gap(Dp(16.0));
            caption(
                "Inset dividers separate related content, such as emails in a list. They should be used with anchoring elements such as icons or avatars, and align with the leading edge of the screen.",
            );
        });
    gap(Dp(64.0));
    pair()
        .first(|| {
            guidance(
                NONE,
                218.0,
                lavender(),
                "3 types of related content separated by inset dividers on a mobile screen",
                "Inset dividers in a list of related items",
            )
            .figure(|| {
                phone(312.0, 150.0, 368.0).content(|| {
                    gap(Dp(16.0));
                    list_screen(PODCAST, INSET).count(6);
                });
            });
        })
        .second(|| {
            guidance(
                NONE,
                218.0,
                lavender(),
                "Inset divider separating body text and selection chips",
                "Inset dividers can be placed in the middle of a layout to separate elements such as body text from selection chips",
            )
            .figure(|| {
                phone(312.0, 40.0, 360.0).content(|| {
                    media(76.0, srgb(214, 196, 176));
                    plant_copy();
                    rule(INSET_MARGIN, INSET_MARGIN);
                    text()
                        .content("Select Type")
                        .style(type_style(12.0, 16.0))
                        .modifier(Modifier::new().padding(Padding::new(
                            Dp(16.0),
                            Dp(14.0),
                            Dp(16.0),
                            Dp(10.0),
                        )));
                    chips(
                        vec!["Succulent".into(), "Cactus".into(), "Blend".into()],
                        1,
                    );
                });
            });
        });
    gap(Dp(64.0));
    split()
        .lead(|| {
            subheading("Using dividers both ways on the same screen".into());
            gap(Dp(16.0));
            prose(
                "If dividers are used both ways in a UI, they must reinforce the hierarchy of information within different sections.",
            );
            gap(Dp(16.0));
            numbered(vec![
                "To separate a different kind of content, use a full-width divider".into(),
                "To separate nested content items, use inset dividers".into(),
            ]);
        })
        .figure(|| {
            guidance(
                DO,
                216.0,
                blush(),
                "Dividers used to show content hierarchy: full-width for sections, inset for related items",
                "Use a combination of inset and full-width dividers to reflect the hierarchy of information",
            )
            .figure(|| {
                hierarchy_figure();
            });
        });
    gap(Dp(64.0));
    split()
        .lead(|| {
            prose(
                "List items with repetitive formats may not require an inset divider, in which using only the margin between items is acceptable.",
            );
        })
        .figure(|| {
            guidance(
                DO,
                220.0,
                lavender(),
                "List with repetitive item formats. Visual separation achieved through margins only",
                "Content may not require a divider line",
            )
            .figure(|| {
                let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                row()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        boxed().modifier(
                            Modifier::new()
                                .width(Dp(120.0))
                                .fill_max_height()
                                .background(srgb(234, 182, 230)),
                        );
                        column()
                            .modifier(
                                Modifier::new()
                                    .weight(1.0)
                                    .fill_max_height()
                                    .background(scheme.surface),
                            )
                            .children(|| {
                                list_screen(PODCAST, OPEN).count(6);
                            });
                        boxed().modifier(
                            Modifier::new().width(Dp(6.0)).fill_max_height().background(frame()),
                        );
                        spacer().modifier(Modifier::new().width(Dp(60.0)));
                    });
            });
        });
}

/// Points from a label outside the device to an observed divider.
#[tessera]
fn pointer(geometry: State<SpaceGeometry>, #[prop(into)] target: String, number: Option<usize>) {
    let values = space_geometry(&geometry.get());
    let Some(line) = values.get(target.as_str()).copied() else {
        return;
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let height = if number.is_some() { 24.0 } else { 28.0 };
    row()
        .main_axis_alignment(MainAxisAlignment::End)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp::ZERO, Dp(line.y + 0.5 - height / 2.0))
                .width(Dp(line.x + 3.5)),
        )
        .children(move || {
            if let Some(number) = number {
                badge(number);
            } else {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .height(Dp(28.0))
                            .background_with_shape(srgb(43, 41, 48), Shape::CAPSULE)
                            .padding_symmetric(Dp(12.0), Dp::ZERO),
                    )
                    .children(|| {
                        text()
                            .content("Decorative")
                            .style(type_style(13.0, 18.0))
                            .color(srgb(255, 255, 255));
                    });
            }
            spacer().modifier(
                Modifier::new()
                    .size(Dp(if number.is_some() { 28.0 } else { 64.0 }), Dp(1.0))
                    .background(srgb(73, 69, 79)),
            );
            boxed().modifier(
                Modifier::new()
                    .size(Dp(7.0), Dp(7.0))
                    .background_with_shape(scheme.surface, Shape::Ellipse)
                    .border_with_shape(Dp(1.0), srgb(73, 69, 79), Shape::Ellipse),
            );
        });
}

#[tessera]
fn hierarchy_figure() {
    let geometry = remember(SpaceGeometry::new);
    boxed()
        .alignment(Alignment::TopCenter)
        .modifier(watch_in_space(geometry, "figure").fill_max_size())
        .children(move || {
            dot_grid("#ead7e8");
            phone(300.0, 26.0, 360.0).content(move || {
                row_item(CONTACTS, 0);
                rule(0.0, 0.0).modifier(watch_in_space(geometry, "full"));
                section_title("Family");
                row_item(CONTACTS, 1);
                rule(INSET_MARGIN, INSET_MARGIN).modifier(watch_in_space(geometry, "inset"));
                row_item(CONTACTS, 2);
                rule(INSET_MARGIN, INSET_MARGIN);
                row_item(CONTACTS, 3);
            });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    pointer(geometry, "full").number(1);
                    pointer(geometry, "inset").number(2);
                });
        });
}

#[tessera]
fn vertical_guidelines() {
    prose(
        "A vertical divider can be used to arrange content on a larger screen, such as separating paragraph text from video or imagery media.",
    );
    gap(Dp(48.0));
    stage(
        268.0,
        lavender(),
        "Image and text side-by-side, separated by a vertical divider on a large screen",
    )
    .content(|| {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        row()
            .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                Dp(64.0),
                Dp::ZERO,
                Dp::ZERO,
                Dp::ZERO,
            )))
            .children(move || {
                boxed().modifier(Modifier::new().width(Dp(6.0)).fill_max_height().background(frame()));
                row()
                    .modifier(
                        Modifier::new()
                            .weight(1.0)
                            .fill_max_height()
                            .background(scheme.surface)
                            .padding(Padding::new(Dp(24.0), Dp(16.0), Dp(16.0), Dp::ZERO)),
                    )
                    .children(move || {
                        boxed()
                            .modifier(Modifier::new().weight(3.0))
                            .children(|| {
                                media(252.0, srgb(78, 74, 92));
                            });
                        spacer().modifier(Modifier::new().width(Dp(6.0)));
                        boxed()
                            .modifier(Modifier::new().weight(0.8))
                            .children(|| {
                                media(252.0, srgb(34, 32, 40));
                            });
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        vertical_divider().modifier(Modifier::new().height(Dp(252.0)));
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        column()
                            .modifier(Modifier::new().weight(3.0))
                            .children(move || {
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(|| {
                                        boxed().modifier(
                                            Modifier::new()
                                                .size(Dp(28.0), Dp(28.0))
                                                .background_with_shape(swatch(5), Shape::Ellipse),
                                        );
                                        spacer().modifier(Modifier::new().width(Dp(10.0)));
                                        text()
                                            .content("by Selen Zeynep")
                                            .style(type_style(11.0, 16.0));
                                    });
                                gap(Dp(16.0));
                                text()
                                    .content("What Buttons are Artists Pushing When They Perform Live")
                                    .style(TextStyle {
                                        font_weight: 500,
                                        ..type_style(12.0, 18.0)
                                    });
                                gap(Dp(10.0));
                                text()
                                    .content(
                                        "The general consensus is that artists using digital turntables are just pushing random buttons when they perform live. But is that true?",
                                    )
                                    .style(type_style(11.0, 16.0))
                                    .color(scheme.on_surface_variant);
                            });
                    });
            });
    });
    gap(Dp(16.0));
    caption("Vertical divider in a large screen context");
}

#[tessera]
fn decorative_figure() {
    let geometry = remember(SpaceGeometry::new);
    stage(
        222.0,
        lavender(),
        "Decorative divider between two inbox messages",
    )
    .modifier(watch_in_space(geometry, "figure"))
    .content(move || {
        row()
            .main_axis_alignment(MainAxisAlignment::End)
            .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                Dp::ZERO,
                Dp(40.0),
                Dp(72.0),
                Dp::ZERO,
            )))
            .children(move || {
                phone(236.0, 0.0, 360.0).content(move || {
                    header("Inbox").search(false);
                    row_item(INBOX, 0);
                    rule(0.0, 0.0).modifier(watch_in_space(geometry, "line"));
                    row_item(INBOX, 1);
                    rule(0.0, 0.0);
                });
            });
        boxed()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                pointer(geometry, "line");
            });
    });
}

#[tessera]
fn accessibility() {
    split()
        .lead(|| {
            prose("Dividers are decorative elements, which have no contrast minimums.");
        })
        .figure(|| {
            decorative_figure();
            gap(Dp(16.0));
            caption("Decorative elements have no contrast minimums");
        });
}
