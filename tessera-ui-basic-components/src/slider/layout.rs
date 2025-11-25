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
