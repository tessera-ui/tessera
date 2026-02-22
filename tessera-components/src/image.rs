//! A component for rendering raster images.
//!
//! ## Usage
//!
//! Use to display static or dynamically loaded images.
use std::sync::Arc;

use derive_setters::Setters;
use image::GenericImageView;
use tessera_ui::{
    ComputedData, DimensionValue, MeasurementError, Modifier, Px,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera,
};

use crate::pipelines::image::command::ImageCommand;

pub use crate::pipelines::image::command::ImageData;

/// Specifies the source for image data, which can be either a file path or raw
/// bytes.
///
/// This enum is used by [`load_image_from_source`] to load image data from
/// different sources.
#[derive(Clone, PartialEq, Debug)]
pub enum ImageSource {
    /// Load image from a file path.
    Path(String),
    /// Load image from a byte slice. The data is wrapped in an `Arc` for
    /// efficient sharing.
    Bytes(Arc<[u8]>),
}

/// Decodes an image from a given [`ImageSource`].
///
/// This function handles the loading and decoding of the image data into a
/// format suitable for rendering. It should be called outside of the main UI
/// loop or a component's `measure` closure to avoid performance degradation
/// from decoding the image on every frame.
///
/// # Arguments
///
/// * `source` - A reference to the [`ImageSource`] to load the image from.
///
/// # Returns
///
/// A `Result` containing the decoded [`ImageData`] on success, or an
/// `image::ImageError` on failure.
pub fn load_image_from_source(source: &ImageSource) -> Result<ImageData, image::ImageError> {
    let decoded = match source {
        ImageSource::Path(path) => image::open(path)?,
        ImageSource::Bytes(bytes) => image::load_from_memory(bytes)?,
    };
    let (width, height) = decoded.dimensions();
    Ok(ImageData {
        data: Arc::new(decoded.to_rgba8().into_raw()),
        width,
        height,
    })
}

/// Arguments for the `image` component.
///
/// This struct holds the data and layout properties for an `image` component.
/// It is typically created using fluent setters or by converting from
/// [`ImageData`].
#[derive(PartialEq, Debug, Setters, Clone)]
pub struct ImageArgs {
    /// The decoded image data, represented by [`ImageData`]. This contains the
    /// raw pixel buffer and the image's dimensions.
    #[setters(into)]
    pub data: Arc<ImageData>,

    /// Optional modifier chain applied to the image node.
    pub modifier: Modifier,
}

impl From<ImageData> for ImageArgs {
    fn from(data: ImageData) -> Self {
        Self {
            data: Arc::new(data),
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
/// Display a static asset or a dynamically loaded image from a file or memory.
///
/// ## Parameters
///
/// - `args` — configures the image data and layout; see [`ImageArgs`].
///
/// ## Examples
///
/// ```ignore
/// use std::sync::Arc;
/// use tessera_components::image::{
///     ImageSource, image, load_image_from_source,
/// };
///
/// // In a real app, you might load image bytes from a file at runtime.
/// // For this example, we include the bytes at compile time.
/// let image_bytes = Arc::new(*include_bytes!("image.png"));
/// let image_data =
///     load_image_from_source(&ImageSource::Bytes(image_bytes)).expect("Failed to load image");
///
/// // Render the image using its loaded data.
/// image(image_data);
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
