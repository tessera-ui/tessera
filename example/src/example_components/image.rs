use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

const IMAGE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/assets/scarlet_ut.jpg",
);

pub struct ImageShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    image_data: Arc<ImageData>,
}

impl Default for ImageShowcaseState {
    fn default() -> Self {
        let image_data = Arc::new(
            load_image_from_source(&ImageSource::Path(IMAGE_PATH.to_string()))
                .expect("Failed to load image"),
        );

        Self {
            scrollable_state: Arc::new(ScrollableState::default()),
            image_data,
        }
    }
}

#[tessera]
#[shard]
pub fn image_showcase(#[state] state: ImageShowcaseState) {
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
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            test_content(state);
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content(state: Arc<ImageShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| text("Image Showcase"));

            scope.child(|| {
                spacer(Dp(10.0));
            });

            scope.child(move || {
                image(
                    ImageArgsBuilder::default()
                        .data(state.image_data.clone())
                        .build()
                        .unwrap(),
                )
            });
        },
    )
}
