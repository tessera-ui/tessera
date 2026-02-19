use tessera_ui::{Constraint, DimensionValue, Dp, ParentConstraint, Px, PxPosition};

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
        let active_width = self.active_width(value);
        let center_x =
            active_width.to_f32() + self.handle_gap.to_f32() + self.handle_width.to_f32() / 2.0;
        let max_x = (self.component_width.to_f32() - self.handle_width.to_f32() / 2.0).max(0.0);
        let clamped_x = center_x.clamp(self.handle_width.to_f32() / 2.0, max_x);
        PxPosition::new(
            Px(clamped_x.round() as i32),
            Px(self.component_height.0 / 2),
        )
    }
}

#[derive(Clone, Copy)]
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
        let h_gap = self.base.handle_gap.to_f32(); // Handle gap
        let center_x_track = w / 2.0; // Geometric center of the component, for tracks

        // Calculate Handle Center X using base logic.
        // This maps the 0.0-1.0 value to the physical X position of the handle's
        // center.
        let track_total_length = self.base.track_total_width.to_f32();
        let handle_center_x_raw = (value * track_total_length) + h_gap + (h_w / 2.0);

        // Clamp handle center X within component boundaries, considering handle width
        // and its gaps.
        let min_handle_center_x = h_w / 2.0; // Handle's left edge at 0
        let max_handle_center_x = w - h_w / 2.0; // Handle's right edge at w
        let handle_center_x = handle_center_x_raw.clamp(min_handle_center_x, max_handle_center_x);

        let handle_left = handle_center_x - h_w / 2.0;
        let handle_right = handle_center_x + h_w / 2.0;

        let (li_x, li_w, a_x, a_w, ri_x, ri_w): (f32, f32, f32, f32, f32, f32) = if value > 0.5 {
            // Handle is to the right of center_x_track
            // Left Inactive: From 0 to the start of the active segment, accounting for a
            // single h_gap at the center.
            let li_x_calc = 0.0;
            let li_w_calc = (center_x_track - h_gap / 2.0).max(0.0);

            // Active: From end of left inactive to start of handle's left gap.
            // This segment starts after the h_gap at the center and ends before the
            // handle's left h_gap.
            let a_x_calc = center_x_track + h_gap / 2.0;
            let a_w_calc = (handle_left - h_gap - a_x_calc).max(0.0); // Ensure width is non-negative

            // Right Inactive: From end of handle's right gap to component width.
            let ri_x_calc = handle_right + h_gap;
            let ri_w_calc = (w - ri_x_calc).max(0.0);

            (
                li_x_calc, li_w_calc, a_x_calc, a_w_calc, ri_x_calc, ri_w_calc,
            )
        } else {
            // Handle is to the left of or at center_x_track
            // Left Inactive: From 0 to start of handle's left gap.
            let li_x_calc = 0.0;
            let li_w_calc = (handle_left - h_gap).max(0.0);

            // Active: From end of handle's right gap to before the h_gap at the center.
            let a_x_calc = handle_right + h_gap;
            let a_w_calc = (center_x_track - h_gap / 2.0 - a_x_calc).max(0.0);

            // Right Inactive: From after the h_gap at the center to component width.
            let ri_x_calc = center_x_track + h_gap / 2.0;
            let ri_w_calc = (w - ri_x_calc).max(0.0);

            (
                li_x_calc, li_w_calc, a_x_calc, a_w_calc, ri_x_calc, ri_w_calc,
            )
        };

        CenteredSegments {
            left_inactive: (Px(li_x.round() as i32), Px(li_w.round() as i32)),
            active: (Px(a_x.round() as i32), Px(a_w.round() as i32)),
            right_inactive: (Px(ri_x.round() as i32), Px(ri_w.round() as i32)),
            handle_center: PxPosition::new(
                Px(handle_center_x.round() as i32),
                Px(self.base.component_height.0 / 2),
            ),
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
    let specs = get_slider_specs(args.size);
    let fallback = Dp(260.0).to_px();
    let merged = Constraint::new(
        parent_constraint.width(),
        DimensionValue::Fixed(specs.track_height.to_px()),
    )
    .merge(parent_constraint);

    match merged.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(fallback)
        }
    }
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

#[derive(Clone, Copy)]
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

        let start_center_raw = (start * track_total) + gap + start_half;
        let end_center_raw = (end * track_total) + gap + start_half;

        let start_min = gap + start_half;
        let start_max = (w - gap - start_half).max(start_min);
        let end_min = start_min;
        let end_max = (w - gap - end_half).max(end_min);

        let start_handle_center_x = start_center_raw.clamp(start_min, start_max);
        let end_handle_center_x = end_center_raw.clamp(end_min, end_max);

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
            start_handle_center: PxPosition::new(
                Px(start_handle_center_x.round() as i32),
                Px(self.base.component_height.0 / 2),
            ),
            end_handle_center: PxPosition::new(
                Px(end_handle_center_x.round() as i32),
                Px(self.base.component_height.0 / 2),
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
