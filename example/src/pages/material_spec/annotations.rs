//! Geometry annotations for live component figures.
//!
//! ## Usage
//!
//! Inspect anatomy and spacing around live components.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

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
    modifier::FocusModifierExt as _, remember, remember_with_key, tessera,
};

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

#[derive(Default)]
struct Observations {
    coordinates: PreciseGeometry,
    changed: Option<State<u64>>,
}

#[derive(Clone, Default)]
pub(super) struct ObservedGeometry {
    observations: Arc<Mutex<Observations>>,
    relative: BTreeMap<&'static str, Rect>,
    precision: f64,
}

impl ObservedGeometry {
    pub(super) fn new() -> Self {
        Self::default()
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    pub(super) fn coordinates(&self) -> PreciseGeometry {
        self.observations.lock().unwrap().coordinates.clone()
    }
}

pub(super) fn watch_observed(geometry: State<ObservedGeometry>, name: &'static str) -> Modifier {
    Modifier::new().on_layout_coordinates(move |coordinates| {
        if !geometry.is_alive() {
            return;
        }
        let changed = geometry.with(|geometry| {
            let mut observations = geometry.observations.lock().unwrap();
            if observations.coordinates.get(name) == Some(&coordinates) {
                return None;
            }
            observations.coordinates.insert(name, coordinates);
            observations.changed
        });
        if let Some(changed) = changed.filter(State::is_alive) {
            changed.with_mut(|revision| *revision = revision.wrapping_add(1));
        }
    })
}

pub(super) fn observed_relative_geometry(
    geometry: &ObservedGeometry,
) -> BTreeMap<&'static str, Rect> {
    geometry.relative.clone()
}

#[tessera]
pub(super) fn publish_observed_geometry(geometry: State<ObservedGeometry>) {
    let changed = remember(|| 0_u64);
    let _ = changed.get();
    let (relative, precision) = geometry.with(|geometry| {
        let mut observations = geometry.observations.lock().unwrap();
        observations.changed = Some(changed);
        let coordinates = &observations.coordinates;
        let precision = coordinates.get("figure").map_or(0.0, |root| {
            let ulp = coordinates
                .values()
                .flat_map(|coordinates| {
                    let translation = coordinates.local_to_window.translation();
                    [translation.x.abs(), translation.y.abs()]
                })
                .map(|value| f64::from(value.next_up() - value))
                .fold(0.0, f64::max);
            Dp::from_pixels_f32((ulp / f64::from(root.local_to_window.scale())) as f32).0
        });
        (relative_precise_geometry(coordinates), precision)
    });
    let unchanged = geometry.with(|geometry| {
        // Common window translations can introduce a few f32 projection ULPs.
        let tolerance = 2.0 * (geometry.precision + precision);
        geometry.relative.len() == relative.len()
            && relative.iter().all(|(name, rect)| {
                geometry.relative.get(name).is_some_and(|previous| {
                    previous.w == rect.w
                        && previous.h == rect.h
                        && (previous.x - rect.x).abs() <= tolerance
                        && (previous.y - rect.y).abs() <= tolerance
                })
            })
    });
    if !unchanged {
        geometry.with_mut(|geometry| {
            geometry.relative = relative;
            geometry.precision = precision;
        });
    }
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
        .vector(vector)
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
                    label: Some(format!("Anatomy marker {number}")),
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
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(INK);
        });
}

#[tessera]
pub(super) fn dimension(label: String, x: Dp, y: Dp) {
    boxed()
        .modifier(Modifier::new().offset(x, y).semantics(SemanticsArgs {
            role: Some(Role::Note),
            label: Some(format!("Dimension {label}")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            text()
                .content(label.clone())
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
        assert_eq!(raw["calendar"].y, Px(-252));

        let mut changed = shifted;
        changed.get_mut("calendar").unwrap().x += Px(1);
        assert_eq!(relative_geometry(&changed)["calendar"].x, Px(28));
        changed.get_mut("calendar").unwrap().height = Px(438);
        assert_eq!(relative_geometry(&changed)["calendar"].height, Px(438));
        assert!(relative_geometry(&Geometry::new()).is_empty());
    }
}
