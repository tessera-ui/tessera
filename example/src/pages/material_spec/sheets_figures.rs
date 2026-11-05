//! Bottom sheet article figures drawn with native Tessera primitives.
//!
//! ## Usage
//!
//! Illustrate bottom sheet variants, anatomy, measurements and guidance scenes.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    icon::icon,
    image::image,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use super::{
    super::{ACCENT, INK, srgb},
    Geometry, annotations, caption,
    color::at,
    gap, sheet_albums, sheet_canvas, type_style,
};

fn top_rounded(radius: f64) -> Shape {
    Shape::RoundedRectangle {
        top_left: RoundedCorner::manual(Dp(radius), 3.0),
        top_right: RoundedCorner::manual(Dp(radius), 3.0),
        bottom_left: RoundedCorner::manual(Dp::ZERO, 3.0),
        bottom_right: RoundedCorner::manual(Dp::ZERO, 3.0),
    }
}

/// Drag handle centre below the container top: 22dp padding above the 4dp
/// handle, matching the 22dp top/bottom padding in the measurements table.
const HANDLE_CENTER: f64 = 24.0;

fn badge_ink() -> Color {
    srgb(48, 48, 52)
}

fn watch_figure(geometry: State<Geometry>, name: &'static str) -> Modifier {
    annotations::watch_precise(geometry, name).semantics(SemanticsArgs {
        test_tag: Some(format!("sheet-figure-{name}")),
        merge_descendants: true,
        ..Default::default()
    })
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Glyph {
    #[default]
    Share,
    Add,
    Trash,
    Bag,
    Archive,
    Link,
    Edit,
    Copy,
    Nearby,
    Folder,
    Document,
    Checked,
    Unchecked,
    Mail,
    Video,
    Drive,
}

#[tessera]
fn glyph(kind: Glyph, x: f64, y: f64, size: f64, tint: Color) {
    boxed().modifier(at(x, y)).children(move || {
        let builder = icon().size(Dp(size)).tint(tint);
        match kind {
            Glyph::Share => builder.vector(filled::SHARE_SVG),
            Glyph::Add => builder.vector(filled::ADD_SVG),
            Glyph::Trash => builder.vector(filled::DELETE_SVG),
            Glyph::Bag => builder.vector(filled::SHOPPING_BAG_SVG),
            Glyph::Archive => builder.vector(filled::ARCHIVE_SVG),
            Glyph::Link => builder.vector(filled::LINK_SVG),
            Glyph::Edit => builder.vector(filled::EDIT_SVG),
            Glyph::Copy => builder.vector(filled::CONTENT_COPY_SVG),
            Glyph::Nearby => builder.vector(filled::NEAR_ME_SVG),
            Glyph::Folder => builder.vector(filled::FOLDER_SVG),
            Glyph::Document => builder.vector(filled::DESCRIPTION_SVG),
            Glyph::Checked => builder.vector(filled::CHECK_BOX_SVG),
            Glyph::Unchecked => builder.vector(filled::CHECK_BOX_OUTLINE_BLANK_SVG),
            Glyph::Mail => builder.vector(filled::MAIL_SVG),
            Glyph::Video => builder.vector(filled::VIDEOCAM_SVG),
            Glyph::Drive => builder.vector(filled::ADD_TO_DRIVE_SVG),
        };
    });
}

#[tessera]
fn block(x: f64, y: f64, w: f64, h: f64, color: Color, shape: Option<Shape>) {
    spacer().modifier(
        at(x, y)
            .required_size(Dp(w), Dp(h))
            .background_with_shape(color, shape.unwrap_or(Shape::RECTANGLE)),
    );
}

#[tessera]
fn ring(x: f64, y: f64, w: f64, h: f64, stroke: f64, color: Color, fill: Option<Color>) {
    spacer().modifier(
        at(x, y)
            .required_size(Dp(w), Dp(h))
            .background_with_shape(fill.unwrap_or(Color::TRANSPARENT), Shape::CAPSULE)
            .border_with_shape(Dp(stroke), color, Shape::CAPSULE),
    );
}

#[tessera]
fn label(#[prop(into)] content: String, x: f64, y: f64, size: f64, color: Color) {
    text()
        .content(content)
        .style(type_style(size, (size * 1.4).round()))
        .color(color)
        .modifier(at(x, y));
}

#[tessera]
fn centered(#[prop(into)] content: String, center: f64, y: f64, size: f64, color: Color) {
    boxed()
        .alignment(Alignment::TopCenter)
        .modifier(at(center - 60.0, y).required_width(Dp(120.0)))
        .children(move || {
            text()
                .content(content.clone())
                .style(type_style(size, (size * 1.4).round()))
                .color(color);
        });
}

/// Lavender figure stage with canonical coordinates scaled to the column width.
#[tessera]
fn stage(
    width: f64,
    height: f64,
    content: RenderSlot,
    geometry: Option<State<Geometry>>,
    background: Option<Color>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let background = background.unwrap_or(scheme.secondary_container);
    let shape = Shape::rounded_rectangle(Dp(24.0));
    sheet_canvas(
        width,
        height,
        RenderSlot::new(move || {
            let mut modifier = Modifier::new();
            if let Some(geometry) = geometry {
                modifier = watch_figure(geometry, "figure");
            }
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    modifier
                        .fill_max_size()
                        .background_with_shape(background, shape)
                        .clip_shape(shape),
                )
                .children_shared(content);
        }),
    );
}

/// Phone frame whose screen hosts app content, an optional scrim and overlay.
#[tessera]
fn phone(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    screen: Option<RenderSlot>,
    overlay: Option<RenderSlot>,
    scrim: Option<bool>,
    screen_color: Option<Color>,
    geometry: Option<State<Geometry>>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let screen = screen.unwrap_or_else(RenderSlot::empty);
    let overlay = overlay.unwrap_or_else(RenderSlot::empty);
    let scrim = scrim.unwrap_or(false);
    let screen_color = screen_color.unwrap_or(scheme.surface_container);
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            at(x, y)
                .required_size(Dp(w), Dp(h))
                .background_with_shape(scheme.outline_variant, Shape::rounded_rectangle(Dp(28.0))),
        )
        .children(move || {
            let mut modifier = at(7.0, 7.0);
            if let Some(geometry) = geometry {
                modifier = modifier.then(watch_figure(geometry, "screen"));
            }
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    modifier
                        .required_size(Dp(w - 14.0), Dp(h - 14.0))
                        .background_with_shape(screen_color, Shape::rounded_rectangle(Dp(21.0)))
                        .clip_shape(Shape::rounded_rectangle(Dp(21.0))),
                )
                .children(move || {
                    screen.render();
                    if scrim {
                        let mut modifier = Modifier::new();
                        if let Some(geometry) = geometry {
                            modifier = watch_figure(geometry, "scrim");
                        }
                        spacer().modifier(
                            modifier
                                .fill_max_size()
                                .background(scheme.scrim.with_alpha(0.32)),
                        );
                    }
                    overlay.render();
                });
        });
}

/// Bottom sheet container with 28dp top corners and an optional drag handle.
#[tessera]
fn sheet(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    handle: Option<f64>,
    handle_x: Option<f64>,
    content: Option<RenderSlot>,
    geometry: Option<State<Geometry>>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let content = content.unwrap_or_else(RenderSlot::empty);
    let mut modifier = at(x, y);
    if let Some(geometry) = geometry {
        modifier = modifier.then(watch_figure(geometry, "container"));
    }
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            modifier
                .required_size(Dp(w), Dp(h))
                .background_with_shape(scheme.surface_container_low, top_rounded(28.0))
                .clip_shape(top_rounded(28.0)),
        )
        .children(move || {
            if let Some(center) = handle {
                let mut modifier = at(handle_x.unwrap_or(w / 2.0) - 16.0, center - 2.0);
                if let Some(geometry) = geometry {
                    modifier = modifier.then(watch_figure(geometry, "handle"));
                }
                spacer().modifier(
                    modifier
                        .required_size(Dp(32.0), Dp(4.0))
                        .background_with_shape(
                            scheme.on_surface_variant.with_alpha(0.4),
                            Shape::CAPSULE,
                        ),
                );
            }
            content.render();
        });
}

/// Numbered annotation badge used in figures and legends.
#[tessera]
fn badge(
    number: usize,
    x: f64,
    y: f64,
    round: Option<bool>,
    outlined: Option<bool>,
    #[prop(into)] name: Option<String>,
) {
    let round = round.unwrap_or(false);
    let outlined = outlined.unwrap_or(false);
    let (w, h) = if round { (36.0, 36.0) } else { (24.0, 32.0) };
    let name = name.unwrap_or_else(|| format!("Bottom sheet marker {number}"));
    let mut modifier = at(x, y)
        .required_size(Dp(w), Dp(h))
        .background_with_shape(
            if outlined {
                srgb(254, 247, 255)
            } else {
                badge_ink()
            },
            Shape::CAPSULE,
        )
        .semantics(SemanticsArgs {
            role: Some(Role::Note),
            label: Some(name),
            merge_descendants: true,
            ..Default::default()
        });
    if outlined {
        modifier = modifier.border_with_shape(Dp(1.0), srgb(73, 69, 79), Shape::CAPSULE);
    }
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(if outlined { INK } else { Color::WHITE });
        });
}

/// Numbered legend whose badges match the figure annotations.
#[tessera]
pub(super) fn badge_legend(items: Vec<String>) {
    gap(Dp(12.0));
    column().children(move || {
        for (index, item) in items.iter().enumerate() {
            let item = item.clone();
            row()
                .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
                .children(move || {
                    boxed()
                        .modifier(Modifier::new().size(Dp(24.0), Dp(32.0)))
                        .children(move || {
                            badge(index + 1, 0.0, 0.0)
                                .name(format!("Bottom sheet legend {}", index + 1));
                        });
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    text()
                        .content(item.clone())
                        .style(type_style(14.0, 20.0))
                        .color(srgb(73, 69, 79));
                });
            gap(Dp(8.0));
        }
    });
}

fn leader(svg: &mut String, from: (f64, f64), to: (f64, f64)) {
    svg.push_str(&format!(
        "<path d='M{},{}L{},{}' stroke='#49454f' stroke-width='1' fill='none'/><circle cx='{}' cy='{}' r='3' fill='none' stroke='#49454f'/>",
        from.0, from.1, to.0, to.1, to.0, to.1
    ));
}

fn line(svg: &mut String, from: (f64, f64), to: (f64, f64), color: &str) {
    svg.push_str(&format!(
        "<path d='M{},{}L{},{}' stroke='{color}' stroke-width='1' fill='none'/>",
        from.0, from.1, to.0, to.1
    ));
}

fn ruler(svg: &mut String, from: (f64, f64), to: (f64, f64)) {
    line(svg, from, to, "#e85c79");
    let (dx, dy) = if from.1 == to.1 {
        (0.0, 4.0)
    } else {
        (4.0, 0.0)
    };
    for (x, y) in [from, to] {
        line(svg, (x - dx, y - dy), (x + dx, y + dy), "#e85c79");
    }
}

/// Split two slots side by side on wide articles and stack them on narrow ones.
#[tessera]
pub(super) fn split(start: Option<RenderSlot>, end: Option<RenderSlot>, start_weight: Option<f32>) {
    let start = start.unwrap_or_else(RenderSlot::empty);
    let end = end.unwrap_or_else(RenderSlot::empty);
    let start_weight = start_weight.unwrap_or(0.47);
    let wide = remember(|| true);
    column()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width).0 >= 720.0;
                if wide.is_alive() && wide.get() != next {
                    wide.set(next);
                }
            },
        ))
        .children(move || {
            if wide.get() {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().weight(start_weight))
                            .children(move || start.render());
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || end.render());
                    });
            } else {
                start.render();
                gap(Dp(24.0));
                end.render();
            }
        });
}

/// A figure followed by its caption.
#[tessera]
pub(super) fn captioned(figure: RenderSlot, #[prop(into)] note: String) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            figure.render();
            gap(Dp(16.0));
            caption(note.clone());
        });
    gap(Dp(32.0));
}

/// Two captioned figures shown side by side.
#[tessera]
pub(super) fn figure_pair(
    first: RenderSlot,
    #[prop(into)] first_caption: String,
    second: RenderSlot,
    #[prop(into)] second_caption: String,
) {
    split()
        .start_weight(1.0)
        .start(move || {
            captioned(first, first_caption.clone());
        })
        .end(move || {
            captioned(second, second_caption.clone());
        });
}

/// Standard and modal bottom sheet variants with numbered badges.
#[tessera]
pub(super) fn variants_figure() {
    stage(
        1024.0,
        417.0,
        RenderSlot::new(|| {
            for (index, x) in [(0usize, 63.0), (1, 545.0)] {
                phone(x, -40.0, 414.0, 366.0).scrim(index == 1).overlay(|| {
                    sheet(0.0, 148.0, 400.0, 204.0).handle(18.0);
                });
                badge(index + 1, x + 189.0, 351.0)
                    .round(true)
                    .name(format!("Bottom sheet variant {}", index + 1));
            }
        }),
    );
}

/// Floating M3 modal bottom sheet above a scrim.
#[tessera]
pub(super) fn m2_figure() {
    stage(
        600.0,
        370.0,
        RenderSlot::new(|| {
            phone(100.0, 46.0, 400.0, 360.0).scrim(true).overlay(|| {
                sheet(0.0, 164.0, 386.0, 200.0).handle(21.0);
            });
        }),
    );
}

/// Bottom sheet anatomy with dark or outlined numbered markers.
#[tessera]
pub(super) fn anatomy_figure(outlined: bool) {
    let geometry = remember(Geometry::new);
    stage(
        1024.0,
        502.0,
        RenderSlot::new(move || {
            phone(83.0, -40.0, 857.0, 460.0)
                .scrim(true)
                .geometry(geometry)
                .overlay(move || {
                    sheet(102.0, 199.0, 640.0, 247.0)
                        .handle(HANDLE_CENTER)
                        .geometry(geometry);
                });
            let mut svg = String::new();
            leader(&mut svg, (284.0, 137.0), (284.0, 184.0));
            line(&mut svg, (512.0, 137.0), (512.0, 178.0), "#49454f");
            svg.push_str(
                "<rect x='480.5' y='178.5' width='64' height='25' fill='none' stroke='#49454f' stroke-width='1'/>",
            );
            leader(&mut svg, (957.0, 121.0), (901.0, 121.0));
            annotations::lines(Dp(1024.0), Dp(502.0), svg);
            for (number, x) in [(1usize, 272.0), (2, 500.0), (3, 957.0)] {
                badge(number, x, 105.0).outlined(outlined);
            }
        }),
    )
    .geometry(geometry);
}

/// Bottom sheet margins, maximum width and top spacing.
#[tessera]
pub(super) fn measurements_figure() {
    let geometry = remember(Geometry::new);
    stage(
        1024.0,
        836.0,
        RenderSlot::new(move || {
            phone(83.0, 90.0, 857.0, 655.0)
                .scrim(true)
                .geometry(geometry)
                .overlay(move || {
                    sheet(102.0, 302.0, 640.0, 339.0)
                        .handle(HANDLE_CENTER)
                        .geometry(geometry);
                });
            let mut svg = String::new();
            for (x, y, w, h) in [
                (136.0, 327.0, 752.0, 72.0),
                (136.0, 399.0, 56.0, 339.0),
                (832.0, 399.0, 56.0, 339.0),
            ] {
                svg.push_str(&format!(
                    "<rect x='{x}' y='{y}' width='{w}' height='{h}' fill='#e8457a' fill-opacity='.26'/>"
                ));
            }
            ruler(&mut svg, (954.0, 327.0), (954.0, 399.0));
            ruler(&mut svg, (136.0, 784.0), (192.0, 784.0));
            ruler(&mut svg, (192.0, 784.0), (832.0, 784.0));
            ruler(&mut svg, (832.0, 784.0), (888.0, 784.0));
            annotations::lines(Dp(1024.0), Dp(836.0), svg);
            for (value, x, y) in [
                ("72", 964.0, 352.0),
                ("56", 156.0, 756.0),
                ("640", 500.0, 756.0),
                ("56", 852.0, 756.0),
            ] {
                annotations::dimension(value.into(), Dp(x), Dp(y));
            }
        }),
    )
    .geometry(geometry);
}

#[tessera]
fn files_list(tablet: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let rows = [
        ("Project brief", Glyph::Folder, "2 days ago"),
        ("Onboarding documents", Glyph::Folder, "2 hours ago"),
        ("Clinic codes", Glyph::Folder, "Yesterday"),
        ("Certificate program", Glyph::Folder, "Nov 8"),
        ("On-call protocol", Glyph::Document, "Nov 5"),
        ("Cover letter template", Glyph::Document, "Oct 28"),
        ("Resume", Glyph::Document, "Oct 12"),
        ("CDC Health Literacy", Glyph::Document, "Sep 26"),
    ];
    let (first, pitch, icon_x, name_x, owner_x, divider) = if tablet {
        (-29.0, 43.0, 27.0, 63.0, 245.0, 15.0)
    } else {
        (-24.0, 42.0, 15.0, 51.0, 237.0, 15.0)
    };
    for (index, (name, kind, date)) in rows.into_iter().enumerate() {
        let center = first + index as f64 * pitch;
        glyph(
            kind,
            icon_x,
            center - 10.0,
            20.0,
            if kind == Glyph::Folder {
                scheme.on_surface
            } else {
                srgb(66, 133, 244)
            },
        );
        label(name, name_x, center - 9.0, 13.0, scheme.on_surface);
        label(
            "Lee Villanueva",
            owner_x,
            center - 9.0,
            13.0,
            scheme.on_surface_variant,
        );
        if tablet {
            label(date, 420.0, center - 9.0, 13.0, scheme.on_surface_variant);
        }
        block(
            divider,
            center + pitch / 2.0,
            1000.0,
            1.0,
            scheme.outline_variant,
        );
    }
}

#[tessera]
fn filter_choices(second_icon: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    label("Filters", 22.0, 45.0, 14.0, scheme.on_surface);
    for (index, name) in [
        "Work files",
        "Personal",
        "Projects",
        "Priority",
        "Reminders",
        "Events",
    ]
    .into_iter()
    .enumerate()
    {
        let column = index / 3;
        let center = 98.0 + (index % 3) as f64 * 43.0;
        let icon_x = if column == 0 { 21.0 } else { second_icon };
        glyph(
            if index == 0 {
                Glyph::Checked
            } else {
                Glyph::Unchecked
            },
            icon_x,
            center - 11.0,
            22.0,
            if index == 0 {
                scheme.primary
            } else {
                scheme.on_surface_variant
            },
        );
        label(
            name,
            icon_x + 34.0,
            center - 9.0,
            13.0,
            scheme.on_surface_variant,
        );
    }
}

/// Modal bottom sheet with file filters above a file list.
#[tessera]
pub(super) fn files_figure() {
    stage(
        600.0,
        355.0,
        RenderSlot::new(|| {
            phone(111.0, -40.0, 381.0, 312.0)
                .scrim(true)
                .screen(|| {
                    files_list(false);
                })
                .overlay(|| {
                    sheet(0.0, 103.0, 367.0, 195.0).handle(15.0).content(|| {
                        filter_choices(193.0);
                    });
                });
        }),
    );
}

/// Larger-screen bottom sheet capped at its maximum width.
#[tessera]
pub(super) fn tablet_figure() {
    stage(
        600.0,
        355.0,
        RenderSlot::new(|| {
            phone(56.0, -40.0, 700.0, 330.0)
                .scrim(true)
                .screen(|| {
                    files_list(true);
                })
                .overlay(|| {
                    sheet(185.0, 121.0, 422.0, 195.0).handle(15.0).content(|| {
                        filter_choices(210.0);
                    });
                });
        }),
    );
}

#[tessera]
fn photo(index: usize, x: f64, y: f64, w: f64, h: f64, radius: Option<f64>) {
    image()
        .raster(tessera_ui::remember_with_key(index, move || sheet_albums()[index].clone()).get())
        .modifier(
            at(x, y)
                .required_size(Dp(w), Dp(h))
                .clip_shape(Shape::rounded_rectangle(Dp(radius.unwrap_or(0.0)))),
        );
}

#[tessera]
fn chip(kind: Glyph, #[prop(into)] name: String, x: f64, w: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            at(x, 44.0)
                .required_size(Dp(w), Dp(35.0))
                .border_with_shape(Dp(1.0), scheme.outline, Shape::CAPSULE),
        )
        .children(move || {
            glyph(kind, 14.0, 9.0, 17.0, scheme.on_surface_variant);
            label(name.clone(), 39.0, 8.0, 13.0, scheme.on_surface_variant);
        });
}

#[tessera]
fn avatar(#[prop(into)] initial: String, center: f64, y: f64, diameter: f64, color: Color) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            at(center - diameter / 2.0, y)
                .required_size(Dp(diameter), Dp(diameter))
                .background_with_shape(color, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(initial.clone())
                .style(type_style(diameter * 0.4, diameter * 0.5))
                .color(Color::WHITE);
        });
}

fn avatar_color(index: usize) -> Color {
    [
        srgb(121, 85, 72),
        srgb(0, 121, 107),
        srgb(198, 40, 40),
        srgb(94, 53, 177),
        srgb(230, 81, 0),
    ][index % 5]
}

/// Photo-sharing bottom sheet with contacts and applications.
#[tessera]
pub(super) fn share_figure() {
    stage(
        600.0,
        545.0,
        RenderSlot::new(|| {
            phone(111.0, -40.0, 381.0, 491.0)
                .screen_color(srgb(30, 27, 22))
                .screen(|| {
                    photo(0, 39.0, 0.0, 290.0, 133.0);
                })
                .overlay(|| {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    sheet(0.0, 156.0, 367.0, 321.0)
                        .handle(16.0)
                        .content(move || {
                            chip(Glyph::Copy, "Copy", 83.0, 88.0);
                            chip(Glyph::Nearby, "Nearby", 179.0, 105.0);
                            for (index, (name, center)) in [
                                ("Alejandro", 54.0),
                                ("Ines", 140.0),
                                ("Oli", 226.0),
                                ("Carmen", 312.0),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                avatar(&name[..1], center, 101.0, 53.0, avatar_color(index));
                                centered(name, center, 162.0, 12.0, scheme.on_surface);
                            }
                            for (name, center, kind, color) in [
                                ("Files", 54.0, Glyph::Folder, srgb(66, 133, 244)),
                                ("Gmail", 140.0, Glyph::Mail, srgb(234, 67, 53)),
                                ("Meet", 226.0, Glyph::Video, srgb(52, 168, 83)),
                                ("Drive", 312.0, Glyph::Drive, srgb(251, 188, 4)),
                            ] {
                                block(center - 26.0, 203.0, 53.0, 53.0, Color::WHITE)
                                    .shape(Shape::Ellipse);
                                glyph(kind, center - 14.0, 215.0, 28.0, color);
                                centered(name, center, 264.0, 12.0, scheme.on_surface);
                            }
                            block(151.0, 305.0, 64.0, 2.0, scheme.primary).shape(Shape::CAPSULE);
                        });
                });
        }),
    );
}

#[tessera]
fn menu_stage() {
    stage(
        600.0,
        356.0,
        RenderSlot::new(|| {
            phone(111.0, -40.0, 381.0, 312.0)
                .screen_color(srgb(120, 90, 60))
                .screen(|| {
                    photo(1, 0.0, 0.0, 367.0, 100.0);
                })
                .overlay(|| {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    sheet(0.0, 57.0, 367.0, 241.0).content(move || {
                        for (index, (kind, name)) in [
                            (Glyph::Share, "Share"),
                            (Glyph::Link, "Get link"),
                            (Glyph::Edit, "Edit name"),
                            (Glyph::Trash, "Delete"),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            let center = 56.0 + index as f64 * 53.0;
                            glyph(kind, 21.0, center - 11.0, 22.0, scheme.on_surface_variant);
                            label(name, 70.0, center - 10.0, 14.0, scheme.on_surface);
                        }
                        block(151.0, 228.0, 64.0, 2.0, scheme.primary).shape(Shape::CAPSULE);
                    });
                });
        }),
    );
}

/// Bottom sheet listing menu actions, optionally in the dark scheme.
#[tessera]
pub(super) fn menu_figure(dark: bool) {
    if dark {
        material_theme()
            .theme(|| MaterialTheme::from_seed(ACCENT, true))
            .child(|| {
                menu_stage();
            });
    } else {
        menu_stage();
    }
}

/// Photo bottom sheet at a chosen initial height over a photo grid.
#[tessera]
pub(super) fn photo_actions_figure(sheet_top: f64) {
    stage(
        600.0,
        735.0,
        RenderSlot::new(move || {
            phone(111.0, 49.0, 368.0, 641.0)
                .screen_color(srgb(255, 255, 255))
                .screen(|| {
                    photo(0, 0.0, 0.0, 141.0, 140.0);
                    photo(1, 147.0, 0.0, 207.0, 140.0);
                    photo(2, 0.0, 146.0, 219.0, 165.0);
                    photo(3, 225.0, 146.0, 129.0, 165.0);
                    photo(1, 0.0, 317.0, 165.0, 160.0);
                    photo(0, 171.0, 317.0, 183.0, 160.0);
                })
                .overlay(move || {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    sheet(0.0, sheet_top, 354.0, 627.0 - sheet_top)
                        .handle(15.0)
                        .content(move || {
                            for (index, (kind, name)) in [
                                (Glyph::Share, "Share"),
                                (Glyph::Add, "Add to"),
                                (Glyph::Trash, "Trash"),
                                (Glyph::Bag, "Order\nprints"),
                                (Glyph::Archive, "Move to\narchive"),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let center = 39.0 + index as f64 * 69.0;
                                glyph(kind, center - 11.0, 50.0, 22.0, scheme.on_surface_variant);
                                centered(name, center, 82.0, 11.0, scheme.on_surface);
                            }
                            block(15.0, 152.0, 324.0, 1.0, scheme.outline_variant);
                            label("Send", 15.0, 165.0, 15.0, scheme.on_surface);
                            for (index, name) in [
                                "Alejandro\nOrtega",
                                "Oli Ortega",
                                "Carmen\nVillanueva",
                                "Ana Russo",
                                "Marty Reyes",
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let center = 39.0 + index as f64 * 69.0;
                                avatar(&name[..1], center, 202.0, 43.0, avatar_color(index));
                                centered(name, center, 250.0, 10.0, scheme.on_surface);
                            }
                            label("Add to album", 15.0, 301.0, 15.0, scheme.on_surface);
                            label("View all", 280.0, 302.0, 13.0, scheme.primary);
                            photo(2, 15.0, 336.0, 100.0, 100.0).radius(12.0);
                            photo(3, 125.0, 336.0, 100.0, 100.0).radius(12.0);
                            photo(1, 235.0, 336.0, 100.0, 100.0).radius(12.0);
                        });
                });
        }),
    );
}

fn touch_color() -> Color {
    srgb(128, 64, 140)
}

/// Tapping the scrim dismisses a modal bottom sheet.
#[tessera]
pub(super) fn dismiss_scrim_figure() {
    stage(
        300.0,
        272.0,
        RenderSlot::new(|| {
            phone(-150.0, -40.0, 393.0, 278.0)
                .scrim(true)
                .screen(|| {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    for (index, (start, name)) in [
                        ("", "Lee Villanueva"),
                        ("uri", "Sunita Saachi"),
                        ("uri", "Lee Villanueva"),
                        ("nning", "Daniel Maas"),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let center = 29.0 + index as f64 * 42.0;
                        label(start, 143.0, center - 8.0, 12.0, scheme.on_surface);
                        label(name, 245.0, center - 8.0, 12.0, scheme.on_surface);
                        block(143.0, center + 21.0, 300.0, 1.0, scheme.outline_variant);
                    }
                })
                .overlay(|| {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    ring(232.0, 31.0, 51.0, 51.0, 0.0, Color::TRANSPARENT)
                        .fill(touch_color().with_alpha(0.65));
                    sheet(0.0, 142.0, 379.0, 100.0)
                        .handle(16.0)
                        .handle_x(197.0)
                        .content(move || {
                            glyph(
                                Glyph::Unchecked,
                                184.0,
                                85.0,
                                18.0,
                                scheme.on_surface_variant,
                            );
                            label("Priority", 212.0, 85.0, 13.0, scheme.on_surface_variant);
                        });
                });
        }),
    );
}

/// Swiping the sheet down dismisses a modal bottom sheet.
#[tessera]
pub(super) fn dismiss_swipe_figure() {
    stage(
        300.0,
        274.0,
        RenderSlot::new(|| {
            phone(-7.0, -40.0, 400.0, 293.0)
                .screen_color(srgb(255, 255, 255))
                .screen(|| {
                    photo(2, 0.0, 0.0, 386.0, 158.0);
                })
                .overlay(|| {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    sheet(1.0, 125.0, 385.0, 154.0)
                        .handle(15.0)
                        .handle_x(54.0)
                        .content(move || {
                            for (kind, name, center) in [
                                (Glyph::Trash, "Trash", 55.0),
                                (Glyph::Bag, "Order\nprints", 127.0),
                                (Glyph::Archive, "Move to\narchive", 198.0),
                            ] {
                                glyph(kind, center - 11.0, 49.0, 22.0, scheme.on_surface_variant);
                                centered(name, center, 81.0, 11.0, scheme.on_surface);
                            }
                        });
                    ring(142.0, 89.0, 51.0, 51.0, 0.0, Color::TRANSPARENT)
                        .fill(touch_color().with_alpha(0.65));
                });
        }),
    );
}

/// The top 48dp of a bottom sheet reserved for resize interactions.
#[tessera]
pub(super) fn touch_area_figure() {
    stage(
        600.0,
        420.0,
        RenderSlot::new(|| {
            phone(116.0, -194.0, 370.0, 640.0)
                .screen_color(srgb(254, 247, 255))
                .overlay(|| {
                    sheet(0.0, 300.0, 356.0, 326.0)
                        .handle(HANDLE_CENTER)
                        .content(|| {
                            block(0.0, 0.0, 356.0, 48.0, srgb(240, 160, 240).with_alpha(0.7));
                            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                            block(
                                162.0,
                                HANDLE_CENTER - 2.0,
                                32.0,
                                4.0,
                                scheme.on_surface_variant.with_alpha(0.7),
                            )
                            .shape(Shape::CAPSULE);
                        });
                });
        }),
    );
}

/// Visible focus on the drag handle reached with the Tab key.
#[tessera]
pub(super) fn focus_figure() {
    stage(
        600.0,
        420.0,
        RenderSlot::new(|| {
            phone(116.0, -194.0, 370.0, 497.0)
                .screen_color(srgb(254, 247, 255))
                .overlay(|| {
                    sheet(0.0, 304.0, 356.0, 179.0)
                        .handle(HANDLE_CENTER)
                        .content(|| {
                            ring(156.0, HANDLE_CENTER - 16.0, 44.0, 32.0, 3.0, touch_color());
                        });
                });
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    at(266.0, 341.0)
                        .required_size(Dp(66.0), Dp(36.0))
                        .background_with_shape(touch_color(), Shape::rounded_rectangle(Dp(4.0))),
                )
                .children(|| {
                    text()
                        .content("Tab")
                        .style(type_style(12.0, 16.0))
                        .color(Color::WHITE);
                });
        }),
    );
}

#[tessera]
fn map_background() {
    block(0.0, 0.0, 400.0, 300.0, srgb(242, 242, 240));
    block(0.0, 95.0, 104.0, 100.0, srgb(206, 234, 214));
    for (x, y, w, h) in [
        (0.0, 50.0, 300.0, 4.0),
        (0.0, 210.0, 300.0, 4.0),
        (120.0, 0.0, 4.0, 300.0),
        (240.0, 0.0, 4.0, 300.0),
    ] {
        block(x, y, w, h, Color::WHITE);
    }
}

/// Drag handle focus at a lower preset height, or the sheet raised afterwards.
#[tessera]
pub(super) fn drag_figure(raised: bool) {
    stage(
        300.0,
        275.0,
        RenderSlot::new(move || {
            phone(13.0, -40.0, 400.0, 307.0)
                .screen(|| {
                    map_background();
                })
                .overlay(move || {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    if raised {
                        sheet(0.0, 62.0, 386.0, 231.0)
                            .handle(14.0)
                            .handle_x(182.0)
                            .content(move || {
                                label("Latest in the area", 23.0, 31.0, 19.0, scheme.on_surface);
                                photo(2, 23.0, 88.0, 128.0, 112.0).radius(12.0);
                                photo(3, 163.0, 88.0, 128.0, 112.0).radius(12.0);
                            });
                    } else {
                        sheet(0.0, 189.0, 386.0, 104.0).content(move || {
                            label("Latest in the area", 23.0, 37.0, 19.0, scheme.on_surface);
                        });
                        ring(176.0, 158.0, 51.0, 51.0, 4.0, touch_color())
                            .fill(srgb(171, 128, 184));
                        block(189.0, 182.0, 25.0, 3.0, scheme.on_surface_variant)
                            .shape(Shape::CAPSULE);
                    }
                });
        }),
    );
}

/// Accessibility label and role announced for the drag handle.
#[tessera]
pub(super) fn label_figure() {
    stage(
        600.0,
        420.0,
        RenderSlot::new(|| {
            phone(-212.0, -40.0, 492.0, 389.0)
                .screen_color(srgb(254, 247, 255))
                .overlay(|| {
                    sheet(0.0, 162.0, 478.0, 213.0).handle(24.0).handle_x(311.0);
                });
            let mut svg = String::new();
            svg.push_str(
                "<path d='M126,180L353,180' stroke='#7d3c8c' stroke-width='3' fill='none'/>",
            );
            annotations::lines(Dp(600.0), Dp(420.0), svg);
            ring(65.0, 165.0, 52.0, 30.0, 3.0, touch_color());
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    at(323.0, 102.0)
                        .required_size(Dp(240.0), Dp(130.0))
                        .background_with_shape(
                            srgb(254, 247, 255),
                            Shape::rounded_rectangle(Dp(10.0)),
                        )
                        .border_with_shape(
                            Dp(3.0),
                            touch_color(),
                            Shape::rounded_rectangle(Dp(10.0)),
                        ),
                )
                .children(|| {
                    label("Label", 20.0, 20.0, 13.0, INK);
                    label("Drag handle", 20.0, 40.0, 13.0, touch_color());
                    label("Role", 20.0, 70.0, 13.0, INK);
                    label("Button", 20.0, 90.0, 13.0, touch_color());
                });
        }),
    );
}
