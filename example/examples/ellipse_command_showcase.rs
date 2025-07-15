use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{AsColumnItem, ColumnArgsBuilder, column},
    pipelines::shape::ShadowProps,
    shape_def::Shape,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgsBuilder, surface},
};
use tessera_ui_macros::tessera;

#[tessera]
fn app() {
    let red = Color::new(0.8, 0.2, 0.2, 1.0);
    let green = Color::new(0.2, 0.8, 0.2, 1.0);
    let blue = Color::new(0.2, 0.2, 0.8, 1.0);

    surface(
        SurfaceArgsBuilder::default()
            .color(Color::new(0.1, 0.1, 0.1, 1.0))
            .build()
            .unwrap(),
        None,
        move || {
            column(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                [
                    (move || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .shape(Shape::Ellipse)
                                .color(red)
                                .shadow(Some(ShadowProps {
                                    color: Color::BLACK.with_alpha(0.5),
                                    offset: [2.0, 2.0],
                                    smoothness: 4.0,
                                }))
                                .width(DimensionValue::Fixed(Dp(100.0).into()))
                                .height(DimensionValue::Fixed(Dp(100.0).into()))
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        )
                    })
                    .into_column_item(),
                    (|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Dp(0.0).into()),
                            height: DimensionValue::Fixed(Dp(20.0).into()),
                        })
                    })
                    .into_column_item(),
                    (move || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .shape(Shape::Ellipse)
                                .color(green)
                                .border_width(5.0)
                                .width(DimensionValue::Fixed(Dp(150.0).into()))
                                .height(DimensionValue::Fixed(Dp(80.0).into()))
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        )
                    })
                    .into_column_item(),
                    (|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Dp(0.0).into()),
                            height: DimensionValue::Fixed(Dp(20.0).into()),
                        })
                    })
                    .into_column_item(),
                    (move || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .shape(Shape::RoundedRectangle {
                                    corner_radius: 20.0,
                                })
                                .color(blue)
                                .width(DimensionValue::Fixed(Dp(80.0).into()))
                                .height(DimensionValue::Fixed(Dp(150.0).into()))
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        )
                    })
                    .into_column_item(),
                ],
            )
        },
    )
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::run(app, |app| {
        tessera_ui_basic_components::pipelines::register_pipelines(app);
    })?;
    Ok(())
}
