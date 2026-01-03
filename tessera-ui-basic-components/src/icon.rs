//! A component for rendering raster or vector icons.
//!
//! ## Usage
//!
//! Use to display a scalable icon from image or vector data.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Px,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera, use_context,
};

use crate::{
    image_vector::TintMode,
    pipelines::{
        image::command::{ImageCommand, ImageData},
        image_vector::command::{ImageVectorCommand, ImageVectorData},
    },
    theme::ContentColor,
};

/// Icon content can be provided either as vector geometry or raster pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconContent {
    /// Render the icon via the vector pipeline.
    Vector(Arc<ImageVectorData>),
    /// Render the icon via the raster image pipeline.
    Raster(Arc<ImageData>),
}

impl From<ImageVectorData> for IconContent {
    fn from(data: ImageVectorData) -> Self {
        Self::Vector(Arc::new(data))
    }
}

impl From<Arc<ImageVectorData>> for IconContent {
    fn from(data: Arc<ImageVectorData>) -> Self {
        Self::Vector(data)
    }
}

impl From<ImageData> for IconContent {
    fn from(data: ImageData) -> Self {
        Self::Raster(Arc::new(data))
    }
}

impl From<Arc<ImageData>> for IconContent {
    fn from(data: Arc<ImageData>) -> Self {
        Self::Raster(data)
    }
}

/// Arguments for the [`icon`] component.
#[derive(Debug, Setters, Clone)]
pub struct IconArgs {
    /// Icon content, provided as either raster pixels or vector geometry.
    #[setters(into)]
    pub content: IconContent,
    /// Logical size of the icon. Applied to both width and height unless
    /// explicit overrides are provided through [`width`](IconArgs::width) /
    /// [`height`](IconArgs::height).
    pub size: Dp,
    /// Optional width override. Handy when the icon should `Fill` or `Wrap`
    /// differently from the default square sizing.
    #[setters(strip_option)]
    pub width: Option<DimensionValue>,
    /// Optional height override. Handy when the icon should `Fill` or `Wrap`
    /// differently from the default square sizing.
    #[setters(strip_option)]
    pub height: Option<DimensionValue>,
    /// Tint color applied to vector icons. Defaults to white so it preserves
    /// the original colors (multiplying by white is a no-op). Raster icons
    /// ignore this field.
    pub tint: Color,
    /// How the tint is applied to vector icons.
    pub tint_mode: TintMode,
    /// Rotation angle in degrees.
    pub rotation: f32,
}

impl From<IconContent> for IconArgs {
    fn from(content: IconContent) -> Self {
        Self {
            content,
            size: Dp(24.0),
            width: None,
            height: None,
            tint: use_context::<ContentColor>().get().current,
            tint_mode: TintMode::default(),
            rotation: 0.0,
        }
    }
}

impl From<ImageVectorData> for IconArgs {
    fn from(data: ImageVectorData) -> Self {
        IconContent::from(data).into()
    }
}

impl From<Arc<ImageVectorData>> for IconArgs {
    fn from(data: Arc<ImageVectorData>) -> Self {
        IconContent::from(data).into()
    }
}

impl From<ImageData> for IconArgs {
    fn from(data: ImageData) -> Self {
        IconContent::from(data).into()
    }
}

impl From<Arc<ImageData>> for IconArgs {
    fn from(data: Arc<ImageData>) -> Self {
        IconContent::from(data).into()
    }
}

#[derive(Clone, PartialEq)]
struct IconLayout {
    content: IconContent,
    size: Dp,
    width: Option<DimensionValue>,
    height: Option<DimensionValue>,
    tint: Color,
    tint_mode: TintMode,
    rotation: f32,
}

impl LayoutSpec for IconLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let (intrinsic_width, intrinsic_height) = intrinsic_dimensions(&self.content);
        let size_px = self.size.to_px();

        let preferred_width = self.width.unwrap_or(DimensionValue::Fixed(size_px));
        let preferred_height = self.height.unwrap_or(DimensionValue::Fixed(size_px));

        let constraint = Constraint::new(preferred_width, preferred_height);
        let effective_constraint = constraint.merge(input.parent_constraint());

        let width = match effective_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(intrinsic_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => {
                let parent_max = input
                    .parent_constraint()
                    .width()
                    .get_max()
                    .unwrap_or(Px::MAX);
                max.unwrap_or(parent_max)
                    .max(min.unwrap_or(Px(0)))
                    .max(intrinsic_width)
            }
        };

        let height = match effective_constraint.height {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(intrinsic_height)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => {
                let parent_max = input
                    .parent_constraint()
                    .height()
                    .get_max()
                    .unwrap_or(Px::MAX);
                max.unwrap_or(parent_max)
                    .max(min.unwrap_or(Px(0)))
                    .max(intrinsic_height)
            }
        };

        Ok(ComputedData { width, height })
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        match &self.content {
            IconContent::Vector(data) => {
                let command = ImageVectorCommand {
                    data: data.clone(),
                    tint: self.tint,
                    tint_mode: self.tint_mode,
                    rotation: self.rotation,
                };
                metadata.push_draw_command(command);
            }
            IconContent::Raster(data) => {
                let command = ImageCommand {
                    data: data.clone(),
                    opacity: 1.0,
                };
                metadata.push_draw_command(command);
            }
        }
    }
}

/// # icon
///
/// Renders an icon with consistent sizing and optional tinting for vectors.
///
/// ## Usage
///
/// Display a vector or raster image with a uniform size, often inside a button
/// or as a status indicator.
///
/// ## Parameters
///
/// - `args` — configures the icon's content, size, and tint; see [`IconArgs`].
///
/// ## Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     icon::{IconArgs, icon},
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
/// };
///
/// // Load vector data from an SVG file.
/// // In a real app, this should be done once and the data cached.
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data =
///     load_image_vector_from_source(&ImageVectorSource::Path(svg_path.to_string())).unwrap();
///
/// icon(IconArgs::from(vector_data).tint(Color::new(0.2, 0.5, 0.8, 1.0)));
/// ```
#[tessera]
pub fn icon(args: impl Into<IconArgs>) {
    let icon_args: IconArgs = args.into();

    layout(IconLayout {
        content: icon_args.content,
        size: icon_args.size,
        width: icon_args.width,
        height: icon_args.height,
        tint: icon_args.tint,
        tint_mode: icon_args.tint_mode,
        rotation: icon_args.rotation,
    });
}

fn intrinsic_dimensions(content: &IconContent) -> (Px, Px) {
    match content {
        IconContent::Vector(data) => (
            px_from_f32(data.viewport_width),
            px_from_f32(data.viewport_height),
        ),
        IconContent::Raster(data) => (clamp_u32_to_px(data.width), clamp_u32_to_px(data.height)),
    }
}

fn px_from_f32(value: f32) -> Px {
    let clamped = value.max(0.0).min(i32::MAX as f32);
    Px(clamped.round() as i32)
}

fn clamp_u32_to_px(value: u32) -> Px {
    Px::new(value.min(i32::MAX as u32) as i32)
}
