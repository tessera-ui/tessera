//! Icon component for rendering vector or raster imagery.
//!
//! ## Usage
//!
//! Use to display a tintable symbol in buttons, tabs, and status indicators.
use std::sync::Arc;

use tessera_ui::{
    AxisConstraint, Color, ComputedData, Dp, LayoutResult, MeasurementError, Px,
    layout::{MeasureScope, layout},
    tessera, use_context,
};

use crate::{
    image::{ImageLoadError, TryIntoImageData, clamp_f32_to_px},
    image_vector::{ImageVectorLoadError, TintMode, TryIntoImageVectorData},
    painter::{Painter, PainterLoadError, TryIntoPainter},
    pipelines::{
        image::command::{ImageCommand, ImageData},
        image_vector::command::{ImageVectorCommand, ImageVectorData},
    },
    theme::{ContentColor, MaterialTheme},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IconSize(pub Dp);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IconRotation(pub f32);

struct PlaceholderRaster(Arc<ImageData>);

impl Default for PlaceholderRaster {
    fn default() -> Self {
        Self(Arc::new(ImageData::from_rgba(vec![0, 0, 0, 0], 1, 1)))
    }
}

fn placeholder_raster_data() -> Arc<ImageData> {
    Arc::clone(&tessera_ui::__private::runtime_resource::<PlaceholderRaster>().0)
}

impl IconBuilder {
    /// Sets icon content using a shared painter value.
    pub fn painter(&self, painter: impl Into<Painter>) -> Self {
        let value = Some(painter.into());
        self.props
            .configure("content", move |props| props.content = value);
        Self { props: self.props }
    }

    /// Decodes icon content from any supported source.
    pub fn try_painter<T>(&self, source: T) -> Result<Self, PainterLoadError>
    where
        T: TryIntoPainter,
    {
        let value = Some(source.try_into_painter()?);
        self.props
            .configure("content", move |props| props.content = value);
        Ok(Self { props: self.props })
    }

    /// Sets vector icon content using already-decoded vector geometry.
    pub fn vector(&self, data: impl Into<ImageVectorData>) -> Self {
        let value = Some(Painter::Vector(data.into()));
        self.props
            .configure("content", move |props| props.content = value);
        Self { props: self.props }
    }

    /// Sets raster icon content using already-decoded image pixels.
    pub fn raster(&self, data: impl Into<ImageData>) -> Self {
        let value = Some(Painter::Raster(data.into()));
        self.props
            .configure("content", move |props| props.content = value);
        Self { props: self.props }
    }

    /// Decodes vector icon content from bytes/path/asset input.
    pub fn try_vector<T>(&self, source: T) -> Result<Self, ImageVectorLoadError>
    where
        T: TryIntoImageVectorData,
    {
        let value = Some(Painter::Vector(source.try_into_image_vector_data()?));
        self.props
            .configure("content", move |props| props.content = value);
        Ok(Self { props: self.props })
    }

    /// Decodes raster icon content from bytes/path/asset input.
    pub fn try_raster<T>(&self, source: T) -> Result<Self, ImageLoadError>
    where
        T: TryIntoImageData,
    {
        let value = Some(Painter::Raster(source.try_into_image_data()?));
        self.props
            .configure("content", move |props| props.content = value);
        Ok(Self { props: self.props })
    }
}

#[derive(Clone, PartialEq)]
struct IconLayout {
    content: Painter,
    size: Dp,
    width: Option<AxisConstraint>,
    height: Option<AxisConstraint>,
    tint: Color,
    tint_mode: TintMode,
    rotation: f32,
}

impl tessera_ui::LayoutModifierNode for IconLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.size == other.size
            && self.width == other.width
            && self.height == other.height
            && intrinsic_dimensions(&self.content) == intrinsic_dimensions(&other.content)
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let (intrinsic_width, intrinsic_height) = intrinsic_dimensions(&self.content);
        let size_px = self.size.to_px();

        let preferred_width = self.width.unwrap_or(AxisConstraint::exact(size_px));
        let preferred_height = self.height.unwrap_or(AxisConstraint::exact(size_px));
        let width = input
            .parent_constraint()
            .width()
            .clamp(preferred_width.clamp(intrinsic_width));
        let height = input
            .parent_constraint()
            .height()
            .clamp(preferred_height.clamp(intrinsic_height));

        Ok(LayoutResult::new(ComputedData { width, height }))
    }
}

impl tessera_ui::DrawModifierNode for IconLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        let mut metadata = input.metadata_mut();
        match &self.content {
            Painter::Asset(_) => {}
            Painter::Vector(data) => {
                let command = ImageVectorCommand {
                    data: data.clone(),
                    tint: self.tint,
                    tint_mode: self.tint_mode,
                    rotation: self.rotation,
                };
                metadata.fragment_mut().push_draw_command(command);
            }
            Painter::Raster(data) => {
                let command = ImageCommand {
                    data: data.clone(),
                    opacity: 1.0,
                };
                metadata.fragment_mut().push_draw_command(command);
            }
        }
        content.draw(input);
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
/// - `content` - optional painter payload for vector or raster imagery.
/// - `size` - optional preferred square size.
/// - `width` / `height` - optional explicit layout dimensions.
/// - `tint` - optional tint override for vector icons.
/// - `tint_mode` - vector tint blending mode, defaulting to solid recoloring.
/// - `rotation` - clockwise degrees, inheriting control motion when omitted.
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
///     .painter(filled::STAR_SVG)
///     .size(tessera_ui::Dp(20.0))
///     .tint(Color::new(0.2, 0.5, 0.8, 1.0));
/// # }
/// ```
#[tessera]
pub fn icon(
    #[prop(skip_setter)] content: Option<Painter>,
    size: Option<Dp>,
    width: Option<AxisConstraint>,
    height: Option<AxisConstraint>,
    tint: Option<Color>,
    tint_mode: Option<TintMode>,
    rotation: Option<f32>,
) {
    let content = content
        .and_then(Painter::resolve)
        .unwrap_or_else(|| Painter::Raster((*placeholder_raster_data()).clone()));
    let default_size = use_context::<IconSize>()
        .map(|size| size.get().0)
        .unwrap_or(Dp(24.0));
    let size = size.unwrap_or(default_size);
    let tint = if let Some(tint) = tint {
        tint
    } else {
        let theme = use_context::<MaterialTheme>();
        use_context::<ContentColor>()
            .map(|c| c.get().current)
            .or_else(|| theme.map(|t| t.get().color_scheme.on_surface))
            .unwrap_or_else(|| ContentColor::default().current)
    };
    let tint_mode = tint_mode.unwrap_or(TintMode::Solid);
    let inherited_rotation = use_context::<IconRotation>()
        .map(|value| value.get().0)
        .unwrap_or(0.0);
    let rotation = rotation.unwrap_or(inherited_rotation);
    let policy = IconLayout {
        content,
        size,
        width,
        height,
        tint,
        tint_mode,
        rotation,
    };
    layout().policy(policy.clone()).draw(policy);
}

fn intrinsic_dimensions(content: &Painter) -> (Px, Px) {
    match content {
        Painter::Asset(_) => (Px::ZERO, Px::ZERO),
        Painter::Vector(data) => (
            clamp_f32_to_px(data.viewport_width),
            clamp_f32_to_px(data.viewport_height),
        ),
        Painter::Raster(data) => (clamp_u32_to_px(data.width), clamp_u32_to_px(data.height)),
    }
}

fn clamp_u32_to_px(value: u32) -> Px {
    Px::new(value.min(i32::MAX as u32) as i32)
}
