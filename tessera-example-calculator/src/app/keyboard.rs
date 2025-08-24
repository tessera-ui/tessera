use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, tessera};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::BoxedArgsBuilder,
    boxed_ui,
    button::{ButtonArgsBuilder, button},
    column::ColumnArgsBuilder,
    column_ui,
    glass_button::{GlassButtonArgsBuilder, glass_button},
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    text::{TextArgsBuilder, text},
};

use crate::{CalStyle, app::AppState};

#[tessera]
pub fn keyboard(app_state: Arc<AppState>, style: CalStyle) {
    column_ui!(
        ColumnArgsBuilder::default()
            .width(tessera_ui::DimensionValue::FILLED)
            .height(tessera_ui::DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (move || row_top(app_state.clone(), style), 1.0)
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (move || row_1(app_state.clone(), style), 1.0)
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (move || row_2(app_state.clone(), style), 1.0)
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (move || row_3(app_state.clone(), style), 1.0)
        },
        || spacer_v(),
        {
            let app_state = app_state.clone();
            (move || row_4(app_state.clone(), style), 1.0)
        },
        || spacer_v()
    )
}

fn row_top(app_state: Arc<AppState>, style: CalStyle) {
    row_ui!(
        RowArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("C", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("<-", state, style), 1.0)
        },
        || spacer_h(),
    )
}

fn row_1(app_state: Arc<AppState>, style: CalStyle) {
    row_ui!(
        RowArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("1", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("2", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("3", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("*", state, style), 1.0)
        },
        || spacer_h()
    )
}

fn row_2(app_state: Arc<AppState>, style: CalStyle) {
    row_ui!(
        RowArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("4", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("5", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("6", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("-", state, style), 1.0)
        },
        || spacer_h()
    )
}

fn row_3(app_state: Arc<AppState>, style: CalStyle) {
    row_ui!(
        RowArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("7", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("8", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("9", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("+", state, style), 1.0)
        },
        || spacer_h()
    )
}

fn row_4(app_state: Arc<AppState>, style: CalStyle) {
    row_ui!(
        RowArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("0", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure(".", state, style), 1.0)
        },
        || spacer_h(),
        {
            let state = app_state.clone();
            (make_num_closure("/", state, style), 1.0)
        },
        || spacer_h()
    )
}

fn make_on_click(key: &'static str, app_state: Arc<AppState>) -> Arc<dyn Fn() + Send + Sync> {
    // helper to produce the on_click handler; extracted to keep `num_key` concise
    let key_owned = key.to_string();
    let app_state = app_state.clone();
    Arc::new(move || match key_owned.as_str() {
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
            expr.push_str(key_owned.as_str());
        }
    })
}

/// Returns a simple zero-argument closure that calls `num_key` with the provided parameters.
/// This reduces repetition in `keyboard` where many identical closure wrappers were used.
fn make_num_closure(key: &'static str, state: Arc<AppState>, style: CalStyle) -> impl Fn() {
    // Clone the Arc inside the closure so the closure does not move out of the captured variable.
    move || num_key(key, state.clone(), style)
}

#[tessera]
fn num_key(key: &'static str, app_state: Arc<AppState>, style: CalStyle) {
    if key.is_empty() {
        return;
    }

    let key_str = key.to_string();
    let ripple_state = app_state
        .ripple_states
        .entry(key_str.clone())
        .or_default()
        .clone();

    let on_click = make_on_click(key, app_state.clone());

    let content_closure = move || {
        key_content(key);
    };

    match style {
        CalStyle::Glass => {
            glass_button(
                GlassButtonArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .shape(Shape::Ellipse)
                    .blur_radius(0.0)
                    .noise_amount(0.0)
                    .on_click(on_click)
                    .build()
                    .unwrap(),
                ripple_state,
                content_closure,
            );
        }
        CalStyle::Material => {
            button(
                ButtonArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .shape(Shape::Ellipse)
                    .color(Color::GRAY)
                    .on_click(on_click)
                    .build()
                    .unwrap(),
                ripple_state,
                content_closure,
            );
        }
    }
}

#[tessera]
fn key_content(key: &'static str) {
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
