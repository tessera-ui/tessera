use tessera_ui::DimensionValue;

use crate::{
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
};

use super::{SliderColors, SliderLayout};

pub(super) fn render_active_segment(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fixed(layout.track_height))
            .style(colors.active_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                bottom_left: RoundedCorner::Capsule,
            })
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

pub(super) fn render_inactive_segment(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fixed(layout.track_height))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                top_right: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::Capsule,
                bottom_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            })
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

pub(super) fn render_focus(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(layout.focus_width))
            .height(DimensionValue::Fixed(layout.focus_height))
            .style(colors.handle_focus.into())
            .shape(Shape::capsule())
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

pub(super) fn render_handle(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(layout.handle_width))
            .height(DimensionValue::Fixed(layout.handle_height))
            .style(colors.handle.into())
            .shape(Shape::capsule())
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

pub(super) fn render_decoration_dot(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(layout.decoration_diameter))
            .height(DimensionValue::Fixed(layout.decoration_diameter))
            .style(colors.handle.into())
            .shape(Shape::Ellipse)
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}
