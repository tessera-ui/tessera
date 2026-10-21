//! Badge geometry, theme, and semantic regressions.
//!
//! ## Usage
//!
//! Keep notification markers stable when content and themes change.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State, accesskit::Role, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialTheme, material_theme},
};

use super::{BadgeDefaults, badge, badge_with_content, badged_box};

fn tagged(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.into()),
        ..Default::default()
    })
}

#[tessera]
fn layout_fixture() {
    material_theme().child(|| {
        column().children(|| {
            badged_box()
                .modifier(tagged("group").size(Dp(64.0), Dp(48.0)))
                .content(|| {
                    spacer().modifier(tagged("anchor").size(Dp(24.0), Dp(24.0)));
                    spacer().modifier(tagged("detail").size(Dp(8.0), Dp(8.0)));
                })
                .badge(|| {
                    badge_with_content().modifier(tagged("large"));
                    badge().modifier(tagged("dot"));
                });
            badged_box().modifier(tagged("empty"));
            badged_box().modifier(tagged("anchor_only")).content(|| {
                spacer().modifier(Modifier::new().size(Dp(24.0), Dp(24.0)));
            });
        });
    });
}

#[test]
fn badge_slots_accept_empty_and_multiple_layout_children_and_keep_anchor_constraints() {
    crate::test_utils::run_isolated_test(
        "badge::tests::badge_slots_accept_empty_and_multiple_layout_children_and_keep_anchor_constraints",
        || {
            tessera_ui::assert_layout! {
                viewport: (200, 120),
                content: {
                    layout_fixture();
                },
                expect: {
                    node("group").position(0, 0).size(64, 48);
                    node("anchor").position(20, 12).size(24, 24);
                    node("detail").position(28, 20).size(8, 8);
                    node("large").position(52, -2).size(16, 16);
                    node("dot").position(52, -2).size(6, 6);
                    node("empty").position(0, 48).size(0, 0);
                    node("anchor_only").position(0, 48).size(24, 24);
                }
            }
        },
    );
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    count: bool,
    dark: bool,
    custom: bool,
    explicit: bool,
    large_text: bool,
    activations: usize,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let palettes = remember(|| {
        [
            MaterialTheme::default(),
            MaterialTheme::from_seed(Color::from_rgb(0.135633335, 0.080219820, 0.371237695), true),
        ]
    });
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(move || {
            let mut theme =
                palettes.with(|palettes| palettes[usize::from(settings.get().dark)].clone());
            if settings.get().large_text {
                theme.typography.label_small.font_size = Dp(18.0);
                theme.typography.label_small.line_height = Some(Dp(24.0));
            }
            theme
        })
        .child(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            surface()
                .modifier(Modifier::new().fill_max_size())
                .style(SurfaceStyle::Filled {
                    color: scheme.surface,
                })
                .content_color(Color::GREEN)
                .child(move || {
                    column()
                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                        .children(move || {
                            surface()
                                .modifier(Modifier::new().size(Dp(64.0), Dp(64.0)))
                                .style(SurfaceStyle::Filled {
                                    color: Color::TRANSPARENT,
                                })
                                .content_color(Color::GREEN)
                                .content_alignment(Alignment::Center)
                                .accessibility_role(Role::Button)
                                .accessibility_label("Open inbox")
                                .on_click(move || settings.with_mut(|value| value.activations += 1))
                                .child(move || {
                                    badged_box()
                                        .modifier(
                                            Modifier::new().semantics(SemanticsArgs {
                                                test_tag: Some("group".into()),
                                                role: Some(Role::Image),
                                                label: Some(
                                                    if settings.get().count {
                                                        "Inbox, 3 unread items"
                                                    } else {
                                                        "Inbox, new activity"
                                                    }
                                                    .into(),
                                                ),
                                                ..Default::default()
                                            }),
                                        )
                                        .content(|| {
                                            spacer().modifier(
                                                tagged("anchor").size(Dp(24.0), Dp(24.0)),
                                            );
                                        })
                                        .badge(move || {
                                            let modifier =
                                                Modifier::new().semantics(SemanticsArgs {
                                                    test_tag: Some("badge".into()),
                                                    hidden: true,
                                                    ..Default::default()
                                                });
                                            if settings.get().count {
                                                badge_with_content()
                                                    .modifier(modifier)
                                                    .container_color_optional(
                                                        settings
                                                            .get()
                                                            .custom
                                                            .then_some(Color::BLUE),
                                                    )
                                                    .content_color_optional(
                                                        settings
                                                            .get()
                                                            .explicit
                                                            .then_some(Color::RED),
                                                    )
                                                    .content(|| {
                                                        let color = use_context::<ContentColor>()
                                                            .unwrap()
                                                            .get()
                                                            .current;
                                                        surface()
                                                            .modifier(
                                                                tagged("ink")
                                                                    .size(Dp(4.0), Dp(4.0)),
                                                            )
                                                            .style(SurfaceStyle::Filled { color });
                                                        text()
                                                            .modifier(tagged("number"))
                                                            .content("3");
                                                    });
                                            } else {
                                                badge().modifier(modifier);
                                            }
                                        });
                                });
                        });
                });
        });
}

fn rect(renderer: &HeadlessRenderer, key: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == key || node["accessibility"]["label"] == key {
            return ["x", "y", "width", "height"].map(|key| node["bounds"][key].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {key}");
}

fn assert_pixel(renderer: &HeadlessRenderer, x: i64, y: i64, color: Color) {
    let pixels = renderer.read_pixels().unwrap();
    let offset = ((y as u32 * renderer.physical_size().width + x as u32) * 4) as usize;
    let channel = |v: f32| {
        ((if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }) * 255.0)
            .round() as u8
    };
    let expected = [channel(color.r), channel(color.g), channel(color.b), 255];
    let actual: [u8; 4] = pixels[offset..offset + 4].try_into().unwrap();
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, b)| a.abs_diff(b) <= 2),
        "{actual:?} != {expected:?}"
    );
}

fn check_rendering(samples: u32) {
    for (scale, expected) in [
        (1.0, [24, 6, 14, 16, 24]),
        (1.25, [30, 8, 18, 20, 30]),
        (1.5, [36, 9, 21, 24, 36]),
        (2.0, [48, 12, 28, 32, 48]),
    ] {
        check_density(samples, scale, expected);
    }
}

fn check_density(samples: u32, scale: f64, expected: [i64; 5]) {
    let [anchor, dot_size, vertical, line, large_line] = expected;
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 240,
            height: 240,
            scale_factor: scale,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    let settings = SETTINGS.get().unwrap();
    let group = rect(&renderer, "group");
    assert_eq!((group[2], group[3]), (anchor, anchor));
    let dot = rect(&renderer, "badge");
    assert_eq!(
        (dot[0] - group[0], dot[1] - group[1], dot[2], dot[3]),
        (anchor - dot_size, 0, dot_size, dot_size)
    );
    assert_pixel(
        &renderer,
        dot[0] + dot_size / 2,
        dot[1] + dot_size / 2,
        BadgeDefaults::container_color(),
    );
    for dark in [false, true] {
        let scheme = if dark {
            MaterialTheme::from_seed(Color::from_rgb(0.135633335, 0.080219820, 0.371237695), true)
                .color_scheme
        } else {
            MaterialTheme::default().color_scheme
        };
        for (custom, explicit, large_text) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (false, false, true),
            (false, false, false),
        ] {
            settings.set(Settings {
                count: true,
                dark,
                custom,
                explicit,
                large_text,
                ..Default::default()
            });
            renderer.render_frames(5);
            assert_eq!(rect(&renderer, "group"), group);
            let badge = rect(&renderer, "badge");
            assert_eq!(badge[0] - group[0], anchor / 2);
            assert_eq!(badge[1] - group[1], vertical - badge[3]);
            assert_eq!(badge[3], if large_text { large_line } else { line });
            let number = rect(&renderer, "3");
            assert_eq!(number[3], if large_text { large_line } else { line });
            let ink = rect(&renderer, "ink");
            assert_pixel(
                &renderer,
                ink[0] + ink[2] / 2,
                ink[1] + ink[3] / 2,
                if explicit {
                    Color::RED
                } else if custom {
                    Color::GREEN
                } else {
                    scheme.on_error
                },
            );
            assert_pixel(
                &renderer,
                badge[0] + 2,
                badge[1] + badge[3] / 2,
                if custom { Color::BLUE } else { scheme.error },
            );
            let snapshot = renderer.accessibility_snapshot().unwrap();
            let (_, image) = snapshot
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Inbox, 3 unread items"))
                .unwrap();
            assert_eq!(image.role(), Role::Image);
            assert!(
                !snapshot
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("3"))
            );
        }
    }
    settings.set(Settings {
        count: true,
        ..Default::default()
    });
    renderer.render_frames(5);
    let badge = rect(&renderer, "badge");
    let point = PxPosition::from_f32_xy(
        (badge[0] + badge[2] / 2) as f32,
        (badge[1] + badge[3] / 2) as f32,
    );
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_eq!(
        settings.get().activations,
        1,
        "badge drawing must not intercept its parent's input"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn badge_theme_geometry_semantics_and_pointer_passthrough_msaa1() {
    crate::test_utils::run_isolated_test(
        "badge::tests::badge_theme_geometry_semantics_and_pointer_passthrough_msaa1",
        || check_rendering(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn badge_theme_geometry_semantics_and_pointer_passthrough_msaa4() {
    crate::test_utils::run_isolated_test(
        "badge::tests::badge_theme_geometry_semantics_and_pointer_passthrough_msaa4",
        || check_rendering(4),
    );
}
