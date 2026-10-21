//! Selectable chip shape motion and fixed-shape regressions.
//!
//! ## Usage
//!
//! Preserve selection and pressed silhouettes while chip states change.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State, remember, tessera,
};

use crate::{
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    theme::{MaterialTheme, material_theme},
};

use super::{ChipBorder, ChipDefaults, ChipShapeStyle, ChipStyle, ChipVariant, chip};

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    selected: bool,
    disabled: bool,
    standard: bool,
    fixed: bool,
    input: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    SETTINGS.set(Some(settings));
    material_theme().child(move || {
        column()
            .modifier(
                Modifier::new()
                    .fill_max_size()
                    .background(Color::WHITE)
                    .padding_all(Dp(16.0)),
            )
            .children(move || {
                let config = settings.get();
                let variant = if config.input {
                    ChipVariant::Input
                } else {
                    ChipVariant::Filter
                };
                let mut colors = ChipDefaults::colors(
                    &MaterialTheme::default().color_scheme,
                    variant,
                    ChipStyle::Flat,
                );
                colors.container_color = Color::RED;
                colors.selected_container_color = Color::RED;
                colors.disabled_container_color = Color::RED;
                colors.disabled_selected_container_color = Color::RED;
                colors.label_color = Color::RED;
                colors.selected_label_color = Color::RED;
                colors.disabled_label_color = Color::RED;
                colors.disabled_selected_label_color = Color::RED;
                chip()
                    .variant(variant)
                    .label("Shape sample")
                    .colors(colors)
                    .border(ChipBorder {
                        width: Dp::ZERO,
                        selected_width: Dp::ZERO,
                        color: Color::TRANSPARENT,
                        selected_color: Color::TRANSPARENT,
                        disabled_color: Color::TRANSPARENT,
                        disabled_selected_color: Color::TRANSPARENT,
                    })
                    .elevation(Dp::ZERO)
                    .modifier(Modifier::new().width(Dp(160.0)).semantics(SemanticsArgs {
                        test_tag: Some("shape_chip".into()),
                        ..Default::default()
                    }))
                    .selected(config.selected)
                    .enabled(!config.disabled)
                    .shape_style(if config.standard {
                        ChipShapeStyle::Standard
                    } else {
                        ChipShapeStyle::Expressive
                    })
                    .shape_optional(config.fixed.then_some(Shape::RECTANGLE))
                    .on_click(|| {});
            });
    });
}

fn bounds(renderer: &HeadlessRenderer) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut nodes = vec![&snapshot["root"]];
    while let Some(node) = nodes.pop() {
        if node["accessibility"]["key"] == "shape_chip" {
            return ["x", "y", "width", "height"]
                .map(|field| node["bounds"][field].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            nodes.extend(children);
        }
    }
    panic!("missing chip");
}

fn corners(renderer: &HeadlessRenderer) -> Vec<bool> {
    let [x, y, width, height] = bounds(renderer);
    assert_eq!((width, height), (160, 48));
    let data = renderer.read_pixels().unwrap();
    let mut samples = Vec::new();
    for (right, bottom) in [(false, false), (true, false), (false, true), (true, true)] {
        for (dx, dy) in [(2, 4), (4, 4), (0, 0)] {
            let px = x + if right { width - 1 - dx } else { dx };
            let py = y + if bottom { 39 - dy } else { 8 + dy };
            let offset = ((py as u32 * renderer.physical_size().width + px as u32) * 4) as usize;
            samples.push(data[offset + 1] < 128);
        }
    }
    samples
}

fn assert_shape(renderer: &HeadlessRenderer, expected: [bool; 3]) {
    assert_eq!(corners(renderer), expected.repeat(4));
}

fn check_shapes(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 220,
            height: 140,
            sample_count: samples,
            frame_time: Duration::from_millis(10),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    let settings = SETTINGS.get().expect("settings");
    for input in [false, true] {
        settings.set(Settings {
            input,
            ..Default::default()
        });
        renderer.render_frames(100);
        assert_shape(&renderer, [false, true, false]);
        let rest = corners(&renderer);
        settings.with_mut(|s| s.selected = true);
        renderer.step_frame();
        assert_eq!(corners(&renderer), rest);
        renderer.render_frames(100);
        assert_shape(&renderer, [false, false, false]);
        let point = PxPosition::from_f32_xy(96.0, 40.0);
        let outside = PxPosition::from_f32_xy(210.0, 130.0);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(2);
        assert_shape(&renderer, [false, false, false]);
        renderer.render_frames(100);
        assert_shape(&renderer, [true, true, false]);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
        renderer.render_frames(100);
        assert_shape(&renderer, [false, false, false]);
        settings.with_mut(|s| {
            s.standard = true;
        });
        renderer.render_frames(2);
        assert_shape(&renderer, [true, true, false]);
        settings.with_mut(|s| {
            s.fixed = true;
            s.standard = false;
        });
        renderer.render_frames(2);
        assert_shape(&renderer, [true, true, true]);
        settings.with_mut(|s| {
            s.fixed = false;
            s.disabled = true;
        });
        renderer.render_frames(2);
        assert_shape(&renderer, [false, false, false]);
        settings.with_mut(|s| s.selected = false);
        renderer.render_frames(2);
        assert_shape(&renderer, [false, true, false]);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_chip_selection_press_and_fixed_shapes_msaa1() {
    crate::test_utils::run_isolated_test(
        "chip::shape_tests::expressive_chip_selection_press_and_fixed_shapes_msaa1",
        || check_shapes(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn expressive_chip_selection_press_and_fixed_shapes_msaa4() {
    crate::test_utils::run_isolated_test(
        "chip::shape_tests::expressive_chip_selection_press_and_fixed_shapes_msaa4",
        || check_shapes(4),
    );
}
