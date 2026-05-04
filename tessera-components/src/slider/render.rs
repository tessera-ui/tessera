use tessera_ui::{AxisConstraint, Color, Modifier, Px, tessera};

use crate::{
    modifier::ModifierExt,
    shape_def::{RoundedCorner, Shape},
    surface::surface,
};

use super::{SliderColors, SliderLayout};

#[tessera]
pub(super) fn active_segment(layout: Option<SliderLayout>, colors: Option<SliderColors>) {
    let layout = layout.expect("active_segment requires layout");
    let colors = colors.expect("active_segment requires colors");
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.track_height))),
        )
        .style(colors.active_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
        })
        .child(|| {});
}

#[tessera]
pub(super) fn inactive_segment(layout: Option<SliderLayout>, colors: Option<SliderColors>) {
    let layout = layout.expect("inactive_segment requires layout");
    let colors = colors.expect("inactive_segment requires colors");
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.track_height))),
        )
        .style(colors.inactive_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
        })
        .child(|| {});
}

#[tessera]
pub(super) fn slider_handle(
    layout: Option<SliderLayout>,
    width: Option<tessera_ui::Px>,
    colors: Option<SliderColors>,
) {
    let layout = layout.expect("slider_handle requires layout");
    let width = width.expect("slider_handle requires width");
    let colors = colors.expect("slider_handle requires colors");
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(width)),
            Some(AxisConstraint::exact(layout.handle_height)),
        ))
        .style(colors.thumb.into())
        .shape(Shape::CAPSULE)
        .child(|| {});
}

#[tessera]
pub(super) fn stop_indicator(layout: Option<SliderLayout>, colors: Option<SliderColors>) {
    let layout = layout.expect("stop_indicator requires layout");
    let colors = colors.expect("stop_indicator requires colors");
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.stop_indicator_diameter)),
        ))
        .style(colors.active_track.into())
        .shape(Shape::Ellipse)
        .child(|| {});
}

#[tessera]
pub(super) fn slider_tick(diameter: Option<Px>, color: Option<Color>) {
    let diameter = diameter.expect("slider_tick requires diameter");
    let color = color.expect("slider_tick requires color");
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(diameter)),
            Some(AxisConstraint::exact(diameter)),
        ))
        .style(color.into())
        .shape(Shape::Ellipse)
        .child(|| {});
}

#[tessera]
pub(super) fn centered_tracks(
    layout: Option<crate::slider::layout::CenteredSliderLayout>,
    colors: Option<SliderColors>,
) {
    let layout = layout.expect("centered_tracks requires layout");
    let colors = colors.expect("centered_tracks requires colors");
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        )
        .style(colors.inactive_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        })
        .child(|| {});
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        )
        .style(colors.active_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        })
        .child(|| {});
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        )
        .style(colors.inactive_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
        })
        .child(|| {});
}

#[tessera]
pub(super) fn centered_stops(
    layout: Option<crate::slider::layout::CenteredSliderLayout>,
    colors: Option<SliderColors>,
) {
    let layout = layout.expect("centered_stops requires layout");
    let colors = colors.expect("centered_stops requires colors");
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ))
        .style(colors.active_track.into())
        .shape(Shape::Ellipse)
        .child(|| {});
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ))
        .style(colors.active_track.into())
        .shape(Shape::Ellipse)
        .child(|| {});
}

#[tessera]
pub(super) fn range_stops(
    layout: Option<crate::slider::layout::RangeSliderLayout>,
    colors: Option<SliderColors>,
) {
    let layout = layout.expect("range_stops requires layout");
    let colors = colors.expect("range_stops requires colors");
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ))
        .style(colors.active_track.into())
        .shape(Shape::Ellipse)
        .child(|| {});
    surface()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
            Some(AxisConstraint::exact(layout.base.stop_indicator_diameter)),
        ))
        .style(colors.active_track.into())
        .shape(Shape::Ellipse)
        .child(|| {});
}

#[tessera]
pub(super) fn range_tracks(
    layout: Option<crate::slider::layout::RangeSliderLayout>,
    colors: Option<SliderColors>,
) {
    let layout = layout.expect("range_tracks requires layout");
    let colors = colors.expect("range_tracks requires colors");
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        )
        .style(colors.inactive_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        })
        .child(|| {});
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        )
        .style(colors.active_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
        })
        .child(|| {});
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .constrain(None, Some(AxisConstraint::exact(layout.base.track_height))),
        )
        .style(colors.inactive_track.into())
        .shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.base.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.base.track_corner_radius, 3.0),
        })
        .child(|| {});
}
