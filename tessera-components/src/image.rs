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
    AssetExt, ComputedData, DimensionValue, MeasurementError, Modifier, Prop, Px,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera,
};
use thiserror::Error;

use crate::pipelines::image::command::ImageCommand;

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

/// Arguments for the `image` component.
#[derive(Debug, Prop, Clone)]
pub struct ImageArgs {
    /// Decoded image data containing RGBA pixels and dimensions.
    #[prop(into)]
    pub data: Arc<ImageData>,

    /// Optional modifier chain applied to the image node.
    pub modifier: Modifier,
}

impl ImageArgs {
    /// Replaces the image data with already-decoded raster pixels.
    pub fn raster(mut self, data: impl Into<Arc<ImageData>>) -> Self {
        self.data = data.into();
        self
    }

    /// Decodes raster image data from bytes/path/asset input and stores it.
    pub fn try_raster<T>(mut self, source: T) -> Result<Self, ImageLoadError>
    where
        T: TryIntoImageData,
    {
        self.data = Arc::new(source.try_into_image_data()?);
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
        self.data = Arc::new(decode_image_from_bytes(bytes.as_ref())?);
        Ok(self)
    }
}

impl From<ImageData> for ImageArgs {
    fn from(data: ImageData) -> Self {
        Self {
            data: Arc::new(data),
            modifier: Modifier::new(),
        }
    }
}

impl From<Arc<ImageData>> for ImageArgs {
    fn from(data: Arc<ImageData>) -> Self {
        Self {
            data,
            modifier: Modifier::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct ImageLayout {
    data: Arc<ImageData>,
}

impl LayoutSpec for ImageLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let intrinsic_width = Px(self.data.width as i32);
        let intrinsic_height = Px(self.data.height as i32);

        let width = match input.parent_constraint().width() {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(intrinsic_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect("Seems that you are trying to fill an infinite width, which is not allowed")
                .max(min.unwrap_or(Px(0)))
                .max(intrinsic_width),
        };

        let height = match input.parent_constraint().height() {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(intrinsic_height)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect(
                    "Seems that you are trying to fill an infinite height, which is not allowed",
                )
                .max(min.unwrap_or(Px(0)))
                .max(intrinsic_height),
        };

        Ok(ComputedData { width, height })
    }

    fn record(&self, input: &RenderInput<'_>) {
        let image_command = ImageCommand {
            data: self.data.clone(),
            opacity: 1.0,
        };
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(image_command);
    }
}

/// # image
///
/// Renders a raster image, fitting it to the available space or its intrinsic
/// size.
///
/// ## Usage
///
/// Display a static asset or pre-decoded image pixels.
///
/// ## Parameters
///
/// - `args` - configures the image data and layout; see [`ImageArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use std::sync::Arc;
/// use tessera_components::image::{ImageArgs, ImageData, image};
///
/// let image_data = ImageData {
///     data: Arc::new(vec![255, 255, 255, 255]),
///     width: 1,
///     height: 1,
/// };
///
/// image(&ImageArgs::from(image_data));
/// # }
/// ```
#[tessera]
pub fn image(args: &ImageArgs) {
    let modifier = args.modifier.clone();
    let inner_args = args.clone();
    modifier.run(move || image_inner(&inner_args));
}

#[tessera]
fn image_inner(args: &ImageArgs) {
    let data = args.data.clone();
    layout(ImageLayout { data });
}
