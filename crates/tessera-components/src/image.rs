//! Raster image component and decoding utilities.
//!
//! ## Usage
//!
//! Use to display images from pre-decoded data or bytes/assets loaded once.
use tessera_ui::{
    Color, ComputedData, LayoutResult, MeasurementError, Modifier, Px,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::{
    image_vector::TintMode,
    painter::Painter,
    pipelines::{image::command::ImageCommand, image_vector::command::ImageVectorCommand},
};

pub use crate::pipelines::image::command::ImageData;

fn placeholder_image_data() -> ImageData {
    ImageData::from_rgba(vec![0, 0, 0, 0], 1, 1)
}

pub(crate) fn decode_raster(bytes: &[u8]) -> Result<ImageData, image::ImageError> {
    let decoded = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = decoded.dimensions();
    Ok(ImageData::from_rgba(decoded.into_raw(), width, height))
}

impl ImageBuilder {
    /// Replaces the image content with a shared painter value.
    pub fn painter(&self, painter: impl Into<Painter>) -> Self {
        let value = Some(painter.into());
        self.props
            .configure("painter", move |props| props.painter = value);
        Self { props: self.props }
    }
}

#[derive(Clone, PartialEq)]
struct ImageLayout {
    painter: Painter,
}

impl tessera_ui::LayoutModifierNode for ImageLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let (intrinsic_width, intrinsic_height) = match &self.painter {
            Painter::Source(_) => (Px::ZERO, Px::ZERO),
            Painter::Vector(data) => (
                clamp_f32_to_px(data.viewport_width),
                clamp_f32_to_px(data.viewport_height),
            ),
            Painter::Raster(data) => (Px(data.width as i32), Px(data.height as i32)),
        };

        let width = input.parent_constraint().width().clamp(intrinsic_width);
        let height = input.parent_constraint().height().clamp(intrinsic_height);

        Ok(LayoutResult::new(ComputedData { width, height }))
    }
}

impl tessera_ui::DrawModifierNode for ImageLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        match &self.painter {
            Painter::Source(_) => {}
            Painter::Raster(data) => {
                let image_command = ImageCommand {
                    data: data.clone(),
                    opacity: 1.0,
                };
                input
                    .metadata_mut()
                    .fragment_mut()
                    .push_draw_command(image_command);
            }
            Painter::Vector(data) => {
                let vector_command = ImageVectorCommand {
                    data: data.clone(),
                    tint: Color::WHITE,
                    tint_mode: TintMode::Multiply,
                    rotation: 0.0,
                };
                input
                    .metadata_mut()
                    .fragment_mut()
                    .push_draw_command(vector_command);
            }
        }
        content.draw(input);
    }
}

/// # image
///
/// Renders a raster image, fitting it to the available space or its intrinsic
/// size.
///
/// ## Usage
///
/// Display a raster or vector asset using a shared painter payload.
///
/// ## Parameters
///
/// - `painter` - optional painter payload for vector or raster imagery.
/// - `modifier` - node-local layout, drawing, and interaction modifiers.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::image::{ImageData, image};
///
/// let image_data = ImageData::from_rgba(vec![255, 255, 255, 255], 1, 1);
///
/// image().painter(image_data);
/// # }
/// ```
#[tessera]
pub fn image(#[prop(skip_setter)] painter: Option<Painter>, modifier: Option<Modifier>) {
    let modifier = modifier.unwrap_or_default();
    let painter = painter
        .and_then(Painter::resolve)
        .unwrap_or_else(|| Painter::Raster(placeholder_image_data()));
    let policy = ImageLayout {
        painter: painter.clone(),
    };
    layout()
        .modifier(modifier)
        .policy(policy.clone())
        .draw(policy);
}

pub(crate) fn clamp_f32_to_px(value: f32) -> Px {
    let clamped = value.max(0.0).min(i32::MAX as f32);
    Px(clamped.round() as i32)
}
