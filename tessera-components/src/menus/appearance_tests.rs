use std::cell::Cell;

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Node, Rect, Role, Toggled},
    remember, tessera,
};

use crate::{
    modifier::ModifierExt as _,
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, material_theme},
};

use super::{MenuColorStyle, MenuController, MenuPosition, menu_group, menu_item, menu_provider};

#[derive(Clone, Copy)]
struct Settings {
    style: MenuColorStyle,
    selected: bool,
    enabled: bool,
    width: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            style: MenuColorStyle::Standard,
            selected: false,
            enabled: true,
            width: 240.0,
        }
    }
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static ACTIVATIONS: Cell<Option<State<u32>>> = const { Cell::new(None) };
}

const STANDARD: Color = Color::new(0.4, 0.5, 0.6, 1.0);
const SELECTED: Color = Color::new(0.7, 0.3, 0.2, 1.0);
const VIBRANT_SELECTED: Color = Color::new(0.2, 0.3, 0.1, 1.0);

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let activations = remember(|| 0);
    let controller = remember(|| {
        let mut controller = MenuController::new();
        controller.open();
        controller
    });
    SETTINGS.set(Some(settings));
    ACTIVATIONS.set(Some(activations));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface_container_low = STANDARD;
            theme.color_scheme.tertiary_container = SELECTED;
            theme.color_scheme.tertiary = VIBRANT_SELECTED;
            theme
        })
        .child(move || {
            menu_provider()
                .controller(controller)
                .grouped(true)
                .color_style(settings.get().style)
                .modifier(Modifier::new().width(Dp(settings.get().width)))
                .main_content(|| {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: Color::WHITE,
                        })
                        .modifier(Modifier::new().fill_max_size())
                        .child(|| {});
                })
                .menu_content(move || {
                    menu_group()
                        .position(MenuPosition::First)
                        .accessibility_label("Primary group")
                        .children(move || {
                            menu_item()
                                .label("A long menu action that must wrap")
                                .supporting_text("Additional context")
                                .trailing_text("Ctrl+S")
                                .position(MenuPosition::First)
                                .close_on_click(false)
                                .on_click(move || activations.set(activations.get() + 1));
                            menu_item()
                                .label("Priority")
                                .position(MenuPosition::Last)
                                .selected(settings.get().selected)
                                .enabled(settings.get().enabled)
                                .close_on_click(false)
                                .on_click(move || {
                                    settings.with_mut(|value| value.selected = !value.selected)
                                });
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .accessibility_label("Other group")
                        .children(move || {
                            menu_item()
                                .label("Unavailable")
                                .enabled(false)
                                .on_click(move || activations.set(activations.get() + 1));
                            menu_item().label("No action");
                        });
                });
        });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {role:?}: {label}"))
        .1
}

fn click(renderer: &mut HeadlessRenderer, bounds: Rect) {
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(2);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(480.0, 580.0));
    renderer.render_frames(90);
}

fn assert_color(pixels: &[u8], width: usize, x: f64, y: f64, color: Color) {
    let offset = (y.round() as usize * width + x.round() as usize) * 4;
    let channels: [f32; 4] = color.into();
    for (index, expected) in channels.into_iter().enumerate() {
        let encoded = if index == 3 || expected <= 0.0031308 {
            if index == 3 {
                expected
            } else {
                expected * 12.92
            }
        } else {
            1.055 * expected.powf(1.0 / 2.4) - 0.055
        };
        assert!(
            (pixels[offset + index] as f32 - encoded * 255.0).abs() <= 2.0,
            "pixel at ({x}, {y}), channel {index}: {}, expected {color:?}",
            pixels[offset + index]
        );
    }
}

fn check_appearance(samples: u32) {
    let scale = 1.5;
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 750,
            height: 900,
            scale_factor: scale,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let settings = SETTINGS.get().expect("settings");
    let initial = renderer.read_pixels().expect("pixels");
    let first = node(&renderer, "Primary group", Role::Group)
        .bounds()
        .expect("bounds");
    let last = node(&renderer, "Other group", Role::Group)
        .bounds()
        .expect("bounds");
    assert!(
        (last.y0 - first.y1 - 2.0 * scale).abs() <= 1.0,
        "group gap: {first:?}, {last:?}"
    );
    let label = node(&renderer, "A long menu action that must wrap", Role::Label)
        .bounds()
        .expect("label bounds");
    let shortcut = node(&renderer, "Ctrl+S", Role::Label)
        .bounds()
        .expect("shortcut bounds");
    assert!(
        (label.x0 - first.x0 - 16.0 * scale).abs() <= 1.0,
        "no phantom icon inset: {label:?}, {first:?}"
    );
    assert!(
        label.x1 + 11.0 * scale <= shortcut.x0,
        "label must reserve shortcut space"
    );
    assert!(
        (first.x1 - shortcut.x1 - 16.0 * scale).abs() <= 1.0,
        "shortcut end inset"
    );
    let priority = node(&renderer, "Priority", Role::MenuItemRadio);
    assert_eq!(priority.toggled(), Some(Toggled::False));
    let bounds = priority.bounds().expect("priority bounds");
    assert!(bounds.y1 - bounds.y0 >= 48.0 * scale, "touch target");
    for label in ["Unavailable", "No action"] {
        let item = node(&renderer, label, Role::MenuItem);
        assert!(item.is_disabled());
        assert!(!item.supports_action(tessera_ui::accesskit::Action::Click));
        click(&mut renderer, item.bounds().expect("bounds"));
    }
    assert_eq!(ACTIVATIONS.get().expect("activations").get(), 0);
    for style in [MenuColorStyle::Standard, MenuColorStyle::Vibrant] {
        for selected in [false, true] {
            settings.set(Settings {
                style,
                selected,
                ..Settings::default()
            });
            renderer.render_frames(3);
            let priority = node(&renderer, "Priority", Role::MenuItemRadio);
            assert_eq!(
                priority.toggled(),
                Some(if selected {
                    Toggled::True
                } else {
                    Toggled::False
                })
            );
            assert_eq!(
                priority.bounds().expect("bounds"),
                bounds,
                "selection preserves target"
            );
            let expected = match (style, selected) {
                (MenuColorStyle::Standard, false) => STANDARD,
                (MenuColorStyle::Standard, true) | (MenuColorStyle::Vibrant, false) => SELECTED,
                (MenuColorStyle::Vibrant, true) => VIBRANT_SELECTED,
            };
            assert_color(
                &renderer.read_pixels().expect("pixels"),
                750,
                bounds.x1 - 20.0 * scale,
                (bounds.y0 + bounds.y1) / 2.0,
                expected,
            );
        }
    }
    settings.set(Settings {
        width: 160.0,
        ..Settings::default()
    });
    renderer.render_frames(3);
    let narrow = node(&renderer, "A long menu action that must wrap", Role::Label)
        .bounds()
        .expect("bounds");
    let shortcut = node(&renderer, "Ctrl+S", Role::Label)
        .bounds()
        .expect("bounds");
    assert!(
        narrow.y1 - narrow.y0 > label.y1 - label.y0,
        "narrow label wraps"
    );
    assert!(
        narrow.x1 + 11.0 * scale <= shortcut.x0,
        "narrow text cannot overlap shortcut"
    );
    settings.set(Settings::default());
    renderer.render_frames(90);
    assert_eq!(renderer.read_pixels().expect("restored pixels"), initial);
    click(&mut renderer, bounds);
    assert!(
        settings.get().selected,
        "actual pointer toggles the controlled selection"
    );
    settings.with_mut(|value| value.enabled = false);
    renderer.render_frames(3);
    assert_eq!(
        node(&renderer, "Priority", Role::MenuItemRadio)
            .bounds()
            .expect("disabled bounds"),
        bounds
    );
    click(&mut renderer, bounds);
    assert!(settings.get().selected, "disabled item cannot toggle");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_expressive_menu_groups_reserve_text_and_restore_pixels() {
    check_appearance(1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_expressive_menu_groups_reserve_text_and_restore_pixels_msaa() {
    check_appearance(4);
}
