use tessera_ui::{Color, DimensionValue, Px};
use tessera_ui_basic_components::{
    pipelines::ShadowProps,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_ui_macros::tessera;

use crate::material_colors::md_colors;

/// Outlined surface example with border and shadow
#[tessera]
pub fn outlined_surface_example() {
    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER_TRANSPARENT) // Material Design surface-container with opacity
            .border_width(5.0)
            .border_color(Some(md_colors::OUTLINE)) // Material Design outline color
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
            })
            .width(DimensionValue::Fixed(Px(200)))
            .height(DimensionValue::Fixed(Px(100)))
            .padding(10.0.into())
            .shadow(Some(ShadowProps {
                color: Color::new(0.0, 0.0, 0.0, 0.5),
                offset: [3.0, 3.0],
                smoothness: 5.0,
            }))
            .build()
            .unwrap(),
        None, // Non-interactive
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
            .color(md_colors::TERTIARY_TRANSPARENT) // Material Design tertiary color with transparency
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
            })
            .width(DimensionValue::Fixed(Px(150)))
            .height(DimensionValue::Fixed(Px(70)))
            .padding(5.0.into())
            .build()
            .unwrap(),
        None, // Non-interactive
        || {
            text("Transparent Fill");
        },
    )
}
