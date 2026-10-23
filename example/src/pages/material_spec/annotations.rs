//! Geometry annotations for live component figures.
//!
//! ## Usage
//!
//! Inspect anatomy and spacing around live components.

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
    Dp, Modifier, PxRect, State, accesskit::Role, modifier::FocusModifierExt as _,
    remember_with_key, tessera,
};

use super::{INK, srgb, type_style};

pub(super) type Geometry = BTreeMap<&'static str, PxRect>;

pub(super) fn watch(geometry: State<Geometry>, name: &'static str) -> Modifier {
    Modifier::new().on_globally_positioned(move |bounds: PxRect| {
        if geometry.with(|geometry| geometry.get(name).copied()) != Some(bounds) {
            geometry.with_mut(|geometry| {
                geometry.insert(name, bounds);
            });
        }
    })
}

#[derive(Clone, Copy)]
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
