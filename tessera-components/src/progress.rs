//! Material progress indicators.
//!
//! ## Usage
//!
//! Use to indicate the completion of a task or a specific value in a range.
use tessera_ui::{
    Color, ComputedData, Constraint, Dp, LayoutResult, MeasurementError, Modifier,
    ParentConstraint, Px, PxPosition,
    accesskit::Role,
    layout::{LayoutPolicy, MeasureScope, RenderInput, RenderPolicy, layout},
    receive_frame_nanos, remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pipelines::progress_arc::command::{ProgressArcCap, ProgressArcCommand},
    shape_def::Shape,
    surface::surface,
    theme::MaterialTheme,
};

/// Stroke cap for progress indicators.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressStrokeCap {
    /// Rounded stroke ends.
    #[default]
    Round,
    /// Flat stroke ends.
    Butt,
}

impl ProgressStrokeCap {
    fn effective_is_butt(self, component_width: Px, component_height: Px) -> bool {
        self == ProgressStrokeCap::Butt || component_height.0 > component_width.0
    }
}

#[derive(Clone, PartialEq)]
struct LinearProgressLayout {
    progress: Option<f32>,
    stroke_cap: ProgressStrokeCap,
    gap_size: Dp,
    draw_stop_indicator: bool,
    animation_cycle: Option<f32>,
}

impl LayoutPolicy for LinearProgressLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let (self_width, self_height) = resolve_linear_size(input.parent_constraint());
        let is_butt = self.stroke_cap.effective_is_butt(self_width, self_height);
        let gap_fraction =
            adjusted_linear_gap_fraction(self_width, self_height, self.gap_size, is_butt);
        let child_height = self_height;

        let children = input.children();

        if let Some(progress) = self.progress {
            let progress = if progress.is_nan() {
                0.0
            } else {
                progress.clamp(0.0, 1.0)
            };
            let track_start = progress + progress.min(gap_fraction);

            let track = children[0];
            let indicator = children[1];
            let stop_id = if self.draw_stop_indicator {
                children.get(2).copied()
            } else {
                None
            };

            if let Some((x, w)) =
                linear_segment_bounds(0.0, progress, self_width, self_height, is_butt)
            {
                let constraint = Constraint::new(w, child_height);
                indicator.measure(&constraint)?;
                result.place_child(indicator, PxPosition::new(x, Px(0)));
            } else {
                let constraint = Constraint::new(Px(0), child_height);
                indicator.measure(&constraint)?;
                result.place_child(indicator, PxPosition::new(Px(0), Px(0)));
            }

            if track_start <= 1.0
                && let Some((x, w)) =
                    linear_segment_bounds(track_start, 1.0, self_width, self_height, is_butt)
            {
                let constraint = Constraint::new(w, child_height);
                track.measure(&constraint)?;
                result.place_child(track, PxPosition::new(x, Px(0)));
            } else {
                let constraint = Constraint::new(Px(0), child_height);
                track.measure(&constraint)?;
                result.place_child(track, PxPosition::new(Px(0), Px(0)));
            }

            if let Some(stop) = stop_id {
                let (pos, stop_size) = stop_indicator_bounds(self_width, self_height);
                let constraint = Constraint::new(stop_size, stop_size);
                stop.measure(&constraint)?;
                result.place_child(stop, pos);
            }
        } else {
            let cycle = self.animation_cycle.unwrap_or(0.0);
            let first_head = keyframe_0_to_1(cycle, 0, 1000, 1750, emphasized_accelerate);
            let first_tail = keyframe_0_to_1(cycle, 250, 1000, 1750, emphasized_accelerate);
            let second_head = keyframe_0_to_1(cycle, 650, 850, 1750, emphasized_accelerate);
            let second_tail = keyframe_0_to_1(cycle, 900, 850, 1750, emphasized_accelerate);

            let track_before = children[0];
            let line1 = children[1];
            let track_between = children[2];
            let line2 = children[3];
            let track_after = children[4];

            let mut set_segment = |child: tessera_ui::layout::LayoutChild<'_>,
                                   start: f32,
                                   end: f32|
             -> Result<(), MeasurementError> {
                if let Some((x, w)) =
                    linear_segment_bounds(start, end, self_width, self_height, is_butt)
                {
                    let constraint = Constraint::new(w, child_height);
                    child.measure(&constraint)?;
                    result.place_child(child, PxPosition::new(x, Px(0)));
                } else {
                    let constraint = Constraint::new(Px(0), child_height);
                    child.measure(&constraint)?;
                    result.place_child(child, PxPosition::new(Px(0), Px(0)));
                }
                Ok(())
            };

            if first_head < 1.0 - gap_fraction {
                let start = if first_head > 0.0 {
                    first_head + gap_fraction
                } else {
                    0.0
                };
                set_segment(track_before, start, 1.0)?;
            } else {
                set_segment(track_before, 0.0, 0.0)?;
            }

            if first_head - first_tail > 0.0 {
                set_segment(line1, first_head, first_tail)?;
            } else {
                set_segment(line1, 0.0, 0.0)?;
            }

            if first_tail > gap_fraction {
                let start = if second_head > 0.0 {
                    second_head + gap_fraction
                } else {
                    0.0
                };
                let end = if first_tail < 1.0 {
                    first_tail - gap_fraction
                } else {
                    1.0
                };
                set_segment(track_between, start, end)?;
            } else {
                set_segment(track_between, 0.0, 0.0)?;
            }

            if second_head - second_tail > 0.0 {
                set_segment(line2, second_head, second_tail)?;
            } else {
                set_segment(line2, 0.0, 0.0)?;
            }

            if second_tail > gap_fraction {
                let end = if second_tail < 1.0 {
                    second_tail - gap_fraction
                } else {
                    1.0
                };
                set_segment(track_after, 0.0, end)?;
            } else {
                set_segment(track_after, 0.0, 0.0)?;
            }
        }

        Ok(result.with_size(ComputedData {
            width: self_width,
            height: self_height,
        }))
    }
}

#[derive(Clone, PartialEq)]
struct CircularProgressLayout {
    progress: Option<f32>,
    diameter: Dp,
    stroke_width: Dp,
    color: Color,
    track_color: Color,
    stroke_cap: ProgressStrokeCap,
    gap_size: Dp,
    animation_start: Instant,
}

impl LayoutPolicy for CircularProgressLayout {
    fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let diameter_px = self.diameter.to_px();
        Ok(LayoutResult::new(ComputedData {
            width: diameter_px,
            height: diameter_px,
        }))
    }
}

impl RenderPolicy for CircularProgressLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
        let diameter_px = self.diameter.to_px();
        let stroke_px = self.stroke_width.to_px();

        let is_butt = self.stroke_cap.effective_is_butt(diameter_px, diameter_px);
        let cap = if is_butt {
            ProgressArcCap::Butt
        } else {
            ProgressArcCap::Round
        };

        let start_base = 270.0;
        let gap_sweep =
            circular_gap_sweep_degrees(self.diameter, self.stroke_width, self.gap_size, is_butt);

        let mut metadata = input.metadata_mut();

        if let Some(progress) = self.progress {
            let progress = if progress.is_nan() {
                0.0
            } else {
                progress.clamp(0.0, 1.0)
            };
            let sweep = progress * 360.0;
            let gap = sweep.min(gap_sweep);
            let track_start = start_base + sweep + gap;
            let track_sweep = 360.0 - sweep - gap * 2.0;

            if self.track_color.a > 0.0 && track_sweep > 0.0 {
                metadata
                    .fragment_mut()
                    .push_draw_command(ProgressArcCommand {
                        color: self.track_color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: track_start,
                        sweep_angle_degrees: track_sweep,
                        cap,
                    });
            }
            if self.color.a > 0.0 && sweep > 0.0 {
                metadata
                    .fragment_mut()
                    .push_draw_command(ProgressArcCommand {
                        color: self.color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: start_base,
                        sweep_angle_degrees: sweep,
                        cap,
                    });
            }
        } else {
            let elapsed_ms = Instant::now()
                .saturating_duration_since(self.animation_start)
                .as_millis() as f32;
            let cycle_ms = elapsed_ms % 6000.0;

            let global_rotation = (cycle_ms / 6000.0) * 1080.0;
            let additional_rotation = circular_additional_rotation_degrees(cycle_ms);
            let rotation = global_rotation + additional_rotation;

            let progress = circular_indeterminate_progress(cycle_ms);
            let sweep = progress * 360.0;
            let gap = sweep.min(gap_sweep);

            let track_start = rotation + sweep + gap;
            let track_sweep = 360.0 - sweep - gap * 2.0;

            if self.track_color.a > 0.0 && track_sweep > 0.0 {
                metadata
                    .fragment_mut()
                    .push_draw_command(ProgressArcCommand {
                        color: self.track_color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: track_start,
                        sweep_angle_degrees: track_sweep,
                        cap,
                    });
            }
            if self.color.a > 0.0 && sweep > 0.0 {
                metadata
                    .fragment_mut()
                    .push_draw_command(ProgressArcCommand {
                        color: self.color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: rotation,
                        sweep_angle_degrees: sweep,
                        cap,
                    });
            }
        }
    }
}

/// Material Design 3 defaults for progress indicators.
pub struct ProgressIndicatorDefaults;

impl ProgressIndicatorDefaults {
    /// Default width for linear progress indicators.
    pub const LINEAR_INDICATOR_WIDTH: Dp = Dp(240.0);
    /// Default height for linear progress indicators.
    pub const LINEAR_INDICATOR_HEIGHT: Dp = Dp(4.0);
    /// Default stop indicator size for linear progress indicators.
    pub const LINEAR_TRACK_STOP_INDICATOR_SIZE: Dp = Dp(4.0);
    /// Default gap between the indicator and the track for linear indicators.
    pub const LINEAR_INDICATOR_TRACK_GAP_SIZE: Dp = Dp(4.0);
    /// Maximum trailing inset for the stop indicator.
    pub const STOP_INDICATOR_TRAILING_SPACE: Dp = Dp(6.0);

    /// Default diameter for circular progress indicators.
    pub const CIRCULAR_INDICATOR_DIAMETER: Dp = Dp(40.0);
    /// Default stroke width for circular progress indicators.
    pub const CIRCULAR_STROKE_WIDTH: Dp = Dp(4.0);
    /// Default gap between the indicator and the track for circular indicators.
    pub const CIRCULAR_INDICATOR_TRACK_GAP_SIZE: Dp = Dp(4.0);
}

fn cubic_bezier(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    let u = 1.0 - t;
    (u * u * u * a) + (3.0 * u * u * t * b) + (3.0 * u * t * t * c) + (t * t * t * d)
}

fn cubic_bezier_easing(progress: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let x = progress.clamp(0.0, 1.0);
    let mut lo = 0.0;
    let mut hi = 1.0;
    let mut t = x;

    for _ in 0..16 {
        let mid = (lo + hi) * 0.5;
        let mid_x = cubic_bezier(mid, 0.0, x1, x2, 1.0);
        if mid_x < x {
            lo = mid;
        } else {
            hi = mid;
        }
        t = mid;
    }

    cubic_bezier(t, 0.0, y1, y2, 1.0).clamp(0.0, 1.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn linear_cycle_progress(start: Instant, duration_ms: u32) -> f32 {
    let elapsed_ms = Instant::now().saturating_duration_since(start).as_millis() as u64;
    let duration_ms = duration_ms.max(1) as u64;
    (elapsed_ms % duration_ms) as f32 / duration_ms as f32
}

fn keyframe_0_to_1(
    cycle_progress: f32,
    delay_ms: u32,
    duration_ms: u32,
    total_ms: u32,
    easing: fn(f32) -> f32,
) -> f32 {
    let t_ms = cycle_progress.clamp(0.0, 1.0) * total_ms as f32;
    let delay = delay_ms as f32;
    let duration = duration_ms.max(1) as f32;
    if t_ms <= delay {
        0.0
    } else if t_ms >= delay + duration {
        1.0
    } else {
        easing((t_ms - delay) / duration)
    }
}

fn emphasized_accelerate(progress: f32) -> f32 {
    cubic_bezier_easing(progress, 0.3, 0.0, 0.8, 0.15)
}

fn emphasized_decelerate(progress: f32) -> f32 {
    cubic_bezier_easing(progress, 0.05, 0.7, 0.1, 1.0)
}

fn standard_easing(progress: f32) -> f32 {
    cubic_bezier_easing(progress, 0.2, 0.0, 0.0, 1.0)
}

fn resolve_linear_size(parent: ParentConstraint<'_>) -> (Px, Px) {
    let fallback_width = ProgressIndicatorDefaults::LINEAR_INDICATOR_WIDTH.to_px();
    let fallback_height = ProgressIndicatorDefaults::LINEAR_INDICATOR_HEIGHT.to_px();
    let width = parent.width().clamp(fallback_width);
    let height = parent.height().clamp(fallback_height);
    (width, height)
}

fn linear_segment_bounds(
    start_fraction: f32,
    end_fraction: f32,
    width: Px,
    height: Px,
    is_butt: bool,
) -> Option<(Px, Px)> {
    if width.0 <= 0 || height.0 <= 0 {
        return None;
    }

    let start = start_fraction;
    let end = end_fraction;
    let min_frac = start.min(end);
    let max_frac = start.max(end);
    if (max_frac - min_frac) <= 0.0 {
        return None;
    }

    let w = width.to_f32();
    let bar_start = (min_frac * w).clamp(0.0, w);
    let bar_end = (max_frac * w).clamp(0.0, w);

    if is_butt {
        let seg_w = bar_end - bar_start;
        if seg_w <= 0.0 {
            None
        } else {
            Some((
                Px::saturating_from_f32(bar_start),
                Px::saturating_from_f32(seg_w),
            ))
        }
    } else {
        let stroke_cap_offset = height.to_f32() * 0.5;
        let adjusted_start = bar_start.clamp(stroke_cap_offset, w - stroke_cap_offset);
        let adjusted_end = bar_end.clamp(stroke_cap_offset, w - stroke_cap_offset);
        let seg_w = (adjusted_end - adjusted_start) + stroke_cap_offset * 2.0;
        if seg_w <= 0.0 {
            None
        } else {
            Some((
                Px::saturating_from_f32(adjusted_start - stroke_cap_offset),
                Px::saturating_from_f32(seg_w),
            ))
        }
    }
}

fn adjusted_linear_gap_fraction(width: Px, height: Px, gap_size: Dp, is_butt: bool) -> f32 {
    if width.0 <= 0 {
        return 0.0;
    }
    let adjusted_gap = if is_butt {
        gap_size
    } else {
        Dp(gap_size.0 + Dp::from_pixels_f32(height.to_f32()).0)
    };
    adjusted_gap.to_pixels_f32() / width.to_f32()
}

fn stop_indicator_bounds(width: Px, height: Px) -> (PxPosition, Px) {
    let stop_size_px = Px::saturating_from_f32(
        ProgressIndicatorDefaults::LINEAR_TRACK_STOP_INDICATOR_SIZE
            .to_pixels_f32()
            .min(height.to_f32()),
    );
    let max_stop_offset = ProgressIndicatorDefaults::STOP_INDICATOR_TRAILING_SPACE.to_px();
    let stop_offset =
        ((height.to_f32() - stop_size_px.to_f32()) / 2.0).min(max_stop_offset.to_f32());
    let x = width.to_f32() - stop_size_px.to_f32() - stop_offset;
    let y = (height.to_f32() - stop_size_px.to_f32()) / 2.0;
    (
        PxPosition::new(Px::saturating_from_f32(x), Px::saturating_from_f32(y)),
        stop_size_px,
    )
}

/// # linear_progress_indicator
///
/// Renders a Material Design progress indicator in a horizontal, linear form.
///
/// ## Usage
///
/// Display completion state for determinate work, or show ongoing activity when
/// the remaining duration is unknown.
///
/// ## Parameters
///
/// - `progress` — current progress in the range `0.0..=1.0`; `None` renders the
///   indeterminate indicator.
/// - `modifier` — optional modifier chain for size and layout.
/// - `color` — optional active indicator color.
/// - `track_color` — optional inactive track color.
/// - `stroke_cap` — optional stroke cap style.
/// - `gap_size` — optional gap size between the active indicator and the track.
/// - `draw_stop_indicator` — optional stop-indicator toggle.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::progress::linear_progress_indicator;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// linear_progress_indicator().progress(0.75);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn linear_progress_indicator(
    progress: Option<f32>,
    modifier: Option<Modifier>,
    color: Option<Color>,
    track_color: Option<Color>,
    stroke_cap: Option<ProgressStrokeCap>,
    gap_size: Option<Dp>,
    draw_stop_indicator: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let modifier = modifier.unwrap_or_else(|| {
        Modifier::new().size(
            ProgressIndicatorDefaults::LINEAR_INDICATOR_WIDTH,
            ProgressIndicatorDefaults::LINEAR_INDICATOR_HEIGHT,
        )
    });
    let color = color.unwrap_or(scheme.primary);
    let track_color = track_color.unwrap_or(scheme.secondary_container);
    let stroke_cap = stroke_cap.unwrap_or_default();
    let gap_size = gap_size.unwrap_or(ProgressIndicatorDefaults::LINEAR_INDICATOR_TRACK_GAP_SIZE);
    let draw_stop_indicator = draw_stop_indicator.unwrap_or(true);

    layout().modifier(modifier).child(move || {
        let animation_start = remember(Instant::now);
        let frame_tick = remember(|| 0_u64);
        let should_receive_frames = remember(|| progress.is_none());
        should_receive_frames.set(progress.is_none());

        if should_receive_frames.get() {
            receive_frame_nanos(move |frame_nanos| {
                if !should_receive_frames.get() {
                    return tessera_ui::FrameNanosControl::Stop;
                }
                frame_tick.set(frame_nanos);
                tessera_ui::FrameNanosControl::Continue
            });
        }

        let segment_shape = if stroke_cap == ProgressStrokeCap::Butt {
            Shape::RECTANGLE
        } else {
            Shape::CAPSULE
        };

        let mut semantics = SemanticsArgs {
            role: Some(Role::ProgressIndicator),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            ..Default::default()
        };
        if let Some(progress) = progress {
            let progress = if progress.is_nan() {
                0.0
            } else {
                progress.clamp(0.0, 1.0)
            };
            semantics.numeric_range = Some((0.0, 1.0));
            semantics.numeric_value = Some(progress as f64);
        }

        let stop_shape = if stroke_cap == ProgressStrokeCap::Butt {
            Shape::RECTANGLE
        } else {
            Shape::Ellipse
        };
        let animation_cycle = if progress.is_some() {
            None
        } else {
            Some(linear_cycle_progress(animation_start.get(), 1750))
        };

        layout()
            .modifier(Modifier::new().semantics(semantics))
            .layout_policy(LinearProgressLayout {
                progress,
                stroke_cap,
                gap_size,
                draw_stop_indicator,
                animation_cycle,
            })
            .child(move || {
                if progress.is_some() {
                    surface()
                        .style(track_color.into())
                        .shape(segment_shape)
                        .modifier(Modifier::new().fill_max_size())
                        .child(|| {});
                    surface()
                        .style(color.into())
                        .shape(segment_shape)
                        .modifier(Modifier::new().fill_max_size())
                        .child(|| {});
                    if draw_stop_indicator {
                        surface()
                            .style(color.into())
                            .shape(stop_shape)
                            .modifier(Modifier::new().fill_max_size())
                            .child(|| {});
                    }
                } else {
                    for (color, shape) in [
                        (track_color, segment_shape),
                        (color, segment_shape),
                        (track_color, segment_shape),
                        (color, segment_shape),
                        (track_color, segment_shape),
                    ] {
                        surface()
                            .style(color.into())
                            .shape(shape)
                            .modifier(Modifier::new().fill_max_size())
                            .child(|| {});
                    }
                }
            });
    });
}

fn circular_gap_sweep_degrees(diameter: Dp, stroke_width: Dp, gap_size: Dp, is_butt: bool) -> f32 {
    let adjusted_gap = if is_butt {
        gap_size
    } else {
        Dp(gap_size.0 + stroke_width.0)
    };
    let diameter_dp = diameter.0 as f32;
    let gap_dp = adjusted_gap.0 as f32;
    let circumference = std::f32::consts::PI * diameter_dp.max(0.0001);
    (gap_dp / circumference) * 360.0
}

fn circular_additional_rotation_degrees(cycle_ms: f32) -> f32 {
    fn segment(t: f32, start_ms: f32, end_ms: f32, from: f32, to: f32) -> Option<f32> {
        if t < start_ms {
            None
        } else if t >= end_ms {
            Some(to)
        } else {
            let local = (t - start_ms) / (end_ms - start_ms);
            Some(lerp(from, to, emphasized_decelerate(local)))
        }
    }

    let t = cycle_ms;
    if let Some(v) = segment(t, 0.0, 300.0, 0.0, 90.0) {
        return v;
    }
    if t < 1500.0 {
        return 90.0;
    }
    if let Some(v) = segment(t, 1500.0, 1800.0, 90.0, 180.0) {
        return v;
    }
    if t < 3000.0 {
        return 180.0;
    }
    if let Some(v) = segment(t, 3000.0, 3300.0, 180.0, 270.0) {
        return v;
    }
    if t < 4500.0 {
        return 270.0;
    }
    if let Some(v) = segment(t, 4500.0, 4800.0, 270.0, 360.0) {
        return v;
    }
    360.0
}

fn circular_indeterminate_progress(cycle_ms: f32) -> f32 {
    const MIN_PROGRESS: f32 = 0.1;
    const MAX_PROGRESS: f32 = 0.87;
    if cycle_ms <= 3000.0 {
        let t = (cycle_ms / 3000.0).clamp(0.0, 1.0);
        lerp(MIN_PROGRESS, MAX_PROGRESS, standard_easing(t))
    } else {
        let t = ((cycle_ms - 3000.0) / 3000.0).clamp(0.0, 1.0);
        lerp(MAX_PROGRESS, MIN_PROGRESS, t)
    }
}

/// # circular_progress_indicator
///
/// Renders a Material Design progress indicator in a circular form.
///
/// ## Usage
///
/// Show ongoing activity or a completion fraction for tasks such as loading,
/// syncing, or background work.
///
/// ## Parameters
///
/// - `progress` — current progress in the range `0.0..=1.0`; `None` renders the
///   indeterminate indicator.
/// - `diameter` — optional indicator diameter.
/// - `stroke_width` — optional stroke width.
/// - `color` — optional active indicator color.
/// - `track_color` — optional track color.
/// - `stroke_cap` — optional stroke cap style.
/// - `gap_size` — optional gap size between the indicator and the track.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::progress::circular_progress_indicator;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// circular_progress_indicator().progress(0.6);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn circular_progress_indicator(
    progress: Option<f32>,
    diameter: Option<Dp>,
    stroke_width: Option<Dp>,
    color: Option<Color>,
    track_color: Option<Color>,
    stroke_cap: Option<ProgressStrokeCap>,
    gap_size: Option<Dp>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let diameter = diameter.unwrap_or(ProgressIndicatorDefaults::CIRCULAR_INDICATOR_DIAMETER);
    let stroke_width = stroke_width.unwrap_or(ProgressIndicatorDefaults::CIRCULAR_STROKE_WIDTH);
    let color = color.unwrap_or(scheme.primary);
    let track_color = track_color.unwrap_or(scheme.secondary_container);
    let stroke_cap = stroke_cap.unwrap_or_default();
    let gap_size = gap_size.unwrap_or(ProgressIndicatorDefaults::CIRCULAR_INDICATOR_TRACK_GAP_SIZE);
    let animation_start = remember(Instant::now);
    let frame_tick = remember(|| 0_u64);
    let should_receive_frames = remember(|| progress.is_none());
    should_receive_frames.set(progress.is_none());

    if should_receive_frames.get() {
        receive_frame_nanos(move |frame_nanos| {
            if !should_receive_frames.get() {
                return tessera_ui::FrameNanosControl::Stop;
            }
            frame_tick.set(frame_nanos);
            tessera_ui::FrameNanosControl::Continue
        });
    }

    let mut semantics = SemanticsArgs {
        role: Some(Role::ProgressIndicator),
        label: accessibility_label.clone(),
        description: accessibility_description.clone(),
        ..Default::default()
    };
    if let Some(progress) = progress {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        semantics.numeric_range = Some((0.0, 1.0));
        semantics.numeric_value = Some(progress as f64);
    }

    let policy = CircularProgressLayout {
        progress,
        diameter,
        stroke_width,
        color,
        track_color,
        stroke_cap,
        gap_size,
        animation_start: animation_start.get(),
    };
    layout()
        .modifier(Modifier::new().semantics(semantics))
        .layout_policy(policy.clone())
        .render_policy(policy);
}

/// # progress
///
/// Renders a linear progress indicator that visualizes a value from 0.0 to 1.0.
///
/// ## Usage
///
/// Display the status of an ongoing operation, such as a download or a setup
/// process.
///
/// ## Parameters
///
/// - `value` — current progress value in the range `0.0..=1.0`.
/// - `modifier` — optional modifier chain for size and layout.
/// - `progress_color` — optional active track color.
/// - `track_color` — optional inactive track color.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::progress::progress;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// // Creates a progress bar that is 75% complete.
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// progress().value(0.75);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn progress(
    value: Option<f32>,
    modifier: Option<Modifier>,
    progress_color: Option<Color>,
    track_color: Option<Color>,
) {
    let value = value.unwrap_or(0.0);
    linear_progress_indicator()
        .progress(value)
        .modifier(modifier.unwrap_or_else(|| {
            Modifier::new().size(
                ProgressIndicatorDefaults::LINEAR_INDICATOR_WIDTH,
                ProgressIndicatorDefaults::LINEAR_INDICATOR_HEIGHT,
            )
        }))
        .color(progress_color.unwrap_or_else(|| {
            use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .primary
        }))
        .track_color(track_color.unwrap_or_else(|| {
            use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .surface_variant
        }));
}
