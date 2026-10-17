use tessera_ui::{Dp, ParentConstraint, Px, PxPosition};

use super::{HANDLE_GAP, MIN_TOUCH_TARGET, STOP_INDICATOR_DIAMETER, SliderArgs, SliderSize};

struct SliderSpecs {
    track_height: Dp,
    handle_height: Dp,
    track_corner_radius: Dp,
    icon_size: Option<Dp>,
}

fn get_slider_specs(size: SliderSize) -> SliderSpecs {
    match size {
        SliderSize::ExtraSmall => SliderSpecs {
            track_height: Dp(16.0),
            handle_height: Dp(44.0),
            track_corner_radius: Dp(8.0),
            icon_size: None,
        },
        SliderSize::Small => SliderSpecs {
            track_height: Dp(24.0),
            handle_height: Dp(44.0),
            track_corner_radius: Dp(8.0),
            icon_size: None,
        },
        SliderSize::Medium => SliderSpecs {
            track_height: Dp(40.0),
            handle_height: Dp(52.0),
            track_corner_radius: Dp(12.0),
            icon_size: Some(Dp(24.0)),
        },
        SliderSize::Large => SliderSpecs {
            track_height: Dp(56.0),
            handle_height: Dp(68.0),
            track_corner_radius: Dp(16.0),
            icon_size: Some(Dp(24.0)),
        },
        SliderSize::ExtraLarge => SliderSpecs {
            track_height: Dp(96.0),
            handle_height: Dp(108.0),
            track_corner_radius: Dp(28.0),
            icon_size: Some(Dp(32.0)),
        },
    }
}

const INNER_CORNER_RADIUS: Dp = Dp(2.0);

#[derive(Clone, Copy, PartialEq)]
pub(super) struct SliderLayout {
    pub component_width: Px,
    pub component_height: Px,
    pub track_total_width: Px,
    pub track_height: Px,
    pub track_corner_radius: Dp,
    pub inner_corner_radius: Dp,
    pub track_y: Px,
    pub handle_width: Px,
    pub handle_height: Px,
    pub handle_y: Px,
    pub handle_gap: Px,
    pub stop_indicator_diameter: Px,
    pub stop_indicator_y: Px,
    pub show_stop_indicator: bool,
    pub icon_size: Option<Dp>,
}

impl SliderLayout {
    pub fn active_width(&self, value: f32) -> Px {
        let clamped = value.clamp(0.0, 1.0);
        Px::saturating_from_f32(self.track_total_width.to_f32() * clamped)
    }

    pub fn inactive_width(&self, value: f32) -> Px {
        let active = self.active_width(value);
        Px((self.track_total_width.0 - active.0).max(0))
    }

    pub fn center_child_offset(&self, width: Px) -> Px {
        Px(width.0 / 2)
    }

    pub fn handle_center(&self, value: f32) -> PxPosition {
        if self.track_total_width <= Px::ZERO {
            return PxPosition::from_f32_xy(
                self.component_width.to_f32().max(0.0) / 2.0,
                self.component_height.to_f32() / 2.0,
            );
        }
        let active_width = self.active_width(value);
        let center_x =
            active_width.to_f32() + self.handle_gap.to_f32() + self.handle_width.to_f32() / 2.0;
        let max_x = (self.component_width.to_f32() - self.handle_width.to_f32() / 2.0).max(0.0);
        let clamped_x = center_x.clamp(self.handle_width.to_f32() / 2.0, max_x);
        PxPosition::from_f32_xy(clamped_x, self.component_height.to_f32() / 2.0)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct CenteredSliderLayout {
    pub base: SliderLayout,
}

pub(super) struct CenteredSegments {
    pub left_inactive: (Px, Px),  // x, width
    pub active: (Px, Px),         // x, width
    pub right_inactive: (Px, Px), // x, width
    pub handle_center: PxPosition,
}

impl CenteredSliderLayout {
    pub fn segments(&self, value: f32) -> CenteredSegments {
        let value = value.clamp(0.0, 1.0);
        let w = self.base.component_width.to_f32();
        let h_w = self.base.handle_width.to_f32();
        let gap = self.base.handle_gap.to_f32();
        let midpoint = w / 2.0;
        let handle_center = self.base.handle_center(value);
        let handle_left = handle_center.x - h_w / 2.0;
        let handle_right = handle_center.x + h_w / 2.0;
        let left_end = (handle_left - gap).min(midpoint - gap / 2.0).max(0.0);
        let right_start = (handle_right + gap).max(midpoint + gap / 2.0).min(w);
        let (active_start, active_end) = if value > 0.5 {
            let start = midpoint + gap / 2.0;
            (start, (handle_left - gap).max(start))
        } else {
            let start = handle_right + gap;
            (start, (midpoint - gap / 2.0).max(start))
        };
        let active_start = Px(active_start.round() as i32);
        let active_end = Px(active_end.round() as i32);
        let right_start = Px(right_start.round() as i32);

        CenteredSegments {
            left_inactive: (Px::ZERO, Px(left_end.round() as i32)),
            active: (active_start, active_end - active_start),
            right_inactive: (right_start, self.base.component_width - right_start),
            handle_center,
        }
    }

    pub fn stop_indicator_offset(&self) -> Px {
        self.base.track_corner_radius.to_px()
    }
}

pub(super) fn resolve_component_width(
    args: &SliderArgs,
    parent_constraint: ParentConstraint<'_>,
) -> Px {
    let fallback = Dp(260.0).to_px();
    let _ = args;
    parent_constraint.width().clamp(fallback)
}

pub(super) fn fallback_component_width(args: &SliderArgs) -> Px {
    let _ = args;
    Dp(260.0).to_px()
}

fn slider_layout_from_parts(
    size: SliderSize,
    show_stop_indicator: bool,
    component_width: Px,
    handle_width: Px,
) -> SliderLayout {
    let specs = get_slider_specs(size);

    let track_height = specs.track_height.to_px();
    let touch_target_height = MIN_TOUCH_TARGET.to_px();
    let handle_gap = HANDLE_GAP.to_px();
    let handle_height = specs.handle_height.to_px();
    let stop_indicator_diameter = STOP_INDICATOR_DIAMETER.to_px();
    let track_corner_radius = specs.track_corner_radius;

    let track_total_width = Px((component_width.0 - handle_width.0 - handle_gap.0 * 2).max(0));

    let component_height = Px(track_height
        .0
        .max(handle_height.0)
        .max(touch_target_height.0));
    let track_y = Px((component_height.0 - track_height.0) / 2);

    SliderLayout {
        component_width,
        component_height,
        track_total_width,
        track_height,
        track_corner_radius,
        inner_corner_radius: INNER_CORNER_RADIUS,
        track_y,
        handle_width,
        handle_height,
        handle_gap,
        handle_y: Px((component_height.0 - handle_height.0) / 2),
        stop_indicator_diameter,
        stop_indicator_y: Px((component_height.0 - stop_indicator_diameter.0) / 2),
        show_stop_indicator,
        icon_size: specs.icon_size,
    }
}

pub(super) fn slider_layout_with_handle_width(
    args: &SliderArgs,
    component_width: Px,
    handle_width: Px,
) -> SliderLayout {
    slider_layout_from_parts(
        args.size,
        args.show_stop_indicator,
        component_width,
        handle_width,
    )
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct RangeSliderLayout {
    pub base: SliderLayout,
}

pub(super) struct RangeSegments {
    pub left_inactive: (Px, Px),  // x, width
    pub active: (Px, Px),         // x, width
    pub right_inactive: (Px, Px), // x, width
    pub start_handle_center: PxPosition,
    pub end_handle_center: PxPosition,
}

impl RangeSliderLayout {
    pub fn segments(
        &self,
        start: f32,
        end: f32,
        start_handle_width: Px,
        end_handle_width: Px,
    ) -> RangeSegments {
        let start = start.clamp(0.0, 1.0);
        let end = end.clamp(start, 1.0); // Ensure start <= end

        let w = self.base.component_width.to_f32();
        let gap = self.base.handle_gap.to_f32();
        let start_half = start_handle_width.to_f32() / 2.0;
        let end_half = end_handle_width.to_f32() / 2.0;
        let track_total = (w - start_half - end_half - gap * 2.0).max(0.0);
        if track_total == 0.0 {
            let center = PxPosition::from_f32_xy(
                w.max(0.0) / 2.0,
                self.base.component_height.to_f32() / 2.0,
            );
            return RangeSegments {
                left_inactive: (Px::ZERO, Px::ZERO),
                active: (Px::ZERO, Px::ZERO),
                right_inactive: (Px::ZERO, Px::ZERO),
                start_handle_center: center,
                end_handle_center: center,
            };
        }

        let start_center_raw = (start * track_total) + gap + start_half;
        let end_center_raw = (end * track_total) + gap + start_half;

        let start_min = gap + start_half;
        let start_max = (w - gap - start_half).max(start_min);
        let end_min = start_min;
        let end_max = (w - gap - end_half).max(end_min);

        let start_handle_center_x =
            (start_center_raw.clamp(start_min, start_max) - start_half).round() + start_half;
        let end_handle_center_x =
            (end_center_raw.clamp(end_min, end_max) - end_half).round() + end_half;

        let start_handle_right = start_handle_center_x + start_half;
        let end_handle_left = end_handle_center_x - end_half;

        // Left Inactive: 0 to StartHandleLeft - Gap
        let start_handle_left = start_handle_center_x - start_half;
        let li_end = (start_handle_left - gap).max(0.0);
        let li_w = li_end;
        let li_x: f32 = 0.0;

        // Active: StartHandleRight + Gap to EndHandleLeft - Gap
        let a_start = start_handle_right + gap;
        let a_end = (end_handle_left - gap).max(a_start);
        let a_w = a_end - a_start;
        let a_x = a_start;

        // Right Inactive: EndHandleRight + Gap to Width
        let end_handle_right = end_handle_center_x + end_half;
        let ri_start = end_handle_right + gap;
        let ri_end = w;
        let ri_w = (ri_end - ri_start).max(0.0);
        let ri_x = ri_start;

        RangeSegments {
            left_inactive: (Px(li_x.round() as i32), Px(li_w.round() as i32)),
            active: (Px(a_x.round() as i32), Px(a_w.round() as i32)),
            right_inactive: (Px(ri_x.round() as i32), Px(ri_w.round() as i32)),
            start_handle_center: PxPosition::from_f32_xy(
                start_handle_center_x,
                self.base.component_height.to_f32() / 2.0,
            ),
            end_handle_center: PxPosition::from_f32_xy(
                end_handle_center_x,
                self.base.component_height.to_f32() / 2.0,
            ),
        }
    }
}

pub(super) fn range_slider_layout(
    args: &super::RangeSliderArgs,
    component_width: Px,
) -> RangeSliderLayout {
    RangeSliderLayout {
        base: slider_layout_from_parts(
            args.size,
            args.show_stop_indicator,
            component_width,
            args.thumb_diameter.to_px(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_and_narrow_sliders_keep_a_finite_center_without_reversed_clamps() {
        for handle_width in [2, 3, 4, 6, 8] {
            for width in 0..64 {
                let base = slider_layout_from_parts(
                    SliderSize::ExtraSmall,
                    true,
                    Px(width),
                    Px(handle_width),
                );
                for value in [0.0, 0.2, 0.5, 0.8, 1.0] {
                    let center = base.handle_center(value);
                    assert!(center.x.is_finite() && center.y.is_finite());
                    assert!((0.0..=width as f32).contains(&center.x));
                    if base.track_total_width == Px::ZERO {
                        assert_eq!(center.x, width as f32 / 2.0);
                    }
                    let centered = CenteredSliderLayout { base }.segments(value);
                    assert!(centered.left_inactive.1 >= Px::ZERO);
                    assert!(centered.active.1 >= Px::ZERO);
                    assert!(centered.right_inactive.1 >= Px::ZERO);
                    let range = RangeSliderLayout { base }.segments(
                        0.2,
                        0.8,
                        Px(handle_width),
                        Px(handle_width),
                    );
                    if base.track_total_width == Px::ZERO {
                        assert_eq!(range.start_handle_center, center);
                        assert_eq!(range.end_handle_center, center);
                        assert_eq!(range.active.1, Px::ZERO);
                    }
                }
            }
        }
    }

    #[test]
    fn centered_tracks_preserve_both_handle_gaps_at_and_near_the_midpoint() {
        for width in [279, 280, 281, 350, 420, 560] {
            for handle_width in [2, 3, 4, 5, 6, 8] {
                let layout = CenteredSliderLayout {
                    base: slider_layout_from_parts(
                        SliderSize::ExtraSmall,
                        true,
                        Px(width),
                        Px(handle_width),
                    ),
                };
                let gap = layout.base.handle_gap.to_f32();
                for value in [0.0, 0.2, 0.49, 0.499, 0.5, 0.501, 0.51, 0.8, 1.0] {
                    let segments = layout.segments(value);
                    let left = segments.handle_center.x - handle_width as f32 / 2.0;
                    let right = left + handle_width as f32;
                    let left_end = (segments.left_inactive.0 + segments.left_inactive.1).to_f32();
                    let right_start = segments.right_inactive.0.to_f32();
                    assert!(
                        left - left_end >= gap,
                        "left gap at {width}/{handle_width}/{value}"
                    );
                    assert!(
                        right_start - right >= gap,
                        "right gap at {width}/{handle_width}/{value}"
                    );
                    if value == 0.5 {
                        assert_eq!(left - left_end, gap);
                        assert_eq!(right_start - right, gap);
                        assert_eq!(segments.active.1, Px::ZERO);
                        assert!((segments.handle_center.x - width as f32 / 2.0).abs() <= 0.5);
                    }
                    if segments.active.1 > Px::ZERO {
                        let active_start = segments.active.0.to_f32();
                        let active_end = (segments.active.0 + segments.active.1).to_f32();
                        assert!(active_end <= left - gap || active_start >= right + gap);
                    }
                }
            }
        }
    }

    #[test]
    fn range_tracks_preserve_gaps_with_odd_and_different_handle_widths() {
        for width in [279, 280, 281, 350, 420] {
            for (start_width, end_width) in [(3, 3), (3, 6), (6, 3), (4, 4)] {
                let layout = RangeSliderLayout {
                    base: slider_layout_from_parts(SliderSize::ExtraSmall, true, Px(width), Px(6)),
                };
                let segments = layout.segments(0.2, 0.8, Px(start_width), Px(end_width));
                let gap = layout.base.handle_gap.to_f32();
                let start_left = segments.start_handle_center.x - start_width as f32 / 2.0;
                let end_left = segments.end_handle_center.x - end_width as f32 / 2.0;
                assert_eq!(start_left - segments.left_inactive.1.to_f32(), gap);
                assert_eq!(
                    segments.active.0.to_f32() - start_left - start_width as f32,
                    gap
                );
                assert_eq!(
                    end_left - (segments.active.0 + segments.active.1).to_f32(),
                    gap
                );
                assert_eq!(
                    segments.right_inactive.0.to_f32() - end_left - end_width as f32,
                    gap
                );
            }
        }
    }
}
