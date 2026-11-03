//! Navigation drawer guideline and accessibility illustrations.
//!
//! ## Usage
//!
//! Compare recommended and discouraged drawer layouts with their published
//! guidance.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    divider::horizontal_divider,
    icon::icon,
    material_icons::{filled, outlined},
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_bar::{navigation_bar, navigation_bar_item},
    navigation_drawer::{NavigationDrawerStyle, navigation_drawer, navigation_drawer_item},
    row::row,
    scrollable::ScrollableController,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, TextStyle},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _, remember,
    tessera, use_context,
};

use super::{
    Geometry, annotations, bullets, drawer_content, drawer_figure, drawer_marker, drawer_scene,
    drawer_stage, gap, launch, leader, legend_entries, mail_messages, prose, responsive_figure,
    small_rail, spec_table, srgb, subheading, type_style, visibility_examples,
};

const NONE: u8 = 0;
const DO: u8 = 1;
const CAUTION: u8 = 2;
const DONT: u8 = 3;

#[derive(Clone, Copy)]
struct GuideSpec {
    width: f64,
    height: f64,
    tone: u8,
    label: &'static str,
    caption: &'static str,
}

const fn spec_of(
    width: f64,
    height: f64,
    tone: u8,
    label: &'static str,
    caption: &'static str,
) -> GuideSpec {
    GuideSpec {
        width,
        height,
        tone,
        label,
        caption,
    }
}

fn spec(kind: u8) -> GuideSpec {
    match kind {
        0 => spec_of(
            1024.0,
            572.0,
            NONE,
            "Navigation drawer with 4 primary destinations",
            "",
        ),
        1 => spec_of(
            680.0,
            576.0,
            DO,
            "Navigation drawer with multiple destinations in a mail app",
            "Use a navigation drawer for 5 or more primary destinations, or more than 1 level of navigation hierarchy",
        ),
        2 => spec_of(
            680.0,
            576.0,
            CAUTION,
            "Standard navigation drawer and navigation bar used together",
            "Avoid using two navigation components on the same screen",
        ),
        3 => spec_of(
            460.0,
            600.0,
            NONE,
            "Standard navigation drawer with destinations in mail app",
            "Standard navigation drawer",
        ),
        4 => spec_of(
            460.0,
            600.0,
            NONE,
            "Modal navigation drawer with destinations and scrim",
            "Modal navigation drawer",
        ),
        5 => spec_of(
            1024.0,
            600.0,
            NONE,
            "Standard navigation drawer next to app content",
            "Standard navigation drawer providing access to drawer destinations next to app content",
        ),
        6 => spec_of(
            680.0,
            576.0,
            NONE,
            "Modal navigation drawer with 1 active destination and scrim",
            "Modal navigation drawer using a scrim to block interaction with the rest of an app’s content",
        ),
        7 => spec_of(
            680.0,
            443.0,
            NONE,
            "Navigation menu icon in a navigation rail",
            "A modal drawer opened by an action such as clicking a navigation menu icon (1)",
        ),
        8 => spec_of(
            1024.0,
            868.0,
            NONE,
            "Dismissing a modal drawer by tapping the scrim or swiping",
            "1. Dismiss by tapping the scrim\n2. Dismiss by swiping the drawer",
        ),
        10 => spec_of(
            680.0,
            576.0,
            DO,
            "Modal navigation drawer opening from the left side of the screen",
            "A navigation drawer opens from the left side of the screen for left-to-right languages",
        ),
        11 => spec_of(
            400.0,
            698.0,
            DO,
            "Full-width divider separating a group of destinations",
            "Use full-width dividers (1) to separate groups of destinations",
        ),
        12 => spec_of(
            400.0,
            698.0,
            DONT,
            "Dividers separating individual destinations",
            "Don’t use dividers to separate individual destinations",
        ),
        13 => spec_of(
            720.0,
            360.0,
            NONE,
            "Active indicator in a navigation drawer",
            "The active indicator (1) is a background shape communicating which destination of the navigation drawer is currently being displayed",
        ),
        14 => spec_of(
            720.0,
            346.0,
            NONE,
            "Icon and label text of a navigation drawer destination",
            "Actionable list items in a navigation drawer describe each destination using (1) an optional icon and (2) required label text",
        ),
        15 => spec_of(
            720.0,
            374.0,
            NONE,
            "Navigation drawer using only label text",
            "Navigation drawers can use text labels without icons",
        ),
        16 => spec_of(
            400.0,
            396.0,
            DO,
            "Navigation drawer with a truncated text label",
            "Keep text labels concise, but truncate them if they extend beyond the container width",
        ),
        17 => spec_of(
            400.0,
            396.0,
            DONT,
            "Navigation drawer with wrapped label text",
            "Don’t wrap label text",
        ),
        18 => spec_of(
            400.0,
            396.0,
            DONT,
            "Navigation drawer with a smaller text label",
            "Don’t shrink text size in order to fit a text label on a single line",
        ),
        19 => spec_of(
            400.0,
            396.0,
            DO,
            "Navigation drawer with recognizable icons",
            "Use recognizable icons when conventions exist",
        ),
        20 => spec_of(
            400.0,
            396.0,
            DONT,
            "Navigation drawer with icons on only some destinations",
            "Don’t apply icons to some destinations and not others. Icons should be used for all destinations, or none.",
        ),
        21 => spec_of(
            720.0,
            288.0,
            NONE,
            "Navigation drawer subhead section labels",
            "Related destinations can be grouped using short subhead section labels in the navigation drawer",
        ),
        22 => spec_of(
            680.0,
            576.0,
            NONE,
            "Modal navigation drawer with scrim placed behind",
            "Scrim applied behind a modal navigation drawer",
        ),
        23 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "Standard navigation drawers change size to suit the device’s screen",
        ),
        24 => spec_of(
            680.0,
            576.0,
            NONE,
            "Modal navigation drawer on a mobile screen",
            "Use a modal navigation drawer on mobile screens",
        ),
        25 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "Use a navigation rail on tablet screens, or also allow a drawer to open and close via a menu icon",
        ),
        26 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "Use a standard navigation drawer on large and desktop screens",
        ),
        27 => spec_of(
            680.0,
            576.0,
            NONE,
            "Navigation drawer scrolling independently of mail content",
            "When a navigation drawer is scrolled, the body content should remain stationary",
        ),
        28 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "A standard dismissible navigation drawer is opened and closed by tapping the navigation menu icon in the app bar (1), and remains open until the menu icon is tapped again (2)",
        ),
        29 => spec_of(
            1024.0,
            600.0,
            NONE,
            "Permanently visible standard navigation drawer on desktop",
            "A permanently-visible standard navigation drawer on desktop",
        ),
        31 => spec_of(
            692.0,
            305.0,
            NONE,
            "Tapping a navigation drawer destination",
            "Touch: Tap",
        ),
        32 => spec_of(
            692.0,
            305.0,
            NONE,
            "Hovering and clicking a navigation drawer destination",
            "Cursor: Hover, Click",
        ),
        33 => spec_of(
            692.0,
            305.0,
            NONE,
            "Keyboard focus moving between navigation drawer destinations",
            "Focus lands on first navigation item",
        ),
        34 => spec_of(
            692.0,
            305.0,
            NONE,
            "Selecting a focused navigation drawer destination with Space or Enter",
            "The navigation item is selected via Space/Enter",
        ),
        40 => spec_of(
            692.0,
            305.0,
            NONE,
            "Selecting the scrim to close the navigation drawer",
            "Select the scrim to close the navigation drawer",
        ),
        41 => spec_of(
            330.0,
            304.0,
            DO,
            "Navigation drawer with a filled selected icon",
            "Use a filled icon for the selected navigation destination to differentiate from the other destinations",
        ),
        42 => spec_of(
            330.0,
            304.0,
            DONT,
            "Navigation drawer with an outlined selected icon",
            "Avoid keeping the icon style for the selected navigation destination the same as unselected destination's icons. This removes an important visual indicator of which destination is active.",
        ),
        43 => spec_of(
            432.0,
            191.0,
            NONE,
            "Selected and unselected navigation drawer icons",
            "When selected, the icon fills, darkens in light theme (or lightens in dark theme), and is backed by an active indicator shape",
        ),
        44 => spec_of(
            704.0,
            328.0,
            NONE,
            "Navigation drawer accessibility label matching its label text",
            "A navigation drawer’s accessibility label can incorporate its adjacent UI text",
        ),
        _ => spec_of(
            704.0,
            328.0,
            NONE,
            "Navigation drawer accessibility label clarifying its label text",
            "While the visible label text reads Recents, the accessibility label for this destination clarifies its function: Recent images",
        ),
    }
}

pub(super) fn medium_style(size: f64, line: f64) -> TextStyle {
    // The bundled text face is a static 400 instance; the tab face is the
    // matching static 500 instance, so emphasis does not rely on synthesis.
    TextStyle {
        font_family: Some("Example Sans Tabs"),
        font_weight: 500,
        ..type_style(size, line)
    }
}

#[tessera]
fn minor_heading(#[prop(into)] label: String) {
    text()
        .content(label)
        .style(medium_style(16.0, 24.0))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Heading),
            heading_level: Some(4),
            ..Default::default()
        }));
}

#[tessera]
pub(super) fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .style(type_style(14.0, 20.0))
        .color(scheme.on_surface_variant);
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

#[tessera]
fn advice(tone: u8) {
    let (background, ink, asset, label) = match tone {
        DO => (srgb(52, 190, 82), srgb(0, 33, 10), filled::CHECK_SVG, "Do"),
        CAUTION => (
            srgb(255, 207, 36),
            srgb(36, 26, 0),
            filled::PRIORITY_HIGH_SVG,
            "Caution",
        ),
        _ => (
            srgb(214, 46, 40),
            srgb(255, 255, 255),
            filled::CLOSE_SVG,
            "Don’t",
        ),
    };
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(36.0))
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

#[tessera]
pub(super) fn guide(kind: u8) {
    let spec = spec(kind);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            match kind {
                23 => {
                    responsive_figure(0);
                }
                25 => {
                    responsive_figure(2);
                }
                26 => {
                    responsive_figure(3);
                }
                28 => {
                    visibility_examples();
                }
                _ => {
                    guide_stage(kind);
                }
            }
            if spec.tone != NONE {
                advice(spec.tone);
            }
            if !spec.caption.is_empty() {
                gap(Dp(12.0));
                for line in spec.caption.split('\n') {
                    caption(line);
                }
            }
        });
}

#[tessera]
pub(super) fn guide_row(kinds: Vec<u8>) {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, kind) in kinds.iter().copied().enumerate() {
                if index > 0 {
                    spacer().modifier(Modifier::new().width(Dp(20.0)));
                }
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        guide(kind);
                    });
            }
        });
}

#[tessera]
pub(super) fn guide_split(start: Option<RenderSlot>, end: Option<RenderSlot>) {
    let start = start.unwrap_or_else(RenderSlot::empty);
    let end = end.unwrap_or_else(RenderSlot::empty);
    let wide = remember(|| true);
    boxed()
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
                            .modifier(Modifier::new().weight(1.0))
                            .children_shared(start);
                        spacer().modifier(Modifier::new().width(Dp(20.0)));
                        column()
                            .modifier(Modifier::new().weight(2.0))
                            .children_shared(end);
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        start.render();
                        gap(Dp(24.0));
                        end.render();
                    });
            }
        });
}

#[tessera]
fn guide_stage(kind: u8) {
    let spec = spec(kind);
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    drawer_stage(spec.width, spec.height, spec.label.into(), geometry)
        .dark(kind == 2)
        .square_bottom(spec.tone != NONE)
        .background_optional((kind == 43).then(|| srgb(247, 242, 250)))
        .child(move || {
            guide_scene(kind, selected);
        });
}

#[tessera]
fn device(x: f64, y: f64, width: f64, height: f64, child: Option<RenderSlot>) {
    let child = child.unwrap_or_else(RenderSlot::empty);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .required_size(Dp(width), Dp(height))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .border_with_shape(
                    Dp(8.0),
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            boxed()
                .modifier(Modifier::new().offset(Dp(8.0), Dp(8.0)))
                .children_shared(child);
        });
}

#[tessera]
fn guide_scene(kind: u8, selected: State<usize>) {
    match kind {
        0 => {
            drawer_scene(true, selected, 312.0, 84.0, 432.0, 640.0)
                .folders(true)
                .backdrop(true);
        }
        1 | 6 | 10 | 22 | 24 => {
            drawer_scene(true, selected, 124.0, 48.0, 432.0, 640.0)
                .folders(true)
                .backdrop(true);
        }
        2 => {
            drawer_scene(false, selected, 64.0, 64.0, 760.0, 640.0).folders(true);
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(432.0), Dp(476.0))
                        .required_size(Dp(384.0), Dp(80.0))
                        .clip_to_bounds(),
                )
                .children(move || {
                    navigation_bar().content(move |icon_position| {
                        for (index, (label, asset)) in [
                            ("Inbox", outlined::INBOX_SVG),
                            ("Favorites", outlined::FAVORITE_BORDER_SVG),
                            ("Trash", outlined::DELETE_SVG),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            navigation_bar_item()
                                .icon_position(icon_position)
                                .label(label)
                                .selected(index == 0)
                                .icon(move || {
                                    icon().vector(asset).size(Dp(24.0));
                                });
                        }
                    });
                });
        }
        3 => {
            drawer_scene(false, selected, 24.0, 64.0, 520.0, 640.0).folders(true);
        }
        4 => {
            drawer_scene(true, selected, 24.0, 64.0, 520.0, 640.0)
                .folders(true)
                .backdrop(true);
        }
        5 | 29 => {
            device(48.0, 48.0, 976.0, 600.0).child(move || {
                row().children(move || {
                    drawer_content(false, selected)
                        .folders(true)
                        .height(Dp(584.0))
                        .interactive(true);
                    mail_messages(selected, 600.0);
                });
            });
        }
        7 => {
            device(120.0, 48.0, 620.0, 440.0).child(move || {
                row().children(move || {
                    column().children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().size(Dp(80.0), Dp(56.0)))
                            .children(|| {
                                icon().vector(outlined::MENU_SVG).size(Dp(24.0));
                            });
                        small_rail();
                    });
                    mail_messages(selected, 520.0);
                });
            });
            let mut svg = String::new();
            leader(&mut svg, (80.0, 92.0), (152.0, 92.0));
            annotations::lines(Dp(680.0), Dp(443.0), svg);
            drawer_marker(1, 56.0, 76.0);
        }
        8 => {
            for x in [56.0, 528.0] {
                drawer_scene(true, selected, x, 72.0, 440.0, 820.0)
                    .folders(true)
                    .backdrop(true);
            }
            drawer_marker(1, 264.0, 24.0);
            drawer_marker(2, 736.0, 24.0);
            let svg = "<circle cx='460' cy='420' r='24' fill='#ffffff' fill-opacity='.5' stroke='#ffffff' stroke-width='2'/><circle cx='840' cy='420' r='24' fill='#1d1b20' fill-opacity='.24'/><path d='M880,420H720m0,0 16,-10m-16,10 16,10' stroke='#1d1b20' stroke-width='3' fill='none'/>";
            annotations::lines(Dp(1024.0), Dp(868.0), svg.into());
        }
        11 => {
            drawer_scene(false, selected, 40.0, 40.0, 420.0, 700.0).folders(true);
            let mut svg = String::new();
            leader(&mut svg, (24.0, 360.0), (72.0, 360.0));
            annotations::lines(Dp(400.0), Dp(698.0), svg);
            drawer_marker(1, 0.0, 344.0);
        }
        12 => {
            device(40.0, 40.0, 420.0, 700.0).child(move || {
                custom_drawer(12, selected);
            });
        }
        13 => {
            drawer_scene(false, selected, 120.0, 40.0, 640.0, 400.0);
            let mut svg = String::new();
            leader(&mut svg, (360.0, 32.0), (360.0, 152.0));
            annotations::lines(Dp(720.0), Dp(360.0), svg);
            drawer_marker(1, 348.0, 0.0);
        }
        14 => {
            drawer_scene(false, selected, 120.0, 40.0, 640.0, 400.0);
            let mut svg = String::new();
            leader(&mut svg, (100.0, 144.0), (154.0, 144.0));
            leader(&mut svg, (210.0, 32.0), (210.0, 136.0));
            annotations::lines(Dp(720.0), Dp(346.0), svg);
            drawer_marker(1, 76.0, 128.0);
            drawer_marker(2, 198.0, 0.0);
        }
        15 => {
            device(120.0, 40.0, 640.0, 400.0).child(move || {
                custom_drawer(15, selected);
            });
        }
        16..=20 => {
            device(40.0, 40.0, 420.0, 420.0).child(move || {
                custom_drawer(kind, selected);
            });
        }
        21 => {
            device(120.0, -240.0, 640.0, 560.0).child(move || {
                custom_drawer(21, selected);
            });
        }
        27 => {
            device(56.0, 40.0, 680.0, 600.0).child(move || {
                row().children(move || {
                    scroll_drawer(selected);
                    mail_messages(selected, 296.0);
                });
            });
        }
        31 | 32 => {
            a11y_card(
                166.0,
                32.0,
                360.0,
                273.0,
                false,
                if kind == 31 { 0 } else { 1 },
            );
            let svg = if kind == 31 {
                "<circle cx='300' cy='128' r='20' fill='#844596' fill-opacity='.24'/>"
            } else {
                "<path d='M330,176v19l5,-5l4,9l3,-1.5l-4,-8.5h7z' fill='#1d1b20' stroke='#ffffff' stroke-width='1'/>"
            };
            annotations::lines(Dp(692.0), Dp(305.0), svg.into());
        }
        33 | 34 => {
            key_chip("Tab", 186.0, 23.0, 74.0);
            if kind == 33 {
                key_chip("Arrow \u{2193}", 553.0, 23.0, 100.0);
            } else {
                key_chip("Space/Enter", 549.0, 23.0, 108.0);
            }
            a11y_card(24.0, 89.0, 360.0, 216.0, false, 0);
            a11y_card(417.0, 89.0, 360.0, 216.0, false, 0);
            focus_ring(36.0, 157.0, 336.0);
            if kind == 33 {
                focus_ring(429.0, 213.0, 336.0).fill(true);
                drawer_marker(1, 27.0, 149.0);
                drawer_marker(2, 426.0, 207.0);
            } else {
                focus_ring(429.0, 157.0, 336.0).fill(true);
            }
        }
        40 => {
            drawer_scene(true, selected, 38.0, 36.0, 720.0, 400.0).folders(true);
            let svg = "<circle cx='560' cy='204' r='28' fill='#844596' fill-opacity='.3' stroke='#6e2b82' stroke-width='4'/>";
            annotations::lines(Dp(692.0), Dp(305.0), svg.into());
        }
        41 | 42 => {
            a11y_card(33.0, 33.0, 360.0, 280.0, false, 0).outlined_selected(kind == 42);
        }
        43 => {
            icon_states();
        }
        _ => {
            labeling_figure(kind == 44);
        }
    }
}

#[tessera]
fn scroll_drawer(selected: State<usize>) {
    let scroll = remember(ScrollableController::new);
    boxed()
        .modifier(Modifier::new().required_size(Dp(360.0), Dp(584.0)))
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
                .children(move || {
                    drawer_content(false, selected)
                        .folders(true)
                        .height(Dp(820.0))
                        .interactive(true);
                });
        });
}

#[tessera]
fn subhead_row(#[prop(into)] label: String) {
    let style = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .typography
        .title_small;
    boxed()
        .alignment(Alignment::CenterStart)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            text().content(label.clone()).style(style);
        });
}

#[tessera]
fn custom_drawer(variant: u8, selected: State<usize>) {
    navigation_drawer()
        .style(NavigationDrawerStyle::Standard)
        .modifier(Modifier::new().height(Dp(720.0)))
        .content(move || {
            if variant != 21 {
                subhead_row("Mail");
            }
            for index in 0..4usize {
                if variant == 17 && index == 2 {
                    plain_item(true);
                } else if variant == 18 && index == 2 {
                    plain_item(false);
                } else {
                    drawer_item(variant, index, selected);
                }
                if variant == 12 {
                    divider_row();
                }
            }
            if variant == 21 {
                gap(Dp(16.0));
                divider_row();
                subhead_row("Personal Folders");
                for label in ["Friends", "Volunteering", "Work"] {
                    navigation_drawer_item().label(label).icon(|| {
                        icon().vector(outlined::FOLDER_SVG).size(Dp(24.0));
                    });
                }
            }
        });
}

#[tessera]
fn divider_row() {
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(16.0), Dp(4.0)),
        )
        .children(|| {
            horizontal_divider();
        });
}

#[tessera]
fn drawer_item(variant: u8, index: usize, selected: State<usize>) {
    let home = matches!(variant, 41 | 42);
    let label = match (home, index, variant) {
        (true, 0, _) => "Home",
        (_, 0, _) => "Inbox",
        (false, 1, _) => "Outbox",
        (true, 1, _) => "Inbox",
        (_, 2, 16) => "Notifications from fami…",
        (_, 2, _) => "Favorites",
        _ => "Trash",
    };
    let active = selected.get() == index;
    let asset = match (home, index, active && variant != 42) {
        (true, 0, true) => filled::HOME_SVG,
        (true, 0, false) => outlined::HOME_SVG,
        (false, 0, true) => filled::INBOX_SVG,
        (_, 0, _) => outlined::INBOX_SVG,
        (true, 1, _) => outlined::INBOX_SVG,
        (_, 1, _) => outlined::SEND_SVG,
        (_, 2, _) => {
            if variant == 16 {
                outlined::NOTIFICATIONS_SVG
            } else {
                outlined::FAVORITE_BORDER_SVG
            }
        }
        _ => outlined::DELETE_SVG,
    };
    let with_icon = variant != 15 && !(variant == 20 && index >= 2);
    let badge = if index == 0 && !home {
        Some(RenderSlot::new(|| {
            text().content("24");
        }))
    } else {
        None
    };
    if with_icon {
        navigation_drawer_item()
            .label(label)
            .selected(active)
            .badge_optional(badge)
            .on_click(move || selected.set(index))
            .icon(move || {
                icon().vector(asset).size(Dp(24.0));
            });
    } else {
        navigation_drawer_item()
            .label(label)
            .selected(active)
            .badge_optional(badge)
            .on_click(move || selected.set(index));
    }
}

#[tessera]
fn plain_item(two_lines: bool) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let base = theme.typography.label_large;
    let style = if two_lines {
        base
    } else {
        TextStyle {
            font_size: Dp(10.0),
            line_height: Some(Dp(16.0)),
            ..base
        }
    };
    let color = theme.color_scheme.on_surface_variant;
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            icon()
                .vector(outlined::NOTIFICATIONS_SVG)
                .size(Dp(24.0))
                .tint(color);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            column().children(move || {
                if two_lines {
                    text()
                        .content("Notifications from family")
                        .style(style)
                        .color(color);
                    text().content("and friends").style(style).color(color);
                } else {
                    text()
                        .content("Notifications from family and friends")
                        .style(style)
                        .color(color);
                }
            });
        });
}

fn a11y_purple() -> tessera_ui::Color {
    srgb(132, 69, 150)
}

#[tessera]
fn a11y_item(photos: bool, index: usize, selected: usize, outlined_selected: bool) {
    let labels = if photos {
        ["Photos", "Albums", "Recents", "Trash"]
    } else {
        ["Inbox", "Outbox", "Favorites", "Trash"]
    };
    let filled_icon = selected == index && !outlined_selected;
    let asset = match (index, filled_icon) {
        (0, true) => filled::INBOX_SVG,
        (0, false) => outlined::INBOX_SVG,
        (1, true) => filled::SEND_SVG,
        (1, false) => outlined::SEND_SVG,
        (2, true) => filled::FAVORITE_SVG,
        (2, false) => outlined::FAVORITE_BORDER_SVG,
        (_, true) => filled::DELETE_SVG,
        (_, false) => outlined::DELETE_SVG,
    };
    let badge = if index == 1 && !photos {
        Some(RenderSlot::new(|| {
            text().content("100+");
        }))
    } else {
        None
    };
    navigation_drawer_item()
        .label(labels[index])
        .selected(selected == index)
        .badge_optional(badge)
        .icon(move || {
            icon().vector(asset).size(Dp(24.0));
        });
}

#[tessera]
fn a11y_card(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    photos: bool,
    selected: usize,
    outlined_selected: Option<bool>,
) {
    let outlined_selected = outlined_selected.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .required_size(Dp(width), Dp(height))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            navigation_drawer()
                .style(NavigationDrawerStyle::Modal)
                .modifier(Modifier::new().height(Dp(height)))
                .content(move || {
                    subhead_row("Mail");
                    for index in 0..4usize {
                        a11y_item(photos, index, selected, outlined_selected);
                    }
                });
        });
}

#[tessera]
fn focus_ring(x: f64, y: f64, width: f64, fill: Option<bool>) {
    let base = Modifier::new()
        .offset(Dp(x - 4.0), Dp(y - 4.0))
        .required_size(Dp(width + 8.0), Dp(64.0));
    let modifier = if fill.unwrap_or(false) {
        base.background_with_shape(srgb(29, 27, 32).with_alpha(0.1), Shape::CAPSULE)
    } else {
        base
    };
    boxed().modifier(modifier.border_with_shape(Dp(3.0), a11y_purple(), Shape::CAPSULE));
}

#[tessera]
fn key_chip(#[prop(into)] label: String, x: f64, y: f64, width: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .required_size(Dp(width), Dp(40.0))
                .background_with_shape(a11y_purple(), Shape::rounded_rectangle(Dp(4.0))),
        )
        .children(move || {
            text()
                .content(label.clone())
                .style(medium_style(14.0, 20.0))
                .color(tessera_ui::Color::WHITE);
        });
}

#[tessera]
fn icon_states() {
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(48.0), Dp(40.0))
                .required_size(Dp(336.0), Dp(112.0)),
        )
        .children(|| {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(|| {
                    a11y_item(false, 0, 0, false);
                    a11y_item(false, 1, 0, false);
                });
        });
}

#[tessera]
fn labeling_figure(photos: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (row, accessible) = if photos {
        (0.0, "Photos")
    } else {
        (2.0, "Recent Images")
    };
    let row_y = 102.0 + 56.0 * row;
    a11y_card(32.0, 34.0, 360.0, 320.0, true, 0);
    focus_ring(44.0, row_y, 336.0);
    let center = row_y + 28.0;
    annotations::lines(
        Dp(704.0),
        Dp(328.0),
        format!(
            "<path d='M384,{center}H448V162H487' stroke='#844596' stroke-width='3' fill='none'/>"
        ),
    );
    column()
        .modifier(
            Modifier::new()
                .offset(Dp(487.0), Dp(84.0))
                .required_size(Dp(172.0), Dp(151.0))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(8.0)))
                .border_with_shape(Dp(3.0), a11y_purple(), Shape::rounded_rectangle(Dp(8.0)))
                .padding_symmetric(Dp(22.0), Dp(18.0)),
        )
        .children(move || {
            for (index, (name, value)) in [("Label", accessible), ("Role", "Tab")]
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    gap(Dp(8.0));
                }
                text()
                    .content(name)
                    .style(type_style(16.0, 24.0))
                    .color(scheme.on_surface);
                text()
                    .content(value)
                    .style(medium_style(16.0, 24.0))
                    .color(a11y_purple());
            }
        });
}

#[tessera]
pub(super) fn usage(open: State<bool>, selected: State<usize>) {
    guide_split()
        .start(|| {
            prose("Navigation drawers provide access to destinations and app functionality, such as switching accounts. They can either be permanently on-screen or opened and closed by a navigation menu icon. One navigation destination is always active.");
            gap(Dp(16.0));
            prose("Navigation drawers are recommended for:");
            gap(Dp(12.0));
            bullets(vec![
                "Apps with 5 or more top-level destinations".into(),
                "Apps with 2 or more levels of navigation hierarchy".into(),
                "Quick navigation between unrelated destinations".into(),
                "Replacing the navigation rail or navigation bar on large screens".into(),
            ]);
        })
        .end(|| {
            guide(1);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("Avoid using a navigation drawer with other primary navigation components, such as a navigation bar.");
            gap(Dp(16.0));
            prose("Instead, choose a single navigation component based on product requirements and breakpoints:");
            gap(Dp(12.0));
            bullets(vec![
                "Navigation bars for compact breakpoints".into(),
                "Navigation rails for medium and expanded breakpoints".into(),
                "Standard navigation drawers for expanded, large and extra-large breakpoints".into(),
            ]);
        })
        .end(|| {
            guide(2);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("There are two variants of navigation drawers:");
            gap(Dp(16.0));
            legend_entries(vec![
                (1, "Standard navigation drawer".into()),
                (2, "Modal navigation drawer".into()),
            ]);
        })
        .end(|| {
            guide_row(vec![3, 4]);
        });
    gap(Dp(48.0));
    subheading("Standard navigation drawer".into());
    gap(Dp(24.0));
    guide_split()
        .start(|| {
            prose("Standard navigation drawers provide access to drawer destinations and app content for layouts in expanded, large, and extra-large breakpoints.");
            gap(Dp(16.0));
            prose("Standard drawers can be permanently visible (best for frequently switching destinations) or opened and closed by tapping a menu icon (best for focusing more on screen content).");
            gap(Dp(16.0));
            prose("In medium and compact breakpoints, use modal drawers instead.");
        })
        .end(|| {
            guide(5);
        });
    gap(Dp(48.0));
    subheading("Modal navigation drawer".into());
    gap(Dp(24.0));
    guide_split()
        .start(|| {
            prose("Modal navigation drawers use a scrim to block interaction with the rest of an app’s content, and don’t affect the screen’s layout grid.");
            gap(Dp(16.0));
            prose("Modal navigation drawers can be used in any breakpoint, but are primarily used in compact and medium sizes where space is limited or prioritized for app content.");
            gap(Dp(16.0));
            prose("They can be swapped with standard drawers on expanded, large, and extra-large breakpoints.");
        })
        .end(|| {
            guide(6);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("Modal navigation drawers are always opened by an action outside of the drawer, such as clicking a navigation menu icon in a navigation rail.");
            gap(Dp(16.0));
            prose("Modal drawers can be dismissed by:");
            gap(Dp(12.0));
            bullets(vec![
                "Selecting a drawer item".into(),
                "Tapping the scrim".into(),
                "Swiping toward the drawer’s anchoring edge (for example, swiping right-to-left for a left-aligned navigation drawer)".into(),
            ]);
        })
        .end(|| {
            guide(7);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("Modal drawers can be dismissed by tapping the scrim or swiping the drawer toward its anchoring screen edge.");
        })
        .end(|| {
            guide(8);
        });
    gap(Dp(32.0));
    launch(open, selected);
}

#[tessera]
pub(super) fn anatomy() {
    prose(
        "Navigation drawers are essentially a list contained within a side sheet. They can also include headers, subheads, and dividers to organize longer lists.",
    );
    gap(Dp(32.0));
    drawer_figure(true, 4);
    gap(Dp(12.0));
    caption(
        "Navigation drawers can include headers, subheads, and dividers to organize longer lists",
    );
    gap(Dp(16.0));
    legend_entries(vec![
        (1, "Active Indicator".into()),
        (2, "Icon".into()),
        (3, "Label".into()),
        (4, "Badge label".into()),
        (5, "Sheet".into()),
        (6, "Divider".into()),
        (7, "Section label (optional)".into()),
        (8, "Scrim".into()),
    ]);
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Sheet");
            gap(Dp(8.0));
            prose("A sheet holds all navigation drawer elements. Side sheets are used as the container for standard and modal navigation drawers.");
            gap(Dp(16.0));
            prose("Navigation drawers that open from the side are always placed on the start edge of the screen, on the left for left-to-right (LTR) languages, and on the right for right-to-left (RTL) languages.");
        })
        .end(|| {
            guide(10);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Divider (optional)");
            gap(Dp(8.0));
            prose("Dividers can be used to separate groups of destinations within the navigation drawer.");
        })
        .end(|| {
            guide_row(vec![11, 12]);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Active indicator");
            gap(Dp(8.0));
            prose("The active indicator is a background shape communicating which destination of the navigation drawer is currently being displayed.");
        })
        .end(|| {
            guide(13);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Label text and icons");
            gap(Dp(8.0));
            prose("Destinations in a navigation drawer take the form of actionable list items. Each item describes its destination using label text and an optional icon.");
        })
        .end(|| {
            guide(14);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose(
                "Label text should be clear and short enough that it isn’t cut off by the sheet.",
            );
        })
        .end(|| {
            guide(15);
        });
    gap(Dp(48.0));
    guide_row(vec![16, 17, 18]);
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("Icons can supplement labels as indicators of a destination. When used, they should always be placed before text. Other app components and content should reference these icons.");
        })
        .end(|| {
            guide_row(vec![19, 20]);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Section label (optional)");
            gap(Dp(8.0));
            prose("Short subhead section labels can help group related destinations in the navigation drawer.");
        })
        .end(|| {
            guide(21);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Scrim (modal only)");
            gap(Dp(8.0));
            prose("Modal navigation drawers use a scrim to block interaction with the rest of the app. The scrim is placed directly behind the drawer’s sheet and can be tapped or clicked to dismiss the drawer.");
        })
        .end(|| {
            guide(22);
        });
}

#[tessera]
pub(super) fn responsive_layout() {
    guide_split()
        .start(|| {
            prose("A product’s navigation component should change to suit the breakpoint and form factor of the screen.");
            gap(Dp(16.0));
            prose("Modal navigation drawers can be used at any breakpoint but are most common in compact and medium breakpoints.");
            gap(Dp(16.0));
            prose("Standard navigation drawers are best for expanded, large, and extra-large breakpoints.");
            gap(Dp(16.0));
            prose("Use a transition when swapping components. For example, when switching from a portrait to landscape layout, the navigation rail should transform into a navigation drawer.");
        })
        .end(|| {
            guide(23);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Compact breakpoint");
            gap(Dp(8.0));
            prose("Use modal navigation drawers in compact breakpoints. Or swap the drawer for a navigation bar.");
            gap(Dp(16.0));
            prose("On web, when the screen size is smaller than 320 CSS pixels, swap the navigation drawer for a navigation bar to ensure accessibility.");
        })
        .end(|| {
            guide(24);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Medium & expanded breakpoints");
            gap(Dp(8.0));
            prose("Use a modal navigation drawer alone or with a navigation rail on medium and expanded breakpoints.");
            gap(Dp(16.0));
            prose("When a navigation rail and modal navigation drawer are used together, the drawer can repeat destinations in the navigation rail as long as the drawer offers enough visual separation between levels of the navigation hierarchy.");
            gap(Dp(16.0));
            prose("A standard navigation drawer can be used in single pane layouts in expanded breakpoints.");
        })
        .end(|| {
            guide(25);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Large and extra-large breakpoints");
            gap(Dp(8.0));
            prose("For web experiences on laptop and desktop devices, use either a standard navigation drawer, or a navigation rail that transitions into a modal navigation drawer.");
        })
        .end(|| {
            guide(26);
        });
}

#[tessera]
pub(super) fn behavior(open: State<bool>, selected: State<usize>) {
    guide_split()
        .start(|| {
            minor_heading("Scrolling");
            gap(Dp(8.0));
            prose("Navigation drawers can be vertically scrolled, independent of the rest of the screen’s content and UI. If the list of navigation destinations is longer than the height of the drawer, the drawer’s contents can be scrolled within the drawer.");
        })
        .end(|| {
            guide(27);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Visibility");
            gap(Dp(8.0));
            prose("Dismissible standard drawers can be used for layouts that prioritize content (such as a photo gallery) or for apps where users are unlikely to switch destinations often. They should use a visible navigation menu icon to open and close the drawer.");
        })
        .end(|| {
            guide(28);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("Permanently visible standard drawers allow quick navigation between unrelated destinations. They can’t be closed or dismissed by the user.");
        })
        .end(|| {
            guide(29);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Appearing");
            gap(Dp(8.0));
            prose("When a navigation drawer animates on screen, it uses an enter and exit transition pattern.");
        })
        .end(move || {
            launch(open, selected);
            gap(Dp(12.0));
            caption("A navigation drawer animating on screen");
        });
}

#[tessera]
pub(super) fn use_cases(open: State<bool>, selected: State<usize>) {
    prose("Users should be able to:");
    gap(Dp(12.0));
    bullets(vec![
        "Move between navigation destinations with assistive technology".into(),
        "Select a particular navigation destination from a set".into(),
        "Get appropriate feedback based on input type".into(),
    ]);
    gap(Dp(16.0));
    launch(open, selected);
}

#[tessera]
pub(super) fn interaction() {
    guide_split()
        .start(|| {
            minor_heading("Touch");
            gap(Dp(8.0));
            bullets(vec![
                "When a navigation item is tapped, the active indicator appears in place, providing feedback to the user that it is selected".into(),
                "A touch ripple passes through the indicator".into(),
                "The icon switches from outlined to filled".into(),
                "The icon changes color, becoming darker".into(),
            ]);
        })
        .end(|| {
            guide(31);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("Cursor");
            gap(Dp(8.0));
            bullets(vec![
                "When hovered, the hover indicator appears providing a visual cue that the destination is interactive".into(),
                "When clicked, a ripple passes through the indicator".into(),
                "The icon switches from outlined to filled".into(),
                "The icon changes color, becoming darker in light theme and lighter in dark theme, to increase the contrast".into(),
            ]);
        })
        .end(|| {
            guide(32);
        });
}

#[tessera]
pub(super) fn initial_focus(open: State<bool>, selected: State<usize>) {
    guide_split()
        .start(|| {
            prose("Initial focus lands directly on the first navigation item, since that is the first interactive element of the component.");
        })
        .end(|| {
            guide(33);
        });
    gap(Dp(24.0));
    launch(open, selected);
}

#[tessera]
pub(super) fn closing(open: State<bool>, selected: State<usize>) {
    guide_split()
        .start(|| {
            prose("The modal navigation drawer can be dismissed by selecting the scrim that covers the rest of the screen.");
        })
        .end(|| {
            guide(40);
        });
    gap(Dp(24.0));
    launch(open, selected);
}

#[tessera]
pub(super) fn visual_indicators() {
    guide_split()
        .start(|| {
            prose("Icons are the primary focus of the navigation and such give the dominant cue of its state. Use a filled icon for the selected destination to differentiate from the outlined icons of non-selected destinations.");
        })
        .end(|| {
            guide(34);
        });
    gap(Dp(48.0));
    guide_row(vec![41, 42]);
    gap(Dp(48.0));
    guide_split().end(|| {
        guide(43);
    });
}

#[tessera]
pub(super) fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Tab".into(),
                "Focus lands on the first navigation destination".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Selects the focused navigation destination, and focus moves to the newly opened section (if applicable)".into(),
            ],
            vec![
                "Arrow".into(),
                "Navigate between destinations within the navigation drawer".into(),
            ],
        ],
    );
}

#[tessera]
pub(super) fn labeling() {
    guide_split()
        .start(|| {
            prose("The accessibility label for a navigation item is typically the same as the destination name.");
            gap(Dp(16.0));
            prose("If the UI text is correctly linked, assistive tech (such as a screenreader) will read the UI text followed by the component’s role.");
            gap(Dp(16.0));
            prose("For Android Views (MDC-Android), a more descriptive accessibility label is not available to be set and the role is not announced.");
        })
        .end(|| {
            guide(44);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("When the visible UI text is ambiguous, accessibility labels need to be more descriptive. For example, a navigation destination visibly labeled Recents would benefit from additional information in its accessibility label to clarify the destination’s intent.");
        })
        .end(|| {
            guide(45);
        });
}
