use tessera_ui::{Color, DimensionValue, Modifier, Px};

use crate::{
    modifier::ModifierExt,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, surface},
};

use super::{SliderColors, SliderLayout};

pub(super) fn render_active_segment(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.track_height)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            }),
        || {},
    );
}

pub(super) fn render_inactive_segment(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.track_height)),
            ))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            }),
        || {},
    );
}

pub(super) fn render_handle(layout: SliderLayout, width: tessera_ui::Px, colors: &SliderColors) {
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(width)),
                Some(DimensionValue::Fixed(layout.handle_height)),
            ))
            .style(colors.thumb.into())
            .shape(Shape::capsule()),
        || {},
    );
}

pub(super) fn render_stop_indicator(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(layout.stop_indicator_diameter)),
                Some(DimensionValue::Fixed(layout.stop_indicator_diameter)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::Ellipse),
        || {},
    );
}

pub(super) fn render_tick(diameter: Px, color: Color) {
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(diameter)),
                Some(DimensionValue::Fixed(diameter)),
            ))
            .style(color.into())
            .shape(Shape::Ellipse),
        || {},
    );
}

pub(super) fn render_centered_tracks(
    layout: crate::slider::layout::CenteredSliderLayout,
    colors: &SliderColors,
) {
    // Left Inactive
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.base.track_height)),
            ))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            }),
        || {},
    );

    // Active (Middle)
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.base.track_height)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            }),
        || {},
    );

    // Right Inactive
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.base.track_height)),
            ))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            }),
        || {},
    );
}

pub(super) fn render_centered_stops(
    layout: crate::slider::layout::CenteredSliderLayout,
    colors: &SliderColors,
) {
    // Left Stop
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::Ellipse),
        || {},
    );

    // Right Stop
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::Ellipse),
        || {},
    );
}

pub(super) fn render_range_stops(
    layout: crate::slider::layout::RangeSliderLayout,
    colors: &SliderColors,
) {
    // Left Stop
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::Ellipse),
        || {},
    );

    // Right Stop
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
                Some(DimensionValue::Fixed(layout.base.stop_indicator_diameter)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::Ellipse),
        || {},
    );
}

pub(super) fn render_range_tracks(
    layout: crate::slider::layout::RangeSliderLayout,
    colors: &SliderColors,
) {
    // Left Inactive
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.base.track_height)),
            ))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            }),
        || {},
    );

    // Active (Middle)
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.base.track_height)),
            ))
            .style(colors.active_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            }),
        || {},
    );

    // Right Inactive
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::FILLED),
                Some(DimensionValue::Fixed(layout.base.track_height)),
            ))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
                top_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            }),
        || {},
    );
}
