use std::time::Instant;

use tessera_ui::{shard, tessera};
use tessera_ui_basic_components::{alignment::Alignment, boxed::AsBoxedItem};

use crate::{
    background::{BackgroundArgsBuilder, background},
    logo::{CrystalShardArgsBuilder, crystal_shard},
};

#[derive(Debug)]
pub struct AppState {
    pub start_time: Instant,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

#[tessera]
#[shard]
pub fn app(#[state] state: AppState) {
    let time = state.start_time.elapsed().as_secs_f32();

    let logo = (move || crystal_shard(CrystalShardArgsBuilder::default().build().unwrap()))
        .into_boxed_item();

    let background_args = BackgroundArgsBuilder::default()
        .time(time)
        .alignment(Alignment::Center)
        .build()
        .unwrap();

    background(background_args, [logo]);
}
