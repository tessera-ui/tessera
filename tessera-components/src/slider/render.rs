use tessera_ui::{
    AxisConstraint, Color, Constraint, Dp, DrawModifierContent, DrawModifierContext,
    DrawModifierNode, Modifier, Px, PxPosition, PxSize, State,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    tessera, use_context,
};

use crate::{
    modifier::ModifierExt,
    pipelines::shape::command::ShapeCommand,
    shape_def::{RoundedCorner, Shape},
    surface::surface,
    theme::MaterialTheme,
};

use super::{SliderColors, SliderLayout, SliderOrientation};

#[derive(Clone, PartialEq)]
struct SliderFocusRing {
    focused: bool,
    color: Color,
    orientation: SliderOrientation,
}

impl DrawModifierNode for SliderFocusRing {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        content.draw(ctx.render_input);
        if !self.focused {
            return;
        }
        let mut metadata = ctx.render_input.metadata_mut();
        let bounds = metadata.computed_data().expect("slider handle bounds");
        let radius = Dp(10.0).to_pixels_f32();
        let (size, position) = match self.orientation {
            SliderOrientation::Horizontal => (
                PxSize::new(Dp(20.0).to_px(), bounds.height + Dp(20.0).to_px()),
                PxPosition::from_f32_xy(bounds.width.to_f32() / 2.0 - radius, -radius),
            ),
            SliderOrientation::Vertical => (
                PxSize::new(bounds.width + Dp(20.0).to_px(), Dp(20.0).to_px()),
                PxPosition::from_f32_xy(-radius, bounds.height.to_f32() / 2.0 - radius),
            ),
        };
        let fragment = metadata.fragment_mut();
        let index = fragment.push_draw_command(ShapeCommand::OutlinedRect {
            color: self.color,
            corner_radii: [radius; 4],
            corner_g2: [2.0; 4],
            border_width: Dp(4.0).to_pixels_f32(),
        });
        let op = &mut fragment.ops_mut()[index as usize];
        op.position_override = Some(position);
        op.size_override = Some(size);
    }
}

#[tessera]
pub(super) fn active_segment(layout: Option<SliderLayout>, colors: Option<SliderColors>) {
    let layout = layout.expect("active_segment requires layout");
    let colors = colors.expect("active_segment requires colors");
    surface()
        .modifier(layout.axis.track_modifier(layout.track_height))
        .style(colors.active_track.into())
        .shape(layout.axis.shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
        }))
        .child(|| {});
}

#[tessera]
pub(super) fn inactive_segment(layout: Option<SliderLayout>, colors: Option<SliderColors>) {
    let layout = layout.expect("inactive_segment requires layout");
    let colors = colors.expect("inactive_segment requires colors");
    surface()
        .modifier(layout.axis.track_modifier(layout.track_height))
        .style(colors.inactive_track.into())
        .shape(layout.axis.shape(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
            top_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            bottom_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            bottom_left: RoundedCorner::manual(layout.inner_corner_radius, 3.0),
        }))
        .child(|| {});
}

#[tessera]
pub(super) fn slider_handle(
    layout: Option<SliderLayout>,
    width: Option<tessera_ui::Px>,
    colors: Option<SliderColors>,
    indicator_anchor: Option<State<tessera_ui::LayoutCoordinates>>,
    focused: Option<bool>,
) {
    let layout = layout.expect("slider_handle requires layout");
    let width = width.expect("slider_handle requires width");
    let colors = colors.expect("slider_handle requires colors");
    let constraint = layout.axis.constraint(Constraint::new(
        AxisConstraint::exact(width),
        AxisConstraint::exact(layout.handle_height),
    ));
    let modifier = Modifier::new().constrain(Some(constraint.width), Some(constraint.height));
    let modifier = if let Some(anchor) = indicator_anchor {
        modifier.on_layout_coordinates(move |rect| {
            if anchor.get() != rect {
                anchor.set(rect);
            }
        })
    } else {
        modifier
    };
    let scheme = use_context::<MaterialTheme>()
        .map(|theme| theme.get().color_scheme)
        .unwrap_or_else(|| MaterialTheme::default().color_scheme);
    let modifier = modifier.push_draw(SliderFocusRing {
        focused: focused.unwrap_or(false),
        color: scheme.secondary,
        orientation: layout.axis.orientation,
    });
    surface()
        .modifier(modifier)
        .style(colors.thumb.into())
        .shape(Shape::CAPSULE)
        .child(|| {});
}

#[tessera]
pub(super) fn stop_indicator(
    layout: Option<SliderLayout>,
    colors: Option<SliderColors>,
    count: Option<usize>,
) {
    let layout = layout.expect("stop_indicator requires layout");
    let colors = colors.expect("stop_indicator requires colors");
    for _ in 0..count.unwrap_or(1) {
        surface()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(layout.stop_indicator_diameter)),
                Some(AxisConstraint::exact(layout.stop_indicator_diameter)),
            ))
            .style(colors.inactive_tick.into())
            .shape(Shape::Ellipse)
            .child(|| {});
    }
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
pub(super) fn split_tracks(layout: Option<SliderLayout>, colors: Option<SliderColors>) {
    let layout = layout.expect("split_tracks requires layout");
    let colors = colors.expect("split_tracks requires colors");
    for (color, left, right) in [
        (
            colors.inactive_track,
            layout.track_corner_radius,
            layout.inner_corner_radius,
        ),
        (
            colors.active_track,
            layout.inner_corner_radius,
            layout.inner_corner_radius,
        ),
        (
            colors.inactive_track,
            layout.inner_corner_radius,
            layout.track_corner_radius,
        ),
    ] {
        surface()
            .modifier(layout.axis.track_modifier(layout.track_height))
            .style(color.into())
            .shape(layout.axis.shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(left, 3.0),
                bottom_left: RoundedCorner::manual(left, 3.0),
                top_right: RoundedCorner::manual(right, 3.0),
                bottom_right: RoundedCorner::manual(right, 3.0),
            }))
            .child(|| {});
    }
}
