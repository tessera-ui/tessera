use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    icon::{IconArgsBuilder, icon},
    image_vector::{ImageVectorSource, load_image_vector_from_source},
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
                    icon(
                        IconArgsBuilder::default()
                            .content(vector_data.clone())
                            .size(Dp(200.0))
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
