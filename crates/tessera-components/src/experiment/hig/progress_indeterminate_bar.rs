use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, ComputedData, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    LayoutPolicy, LayoutResult, MeasurementError, Modifier, State,
    accesskit::Role,
    launch,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
    time::Instant,
    use_context, with_frame_time,
};

use crate::pipelines::perceptual_fill::PerceptualSweepCommand;

use super::super::theme::{Appearance, ControlSize, HigTheme};

/// Size of an [`indeterminate_progress_bar`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum IndeterminateBarSize {
    /// 20 dp layout height, with an 8 dp track.
    #[default]
    Regular,
    /// 12 dp layout height, with a 6 dp track.
    Small,
    /// Same dimensions as small.
    Mini,
    /// Same dimensions as regular.
    Large,
}

impl From<ControlSize> for IndeterminateBarSize {
    fn from(size: ControlSize) -> Self {
        match size {
            ControlSize::Mini => Self::Mini,
            ControlSize::Small => Self::Small,
            ControlSize::Regular => Self::Regular,
            ControlSize::Large | ControlSize::ExtraLarge => Self::Large,
        }
    }
}

impl IndeterminateBarSize {
    fn heights(self) -> (Dp, Dp) {
        match self {
            Self::Regular | Self::Large => (Dp(20.0), Dp(8.0)),
            Self::Small | Self::Mini => (Dp(12.0), Dp(6.0)),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct BarLayout(Dp);

impl LayoutPolicy for BarLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self == other
    }

    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        Ok(LayoutResult::new(ComputedData {
            width: parent.width.clamp(Dp(160.0).into()),
            height: parent.height.clamp(self.0.into()),
        }))
    }
}

const HALF_PERIOD: Duration = Duration::from_millis(950);
const PERIOD: Duration = Duration::from_millis(1900);
const SWEEP: Duration = Duration::from_millis(830);
const STOP_RESERVE: Duration = Duration::from_millis(300);

fn cycle_phase(elapsed: Duration) -> Duration {
    Duration::from_nanos((elapsed.as_nanos() % PERIOD.as_nanos()) as u64)
}

fn stop_deadline(elapsed: Duration) -> Duration {
    // Leave enough time to finish the sweep before resetting to the resting cap.
    let remaining = PERIOD - cycle_phase(elapsed);
    elapsed
        + remaining
        + if remaining < STOP_RESERVE {
            PERIOD
        } else {
            Duration::ZERO
        }
}

#[derive(Clone, Copy, PartialEq)]
struct BarMotion {
    origin: Instant,
    elapsed: Duration,
    stop_at: Option<Duration>,
}

async fn animate(motion: State<Option<BarMotion>>, running: bool, displayed: bool) {
    if !running && !displayed {
        motion.set_if_changed(None);
        return;
    }
    if !running && motion.get().is_none() {
        return;
    }
    let mut first = true;
    loop {
        let keep_going = with_frame_time(|now| {
            let mut next = motion.get();
            if let Some(active) = &mut next {
                active.elapsed = now.saturating_duration_since(active.origin);
                if first && !running && active.stop_at.is_none() {
                    active.stop_at = Some(stop_deadline(active.elapsed));
                }
                if active.stop_at.is_some_and(|end| active.elapsed >= end) {
                    next = None;
                } else if running {
                    active.stop_at = None;
                }
            }
            if running && next.is_none() {
                next = Some(BarMotion {
                    origin: now,
                    elapsed: Duration::ZERO,
                    stop_at: None,
                });
            }
            motion.set_if_changed(next);
            next.is_some()
        })
        .await;
        if !keep_going {
            return;
        }
        first = false;
    }
}

fn interpolate(t: f32, keys: &[(f32, f32)]) -> f32 {
    for pair in keys.windows(2) {
        let [(t0, v0), (t1, v1)] = [pair[0], pair[1]];
        if t <= t1 {
            return v0 + (v1 - v0) * (t - t0) / (t1 - t0);
        }
    }
    keys.last().expect("nonempty sweep keyframes").1
}

fn ease_in_out(t: f32) -> f32 {
    // Invert the time axis of the (.42, 0) / (.58, 1) timing curve.
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..20 {
        let u = (low + high) * 0.5;
        let v = 1.0 - u;
        let x = 3.0 * u * v * (0.42 * v + 0.58 * u) + u * u * u;
        if x < t {
            low = u;
        } else {
            high = u;
        }
    }
    let u = (low + high) * 0.5;
    u * u * (3.0 - 2.0 * u)
}

#[derive(Clone, PartialEq)]
struct BarPaint {
    height: Dp,
    track: Color,
    lower_edge: f32,
    accent: Color,
    motion: State<Option<BarMotion>>,
}

impl DrawModifierNode for BarPaint {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let elapsed = self.motion.get().map_or(Duration::ZERO, |m| m.elapsed);
        let phase = cycle_phase(elapsed);
        let mirrored = phase >= HALF_PERIOD;
        let half = if mirrored { phase - HALF_PERIOD } else { phase };
        let t = (half.as_secs_f32() / SWEEP.as_secs_f32()).min(1.0);
        let sx = interpolate(t, &[(0.0, 0.6), (0.05, 1.0), (0.8181818, 1.0), (1.0, 0.6)]);
        let sy = interpolate(t, &[(0.0, 1.0), (0.05, 0.7), (0.58, 1.0)]);
        let width = interpolate(
            t,
            &[
                (0.0, 80.0),
                (0.01, 80.0),
                (0.21, 200.0),
                (0.9, 200.0),
                (1.0, 80.0),
            ],
        );
        let tail_opacity = interpolate(
            t,
            &[
                (0.0, 1.0),
                (0.1, 1.0),
                (0.35, 0.0),
                (0.55, 0.0),
                (0.85, 1.0),
            ],
        );
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualSweepCommand {
                track: self.track,
                lower_edge: self.lower_edge,
                accent: self.accent,
                height: self.height,
                advance: ease_in_out(t),
                width: Dp(f64::from(width * sx)),
                stretch: [sx, sy],
                tail_opacity,
                mirrored,
                opacity: 1.0,
            });
        content.draw(context.render_input);
    }
}

/// The indeterminate linear style of [`progress_view`](super::progress_view).
#[tessera]
pub(super) fn indeterminate_progress_bar(
    running: bool,
    displayed_when_stopped: Option<bool>,
    size: Option<IndeterminateBarSize>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let displayed = displayed_when_stopped.unwrap_or(true);
    let (height, track_height) = size.unwrap_or_default().heights();
    let motion = remember(|| None);
    launch((running, displayed), animate(motion, running, displayed));
    let mut modifier = modifier.unwrap_or_default();
    if running || displayed {
        let dark = use_context::<HigTheme>()
            .is_some_and(|theme| theme.get().appearance == Appearance::Dark);
        let (track, lower_edge) = if dark {
            (Color::WHITE.with_alpha(0.078125), 0.0175)
        } else {
            (Color::BLACK.with_alpha(0.0625), 0.033)
        };
        let accent = if window_active.unwrap_or(true) {
            Color::from_rgb(0.0, 0.19461783, 1.0)
        } else {
            Color::from_rgb(0.40197778, 0.40197778, 0.40197778)
        };
        modifier = modifier
            .clear_and_set_semantics(SemanticsArgs {
                role: Some(Role::ProgressIndicator),
                label: accessibility_label,
                description: accessibility_description,
                ..Default::default()
            })
            .push_draw(BarPaint {
                height: track_height,
                track,
                lower_edge,
                accent,
                motion,
            });
    }
    layout(BarLayout(height)).modifier(modifier);
}
