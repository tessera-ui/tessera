//! Material progress indicators.
//!
//! ## Usage
//!
//! Use to indicate the completion of a task or a specific value in a range.
use std::time::Instant;

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, Modifier, ParentConstraint, Px,
    PxPosition, accesskit::Role, remember, tessera, use_context,
};

use crate::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pipelines::progress_arc::command::{ProgressArcCap, ProgressArcCommand},
    shape_def::Shape,
    surface::{SurfaceArgs, surface},
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

fn resolve_dimension(dimension: DimensionValue, fallback: Px) -> Px {
    match dimension {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(fallback)
        }
    }
}

fn resolve_linear_size(parent: ParentConstraint<'_>) -> (Px, Px) {
    let fallback_width = ProgressIndicatorDefaults::LINEAR_INDICATOR_WIDTH.to_px();
    let fallback_height = ProgressIndicatorDefaults::LINEAR_INDICATOR_HEIGHT.to_px();
    let merged = Constraint::new(parent.width(), parent.height()).merge(parent);
    let width = resolve_dimension(merged.width, fallback_width);
    let height = resolve_dimension(merged.height, fallback_height);
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

/// Arguments for configuring a Material Design linear progress indicator.
#[derive(Clone, Debug, Setters)]
pub struct LinearProgressIndicatorArgs {
    /// Current progress in the range 0.0..=1.0.
    ///
    /// When omitted, the indicator renders in an indeterminate mode.
    #[setters(strip_option)]
    pub progress: Option<f32>,

    /// Modifier chain applied to the indicator subtree.
    pub modifier: Modifier,

    /// Color of the active indicator.
    pub color: Color,

    /// Color of the inactive track.
    pub track_color: Color,

    /// Stroke cap used for the indicator ends.
    pub stroke_cap: ProgressStrokeCap,

    /// Size of the gap between the active indicator and the track.
    pub gap_size: Dp,

    /// Whether to draw a stop indicator at the end of the track.
    pub draw_stop_indicator: bool,

    /// Optional accessibility label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,

    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl Default for LinearProgressIndicatorArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            progress: None,
            modifier: Modifier::new().size(
                ProgressIndicatorDefaults::LINEAR_INDICATOR_WIDTH,
                ProgressIndicatorDefaults::LINEAR_INDICATOR_HEIGHT,
            ),
            color: scheme.primary,
            track_color: scheme.secondary_container,
            stroke_cap: ProgressStrokeCap::default(),
            gap_size: ProgressIndicatorDefaults::LINEAR_INDICATOR_TRACK_GAP_SIZE,
            draw_stop_indicator: true,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
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
/// - `args` — configures the indicator; see [`LinearProgressIndicatorArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::progress::{
///     LinearProgressIndicatorArgs, linear_progress_indicator,
/// };
///
/// linear_progress_indicator(LinearProgressIndicatorArgs::default().progress(0.75));
/// # }
/// # component();
/// ```
#[tessera]
pub fn linear_progress_indicator(args: impl Into<LinearProgressIndicatorArgs>) {
    let args: LinearProgressIndicatorArgs = args.into();
    let modifier = args.modifier;
    modifier.run(move || linear_progress_indicator_inner(args));
}

#[tessera]
fn linear_progress_indicator_inner(args: LinearProgressIndicatorArgs) {
    let args_for_accessibility = args.clone();
    let animation_start = remember(Instant::now);

    let segment_shape = if args.stroke_cap == ProgressStrokeCap::Butt {
        Shape::RECTANGLE
    } else {
        Shape::capsule()
    };

    let mut semantics = SemanticsArgs::new().role(Role::ProgressIndicator);
    if let Some(label) = args_for_accessibility.accessibility_label.clone() {
        semantics = semantics.label(label);
    }
    if let Some(description) = args_for_accessibility.accessibility_description.clone() {
        semantics = semantics.description(description);
    }
    if let Some(progress) = args_for_accessibility.progress {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        semantics = semantics
            .numeric_range(0.0, 1.0)
            .numeric_value(progress as f64);
    }

    let args_for_children = args.clone();
    let args_for_measure = args;
    let animation_start_for_measure = animation_start;

    Modifier::new().semantics(semantics).run(move || {
        if args_for_children.progress.is_some() {
            surface(
                SurfaceArgs::default()
                    .style(args_for_children.track_color.into())
                    .shape(segment_shape)
                    .modifier(Modifier::new().fill_max_size()),
                || {},
            );
            surface(
                SurfaceArgs::default()
                    .style(args_for_children.color.into())
                    .shape(segment_shape)
                    .modifier(Modifier::new().fill_max_size()),
                || {},
            );
            if args_for_children.draw_stop_indicator {
                let stop_shape = if args_for_children.stroke_cap == ProgressStrokeCap::Butt {
                    Shape::RECTANGLE
                } else {
                    Shape::Ellipse
                };
                surface(
                    SurfaceArgs::default()
                        .style(args_for_children.color.into())
                        .shape(stop_shape)
                        .modifier(Modifier::new().fill_max_size()),
                    || {},
                );
            }
        } else {
            for (color, shape) in [
                (args_for_children.track_color, segment_shape),
                (args_for_children.color, segment_shape),
                (args_for_children.track_color, segment_shape),
                (args_for_children.color, segment_shape),
                (args_for_children.track_color, segment_shape),
            ] {
                surface(
                    SurfaceArgs::default()
                        .style(color.into())
                        .shape(shape)
                        .modifier(Modifier::new().fill_max_size()),
                    || {},
                );
            }
        }

        measure(Box::new(move |input| {
            let args = args_for_measure.clone();
            let (self_width, self_height) = resolve_linear_size(input.parent_constraint);
            let is_butt = args.stroke_cap.effective_is_butt(self_width, self_height);
            let gap_fraction =
                adjusted_linear_gap_fraction(self_width, self_height, args.gap_size, is_butt);
            let child_height = self_height;

            if let Some(progress) = args.progress {
                let progress = if progress.is_nan() {
                    0.0
                } else {
                    progress.clamp(0.0, 1.0)
                };
                let track_start = progress + progress.min(gap_fraction);

                let track_id = input.children_ids[0];
                let indicator_id = input.children_ids[1];
                let stop_id = if args.draw_stop_indicator {
                    input.children_ids.get(2).copied()
                } else {
                    None
                };

                if let Some((x, w)) =
                    linear_segment_bounds(0.0, progress, self_width, self_height, is_butt)
                {
                    let constraint = Constraint::new(
                        DimensionValue::Fixed(w),
                        DimensionValue::Fixed(child_height),
                    );
                    input.measure_child(indicator_id, &constraint)?;
                    input.place_child(indicator_id, PxPosition::new(x, Px(0)));
                } else {
                    let constraint = Constraint::new(
                        DimensionValue::Fixed(Px(0)),
                        DimensionValue::Fixed(child_height),
                    );
                    input.measure_child(indicator_id, &constraint)?;
                    input.place_child(indicator_id, PxPosition::new(Px(0), Px(0)));
                }

                if track_start <= 1.0
                    && let Some((x, w)) =
                        linear_segment_bounds(track_start, 1.0, self_width, self_height, is_butt)
                {
                    let constraint = Constraint::new(
                        DimensionValue::Fixed(w),
                        DimensionValue::Fixed(child_height),
                    );
                    input.measure_child(track_id, &constraint)?;
                    input.place_child(track_id, PxPosition::new(x, Px(0)));
                } else {
                    let constraint = Constraint::new(
                        DimensionValue::Fixed(Px(0)),
                        DimensionValue::Fixed(child_height),
                    );
                    input.measure_child(track_id, &constraint)?;
                    input.place_child(track_id, PxPosition::new(Px(0), Px(0)));
                }

                if let Some(stop_id) = stop_id {
                    let (pos, stop_size) = stop_indicator_bounds(self_width, self_height);
                    let constraint = Constraint::new(
                        DimensionValue::Fixed(stop_size),
                        DimensionValue::Fixed(stop_size),
                    );
                    input.measure_child(stop_id, &constraint)?;
                    input.place_child(stop_id, pos);
                }
            } else {
                let cycle = linear_cycle_progress(animation_start_for_measure.get(), 1750);
                let first_head = keyframe_0_to_1(cycle, 0, 1000, 1750, emphasized_accelerate);
                let first_tail = keyframe_0_to_1(cycle, 250, 1000, 1750, emphasized_accelerate);
                let second_head = keyframe_0_to_1(cycle, 650, 850, 1750, emphasized_accelerate);
                let second_tail = keyframe_0_to_1(cycle, 900, 850, 1750, emphasized_accelerate);

                let ids = input.children_ids;
                let track_before_id = ids[0];
                let line1_id = ids[1];
                let track_between_id = ids[2];
                let line2_id = ids[3];
                let track_after_id = ids[4];

                let set_segment =
                    |node_id, start: f32, end: f32| -> Result<(), tessera_ui::MeasurementError> {
                        if let Some((x, w)) =
                            linear_segment_bounds(start, end, self_width, self_height, is_butt)
                        {
                            let constraint = Constraint::new(
                                DimensionValue::Fixed(w),
                                DimensionValue::Fixed(child_height),
                            );
                            input.measure_child(node_id, &constraint)?;
                            input.place_child(node_id, PxPosition::new(x, Px(0)));
                        } else {
                            let constraint = Constraint::new(
                                DimensionValue::Fixed(Px(0)),
                                DimensionValue::Fixed(child_height),
                            );
                            input.measure_child(node_id, &constraint)?;
                            input.place_child(node_id, PxPosition::new(Px(0), Px(0)));
                        }
                        Ok(())
                    };

                if first_head < 1.0 - gap_fraction {
                    let start = if first_head > 0.0 {
                        first_head + gap_fraction
                    } else {
                        0.0
                    };
                    set_segment(track_before_id, start, 1.0)?;
                } else {
                    set_segment(track_before_id, 0.0, 0.0)?;
                }

                if first_head - first_tail > 0.0 {
                    set_segment(line1_id, first_head, first_tail)?;
                } else {
                    set_segment(line1_id, 0.0, 0.0)?;
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
                    set_segment(track_between_id, start, end)?;
                } else {
                    set_segment(track_between_id, 0.0, 0.0)?;
                }

                if second_head - second_tail > 0.0 {
                    set_segment(line2_id, second_head, second_tail)?;
                } else {
                    set_segment(line2_id, 0.0, 0.0)?;
                }

                if second_tail > gap_fraction {
                    let end = if second_tail < 1.0 {
                        second_tail - gap_fraction
                    } else {
                        1.0
                    };
                    set_segment(track_after_id, 0.0, end)?;
                } else {
                    set_segment(track_after_id, 0.0, 0.0)?;
                }
            }

            Ok(ComputedData {
                width: self_width,
                height: self_height,
            })
        }));
    });
}

/// Arguments for configuring a Material Design circular progress indicator.
#[derive(Clone, Debug, Setters)]
pub struct CircularProgressIndicatorArgs {
    /// Current progress in the range 0.0..=1.0.
    ///
    /// When omitted, the indicator renders in an indeterminate mode.
    #[setters(strip_option)]
    pub progress: Option<f32>,

    /// Diameter of the indicator.
    pub diameter: Dp,

    /// Stroke width of the indicator.
    pub stroke_width: Dp,

    /// Color of the active indicator.
    pub color: Color,

    /// Color of the track behind the indicator.
    pub track_color: Color,

    /// Stroke cap used for the arc ends.
    pub stroke_cap: ProgressStrokeCap,

    /// Gap between the indicator and the track.
    pub gap_size: Dp,

    /// Optional accessibility label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,

    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl Default for CircularProgressIndicatorArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            progress: None,
            diameter: ProgressIndicatorDefaults::CIRCULAR_INDICATOR_DIAMETER,
            stroke_width: ProgressIndicatorDefaults::CIRCULAR_STROKE_WIDTH,
            color: scheme.primary,
            track_color: scheme.secondary_container,
            stroke_cap: ProgressStrokeCap::default(),
            gap_size: ProgressIndicatorDefaults::CIRCULAR_INDICATOR_TRACK_GAP_SIZE,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
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
/// - `args` — configures the indicator; see [`CircularProgressIndicatorArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::progress::{
///     CircularProgressIndicatorArgs, circular_progress_indicator,
/// };
///
/// circular_progress_indicator(CircularProgressIndicatorArgs::default().progress(0.6));
/// # }
/// # component();
/// ```
#[tessera]
pub fn circular_progress_indicator(args: impl Into<CircularProgressIndicatorArgs>) {
    let args: CircularProgressIndicatorArgs = args.into();
    let args_for_accessibility = args.clone();
    let animation_start = remember(Instant::now);

    let mut semantics = SemanticsArgs::new().role(Role::ProgressIndicator);
    if let Some(label) = args_for_accessibility.accessibility_label.clone() {
        semantics = semantics.label(label);
    }
    if let Some(description) = args_for_accessibility.accessibility_description.clone() {
        semantics = semantics.description(description);
    }
    if let Some(progress) = args_for_accessibility.progress {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        semantics = semantics
            .numeric_range(0.0, 1.0)
            .numeric_value(progress as f64);
    }

    Modifier::new().semantics(semantics).run(move || {
        measure(Box::new(move |input| {
            let diameter_px = args.diameter.to_px();
            let stroke_px = args.stroke_width.to_px();

            let is_butt = args.stroke_cap.effective_is_butt(diameter_px, diameter_px);
            let cap = if is_butt {
                ProgressArcCap::Butt
            } else {
                ProgressArcCap::Round
            };

            let start_base = 270.0;
            let gap_sweep = circular_gap_sweep_degrees(
                args.diameter,
                args.stroke_width,
                args.gap_size,
                is_butt,
            );

            if let Some(progress) = args.progress {
                let progress = if progress.is_nan() {
                    0.0
                } else {
                    progress.clamp(0.0, 1.0)
                };
                let sweep = progress * 360.0;
                let gap = sweep.min(gap_sweep);
                let track_start = start_base + sweep + gap;
                let track_sweep = 360.0 - sweep - gap * 2.0;

                if args.track_color.a > 0.0 && track_sweep > 0.0 {
                    input.metadata_mut().push_draw_command(ProgressArcCommand {
                        color: args.track_color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: track_start,
                        sweep_angle_degrees: track_sweep,
                        cap,
                    });
                }
                if args.color.a > 0.0 && sweep > 0.0 {
                    input.metadata_mut().push_draw_command(ProgressArcCommand {
                        color: args.color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: start_base,
                        sweep_angle_degrees: sweep,
                        cap,
                    });
                }
            } else {
                let elapsed_ms = Instant::now()
                    .saturating_duration_since(animation_start.get())
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

                if args.track_color.a > 0.0 && track_sweep > 0.0 {
                    input.metadata_mut().push_draw_command(ProgressArcCommand {
                        color: args.track_color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: track_start,
                        sweep_angle_degrees: track_sweep,
                        cap,
                    });
                }
                if args.color.a > 0.0 && sweep > 0.0 {
                    input.metadata_mut().push_draw_command(ProgressArcCommand {
                        color: args.color,
                        stroke_width_px: stroke_px.to_f32(),
                        start_angle_degrees: rotation,
                        sweep_angle_degrees: sweep,
                        cap,
                    });
                }
            }

            Ok(ComputedData {
                width: diameter_px,
                height: diameter_px,
            })
        }));
    });
}

/// Arguments for the `progress` component.
#[derive(Clone, Debug, Setters)]
pub struct ProgressArgs {
    /// The current value of the progress bar, ranging from 0.0 to 1.0.
    pub value: f32,

    /// Modifier chain applied to the progress bar subtree.
    pub modifier: Modifier,

    /// The color of the active part of the track.
    pub progress_color: Color,

    /// The color of the inactive part of the track.
    pub track_color: Color,
}

impl Default for ProgressArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            value: 0.0,
            modifier: Modifier::new().size(
                ProgressIndicatorDefaults::LINEAR_INDICATOR_WIDTH,
                ProgressIndicatorDefaults::LINEAR_INDICATOR_HEIGHT,
            ),
            progress_color: scheme.primary,
            track_color: scheme.surface_variant,
        }
    }
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
/// - `args` — configures the progress bar's value and appearance; see
///   [`ProgressArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::progress::{ProgressArgs, progress};
///
/// // Creates a progress bar that is 75% complete.
/// progress(ProgressArgs::default().value(0.75));
/// # }
/// # component();
/// ```
#[tessera]
pub fn progress(args: impl Into<ProgressArgs>) {
    let args: ProgressArgs = args.into();

    linear_progress_indicator(
        LinearProgressIndicatorArgs::default()
            .progress(args.value)
            .modifier(args.modifier)
            .color(args.progress_color)
            .track_color(args.track_color),
    );
}
