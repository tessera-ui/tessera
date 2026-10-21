//! Chip interaction colors and elevation rendering regressions.
//!
//! ## Usage
//!
//! Preserve chip shadows and state colors during pointer and focus changes.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, TreeId},
    remember, tessera,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, material_theme},
};

use super::{ChipDefaults, ChipStyle, ChipVariant, chip};

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    variant: ChipVariant,
    style: ChipStyle,
    selected: bool,
    enabled: bool,
    reference: Dp,
    custom: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

fn tagged(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.into()),
        ..Default::default()
    })
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        variant: ChipVariant::Assist,
        style: ChipStyle::Elevated,
        selected: false,
        enabled: true,
        reference: Dp(1.0),
        custom: false,
    });
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.primary = Color::RED;
            theme.color_scheme.on_surface = Color::BLACK;
            theme.color_scheme.on_surface_variant = Color::GREEN;
            theme.color_scheme.outline_variant = Color::BLUE;
            theme
        })
        .child(move || {
            row()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(Color::WHITE)
                        .padding_all(Dp(32.0)),
                )
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(move || {
                    let config = settings.get();
                    let custom = config.custom.then(|| {
                        let mut colors = ChipDefaults::colors(
                            &MaterialTheme::default().color_scheme,
                            config.variant,
                            config.style,
                        );
                        colors.label_color = Color::GREEN;
                        colors.leading_icon_color = Color::BLUE;
                        colors.selected_leading_icon_color = Color::BLUE;
                        colors
                    });
                    chip()
                        .shape(ChipDefaults::shape())
                        .variant(config.variant)
                        .style(config.style)
                        .selected(config.selected)
                        .enabled(config.enabled)
                        .colors_optional(custom)
                        .modifier(tagged("chip").width(Dp(180.0)))
                        .accessibility_label("Chip action")
                        .label("Sample chip")
                        .leading_icon(filled::HOME_SVG)
                        .on_click(|| {});
                    spacer().modifier(Modifier::new().width(Dp(32.0)));
                    boxed()
                        .modifier(tagged("reference").size(Dp(180.0), Dp(48.0)))
                        .alignment(Alignment::Center)
                        .children(move || {
                            surface()
                                .modifier(Modifier::new().size(Dp(180.0), Dp(32.0)))
                                .shape(ChipDefaults::shape())
                                .style(SurfaceStyle::Filled {
                                    color: Color::WHITE,
                                })
                                .elevation(settings.get().reference);
                        });
                });
        });
}

fn rect(renderer: &HeadlessRenderer, key: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == key || node["accessibility"]["label"] == key {
            return ["x", "y", "width", "height"]
                .map(|field| node["bounds"][field].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {key}");
}

fn shadow(renderer: &HeadlessRenderer, key: &str) -> Vec<u8> {
    let [x, y, width, _] = rect(renderer, key);
    let pixels = renderer.read_pixels().unwrap();
    let mut strip = Vec::new();
    for py in y + 41..y + 63 {
        let start = ((py as u32 * renderer.physical_size().width + x as u32) * 4) as usize;
        strip.extend_from_slice(&pixels[start..start + width as usize * 4]);
    }
    strip
}

fn assert_shadow(renderer: &HeadlessRenderer, elevated: bool) {
    let actual = shadow(renderer, "chip");
    let expected = shadow(renderer, "reference");
    let differences: Vec<_> = actual
        .iter()
        .zip(&expected)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(10)
        .map(|(i, (a, b))| (i, *a, *b))
        .collect();
    assert!(
        differences.is_empty(),
        "chip {:?}, reference {:?}, first differences {:?}",
        rect(renderer, "chip"),
        rect(renderer, "reference"),
        differences
    );
    assert_eq!(
        actual.chunks_exact(4).any(|pixel| pixel[..3] != [255; 3]),
        elevated
    );
}

fn count_icon(renderer: &HeadlessRenderer, color: [u8; 3]) -> usize {
    let [x, y, _, height] = rect(renderer, "Sample chip");
    let pixels = renderer.read_pixels().unwrap();
    let mut count = 0;
    for py in y + height / 2 - 9..y + height / 2 + 9 {
        for px in x - 26..x - 8 {
            let offset = ((py as u32 * renderer.physical_size().width + px as u32) * 4) as usize;
            if pixels[offset..offset + 3]
                .iter()
                .zip(color)
                .all(|(a, b)| a.abs_diff(b) <= 2)
            {
                count += 1;
            }
        }
    }
    count
}

fn check_appearance(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 160,
            sample_count: samples,
            frame_time: Duration::from_millis(10),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    let settings = SETTINGS.get().unwrap();
    let point = PxPosition::from_f32_xy(120.0, 56.0);
    let outside = PxPosition::from_f32_xy(580.0, 140.0);
    assert_shadow(&renderer, true);
    for variant in [
        ChipVariant::Assist,
        ChipVariant::Suggestion,
        ChipVariant::Filter,
        ChipVariant::Input,
    ] {
        settings.with_mut(|s| {
            s.variant = variant;
            s.reference = Dp(3.0);
        });
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.render_frames(5);
        assert_ne!(shadow(&renderer, "chip"), shadow(&renderer, "reference"));
        renderer.render_frames(20);
        assert_shadow(&renderer, true);
        settings.with_mut(|s| {
            s.enabled = false;
            s.reference = Dp::ZERO;
        });
        renderer.render_frames(2);
        assert_shadow(&renderer, false);
        settings.with_mut(|s| {
            s.enabled = true;
            s.reference = Dp(1.0);
        });
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(20);
        assert_shadow(&renderer, true);
    }
    settings.with_mut(|s| {
        s.variant = ChipVariant::Filter;
        s.style = ChipStyle::Flat;
        s.selected = true;
        s.reference = Dp(1.0);
    });
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.render_frames(20);
    assert_shadow(&renderer, true);
    settings.with_mut(|s| s.reference = Dp::ZERO);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(20);
    assert_shadow(&renderer, false);
    renderer.pointer_move(MOUSE_POINTER_ID, outside);
    renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    FocusManager::current().clear_focus(true);
    settings.with_mut(|s| {
        s.variant = ChipVariant::Input;
        s.selected = false;
    });
    renderer.render_frames(30);
    assert!(count_icon(&renderer, [0, 255, 0]) > 20);
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.render_frames(5);
    assert!(count_icon(&renderer, [246, 80, 0]) > 20);
    settings.with_mut(|s| s.custom = true);
    renderer.render_frames(5);
    assert!(count_icon(&renderer, [0, 80, 246]) > 20);
    settings.with_mut(|s| s.custom = false);
    renderer.pointer_move(MOUSE_POINTER_ID, outside);
    let id = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Chip action"))
        .unwrap()
        .0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }));
    renderer.render_frames(10);
    assert!(count_icon(&renderer, [243, 89, 0]) > 20);
    let [x, y, _, _] = rect(&renderer, "chip");
    let pixels = renderer.read_pixels().unwrap();
    let offset = (((y + 8) as u32 * renderer.physical_size().width + (x + 90) as u32) * 4) as usize;
    let [red, green, blue]: [u8; 3] = pixels[offset..offset + 3].try_into().unwrap();
    assert!(
        green > red.saturating_add(50) && green > blue.saturating_add(50),
        "focus outline must use the green variant role, got {red}, {green}, {blue}"
    );
    settings.with_mut(|s| s.enabled = false);
    renderer.render_frames(2);
    assert!(count_icon(&renderer, [206, 206, 206]) > 20);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_chip_shadows_colors_and_disabled_transitions_msaa1() {
    crate::test_utils::run_isolated_test(
        "chip::appearance_tests::native_chip_shadows_colors_and_disabled_transitions_msaa1",
        || check_appearance(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_chip_shadows_colors_and_disabled_transitions_msaa4() {
    crate::test_utils::run_isolated_test(
        "chip::appearance_tests::native_chip_shadows_colors_and_disabled_transitions_msaa4",
        || check_appearance(4),
    );
}
