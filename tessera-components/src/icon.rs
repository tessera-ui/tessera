//! Icon component for rendering vector or raster imagery.
//!
//! ## Usage
//!
//! Use to display a tintable symbol in buttons, tabs, and status indicators.
use std::sync::{Arc, OnceLock};

use tessera_ui::{
    AssetExt, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Prop, Px,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera, use_context,
};

use crate::{
    image::{ImageLoadError, TryIntoImageData},
    image_vector::{ImageVectorLoadError, TintMode, TryIntoImageVectorData},
    pipelines::{
        image::command::{ImageCommand, ImageData},
        image_vector::command::{ImageVectorCommand, ImageVectorData},
    },
    theme::{ContentColor, MaterialTheme},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum IconContent {
    Vector(Arc<ImageVectorData>),
    Raster(Arc<ImageData>),
}

fn placeholder_raster_data() -> Arc<ImageData> {
    static PLACEHOLDER: OnceLock<Arc<ImageData>> = OnceLock::new();
    PLACEHOLDER
        .get_or_init(|| {
            Arc::new(ImageData {
                data: Arc::new(vec![0, 0, 0, 0]),
                width: 1,
                height: 1,
            })
        })
        .clone()
}

fn default_tint_color() -> Color {
    let theme = use_context::<MaterialTheme>();
    use_context::<ContentColor>()
        .map(|c| c.get().current)
        .or_else(|| theme.map(|t| t.get().color_scheme.on_surface))
        .unwrap_or_else(|| ContentColor::default().current)
}

/// Arguments for the [`icon`] component.
#[derive(Debug, Prop, Clone)]
pub struct IconArgs {
    /// Icon content, internally tracked as either vector geometry or raster
    /// pixels.
    #[prop(skip_setter)]
    content: IconContent,
    /// Logical size of the icon. Applied to both width and height unless
    /// explicit overrides are provided through [`width`](IconArgs::width) /
    /// [`height`](IconArgs::height).
    pub size: Dp,
    /// Optional width override. Handy when the icon should `Fill` or `Wrap`
    /// differently from the default square sizing.
    pub width: Option<DimensionValue>,
    /// Optional height override. Handy when the icon should `Fill` or `Wrap`
    /// differently from the default square sizing.
    pub height: Option<DimensionValue>,
    /// Tint color applied to vector icons. Defaults to the current content
    /// color. Raster icons ignore this field.
    pub tint: Color,
    /// How the tint is applied to vector icons.
    pub tint_mode: TintMode,
    /// Rotation angle in degrees.
    pub rotation: f32,
}

impl Default for IconArgs {
    fn default() -> Self {
        Self {
            content: IconContent::Raster(placeholder_raster_data()),
            size: Dp(24.0),
            width: None,
            height: None,
            tint: default_tint_color(),
            tint_mode: TintMode::default(),
            rotation: 0.0,
        }
    }
}

impl IconArgs {
    /// Sets vector icon content using already-decoded vector geometry.
    pub fn vector(mut self, data: impl Into<Arc<ImageVectorData>>) -> Self {
        self.content = IconContent::Vector(data.into());
        self
    }

    /// Sets raster icon content using already-decoded image pixels.
    pub fn raster(mut self, data: impl Into<Arc<ImageData>>) -> Self {
        self.content = IconContent::Raster(data.into());
        self
    }

    /// Decodes vector icon content from bytes/path/asset input.
    pub fn try_vector<T>(mut self, source: T) -> Result<Self, ImageVectorLoadError>
    where
        T: TryIntoImageVectorData,
    {
        self.content = IconContent::Vector(source.try_into_image_vector_data()?);
        Ok(self)
    }

    /// Decodes raster icon content from bytes/path/asset input.
    pub fn try_raster<T>(mut self, source: T) -> Result<Self, ImageLoadError>
    where
        T: TryIntoImageData,
    {
        self.content = IconContent::Raster(Arc::new(source.try_into_image_data()?));
        Ok(self)
    }

    /// Decodes vector icon content from an asset handle.
    pub fn try_vector_asset<T>(mut self, asset: T) -> Result<Self, ImageVectorLoadError>
    where
        T: AssetExt,
    {
        let bytes = asset
            .read()
            .map_err(|source| ImageVectorLoadError::AssetRead { source })?;
        self.content = IconContent::Vector(bytes.as_ref().try_into_image_vector_data()?);
        Ok(self)
    }

    /// Decodes raster icon content from an asset handle.
    pub fn try_raster_asset<T>(mut self, asset: T) -> Result<Self, ImageLoadError>
    where
        T: AssetExt,
    {
        let bytes = asset
            .read()
            .map_err(|source| ImageLoadError::AssetRead { source })?;
        self.content = IconContent::Raster(Arc::new(bytes.as_ref().try_into_image_data()?));
        Ok(self)
    }
}

impl From<ImageVectorData> for IconArgs {
    fn from(data: ImageVectorData) -> Self {
        Self::default().vector(Arc::new(data))
    }
}

impl From<Arc<ImageVectorData>> for IconArgs {
    fn from(data: Arc<ImageVectorData>) -> Self {
        Self::default().vector(data)
    }
}

impl From<ImageData> for IconArgs {
    fn from(data: ImageData) -> Self {
        Self::default().raster(Arc::new(data))
    }
}

impl From<Arc<ImageData>> for IconArgs {
    fn from(data: Arc<ImageData>) -> Self {
        Self::default().raster(data)
    }
}

impl From<crate::material_icons::Asset> for Arc<ImageVectorData> {
    fn from(asset: crate::material_icons::Asset) -> Self {
        asset
            .try_into_image_vector_data()
            .expect("bundled material icon svg should load")
    }
}

impl From<crate::material_icons::Asset> for IconArgs {
    fn from(asset: crate::material_icons::Asset) -> Self {
        Self::default().vector(asset)
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
                metadata.fragment_mut().push_draw_command(command);
            }
            IconContent::Raster(data) => {
                let command = ImageCommand {
                    data: data.clone(),
                    opacity: 1.0,
                };
                metadata.fragment_mut().push_draw_command(command);
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
/// Display a vector or raster symbol with uniform size, typically in controls
/// and compact labels.
///
/// ## Parameters
///
/// - `args` - configures the icon content, size, and tint; see [`IconArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     icon::{IconArgs, icon},
///     material_icons::filled,
/// };
/// use tessera_ui::Color;
///
/// let args = IconArgs::default()
///     .vector(filled::STAR_SVG)
///     .size(tessera_ui::Dp(20.0))
///     .tint(Color::new(0.2, 0.5, 0.8, 1.0));
///
/// icon(&args);
/// # }
/// ```
#[tessera]
pub fn icon(args: &IconArgs) {
    layout(IconLayout {
        content: args.content.clone(),
        size: args.size,
        width: args.width,
        height: args.height,
        tint: args.tint,
        tint_mode: args.tint_mode,
        rotation: args.rotation,
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
