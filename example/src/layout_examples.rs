use tessera::{DimensionValue, Px, ShadowProps};
use tessera_basic_components::{
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

/// Outlined surface example with border and shadow
#[tessera]
pub fn outlined_surface_example() {
    surface(
        SurfaceArgsBuilder::default()
            .color([0.3, 0.3, 0.3, 0.5]) // Semi-transparent fill color
            .border_width(5.0)
            .border_color(Some([1.0, 0.0, 0.0, 1.0])) // Red border, RGBA
            .corner_radius(15.0)
            .width(DimensionValue::Fixed(Px(200)))
            .height(DimensionValue::Fixed(Px(100)))
            .padding(10.0.into())
            .shadow(Some(ShadowProps {
                color: [0.0, 0.0, 0.0, 0.5],
                offset: [3.0, 3.0],
                smoothness: 5.0,
            }))
            .build()
            .unwrap(),
        || {
            text("Outlined Box");
        },
    )
}

/// Transparent surface example
#[tessera]
pub fn transparent_surface_example() {
    surface(
        SurfaceArgsBuilder::default()
            .color([0.0, 0.0, 1.0, 0.3]) // Transparent blue fill
            .corner_radius(10.0)
            .width(DimensionValue::Fixed(Px(150)))
            .height(DimensionValue::Fixed(Px(70)))
            .padding(5.0.into())
            .build()
            .unwrap(),
        || {
            text("Transparent Fill");
        },
    )
}
