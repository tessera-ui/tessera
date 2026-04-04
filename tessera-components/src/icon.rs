//! Icon component for rendering vector or raster imagery.
//!
//! ## Usage
//!
//! Use to display a tintable symbol in buttons, tabs, and status indicators.
use std::sync::{Arc, OnceLock};

use tessera_ui::{
    AssetExt, AxisConstraint, Color, ComputedData, Dp, MeasurementError, Px,
    layout::{
        LayoutInput, LayoutOutput, LayoutPolicy, RenderInput, RenderPolicy, layout_primitive,
    },
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

/// Icon payload used by [`icon`] to render either vector or raster content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconContent {
    /// Vector icon content backed by decoded vector geometry.
    Vector(Arc<ImageVectorData>),
    /// Raster icon content backed by decoded image pixels.
    Raster(Arc<ImageData>),
}

impl From<Arc<ImageVectorData>> for IconContent {
    fn from(data: Arc<ImageVectorData>) -> Self {
        Self::Vector(data)
    }
}

impl From<Arc<ImageData>> for IconContent {
    fn from(data: Arc<ImageData>) -> Self {
        Self::Raster(data)
    }
}

impl From<crate::material_icons::Asset> for IconContent {
    fn from(asset: crate::material_icons::Asset) -> Self {
        Self::Vector(asset.into())
    }
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

impl IconBuilder {
    /// Sets vector icon content using already-decoded vector geometry.
    pub fn vector(mut self, data: impl Into<Arc<ImageVectorData>>) -> Self {
        self.props.content = Some(IconContent::Vector(data.into()));
        self
    }

    /// Sets raster icon content using already-decoded image pixels.
    pub fn raster(mut self, data: impl Into<Arc<ImageData>>) -> Self {
        self.props.content = Some(IconContent::Raster(data.into()));
        self
    }

    /// Decodes vector icon content from bytes/path/asset input.
    pub fn try_vector<T>(mut self, source: T) -> Result<Self, ImageVectorLoadError>
    where
        T: TryIntoImageVectorData,
    {
        self.props.content = Some(IconContent::Vector(source.try_into_image_vector_data()?));
        Ok(self)
    }

    /// Decodes raster icon content from bytes/path/asset input.
    pub fn try_raster<T>(mut self, source: T) -> Result<Self, ImageLoadError>
    where
        T: TryIntoImageData,
    {
        self.props.content = Some(IconContent::Raster(Arc::new(source.try_into_image_data()?)));
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
        self.props.content = Some(IconContent::Vector(
            bytes.as_ref().try_into_image_vector_data()?,
        ));
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
        self.props.content = Some(IconContent::Raster(Arc::new(
            bytes.as_ref().try_into_image_data()?,
        )));
        Ok(self)
    }
}

impl From<crate::material_icons::Asset> for Arc<ImageVectorData> {
    fn from(asset: crate::material_icons::Asset) -> Self {
        asset
            .try_into_image_vector_data()
            .expect("bundled material icon svg should load")
    }
}

#[derive(Clone, PartialEq)]
struct IconLayout {
    content: IconContent,
    size: Dp,
    width: Option<AxisConstraint>,
    height: Option<AxisConstraint>,
    tint: Color,
    tint_mode: TintMode,
    rotation: f32,
}

impl LayoutPolicy for IconLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let (intrinsic_width, intrinsic_height) = intrinsic_dimensions(&self.content);
        let size_px = self.size.to_px();

        let preferred_width = self.width.unwrap_or(AxisConstraint::exact(size_px));
        let preferred_height = self.height.unwrap_or(AxisConstraint::exact(size_px));
        let width = preferred_width
            .intersect(input.parent_constraint().width())
            .clamp(intrinsic_width);
        let height = preferred_height
            .intersect(input.parent_constraint().height())
            .clamp(intrinsic_height);

        Ok(ComputedData { width, height })
    }
}

impl RenderPolicy for IconLayout {
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
/// - `content` - optional vector or raster icon payload.
/// - `size` - optional preferred square size.
/// - `width` / `height` - optional explicit layout dimensions.
/// - `tint` - optional tint override for vector icons.
/// - `tint_mode` - tint blending mode for vector icons.
/// - `rotation` - clockwise rotation in degrees.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{icon::icon, material_icons::filled};
/// use tessera_ui::Color;
///
/// icon()
///     .vector(filled::STAR_SVG)
///     .size(tessera_ui::Dp(20.0))
///     .tint(Color::new(0.2, 0.5, 0.8, 1.0));
/// # }
/// ```
#[tessera]
pub fn icon(
    #[prop(skip_setter)] content: Option<IconContent>,
    size: Option<Dp>,
    width: Option<AxisConstraint>,
    height: Option<AxisConstraint>,
    tint: Option<Color>,
    tint_mode: TintMode,
    rotation: f32,
) {
    let content = content.unwrap_or_else(|| IconContent::Raster(placeholder_raster_data()));
    let size = size.unwrap_or(Dp(24.0));
    let tint = tint.unwrap_or_else(default_tint_color);
    let policy = IconLayout {
        content,
        size,
        width,
        height,
        tint,
        tint_mode,
        rotation,
    };
    layout_primitive()
        .layout_policy(policy.clone())
        .render_policy(policy);
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
