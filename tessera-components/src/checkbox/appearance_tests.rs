//! Checkbox geometry and interactive outline regressions.
//!
//! ## Usage
//!
//! Preserve selection strokes and state colors across display densities.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    PressKeyEventType, PxPosition, State, remember, tessera,
};

use crate::{
    checkmark::checkmark,
    column::column,
    theme::{MaterialTheme, material_theme},
};

use super::checkbox;

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    checked: bool,
    mixed: bool,
    disabled: bool,
    custom: bool,
    tiny_stroke: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.primary = Color::BLACK;
            theme.color_scheme.on_primary = Color::WHITE;
            theme.color_scheme.on_surface = Color::BLUE;
            theme.color_scheme.on_surface_variant = Color::RED;
            theme.color_scheme.surface = Color::WHITE;
            theme
        })
        .child(move || {
            column().children(move || {
                let value = settings.get();
                if value.tiny_stroke {
                    checkmark()
                        .color(Color::WHITE)
                        .size(Dp(18.0))
                        .stroke_width(Dp(2.0).to_pixels_f32().floor())
                        .progress(0.0000000001)
                        .checkbox_geometry(true);
                    return;
                }
                checkbox()
                    .checked(value.checked)
                    .indeterminate(value.mixed)
                    .disabled(value.disabled)
                    .color_optional(value.custom.then_some(Color::GREEN))
                    .accessibility_label("Selection");
            });
        });
}

fn check_appearance(samples: u32) {
    for density in [2.0, 1.0, 1.25, 1.5] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 160,
                height: 160,
                scale_factor: density,
                sample_count: samples,
                ..Default::default()
            },
        )
        .expect("renderer");
        renderer.render_frames(30);
        let settings = SETTINGS.get().unwrap();
        settings.with_mut(|value| value.mixed = true);
        renderer.render_frames(30);
        let bounds = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some("Selection"))
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(bounds.x1 - bounds.x0, 48.0 * density);
        assert_eq!(bounds.y1 - bounds.y0, 48.0 * density);
        let size = (18.0 * density) as u32;
        let left = bounds.x0 as u32 + ((bounds.x1 - bounds.x0) as u32 - size) / 2;
        let top = bounds.y0 as u32 + ((bounds.y1 - bounds.y0) as u32 - size) / 2;
        let cx = left + size / 2;
        let cy = top + size / 2;
        let width = renderer.physical_size().width;
        let pixels = renderer.read_pixels().unwrap();
        let bright_rows = (top..top + size)
            .filter(|y| pixels[((y * width + cx) * 4) as usize] > 220)
            .count();
        if density == 1.5 {
            let visible_rows = (top..top + size)
                .filter(|y| pixels[((y * width + cx) * 4) as usize] > 20)
                .count();
            assert_eq!(
                (bright_rows, visible_rows),
                (2, 4),
                "3px dash edge coverage"
            );
        } else {
            assert_eq!(
                bright_rows,
                (2.0 * density) as usize,
                "2dp dash at {density}"
            );
        }
        if density == 2.0 {
            settings.with_mut(|value| {
                value.mixed = false;
                value.checked = true;
            });
            renderer.render_frames(30);
            let pixels = renderer.read_pixels().unwrap();
            let pixel = |x: u32, y: u32| pixels[((y * width + x) * 4) as usize];
            assert!(
                pixel(cx - 3, cy + 5) > 220,
                "the check joint must be near (40%, 65%)"
            );
            assert!(
                pixel(cx - 1, cy + 10) < 80,
                "the old low joint must remain unpainted"
            );
        }
        settings.set(Settings::default());
        FocusManager::current().clear_focus(true);
        renderer.render_frames(30);
        let edge = (left + 1, cy);
        let is_color = |pixels: &[u8], channel: usize| {
            let start = ((edge.1 * width + edge.0) * 4) as usize;
            pixels[start + 3] > 128
                && pixels[start + channel] > 180
                && pixels[start + channel].saturating_sub(pixels[start + (channel + 1) % 3]) > 90
                && pixels[start + channel].saturating_sub(pixels[start + (channel + 2) % 3]) > 90
        };
        let pixels = renderer.read_pixels().unwrap();
        let edge_offset = ((edge.1 * width + edge.0) * 4) as usize;
        assert!(
            is_color(&pixels, 0),
            "resting outline at {density}: {:?}, bounds {bounds:?}",
            &pixels[edge_offset..edge_offset + 4]
        );
        if density == 1.25 {
            let inner_offset = ((cy * width + left + 2) * 4) as usize;
            assert_eq!(
                (pixels[edge_offset + 3], pixels[inner_offset + 3]),
                (255, 0),
                "the floored 2px outline must fully cover its interior pixel and leave the next inner pixel clear at density 1.25"
            );
        }
        let point = PxPosition::from_f32_xy(cx as f32, cy as f32);
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.render_frames(30);
        assert!(
            is_color(&renderer.read_pixels().unwrap(), 2),
            "hover outline"
        );
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(30);
        assert!(
            is_color(&renderer.read_pixels().unwrap(), 2),
            "pressed outline"
        );
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(150.0, 150.0));
        renderer.pointer_release(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(150.0, 150.0),
            PressKeyEventType::Left,
        );
        renderer.render_frames(30);
        FocusManager::current().clear_focus(true);
        renderer.render_frames(30);
        let pixels = renderer.read_pixels().unwrap();
        assert!(
            is_color(&pixels, 0),
            "restored outline at {density}: {:?}, focus {:?}",
            &pixels[edge_offset..edge_offset + 4],
            renderer.accessibility_snapshot().unwrap().focus,
        );
        FocusManager::current().request_move_focus(FocusDirection::Next);
        renderer.render_frames(30);
        assert!(
            is_color(&renderer.read_pixels().unwrap(), 2),
            "focused outline"
        );
        settings.with_mut(|value| value.custom = true);
        renderer.render_frames(30);
        let pixels = renderer.read_pixels().unwrap();
        assert!(
            is_color(&pixels, 1),
            "explicit outline color takes precedence at {density}: {:?}",
            &pixels[edge_offset..edge_offset + 4],
        );
        settings.with_mut(|value| value.tiny_stroke = true);
        renderer.render_frames(30);
        let pixels = renderer.read_pixels().unwrap();
        let painted_pixels = pixels.chunks_exact(4).filter(|pixel| pixel[3] > 20).count();
        assert!(
            painted_pixels < 40,
            "a subpixel segment must not fill its quad: {painted_pixels}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_density_geometry_and_interaction_colors_msaa1() {
    crate::test_utils::run_isolated_test(
        "checkbox::appearance_tests::checkbox_density_geometry_and_interaction_colors_msaa1",
        || check_appearance(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_density_geometry_and_interaction_colors_msaa4() {
    crate::test_utils::run_isolated_test(
        "checkbox::appearance_tests::checkbox_density_geometry_and_interaction_colors_msaa4",
        || check_appearance(4),
    );
}
