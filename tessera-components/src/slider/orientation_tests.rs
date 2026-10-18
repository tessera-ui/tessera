use std::cell::Cell;

use tessera_ui::{
    Color, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PlacementModifierNode, PressKeyEventType, PxPosition, State,
    accesskit::{Orientation, Rect, Role},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
};

use crate::{
    boxed::boxed,
    modifier::ModifierExt as _,
    theme::{MaterialTheme, material_theme},
};

use super::{SliderOrientation, SliderSize, centered_slider, range_slider, slider};

const OVERRIDE_ACTIVE: Color = Color::new(0.0, 1.0, 1.0, 1.0);
const OVERRIDE_INACTIVE: Color = Color::new(1.0, 0.0, 1.0, 1.0);
const OVERRIDE_ACTIVE_TICK: Color = Color::new(1.0, 0.3, 0.0, 1.0);
const OVERRIDE_INACTIVE_TICK: Color = Color::new(0.0, 0.2, 1.0, 1.0);
const OVERRIDE_THUMB: Color = Color::new(1.0, 1.0, 0.0, 1.0);

#[derive(Clone, Copy, Debug, Default)]
enum Kind {
    #[default]
    Single,
    Centered,
    Range,
}

#[derive(Clone, Copy, Debug, Default)]
enum Palette {
    #[default]
    Light,
    Dark,
    Contrast,
}

impl Palette {
    fn theme(self) -> MaterialTheme {
        match self {
            Self::Light => MaterialTheme::default(),
            Self::Dark => MaterialTheme::from_seed(Color::BLUE, true),
            Self::Contrast => {
                let mut theme = MaterialTheme::default();
                theme.color_scheme.primary = Color::RED;
                theme.color_scheme.secondary_container = Color::GREEN;
                theme.color_scheme.surface_variant = Color::BLUE;
                theme.color_scheme.on_surface = Color::new(0.0, 0.3, 0.8, 1.0);
                theme.color_scheme.surface = Color::new(0.2, 0.2, 0.2, 1.0);
                theme
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin(PxPosition);

impl PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + self.0
    }
}

#[derive(Clone, Copy, Default)]
struct Settings {
    kind: Kind,
    size: SliderSize,
    orientation: SliderOrientation,
    reversed: bool,
    disabled: bool,
    steps: usize,
    length: Option<f64>,
    inset_icon: bool,
    origin: Option<(f64, f64)>,
    palette: Palette,
    background: Option<Color>,
    overrides: bool,
    override_ticks: bool,
    references: bool,
    stops: Option<bool>,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static VALUE: Cell<Option<State<f32>>> = const { Cell::new(None) };
    static RANGE: Cell<Option<State<(f32, f32)>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let value = remember(|| 0.25_f32);
    let range = remember(|| (0.25_f32, 0.75_f32));
    SETTINGS.set(Some(settings));
    VALUE.set(Some(value));
    RANGE.set(Some(range));
    material_theme()
        .theme(move || settings.with(|settings| settings.palette.theme()))
        .child(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().background(
                    settings.with(|settings| settings.background.unwrap_or(Color::WHITE)),
                ))
                .children(move || {
                    let settings = settings.get();
                    let modifier = if let Some((x, y)) = settings.origin {
                        Modifier::new().push_placement(FractionalOrigin(PxPosition::from_f32_xy(
                            super::Dp(x).to_pixels_f32(),
                            super::Dp(y).to_pixels_f32(),
                        )))
                    } else {
                        Modifier::new().offset(super::Dp(40.0), super::Dp(40.0))
                    };
                    let modifier = match settings.orientation {
                        SliderOrientation::Horizontal => {
                            modifier.width(super::Dp(settings.length.unwrap_or(280.0)))
                        }
                        SliderOrientation::Vertical => {
                            modifier.height(super::Dp(settings.length.unwrap_or(280.0)))
                        }
                    };
                    match settings.kind {
                        Kind::Single => {
                            slider()
                                .modifier(modifier)
                                .value(value.get())
                                .on_change(move |next| value.set(next))
                                .size(settings.size)
                                .inset_icon_optional(settings.inset_icon.then(|| {
                                    crate::material_icons::filled::ARROW_UPWARD_SVG.into()
                                }))
                                .orientation(settings.orientation)
                                .reverse_direction(settings.reversed)
                                .disabled(settings.disabled)
                                .steps(settings.steps)
                                .active_track_color_optional(
                                    settings.overrides.then_some(OVERRIDE_ACTIVE),
                                )
                                .inactive_track_color_optional(
                                    settings.overrides.then_some(OVERRIDE_INACTIVE),
                                )
                                .thumb_color_optional(settings.overrides.then_some(OVERRIDE_THUMB))
                                .active_tick_color_optional(
                                    settings.override_ticks.then_some(OVERRIDE_ACTIVE_TICK),
                                )
                                .inactive_tick_color_optional(
                                    settings.override_ticks.then_some(OVERRIDE_INACTIVE_TICK),
                                )
                                .show_stop_indicator_optional(settings.stops)
                                .accessibility_label("Value");
                        }
                        Kind::Centered => {
                            centered_slider()
                                .modifier(modifier)
                                .value(value.get())
                                .on_change(move |next| value.set(next))
                                .size(settings.size)
                                .orientation(settings.orientation)
                                .reverse_direction(settings.reversed)
                                .disabled(settings.disabled)
                                .steps(settings.steps)
                                .active_track_color_optional(
                                    settings.overrides.then_some(OVERRIDE_ACTIVE),
                                )
                                .inactive_track_color_optional(
                                    settings.overrides.then_some(OVERRIDE_INACTIVE),
                                )
                                .thumb_color_optional(settings.overrides.then_some(OVERRIDE_THUMB))
                                .active_tick_color_optional(
                                    settings.override_ticks.then_some(OVERRIDE_ACTIVE_TICK),
                                )
                                .inactive_tick_color_optional(
                                    settings.override_ticks.then_some(OVERRIDE_INACTIVE_TICK),
                                )
                                .show_stop_indicator_optional(settings.stops)
                                .accessibility_label("Value");
                        }
                        Kind::Range => {
                            range_slider()
                                .modifier(modifier)
                                .value(range.get())
                                .on_change(move |next| range.set(next))
                                .size(settings.size)
                                .orientation(settings.orientation)
                                .reverse_direction(settings.reversed)
                                .disabled(settings.disabled)
                                .steps(settings.steps)
                                .active_track_color_optional(
                                    settings.overrides.then_some(OVERRIDE_ACTIVE),
                                )
                                .inactive_track_color_optional(
                                    settings.overrides.then_some(OVERRIDE_INACTIVE),
                                )
                                .thumb_color_optional(settings.overrides.then_some(OVERRIDE_THUMB))
                                .active_tick_color_optional(
                                    settings.override_ticks.then_some(OVERRIDE_ACTIVE_TICK),
                                )
                                .inactive_tick_color_optional(
                                    settings.override_ticks.then_some(OVERRIDE_INACTIVE_TICK),
                                )
                                .show_stop_indicator_optional(settings.stops)
                                .accessibility_label("Interval");
                        }
                    }
                    if settings.references {
                        let scheme = settings.palette.theme().color_scheme;
                        let active = if settings.overrides {
                            OVERRIDE_ACTIVE
                        } else {
                            scheme.primary
                        };
                        let inactive = if settings.overrides {
                            OVERRIDE_INACTIVE
                        } else {
                            scheme.secondary_container
                        };
                        let thumb = if settings.overrides {
                            OVERRIDE_THUMB
                        } else {
                            scheme.primary
                        };
                        for (index, layers) in [
                            vec![active],
                            vec![inactive],
                            vec![thumb],
                            vec![if settings.override_ticks {
                                OVERRIDE_ACTIVE_TICK
                            } else {
                                scheme.on_primary
                            }],
                            vec![if settings.override_ticks {
                                OVERRIDE_INACTIVE_TICK
                            } else {
                                scheme.on_secondary_container
                            }],
                            vec![scheme.on_surface.with_alpha(0.38)],
                            vec![scheme.on_surface.with_alpha(0.12)],
                            vec![scheme.surface, scheme.on_surface.with_alpha(0.38)],
                            vec![
                                scheme.on_surface.with_alpha(0.38),
                                scheme.on_surface.with_alpha(0.12),
                            ],
                            vec![
                                scheme.on_surface.with_alpha(0.12),
                                scheme.on_surface.with_alpha(0.38),
                            ],
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .offset(
                                            super::Dp(200.0 + (index % 5) as f64 * 20.0),
                                            super::Dp(200.0 + (index / 5) as f64 * 20.0),
                                        )
                                        .size(super::Dp(10.0), super::Dp(10.0)),
                                )
                                .children(move || {
                                    for color in &layers {
                                        boxed().modifier(
                                            Modifier::new().fill_max_size().background(*color),
                                        );
                                    }
                                });
                        }
                    }
                });
        });
}

fn control(renderer: &HeadlessRenderer, label: &str) -> tessera_ui::accesskit::Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .expect("control")
        .1
}

fn click(renderer: &mut HeadlessRenderer, x: f64, y: f64) {
    let position = PxPosition::from_f32_xy(x as f32, y as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(2);
}

fn drag(renderer: &mut HeadlessRenderer, from: PxPosition, to: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, from);
    renderer.pointer_press(MOUSE_POINTER_ID, from, PressKeyEventType::Left);
    renderer.step_frame();
    for fraction in [0.25, 0.5, 0.75, 1.0] {
        renderer.pointer_move(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(
                from.x + (to.x - from.x) * fraction,
                from.y + (to.y - from.y) * fraction,
            ),
        );
        renderer.step_frame();
    }
    renderer.pointer_release(MOUSE_POINTER_ID, to, PressKeyEventType::Left);
    renderer.render_frames(2);
}

fn check_orientation(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 500,
            height: 500,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(2);
    let settings = SETTINGS.get().unwrap();
    for kind in [Kind::Single, Kind::Centered, Kind::Range] {
        for size in [
            SliderSize::ExtraSmall,
            SliderSize::Small,
            SliderSize::Medium,
            SliderSize::Large,
            SliderSize::ExtraLarge,
        ] {
            settings.set(Settings {
                kind,
                size,
                ..Default::default()
            });
            renderer.render_frames(2);
            let label = if matches!(kind, Kind::Range) {
                "Interval"
            } else {
                "Value"
            };
            let horizontal = control(&renderer, label).bounds().unwrap();
            let original = renderer.read_pixels().unwrap();
            let handle_ids: Vec<_> = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .filter(|(_, node)| node.role() == Role::Slider)
                .map(|(id, _)| id)
                .collect();
            for reversed in [false, true] {
                settings.with_mut(|settings| {
                    settings.orientation = SliderOrientation::Vertical;
                    settings.reversed = reversed;
                });
                renderer.render_frames(2);
                let vertical = control(&renderer, label).bounds().unwrap();
                assert_eq!(vertical.width(), horizontal.height());
                assert_eq!(vertical.height(), horizontal.width());
                let actual = renderer.read_pixels().unwrap();
                let mut visible = 0;
                for y in vertical.y0 as usize..vertical.y1 as usize {
                    for x in vertical.x0 as usize..vertical.x1 as usize {
                        let main = y - vertical.y0 as usize;
                        let hx = horizontal.x0 as usize
                            + if reversed {
                                horizontal.width() as usize - 1 - main
                            } else {
                                main
                            };
                        let hy = horizontal.y0 as usize + x - vertical.x0 as usize;
                        let actual_offset = (y * 500 + x) * 4;
                        let expected_offset = (hy * 500 + hx) * 4;
                        for channel in 0..4 {
                            let diff = actual[actual_offset + channel]
                                .abs_diff(original[expected_offset + channel]);
                            assert!(
                                diff <= 3,
                                "orientation pixel at {x},{y}, {kind:?}, {size:?}, reversed={reversed}, channel={channel}: {} != {}",
                                actual[actual_offset + channel],
                                original[expected_offset + channel]
                            );
                        }
                        visible += usize::from(actual[actual_offset] < 250);
                    }
                }
                assert!(
                    visible > 500,
                    "the visible track and handle must be compared"
                );
                for (_, node) in renderer.accessibility_snapshot().unwrap().nodes {
                    if node.role() == Role::Slider {
                        assert_eq!(node.orientation(), Some(Orientation::Vertical));
                    }
                }
                assert_eq!(
                    renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .filter(|(_, node)| node.role() == Role::Slider)
                        .map(|(id, _)| id)
                        .collect::<Vec<_>>(),
                    handle_ids,
                    "axis changes preserve handle identity and logical traversal order"
                );
            }
        }
    }

    for orientation in [SliderOrientation::Horizontal, SliderOrientation::Vertical] {
        settings.set(Settings {
            orientation,
            steps: 3,
            ..Default::default()
        });
        renderer.render_frames(2);
        let without = renderer.read_pixels().unwrap();
        settings.with_mut(|settings| settings.inset_icon = true);
        renderer.render_frames(2);
        let with = renderer.read_pixels().unwrap();
        let difference = with.iter().zip(&without).position(|(a, b)| a != b);
        assert!(
            difference.is_none(),
            "XS omits unsupported inset icons without losing its track children; first difference {difference:?}"
        );
    }
    for orientation in [SliderOrientation::Horizontal, SliderOrientation::Vertical] {
        settings.set(Settings {
            orientation,
            steps: 3,
            origin: Some((40.333333333, 40.5)),
            length: Some(273.0),
            ..Default::default()
        });
        renderer.step_frame();
        let rect = control(&renderer, "Value").bounds().unwrap();
        assert_eq!((rect.x0, rect.y0), (60.5, 60.75));
        let first = renderer.read_pixels().unwrap();
        renderer.render_frames(12);
        assert!(
            renderer.read_pixels().unwrap() == first,
            "fractional shapes retain first-frame pixel coverage after repeated rendering"
        );
    }
    settings.set(Settings {
        size: SliderSize::Medium,
        inset_icon: true,
        ..Default::default()
    });
    renderer.render_frames(2);
    let horizontal_icon = renderer.read_pixels().unwrap();
    for reversed in [false, true] {
        settings.with_mut(|settings| {
            settings.orientation = SliderOrientation::Vertical;
            settings.reversed = reversed;
        });
        renderer.render_frames(2);
        let vertical_icon = renderer.read_pixels().unwrap();
        for y in 0..36 {
            for x in 0..36 {
                let source = ((81 + y) * 500 + 72 + x) * 4;
                let target = (((if reversed { 432 } else { 72 }) + y) * 500 + 81 + x) * 4;
                for channel in 0..4 {
                    assert!(
                        horizontal_icon[source + channel].abs_diff(vertical_icon[target + channel])
                            <= 3,
                        "inset glyph stays upright, reversed={reversed}, {x},{y}, channel={channel}"
                    );
                }
            }
        }
    }

    for kind in [Kind::Single, Kind::Centered, Kind::Range] {
        for reversed in [false, true] {
            settings.set(Settings {
                kind,
                orientation: SliderOrientation::Vertical,
                reversed,
                steps: 3,
                ..Default::default()
            });
            VALUE.get().unwrap().set(0.25);
            RANGE.get().unwrap().set((0.25, 0.75));
            renderer.render_frames(2);
            let rect: Rect = control(
                &renderer,
                if matches!(kind, Kind::Range) {
                    "Interval"
                } else {
                    "Value"
                },
            )
            .bounds()
            .unwrap();
            let x = (rect.x0 + rect.x1) / 2.0;
            let minimum = if reversed {
                rect.y1 - 2.0
            } else {
                rect.y0 + 2.0
            };
            let maximum = if reversed {
                rect.y0 + 2.0
            } else {
                rect.y1 - 2.0
            };
            click(&mut renderer, x, minimum);
            if matches!(kind, Kind::Range) {
                assert_eq!(RANGE.get().unwrap().get(), (0.0, 0.75));
            } else {
                assert_eq!(VALUE.get().unwrap().get(), 0.0);
            }
            click(&mut renderer, x, maximum);
            if matches!(kind, Kind::Range) {
                assert_eq!(RANGE.get().unwrap().get(), (0.0, 1.0));
            } else {
                assert_eq!(VALUE.get().unwrap().get(), 1.0);
            }
            let press = PxPosition::from_f32_xy(x as f32, maximum as f32);
            let middle = PxPosition::from_f32_xy(x as f32, ((rect.y0 + rect.y1) / 2.0) as f32);
            drag(&mut renderer, press, middle);
            if matches!(kind, Kind::Range) {
                assert_eq!(RANGE.get().unwrap().get(), (0.0, 0.5));
                let press = PxPosition::from_f32_xy(x as f32, minimum as f32);
                let end = PxPosition::from_f32_xy(x as f32, maximum as f32);
                drag(&mut renderer, press, end);
                assert_eq!(
                    RANGE.get().unwrap().get(),
                    (0.5, 0.5),
                    "dragging cannot cross the peer handle"
                );
                RANGE.get().unwrap().set((0.0, 1.0));
                renderer.render_frames(2);
            } else {
                assert_eq!(VALUE.get().unwrap().get(), 0.5);
                click(&mut renderer, x, maximum);
            }
            settings.with_mut(|settings| settings.disabled = true);
            renderer.render_frames(2);
            click(&mut renderer, x, (rect.y0 + rect.y1) / 2.0);
            if matches!(kind, Kind::Range) {
                assert_eq!(RANGE.get().unwrap().get(), (0.0, 1.0));
            } else {
                assert_eq!(VALUE.get().unwrap().get(), 1.0);
            }
            let before = renderer.read_pixels().unwrap();
            for (length, pixels) in [(0.0, 0.0), (1.0, 1.0), (280.0, 420.0)] {
                settings.with_mut(|settings| settings.length = Some(length));
                renderer.render_frames(2);
                assert_eq!(
                    control(
                        &renderer,
                        if matches!(kind, Kind::Range) {
                            "Interval"
                        } else {
                            "Value"
                        }
                    )
                    .bounds()
                    .unwrap()
                    .height(),
                    pixels
                );
            }
            assert_eq!(
                renderer.read_pixels().unwrap(),
                before,
                "collapsed length must recover its pixels"
            );
        }
    }
    check_controlled_steps(&mut renderer, settings);
    check_color_states_and_markers(&mut renderer, settings);
}

fn check_color_states_and_markers(renderer: &mut HeadlessRenderer, settings: State<Settings>) {
    let pixel = |pixels: &[u8], x: usize, y: usize| -> [u8; 4] {
        pixels[(y * 500 + x) * 4..(y * 500 + x + 1) * 4]
            .try_into()
            .unwrap()
    };
    for palette in [Palette::Light, Palette::Dark, Palette::Contrast] {
        for (overrides, override_ticks) in [(false, false), (true, false), (true, true)] {
            for kind in [Kind::Single, Kind::Centered, Kind::Range] {
                for orientation in [SliderOrientation::Horizontal, SliderOrientation::Vertical] {
                    for reversed in [false, true] {
                        for disabled in [false, true] {
                            VALUE.get().unwrap().set(0.75);
                            RANGE.get().unwrap().set((0.25, 0.75));
                            settings.set(Settings {
                                kind,
                                orientation,
                                reversed,
                                disabled,
                                palette,
                                overrides,
                                override_ticks,
                                size: SliderSize::Medium,
                                steps: 7,
                                references: true,
                                background: Some(Color::new(0.4, 0.2, 0.1, 1.0)),
                                ..Default::default()
                            });
                            FocusManager::current().clear_focus(true);
                            renderer.render_frames(2);
                            let rect = control(
                                renderer,
                                if matches!(kind, Kind::Range) {
                                    "Interval"
                                } else {
                                    "Value"
                                },
                            )
                            .bounds()
                            .unwrap();
                            let point = |main: usize, cross: usize| {
                                let main = if reversed { 419 - main } else { main };
                                let (x, y) = if orientation == SliderOrientation::Horizontal {
                                    (main, cross)
                                } else {
                                    (cross, main)
                                };
                                (rect.x0 as usize + x, rect.y0 as usize + y)
                            };
                            let pixels = renderer.read_pixels().unwrap();
                            for y in rect.y0 as usize..rect.y0 as usize + 7 {
                                for x in rect.x0 as usize..rect.x0 as usize + 7 {
                                    assert_eq!(
                                        pixel(&pixels, x, y),
                                        pixel(&pixels, 250, 250),
                                        "unplaced markers must not draw at the control origin: {kind:?}, {orientation:?}, reversed={reversed}, disabled={disabled}"
                                    );
                                }
                            }
                            let reference = |index: usize| {
                                pixel(&pixels, 307 + (index % 5) * 30, 307 + (index / 5) * 30)
                            };
                            let active_main = match kind {
                                Kind::Single => 100,
                                Kind::Centered => 250,
                                Kind::Range => 200,
                            };
                            let active_tick = match kind {
                                Kind::Single => 161,
                                Kind::Centered => 260,
                                Kind::Range => 210,
                            };
                            let inactive_tick = if matches!(kind, Kind::Single) {
                                359
                            } else {
                                62
                            };
                            let mut probes = vec![
                                (
                                    active_main,
                                    27,
                                    if disabled { 5 } else { 0 },
                                    "active track",
                                ),
                                (350, 27, if disabled { 6 } else { 1 }, "inactive track"),
                                (309, 39, if disabled { 7 } else { 2 }, "handle"),
                                (active_tick, 39, if disabled { 8 } else { 3 }, "active tick"),
                                (
                                    inactive_tick,
                                    39,
                                    if disabled { 9 } else { 4 },
                                    "inactive tick",
                                ),
                                (402, 39, if disabled { 9 } else { 4 }, "end stop"),
                                (
                                    408,
                                    39,
                                    if disabled { 6 } else { 1 },
                                    "endpoint tick replaced by stop",
                                ),
                            ];
                            if !matches!(kind, Kind::Single) {
                                probes.push((18, 39, if disabled { 9 } else { 4 }, "start stop"));
                                probes.push((
                                    12,
                                    39,
                                    if disabled { 6 } else { 1 },
                                    "start tick replaced by stop",
                                ));
                            }
                            for (main, cross, expected, name) in probes {
                                let (x, y) = point(main, cross);
                                let actual = pixel(&pixels, x, y);
                                let expected = reference(expected);
                                assert!(
                                    actual
                                        .iter()
                                        .zip(expected)
                                        .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
                                    "{name}: {actual:?} != {expected:?}; {kind:?}, {palette:?}, {orientation:?}, reversed={reversed}, disabled={disabled}, overrides={overrides}"
                                );
                            }
                            settings.with_mut(|settings| settings.stops = Some(false));
                            renderer.render_frames(2);
                            let without_stops = renderer.read_pixels().unwrap();
                            let (x, y) = point(408, 39);
                            let index = if disabled { 9 } else { 4 };
                            let expected = pixel(
                                &without_stops,
                                307 + (index % 5) * 30,
                                307 + (index / 5) * 30,
                            );
                            assert!(
                                pixel(&without_stops, x, y)
                                    .iter()
                                    .zip(expected)
                                    .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
                                "removing stops must retain endpoint ticks"
                            );
                            if !disabled {
                                let (x, y) = point(309, 39);
                                click(renderer, x as f64, y as f64);
                                let focused = renderer.read_pixels().unwrap();
                                let (x, y) = point(307, 39);
                                assert_eq!(
                                    pixel(&focused, x, y),
                                    pixel(&focused, 250, 250),
                                    "focused handle gap must not contain a tick: {kind:?}, {orientation:?}, reversed={reversed}"
                                );
                            }
                            FocusManager::current().clear_focus(true);
                            settings.with_mut(|settings| settings.stops = Some(true));
                            VALUE.get().unwrap().set(1.0);
                            RANGE.get().unwrap().set((0.0, 1.0));
                            renderer.render_frames(2);
                            let endpoints = renderer.read_pixels().unwrap();
                            let (x, y) = point(402, 39);
                            assert_eq!(
                                pixel(&endpoints, x, y),
                                pixel(&endpoints, 250, 250),
                                "a collapsed inactive track must not leave a floating end stop"
                            );
                            if matches!(kind, Kind::Range) {
                                let (x, y) = point(18, 39);
                                assert_eq!(
                                    pixel(&endpoints, x, y),
                                    pixel(&endpoints, 250, 250),
                                    "a collapsed inactive track must not leave a floating start stop"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_controlled_steps(renderer: &mut HeadlessRenderer, settings: State<Settings>) {
    let value = VALUE.get().unwrap();
    let range = RANGE.get().unwrap();
    for kind in [Kind::Single, Kind::Centered, Kind::Range] {
        for orientation in [SliderOrientation::Horizontal, SliderOrientation::Vertical] {
            for reversed in [false, true] {
                settings.set(Settings {
                    kind,
                    orientation,
                    reversed,
                    steps: 3,
                    ..Default::default()
                });
                for (supplied, expected, supplied_range, expected_range) in [
                    (0.31, 0.25, (0.31, 0.59), (0.25, 0.5)),
                    (0.625, 0.75, (0.86, 0.21), (0.75, 0.75)),
                    (f32::NAN, 0.0, (f32::NAN, f32::NAN), (0.0, 0.0)),
                    (
                        f32::INFINITY,
                        1.0,
                        (f32::NEG_INFINITY, f32::INFINITY),
                        (0.0, 1.0),
                    ),
                ] {
                    value.set(supplied);
                    range.set(supplied_range);
                    renderer.render_frames(2);
                    if matches!(kind, Kind::Range) {
                        let start = control(renderer, "Interval start");
                        let end = control(renderer, "Interval end");
                        assert_eq!(start.numeric_value(), Some(expected_range.0 as f64));
                        assert_eq!(start.max_numeric_value(), Some(expected_range.1 as f64));
                        assert_eq!(end.numeric_value(), Some(expected_range.1 as f64));
                        assert_eq!(end.min_numeric_value(), Some(expected_range.0 as f64));
                    } else {
                        assert_eq!(
                            control(renderer, "Value").numeric_value(),
                            Some(expected as f64)
                        );
                    }
                    assert_eq!(value.get().to_bits(), supplied.to_bits());
                    assert_eq!(range.get().0.to_bits(), supplied_range.0.to_bits());
                    assert_eq!(range.get().1.to_bits(), supplied_range.1.to_bits());
                    let coerced = renderer.read_pixels().unwrap();
                    value.set(expected);
                    range.set(expected_range);
                    renderer.render_frames(2);
                    assert!(
                        renderer.read_pixels().unwrap() == coerced,
                        "off-grid values must render exactly like their nearest valid ticks: {kind:?}, {orientation:?}, reversed={reversed}"
                    );
                }
                value.set(0.31);
                range.set((0.31, 0.59));
                settings.with_mut(|settings| settings.steps = 0);
                renderer.render_frames(2);
                let continuous = renderer.read_pixels().unwrap();
                if matches!(kind, Kind::Range) {
                    assert_eq!(
                        control(renderer, "Interval start").numeric_value(),
                        Some(0.31_f32 as f64)
                    );
                } else {
                    assert_eq!(
                        control(renderer, "Value").numeric_value(),
                        Some(0.31_f32 as f64)
                    );
                }
                for steps in [3, 0] {
                    settings.with_mut(|settings| settings.steps = steps);
                    renderer.render_frames(2);
                }
                assert!(renderer.read_pixels().unwrap() == continuous);
                assert_eq!(value.get(), 0.31);
                assert_eq!(range.get(), (0.31, 0.59));
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn vertical_slider_pixels_pointer_semantics_and_length_recovery_msaa1() {
    crate::test_utils::run_isolated_test(
        "slider::orientation_tests::vertical_slider_pixels_pointer_semantics_and_length_recovery_msaa1",
        || check_orientation(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn vertical_slider_pixels_pointer_semantics_and_length_recovery_msaa4() {
    crate::test_utils::run_isolated_test(
        "slider::orientation_tests::vertical_slider_pixels_pointer_semantics_and_length_recovery_msaa4",
        || check_orientation(4),
    );
}
