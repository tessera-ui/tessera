use std::sync::{Arc, Mutex};

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgsBuilder, column},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

const IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/assets/grid_background.png",
));

#[derive(Clone)]
struct GlassButtonShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    counter: Arc<Mutex<i32>>,
    ripple_state: Arc<RippleState>,
    image_data: Arc<ImageData>,
}

impl Default for GlassButtonShowcaseState {
    fn default() -> Self {
        let image_data = Arc::new(
            load_image_from_source(&ImageSource::Bytes(Arc::from(IMAGE_BYTES)))
                .expect("Failed to load image from embedded bytes"),
        );

        Self {
            scrollable_state: Default::default(),
            counter: Default::default(),
            ripple_state: Default::default(),
            image_data,
        }
    }
}

#[tessera]
#[shard]
pub fn glass_button_showcase(#[state] state: GlassButtonShowcaseState) {
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
                state.scrollable_state.clone(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .style(Color::WHITE.into())
                            .padding(Dp(16.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            test_content(state.clone());
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content(state: Arc<GlassButtonShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| text("Glass Button Showcase"));

            scope.child(|| {
                spacer(Dp(20.0));
            });

            // Glass Button on top
            let state_clone = state.clone();
            scope.child(move || {
                let image_data = state_clone.image_data.clone();
                boxed(
                    BoxedArgsBuilder::default()
                        .alignment(Alignment::Center)
                        .build()
                        .unwrap(),
                    |scope| {
                        scope.child(move || {
                            image(
                                ImageArgsBuilder::default()
                                    .width(DimensionValue::from(Dp(250.0)))
                                    .height(DimensionValue::from(Dp(250.0)))
                                    .data(image_data)
                                    .build()
                                    .unwrap(),
                            );
                        });

                        scope.child(move || {
                            let on_click = Arc::new({
                                let counter_clone = state_clone.counter.clone();
                                move || {
                                    let mut count = counter_clone.lock().unwrap();
                                    *count += 1;
                                }
                            });

                            glass_button(
                                GlassButtonArgsBuilder::default()
                                    .tint_color(Color::new(0.2, 0.5, 0.8, 0.1))
                                    .on_click(on_click)
                                    .shape(Shape::rounded_rectangle(Dp(25.0)))
                                    .build()
                                    .unwrap(),
                                state_clone.ripple_state.clone(),
                                || text("Click Me!"),
                            );
                        });
                    },
                );
            });

            scope.child(|| {
                spacer(Dp(20.0));
            });

            scope.child(move || {
                let count = state.counter.lock().unwrap();
                text(format!("Click count: {}", *count));
            });
        },
    )
}
