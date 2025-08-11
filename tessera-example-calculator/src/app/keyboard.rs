use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, tessera};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::BoxedArgsBuilder,
    boxed_ui,
    column::ColumnArgsBuilder,
    column_ui,
    glass_button::{GlassButtonArgsBuilder, glass_button},
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    text::{TextArgsBuilder, text},
};

use crate::app::AppState;

#[tessera]
pub fn keyboard(app_state: Arc<AppState>) {
    column_ui!(
        ColumnArgsBuilder::default()
            .width(tessera_ui::DimensionValue::FILLED)
            .height(tessera_ui::DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (
                move || {
                    row_ui!(
                        RowArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("C", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("<-", state), 1.0)
                        },
                        || spacer_h(),
                    )
                },
                1.0,
            )
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (
                move || {
                    row_ui!(
                        RowArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("1", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("2", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("3", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("*", state), 1.0)
                        },
                        || spacer_h()
                    )
                },
                1.0,
            )
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (
                move || {
                    row_ui!(
                        RowArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("4", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("5", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("6", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("-", state), 1.0)
                        },
                        || spacer_h()
                    )
                },
                1.0,
            )
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (
                move || {
                    row_ui!(
                        RowArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("7", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("8", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("9", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("+", state), 1.0)
                        },
                        || spacer_h()
                    )
                },
                1.0,
            )
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (
                move || {
                    row_ui!(
                        RowArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("0", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key(".", state), 1.0)
                        },
                        || spacer_h(),
                        {
                            let state = app_state.clone();
                            (move || num_key("/", state), 1.0)
                        },
                        || spacer_h()
                    )
                },
                1.0,
            )
        },
        || spacer_v()
    )
}

#[tessera]
fn num_key(key: &'static str, app_state: Arc<AppState>) {
    if key.is_empty() {
        return;
    }

    let ripple_state = app_state
        .ripple_states
        .entry(key.to_string())
        .or_default()
        .clone();

    glass_button(
        GlassButtonArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .shape(Shape::Ellipse)
            .blur_radius(0.0)
            .noise_amount(0.0)
            .on_click(Arc::new(move || match key {
                "C" => {
                    app_state.expr.write().clear();
                    app_state.result.write().clone_from(&0.0);
                }
                "<-" => {
                    let mut expr = app_state.expr.write();
                    expr.pop();
                }
                _ => {
                    let mut expr = app_state.expr.write();
                    expr.push_str(key);
                }
            }))
            .build()
            .unwrap(),
        ripple_state.clone(),
        move || {
            let key = match key {
                "<-" => "←",
                "*" => "x",
                "/" => "÷",
                _ => key,
            };
            boxed_ui!(
                BoxedArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .alignment(Alignment::Center)
                    .build()
                    .unwrap(),
                || text(
                    TextArgsBuilder::default()
                        .text(key.to_string())
                        .color(Color::WHITE.with_alpha(0.5))
                        .build()
                        .unwrap(),
                )
            )
        },
    );
}

#[tessera]
fn spacer_h() {
    spacer(
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fixed(Dp(5.0).to_px()))
            .build()
            .unwrap(),
    );
}

#[tessera]
fn spacer_v() {
    spacer(
        SpacerArgsBuilder::default()
            .height(DimensionValue::Fixed(Dp(5.0).to_px()))
            .build()
            .unwrap(),
    );
}
