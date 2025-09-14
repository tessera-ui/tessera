use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    image::{ImageArgsBuilder, ImageSource, image, load_image_from_source},
    surface::{SurfaceArgsBuilder, surface},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let image_path = format!(
        "{}/examples/assets/scarlet_ut.jpg",
        env!("CARGO_MANIFEST_DIR")
    );
    let image_data = load_image_from_source(&ImageSource::Path(image_path))?;

    Renderer::run(
        move || {
            let image_data = image_data.clone();
            surface(
                SurfaceArgsBuilder::default()
                    .padding(Dp(25.0))
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .style(Color::WHITE.into())
                    .build()
                    .unwrap(),
                None,
                move || {
                    image(
                        ImageArgsBuilder::default()
                            .data(image_data)
                            .build()
                            .unwrap(),
                    )
                },
            );
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
