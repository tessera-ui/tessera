//! Vector image component built on top of SVG parsing and tessellation.
//!
//! This module mirrors the ergonomics of [`crate::image`], but keeps the content
//! in vector form so it can scale cleanly at any size. SVG data is parsed with
//! [`usvg`] and tessellated into GPU-friendly triangles using [`lyon`].
//! The resulting [`ImageVectorData`] can be cached and reused across frames.

use std::{fs, path::Path as StdPath, sync::Arc};

use derive_builder::Builder;
use lyon_geom::point;
use lyon_path::Path as LyonPath;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule as LyonFillRule, FillTessellator, FillVertex,
    LineCap as LyonLineCap, LineJoin as LyonLineJoin, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use tessera_ui::{Color, ComputedData, Constraint, DimensionValue, Px, tessera};
use thiserror::Error;
use usvg::{
    BlendMode, FillRule, Group, LineCap as SvgLineCap, LineJoin as SvgLineJoin, Node, Paint,
    PaintOrder, Path, Stroke, Tree, tiny_skia_path::PathSegment,
};

use crate::pipelines::image_vector::{ImageVectorCommand, ImageVectorVertex};

pub use crate::pipelines::image_vector::ImageVectorData;

/// Source for loading SVG vector data.
#[derive(Clone, Debug)]
pub enum ImageVectorSource {
    /// Load from a filesystem path.
    Path(String),
    /// Load from in-memory bytes.
    Bytes(Arc<[u8]>),
}

/// Errors that can occur while decoding or tessellating vector images.
#[derive(Debug, Error)]
pub enum ImageVectorLoadError {
    /// Failed to read a file from disk.
    #[error("failed to read SVG from {path}: {source}")]
    Io {
        /// Failing path.
        path: String,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// SVG parsing failed.
    #[error("failed to parse SVG: {0}")]
    Parse(#[from] usvg::Error),
    /// The SVG viewport dimensions are invalid.
    #[error("SVG viewport must have finite, positive size")]
    InvalidViewport,
    /// Encountered an SVG feature that isn't supported yet.
    #[error("unsupported SVG feature: {0}")]
    UnsupportedFeature(String),
    /// Failed to apply the absolute transform for a path.
    #[error("failed to apply SVG transforms")]
    TransformFailed,
    /// Tessellation of the path geometry failed.
    #[error("tessellation error: {0}")]
    Tessellation(#[from] lyon_tessellation::TessellationError),
    /// No renderable geometry was produced.
    #[error("SVG produced no renderable paths")]
    EmptyGeometry,
}

/// Load [`ImageVectorData`] from the provided source.
pub fn load_image_vector_from_source(
    source: &ImageVectorSource,
) -> Result<ImageVectorData, ImageVectorLoadError> {
    let (bytes, resources_dir) = read_source_bytes(source)?;

    let mut options = usvg::Options::default();
    options.resources_dir = resources_dir;
    let tree = Tree::from_data(&bytes, &options)?;

    build_vector_data(&tree)
}

/// Arguments for [`image_vector`].
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ImageVectorArgs {
    /// Vector geometry to render.
    #[builder(setter(into))]
    pub data: Arc<ImageVectorData>,
    /// Desired width, defaults to wrapping at intrinsic size.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,
    /// Desired height, defaults to wrapping at intrinsic size.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
    /// Optional tint applied multiplicatively to the SVG colors.
    #[builder(default = "Color::WHITE")]
    pub tint: Color,
}

impl From<ImageVectorData> for ImageVectorArgs {
    fn from(data: ImageVectorData) -> Self {
        ImageVectorArgsBuilder::default()
            .data(Arc::new(data))
            .build()
            .expect("ImageVectorArgsBuilder failed with required fields set")
    }
}

#[tessera]
pub fn image_vector(args: impl Into<ImageVectorArgs>) {
    let image_args: ImageVectorArgs = args.into();

    measure(Box::new(move |input| {
        let intrinsic_width = px_from_f32(image_args.data.viewport_width);
        let intrinsic_height = px_from_f32(image_args.data.viewport_height);

        let constraint = Constraint::new(image_args.width, image_args.height);
        let effective_constraint = constraint.merge(input.parent_constraint);

        let width = match effective_constraint.width {
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

        let height = match effective_constraint.height {
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

        let command = ImageVectorCommand {
            data: image_args.data.clone(),
            tint: image_args.tint,
        };

        input
            .metadatas
            .entry(input.current_node_id)
            .or_default()
            .push_draw_command(command);

        Ok(ComputedData { width, height })
    }));
}

fn px_from_f32(value: f32) -> Px {
    let clamped = value.max(0.0).min(i32::MAX as f32);
    Px(clamped.round() as i32)
}

fn read_source_bytes(
    source: &ImageVectorSource,
) -> Result<(Vec<u8>, Option<std::path::PathBuf>), ImageVectorLoadError> {
    match source {
        ImageVectorSource::Path(path) => {
            let bytes = fs::read(path).map_err(|source| ImageVectorLoadError::Io {
                path: path.clone(),
                source,
            })?;
            let dir = StdPath::new(path).parent().map(|p| p.to_path_buf());
            Ok((bytes, dir))
        }
        ImageVectorSource::Bytes(bytes) => Ok((bytes.as_ref().to_vec(), None)),
    }
}

fn build_vector_data(tree: &Tree) -> Result<ImageVectorData, ImageVectorLoadError> {
    let size = tree.size();
    let viewport_width = size.width();
    let viewport_height = size.height();

    if !viewport_width.is_finite()
        || !viewport_height.is_finite()
        || viewport_width <= 0.0
        || viewport_height <= 0.0
    {
        return Err(ImageVectorLoadError::InvalidViewport);
    }

    let mut collector = VectorGeometryCollector::new(viewport_width, viewport_height);
    visit_group(tree.root(), 1.0, &mut collector)?;

    collector.finish()
}

fn visit_group(
    group: &Group,
    inherited_opacity: f32,
    collector: &mut VectorGeometryCollector,
) -> Result<(), ImageVectorLoadError> {
    if group.clip_path().is_some() || group.mask().is_some() || !group.filters().is_empty() {
        return Err(ImageVectorLoadError::UnsupportedFeature(
            "clip paths, masks, and filters are not supported".to_string(),
        ));
    }

    if group.blend_mode() != BlendMode::Normal {
        return Err(ImageVectorLoadError::UnsupportedFeature(
            "non-normal blend modes".to_string(),
        ));
    }

    let accumulated_opacity = inherited_opacity * group.opacity().get();

    for node in group.children() {
        match node {
            Node::Group(child) => visit_group(child, accumulated_opacity, collector)?,
            Node::Path(path) => collector.process_path(path, accumulated_opacity)?,
            Node::Image(_) | Node::Text(_) => {
                return Err(ImageVectorLoadError::UnsupportedFeature(
                    "non-path nodes in SVG are not supported".to_string(),
                ));
            }
        }
    }

    Ok(())
}

struct VectorGeometryCollector {
    viewport_width: f32,
    viewport_height: f32,
    buffers: VertexBuffers<ImageVectorVertex, u32>,
}

impl VectorGeometryCollector {
    fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            buffers: VertexBuffers::new(),
        }
    }

    fn process_path(
        &mut self,
        path: &Path,
        inherited_opacity: f32,
    ) -> Result<(), ImageVectorLoadError> {
        if !path.is_visible() {
            return Ok(());
        }

        if path.rendering_mode() != usvg::ShapeRendering::default() {
            return Err(ImageVectorLoadError::UnsupportedFeature(
                "shape-rendering modes are not supported".to_string(),
            ));
        }

        let lyon_path = convert_to_lyon_path(path)?;

        match path.paint_order() {
            PaintOrder::FillAndStroke => {
                if let Some(fill) = path.fill() {
                    self.tessellate_fill(&lyon_path, fill, inherited_opacity)?;
                }
                if let Some(stroke) = path.stroke() {
                    self.tessellate_stroke(&lyon_path, stroke, inherited_opacity)?;
                }
            }
            PaintOrder::StrokeAndFill => {
                if let Some(stroke) = path.stroke() {
                    self.tessellate_stroke(&lyon_path, stroke, inherited_opacity)?;
                }
                if let Some(fill) = path.fill() {
                    self.tessellate_fill(&lyon_path, fill, inherited_opacity)?;
                }
            }
        }

        Ok(())
    }

    fn tessellate_fill(
        &mut self,
        path: &LyonPath,
        fill: &usvg::Fill,
        inherited_opacity: f32,
    ) -> Result<(), ImageVectorLoadError> {
        let color = color_from_paint(fill.paint(), fill.opacity().get(), inherited_opacity)?;
        let fill_rule = match fill.rule() {
            FillRule::EvenOdd => LyonFillRule::EvenOdd,
            FillRule::NonZero => LyonFillRule::NonZero,
        };

        let options = FillOptions::default().with_fill_rule(fill_rule);
        let viewport = [self.viewport_width, self.viewport_height];

        FillTessellator::new().tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(&mut self.buffers, |vertex: FillVertex| {
                ImageVectorVertex::new(vertex.position().to_array(), color, viewport)
            }),
        )?;

        Ok(())
    }

    fn tessellate_stroke(
        &mut self,
        path: &LyonPath,
        stroke: &Stroke,
        inherited_opacity: f32,
    ) -> Result<(), ImageVectorLoadError> {
        if stroke.dasharray().is_some() {
            return Err(ImageVectorLoadError::UnsupportedFeature(
                "stroke dash arrays".to_string(),
            ));
        }

        let color = color_from_paint(stroke.paint(), stroke.opacity().get(), inherited_opacity)?;

        let mut options = StrokeOptions::default()
            .with_line_width(stroke.width().get())
            .with_line_cap(map_line_cap(stroke.linecap()))
            .with_line_join(map_line_join(stroke.linejoin()));

        options.miter_limit = stroke.miterlimit().get();

        let viewport = [self.viewport_width, self.viewport_height];

        StrokeTessellator::new().tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(&mut self.buffers, |vertex: StrokeVertex| {
                ImageVectorVertex::new(vertex.position().to_array(), color, viewport)
            }),
        )?;

        Ok(())
    }

    fn finish(self) -> Result<ImageVectorData, ImageVectorLoadError> {
        if self.buffers.vertices.is_empty() || self.buffers.indices.is_empty() {
            return Err(ImageVectorLoadError::EmptyGeometry);
        }

        Ok(ImageVectorData::new(
            self.viewport_width,
            self.viewport_height,
            Arc::new(self.buffers.vertices),
            Arc::new(self.buffers.indices),
        ))
    }
}

fn color_from_paint(
    paint: &Paint,
    paint_opacity: f32,
    inherited_opacity: f32,
) -> Result<Color, ImageVectorLoadError> {
    let opacity = (paint_opacity * inherited_opacity).clamp(0.0, 1.0);
    match paint {
        Paint::Color(color) => Ok(Color::new(
            f32::from(color.red) / 255.0,
            f32::from(color.green) / 255.0,
            f32::from(color.blue) / 255.0,
            opacity,
        )),
        _ => Err(ImageVectorLoadError::UnsupportedFeature(
            "only solid color fills and strokes are supported".to_string(),
        )),
    }
}

fn convert_to_lyon_path(path: &Path) -> Result<LyonPath, ImageVectorLoadError> {
    let transformed = path
        .data()
        .clone()
        .transform(path.abs_transform())
        .ok_or(ImageVectorLoadError::TransformFailed)?;

    let mut builder = LyonPath::builder().with_svg();
    for segment in transformed.segments() {
        match segment {
            PathSegment::MoveTo(p0) => {
                builder.move_to(point(p0.x, p0.y));
            }
            PathSegment::LineTo(p0) => {
                builder.line_to(point(p0.x, p0.y));
            }
            PathSegment::QuadTo(p0, p1) => {
                builder.quadratic_bezier_to(point(p0.x, p0.y), point(p1.x, p1.y));
            }
            PathSegment::CubicTo(p0, p1, p2) => {
                builder.cubic_bezier_to(point(p0.x, p0.y), point(p1.x, p1.y), point(p2.x, p2.y));
            }
            PathSegment::Close => {
                builder.close();
            }
        }
    }

    Ok(builder.build())
}

fn map_line_cap(cap: SvgLineCap) -> LyonLineCap {
    match cap {
        SvgLineCap::Butt => LyonLineCap::Butt,
        SvgLineCap::Round => LyonLineCap::Round,
        SvgLineCap::Square => LyonLineCap::Square,
    }
}

fn map_line_join(join: SvgLineJoin) -> LyonLineJoin {
    match join {
        SvgLineJoin::Miter | SvgLineJoin::MiterClip => LyonLineJoin::Miter,
        SvgLineJoin::Round => LyonLineJoin::Round,
        SvgLineJoin::Bevel => LyonLineJoin::Bevel,
    }
}

impl ImageVectorVertex {
    fn new(position: [f32; 2], color: Color, viewport: [f32; 2]) -> Self {
        ImageVectorVertex {
            position: [position[0] / viewport[0], position[1] / viewport[1]],
            color,
        }
    }
}
