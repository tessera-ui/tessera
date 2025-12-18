use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Modifier, State, tessera};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::{BoxedArgsBuilder, boxed},
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    modifier::ModifierExt,
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    spacer::spacer,
    text::{TextArgsBuilder, text},
};

use crate::{CalStyle, app::AppState};

#[tessera]
pub fn keyboard(app_state: State<AppState>, style: CalStyle) {
    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        |scope| {
            scope.child(spacer_v);
            scope.child_weighted(move || row_top(app_state, style), 1.0);
            scope.child(spacer_v);
            scope.child_weighted(move || row_1(app_state, style), 1.0);
            scope.child(spacer_v);
            scope.child_weighted(move || row_2(app_state, style), 1.0);
            scope.child(spacer_v);
            scope.child_weighted(move || row_3(app_state, style), 1.0);
            scope.child(spacer_v);
            scope.child_weighted(move || row_4(app_state, style), 1.0);
            scope.child(spacer_v);
        },
    )
}

fn row_top(state: State<AppState>, style: CalStyle) {
    row(
        RowArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("C", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("<-", state, style), 1.0);
            scope.child(spacer_h);
        },
    )
}

fn row_1(state: State<AppState>, style: CalStyle) {
    row(
        RowArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("1", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("2", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("3", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("*", state, style), 1.0);
            scope.child(spacer_h);
        },
    )
}

fn row_2(state: State<AppState>, style: CalStyle) {
    row(
        RowArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("4", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("5", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("6", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("-", state, style), 1.0);
            scope.child(spacer_h);
        },
    )
}

fn row_3(state: State<AppState>, style: CalStyle) {
    row(
        RowArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("7", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("8", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("9", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("+", state, style), 1.0);
            scope.child(spacer_h);
        },
    )
}

fn row_4(state: State<AppState>, style: CalStyle) {
    row(
        RowArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("0", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure(".", state, style), 1.0);
            scope.child(spacer_h);
            scope.child_weighted(make_num_closure("/", state, style), 1.0);
            scope.child(spacer_h);
        },
    )
}

fn make_on_click(key: &'static str, app_state: State<AppState>) -> Arc<dyn Fn() + Send + Sync> {
    // helper to produce the on_click handler; extracted to keep `num_key` concise
    let key_owned = key.to_string();
    Arc::new(move || match key_owned.as_str() {
        "C" => {
            app_state.with_mut(|s| s.expr.clear());
        }
        "<-" => {
            app_state.with_mut(|s| {
                s.expr.pop();
            });
        }
        _ => {
            app_state.with_mut(|s| {
                s.expr.push_str(key_owned.as_str());
            });
        }
    })
}

/// Returns a simple zero-argument closure that calls `num_key` with the
/// provided parameters. This reduces repetition in `keyboard` where many
/// identical closure wrappers were used.
fn make_num_closure(key: &'static str, state: State<AppState>, style: CalStyle) -> impl Fn() {
    move || num_key(key, state, style)
}

#[tessera]
fn num_key(key: &'static str, app_state: State<AppState>, style: CalStyle) {
    if key.is_empty() {
        return;
    }

    let on_click = make_on_click(key, app_state);

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
                    .on_click_shared(on_click)
                    .build()
                    .unwrap(),
                content_closure,
            );
        }
        CalStyle::Material => {
            button(
                ButtonArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_size())
                    .shape(Shape::Ellipse)
                    .color(Color::GRAY)
                    .on_click_shared(on_click)
                    .build()
                    .unwrap(),
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
    boxed(
        BoxedArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .alignment(Alignment::Center)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text(key.to_string())
                        .color(Color::WHITE.with_alpha(0.5))
                        .build()
                        .unwrap(),
                )
            });
        },
    )
}

#[tessera]
fn spacer_h() {
    spacer(Modifier::new().width(Dp(5.0)));
}

#[tessera]
fn spacer_v() {
    spacer(Modifier::new().height(Dp(5.0)));
}
