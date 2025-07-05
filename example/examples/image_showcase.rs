use tessera::{DimensionValue, Dp, Renderer};
use tessera_basic_components::{
    image::{ImageArgsBuilder, ImageSource, image, load_image_from_source},
    surface::{SurfaceArgsBuilder, surface},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

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
                    .build()
                    .unwrap(),
                None,
                move || {
                    image(
                        ImageArgsBuilder::default()
                            .data(image_data)
                            .width(DimensionValue::Fixed(Dp(200.0).into()))
                            .height(DimensionValue::Fixed(Dp(200.0).into()))
                            .build()
                            .unwrap(),
                    )
                },
            );
        },
        |app| {
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
