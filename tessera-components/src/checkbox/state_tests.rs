//! Checkbox theme, validation, and color precedence regressions.
//!
//! ## Usage
//!
//! Preserve complete selection and validation states across theme changes.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Invalid, Node, NodeId, Role, Toggled, TreeId},
    remember, tessera, use_context,
};

use crate::{
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, material_theme},
};

use super::checkbox;

const CUSTOM_FILL: Color = Color::new(0.0, 1.0, 1.0, 1.0);
const CUSTOM_ICON: Color = Color::new(1.0, 1.0, 0.0, 1.0);

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    dark: bool,
    checked: bool,
    mixed: bool,
    disabled: bool,
    error: bool,
    custom: bool,
    generic_invalid: bool,
    activations: usize,
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

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let palettes = remember(palettes);
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
                .child(move || {
                    column().children(move || {
                        let value = settings.get();
                        checkbox()
                            .checked(value.checked)
                            .indeterminate(value.mixed)
                            .is_error(value.error)
                            .disabled(value.disabled)
                            .color_optional(value.custom.then_some(Color::GREEN))
                            .checked_color_optional(value.custom.then_some(CUSTOM_FILL))
                            .checkmark_color_optional(value.custom.then_some(CUSTOM_ICON))
                            .accessibility_label("Selection")
                            .accessibility_description("Choose an option")
                            .on_toggle(move |checked| {
                                settings.with_mut(|value| {
                                    value.checked = checked;
                                    value.mixed = false;
                                    value.activations += 1;
                                })
                            });
                        spacer().modifier(Modifier::new().size(Dp(8.0), Dp(8.0)).semantics(
                            SemanticsArgs {
                                role: Some(Role::TextInput),
                                label: Some("Custom validation".into()),
                                invalid: value.generic_invalid.then_some(Invalid::Spelling),
                                ..Default::default()
                            },
                        ));
                    });
                });
        });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap()
}

fn pixel(renderer: &HeadlessRenderer, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
    renderer.read_pixels().unwrap()[offset..offset + 4]
        .try_into()
        .unwrap()
}

fn encoded(color: Color, background: Color) -> [u8; 4] {
    let channel = |value: f32| {
        let encoded = if value <= 0.0031308 {
            12.92 * value
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    [
        channel(color.r * color.a + background.r * (1.0 - color.a)),
        channel(color.g * color.a + background.g * (1.0 - color.a)),
        channel(color.b * color.a + background.b * (1.0 - color.a)),
        255,
    ]
}

fn assert_color(actual: [u8; 4], expected: [u8; 4], context: &str) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
        "{context}: {actual:?} != {expected:?}"
    );
}

fn action(renderer: &mut HeadlessRenderer, action: Action) -> bool {
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer, "Selection").0,
        data: None,
    })
}

fn check_states(samples: u32) {
    let palettes = palettes();
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 200,
            height: 160,
            scale_factor: 2.0,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(30);
    let settings = SETTINGS.get().unwrap();
    let identity = node(&renderer, "Selection").0;
    let bounds = node(&renderer, "Selection").1.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (96.0, 96.0));
    let cx = ((bounds.x0 + bounds.x1) / 2.0) as u32;
    let cy = ((bounds.y0 + bounds.y1) / 2.0) as u32;
    let point = PxPosition::from_f32_xy(cx as f32, cy as f32);
    for dark in [false, true] {
        let scheme = &palettes[usize::from(dark)].color_scheme;
        for disabled in [false, true] {
            for error in [false, true] {
                for selection in [Toggled::False, Toggled::True, Toggled::Mixed] {
                    settings.set(Settings {
                        dark,
                        disabled,
                        error,
                        checked: selection == Toggled::True,
                        mixed: selection == Toggled::Mixed,
                        ..Default::default()
                    });
                    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(190.0, 150.0));
                    renderer.render_frames(30);
                    let (id, control) = node(&renderer, "Selection");
                    assert_eq!(id, identity);
                    assert_eq!(control.toggled(), Some(selection));
                    assert_eq!(control.invalid(), error.then_some(Invalid::True));
                    assert_eq!(control.description(), Some("Choose an option"));
                    assert_eq!(control.is_disabled(), disabled);
                    assert_eq!(control.supports_action(Action::Click), !disabled);
                    let active = selection != Toggled::False;
                    let ink = if disabled {
                        scheme.on_surface.with_alpha(0.38)
                    } else if error {
                        scheme.error
                    } else if active {
                        scheme.primary
                    } else {
                        scheme.on_surface_variant
                    };
                    assert_color(
                        pixel(&renderer, cx - 16, cy),
                        encoded(ink, scheme.surface),
                        "outline/container role",
                    );
                    assert_color(
                        pixel(&renderer, cx + 8, cy + 8),
                        encoded(
                            if active { ink } else { Color::TRANSPARENT },
                            scheme.surface,
                        ),
                        "fill role",
                    );
                    if active {
                        let icon = if disabled {
                            scheme.surface
                        } else if error {
                            scheme.on_error
                        } else {
                            scheme.on_primary
                        };
                        let (x, y) = if selection == Toggled::Mixed {
                            (cx, cy)
                        } else {
                            (cx - 3, cy + 5)
                        };
                        assert_color(
                            pixel(&renderer, x, y),
                            encoded(icon, scheme.surface),
                            "selected icon role",
                        );
                    }
                    assert_eq!(settings.get().activations, 0);
                }
            }
        }
        settings.set(Settings {
            dark,
            error: true,
            ..Default::default()
        });
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.render_frames(30);
        assert_color(
            pixel(&renderer, cx - 16, cy),
            encoded(scheme.error, scheme.surface),
            "error hover",
        );
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(30);
        assert_color(
            pixel(&renderer, cx - 16, cy),
            encoded(scheme.error, scheme.surface),
            "error press",
        );
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(190.0, 150.0));
        renderer.pointer_release(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(190.0, 150.0),
            PressKeyEventType::Left,
        );
        renderer.render_frames(30);
        assert!(action(&mut renderer, Action::Focus));
        renderer.render_frames(30);
        assert_color(
            pixel(&renderer, cx - 16, cy),
            encoded(scheme.error, scheme.surface),
            "error focus",
        );
        assert!(action(&mut renderer, Action::Click));
        renderer.render_frames(30);
        assert_eq!(settings.get().activations, 1);
        assert_eq!(
            node(&renderer, "Selection").1.invalid(),
            Some(Invalid::True)
        );
        settings.with_mut(|value| {
            value.error = false;
            value.generic_invalid = true;
        });
        renderer.render_frames(30);
        assert_eq!(node(&renderer, "Selection").1.invalid(), None);
        assert_eq!(
            node(&renderer, "Custom validation").1.invalid(),
            Some(Invalid::Spelling)
        );
        settings.with_mut(|value| {
            value.error = true;
            value.custom = true;
            value.generic_invalid = false;
        });
        renderer.render_frames(30);
        assert_eq!(node(&renderer, "Custom validation").1.invalid(), None);
        assert_color(
            pixel(&renderer, cx + 8, cy + 8),
            encoded(CUSTOM_FILL, scheme.surface),
            "custom checked color overrides error",
        );
        assert_color(
            pixel(&renderer, cx - 3, cy + 5),
            encoded(CUSTOM_ICON, scheme.surface),
            "custom icon overrides error",
        );
        settings.with_mut(|value| value.checked = false);
        renderer.render_frames(30);
        assert_color(
            pixel(&renderer, cx - 16, cy),
            encoded(Color::GREEN, scheme.surface),
            "custom focused outline overrides error",
        );
        settings.with_mut(|value| value.disabled = true);
        renderer.render_frames(30);
        assert!(!action(&mut renderer, Action::Click));
        assert_eq!(settings.get().activations, 1);
        FocusManager::current().clear_focus(true);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_light_dark_validation_colors_and_semantics_msaa1() {
    crate::test_utils::run_isolated_test(
        "checkbox::state_tests::checkbox_light_dark_validation_colors_and_semantics_msaa1",
        || check_states(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_light_dark_validation_colors_and_semantics_msaa4() {
    crate::test_utils::run_isolated_test(
        "checkbox::state_tests::checkbox_light_dark_validation_colors_and_semantics_msaa4",
        || check_states(4),
    );
}
