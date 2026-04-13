//! Raster image component and decoding utilities.
//!
//! ## Usage
//!
//! Use to display images from pre-decoded data or bytes/assets loaded once.
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use image::GenericImageView;
use tessera_ui::{
    AssetExt, Color, ComputedData, LayoutResult, MeasurementError, Modifier, Px,
    layout::{LayoutPolicy, MeasureScope, RenderInput, RenderPolicy, layout},
    tessera,
};
use thiserror::Error;

use crate::{
    image_vector::{ImageVectorLoadError, TintMode, TryIntoImageVectorData},
    painter::{Painter, PainterLoadError, TryIntoPainter, try_painter_asset},
    pipelines::{
        image::command::ImageCommand,
        image_vector::command::{ImageVectorCommand, ImageVectorData},
    },
};

pub use crate::pipelines::image::command::ImageData;

/// Errors that can occur while loading raster image data.
#[derive(Debug, Error)]
pub enum ImageLoadError {
    /// Failed to read bytes from an asset handle.
    #[error("failed to read image bytes from asset: {source}")]
    AssetRead {
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// Image decoding failed.
    #[error(transparent)]
    Decode(#[from] image::ImageError),
}

/// Converts a source into decoded raster image data.
pub trait TryIntoImageData {
    /// Convert this source into decoded image data.
    fn try_into_image_data(self) -> Result<ImageData, ImageLoadError>;
}

fn placeholder_image_data() -> Arc<ImageData> {
    Arc::new(ImageData {
        data: Arc::new(vec![0, 0, 0, 0]),
        width: 1,
        height: 1,
    })
}

fn decode_dynamic_image(decoded: image::DynamicImage) -> ImageData {
    let (width, height) = decoded.dimensions();
    ImageData {
        data: Arc::new(decoded.to_rgba8().into_raw()),
        width,
        height,
    }
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
    pub fn painter(mut self, painter: impl Into<Painter>) -> Self {
        self.props.painter = Some(painter.into());
        self
    }

    /// Decodes image content from any supported source.
    pub fn try_painter<T>(mut self, source: T) -> Result<Self, PainterLoadError>
    where
        T: TryIntoPainter,
    {
        self.props.painter = Some(source.try_into_painter()?);
        Ok(self)
    }

    /// Decodes image content from an asset handle.
    pub fn try_painter_asset<T>(mut self, asset: T) -> Result<Self, PainterLoadError>
    where
        T: AssetExt,
    {
        self.props.painter = Some(try_painter_asset(asset)?);
        Ok(self)
    }

    /// Replaces the image data with already-decoded raster pixels.
    pub fn raster(mut self, data: impl Into<Arc<ImageData>>) -> Self {
        self.props.painter = Some(Painter::Raster(data.into()));
        self
    }

    /// Replaces the image content with already-decoded vector geometry.
    pub fn vector(mut self, data: impl Into<Arc<ImageVectorData>>) -> Self {
        self.props.painter = Some(Painter::Vector(data.into()));
        self
    }

    /// Decodes raster image data from bytes/path/asset input and stores it.
    pub fn try_raster<T>(mut self, source: T) -> Result<Self, ImageLoadError>
    where
        T: TryIntoImageData,
    {
        self.props.painter = Some(Painter::Raster(Arc::new(source.try_into_image_data()?)));
        Ok(self)
    }

    /// Decodes vector image data from bytes/path input and stores it.
    pub fn try_vector<T>(mut self, source: T) -> Result<Self, ImageVectorLoadError>
    where
        T: TryIntoImageVectorData,
    {
        self.props.painter = Some(Painter::Vector(source.try_into_image_vector_data()?));
        Ok(self)
    }

    /// Decodes raster image data from an asset handle and stores it.
    pub fn try_raster_asset<T>(mut self, asset: T) -> Result<Self, ImageLoadError>
    where
        T: AssetExt,
    {
        let bytes = asset
            .read()
            .map_err(|source| ImageLoadError::AssetRead { source })?;
        self.props.painter = Some(Painter::Raster(Arc::new(decode_image_from_bytes(
            bytes.as_ref(),
        )?)));
        Ok(self)
    }

    /// Decodes vector image data from an asset handle and stores it.
    pub fn try_vector_asset<T>(mut self, asset: T) -> Result<Self, ImageVectorLoadError>
    where
        T: AssetExt,
    {
        let bytes = asset
            .read()
            .map_err(|source| ImageVectorLoadError::AssetRead { source })?;
        self.props.painter = Some(Painter::Vector(
            bytes.as_ref().try_into_image_vector_data()?,
        ));
        Ok(self)
    }
}

#[derive(Clone, PartialEq)]
struct ImageLayout {
    painter: Painter,
}

impl LayoutPolicy for ImageLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let (intrinsic_width, intrinsic_height) = match &self.painter {
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

impl RenderPolicy for ImageLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
        match &self.painter {
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
/// use std::sync::Arc;
/// use tessera_components::image::{ImageData, image};
///
/// let image_data = ImageData {
///     data: Arc::new(vec![255, 255, 255, 255]),
///     width: 1,
///     height: 1,
/// };
///
/// image().painter(image_data);
/// # }
/// ```
#[tessera]
pub fn image(#[prop(skip_setter)] painter: Option<Painter>, modifier: Modifier) {
    let painter = painter.unwrap_or_else(|| Painter::Raster(placeholder_image_data()));
    let policy = ImageLayout {
        painter: painter.clone(),
    };
    layout()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy);
}

fn clamp_f32_to_px(value: f32) -> Px {
    let clamped = value.max(0.0).min(i32::MAX as f32);
    Px(clamped.round() as i32)
}
