use std::time::Duration;

use tessera_components::{column::column, scrollable::ScrollableController};
use tessera_ui::{
    Color, CursorEventContent, GestureState, MOUSE_POINTER_ID, PointerChange, PressKeyEventType,
    ScrollDeltaUnit, ScrollEventContent, ScrollEventSource,
    accesskit::{Node, Role},
    current_frame_time,
    headless::{HeadlessConfig, HeadlessRenderer},
    scaled_box,
    testing::TestProbe,
};

use super::*;

#[derive(Clone, Copy, Default, PartialEq)]
struct FixtureHandles {
    scroll: Option<State<ScrollableController>>,
    notice: Option<State<String>>,
}

fn geometry(renderer: &HeadlessRenderer, dark: bool) -> PreciseGeometry {
    let snapshot = renderer.inspect_components();
    [
        "figure",
        "container",
        "divider",
        "back",
        "headline",
        "close",
        "scrim",
        "handle",
    ]
    .into_iter()
    .filter_map(|name| {
        let tag = format!("sheet-color-{}-{name}", if dark { "dark" } else { "light" });
        snapshot
            .find_tag(&tag)
            .and_then(|node| node.layout_coordinates())
            .map(|value| (name, value))
    })
    .collect()
}

#[tessera]
fn close_calibration_fixture(scale: f32, modal: bool) {
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(at(10.25, 80.5))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(Modifier::new().size(Dp(1024.0), Dp(881.0)))
                    .children(move || {
                        for dark in [false, true] {
                            material_theme()
                                .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                                .child(move || {
                                    let scheme =
                                        MaterialTheme::from_seed(ACCENT, dark).color_scheme;
                                    boxed()
                                        .alignment(Alignment::TopStart)
                                        .modifier(
                                            at(872.0, 126.0 + if dark { 440.5 } else { 0.0 })
                                                .size(Dp(48.0), Dp(48.0))
                                                .background(if modal {
                                                    scheme.surface_container_low
                                                } else {
                                                    scheme.surface
                                                }),
                                        )
                                        .children(move || {
                                            icon_button()
                                                .standard()
                                                .icon(filled::CLOSE_SVG)
                                                .content_color(scheme.on_surface_variant)
                                                .accessibility_label(if dark {
                                                    "Dark reference close"
                                                } else {
                                                    "Light reference close"
                                                })
                                                .on_click(|| {});
                                        });
                                });
                        }
                    });
            });
        });
}

struct CloseReference {
    width: u32,
    ink: [Vec<(usize, [u8; 4])>; 2],
}

fn calibrate_close(samples: u32, modal: bool, width: u32, scale: f32) -> CloseReference {
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            close_calibration_fixture(scale, modal);
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width,
            height: 1000,
            sample_count: samples,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("Native GPU adapter for independent icon calibration");
    renderer.render_frames(8);
    let pixels = renderer.read_pixels().unwrap();
    let mut ink = [Vec::new(), Vec::new()];
    for dark in [false, true] {
        let bounds = node(
            &renderer,
            if dark {
                "Dark reference close"
            } else {
                "Light reference close"
            },
            Role::Button,
        )
        .bounds()
        .unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
            (
                10.25 + 872.0 * f64::from(scale),
                80.5 + (126.0 + if dark { 440.5 } else { 0.0 }) * f64::from(scale),
                48.0 * f64::from(scale),
                48.0 * f64::from(scale),
            ),
        );
        let scheme = MaterialTheme::from_seed(ACCENT, dark).color_scheme;
        let background = color_bytes(if modal {
            scheme.surface_container_low
        } else {
            scheme.surface
        });
        let left = (bounds.x0 + 15.0 * f64::from(scale)).ceil() as usize;
        let right = (bounds.x0 + 33.0 * f64::from(scale)).floor() as usize;
        let top = (bounds.y0 + 15.0 * f64::from(scale)).ceil() as usize;
        let bottom = (bounds.y0 + 33.0 * f64::from(scale)).floor() as usize;
        for y in top..bottom {
            for x in left..right {
                let i = (y * width as usize + x) * 4;
                let rgba: [u8; 4] = pixels[i..i + 4].try_into().unwrap();
                if (0..3)
                    .any(|channel| (rgba[channel] as i16 - background[channel] as i16).abs() >= 8)
                {
                    ink[dark as usize].push((i, rgba));
                }
            }
        }
        assert!(
            ink[dark as usize].len() >= 2,
            "Independent close icon must supply real {} ink at scale{scale}",
            if dark { "dark" } else { "light" }
        );
    }
    drop(renderer);
    CloseReference { width, ink }
}

#[tessera]
fn fixture(side: bool, modal: bool, probe: TestProbe<FixtureHandles>) {
    let scroll = remember(ScrollableController::new);
    let notice = remember(String::new);
    probe.set(FixtureHandles {
        scroll: Some(scroll),
        notice: Some(notice),
    });
    material_theme().child(move || {
        column()
            .modifier(
                at(10.25, 20.5)
                    .fill_max_width()
                    .height(Dp(600.0))
                    .vertical_scroll(scroll),
            )
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(160.0)));
                sheet_colors(side, modal).notice(notice);
                spacer().modifier(Modifier::new().height(Dp(700.0)));
            });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    snapshot
        .find(&crate::res::I18N.text(label), role)
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
        .1
        .clone()
}

fn color_bytes(color: Color) -> [u8; 4] {
    let channel = |linear: f32| {
        let encoded = if linear <= 0.0031308 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        (encoded * 255.0).round() as u8
    };
    [
        channel(color.r),
        channel(color.g),
        channel(color.b),
        (color.a * 255.0).round() as u8,
    ]
}

fn assert_pixels(
    renderer: &HeadlessRenderer,
    side: bool,
    modal: bool,
    close: Option<&CloseReference>,
) {
    let pixels = renderer.read_pixels().unwrap();
    let width = renderer.physical_size().width as usize;
    let light = geometry(renderer, false)["figure"];
    let dark = geometry(renderer, true)["figure"];
    let x = light
        .local_to_window
        .map_point(PxPosition::from_f32_xy(28.0, 0.0))
        .x
        .floor() as usize;
    let seam = dark.local_to_window.translation().y;
    for (dark, y) in [(false, (seam - 2.0).floor()), (true, (seam + 2.0).ceil())] {
        let expected = color_bytes(MaterialTheme::from_seed(ACCENT, dark).color_scheme.surface);
        let i = (y as usize * width + x) * 4;
        for channel in 0..4 {
            assert!(
                (pixels[i + channel] as i16 - expected[channel] as i16).abs() <= 2,
                "Panel seam must paint the corresponding {} background: {:?} versus {expected:?}",
                if dark { "dark" } else { "light" },
                &pixels[i..i + 4]
            );
        }
    }
    for dark in [false, true] {
        let scheme = MaterialTheme::from_seed(ACCENT, dark).color_scheme;
        let geometry = geometry(renderer, dark);
        let root = geometry["figure"];
        if side {
            let bounds = node(
                renderer,
                &format!("{} sheet color close", if dark { "Dark" } else { "Light" }),
                Role::Button,
            )
            .bounds()
            .unwrap();
            assert_eq!(
                (bounds.x0, bounds.y0),
                (
                    10.25 + 872.0 * f64::from(root.local_to_window.scale()),
                    80.5 + (126.0 + if dark { 440.5 } else { 0.0 })
                        * f64::from(root.local_to_window.scale()),
                ),
            );
            let reference = close.expect("Side sheet close icon calibration");
            assert_eq!(reference.width as usize, width);
            let ink = &reference.ink[dark as usize];
            for &(i, expected) in ink {
                for channel in 0..4 {
                    assert!(
                        (pixels[i + channel] as i16 - expected[channel] as i16).abs() <= 2,
                        "Sheet close role wiring must match the independent {} IconButton at ({},{}), channel{channel}: actual{:?}, reference{expected:?}, modal{modal}, width{width}",
                        if dark { "dark" } else { "light" },
                        i / 4 % width,
                        i / 4 / width,
                        &pixels[i..i + 4]
                    );
                }
            }
        }
        let point = root.local_to_window.map_point(PxPosition::from_f32_xy(
            if side { 700.0 } else { 500.0 },
            300.0,
        ));
        let i = (point.y.floor() as usize * width + point.x.floor() as usize) * 4;
        let expected = color_bytes(if side && !modal {
            scheme.surface
        } else {
            scheme.surface_container_low
        });
        for channel in 0..4 {
            assert!(
                (pixels[i + channel] as i16 - expected[channel] as i16).abs() <= 2,
                "Native {} sheet container role channel {channel}: actual {:?}, expected {expected:?}",
                if dark { "dark" } else { "light" },
                &pixels[i..i + 4]
            );
        }
        if !side {
            let pixel = |x: f32, y: f32| {
                let point = root
                    .local_to_window
                    .map_point(PxPosition::from_f32_xy(x, y));
                let i = (point.y.floor() as usize * width + point.x.floor() as usize) * 4;
                [pixels[i], pixels[i + 1], pixels[i + 2]]
            };
            let frame = pixel(302.0, 250.0);
            let outline = color_bytes(scheme.outline_variant);
            assert!(
                (0..3).all(|c| (frame[c] as i16 - outline[c] as i16).abs() <= 3),
                "The {} phone frame must reach the sheet: {frame:?} versus {outline:?}",
                if dark { "dark" } else { "light" }
            );
            let luminance = |rgb: [u8; 3]| rgb.iter().map(|&c| i32::from(c)).sum::<i32>();
            let surface = color_bytes(scheme.surface);
            let scrim = pixel(400.0, 60.0);
            assert!(
                luminance(scrim) + 6 < luminance([surface[0], surface[1], surface[2]]),
                "The {} scrim must dim the screen above the sheet: {scrim:?} versus {surface:?}",
                if dark { "dark" } else { "light" }
            );
        }
    }
}

fn assert_geometry(renderer: &HeadlessRenderer, side: bool, scale: f32) {
    for dark in [false, true] {
        let values = geometry(renderer, dark);
        let root = values["figure"];
        assert_eq!(root.local_to_window.scale(), scale);
        let half = if side { 440.5 } else { 502.5 };
        assert_eq!(
            (
                root.local_to_window.translation().x,
                root.local_to_window.translation().y
            ),
            (10.25, 80.5 + if dark { half * scale } else { 0.0 })
        );
        let container = annotations::relative_precise(&values, "container").unwrap();
        let expected = if side {
            (612.0, 114.0, 320.0)
        } else {
            (306.0, 131.0, 412.0)
        };
        assert_eq!((container.x, container.y, container.w), expected);
        let bounds = node(
            renderer,
            &format!(
                "{} {} sheet color marker 3",
                if dark { "Dark" } else { "Light" },
                if side { "side" } else { "bottom" }
            ),
            Role::Note,
        )
        .bounds()
        .unwrap();
        assert_eq!(
            (bounds.width(), bounds.height()),
            (24.0 * scale as f64, 32.0 * scale as f64)
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter for combined sheet color diagrams and physical clicks"]
fn sheet_color_pairs_keep_fractional_seams_roles_and_callbacks_when_scrolled_and_resized() {
    for samples in [1, 4] {
        for (side, modal) in [(false, true), (true, false), (true, true)] {
            let probe = TestProbe::<FixtureHandles>::default();
            let observed = probe.clone();
            let close_references = side.then(|| {
                [
                    calibrate_close(samples, modal, 640, 0.625),
                    calibrate_close(samples, modal, 512, 0.5),
                ]
            });
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || {
                    fixture(side, modal, probe.clone());
                }),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width: 640,
                    height: 1000,
                    sample_count: samples,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .expect("Native GPU adapter");
            renderer.render_frames(16);
            renderer.inject_pointer(
                PointerChange::new(
                    current_frame_time().unwrap(),
                    MOUSE_POINTER_ID,
                    CursorEventContent::Scroll(ScrollEventContent {
                        delta_x: 0.0,
                        delta_y: -100.0,
                        unit: ScrollDeltaUnit::Pixel,
                        source: ScrollEventSource::Wheel,
                    }),
                    GestureState::default(),
                )
                .with_position(PxPosition::from_f32_xy(40.0, 100.0)),
            );
            renderer.render_frames(120);
            assert_eq!(
                observed
                    .get()
                    .scroll
                    .unwrap()
                    .with(|scroll| scroll.child_position().y),
                -100.0
            );
            assert_geometry(&renderer, side, 0.625);
            assert_pixels(
                &renderer,
                side,
                modal,
                close_references.as_ref().map(|values| &values[0]),
            );
            renderer.resize(512, 1000);
            renderer.render_frames(16);
            assert_geometry(&renderer, side, 0.5);
            assert_pixels(
                &renderer,
                side,
                modal,
                close_references.as_ref().map(|values| &values[1]),
            );
            if side {
                for dark in [false, true] {
                    observed.get().notice.unwrap().set(String::new());
                    let bounds = node(
                        &renderer,
                        &format!("{} sheet color close", if dark { "Dark" } else { "Light" }),
                        Role::Button,
                    )
                    .bounds()
                    .unwrap();
                    assert_eq!((bounds.width(), bounds.height()), (24.0, 24.0));
                    let point = PxPosition::from_f32_xy(
                        ((bounds.x0 + bounds.x1) / 2.0) as f32,
                        ((bounds.y0 + bounds.y1) / 2.0) as f32,
                    );
                    renderer.pointer_move(MOUSE_POINTER_ID, point);
                    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
                    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
                    renderer.render_frames(4);
                    assert_eq!(
                        I18N.text(observed.get().notice.unwrap().get()),
                        "Sheet close action selected"
                    );
                }
            }
        }
    }
}
