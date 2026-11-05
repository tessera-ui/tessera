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

use crate::res::I18N;

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
            "ui-navigation-drawer-with-4-primary-destinations",
            "",
        ),
        1 => spec_of(
            680.0,
            576.0,
            DO,
            "ui-navigation-drawer-with-multiple-destinations-in-a-mail-app",
            "ui-use-a-navigation-drawer-for-5-or-more-primary-destinations-or-more-than-1-level-of-navigat",
        ),
        2 => spec_of(
            680.0,
            576.0,
            CAUTION,
            "ui-standard-navigation-drawer-and-navigation-bar-used-together",
            "ui-avoid-using-two-navigation-components-on-the-same-screen",
        ),
        3 => spec_of(
            460.0,
            600.0,
            NONE,
            "ui-standard-navigation-drawer-with-destinations-in-mail-app",
            "ui-standard-navigation-drawer",
        ),
        4 => spec_of(
            460.0,
            600.0,
            NONE,
            "ui-modal-navigation-drawer-with-destinations-and-scrim",
            "ui-modal-navigation-drawer",
        ),
        5 => spec_of(
            1024.0,
            600.0,
            NONE,
            "ui-standard-navigation-drawer-next-to-app-content",
            "ui-standard-navigation-drawer-providing-access-to-drawer-destinations-next-to-app-content",
        ),
        6 => spec_of(
            680.0,
            576.0,
            NONE,
            "ui-modal-navigation-drawer-with-1-active-destination-and-scrim",
            "ui-modal-navigation-drawer-using-a-scrim-to-block-interaction-with-the-rest-of-an-app-s-conte",
        ),
        7 => spec_of(
            680.0,
            443.0,
            NONE,
            "ui-navigation-menu-icon-in-a-navigation-rail",
            "ui-a-modal-drawer-opened-by-an-action-such-as-clicking-a-navigation-menu-icon-1",
        ),
        8 => spec_of(
            1024.0,
            868.0,
            NONE,
            "ui-dismissing-a-modal-drawer-by-tapping-the-scrim-or-swiping",
            "ui-1-dismiss-by-tapping-the-scrim-2-dismiss-by-swiping-the-drawer",
        ),
        10 => spec_of(
            680.0,
            576.0,
            DO,
            "ui-modal-navigation-drawer-opening-from-the-left-side-of-the-screen",
            "ui-a-navigation-drawer-opens-from-the-left-side-of-the-screen-for-left-to-right-languages",
        ),
        11 => spec_of(
            400.0,
            698.0,
            DO,
            "ui-full-width-divider-separating-a-group-of-destinations",
            "ui-use-full-width-dividers-1-to-separate-groups-of-destinations",
        ),
        12 => spec_of(
            400.0,
            698.0,
            DONT,
            "ui-dividers-separating-individual-destinations",
            "ui-don-t-use-dividers-to-separate-individual-destinations",
        ),
        13 => spec_of(
            720.0,
            360.0,
            NONE,
            "ui-active-indicator-in-a-navigation-drawer",
            "ui-the-active-indicator-1-is-a-background-shape-communicating-which-destination-of-the-naviga",
        ),
        14 => spec_of(
            720.0,
            346.0,
            NONE,
            "ui-icon-and-label-text-of-a-navigation-drawer-destination",
            "ui-actionable-list-items-in-a-navigation-drawer-describe-each-destination-using-1-an-optional",
        ),
        15 => spec_of(
            720.0,
            374.0,
            NONE,
            "ui-navigation-drawer-using-only-label-text",
            "ui-navigation-drawers-can-use-text-labels-without-icons",
        ),
        16 => spec_of(
            400.0,
            396.0,
            DO,
            "ui-navigation-drawer-with-a-truncated-text-label",
            "ui-keep-text-labels-concise-but-truncate-them-if-they-extend-beyond-the-container-width",
        ),
        17 => spec_of(
            400.0,
            396.0,
            DONT,
            "ui-navigation-drawer-with-wrapped-label-text",
            "ui-don-t-wrap-label-text",
        ),
        18 => spec_of(
            400.0,
            396.0,
            DONT,
            "ui-navigation-drawer-with-a-smaller-text-label",
            "ui-don-t-shrink-text-size-in-order-to-fit-a-text-label-on-a-single-line",
        ),
        19 => spec_of(
            400.0,
            396.0,
            DO,
            "ui-navigation-drawer-with-recognizable-icons",
            "ui-use-recognizable-icons-when-conventions-exist",
        ),
        20 => spec_of(
            400.0,
            396.0,
            DONT,
            "ui-navigation-drawer-with-icons-on-only-some-destinations",
            "ui-don-t-apply-icons-to-some-destinations-and-not-others-icons-should-be-used-for-all-destina",
        ),
        21 => spec_of(
            720.0,
            288.0,
            NONE,
            "ui-navigation-drawer-subhead-section-labels",
            "ui-related-destinations-can-be-grouped-using-short-subhead-section-labels-in-the-navigation-d",
        ),
        22 => spec_of(
            680.0,
            576.0,
            NONE,
            "ui-modal-navigation-drawer-with-scrim-placed-behind",
            "ui-scrim-applied-behind-a-modal-navigation-drawer",
        ),
        23 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "ui-standard-navigation-drawers-change-size-to-suit-the-device-s-screen",
        ),
        24 => spec_of(
            680.0,
            576.0,
            NONE,
            "ui-modal-navigation-drawer-on-a-mobile-screen",
            "ui-use-a-modal-navigation-drawer-on-mobile-screens",
        ),
        25 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "ui-use-a-navigation-rail-on-tablet-screens-or-also-allow-a-drawer-to-open-and-close-via-a-men",
        ),
        26 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "ui-use-a-standard-navigation-drawer-on-large-and-desktop-screens",
        ),
        27 => spec_of(
            680.0,
            576.0,
            NONE,
            "ui-navigation-drawer-scrolling-independently-of-mail-content",
            "ui-when-a-navigation-drawer-is-scrolled-the-body-content-should-remain-stationary",
        ),
        28 => spec_of(
            0.0,
            0.0,
            NONE,
            "",
            "ui-a-standard-dismissible-navigation-drawer-is-opened-and-closed-by-tapping-the-navigation-me",
        ),
        29 => spec_of(
            1024.0,
            600.0,
            NONE,
            "ui-permanently-visible-standard-navigation-drawer-on-desktop",
            "ui-a-permanently-visible-standard-navigation-drawer-on-desktop",
        ),
        31 => spec_of(
            692.0,
            305.0,
            NONE,
            "ui-tapping-a-navigation-drawer-destination",
            "ui-touch-tap",
        ),
        32 => spec_of(
            692.0,
            305.0,
            NONE,
            "ui-hovering-and-clicking-a-navigation-drawer-destination",
            "ui-cursor-hover-click",
        ),
        33 => spec_of(
            692.0,
            305.0,
            NONE,
            "ui-keyboard-focus-moving-between-navigation-drawer-destinations",
            "ui-focus-lands-on-first-navigation-item",
        ),
        34 => spec_of(
            692.0,
            305.0,
            NONE,
            "ui-selecting-a-focused-navigation-drawer-destination-with-space-or-enter",
            "ui-the-navigation-item-is-selected-via-space-enter",
        ),
        40 => spec_of(
            692.0,
            305.0,
            NONE,
            "ui-selecting-the-scrim-to-close-the-navigation-drawer",
            "ui-select-the-scrim-to-close-the-navigation-drawer",
        ),
        41 => spec_of(
            330.0,
            304.0,
            DO,
            "ui-navigation-drawer-with-a-filled-selected-icon",
            "ui-use-a-filled-icon-for-the-selected-navigation-destination-to-differentiate-from-the-other",
        ),
        42 => spec_of(
            330.0,
            304.0,
            DONT,
            "ui-navigation-drawer-with-an-outlined-selected-icon",
            "ui-avoid-keeping-the-icon-style-for-the-selected-navigation-destination-the-same-as-unselecte",
        ),
        43 => spec_of(
            432.0,
            191.0,
            NONE,
            "ui-selected-and-unselected-navigation-drawer-icons",
            "ui-when-selected-the-icon-fills-darkens-in-light-theme-or-lightens-in-dark-theme-and-is-backe",
        ),
        44 => spec_of(
            704.0,
            328.0,
            NONE,
            "ui-navigation-drawer-accessibility-label-matching-its-label-text",
            "ui-a-navigation-drawer-s-accessibility-label-can-incorporate-its-adjacent-ui-text",
        ),
        _ => spec_of(
            704.0,
            328.0,
            NONE,
            "ui-navigation-drawer-accessibility-label-clarifying-its-label-text",
            "ui-while-the-visible-label-text-reads-recents-the-accessibility-label-for-this-destination-cl",
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
        .content(I18N.text(label))
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
        .content(I18N.text(value))
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
            "ui-caution",
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
                .content(I18N.text(label))
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
                            ("ui-inbox", outlined::INBOX_SVG),
                            ("ui-favorites", outlined::FAVORITE_BORDER_SVG),
                            ("ui-trash", outlined::DELETE_SVG),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            navigation_bar_item()
                                .icon_position(icon_position)
                                .label(I18N.text(label))
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
            key_chip("ui-tab", 186.0, 23.0, 74.0);
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
            text().content(I18N.text(&(label))).style(style);
        });
}

#[tessera]
fn custom_drawer(variant: u8, selected: State<usize>) {
    navigation_drawer()
        .style(NavigationDrawerStyle::Standard)
        .modifier(Modifier::new().height(Dp(720.0)))
        .content(move || {
            if variant != 21 {
                subhead_row("ui-mail");
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
                subhead_row("ui-personal-folders-2672");
                for label in ["ui-friends", "ui-volunteering", "ui-work"] {
                    navigation_drawer_item().label(I18N.text(label)).icon(|| {
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
        (true, 0, _) => "ui-home",
        (_, 0, _) => "ui-inbox",
        (false, 1, _) => "ui-outbox",
        (true, 1, _) => "ui-inbox",
        (_, 2, 16) => "ui-notifications-from-fami",
        (_, 2, _) => "ui-favorites",
        _ => "ui-trash",
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
        Some(RenderSlot::new(move || {
            text().content("24");
        }))
    } else {
        None
    };
    if with_icon {
        navigation_drawer_item()
            .label(I18N.text(label))
            .selected(active)
            .badge_optional(badge)
            .on_click(move || selected.set(index))
            .icon(move || {
                icon().vector(asset).size(Dp(24.0));
            });
    } else {
        navigation_drawer_item()
            .label(I18N.text(label))
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
                        .content(I18N.text("ui-notifications-from-family"))
                        .style(style)
                        .color(color);
                    text()
                        .content(I18N.text("ui-and-friends"))
                        .style(style)
                        .color(color);
                } else {
                    text()
                        .content(I18N.text("ui-notifications-from-family-and-friends"))
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
        ["ui-photos", "ui-albums", "ui-recents", "ui-trash"]
    } else {
        ["ui-inbox", "ui-outbox", "ui-favorites", "ui-trash"]
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
        Some(RenderSlot::new(move || {
            text().content("100+");
        }))
    } else {
        None
    };
    navigation_drawer_item()
        .label(I18N.text(labels[index]))
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
                    subhead_row("ui-mail");
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
                .content(I18N.text(&(label)))
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
        (0.0, "ui-photos")
    } else {
        (2.0, "ui-recent-images")
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
            for (index, (name, value)) in [("ui-label", accessible), ("ui-role", "ui-tab")]
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    gap(Dp(8.0));
                }
                text()
                    .content(I18N.text(name))
                    .style(type_style(16.0, 24.0))
                    .color(scheme.on_surface);
                text()
                    .content(I18N.text(value))
                    .style(medium_style(16.0, 24.0))
                    .color(a11y_purple());
            }
        });
}

#[tessera]
pub(super) fn usage(open: State<bool>, selected: State<usize>) {
    guide_split()
        .start(|| {
            prose("ui-navigation-drawers-provide-access-to-destinations-and-app-functionality-such-as-switching");
            gap(Dp(16.0));
            prose("ui-navigation-drawers-are-recommended-for");
            gap(Dp(12.0));
            bullets(vec![
                "ui-apps-with-5-or-more-top-level-destinations".into(),
                "ui-apps-with-2-or-more-levels-of-navigation-hierarchy".into(),
                "ui-quick-navigation-between-unrelated-destinations".into(),
                "ui-replacing-the-navigation-rail-or-navigation-bar-on-large-screens".into(),
            ]);
        })
        .end(|| {
            guide(1);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("ui-avoid-using-a-navigation-drawer-with-other-primary-navigation-components-such-as-a-navigat");
            gap(Dp(16.0));
            prose("ui-instead-choose-a-single-navigation-component-based-on-product-requirements-and-breakpoints");
            gap(Dp(12.0));
            bullets(vec![
                "ui-navigation-bars-for-compact-breakpoints".into(),
                "ui-navigation-rails-for-medium-and-expanded-breakpoints".into(),
                "ui-standard-navigation-drawers-for-expanded-large-and-extra-large-breakpoints".into(),
            ]);
        })
        .end(|| {
            guide(2);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("ui-there-are-two-variants-of-navigation-drawers");
            gap(Dp(16.0));
            legend_entries(vec![
                (1, "ui-standard-navigation-drawer".into()),
                (2, "ui-modal-navigation-drawer".into()),
            ]);
        })
        .end(|| {
            guide_row(vec![3, 4]);
        });
    gap(Dp(48.0));
    subheading("ui-standard-navigation-drawer".into());
    gap(Dp(24.0));
    guide_split()
        .start(|| {
            prose("ui-standard-navigation-drawers-provide-access-to-drawer-destinations-and-app-content-for-layo");
            gap(Dp(16.0));
            prose("ui-standard-drawers-can-be-permanently-visible-best-for-frequently-switching-destinations-or");
            gap(Dp(16.0));
            prose("ui-in-medium-and-compact-breakpoints-use-modal-drawers-instead");
        })
        .end(|| {
            guide(5);
        });
    gap(Dp(48.0));
    subheading("ui-modal-navigation-drawer".into());
    gap(Dp(24.0));
    guide_split()
        .start(|| {
            prose("ui-modal-navigation-drawers-use-a-scrim-to-block-interaction-with-the-rest-of-an-app-s-conten");
            gap(Dp(16.0));
            prose("ui-modal-navigation-drawers-can-be-used-in-any-breakpoint-but-are-primarily-used-in-compact-a");
            gap(Dp(16.0));
            prose("ui-they-can-be-swapped-with-standard-drawers-on-expanded-large-and-extra-large-breakpoints");
        })
        .end(|| {
            guide(6);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("ui-modal-navigation-drawers-are-always-opened-by-an-action-outside-of-the-drawer-such-as-clic");
            gap(Dp(16.0));
            prose("ui-modal-drawers-can-be-dismissed-by");
            gap(Dp(12.0));
            bullets(vec![
                "ui-selecting-a-drawer-item".into(),
                "ui-tapping-the-scrim".into(),
                "ui-swiping-toward-the-drawer-s-anchoring-edge-for-example-swiping-right-to-left-for-a-left-al".into(),
            ]);
        })
        .end(|| {
            guide(7);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("ui-modal-drawers-can-be-dismissed-by-tapping-the-scrim-or-swiping-the-drawer-toward-its-ancho");
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
        "ui-navigation-drawers-are-essentially-a-list-contained-within-a-side-sheet-they-can-also-incl",
    );
    gap(Dp(32.0));
    drawer_figure(true, 4);
    gap(Dp(12.0));
    caption(
        "ui-navigation-drawers-can-include-headers-subheads-and-dividers-to-organize-longer-lists",
    );
    gap(Dp(16.0));
    legend_entries(vec![
        (1, "ui-active-indicator-2814".into()),
        (2, "ui-icon".into()),
        (3, "ui-label".into()),
        (4, "ui-badge-label".into()),
        (5, "ui-sheet".into()),
        (6, "ui-divider".into()),
        (7, "ui-section-label-optional".into()),
        (8, "Scrim".into()),
    ]);
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-sheet");
            gap(Dp(8.0));
            prose("ui-a-sheet-holds-all-navigation-drawer-elements-side-sheets-are-used-as-the-container-for-sta");
            gap(Dp(16.0));
            prose("ui-navigation-drawers-that-open-from-the-side-are-always-placed-on-the-start-edge-of-the-scre");
        })
        .end(|| {
            guide(10);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-divider-optional");
            gap(Dp(8.0));
            prose("ui-dividers-can-be-used-to-separate-groups-of-destinations-within-the-navigation-drawer");
        })
        .end(|| {
            guide_row(vec![11, 12]);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-active-indicator");
            gap(Dp(8.0));
            prose("ui-the-active-indicator-is-a-background-shape-communicating-which-destination-of-the-navigati");
        })
        .end(|| {
            guide(13);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-label-text-and-icons");
            gap(Dp(8.0));
            prose("ui-destinations-in-a-navigation-drawer-take-the-form-of-actionable-list-items-each-item-descr");
        })
        .end(|| {
            guide(14);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose(
                "ui-label-text-should-be-clear-and-short-enough-that-it-isn-t-cut-off-by-the-sheet",
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
            prose("ui-icons-can-supplement-labels-as-indicators-of-a-destination-when-used-they-should-always-be");
        })
        .end(|| {
            guide_row(vec![19, 20]);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-section-label-optional");
            gap(Dp(8.0));
            prose("ui-short-subhead-section-labels-can-help-group-related-destinations-in-the-navigation-drawer");
        })
        .end(|| {
            guide(21);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-scrim-modal-only");
            gap(Dp(8.0));
            prose("ui-modal-navigation-drawers-use-a-scrim-to-block-interaction-with-the-rest-of-the-app-the-scr");
        })
        .end(|| {
            guide(22);
        });
}

#[tessera]
pub(super) fn responsive_layout() {
    guide_split()
        .start(|| {
            prose("ui-a-product-s-navigation-component-should-change-to-suit-the-breakpoint-and-form-factor-of-t");
            gap(Dp(16.0));
            prose("ui-modal-navigation-drawers-can-be-used-at-any-breakpoint-but-are-most-common-in-compact-and");
            gap(Dp(16.0));
            prose("ui-standard-navigation-drawers-are-best-for-expanded-large-and-extra-large-breakpoints");
            gap(Dp(16.0));
            prose("ui-use-a-transition-when-swapping-components-for-example-when-switching-from-a-portrait-to-la");
        })
        .end(|| {
            guide(23);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-compact-breakpoint");
            gap(Dp(8.0));
            prose("ui-use-modal-navigation-drawers-in-compact-breakpoints-or-swap-the-drawer-for-a-navigation-ba");
            gap(Dp(16.0));
            prose("ui-on-web-when-the-screen-size-is-smaller-than-320-css-pixels-swap-the-navigation-drawer-for");
        })
        .end(|| {
            guide(24);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-medium-expanded-breakpoints");
            gap(Dp(8.0));
            prose("ui-use-a-modal-navigation-drawer-alone-or-with-a-navigation-rail-on-medium-and-expanded-break");
            gap(Dp(16.0));
            prose("ui-when-a-navigation-rail-and-modal-navigation-drawer-are-used-together-the-drawer-can-repeat");
            gap(Dp(16.0));
            prose("ui-a-standard-navigation-drawer-can-be-used-in-single-pane-layouts-in-expanded-breakpoints");
        })
        .end(|| {
            guide(25);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-large-and-extra-large-breakpoints");
            gap(Dp(8.0));
            prose("ui-for-web-experiences-on-laptop-and-desktop-devices-use-either-a-standard-navigation-drawer");
        })
        .end(|| {
            guide(26);
        });
}

#[tessera]
pub(super) fn behavior(open: State<bool>, selected: State<usize>) {
    guide_split()
        .start(|| {
            minor_heading("ui-scrolling");
            gap(Dp(8.0));
            prose("ui-navigation-drawers-can-be-vertically-scrolled-independent-of-the-rest-of-the-screen-s-cont");
        })
        .end(|| {
            guide(27);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-visibility");
            gap(Dp(8.0));
            prose("ui-dismissible-standard-drawers-can-be-used-for-layouts-that-prioritize-content-such-as-a-pho");
        })
        .end(|| {
            guide(28);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("ui-permanently-visible-standard-drawers-allow-quick-navigation-between-unrelated-destinations");
        })
        .end(|| {
            guide(29);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-appearing");
            gap(Dp(8.0));
            prose("ui-when-a-navigation-drawer-animates-on-screen-it-uses-an-enter-and-exit-transition-pattern");
        })
        .end(move || {
            launch(open, selected);
            gap(Dp(12.0));
            caption("ui-a-navigation-drawer-animating-on-screen");
        });
}

#[tessera]
pub(super) fn use_cases(open: State<bool>, selected: State<usize>) {
    prose("ui-users-should-be-able-to");
    gap(Dp(12.0));
    bullets(vec![
        "ui-move-between-navigation-destinations-with-assistive-technology".into(),
        "ui-select-a-particular-navigation-destination-from-a-set".into(),
        "ui-get-appropriate-feedback-based-on-input-type".into(),
    ]);
    gap(Dp(16.0));
    launch(open, selected);
}

#[tessera]
pub(super) fn interaction() {
    guide_split()
        .start(|| {
            minor_heading("ui-touch");
            gap(Dp(8.0));
            bullets(vec![
                "ui-when-a-navigation-item-is-tapped-the-active-indicator-appears-in-place-providing-feedback".into(),
                "ui-a-touch-ripple-passes-through-the-indicator".into(),
                "ui-the-icon-switches-from-outlined-to-filled".into(),
                "ui-the-icon-changes-color-becoming-darker".into(),
            ]);
        })
        .end(|| {
            guide(31);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            minor_heading("ui-cursor");
            gap(Dp(8.0));
            bullets(vec![
                "ui-when-hovered-the-hover-indicator-appears-providing-a-visual-cue-that-the-destination-is-in".into(),
                "ui-when-clicked-a-ripple-passes-through-the-indicator".into(),
                "ui-the-icon-switches-from-outlined-to-filled".into(),
                "ui-the-icon-changes-color-becoming-darker-in-light-theme-and-lighter-in-dark-theme-to-increas".into(),
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
            prose("ui-initial-focus-lands-directly-on-the-first-navigation-item-since-that-is-the-first-interact");
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
            prose("ui-the-modal-navigation-drawer-can-be-dismissed-by-selecting-the-scrim-that-covers-the-rest-o");
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
            prose("ui-icons-are-the-primary-focus-of-the-navigation-and-such-give-the-dominant-cue-of-its-state");
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
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-focus-lands-on-the-first-navigation-destination".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-selects-the-focused-navigation-destination-and-focus-moves-to-the-newly-opened-section-if".into(),
            ],
            vec![
                "ui-arrow".into(),
                "ui-navigate-between-destinations-within-the-navigation-drawer".into(),
            ],
        ],
    );
}

#[tessera]
pub(super) fn labeling() {
    guide_split()
        .start(|| {
            prose("ui-the-accessibility-label-for-a-navigation-item-is-typically-the-same-as-the-destination-nam");
            gap(Dp(16.0));
            prose("ui-if-the-ui-text-is-correctly-linked-assistive-tech-such-as-a-screenreader-will-read-the-ui");
            gap(Dp(16.0));
            prose("ui-for-android-views-mdc-android-a-more-descriptive-accessibility-label-is-not-available-to-b");
        })
        .end(|| {
            guide(44);
        });
    gap(Dp(48.0));
    guide_split()
        .start(|| {
            prose("ui-when-the-visible-ui-text-is-ambiguous-accessibility-labels-need-to-be-more-descriptive-for");
        })
        .end(|| {
            guide(45);
        });
}
