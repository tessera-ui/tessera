use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    image_vector::{
        ImageVectorArgsBuilder, ImageVectorSource, image_vector, load_image_vector_from_source,
    },
    surface::{SurfaceArgsBuilder, surface},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let svg_path = format!("{}/../assets/emoji_u1f416.svg", env!("CARGO_MANIFEST_DIR"));
    let vector_data = load_image_vector_from_source(&ImageVectorSource::Path(svg_path))?;

    Renderer::run(
        move || {
            let vector_data = Arc::new(vector_data.clone());
            surface(
                SurfaceArgsBuilder::default()
                    .padding(Dp(24.0))
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .style(Color::WHITE.into())
                    .build()
                    .unwrap(),
                None,
                move || {
                    image_vector(
                        ImageVectorArgsBuilder::default()
                            .data(vector_data.clone())
                            .width(DimensionValue::Fixed(Dp(200.0).into()))
                            .height(DimensionValue::Fixed(Dp(200.0).into()))
                            .build()
                            .unwrap(),
                    );
                },
            );
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
