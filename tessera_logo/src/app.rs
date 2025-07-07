use crate::{
    background::{background, BackgroundArgsBuilder},
    logo::{crystal_shard, CrystalShardArgsBuilder},
};
use std::sync::Arc;
use std::time::Instant;
use tessera::Px;
use tessera_basic_components::{
    alignment::Alignment,
    boxed::{boxed, AsBoxedItem, BoxedArgsBuilder},
    spacer::{spacer, SpacerArgsBuilder},
};
use tessera_macros::tessera;

#[derive(Debug)]
pub struct AppState {
    pub start_time: Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

#[tessera]
pub fn app(state: Arc<AppState>) {
    let time = state.start_time.elapsed().as_secs_f32();

    let shard = (move || crystal_shard(CrystalShardArgsBuilder::default().build().unwrap()))
        .into_boxed_item();

    // The existing logo component, which centers the shard
    let inner_logo = (move || {
        boxed(
            BoxedArgsBuilder::default()
                .alignment(Alignment::Center)
                .build()
                .unwrap(),
            [shard],
        );
    })
    .into_boxed_item();

    // A spacer that is larger than the logo, creating a padding effect.
    // The logo is ~400px, so we add 100px padding on all sides.
    let padding_spacer = (move || {
        spacer(
            SpacerArgsBuilder::default()
                .width(tessera::DimensionValue::Fixed(Px(500)))
                .height(tessera::DimensionValue::Fixed(Px(500)))
                .build()
                .unwrap(),
        )
    })
    .into_boxed_item();

    // The outer container that holds both the logo and the spacer.
    // The spacer forces the size, and the logo is centered within it.
    let padded_logo = (move || {
        boxed(
            BoxedArgsBuilder::default()
                .alignment(Alignment::Center)
                .build()
                .unwrap(),
            [padding_spacer, inner_logo],
        );
    })
    .into_boxed_item();

    let background_args = BackgroundArgsBuilder::default()
        .time(time)
        .alignment(Alignment::Center)
        .build()
        .unwrap();

    background(background_args, [padded_logo]);
}
