//! Interactive measurement-diagram layout regressions.
//!
//! ## Usage
//!
//! Keep native control bounds and target annotations aligned when resized.

use std::{cell::Cell, time::Duration};

use tessera_components::theme::material_theme;
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Role},
    tessera,
};

thread_local! {static SCENE: Cell<usize> = const {Cell::new(0)};}

struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| match SCENE.get() {
        0 => {
            super::sizes();
        }
        1 => {
            super::targets();
        }
        _ => {
            super::corners();
        }
    });
}

fn control(renderer: &HeadlessRenderer, label: &str) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing native measurement control {label}"))
        .1
}

#[test]
#[ignore = "requires a native GPU adapter for actual measurement diagrams"]
fn measurement_diagrams_preserve_native_sizes_expanded_targets_and_clicks_after_resize() {
    for scene in 0..3 {
        SCENE.set(scene);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1024,
                height: 1400,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        for width in [1024, 390, 1024] {
            renderer.resize(width, 1400);
            renderer.render_frames(16);
            let scale = f64::from(width) / if scene == 1 { 676.0 } else { 1024.0 };
            let pixels = renderer.read_pixels().unwrap();
            let pixel = |x: f64, y: f64| -> [u8; 3] {
                let offset = ((y.floor() as u32 * width + x.floor() as u32) * 4) as usize;
                pixels[offset..offset + 3].try_into().unwrap()
            };
            let difference = |a: [u8; 3], b: [u8; 3]| -> u16 {
                a.into_iter()
                    .zip(b)
                    .map(|(a, b)| u16::from(a.abs_diff(b)))
                    .sum()
            };
            if scene == 0 {
                for (name, expected_width, expected_height) in [
                    ("Measure ExtraSmall Narrow", 48.0, 48.0),
                    ("Measure Small Wide", 52.0, 48.0),
                    ("Measure Medium Uniform", 56.0, 56.0),
                    ("Measure Large Narrow", 64.0, 96.0),
                    ("Measure ExtraLarge Wide", 184.0, 136.0),
                ] {
                    let bounds = control(&renderer, name).bounds().unwrap();
                    assert!(
                        (bounds.width() - expected_width * scale).abs() <= 1.0,
                        "{name} width at{width}: {bounds:?}"
                    );
                    assert!(
                        (bounds.height() - expected_height * scale).abs() <= 1.0,
                        "{name} height at{width}: {bounds:?}"
                    );
                }
                // A uniform round control is a circle, not a superellipse.
                // At 1/8 of both edges this sample lies outside the circle
                // but inside a G2=3 superellipse. Regular corners may use G2=3.
                let bounds = control(&renderer, "Measure ExtraLarge Uniform")
                    .bounds()
                    .unwrap();
                let background = pixel(bounds.x0 + 1.0, bounds.y0 + 1.0);
                let corner = pixel(
                    bounds.x0 + bounds.width() / 8.0,
                    bounds.y0 + bounds.height() / 8.0,
                );
                let fill = pixel(
                    (bounds.x0 + bounds.x1) / 2.0,
                    bounds.y0 + bounds.height() / 8.0,
                );
                assert!(
                    difference(background, fill) > 200,
                    "filled control missing at {width}"
                );
                assert!(
                    difference(background, corner) < 80,
                    "capsule acquired superellipse corners at {width}"
                );
            }
            if scene == 2 && width == 1024 {
                let bounds = control(&renderer, "Corner row 0 ExtraLarge")
                    .bounds()
                    .unwrap();
                let y = (bounds.y0 + bounds.y1) / 2.0;
                let background = pixel(bounds.x0 + 4.5, y);
                let rule = pixel(bounds.x0 + 0.5, y);
                let interior = pixel(bounds.x0 + 2.5, y);
                assert!(difference(rule, background) > 30, "thin outline missing");
                assert!(
                    difference(interior, background) < 30,
                    "diagram inherited the XL three-dp component outline"
                );
            }
            let name = match scene {
                0 => "Measure ExtraSmall Uniform",
                1 => "Target ExtraSmall Narrow",
                _ => "Corner row 1 Small",
            };
            let before = control(&renderer, name);
            let bounds = before.bounds().unwrap();
            assert!((bounds.width() - 48.0 * scale).abs() <= 1.0);
            assert!((bounds.height() - 48.0 * scale).abs() <= 1.0);
            let position = PxPosition::from_f32_xy(
                (bounds.x0 + 2.0 * scale) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, position);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.render_frames(4);
            assert_ne!(
                control(&renderer, name).description(),
                before.description(),
                "expanded target failed: scene={scene}, width={width}, bounds={bounds:?}, point={position:?}"
            );
            if let Ok(directory) = std::env::var("TESSERA_ICON_MEASUREMENT_OUTPUT") {
                let path = std::path::Path::new(&directory)
                    .join(format!("measurements-{scene}-{width}.png"));
                tessera_ui::headless::write_png_file(
                    &path,
                    width,
                    1400,
                    &renderer.read_pixels().unwrap(),
                )
                .unwrap();
            }
        }
    }
}
