//! Combined light and dark native sheet color diagrams.
//!
//! ## Usage
//!
//! Verify fractional panel seams, semantic color roles and real icon callbacks.

use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use tessera_components::{column::column, scrollable::ScrollableController};
use tessera_ui::{
    Color, CursorEventContent, GestureState, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    PointerChange, PressKeyEventType, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource,
    accesskit::{Node, Role},
    current_frame_time, scaled_box,
};

use super::*;

thread_local! {
    static CONFIG:Cell<(bool,bool)>=const{Cell::new((false,true))};
    pub(super) static GEOMETRY:RefCell<[Option<State<PreciseGeometry>>;2]>=const{RefCell::new([None,None])};
    pub(super) static NOTICE:Cell<Option<State<String>>>=const{Cell::new(None)};
    static SCROLL:Cell<Option<State<ScrollableController>>>=const{Cell::new(None)};
    static CALIBRATION: Cell<(f32, bool)> = const { Cell::new((0.625, false)) };
}

#[tessera]
fn close_calibration_fixture() {
    let (scale, modal) = CALIBRATION.get();
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
    CALIBRATION.set((scale, modal));
    let mut renderer = HeadlessRenderer::new(
        Box::new(close_calibration_fixture),
        vec![Box::new(Pipelines)],
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
fn fixture() {
    let scroll = remember(ScrollableController::new);
    SCROLL.set(Some(scroll));
    material_theme().child(move || {
        column()
            .modifier(
                at(10.25, 20.5)
                    .fill_max_width()
                    .height(Dp(600.0))
                    .vertical_scroll(scroll),
            )
            .children(|| {
                spacer().modifier(Modifier::new().height(Dp(160.0)));
                let (side, modal) = CONFIG.get();
                sheet_colors(side, modal);
                spacer().modifier(Modifier::new().height(Dp(700.0)));
            });
    });
}

struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("Missing {label} {role:?}"))
        .1
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
    if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
        tessera_ui::headless::write_png_file(
            std::path::PathBuf::from(directory)
                .join(format!("sheet-colors-{side}-{modal}-{width}.png")),
            width as u32,
            renderer.physical_size().height,
            &pixels,
        )
        .unwrap();
    }
    let light = GEOMETRY.with_borrow(|values| values[0].unwrap()).get()["figure"];
    let dark = GEOMETRY.with_borrow(|values| values[1].unwrap()).get()["figure"];
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
        let geometry = GEOMETRY
            .with_borrow(|values| values[dark as usize].unwrap())
            .get();
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
            assert!(ink.len() >= 2);
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
    }
}

fn assert_geometry(renderer: &HeadlessRenderer, side: bool, scale: f32) {
    for dark in [false, true] {
        let values = GEOMETRY
            .with_borrow(|values| values[dark as usize].unwrap())
            .get();
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
            CONFIG.set((side, modal));
            GEOMETRY.with_borrow_mut(|values| *values = [None, None]);
            NOTICE.set(None);
            let close_references = side.then(|| {
                [
                    calibrate_close(samples, modal, 640, 0.625),
                    calibrate_close(samples, modal, 512, 0.5),
                ]
            });
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(Pipelines)],
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
                SCROLL
                    .get()
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
                    NOTICE.get().unwrap().set(String::new());
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
                    assert_eq!(NOTICE.get().unwrap().get(), "Sheet close action selected");
                }
            }
        }
    }
}
