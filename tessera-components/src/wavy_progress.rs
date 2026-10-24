//! Expressive progress indicators with moving waves.
//!
//! ## Usage
//!
//! Show task completion or ongoing work with an animated waveform.

mod circular;
mod geometry;
mod shapes;

#[cfg(test)]
mod tests;

use tessera_ui::{
    Color, ComputedData, Dp, FrameNanosControl, LayoutResult, MeasurementError, Modifier,
    accesskit::Role,
    current_frame_nanos,
    layout::{MeasureScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    progress::{ProgressStrokeCap, cubic_bezier_easing},
    theme::MaterialTheme,
};

use self::{
    circular::{Cache, CircularWaveDraw, vertices},
    geometry::LinearWaveDraw,
};

const AMPLITUDE_DURATION: u64 = 500_000_000;
const INDETERMINATE_DURATION: u64 = 1_750_000_000;

/// Material defaults for expressive progress indicators.
pub struct WavyProgressIndicatorDefaults;

impl WavyProgressIndicatorDefaults {
    /// Default linear container width.
    pub const LINEAR_WIDTH: Dp = Dp(240.0);
    /// Default linear container height, including the wave.
    pub const LINEAR_HEIGHT: Dp = Dp(10.0);
    /// Default active and inactive stroke width.
    pub const STROKE_WIDTH: Dp = Dp(4.0);
    /// Default wavelength for known completion.
    pub const LINEAR_DETERMINATE_WAVELENGTH: Dp = Dp(40.0);
    /// Default wavelength for ongoing work.
    pub const LINEAR_INDETERMINATE_WAVELENGTH: Dp = Dp(20.0);
    /// Default visible spacing between active and inactive strokes.
    pub const TRACK_GAP: Dp = Dp(4.0);
    /// Default stop indicator diameter.
    pub const STOP_SIZE: Dp = Dp(4.0);
    /// Default circular container diameter.
    pub const CIRCULAR_SIZE: Dp = Dp(48.0);
    /// Preferred circular wavelength; the number of waves is rounded to fit.
    pub const CIRCULAR_WAVELENGTH: Dp = Dp(15.0);

    /// Returns the default wave amplitude for known completion, flattening the
    /// first 10% and final 5% of progress.
    pub fn indicator_amplitude(progress: f32) -> f32 {
        let progress = normalized(progress);
        if progress <= 0.1 || progress >= 0.95 {
            0.0
        } else {
            1.0
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CircularWaveLayout;

impl tessera_ui::LayoutModifierNode for CircularWaveLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: input
                .parent_constraint()
                .width()
                .clamp(WavyProgressIndicatorDefaults::CIRCULAR_SIZE.to_px()),
            height: input
                .parent_constraint()
                .height()
                .clamp(WavyProgressIndicatorDefaults::CIRCULAR_SIZE.to_px()),
        }))
    }
}

fn circular_indeterminate(elapsed: u64) -> (f32, f32) {
    let millis = (elapsed % 6_000_000_000) as f32 / 1_000_000.0;
    let progress = crate::progress::circular_indeterminate_progress(millis);
    let rotation = millis / 6000.0 * 1080.0
        + crate::progress::circular_additional_rotation_degrees(millis)
        + 90.0;
    (progress, rotation.to_radians())
}

/// # circular_wavy_progress_indicator
///
/// Show completion or ongoing work with an expressive circular waveform.
///
/// ## Usage
///
/// Indicate a loading operation or task completion in a compact space.
///
/// ## Parameters
///
/// - `progress` — completion in `0.0..=1.0`; omit for ongoing work.
/// - `modifier` — placement and size; defaults to 48dp square.
/// - `color`, `track_color` — optional active and inactive colors.
/// - `stroke_width`, `track_stroke_width` — independent widths; default to 4dp.
/// - `stroke_cap`, `track_stroke_cap` — independent caps; default to rounded.
/// - `amplitude` — wave fraction in `0.0..=1.0`; defaults to full height,
///   flattening at known progress at or below 10% and at or above 95%.
/// - `wavelength` — preferred wavelength, default 15dp. The wave count is
///   rounded to fit, with a minimum of five and a maximum of 1024.
/// - `wave_speed` — wave travel per second; defaults to one wavelength. Zero
///   stops wave travel while indeterminate progress can continue.
/// - `gap_size` — visible spacing between active and inactive arcs; default
///   4dp.
/// - `paused` — freeze motion and stop requesting animation frames; changed
///   appearance parameters still apply immediately.
/// - `reverse_direction` — mirror the indicator for counterclockwise flow.
/// - `accessibility_label`, `accessibility_description` — localized task
///   details.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     theme::material_theme,
///     wavy_progress::{WavyProgressIndicatorDefaults, circular_wavy_progress_indicator},
/// };
/// use tessera_ui::tessera;
///
/// assert_eq!(WavyProgressIndicatorDefaults::CIRCULAR_SIZE.0, 48.0);
/// #[tessera]
/// fn content() {
///     material_theme().child(|| {
///         circular_wavy_progress_indicator()
///             .progress(0.5)
///             .paused(true);
///     });
/// }
/// content();
/// ```
#[tessera]
pub fn circular_wavy_progress_indicator(
    progress: Option<f32>,
    modifier: Option<Modifier>,
    color: Option<Color>,
    track_color: Option<Color>,
    stroke_width: Option<Dp>,
    track_stroke_width: Option<Dp>,
    stroke_cap: Option<ProgressStrokeCap>,
    track_stroke_cap: Option<ProgressStrokeCap>,
    amplitude: Option<f32>,
    wavelength: Option<Dp>,
    wave_speed: Option<Dp>,
    gap_size: Option<Dp>,
    paused: Option<bool>,
    reverse_direction: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    use tessera_ui::FocusModifierExt as _;

    let progress = progress.map(normalized);
    let wavelength =
        nonnegative(wavelength.unwrap_or(WavyProgressIndicatorDefaults::CIRCULAR_WAVELENGTH));
    let speed = nonnegative(wave_speed.unwrap_or(wavelength));
    let stroke = nonnegative(stroke_width.unwrap_or(WavyProgressIndicatorDefaults::STROKE_WIDTH));
    let amplitude = if wavelength.0 == 0.0 {
        0.0
    } else {
        amplitude.map(normalized).unwrap_or_else(|| {
            progress
                .map(WavyProgressIndicatorDefaults::indicator_amplitude)
                .unwrap_or(1.0)
        })
    };
    let measured_size = remember(|| None::<[f32; 2]>);
    let size = measured_size
        .get()
        .unwrap_or([WavyProgressIndicatorDefaults::CIRCULAR_SIZE.to_pixels_f32(); 2]);
    let vertex_count = vertices(
        size[0],
        size[1],
        stroke.to_pixels_f32(),
        wavelength.to_pixels_f32(),
    );
    let period_nanos = (wavelength.0 > 0.0 && speed.0 > 0.0).then(|| {
        ((wavelength.0 / speed.0 * 1000.0 * vertex_count as f64)
            .round()
            .max(50.0)
            * 1_000_000.0)
            .min(u64::MAX as f64) as u64
    });
    let motion = Motion {
        amplitude,
        period_nanos,
        indeterminate: progress.is_none(),
        paused: paused.unwrap_or(false),
        circular: true,
    };
    let now = current_frame_nanos();
    let clock = remember(|| Clock::new(motion, now));
    if clock.with(|clock| clock.motion != motion) {
        clock.with_mut(|clock| clock.configure(motion, now));
    }
    if clock.with(Clock::needs_frames) {
        receive_frame_nanos(move |frame| {
            clock.with_mut(|clock| clock.advance(frame));
            if clock.with(Clock::needs_frames) {
                FrameNanosControl::Continue
            } else {
                FrameNanosControl::Stop
            }
        });
    }
    let cache = remember(Cache::default).get();
    let clock = clock.get();
    let (fraction, rotation) = progress
        .map(|progress| (progress, 0.0))
        .unwrap_or_else(|| circular_indeterminate(clock.elapsed));
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    layout()
        .modifier(
            modifier
                .unwrap_or_default()
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(Role::ProgressIndicator),
                    label: accessibility_label,
                    description: accessibility_description,
                    numeric_value: progress.map(f64::from),
                    numeric_range: progress.map(|_| (0.0, 1.0)),
                    ..Default::default()
                })
                .on_layout_coordinates(move |coordinates: tessera_ui::LayoutCoordinates| {
                    let size = [
                        coordinates.local_size.width.to_f32(),
                        coordinates.local_size.height.to_f32(),
                    ];
                    if measured_size.get() != Some(size) {
                        measured_size.set(Some(size));
                    }
                }),
        )
        .policy(CircularWaveLayout)
        .draw(CircularWaveDraw {
            cache,
            wavelength,
            progress: fraction,
            amplitude: clock.amplitude(),
            phase: if clock.amplitude() > 0.0 && period_nanos.is_some() {
                clock.phase.fract() as f32
            } else {
                0.0
            },
            rotation,
            stroke_width: stroke,
            track_stroke_width: nonnegative(
                track_stroke_width.unwrap_or(WavyProgressIndicatorDefaults::STROKE_WIDTH),
            ),
            gap: nonnegative(gap_size.unwrap_or(WavyProgressIndicatorDefaults::TRACK_GAP)),
            stroke_cap: stroke_cap.unwrap_or_default(),
            track_stroke_cap: track_stroke_cap.unwrap_or_default(),
            color: color.unwrap_or(scheme.primary),
            track_color: track_color.unwrap_or(scheme.secondary_container),
            reverse: reverse_direction.unwrap_or(false),
        });
}

fn normalized(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn nonnegative(value: Dp) -> Dp {
    if value.0.is_finite() {
        Dp(value.0.max(0.0))
    } else {
        Dp::ZERO
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Motion {
    amplitude: f32,
    period_nanos: Option<u64>,
    indeterminate: bool,
    paused: bool,
    circular: bool,
}

#[derive(Clone, Copy)]
struct Clock {
    motion: Motion,
    elapsed: u64,
    frame: u64,
    phase: f64,
    amplitude_origin: f32,
    amplitude_start: u64,
}

impl Clock {
    fn new(motion: Motion, frame: u64) -> Self {
        Self {
            motion,
            elapsed: 0,
            frame,
            phase: 0.0,
            amplitude_origin: motion.amplitude,
            amplitude_start: 0,
        }
    }

    fn amplitude(&self) -> f32 {
        let t =
            self.elapsed.saturating_sub(self.amplitude_start) as f64 / AMPLITUDE_DURATION as f64;
        if t >= 1.0 || self.amplitude_origin == self.motion.amplitude {
            return self.motion.amplitude;
        }
        let easing = if self.amplitude_origin < self.motion.amplitude {
            cubic_bezier_easing(t as f32, 0.2, 0.0, 0.0, 1.0)
        } else {
            cubic_bezier_easing(t as f32, 0.3, 0.0, 0.8, 0.15)
        };
        self.amplitude_origin + (self.motion.amplitude - self.amplitude_origin) * easing
    }

    fn needs_frames(&self) -> bool {
        !self.motion.paused
            && (self.motion.indeterminate
                || (self.amplitude_origin != self.motion.amplitude
                    && self.elapsed.saturating_sub(self.amplitude_start) < AMPLITUDE_DURATION)
                || (self.amplitude() > 0.0 && self.motion.period_nanos.is_some()))
    }

    fn advance(&mut self, frame: u64) {
        let delta = frame.saturating_sub(self.frame);
        if !self.motion.paused
            && (!self.motion.circular || self.amplitude() > 0.0)
            && let Some(period) = self.motion.period_nanos
        {
            self.phase += delta as f64 / period as f64;
            if !self.motion.circular {
                self.phase = self.phase.fract();
            }
        }
        if self.needs_frames() {
            self.elapsed = self.elapsed.saturating_add(delta);
        }
        self.frame = frame;
    }

    fn configure(&mut self, motion: Motion, frame: u64) {
        self.advance(frame);
        if motion.amplitude != self.motion.amplitude {
            self.amplitude_origin = if motion.paused {
                motion.amplitude
            } else {
                self.amplitude()
            };
            self.amplitude_start = self.elapsed;
        }
        if motion.period_nanos.is_none() {
            self.phase = 0.0;
        }
        self.motion = motion;
    }
}

fn indeterminate_fractions(elapsed: u64) -> [[f32; 2]; 2] {
    let millis = (elapsed % INDETERMINATE_DURATION) as f64 / 1_000_000.0;
    let keyframe = |delay: f64, duration: f64| {
        let t = ((millis - delay) / duration).clamp(0.0, 1.0) as f32;
        if t == 0.0 || t == 1.0 {
            t
        } else {
            cubic_bezier_easing(t, 0.3, 0.0, 0.8, 0.15)
        }
    };
    [
        [keyframe(250.0, 1000.0), keyframe(0.0, 1000.0)],
        [keyframe(900.0, 850.0), keyframe(650.0, 850.0)],
    ]
}

#[derive(Clone, Copy, PartialEq)]
struct LinearWaveLayout;

impl tessera_ui::LayoutModifierNode for LinearWaveLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: input
                .parent_constraint()
                .width()
                .clamp(WavyProgressIndicatorDefaults::LINEAR_WIDTH.to_px()),
            height: input
                .parent_constraint()
                .height()
                .clamp(WavyProgressIndicatorDefaults::LINEAR_HEIGHT.to_px()),
        }))
    }
}

/// # linear_wavy_progress_indicator
///
/// Show completion or ongoing work with an expressive linear waveform.
///
/// ## Usage
///
/// Indicate a download, upload, or background operation with known or unknown
/// duration.
///
/// ## Parameters
///
/// - `progress` — completion in `0.0..=1.0`; omit for an indeterminate
///   animation.
/// - `modifier` — placement and size; defaults to 240dp by 10dp.
/// - `color`, `track_color` — optional active and inactive colors.
/// - `stroke_width`, `track_stroke_width` — independent stroke widths; default
///   to 4dp.
/// - `stroke_cap`, `track_stroke_cap` — active and inactive caps; default to
///   rounded.
/// - `amplitude` — wave height fraction in `0.0..=1.0`; defaults to full
///   height, flattening at progress at or below 10% and at or above 95%.
/// - `wavelength` — wave length; defaults to 40dp for known progress and 20dp
///   otherwise.
/// - `wave_speed` — distance traveled per second; defaults to one wavelength.
///   Zero stops wave travel while indeterminate progress can continue.
/// - `gap_size`, `stop_size` — track spacing and determinate stop diameter;
///   both default to 4dp, and a zero stop diameter hides the stop.
/// - `paused` — freeze motion and stop animation frame requests; changed
///   appearance parameters still apply immediately.
/// - `reverse_direction` — rotate the indicator by 180 degrees for reverse
///   flow.
/// - `accessibility_label`, `accessibility_description` — localized task
///   information.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     theme::material_theme,
///     wavy_progress::{WavyProgressIndicatorDefaults, linear_wavy_progress_indicator},
/// };
/// use tessera_ui::tessera;
///
/// assert_eq!(
///     WavyProgressIndicatorDefaults::indicator_amplitude(0.05),
///     0.0
/// );
/// assert_eq!(WavyProgressIndicatorDefaults::indicator_amplitude(0.5), 1.0);
/// #[tessera]
/// fn content() {
///     material_theme().child(|| {
///         linear_wavy_progress_indicator().progress(0.5).paused(true);
///     });
/// }
/// content();
/// ```
#[tessera]
pub fn linear_wavy_progress_indicator(
    progress: Option<f32>,
    modifier: Option<Modifier>,
    color: Option<Color>,
    track_color: Option<Color>,
    stroke_width: Option<Dp>,
    track_stroke_width: Option<Dp>,
    stroke_cap: Option<ProgressStrokeCap>,
    track_stroke_cap: Option<ProgressStrokeCap>,
    amplitude: Option<f32>,
    wavelength: Option<Dp>,
    wave_speed: Option<Dp>,
    gap_size: Option<Dp>,
    stop_size: Option<Dp>,
    paused: Option<bool>,
    reverse_direction: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let progress = progress.map(normalized);
    let amplitude = amplitude.map(normalized).unwrap_or_else(|| {
        progress
            .map(WavyProgressIndicatorDefaults::indicator_amplitude)
            .unwrap_or(1.0)
    });
    let wavelength = nonnegative(wavelength.unwrap_or(if progress.is_some() {
        WavyProgressIndicatorDefaults::LINEAR_DETERMINATE_WAVELENGTH
    } else {
        WavyProgressIndicatorDefaults::LINEAR_INDETERMINATE_WAVELENGTH
    }));
    let speed = nonnegative(wave_speed.unwrap_or(wavelength));
    let period_nanos = (wavelength.0 > 0.0 && speed.0 > 0.0).then(|| {
        ((wavelength.0 / speed.0 * 1000.0).round().max(50.0) * 1_000_000.0).min(u64::MAX as f64)
            as u64
    });
    let motion = Motion {
        amplitude,
        period_nanos,
        indeterminate: progress.is_none(),
        paused: paused.unwrap_or(false),
        circular: false,
    };
    let now = current_frame_nanos();
    let clock = remember(|| Clock::new(motion, now));
    if clock.with(|clock| clock.motion != motion) {
        clock.with_mut(|clock| clock.configure(motion, now));
    }
    if clock.with(Clock::needs_frames) {
        receive_frame_nanos(move |frame| {
            clock.with_mut(|clock| clock.advance(frame));
            if clock.with(Clock::needs_frames) {
                FrameNanosControl::Continue
            } else {
                FrameNanosControl::Stop
            }
        });
    }
    let clock = clock.get();
    let fractions = progress
        .map(|progress| [[0.0, progress], [0.0, 0.0]])
        .unwrap_or_else(|| indeterminate_fractions(clock.elapsed));
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let semantics = SemanticsArgs {
        role: Some(Role::ProgressIndicator),
        label: accessibility_label,
        description: accessibility_description,
        numeric_value: progress.map(f64::from),
        numeric_range: progress.map(|_| (0.0, 1.0)),
        bounds_padding: Some(Padding::vertical(Dp(10.0))),
        ..Default::default()
    };
    layout()
        .modifier(
            modifier
                .unwrap_or_default()
                .clip_to_bounds()
                .semantics(semantics),
        )
        .policy(LinearWaveLayout)
        .draw(LinearWaveDraw {
            fractions,
            count: if progress.is_some() { 1 } else { 2 },
            amplitude: clock.amplitude(),
            phase: clock.phase as f32,
            wavelength,
            stroke_width: nonnegative(
                stroke_width.unwrap_or(WavyProgressIndicatorDefaults::STROKE_WIDTH),
            ),
            track_stroke_width: nonnegative(
                track_stroke_width.unwrap_or(WavyProgressIndicatorDefaults::STROKE_WIDTH),
            ),
            gap: nonnegative(gap_size.unwrap_or(WavyProgressIndicatorDefaults::TRACK_GAP)),
            stop: if progress.is_some() {
                nonnegative(stop_size.unwrap_or(WavyProgressIndicatorDefaults::STOP_SIZE))
            } else {
                Dp::ZERO
            },
            stroke_cap: stroke_cap.unwrap_or_default(),
            track_stroke_cap: track_stroke_cap.unwrap_or_default(),
            color: color.unwrap_or(scheme.primary),
            track_color: track_color.unwrap_or(scheme.secondary_container),
            reverse: reverse_direction.unwrap_or(false),
        });
}
