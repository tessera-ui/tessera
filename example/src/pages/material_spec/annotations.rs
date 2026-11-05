use std::collections::BTreeMap;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    image::image,
    image_vector::TryIntoImageVectorData as _,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    text::text,
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, PxRect, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember_with_key, tessera,
};

use crate::res::I18N;

use super::{INK, srgb, type_style};

pub(super) type Geometry = BTreeMap<&'static str, PxRect>;

pub(super) type PreciseGeometry = BTreeMap<&'static str, LayoutCoordinates>;

pub(super) type SpaceGeometry = BTreeMap<&'static str, tessera_ui::FloatRect>;

pub(super) fn watch_in_space(geometry: State<SpaceGeometry>, name: &'static str) -> Modifier {
    let modifier = Modifier::new();
    let modifier = if name == "figure" {
        modifier.layout_coordinate_space()
    } else {
        modifier
    };
    modifier.on_layout_rect_in_space(move |rect| {
        if geometry.is_alive()
            && geometry.with(|geometry| geometry.get(name).copied()) != Some(rect)
        {
            geometry.with_mut(|geometry| {
                geometry.insert(name, rect);
            });
        }
    })
}

pub(super) fn space_geometry(geometry: &SpaceGeometry) -> BTreeMap<&'static str, Rect> {
    geometry
        .iter()
        .map(|(&name, rect)| {
            (
                name,
                Rect {
                    x: Dp::from_pixels_f32(rect.x).0,
                    y: Dp::from_pixels_f32(rect.y).0,
                    w: Dp::from_pixels_f32(rect.width).0,
                    h: Dp::from_pixels_f32(rect.height).0,
                },
            )
        })
        .collect()
}

pub(super) fn watch_observed(geometry: State<SpaceGeometry>, name: &'static str) -> Modifier {
    watch_in_space(geometry, name).semantics(SemanticsArgs {
        test_tag: Some(format!("observed-{name}")),
        merge_descendants: true,
        ..Default::default()
    })
}

pub(super) fn watch_precise(geometry: State<PreciseGeometry>, name: &'static str) -> Modifier {
    Modifier::new().on_layout_coordinates(move |coordinates| {
        if geometry.with(|geometry| geometry.get(name).copied()) != Some(coordinates) {
            geometry.with_mut(|geometry| {
                geometry.insert(name, coordinates);
            });
        }
    })
}

pub(super) fn relative_precise_geometry(
    geometry: &PreciseGeometry,
) -> BTreeMap<&'static str, Rect> {
    geometry
        .keys()
        .filter_map(|name| relative_precise(geometry, name).map(|rect| (*name, rect)))
        .collect()
}

pub(super) fn relative_precise(geometry: &PreciseGeometry, name: &str) -> Option<Rect> {
    let root = geometry.get("figure")?;
    let coordinates = geometry.get(name)?;
    let origin = root
        .local_to_window
        .inverse_point(coordinates.local_to_window.translation());
    let scale = coordinates.local_to_window.scale() / root.local_to_window.scale();
    Some(Rect {
        x: Dp::from_pixels_f32(origin.x).0,
        y: Dp::from_pixels_f32(origin.y).0,
        w: Dp::from_pixels_f32(coordinates.local_size.width.to_f32() * scale).0,
        h: Dp::from_pixels_f32(coordinates.local_size.height.to_f32() * scale).0,
    })
}

pub(super) fn watch(geometry: State<Geometry>, name: &'static str) -> Modifier {
    Modifier::new().on_globally_positioned(move |bounds: PxRect| {
        if geometry.with(|geometry| geometry.get(name).copied()) != Some(bounds) {
            geometry.with_mut(|geometry| {
                geometry.insert(name, bounds);
            });
        }
    })
}

pub(super) fn relative_geometry(geometry: &Geometry) -> Geometry {
    let Some(origin) = geometry.get("figure") else {
        return Geometry::new();
    };
    geometry
        .iter()
        .map(|(name, rect)| {
            (
                *name,
                PxRect {
                    x: rect.x - origin.x,
                    y: rect.y - origin.y,
                    width: rect.width,
                    height: rect.height,
                },
            )
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub(super) fn relative(geometry: &Geometry, name: &str) -> Option<Rect> {
    let root = geometry.get("figure")?;
    let rect = geometry.get(name)?;
    Some(Rect {
        x: Dp::from_pixels_f32((rect.x.0 - root.x.0) as f32).0,
        y: Dp::from_pixels_f32((rect.y.0 - root.y.0) as f32).0,
        w: Dp::from_pixels_f32(rect.width.to_f32()).0,
        h: Dp::from_pixels_f32(rect.height.to_f32()).0,
    })
}

pub(super) fn dotted(svg: &mut String, x: f64, y: f64, length: f64, vertical: bool, color: &str) {
    for step in 0..(length.abs() / 2.0).ceil() as usize {
        let distance = step as f64 * 2.0 * length.signum();
        let (x, y) = if vertical {
            (x, y + distance)
        } else {
            (x + distance, y)
        };
        svg.push_str(&format!(
            "<rect x='{x}' y='{y}' width='0.7' height='0.7' fill='{color}'/>"
        ));
    }
}

pub(super) fn outline(svg: &mut String, rect: Rect) {
    dotted(svg, rect.x, rect.y, rect.w, false, "#49454f");
    dotted(svg, rect.x, rect.y + rect.h, rect.w, false, "#49454f");
    dotted(svg, rect.x, rect.y, rect.h, true, "#49454f");
    dotted(svg, rect.x + rect.w, rect.y, rect.h, true, "#49454f");
}

#[tessera]
pub(super) fn lines(width: Dp, height: Dp, svg: String) {
    if svg.trim().is_empty() {
        return;
    }
    let vector = remember_with_key((width.0.to_bits(), height.0.to_bits(), svg.clone()), || {
        format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}'>{svg}</svg>",
            width.0.max(1.0),
            height.0.max(1.0)
        )
        .into_bytes()
        .try_into_image_vector_data()
        .expect("annotation geometry")
    })
    .get();
    image()
        .painter(vector)
        .modifier(Modifier::new().size(width, height));
}

#[tessera]
pub(super) fn marker(number: usize, x: Dp, y: Dp) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(I18N.format("ui-anatomy-marker-value", [("v0", (number).into())])),
                    merge_descendants: true,
                    ..Default::default()
                })
                .background_with_shape(srgb(247, 242, 250), Shape::rounded_rectangle(Dp(16.0)))
                .border_with_shape(
                    Dp(1.0),
                    srgb(73, 69, 79),
                    Shape::rounded_rectangle(Dp(16.0)),
                ),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(14.0, 20.0))
                .color(INK);
        });
}

#[tessera]
pub(super) fn dimension(label: String, x: Dp, y: Dp) {
    boxed()
        .modifier(Modifier::new().offset(x, y).semantics(SemanticsArgs {
            role: Some(Role::Note),
            label: Some(I18N.format("ui-dimension-value", [("v0", I18N.text(&label).into())])),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            text(I18N.text(&(label)))
                .style(type_style(14.0, 20.0))
                .color(INK);
        });
}

#[cfg(test)]
mod tests {
    use tessera_ui::Px;

    use super::{Geometry, PxRect, relative_geometry};

    #[test]
    fn relative_figure_geometry_ignores_translation_but_preserves_local_changes() {
        let raw = Geometry::from([
            ("figure", PxRect::new(Px(137), Px(-293), Px(901), Px(599))),
            ("calendar", PxRect::new(Px(164), Px(-252), Px(361), Px(437))),
        ]);
        let expected = Geometry::from([
            ("figure", PxRect::new(Px(0), Px(0), Px(901), Px(599))),
            ("calendar", PxRect::new(Px(27), Px(41), Px(361), Px(437))),
        ]);
        assert_eq!(relative_geometry(&raw), expected);
        let shifted = raw
            .iter()
            .map(|(name, rect)| {
                (
                    *name,
                    PxRect {
                        x: rect.x + Px(-731),
                        y: rect.y + Px(1209),
                        ..*rect
                    },
                )
            })
            .collect::<Geometry>();
        assert_eq!(relative_geometry(&shifted), expected);

        let mut changed = shifted;
        changed.get_mut("calendar").unwrap().x += Px(1);
        assert_eq!(relative_geometry(&changed)["calendar"].x, Px(28));
        changed.get_mut("calendar").unwrap().height = Px(438);
        assert_eq!(relative_geometry(&changed)["calendar"].height, Px(438));
        assert!(relative_geometry(&Geometry::new()).is_empty());
    }
}
