use std::sync::{Arc, Mutex};

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::CrossAxisAlignment,
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, IconContent},
    icon_button::{IconButtonArgsBuilder, icon_button},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    md3_color::global_md3_scheme,
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

const ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

#[derive(Clone)]
struct ButtonShowcaseState {
    scrollable_state: ScrollableState,
    button1_ripple: RippleState,
    button2_ripple: RippleState,
    button3_ripple: RippleState,
    icon_button_ripple: RippleState,
    counter: Arc<Mutex<i32>>,
    icon_data: Arc<ImageVectorData>,
}

impl ButtonShowcaseState {
    fn new() -> Self {
        let icon_data = Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
                .expect("Failed to load icon SVG"),
        );

        Self {
            scrollable_state: Default::default(),
            button1_ripple: RippleState::new(),
            button2_ripple: RippleState::new(),
            button3_ripple: RippleState::new(),
            icon_button_ripple: RippleState::new(),
            counter: Default::default(),
            icon_data,
        }
    }
}

impl Default for ButtonShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

#[tessera]
#[shard]
pub fn button_showcase(
    #[state(default_with = "ButtonShowcaseState::new")] state: ButtonShowcaseState,
) {
    let scrollable_state = state.scrollable_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
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

                                    let state_clone = state.clone();
                                    scope.child(move || {
                                        let button_args = ButtonArgsBuilder::default()
                                            .shape(Shape::rounded_rectangle(Dp(12.0)))
                                            .color(global_md3_scheme().primary)
                                            .hover_color(Some(
                                                global_md3_scheme().primary_container,
                                            ))
                                            .on_click(Arc::new(|| {
                                                println!("Styled button clicked!");
                                            }))
                                            .padding(Dp(15.0))
                                            .build()
                                            .unwrap();

                                        button(
                                            button_args,
                                            state_clone.button3_ripple.clone(),
                                            || {
                                                text(
                                                    TextArgsBuilder::default()
                                                        .text("Styled")
                                                        .color(global_md3_scheme().on_primary)
                                                        .build()
                                                        .unwrap(),
                                                );
                                            },
                                        );
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Icon Button")
                                                .size(Dp(16.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    let state_clone = state.clone();
                                    scope.child(move || {
                                        let on_click_counter = state_clone.counter.clone();
                                        let icon = IconArgsBuilder::default()
                                            .content(IconContent::from(
                                                state_clone.icon_data.clone(),
                                            ))
                                            .size(Dp(24.0))
                                            .build()
                                            .unwrap();

                                        let button_args = IconButtonArgsBuilder::default()
                                            .button(
                                                ButtonArgsBuilder::default()
                                                    .shape(Shape::Ellipse)
                                                    .color(global_md3_scheme().surface_variant)
                                                    .hover_color(Some(global_md3_scheme().surface))
                                                    .on_click(Arc::new(move || {
                                                        let mut count =
                                                            on_click_counter.lock().unwrap();
                                                        *count += 1;
                                                        println!("Icon button clicked!");
                                                    }))
                                                    .padding(Dp(12.0))
                                                    .build()
                                                    .unwrap(),
                                            )
                                            .icon(icon)
                                            .build()
                                            .unwrap();

                                        icon_button(
                                            button_args,
                                            state_clone.icon_button_ripple.clone(),
                                        );
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
