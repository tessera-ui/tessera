//! Expressive loading indicators with morphing shapes.
//!
//! ## Usage
//!
//! Indicate brief loading tasks with animated shapes or a known completion
//! value.

use tessera_ui::{
    Color, ComputedData, Dp, FrameNanosControl, LayoutResult, MeasurementError, Modifier,
    accesskit::Role,
    current_frame_nanos,
    layout::{MeasureScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pipelines::loading_shape::command::LoadingShapeCommand,
    shape_def::Shape,
    theme::MaterialTheme,
};

const MORPH_INTERVAL_NANOS: u64 = 650_000_000;
const MORPH_DURATION_NANOS: u64 = 297_000_000;
const ROTATION_DURATION_NANOS: u64 = 4_666_000_000;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
    sequence: usize,
    progress: f32,
    rotation: f32,
}

fn normalize_progress(progress: f32) -> f32 {
    if progress.is_nan() {
        0.0
    } else {
        progress.clamp(0.0, 1.0)
    }
}

fn spring_value(nanos: u64) -> f32 {
    let seconds = (nanos / 1_000_000) as f64 / 1000.0;
    let omega = 200.0_f64.sqrt();
    let damping = f64::from(0.6_f32);
    let decay = omega * damping;
    let frequency = omega * (1.0 - damping * damping).sqrt();
    (1.0 - (-decay * seconds).exp()
        * ((frequency * seconds).cos() + decay / frequency * (frequency * seconds).sin()))
        as f32
}

fn indeterminate_pose(nanos: u64) -> Pose {
    let step = nanos / MORPH_INTERVAL_NANOS;
    let phase = nanos % MORPH_INTERVAL_NANOS;
    let global_rotation =
        (nanos % ROTATION_DURATION_NANOS) as f64 / ROTATION_DURATION_NANOS as f64 * 360.0;
    let (step, progress) = if phase < MORPH_DURATION_NANOS {
        (step, spring_value(phase))
    } else {
        (step + 1, 0.0)
    };
    Pose {
        sequence: (step % 7) as usize,
        progress,
        rotation: (progress * 90.0 + ((step + 1) % 4) as f32 * 90.0 + global_rotation as f32)
            % 360.0,
    }
}

#[derive(Clone, Copy)]
struct AnimationClock {
    elapsed: u64,
    frame: u64,
    running: bool,
}

impl AnimationClock {
    fn advance(&mut self, frame: u64) {
        if self.running {
            self.elapsed = self
                .elapsed
                .saturating_add(frame.saturating_sub(self.frame));
        }
        self.frame = frame;
    }

    fn set_running(&mut self, running: bool, frame: u64) {
        self.advance(frame);
        self.running = running;
    }
}

#[derive(Clone, Copy, PartialEq)]
struct LoadingLayout;

impl tessera_ui::LayoutModifierNode for LoadingLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: input.parent_constraint().width().clamp(Dp(48.0).to_px()),
            height: input.parent_constraint().height().clamp(Dp(48.0).to_px()),
        }))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct LoadingDraw {
    pose: Pose,
    color: Color,
}

impl tessera_ui::DrawModifierNode for LoadingDraw {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        if self.color.a > 0.0 {
            ctx.render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(LoadingShapeCommand {
                    sequence: self.pose.sequence,
                    progress: self.pose.progress,
                    rotation: self.pose.rotation,
                    color: self.color,
                });
        }
        content.draw(ctx.render_input);
    }
}

/// # loading_indicator
///
/// Show a morphing shape for brief loading tasks or known progress.
///
/// ## Usage
///
/// Indicate pending content, a refresh, or completion of a short task.
///
/// ## Parameters
///
/// - `modifier` — placement and size; defaults to 48dp square.
/// - `progress` — completion in `0.0..=1.0`; omit for a repeating seven-shape
///   animation.
/// - `contained` — whether to show a rounded primary-container background.
/// - `indicator_color` — optional shape color override.
/// - `container_color`, `container_shape` — optional contained appearance
///   overrides.
/// - `paused` — pause the repeating animation and stop frame requests.
/// - `accessibility_label` — localized task description; defaults to "Loading".
///
/// ## Examples
///
/// ```
/// use tessera_components::loading_indicator::loading_indicator;
/// use tessera_ui::{remember, tessera};
///
/// let completion = 0.5_f32;
/// assert!((0.0..=1.0).contains(&completion));
///
/// #[tessera]
/// fn pending_content() {
///     let completion = remember(|| 0.5_f32);
///     loading_indicator()
///         .progress(completion.get())
///         .contained(true);
/// }
/// ```
#[tessera]
pub fn loading_indicator(
    modifier: Option<Modifier>,
    progress: Option<f32>,
    contained: Option<bool>,
    indicator_color: Option<Color>,
    container_color: Option<Color>,
    container_shape: Option<Shape>,
    paused: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let progress = progress.map(normalize_progress);
    let running = progress.is_none() && !paused.unwrap_or(false);
    let now = current_frame_nanos();
    let clock = remember(|| AnimationClock {
        elapsed: 0,
        frame: now,
        running,
    });
    if clock.with(|clock| clock.running) != running {
        clock.with_mut(|clock| clock.set_running(running, now));
    }
    if running {
        receive_frame_nanos(move |frame| {
            if !clock.with(|clock| clock.running) {
                return FrameNanosControl::Stop;
            }
            clock.with_mut(|clock| clock.advance(frame));
            FrameNanosControl::Continue
        });
    }
    let pose = if let Some(progress) = progress {
        Pose {
            sequence: 7,
            progress,
            rotation: -180.0 * progress,
        }
    } else {
        indeterminate_pose(clock.with(|clock| clock.elapsed))
    };
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let contained = contained.unwrap_or(false);
    let color = indicator_color.unwrap_or(if contained {
        scheme.on_primary_container
    } else {
        scheme.primary
    });
    let background = if contained {
        container_color.unwrap_or(scheme.primary_container)
    } else {
        Color::TRANSPARENT
    };
    let semantics = SemanticsArgs {
        role: Some(Role::ProgressIndicator),
        label: Some(accessibility_label.unwrap_or_else(|| "Loading".into())),
        numeric_value: progress.map(f64::from),
        numeric_range: progress.map(|_| (0.0, 1.0)),
        ..Default::default()
    };
    layout()
        .modifier(
            modifier
                .unwrap_or_default()
                .background_with_shape(background, container_shape.unwrap_or(Shape::CAPSULE))
                .clip_to_bounds()
                .semantics(semantics),
        )
        .policy(LoadingLayout)
        .draw(LoadingDraw { pose, color });
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, sync::atomic::Ordering};

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, State,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use crate::{
        boxed::boxed,
        column::column,
        pipelines::loading_shape::{
            geometry::{center, geometry},
            pipeline::INSTANCE_ALLOCATIONS,
        },
        theme::material_theme,
    };

    use super::*;

    #[derive(Clone)]
    struct Config {
        progress: Option<f32>,
        paused: bool,
    }
    thread_local! {
        static CONFIG: Cell<Option<State<Config>>> = const { Cell::new(None) };
        static POSE: Cell<Option<State<Pose>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture() {
        let config = remember(|| Config {
            progress: Some(0.0),
            paused: false,
        });
        CONFIG.set(Some(config));
        material_theme().child(move || {
            column().children(move || {
                for (index, contained) in [false, true].into_iter().enumerate() {
                    boxed()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some(format!("indicator-{index}")),
                            merge_descendants: true,
                            ..Default::default()
                        }))
                        .children(move || {
                            let config = config.get();
                            loading_indicator()
                                .progress_optional(config.progress)
                                .paused(config.paused)
                                .contained(contained)
                                .accessibility_label(if contained {
                                    "Refresh"
                                } else {
                                    "Download"
                                });
                        });
                }
            });
        });
    }

    #[tessera]
    fn shape_fixture() {
        let pose = remember(|| Pose {
            sequence: 0,
            progress: 0.0,
            rotation: 0.0,
        });
        POSE.set(Some(pose));
        layout()
            .modifier(Modifier::new().size(Dp(64.0), Dp(64.0)))
            .policy(LoadingLayout)
            .draw(LoadingDraw {
                pose: pose.get(),
                color: Color::WHITE,
            });
    }

    #[test]
    fn spring_samples_and_shape_schedule_match_the_reference_animation() {
        for (millis, expected) in [
            (50, 0.184660926),
            (100, 0.527367473),
            (150, 0.826900542),
            (200, 1.011090755),
            (250, 1.086345792),
        ] {
            assert!((spring_value(millis * 1_000_000) - expected).abs() < 0.000002);
        }
        assert_eq!(indeterminate_pose(0).sequence, 0);
        assert_eq!(indeterminate_pose(0).rotation, 90.0);
        assert!(indeterminate_pose(250_000_000).progress > 1.0);
        assert_eq!(indeterminate_pose(MORPH_DURATION_NANOS).sequence, 1);
        assert_eq!(indeterminate_pose(MORPH_DURATION_NANOS).progress, 0.0);
        assert_eq!(indeterminate_pose(MORPH_INTERVAL_NANOS).sequence, 1);
        assert_eq!(indeterminate_pose(MORPH_INTERVAL_NANOS * 7).sequence, 0);
    }

    #[test]
    fn paused_clock_excludes_time_spent_waiting() {
        let mut clock = AnimationClock {
            elapsed: 0,
            frame: 0,
            running: true,
        };
        clock.advance(100_000_000);
        clock.set_running(false, 150_000_000);
        clock.advance(5_000_000_000);
        assert_eq!(clock.elapsed, 150_000_000);
        clock.set_running(true, 10_000_000_000);
        clock.advance(10_050_000_000);
        assert_eq!(clock.elapsed, 200_000_000);
        assert_eq!(normalize_progress(f32::NAN), 0.0);
        assert_eq!(normalize_progress(f32::INFINITY), 1.0);
        assert_eq!(normalize_progress(-1.0), 0.0);
    }

    #[test]
    fn both_appearances_default_to_48dp_and_accept_bounded_sizes() {
        let snapshot = layout_test(|| {
            fixture();
        })
        .viewport_px(160, 160)
        .run();
        snapshot.node("indicator-0").size(48, 48);
        snapshot.node("indicator-1").size(48, 48);
        let snapshot = layout_test(|| {
            fixture();
        })
        .viewport_px(32, 64)
        .run();
        snapshot.node("indicator-0").size(32, 48);
        let mut session = start_layout_test_session(layout_test(|| {
            fixture();
        }));
        pump_layout_test_session(&mut session, 0);
        CONFIG
            .get()
            .unwrap()
            .with_mut(|config| config.progress = Some(1.0));
        pump_layout_test_session(&mut session, 16_666_667)
            .node("indicator-0")
            .size(48, 48);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_loading_animation_pauses_reuses_buffers_and_publishes_progress() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 128,
                height: 128,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let initial = renderer.read_pixels().unwrap();
        let tree = renderer.accessibility_snapshot().unwrap();
        let indicators: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::ProgressIndicator)
            .collect();
        assert_eq!(indicators.len(), 2);
        assert!(
            indicators
                .iter()
                .all(|(_, node)| node.numeric_value() == Some(0.0))
        );
        CONFIG
            .get()
            .unwrap()
            .with_mut(|config| config.progress = Some(1.0));
        renderer.render_frames(3);
        assert_ne!(initial, renderer.read_pixels().unwrap());
        CONFIG
            .get()
            .unwrap()
            .with_mut(|config| config.progress = Some(0.0));
        renderer.render_frames(3);
        assert_eq!(initial, renderer.read_pixels().unwrap());
        CONFIG
            .get()
            .unwrap()
            .with_mut(|config| config.progress = None);
        renderer.render_frames(5);
        let before = renderer.read_pixels().unwrap();
        renderer.render_frames(10);
        assert_ne!(before, renderer.read_pixels().unwrap());
        let allocations = INSTANCE_ALLOCATIONS.load(Ordering::Relaxed);
        renderer.render_frames(180);
        assert_eq!(allocations, INSTANCE_ALLOCATIONS.load(Ordering::Relaxed));
        CONFIG
            .get()
            .unwrap()
            .with_mut(|config| config.paused = true);
        renderer.render_frames(3);
        let paused = renderer.read_pixels().unwrap();
        let allocations = INSTANCE_ALLOCATIONS.load(Ordering::Relaxed);
        renderer.render_frames(180);
        assert_eq!(paused, renderer.read_pixels().unwrap());
        assert_eq!(allocations, INSTANCE_ALLOCATIONS.load(Ordering::Relaxed));
        let tree = renderer.accessibility_snapshot().unwrap();
        assert!(
            tree.nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::ProgressIndicator)
                .all(|(_, node)| node.numeric_value().is_none())
        );
    }

    fn reference_coverage(pose: Pose) -> Vec<f32> {
        let data = geometry();
        let sequence = data.sequences[pose.sequence];
        let center = center(sequence, pose.progress);
        let (sin, cos) = pose.rotation.to_radians().sin_cos();
        let mut outline = Vec::new();
        for curve in
            &data.curves[sequence.offset as usize..(sequence.offset + sequence.count) as usize]
        {
            for step in 0..64 {
                let t = step as f32 / 64.0;
                let u = 1.0 - t;
                let weights = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
                let mut point = [0.0; 2];
                for axis in 0..2 {
                    for i in 0..4 {
                        point[axis] += (curve.start[i][axis]
                            + (curve.end[i][axis] - curve.start[i][axis]) * pose.progress)
                            * weights[i];
                    }
                    point[axis] = (point[axis] - center[axis]) * 64.0 * sequence.scale;
                }
                outline.push([
                    32.0 + point[0] * cos - point[1] * sin,
                    32.0 + point[0] * sin + point[1] * cos,
                ]);
            }
        }
        let mut coverage = vec![0.0; 64 * 64];
        let mut crossings = Vec::new();
        for row in 0..64 {
            for sample in 0..16 {
                let y = row as f32 + (sample as f32 + 0.5) / 16.0;
                crossings.clear();
                for (index, a) in outline.iter().enumerate() {
                    let b = outline[(index + 1) % outline.len()];
                    if (a[1] <= y && b[1] > y) || (b[1] <= y && a[1] > y) {
                        crossings.push(a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]));
                    }
                }
                crossings.sort_by(f32::total_cmp);
                for pair in crossings.chunks_exact(2) {
                    for x in 0..64 {
                        coverage[row * 64 + x] +=
                            (pair[1].min(x as f32 + 1.0) - pair[0].max(x as f32)).clamp(0.0, 1.0)
                                / 16.0;
                    }
                }
            }
        }
        coverage
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_loading_shapes_match_supersampled_cubic_references() {
        for sample_count in [1, 4] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(|| {
                    shape_fixture();
                }),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 64,
                    height: 64,
                    sample_count,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(2);
            for sequence in 0..8 {
                for progress in [0.0, 0.5, 1.086] {
                    let pose = Pose {
                        sequence,
                        progress,
                        rotation: 37.0,
                    };
                    POSE.get().unwrap().set(pose);
                    renderer.render_frames(2);
                    let pixels = renderer.read_pixels().unwrap();
                    let reference = reference_coverage(pose);
                    let total: f32 = reference.iter().sum();
                    if let Some(directory) = std::env::var_os("TESSERA_LOADING_QA_DIR") {
                        let directory = std::path::Path::new(&directory);
                        std::fs::create_dir_all(directory).unwrap();
                        let comparison = image::RgbaImage::from_fn(192, 64, |x, y| {
                            let i = (y as usize * 64) + x as usize % 64;
                            let actual = pixels[i * 4 + 3] as f32 / 255.0;
                            let expected = reference[i];
                            let value = if x < 64 {
                                actual
                            } else if x < 128 {
                                expected
                            } else {
                                (actual - expected).abs() * 4.0
                            };
                            let channel = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                            image::Rgba([channel, channel, channel, 255])
                        });
                        comparison
                            .save(directory.join(format!(
                                "loading-shape-{sequence}-{progress}-msaa{sample_count}.png"
                            )))
                            .unwrap();
                    }
                    let error: f32 = pixels
                        .chunks_exact(4)
                        .zip(reference)
                        .map(|(pixel, expected)| (pixel[3] as f32 / 255.0 - expected).abs())
                        .sum();
                    assert!(
                        error / total < 0.025,
                        "sequence {sequence}, progress {progress}, MSAA {sample_count}: coverage error {}",
                        error / total
                    );
                }
            }
        }
    }
}
