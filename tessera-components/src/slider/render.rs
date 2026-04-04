use tessera_ui::{AxisConstraint, Color, Modifier, Px};

use crate::{
    modifier::ModifierExt,
    shape_def::{RoundedCorner, Shape},
    surface::surface,
};

use super::{SliderColors, SliderLayout};

fn render_surface(
    modifier: Modifier,
    style: impl Into<crate::surface::SurfaceStyle>,
    shape: Shape,
) {
    surface()
        .modifier(modifier)
        .style(style.into())
        .shape(shape)
        .with_child(|| {});
}

pub(super) fn render_active_segment(layout: SliderLayout, colors: &SliderColors) {
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.track_height))),
        colors.active_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
        },
    );
}

pub(super) fn render_inactive_segment(layout: SliderLayout, colors: &SliderColors) {
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.track_height))),
        colors.inactive_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
        },
    );
}

pub(super) fn render_handle(layout: SliderLayout, width: tessera_ui::Px, colors: &SliderColors) {
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(width)),
            Some(AxisConstraint::exact(layout.handle_height)),
        ),
        colors.thumb,
        Shape::capsule(),
    );
}

pub(super) fn render_stop_indicator(layout: SliderLayout, colors: &SliderColors) {
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.stop_indicator_diameter)),
        ),
        colors.active_track,
        Shape::Ellipse,
    );
}

pub(super) fn render_tick(diameter: Px, color: Color) {
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(diameter)),
            Some(AxisConstraint::exact(diameter)),
        ),
        color,
        Shape::Ellipse,
    );
}

pub(super) fn render_centered_tracks(
    layout: crate::slider::layout::CenteredSliderLayout,
    colors: &SliderColors,
) {
    // Left Inactive
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        colors.inactive_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        },
    );

    // Active (Middle)
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        colors.active_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        },
    );

    // Right Inactive
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        colors.inactive_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
        },
    );
}

pub(super) fn render_centered_stops(
    layout: crate::slider::layout::CenteredSliderLayout,
    colors: &SliderColors,
) {
    // Left Stop
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ),
        colors.active_track,
        Shape::Ellipse,
    );

    // Right Stop
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ),
        colors.active_track,
        Shape::Ellipse,
    );
}

pub(super) fn render_range_stops(
    layout: crate::slider::layout::RangeSliderLayout,
    colors: &SliderColors,
) {
    // Left Stop
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ),
        colors.active_track,
        Shape::Ellipse,
    );

    // Right Stop
    render_surface(
        Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ),
        colors.active_track,
        Shape::Ellipse,
    );
}

pub(super) fn render_range_tracks(
    layout: crate::slider::layout::RangeSliderLayout,
    colors: &SliderColors,
) {
    // Left Inactive
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        colors.inactive_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        },
    );

    // Active (Middle)
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        colors.active_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        },
    );

    // Right Inactive
    render_surface(
        Modifier::new()
            .fill_max_width()
            .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        colors.inactive_track,
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
        },
    );
}
