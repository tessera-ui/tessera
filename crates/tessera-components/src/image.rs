//! Raster image component and decoding utilities.
//!
//! ## Usage
//!
//! Use to display images from pre-decoded data or bytes/assets loaded once.
use std::path::{Path, PathBuf};

use image::GenericImageView;
use tessera_ui::{
    Color, ComputedData, LayoutResult, MeasurementError, Modifier, Px,
    layout::{MeasureScope, layout},
    tessera,
};

pub use image::ImageError as ImageLoadError;

use crate::{
    image_vector::{ImageVectorLoadError, TintMode, TryIntoImageVectorData},
    painter::{Painter, PainterLoadError, TryIntoPainter},
    pipelines::{
        image::command::ImageCommand,
        image_vector::command::{ImageVectorCommand, ImageVectorData},
    },
};

pub use crate::pipelines::image::command::ImageData;

/// Converts a source into decoded raster image data.
pub trait TryIntoImageData {
    /// Convert this source into decoded image data.
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError>;
}

fn placeholder_image_data() -> ImageData {
    ImageData::from_rgba(vec![0, 0, 0, 0], 1, 1)
}

fn decode_dynamic_image(decoded: image::DynamicImage) -> ImageData {
    let (width, height) = decoded.dimensions();
    ImageData::from_rgba(decoded.into_rgba8().into_raw(), width, height)
}

fn decode_image_from_bytes(bytes: &[u8]) -> Result<ImageData, ImageLoadError> {
    let decoded = image::load_from_memory(bytes)?;
    Ok(decode_dynamic_image(decoded))
}

fn decode_image_from_path(path: &Path) -> Result<ImageData, ImageLoadError> {
    let decoded = image::open(path)?;
    Ok(decode_dynamic_image(decoded))
}

impl TryIntoImageData for ImageData {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        Ok(self)
    }
}

impl TryIntoImageData for Vec<u8> {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        decode_image_from_bytes(&self)
    }
}

impl TryIntoImageData for &[u8] {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        decode_image_from_bytes(self)
    }
}

impl TryIntoImageData for String {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        decode_image_from_path(Path::new(&self))
    }
}

impl TryIntoImageData for &str {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        decode_image_from_path(Path::new(self))
    }
}

impl TryIntoImageData for PathBuf {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        decode_image_from_path(self.as_path())
    }
}

impl TryIntoImageData for &Path {
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError> {
        decode_image_from_path(self)
    }
}

impl ImageBuilder {
    /// Replaces the image content with a shared painter value.
    pub fn painter(&self, painter: impl Into<Painter>) -> Self {
        let value = Some(painter.into());
        self.props
            .configure("painter", move |props| props.painter = value);
        Self { props: self.props }
    }

    /// Decodes image content from any supported source.
    pub fn try_painter<T>(&self, source: T) -> Result<Self, PainterLoadError>
    where
        T: TryIntoPainter,
    {
        let value = Some(source.try_into_painter()?);
        self.props
            .configure("painter", move |props| props.painter = value);
        Ok(Self { props: self.props })
    }

    /// Replaces the image data with already-decoded raster pixels.
    pub fn raster(&self, data: impl Into<ImageData>) -> Self {
        let value = Some(Painter::Raster(data.into()));
        self.props
            .configure("painter", move |props| props.painter = value);
        Self { props: self.props }
    }

    /// Replaces the image content with already-decoded vector geometry.
    pub fn vector(&self, data: impl Into<ImageVectorData>) -> Self {
        let value = Some(Painter::Vector(data.into()));
        self.props
            .configure("painter", move |props| props.painter = value);
        Self { props: self.props }
    }

    /// Decodes raster image data from bytes/path/asset input and stores it.
    pub fn try_raster<T>(&self, source: T) -> Result<Self, ImageLoadError>
    where
        T: TryIntoImageData,
    {
        let value = Some(Painter::Raster(source.try_into_image_data()?));
        self.props
            .configure("painter", move |props| props.painter = value);
        Ok(Self { props: self.props })
    }

    /// Decodes vector image data from bytes/path input and stores it.
    pub fn try_vector<T>(&self, source: T) -> Result<Self, ImageVectorLoadError>
    where
        T: TryIntoImageVectorData,
    {
        let value = Some(Painter::Vector(source.try_into_image_vector_data()?));
        self.props
            .configure("painter", move |props| props.painter = value);
        Ok(Self { props: self.props })
    }
}

#[derive(Clone, PartialEq)]
struct ImageLayout {
    painter: Painter,
}

impl tessera_ui::LayoutModifierNode for ImageLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let (intrinsic_width, intrinsic_height) = match &self.painter {
            Painter::Asset(_) => (Px::ZERO, Px::ZERO),
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
            Painter::Asset(_) => {}
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
