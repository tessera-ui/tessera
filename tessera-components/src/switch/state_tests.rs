//! Switch theme colors and noninteractive accessibility regressions.
//!
//! ## Usage
//!
//! Preserve switch colors and accessible values across state changes.

use std::{cell::Cell, sync::Arc};

use tessera_ui::{
    Color, Dp, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
    remember, tessera, use_context,
};

use crate::{
    icon::icon,
    modifier::{InteractionVisualState, ModifierExt as _},
    pipelines::image_vector::command::{ImageVectorData, ImageVectorVertex},
    surface::{SurfaceStyle, surface},
    theme::{ContentColor, MaterialTheme, material_theme},
};

use super::switch;

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    checked: bool,
    disabled: bool,
    dark: bool,
    passive: bool,
    custom: bool,
    icon: bool,
    icon_size: Option<Dp>,
    activations: usize,
    presentation: Option<InteractionVisualState>,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

fn palettes() -> [MaterialTheme; 2] {
    [
        MaterialTheme::default(),
        MaterialTheme::from_seed(Color::from_rgb(0.135633335, 0.080219820, 0.371237695), true),
    ]
}

fn square_icon() -> ImageVectorData {
    ImageVectorData::new(
        1.0,
        1.0,
        Arc::new(
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
                .map(|position| ImageVectorVertex {
                    position,
                    color: Color::WHITE,
                })
                .to_vec(),
        ),
        Arc::new(vec![0, 1, 2, 0, 2, 3]),
    )
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let palettes = remember(palettes);
    let icon_data = remember(square_icon);
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(move || palettes.with(|palettes| palettes[usize::from(settings.get().dark)].clone()))
        .child(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            surface()
                .modifier(Modifier::new().fill_max_size())
                .style(SurfaceStyle::Filled {
                    color: scheme.surface,
                })
                .content_color(Color::GREEN)
                .child(move || {
                    let value = settings.get();
                    switch()
                        .checked(value.checked)
                        .enabled(!value.disabled)
                        .presentation_state_optional(value.presentation)
                        .thumb_color_optional(value.custom.then_some(Color::GREEN))
                        .thumb_checked_color_optional(value.custom.then_some(Color::GREEN))
                        .accessibility_label("Connectivity")
                        .accessibility_description("Network connection")
                        .on_toggle_optional(if value.passive {
                            None
                        } else {
                            Some(tessera_ui::CallbackWith::new(move |checked| {
                                settings.with_mut(|value| {
                                    value.checked = checked;
                                    value.activations += 1;
                                });
                            }))
                        })
                        .child(move || {
                            let color = use_context::<ContentColor>().unwrap().get().current;
                            if settings.get().icon {
                                icon()
                                    .vector(icon_data.get())
                                    .size_optional(settings.get().icon_size);
                            } else {
                                surface()
                                    .modifier(Modifier::new().size(Dp(8.0), Dp(8.0)))
                                    .style(SurfaceStyle::Filled { color });
                            }
                        });
                });
        });
}

fn node(renderer: &HeadlessRenderer) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Connectivity"))
        .expect("switch semantics")
}

fn action(renderer: &mut HeadlessRenderer, action: Action) -> bool {
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer).0,
        data: None,
    })
}

fn encoded(color: Color) -> [u8; 4] {
    let channel = |value: f32| {
        let encoded = if value <= 0.0031308 {
            12.92 * value
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    [channel(color.r), channel(color.g), channel(color.b), 255]
}

fn composite(ink: Color, background: Color, alpha: f32) -> Color {
    Color::from_rgb(
        ink.r * alpha + background.r * (1.0 - alpha),
        ink.g * alpha + background.g * (1.0 - alpha),
        ink.b * alpha + background.b * (1.0 - alpha),
    )
}

fn assert_pixel(renderer: &HeadlessRenderer, x: u32, y: u32, color: Color, context: &str) {
    let pixels = renderer.read_pixels().unwrap();
    let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
    let actual: [u8; 4] = pixels[offset..offset + 4].try_into().unwrap();
    let expected = encoded(color);
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, b)| a.abs_diff(b) <= 2),
        "{context}: {actual:?} != {expected:?}"
    );
}

fn check_states(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 200,
            height: 140,
            scale_factor: 2.0,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(30);
    let settings = SETTINGS.get().unwrap();
    let palettes = palettes();
    let identity = node(&renderer).0;
    for dark in [false, true] {
        let scheme = &palettes[usize::from(dark)].color_scheme;
        for disabled in [false, true] {
            for checked in [false, true] {
                settings.set(Settings {
                    dark,
                    disabled,
                    checked,
                    ..Default::default()
                });
                FocusManager::current().clear_focus(true);
                renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(190.0, 130.0));
                renderer.render_frames(30);
                let (id, control) = node(&renderer);
                assert_eq!(id, identity);
                assert_eq!(control.role(), Role::Switch);
                assert_eq!(
                    control.toggled(),
                    Some(if checked {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                );
                assert_eq!(control.is_disabled(), disabled);
                assert_eq!(control.supports_action(Action::Click), !disabled);
                let bounds = control.bounds().unwrap();
                assert_eq!(
                    (bounds.x1 - bounds.x0, bounds.y1 - bounds.y0),
                    (104.0, 96.0)
                );
                let x = if checked { 72 } else { 32 };
                let thumb = if disabled && checked {
                    scheme.surface
                } else if disabled {
                    composite(scheme.on_surface, scheme.surface, 0.38)
                } else if checked {
                    scheme.on_primary
                } else {
                    scheme.outline
                };
                let icon = if disabled {
                    composite(
                        if checked {
                            scheme.on_surface
                        } else {
                            scheme.surface_container_highest
                        },
                        scheme.surface,
                        0.38,
                    )
                } else if checked {
                    scheme.on_primary_container
                } else {
                    scheme.surface_container_highest
                };
                assert_pixel(&renderer, x, 62, thumb, "resting thumb");
                assert_pixel(&renderer, x, 48, icon, "thumb icon");
                let track = if disabled {
                    composite(
                        if checked {
                            scheme.on_surface
                        } else {
                            scheme.surface_container_highest
                        },
                        scheme.surface,
                        0.12,
                    )
                } else if checked {
                    scheme.primary
                } else {
                    scheme.surface_container_highest
                };
                assert_pixel(&renderer, 52, 24, track, "track");
                assert_pixel(&renderer, x, 12, scheme.surface, "resting state layer");
                assert_pixel(&renderer, 52, 8, scheme.surface, "no resting focus ring");
                if !disabled {
                    renderer
                        .pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(x as f32, 48.0));
                    renderer.render_frames(30);
                    let active = if checked {
                        scheme.primary_container
                    } else {
                        scheme.on_surface_variant
                    };
                    assert_pixel(&renderer, x, 62, active, "hover thumb");
                    let layer = if checked {
                        scheme.primary
                    } else {
                        scheme.on_surface
                    };
                    assert_pixel(
                        &renderer,
                        x,
                        12,
                        composite(layer, scheme.surface, 0.08),
                        "hover layer ignores surrounding content color",
                    );
                    renderer.pointer_press(
                        MOUSE_POINTER_ID,
                        PxPosition::from_f32_xy(x as f32, 48.0),
                        PressKeyEventType::Left,
                    );
                    renderer.render_frames(30);
                    assert_pixel(&renderer, x, 62, active, "pressed thumb");
                    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(190.0, 130.0));
                    renderer.pointer_release(
                        MOUSE_POINTER_ID,
                        PxPosition::from_f32_xy(190.0, 130.0),
                        PressKeyEventType::Left,
                    );
                    renderer.render_frames(30);
                    assert!(action(&mut renderer, Action::Focus));
                    renderer.render_frames(30);
                    assert_pixel(&renderer, x, 62, active, "focused thumb");
                    assert_pixel(
                        &renderer,
                        52,
                        8,
                        scheme.secondary,
                        "actual accessibility focus ring",
                    );
                    assert_pixel(
                        &renderer,
                        x,
                        12,
                        composite(layer, scheme.surface, 0.1),
                        "focus layer role",
                    );
                    settings.with_mut(|value| value.custom = true);
                    renderer.render_frames(30);
                    assert_pixel(&renderer, x, 62, Color::GREEN, "custom focused thumb");
                }
                assert_eq!(settings.get().activations, 0);
            }
        }
    }
    for dark in [false, true] {
        let scheme = &palettes[usize::from(dark)].color_scheme;
        for checked in [false, true] {
            for size in [None, Some(Dp(12.0))] {
                FocusManager::current().clear_focus(true);
                settings.set(Settings {
                    dark,
                    checked,
                    icon: true,
                    icon_size: size,
                    ..Default::default()
                });
                renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(190.0, 130.0));
                renderer.render_frames(30);
                let x = if checked { 72 } else { 32 };
                let thumb = if checked {
                    scheme.on_primary
                } else {
                    scheme.outline
                };
                let ink = if checked {
                    scheme.on_primary_container
                } else {
                    scheme.surface_container_highest
                };
                let half_width = if size.is_some() { 12 } else { 16 };
                assert_pixel(&renderer, x + half_width - 2, 48, ink, "icon interior");
                assert_pixel(
                    &renderer,
                    x + half_width + 2,
                    48,
                    thumb,
                    "icon respects default or explicit size",
                );
            }
        }
    }
    for presentation in [
        InteractionVisualState::Hover,
        InteractionVisualState::Focus,
        InteractionVisualState::Pressed,
    ] {
        FocusManager::current().clear_focus(true);
        settings.set(Settings {
            checked: true,
            presentation: Some(presentation),
            ..Default::default()
        });
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(190.0, 130.0));
        renderer.render_frames(30);
        let scheme = &palettes[0].color_scheme;
        assert_eq!(settings.get().activations, 0);
        assert_pixel(
            &renderer,
            72,
            62,
            scheme.primary_container,
            "presented interaction thumb",
        );
        assert_pixel(
            &renderer,
            52,
            8,
            if presentation == InteractionVisualState::Focus {
                scheme.secondary
            } else {
                scheme.surface
            },
            "presented focus ring is exclusive to focus",
        );
        if presentation == InteractionVisualState::Pressed {
            assert_pixel(
                &renderer,
                98,
                48,
                scheme.primary_container,
                "presented native28dp pressed thumb",
            );
        }
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(72.0, 48.0));
        renderer.render_frames(30);
        assert_pixel(
            &renderer,
            52,
            8,
            scheme.surface,
            "live hover overrides presented focus",
        );
        settings.with_mut(|value| value.disabled = true);
        renderer.render_frames(30);
        assert_pixel(
            &renderer,
            52,
            8,
            scheme.surface,
            "disabled presentation has no focus ring",
        );
        assert_eq!(node(&renderer).1.toggled(), Some(Toggled::True));
        assert!(!action(&mut renderer, Action::Click));
        assert_eq!(settings.get().activations, 0);
    }
    for disabled in [false, true] {
        for checked in [false, true] {
            settings.set(Settings {
                passive: true,
                checked,
                disabled,
                ..Default::default()
            });
            renderer.render_frames(30);
            let (_, control) = node(&renderer);
            assert_eq!(control.role(), Role::Switch);
            assert_eq!(
                control.toggled(),
                Some(if checked {
                    Toggled::True
                } else {
                    Toggled::False
                })
            );
            assert_eq!(control.description(), Some("Network connection"));
            assert_eq!(control.is_disabled(), disabled);
            assert!(!control.supports_action(Action::Click));
            assert!(!control.supports_action(Action::Focus));
            assert!(!action(&mut renderer, Action::Click));
            assert_eq!(settings.get().activations, 0);
        }
    }
    FocusManager::current().clear_focus(true);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn switch_theme_state_pixels_and_passive_semantics_msaa1() {
    crate::test_utils::run_isolated_test(
        "switch::state_tests::switch_theme_state_pixels_and_passive_semantics_msaa1",
        || check_states(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn switch_theme_state_pixels_and_passive_semantics_msaa4() {
    crate::test_utils::run_isolated_test(
        "switch::state_tests::switch_theme_state_pixels_and_passive_semantics_msaa4",
        || check_states(4),
    );
}
