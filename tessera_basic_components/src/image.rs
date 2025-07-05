use crate::pipelines::image::{ImageCommand, ImageData};
use derive_builder::Builder;
use image::GenericImageView;
use std::sync::Arc;
use tessera::{ComputedData, Constraint, DimensionValue, Px};
use tessera_macros::tessera;

/// Source of the image data.
#[derive(Clone, Debug)]
pub enum ImageSource {
    /// Load image from a file path.
    Path(String),
    /// Load image from raw bytes.
    Bytes(Arc<[u8]>),
}

/// Decodes an image from a given source.
///
/// This function should be called outside of the component's `measure` closure
/// to avoid decoding the image on every frame.
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
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ImageArgs {
    /// The decoded image data.
    pub data: ImageData,

    /// Optional explicit width for the image.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,

    /// Optional explicit height for the image.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
}

impl From<ImageData> for ImageArgs {
    fn from(data: ImageData) -> Self {
        ImageArgsBuilder::default().data(data).build().unwrap()
    }
}

#[tessera]
pub fn image(args: impl Into<ImageArgs>) {
    let image_args: ImageArgs = args.into();

    measure(Box::new(move |input| {
        let intrinsic_width = Px(image_args.data.width as i32);
        let intrinsic_height = Px(image_args.data.height as i32);

        let image_intrinsic_width = image_args.width.unwrap_or(DimensionValue::Wrap {
            min: Some(intrinsic_width),
            max: Some(intrinsic_width),
        });
        let image_intrinsic_height = image_args.height.unwrap_or(DimensionValue::Wrap {
            min: Some(intrinsic_height),
            max: Some(intrinsic_height),
        });

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
                let parent_max = input.parent_constraint.width.to_max_px(Px::MAX);
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
                let parent_max = input.parent_constraint.height.to_max_px(Px::MAX);
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
            .basic_drawable = Some(Box::new(image_command));

        Ok(ComputedData { width, height })
    }));
}
