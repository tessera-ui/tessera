//! This module provides the `image` component and related utilities for rendering images in Tessera UI.
//!
//! It supports loading image data from file paths or raw bytes, decoding them into a format suitable for GPU rendering,
//! and displaying them as part of the UI component tree. The main entry point is the [`image()`] component, which can be
//! sized explicitly or use the intrinsic dimensions of the image. Image data should be loaded and decoded outside the
//! main UI loop for optimal performance, using [`load_image_from_source`].
//!
//! Typical use cases include displaying static images, icons, or dynamically loaded pictures in UI layouts.
//! The module is designed to integrate seamlessly with Tessera's stateless component model and rendering pipeline.

use std::sync::Arc;

use derive_builder::Builder;
use image::GenericImageView;
use tessera_ui::{ComputedData, Constraint, DimensionValue, Px, tessera};

use crate::pipelines::image::ImageCommand;

pub use crate::pipelines::image::ImageData;

/// Specifies the source for image data, which can be either a file path or raw bytes.
///
/// This enum is used by [`load_image_from_source`] to load image data from different sources.
#[derive(Clone, Debug)]
pub enum ImageSource {
    /// Load image from a file path.
    Path(String),
    /// Load image from a byte slice. The data is wrapped in an `Arc` for efficient sharing.
    Bytes(Arc<[u8]>),
}

/// Decodes an image from a given [`ImageSource`].
///
/// This function handles the loading and decoding of the image data into a format
/// suitable for rendering. It should be called outside of the main UI loop or
/// a component's `measure` closure to avoid performance degradation from decoding
/// the image on every frame.
///
/// # Arguments
///
/// * `source` - A reference to the [`ImageSource`] to load the image from.
///
/// # Returns
///
/// A `Result` containing the decoded [`ImageData`] on success, or an `image::ImageError`
/// on failure.
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
/// It is typically created using the [`ImageArgsBuilder`] or by converting from [`ImageData`].
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ImageArgs {
    /// The decoded image data, represented by [`ImageData`]. This contains the raw pixel
    /// buffer and the image's dimensions.
    #[builder(setter(into))]
    pub data: Arc<ImageData>,

    /// Explicit width for the image.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,

    /// Explicit height for the image.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
}

impl From<ImageData> for ImageArgs {
    fn from(data: ImageData) -> Self {
        ImageArgsBuilder::default()
            .data(Arc::new(data))
            .build()
            .unwrap()
    }
}

/// A component that renders an image.
///
/// The `image` component displays an image based on the provided [`ImageData`].
/// It can be explicitly sized or automatically adjust to the intrinsic dimensions
/// of the image. For optimal performance, image data should be loaded and decoded
/// before being passed to this component, for example, by using the
/// [`load_image_from_source`] function.
///
/// # Arguments
///
/// * `args` - The arguments for the image component, which can be an instance of
///   [`ImageArgs`] or anything that converts into it (e.g., [`ImageData`]).
///
/// # Example
///
/// ```no_run
/// use std::sync::Arc;
/// use tessera_ui_basic_components::{
///     image::{image, load_image_from_source, ImageArgsBuilder, ImageSource, ImageData},
/// };
/// use tessera_ui::{Dp, DimensionValue};
///
/// // In a real application, you would load the image data once and store it.
/// // The `include_bytes!` macro is used here to load file contents at compile time.
/// // For dynamic loading from a file path, you could use `ImageSource::Path`.
/// let image_bytes = Arc::new(*include_bytes!("../../example/examples/assets/scarlet_ut.jpg"));
/// let image_data = load_image_from_source(&ImageSource::Bytes(image_bytes))
///     .expect("Failed to load image");
///
/// // Renders the image with its intrinsic size by passing `ImageData` directly.
/// image(image_data.clone());
///
/// // Renders the image with a fixed width using `ImageArgs`.
/// image(
///     ImageArgsBuilder::default()
///         .data(image_data)
///         .width(DimensionValue::Fixed(Dp(100.0).into()))
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn image(args: impl Into<ImageArgs>) {
    let image_args: ImageArgs = args.into();

    measure(Box::new(move |input| {
        let intrinsic_width = Px(image_args.data.width as i32);
        let intrinsic_height = Px(image_args.data.height as i32);

        let image_intrinsic_width = image_args.width;
        let image_intrinsic_height = image_args.height;

        let image_intrinsic_constraint =
            Constraint::new(image_intrinsic_width, image_intrinsic_height);
        let effective_image_constraint = image_intrinsic_constraint.merge(input.parent_constraint);

        let width = match effective_image_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(intrinsic_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => {
                let parent_max = input.parent_constraint.width.get_max().unwrap_or(Px::MAX);
                max.unwrap_or(parent_max)
                    .max(min.unwrap_or(Px(0)))
                    .max(intrinsic_width)
            }
        };

        let height = match effective_image_constraint.height {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(intrinsic_height)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => {
                let parent_max = input.parent_constraint.height.get_max().unwrap_or(Px::MAX);
                max.unwrap_or(parent_max)
                    .max(min.unwrap_or(Px(0)))
                    .max(intrinsic_height)
            }
        };

        let image_command = ImageCommand {
            data: image_args.data.clone(),
        };

        input
            .metadatas
            .entry(input.current_node_id)
            .or_default()
            .push_draw_command(image_command);

        Ok(ComputedData { width, height })
    }));
}
