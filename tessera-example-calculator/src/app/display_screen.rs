use tessera_ui::{Color, Dp, Modifier, State, tessera};
use tessera_ui_basic_components::{
    alignment::MainAxisAlignment,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

use crate::{CalStyle, app::AppState, cal::evaluate};

#[tessera]
pub fn display_screen(app_state: State<AppState>, style: CalStyle) {
    // Outer transparent container with padding; delegate inner rendering to small
    // helpers
    surface(
        SurfaceArgsBuilder::default()
            .style(Color::TRANSPARENT.into())
            .modifier(Modifier::new().padding_all(Dp(5.0)))
            .build()
            .unwrap(),
        move || match style {
            CalStyle::Glass => render_glass_display(app_state),
            CalStyle::Material => render_material_display(app_state),
        },
    );
}

/// Render display when using glass style. Extracted to keep `display_screen`
/// short.
fn render_glass_display(app_state: State<AppState>) {
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .padding(Dp(10.0))
            .refraction_amount(0.0)
            .contrast(1.5)
            .build()
            .unwrap(),
        || {
            content(app_state);
        },
    );
}

/// Render display when using material style. Extracted to keep `display_screen`
/// short.
fn render_material_display(app_state: State<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().padding_all(Dp(10.0)))
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(25.0), 3.0),
                top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
            })
            .style(Color::GREY.into())
            .build()
            .unwrap(),
        move || {
            content(app_state);
        },
    );
}

#[tessera]
fn content(app_state: State<AppState>) {
    row(
        RowArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .main_axis_alignment(MainAxisAlignment::End)
            .build()
            .unwrap(),
        |scope| {
            scope.child(move || {
                let expr = app_state.with(|s| s.expr.clone());

                let content = if expr.is_empty() {
                    String::new()
                } else if let Ok(result) = evaluate(&expr, &mut rsc::Interpreter::new()) {
                    format!("{expr} = {:.2}", result)
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
            });
        },
    );
}
