use std::sync::{Arc, Mutex};

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgs, ColumnArgsBuilder, column},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    icon::{IconArgsBuilder, IconContent},
    icon_button::{GlassIconButtonArgsBuilder, glass_icon_button},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::Shape,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

const IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/assets/grid_background.png",
));
const ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

#[derive(Clone)]
struct GlassButtonShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    counter: Arc<Mutex<i32>>,
    ripple_state: Arc<RippleState>,
    icon_ripple_state: Arc<RippleState>,
    image_data: Arc<ImageData>,
    icon_data: Arc<ImageVectorData>,
}

impl Default for GlassButtonShowcaseState {
    fn default() -> Self {
        let image_data = Arc::new(
            load_image_from_source(&ImageSource::Bytes(Arc::from(IMAGE_BYTES)))
                .expect("Failed to load image from embedded bytes"),
        );
        let icon_data = Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
                .expect("Failed to load icon SVG"),
        );

        Self {
            scrollable_state: Default::default(),
            counter: Default::default(),
            ripple_state: Default::default(),
            icon_ripple_state: Default::default(),
            image_data,
            icon_data,
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

                        let glass_state = state_clone.clone();
                        scope.child(move || {
                            column(
                                ColumnArgs {
                                    cross_axis_alignment: CrossAxisAlignment::Center,
                                    ..Default::default()
                                },
                                move |scope| {
                                    let on_click = Arc::new({
                                        let counter_clone = glass_state.counter.clone();
                                        move || {
                                            let mut count = counter_clone.lock().unwrap();
                                            *count += 1;
                                        }
                                    });

                                    scope.child(move || {
                                        glass_button(
                                            GlassButtonArgsBuilder::default()
                                                .on_click(on_click)
                                                .shape(Shape::rounded_rectangle(Dp(25.0)))
                                                .build()
                                                .unwrap(),
                                            glass_state.ripple_state.clone(),
                                            || text("Click Me!"),
                                        );
                                    });

                                    scope.child(move || {
                                        spacer(SpacerArgs {
                                            height: Dp(20.0).into(),
                                            ..Default::default()
                                        });
                                    });

                                    let icon_state = state_clone.clone();
                                    scope.child(move || {
                                        let icon = IconArgsBuilder::default()
                                            .content(IconContent::from(
                                                icon_state.icon_data.clone(),
                                            ))
                                            .size(Dp(28.0))
                                            .build()
                                            .unwrap();

                                        let on_click = Arc::new({
                                            let counter_clone = icon_state.counter.clone();
                                            move || {
                                                let mut count = counter_clone.lock().unwrap();
                                                *count += 1;
                                                println!("Glass icon button clicked!");
                                            }
                                        });

                                        let args = GlassIconButtonArgsBuilder::default()
                                            .button(
                                                GlassButtonArgsBuilder::default()
                                                    .on_click(on_click)
                                                    .shape(Shape::Ellipse)
                                                    .build()
                                                    .unwrap(),
                                            )
                                            .icon(icon)
                                            .build()
                                            .unwrap();

                                        glass_icon_button(
                                            args,
                                            icon_state.icon_ripple_state.clone(),
                                        );
                                    });
                                },
                            );
                        });
                    },
                );
            });

            scope.child(|| {
                spacer(Dp(20.0));
            });

            let state_clone = state.clone();
            scope.child(move || {
                let count = state_clone.counter.lock().unwrap();
                text(format!("Click count: {}", *count));
            });
        },
    )
}
