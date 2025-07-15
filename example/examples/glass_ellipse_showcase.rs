use std::sync::Arc;

use tessera::{Color, DimensionValue, Dp, Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{AsColumnItem, ColumnArgsBuilder, column},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    ripple_state::RippleState,
    surface::{SurfaceArgs, surface},
};
use tessera_macros::tessera;

#[tessera]
fn app() {
    let ripple_state = Arc::new(RippleState::new());
    let glass_args = FluidGlassArgsBuilder::default()
        .width(DimensionValue::Fixed(Dp(100.0).into()))
        .height(DimensionValue::Fixed(Dp(100.0).into()))
        .shape(tessera_basic_components::shape_def::Shape::Ellipse)
        .blur_radius(5.0)
        .tint_color(Color::new(0.2, 0.3, 0.8, 0.1))
        .build()
        .unwrap();

    surface(
        SurfaceArgs {
            color: Color::new(0.1, 0.1, 0.1, 1.0),
            ..Default::default()
        },
        None,
        move || {
            column(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                [(move || fluid_glass(glass_args, Some(ripple_state), || {})).into_column_item()],
            )
        },
    )
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::run(app, |app| {
        tessera_basic_components::pipelines::register_pipelines(app);
    })?;
    Ok(())
}
