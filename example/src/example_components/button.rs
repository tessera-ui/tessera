use std::sync::{Arc, Mutex};

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::CrossAxisAlignment,
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Default, Clone)]
struct ButtonShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    button1_ripple: Arc<RippleState>,
    button2_ripple: Arc<RippleState>,
    button3_ripple: Arc<RippleState>,
    counter: Arc<Mutex<i32>>,
}

#[tessera]
#[shard]
pub fn button_showcase(#[state] state: ButtonShowcaseState) {
    let scrollable_state = state.scrollable_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                scrollable_state,
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .style(Color::WHITE.into())
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start)
                                    .build()
                                    .unwrap(),
                                move |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Button Showcase")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    let state_clone = state.clone();
                                    let counter_clone = state.counter.clone();
                                    scope.child(move || {
                                        let button_args = ButtonArgsBuilder::default()
                                            .on_click(Arc::new(move || {
                                                let mut count = counter_clone.lock().unwrap();
                                                *count += 1;
                                                println!("Button clicked! Count: {}", *count);
                                            }))
                                            .build()
                                            .unwrap();

                                        button(
                                            button_args,
                                            state_clone.button1_ripple.clone(),
                                            || {
                                                text("Click Me!");
                                            },
                                        );
                                    });

                                    let state_clone = state.clone();
                                    scope.child(move || {
                                        let count = state_clone.counter.lock().unwrap();
                                        text(format!("Click count: {}", *count));
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Disabled Button")
                                                .size(Dp(16.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    let state_clone = state.clone();
                                    scope.child(move || {
                                        // A button without an on_click handler is disabled by default.
                                        let button_args =
                                            ButtonArgsBuilder::default().build().unwrap();

                                        button(
                                            button_args,
                                            state_clone.button2_ripple.clone(),
                                            || {
                                                text("You can't click me");
                                            },
                                        );
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Styled Button")
                                                .size(Dp(16.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(move || {
                                        let button_args = ButtonArgsBuilder::default()
                                            .shape(Shape::rounded_rectangle(Dp(12.0)))
                                            .color(Color::new(0.2, 0.8, 0.2, 1.0))
                                            .hover_color(Some(Color::new(0.3, 0.9, 0.3, 1.0)))
                                            .on_click(Arc::new(|| {
                                                println!("Styled button clicked!");
                                            }))
                                            .padding(Dp(15.0))
                                            .build()
                                            .unwrap();

                                        button(button_args, state.button3_ripple.clone(), || {
                                            text(
                                                TextArgsBuilder::default()
                                                    .text("Styled")
                                                    .color(Color::WHITE)
                                                    .build()
                                                    .unwrap(),
                                            );
                                        });
                                    });
                                },
                            )
                        },
                    );
                },
            )
        },
    );
}
