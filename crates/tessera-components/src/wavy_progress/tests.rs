use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, Modifier, State,
    accesskit::Role,
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{
    animation::test_instant,
    boxed::boxed,
    modifier::ModifierExt as _,
    progress::ProgressStrokeCap,
    surface::{SurfaceStyle, surface},
    test_utils::{assert_pixels_near, pixel_near},
    theme::material_theme,
};

use super::{Clock, Motion, indeterminate_fractions, linear_wavy_progress_indicator};

#[derive(Clone, Copy)]
struct Settings {
    progress: Option<f32>,
    paused: bool,
    reverse: bool,
    amplitude: Option<f32>,
    speed: Option<Dp>,
    width: Option<Dp>,
    height: Option<Dp>,
    stop: Option<Dp>,
    cap: ProgressStrokeCap,
    track_cap: Option<ProgressStrokeCap>,
    stroke: Option<Dp>,
    track_stroke: Option<Dp>,
    custom_colors: bool,
    opacity: f32,
    offset: Dp,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            progress: Some(0.5),
            paused: true,
            reverse: false,
            amplitude: None,
            speed: None,
            width: None,
            height: None,
            stop: None,
            cap: ProgressStrokeCap::Round,
            track_cap: None,
            stroke: None,
            track_stroke: None,
            custom_colors: true,
            opacity: 1.0,
            offset: Dp(20.0),
        }
    }
}

#[tessera]
fn fixture(circular: Option<bool>, probe: Option<TestProbe<Option<State<Settings>>>>) {
    let settings = remember(Settings::default);
    if let Some(probe) = probe {
        probe.set(Some(settings));
    }
    material_theme().child(move || {
        boxed()
            .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
            .children(move || {
                surface()
                    .modifier(Modifier::new().fill_max_size())
                    .style(SurfaceStyle::Filled {
                        color: Color::WHITE,
                    })
                    .child(|| {});
                let wrapper = Modifier::new()
                    .offset(settings.with(|s| s.offset), settings.with(|s| s.offset))
                    .semantics(SemanticsArgs {
                        test_tag: Some("wave".into()),
                        merge_descendants: true,
                        ..Default::default()
                    });
                boxed().modifier(wrapper).children(move || {
                    let settings = settings.get();
                    let mut modifier = Modifier::new();
                    if let Some(width) = settings.width {
                        modifier = modifier.width(width);
                    }
                    if let Some(height) = settings.height {
                        modifier = modifier.height(height);
                    }
                    if circular.unwrap_or(false) {
                        super::circular_wavy_progress_indicator()
                            .modifier(modifier)
                            .progress_optional(settings.progress)
                            .paused(settings.paused)
                            .reverse_direction(settings.reverse)
                            .amplitude_optional(settings.amplitude)
                            .wave_speed_optional(settings.speed)
                            .stroke_cap(settings.cap)
                            .track_stroke_cap(settings.track_cap.unwrap_or(settings.cap))
                            .stroke_width_optional(settings.stroke)
                            .track_stroke_width_optional(settings.track_stroke)
                            .color_optional(
                                settings
                                    .custom_colors
                                    .then_some(Color::BLACK.with_alpha(settings.opacity)),
                            )
                            .track_color_optional(
                                settings
                                    .custom_colors
                                    .then_some(Color::RED.with_alpha(settings.opacity)),
                            )
                            .accessibility_label("Transfer");
                    } else {
                        linear_wavy_progress_indicator()
                            .modifier(modifier)
                            .progress_optional(settings.progress)
                            .paused(settings.paused)
                            .reverse_direction(settings.reverse)
                            .amplitude_optional(settings.amplitude)
                            .wave_speed_optional(settings.speed)
                            .stop_size_optional(settings.stop)
                            .stroke_cap(settings.cap)
                            .track_stroke_cap(settings.track_cap.unwrap_or(settings.cap))
                            .stroke_width_optional(settings.stroke)
                            .track_stroke_width_optional(settings.track_stroke)
                            .color_optional(
                                settings
                                    .custom_colors
                                    .then_some(Color::BLACK.with_alpha(settings.opacity)),
                            )
                            .track_color_optional(
                                settings
                                    .custom_colors
                                    .then_some(Color::RED.with_alpha(settings.opacity)),
                            )
                            .accessibility_label("Transfer");
                    }
                });
            });
    });
}

#[test]
fn default_container_and_parent_constraints_are_measured_by_the_component() {
    let snapshot = UiTest::new().content(fixture).viewport(400, 100).frame();
    snapshot.node("wave").size(240, 10);
    let snapshot = UiTest::new().content(fixture).viewport(80, 8).frame();
    snapshot.node("wave").size(80, 8);
}

#[test]
fn clock_preserves_phase_across_pauses_and_settles_amplitude_changes() {
    let mut motion = Motion {
        amplitude: 1.0,
        period_nanos: Some(1_000_000_000),
        indeterminate: false,
        paused: false,
        circular: false,
    };
    let mut clock = Clock::new(motion);
    clock.tick(test_instant(0));
    clock.tick(test_instant(250_000_000));
    assert_eq!(clock.phase, 0.25);
    motion.paused = true;
    clock.configure(motion, test_instant(300_000_000));
    clock.tick(test_instant(8_000_000_000));
    assert_eq!(clock.elapsed, 300_000_000);
    assert!((clock.phase - 0.3).abs() < 0.000001);
    assert!(!clock.needs_frames());
    motion.paused = false;
    motion.amplitude = 0.0;
    clock.configure(motion, test_instant(9_000_000_000));
    assert_eq!(clock.amplitude(), 1.0);
    clock.tick(test_instant(9_225_000_000));
    assert!(clock.amplitude() > 0.5 && clock.amplitude() < 1.0);
    clock.tick(test_instant(9_450_000_000));
    assert!(clock.amplitude() > 0.0);
    clock.tick(test_instant(9_500_000_000));
    assert_eq!(clock.amplitude(), 0.0);
    assert!(!clock.needs_frames());
    let elapsed = clock.elapsed;
    clock.tick(test_instant(99_000_000_000));
    assert_eq!(clock.elapsed, elapsed);
    assert!(
        (clock.phase - 0.3).abs() < 0.000001,
        "invisible wave travel is recovered from timestamps without requesting frames"
    );
}

#[test]
fn indeterminate_heads_and_tails_use_the_material_timing_and_easing() {
    assert_eq!(indeterminate_fractions(0), [[0.0; 2]; 2]);
    assert_eq!(indeterminate_fractions(1_750_000_000), [[0.0; 2]; 2]);
    for (nanos, segment, endpoint) in [
        (537_500_000, 0, 1),
        (787_500_000, 0, 0),
        (1_106_875_000, 1, 1),
        (1_356_875_000, 1, 0),
    ] {
        assert!((indeterminate_fractions(nanos)[segment][endpoint] - 0.18125).abs() < 0.00003);
    }
}

fn pixel(image: &[u8], x: usize, y: usize) -> [u8; 3] {
    let offset = (y * 600 + x) * 4;
    image[offset..offset + 3].try_into().expect("RGB pixel")
}

fn reference_wave_coverage(image: &[u8]) {
    let mut segments = Vec::new();
    for cell in 0..6 {
        let mut previous = [cell as f32 * 30.0, 7.5];
        for step in 1..=48 {
            let t = step as f32 / 48.0;
            let point = [
                (cell as f32 + t) * 30.0,
                7.5 + if cell % 2 == 0 { 18.0 } else { -18.0 } * t * (1.0 - t),
            ];
            segments.push((previous, point));
            previous = point;
        }
    }
    let mut error = 0.0;
    let mut total = 0.0;
    for y in 0..15 {
        for x in 15..165 {
            let mut covered = 0;
            for sy in 0..8 {
                for sx in 0..8 {
                    let point = [
                        x as f32 + (sx as f32 + 0.5) / 8.0,
                        y as f32 + (sy as f32 + 0.5) / 8.0,
                    ];
                    if segments.iter().any(|(a, b)| {
                        let direction = [b[0] - a[0], b[1] - a[1]];
                        let delta = [point[0] - a[0], point[1] - a[1]];
                        let t = ((delta[0] * direction[0] + delta[1] * direction[1])
                            / (direction[0] * direction[0] + direction[1] * direction[1]))
                            .clamp(0.0, 1.0);
                        let dx = delta[0] - t * direction[0];
                        let dy = delta[1] - t * direction[1];
                        dx * dx + dy * dy <= 9.0
                    }) {
                        covered += 1;
                    }
                }
            }
            let expected = covered as f64 / 64.0;
            let srgb = f64::from(pixel(image, x + 30, y + 30)[0]) / 255.0;
            let linear = if srgb <= 0.04045 {
                srgb / 12.92
            } else {
                ((srgb + 0.055) / 1.055).powf(2.4)
            };
            error += ((1.0 - linear) - expected).abs();
            total += expected;
        }
    }
    assert!(
        error / total < 0.035,
        "quadratic stroke coverage error: {}",
        error / total
    );
}

fn check_pixels_and_motion() {
    let probe = TestProbe::<Option<State<Settings>>>::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(600, 120)
        .scale_factor(1.5)
        .rendered(crate::TesseraComponents)
        .content(move || fixture().probe(probe.clone()));
    renderer.frames(3);
    let settings = observed.get().expect("settings");
    let original = renderer.read_pixels();
    let accessibility = renderer
        .accessibility_snapshot()
        .expect("progress semantics");
    let node = accessibility
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::ProgressIndicator)
        .expect("progress indicator");
    assert_eq!(node.1.numeric_value(), Some(0.5));
    assert_eq!(node.1.min_numeric_value(), Some(0.0));
    assert_eq!(node.1.max_numeric_value(), Some(1.0));
    let bounds = node.1.bounds().expect("enlarged screen-reader target");
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (30.0, 15.0, 390.0, 60.0)
    );
    assert_eq!(pixel(&original, 45, 42), [0, 0, 0], "positive crest");
    assert_eq!(pixel(&original, 75, 33), [0, 0, 0], "negative crest");
    assert_eq!(pixel(&original, 216, 37), [255, 255, 255], "track gap");
    assert_eq!(pixel(&original, 300, 37), [255, 0, 0], "inactive track");
    assert_eq!(pixel(&original, 387, 37), [0, 0, 0], "stop dot");
    reference_wave_coverage(&original);
    settings.with_mut(|settings| settings.reverse = true);
    renderer.frames(2);
    let reversed = renderer.read_pixels();
    for y in 0..15 {
        for x in 0..360 {
            let a = pixel(&original, x + 30, y + 30);
            let b = pixel(&reversed, 389 - x, 44 - y);
            assert!(
                a.into_iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 2),
                "rotated pixel {x}, {y}: {a:?} != {b:?}"
            );
        }
    }
    settings.with_mut(|settings| {
        settings.reverse = false;
        settings.custom_colors = false;
    });
    renderer.frames(2);
    let themed = renderer.read_pixels();
    assert_eq!(pixel(&themed, 45, 42), [101, 87, 137]);
    assert_eq!(pixel(&themed, 300, 37), [232, 222, 248]);
    settings.with_mut(|settings| settings.amplitude = Some(0.0));
    renderer.frames(2);
    let flat_paused = renderer.read_pixels();
    assert_eq!(pixel(&flat_paused, 45, 42), [255, 255, 255]);
    assert_eq!(pixel(&flat_paused, 45, 37), [101, 87, 137]);
    settings.with_mut(|settings| settings.amplitude = None);
    renderer.frames(2);
    assert_pixels_near(&themed, &renderer.read_pixels(), 600);
    settings.with_mut(|settings| {
        settings.progress = None;
        settings.custom_colors = true;
        settings.opacity = 0.5;
    });
    renderer.frames(2);
    assert!(
        pixel_near(&pixel(&renderer.read_pixels(), 300, 37), &[255, 188, 188],),
        "overlapping empty segments must not paint the track repeatedly"
    );
    settings.with_mut(|settings| {
        settings.custom_colors = true;
        settings.opacity = 1.0;
        settings.progress = Some(0.5);
        settings.paused = false;
    });
    renderer.frames(17);
    let moved = renderer.read_pixels();
    assert!(
        original != moved,
        "the component must request animation frames"
    );
    let allocations = renderer
        .graphics_work()
        .group("linear_wave_instances")
        .buffer_allocations;
    renderer.frames(17);
    assert_eq!(
        allocations,
        renderer
            .graphics_work()
            .group("linear_wave_instances")
            .buffer_allocations
    );
    settings.with_mut(|settings| settings.paused = true);
    renderer.frames(2);
    let paused = renderer.read_pixels();
    renderer.frames(31);
    assert_pixels_near(&paused, &renderer.read_pixels(), 600);
    for progress in [
        Some(f32::NAN),
        Some(f32::NEG_INFINITY),
        Some(-0.5),
        Some(1.5),
        Some(f32::INFINITY),
        None,
    ] {
        settings.with_mut(|settings| settings.progress = progress);
        renderer.frames(2);
        let snapshot = renderer.accessibility_snapshot().expect("semantics");
        let node = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::ProgressIndicator)
            .expect("progress node");
        let expected = progress.map(|progress| {
            if progress.is_nan() {
                0.0
            } else {
                progress.clamp(0.0, 1.0) as f64
            }
        });
        assert_eq!(node.1.numeric_value(), expected);
        assert_eq!(node.1.label(), Some("Transfer"));
    }
    settings.with_mut(|settings| {
        settings.progress = None;
        settings.paused = false;
    });
    renderer.frames(23);
    let indeterminate = renderer.read_pixels();
    renderer.frames(17);
    assert!(
        indeterminate != renderer.read_pixels(),
        "indeterminate heads and tails must advance"
    );
    settings.with_mut(|settings| {
        settings.progress = Some(0.95);
        settings.speed = Some(Dp::ZERO);
    });
    renderer.frames(32);
    let flattened = renderer.read_pixels();
    renderer.frames(17);
    assert_pixels_near(&flattened, &renderer.read_pixels(), 600);
    settings.with_mut(|settings| {
        settings.amplitude = Some(0.0);
        settings.paused = true;
    });
    renderer.frames(2);
    assert_pixels_near(&flattened, &renderer.read_pixels(), 600);
    for (width, height) in [(0.0, 10.0), (1.0, 10.0), (240.0, 0.0), (8.0, 30.0)] {
        settings.with_mut(|settings| {
            settings.width = Some(Dp(width));
            settings.height = Some(Dp(height));
        });
        renderer.frames(2);
        let pixels = renderer.read_pixels();
        if width == 0.0 || height == 0.0 {
            assert!(
                pixels.chunks_exact(4).all(|pixel| pixel == &pixels[..4]),
                "zero extent must leave no stroke or stop pixels"
            );
        }
    }
    settings.set(Settings {
        speed: Some(Dp::ZERO),
        paused: false,
        ..Default::default()
    });
    renderer.frames(32);
    settings.with_mut(|settings| settings.paused = true);
    renderer.frames(2);
    assert_pixels_near(&original, &renderer.read_pixels(), 600);
    settings.with_mut(|settings| {
        settings.height = Some(Dp(20.0));
        settings.stroke = Some(Dp(8.0));
        settings.track_stroke = Some(Dp(4.0));
        settings.track_cap = Some(ProgressStrokeCap::Butt);
    });
    renderer.frames(2);
    let mixed = renderer.read_pixels();
    assert_eq!(pixel(&mixed, 45, 54), [0, 0, 0], "thick active crest");
    assert_eq!(
        pixel(&mixed, 227, 45),
        [255, 255, 255],
        "mixed-cap track gap"
    );
    assert_eq!(pixel(&mixed, 228, 45), [255, 0, 0], "butt track start");
    assert_eq!(
        pixel(&mixed, 384, 42),
        [0, 0, 0],
        "square stop follows the track cap"
    );
    settings.with_mut(|settings| {
        settings.height = Some(Dp(14.0));
        settings.track_stroke = Some(Dp(8.0));
        settings.track_cap = None;
    });
    renderer.frames(2);
    let thick = renderer.read_pixels();
    assert_eq!(
        pixel(&thick, 45, 45),
        [0, 0, 0],
        "8dp stroke retains 3dp wave amplitude"
    );
    assert_eq!(
        pixel(&thick, 219, 40),
        [255, 255, 255],
        "4dp gap between thick round caps"
    );
    assert_eq!(pixel(&thick, 300, 40), [255, 0, 0]);
    assert_eq!(
        pixel(&thick, 384, 40),
        [0, 0, 0],
        "4dp stop is inset 2dp in the 8dp track"
    );
    settings.with_mut(|settings| {
        settings.height = None;
        settings.stroke = None;
        settings.track_stroke = None;
        settings.track_cap = None;
        settings.cap = ProgressStrokeCap::Butt;
        settings.stop = Some(Dp::ZERO);
        settings.amplitude = Some(0.0);
    });
    renderer.frames(2);
    settings.with_mut(|settings| settings.paused = false);
    renderer.frames(32);
    let butt = renderer.read_pixels();
    assert_eq!(pixel(&butt, 210, 37), [255, 255, 255]);
    assert_eq!(pixel(&butt, 215, 37), [255, 255, 255]);
    assert_eq!(pixel(&butt, 218, 37), [255, 0, 0]);
    assert_eq!(pixel(&butt, 387, 37), [255, 0, 0]);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_linear_waves_match_quadratic_coverage_motion_and_semantics() {
    crate::test_utils::run_isolated_test(
        "wavy_progress::tests::gpu_linear_waves_match_quadratic_coverage_motion_and_semantics",
        check_pixels_and_motion,
    );
}

#[test]
fn circular_layout_clamps_to_parent_and_uses_the_48dp_default() {
    let snapshot = UiTest::new()
        .content(|| fixture().circular(true))
        .viewport(400, 100)
        .frame();
    snapshot.node("wave").size(48, 48);
    let snapshot = UiTest::new()
        .content(|| fixture().circular(true))
        .viewport(35, 20)
        .frame();
    snapshot.node("wave").size(35, 20);
}

#[test]
fn circular_timing_runs_all_four_rotation_steps_and_preserves_holds() {
    for (millis, degrees) in [
        (0.0, 0.0),
        (150.0, 45.0),
        (300.0, 90.0),
        (1000.0, 90.0),
        (1650.0, 135.0),
        (1800.0, 180.0),
        (2500.0, 180.0),
        (3150.0, 225.0),
        (3300.0, 270.0),
        (4000.0, 270.0),
        (4650.0, 315.0),
        (4800.0, 360.0),
        (5999.0, 360.0),
    ] {
        assert_eq!(
            crate::progress::circular_additional_rotation_degrees(millis),
            degrees
        );
    }
    assert!((super::circular_indeterminate(1_500_000_000).0 - 0.485).abs() < 0.00001);
    assert!((super::circular_indeterminate(3_600_000_000).0 - 0.485).abs() < 0.00003);
    let (progress, rotation) = super::circular_indeterminate(3_000_000_000);
    assert!((progress - 0.87).abs() < 0.00001);
    assert!((rotation.to_degrees() - 810.0).abs() < 0.0001);
    assert_eq!(
        super::circular_indeterminate(6_000_000_000),
        super::circular_indeterminate(0)
    );
    let motion = Motion {
        amplitude: 0.0,
        period_nanos: Some(9_000_000_000),
        indeterminate: false,
        paused: false,
        circular: true,
    };
    let mut clock = Clock::new(motion);
    clock.tick(test_instant(0));
    clock.tick(test_instant(99_000_000_000));
    assert_eq!(clock.phase, 0.0, "a flat circular wave stops offset travel");
    assert!(!clock.needs_frames());
}

fn check_circular_pixels_and_motion() {
    let probe = TestProbe::<Option<State<Settings>>>::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(600, 120)
        .scale_factor(1.5)
        .rendered(crate::TesseraComponents)
        .content(move || fixture().circular(true).probe(probe.clone()));
    renderer.frames(3);
    let settings = observed.get().unwrap();
    let original = renderer.read_pixels();
    let region_contains = |image: &[u8], left: usize, top: usize, rgb: [u8; 3]| {
        (top..top + 10).any(|y| (left..left + 10).any(|x| pixel(image, x, y) == rgb))
    };
    assert!(
        region_contains(&original, 92, 61, [0, 0, 0]),
        "clockwise active arc"
    );
    assert!(
        region_contains(&original, 30, 61, [255, 0, 0]),
        "inactive arc"
    );
    assert_eq!(pixel(&original, 66, 66), [255, 255, 255], "unfilled center");
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let node = snapshot
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::ProgressIndicator)
        .unwrap();
    assert_eq!(node.1.numeric_value(), Some(0.5));
    assert_eq!(node.1.label(), Some("Transfer"));
    let bounds = node.1.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (30.0, 30.0, 102.0, 102.0)
    );
    settings.with_mut(|s| s.reverse = true);
    renderer.frames(2);
    let reversed = renderer.read_pixels();
    for y in 30..102 {
        for x in 30..102 {
            let a = pixel(&original, x, y);
            let b = pixel(&reversed, 131 - x, y);
            assert!(
                a.into_iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 3),
                "mirror {x},{y}: {a:?} {b:?}"
            );
        }
    }
    settings.with_mut(|s| {
        s.reverse = false;
        s.amplitude = Some(0.0);
        s.progress = Some(1.0);
        s.opacity = 0.5;
    });
    renderer.frames(2);
    let flat = renderer.read_pixels();
    assert!(
        pixel_near(&pixel(&flat, 99, 66), &[188, 188, 188]),
        "half-alpha stroke"
    );
    assert!(
        flat.chunks_exact(4)
            .all(|p| p[0] >= 187 && p[1] >= 187 && p[2] >= 187),
        "tile boundaries and joined segments must blend once"
    );
    settings.with_mut(|s| s.offset = Dp(20.5));
    renderer.frames(2);
    let fractional = renderer.read_pixels();
    assert!(
        fractional
            .chunks_exact(4)
            .all(|p| p[0] >= 187 && p[1] >= 187 && p[2] >= 187),
        "fractional tile edges retain uniform alpha"
    );
    settings.with_mut(|s| s.offset = Dp(20.0));
    renderer.frames(2);
    assert_pixels_near(&flat, &renderer.read_pixels(), 600);
    settings.with_mut(|s| {
        s.progress = Some(0.25);
        s.opacity = 1.0;
        s.cap = ProgressStrokeCap::Butt;
        s.track_stroke = Some(Dp(8.0));
    });
    renderer.frames(2);
    let butt = renderer.read_pixels();
    assert_eq!(
        pixel(&butt, 64, 33),
        [255, 255, 255],
        "butt start does not extend behind twelve o'clock"
    );
    assert_eq!(pixel(&butt, 68, 33), [0, 0, 0]);
    assert_eq!(
        pixel(&butt, 99, 68),
        [255, 255, 255],
        "butt end stops at the progress endpoint"
    );
    assert_eq!(pixel(&butt, 99, 63), [0, 0, 0]);
    settings.with_mut(|s| {
        s.cap = ProgressStrokeCap::Round;
        s.track_stroke = None;
    });
    settings.with_mut(|s| {
        s.amplitude = None;
        s.progress = Some(0.5);
        s.opacity = 1.0;
    });
    renderer.frames(2);
    assert_pixels_near(&original, &renderer.read_pixels(), 600);
    settings.with_mut(|s| s.paused = false);
    renderer.frames(23);
    let moving = renderer.read_pixels();
    assert!(moving != original, "the component schedules wave motion");
    let allocations = renderer
        .graphics_work()
        .group("path_stroke_instances")
        .buffer_allocations;
    renderer.frames(19);
    assert_eq!(
        allocations,
        renderer
            .graphics_work()
            .group("path_stroke_instances")
            .buffer_allocations,
        "buffers are reused at steady size"
    );
    settings.with_mut(|s| s.paused = true);
    renderer.frames(2);
    let paused = renderer.read_pixels();
    renderer.frames(20);
    assert_pixels_near(&paused, &renderer.read_pixels(), 600);
    settings.with_mut(|s| {
        s.progress = None;
        s.paused = false;
        s.speed = Some(Dp::ZERO);
    });
    renderer.frames(23);
    let ongoing = renderer.read_pixels();
    renderer.frames(23);
    assert!(
        ongoing != renderer.read_pixels(),
        "ongoing sweep moves without wave travel"
    );
    let snapshot = renderer.accessibility_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::ProgressIndicator)
            .unwrap()
            .1
            .numeric_value(),
        None
    );
    settings.with_mut(|s| {
        s.progress = Some(1.0);
        s.amplitude = Some(0.0);
        s.paused = true;
        s.height = Some(Dp::ZERO);
        s.width = Some(Dp::ZERO);
    });
    renderer.frames(2);
    assert!(
        renderer
            .read_pixels()
            .chunks_exact(4)
            .all(|p| p[..3] == [255, 255, 255]),
        "zero bounds do not draw"
    );
    settings.with_mut(|s| {
        s.width = Some(Dp(52.0));
        s.height = Some(Dp(52.0));
        s.stroke = Some(Dp(8.0));
        s.track_stroke = Some(Dp(8.0));
        s.amplitude = Some(1.0);
    });
    renderer.frames(2);
    assert!(
        region_contains(&renderer.read_pixels(), 98, 65, [0, 0, 0]),
        "thick 52dp wave recovers after empty bounds"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_circular_waves_cover_motion_transparency_semantics_and_resize() {
    crate::test_utils::run_isolated_test(
        "wavy_progress::tests::gpu_circular_waves_cover_motion_transparency_semantics_and_resize",
        check_circular_pixels_and_motion,
    );
}
