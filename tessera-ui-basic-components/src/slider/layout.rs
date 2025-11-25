use tessera_ui::{Constraint, DimensionValue, Dp, Px, PxPosition};

use super::{
    HANDLE_GAP, HANDLE_HEIGHT, MIN_TOUCH_TARGET, STOP_INDICATOR_DIAMETER, SliderArgs, TRACK_HEIGHT,
};

#[derive(Clone, Copy)]
pub(super) struct SliderLayout {
    pub component_width: Px,
    pub component_height: Px,
    pub track_total_width: Px,
    pub track_height: Px,
    pub track_corner_radius: Dp,
    pub track_y: Px,
    pub handle_width: Px,
    pub handle_height: Px,
    pub handle_y: Px,
    pub handle_gap: Px,
    pub focus_width: Px,
    pub focus_height: Px,
    pub focus_y: Px,
    pub stop_indicator_diameter: Px,
    pub stop_indicator_y: Px,
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
        let gap = self.base.handle_gap.to_f32();
        let center_x = w / 2.0;

        // Calculate Handle Center X using base logic
        // We can't just call self.base.handle_center(value) because it returns PxPosition
        // and we need floats for intermediate calcs, but we can reuse the logic.
        let track_total = self.base.track_total_width.to_f32();
        // Mapping: 0.0 -> gap + h/2, 1.0 -> W - gap - h/2
        // active_width (for value) = value * track_total
        // x = active_width + gap + h/2
        let handle_center_x_raw = (value * track_total) + gap + (h_w / 2.0);
        let max_x = (w - h_w / 2.0).max(0.0);
        let handle_center_x = handle_center_x_raw.clamp(h_w / 2.0, max_x);

        let handle_left = handle_center_x - h_w / 2.0;
        let handle_right = handle_center_x + h_w / 2.0;

        let (li_x, li_w, a_x, a_w, ri_x, ri_w): (f32, f32, f32, f32, f32, f32) = if value > 0.5 {
            // Handle is to the right
            // Left Inactive: 0 to min(Center, HandleLeft) - Gap
            let li_end = (center_x.min(handle_left) - gap).max(0.0);
            let li_w = li_end;

            // Active: Center + Gap to HandleLeft - Gap
            let a_start = center_x + gap;
            let a_end = (handle_left - gap).max(a_start);
            let a_w = a_end - a_start;

            // Right Inactive: HandleRight + Gap to Width
            let ri_start = handle_right + gap;
            let ri_end = w;
            let ri_w = (ri_end - ri_start).max(0.0);

            (0.0, li_w, a_start, a_w, ri_start, ri_w)
        } else {
            // Handle is to the left (or center)
            // Left Inactive: 0 to HandleLeft - Gap
            let li_end = (handle_left - gap).max(0.0);
            let li_w = li_end;

            // Active: HandleRight + Gap to Center - Gap
            let a_start = handle_right + gap;
            let a_end = (center_x - gap).max(a_start);
            let a_w = a_end - a_start;

            // Right Inactive: Center + Gap to Width
            let ri_start = center_x + gap;
            let ri_end = w;
            let ri_w = (ri_end - ri_start).max(0.0);

            (0.0, li_w, a_start, a_w, ri_start, ri_w)
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
        // Replicating padding logic: Dp(8.0) - size/2

        Dp(8.0).to_px() - self.base.stop_indicator_diameter / Px(2)
    }
}

pub(super) fn resolve_component_width(args: &SliderArgs, parent_constraint: &Constraint) -> Px {
    let fallback = Dp(260.0).to_px();
    let merged = Constraint::new(args.width, DimensionValue::Fixed(TRACK_HEIGHT.to_px()))
        .merge(parent_constraint);

    match merged.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(fallback)
        }
    }
}

pub(super) fn fallback_component_width(args: &SliderArgs) -> Px {
    match args.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(Dp(260.0).to_px())
        }
    }
}

pub(super) fn slider_layout(args: &SliderArgs, component_width: Px) -> SliderLayout {
    let handle_width = args.thumb_diameter.to_px();
    let track_height = TRACK_HEIGHT.to_px();
    let touch_target_height = MIN_TOUCH_TARGET.to_px();
    let handle_gap = HANDLE_GAP.to_px();
    let handle_height = HANDLE_HEIGHT.to_px();
    let focus_width = Px((handle_width.to_f32() * 1.6).round() as i32);
    let focus_height = Px((handle_height.to_f32() * 1.2).round() as i32);
    let stop_indicator_diameter = STOP_INDICATOR_DIAMETER.to_px();
    let track_corner_radius = Dp(TRACK_HEIGHT.0 / 2.0);

    let track_total_width = Px((component_width.0 - handle_width.0 - handle_gap.0 * 2).max(0));

    let component_height = Px(*[
        track_height.0,
        handle_height.0,
        focus_height.0,
        touch_target_height.0,
    ]
    .iter()
    .max()
    .expect("non-empty"));
    let track_y = Px((component_height.0 - track_height.0) / 2);

    SliderLayout {
        component_width,
        component_height,
        track_total_width,
        track_height,
        track_corner_radius,
        track_y,
        handle_width,
        handle_height,
        handle_gap,
        handle_y: Px((component_height.0 - handle_height.0) / 2),
        focus_width,
        focus_height,
        focus_y: Px((component_height.0 - focus_height.0) / 2),
        stop_indicator_diameter,
        stop_indicator_y: Px((component_height.0 - stop_indicator_diameter.0) / 2),
    }
}

pub(super) fn centered_slider_layout(
    args: &SliderArgs,
    component_width: Px,
) -> CenteredSliderLayout {
    CenteredSliderLayout {
        base: slider_layout(args, component_width),
    }
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
    pub fn segments(&self, start: f32, end: f32) -> RangeSegments {
        let start = start.clamp(0.0, 1.0);
        let end = end.clamp(start, 1.0); // Ensure start <= end

        let w = self.base.component_width.to_f32();
        let h_w = self.base.handle_width.to_f32();
        let gap = self.base.handle_gap.to_f32();
        let track_total = self.base.track_total_width.to_f32();

        // Handle Centers
        // Mapping: 0.0 -> gap + h/2, 1.0 -> W - gap - h/2
        // active_width (for value) = value * track_total
        // x = active_width + gap + h/2

        let start_center_raw = (start * track_total) + gap + (h_w / 2.0);
        let end_center_raw = (end * track_total) + gap + (h_w / 2.0);

        let max_x = (w - h_w / 2.0).max(0.0);

        let start_handle_center_x = start_center_raw.clamp(h_w / 2.0, max_x);
        let end_handle_center_x = end_center_raw.clamp(h_w / 2.0, max_x);

        let start_handle_right = start_handle_center_x + h_w / 2.0;
        let end_handle_left = end_handle_center_x - h_w / 2.0;

        // Left Inactive: 0 to StartHandleLeft - Gap
        let start_handle_left = start_handle_center_x - h_w / 2.0;
        let li_end = (start_handle_left - gap).max(0.0);
        let li_w = li_end;
        let li_x: f32 = 0.0;

        // Active: StartHandleRight + Gap to EndHandleLeft - Gap
        let a_start = start_handle_right + gap;
        let a_end = (end_handle_left - gap).max(a_start);
        let a_w = a_end - a_start;
        let a_x = a_start;

        // Right Inactive: EndHandleRight + Gap to Width
        let end_handle_right = end_handle_center_x + h_w / 2.0;
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
    // Reuse basic slider layout logic for dimensions, but we need to construct a dummy SliderArgs
    // or refactor slider_layout. Since slider_layout mainly uses width and style args which exist
    // in RangeSliderArgs, let's create a temporary adapter or just manually construct if needed.
    // Better yet, let's extract the common args into a helper or just construct SliderArgs.

    // Note: We'll construct a SliderArgs to reuse the layout calculation.
    // This is a bit of a hack but avoids refactoring everything.
    let dummy_args = SliderArgs {
        value: 0.0,
        on_change: std::sync::Arc::new(|_| {}),
        width: args.width,
        active_track_color: args.active_track_color,
        inactive_track_color: args.inactive_track_color,
        thumb_diameter: args.thumb_diameter,
        thumb_color: args.thumb_color,
        state_layer_diameter: args.state_layer_diameter,
        state_layer_color: args.state_layer_color,
        disabled: args.disabled,
        accessibility_label: None,
        accessibility_description: None,
    };

    RangeSliderLayout {
        base: slider_layout(&dummy_args, component_width),
    }
}
