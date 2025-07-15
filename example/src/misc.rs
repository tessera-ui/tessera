use tessera_ui::{DimensionValue, Px};
use tessera_ui_basic_components::spacer::{SpacerArgsBuilder, spacer};

/// Creates a spacer with specified height
pub fn create_spacer(height: i32) -> impl FnOnce() {
    move || {
        spacer(
            SpacerArgsBuilder::default()
                .height(DimensionValue::Fixed(Px(height)))
                .width(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
                .build()
                .unwrap(),
        )
    }
}
