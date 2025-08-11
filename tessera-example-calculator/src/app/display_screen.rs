use std::sync::Arc;

use tessera_ui::{Color, Dp, tessera};
use tessera_ui_basic_components::{
    alignment::MainAxisAlignment,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    row::RowArgsBuilder,
    row_ui,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

use crate::{app::AppState, cal::evaluate};

#[tessera]
pub fn display_screen(app_state: Arc<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .color(Color::TRANSPARENT)
            .padding(Dp(5.0))
            .build()
            .unwrap(),
        None,
        || {
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .padding(Dp(10.0))
                    .refraction_amount(0.0)
                    .contrast(1.5)
                    .build()
                    .unwrap(),
                None,
                || {
                    row_ui!(
                        RowArgsBuilder::default()
                            .width(tessera_ui::DimensionValue::FILLED)
                            .height(tessera_ui::DimensionValue::WRAP)
                            .main_axis_alignment(MainAxisAlignment::End)
                            .build()
                            .unwrap(),
                        move || {
                            let expr = app_state.expr.read();

                            let content = if expr.is_empty() {
                                String::new()
                            } else if let Ok(result) =
                                evaluate(&expr, &mut app_state.interpreter.write())
                            {
                                app_state.result.write().clone_from(&result);
                                format!("{expr} = {:.2}", app_state.result.read())
                            } else {
                                format!("{expr}")
                            };

                            let content = content.replace("/", "÷").replace("*", "×");

                            text(
                                TextArgsBuilder::default()
                                    .text(content)
                                    .color(Color::WHITE.with_alpha(0.5))
                                    .build()
                                    .unwrap(),
                            )
                        }
                    );
                },
            );
        },
    );
}
