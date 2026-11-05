//! Text shaping and rendering with session-owned font resources.
//!
//! ## Usage
//!
//! Measure and render labels, paragraphs, and editable text.

use std::sync::Arc;

use glyphon::fontdb;
use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};
use tessera_ui::{
    Color, PxPosition,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use crate::theme::TextStyle;

use super::command::{TextCommand, TextConstraint};

struct TextResources {
    font_system: Arc<RwLock<glyphon::FontSystem>>,
}

impl Default for TextResources {
    fn default() -> Self {
        Self {
            font_system: Arc::new(init_font_system(glyphon::FontSystem::new())),
        }
    }
}

fn linear_to_srgb_channel(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

pub(crate) fn color_to_glyphon(color: Color) -> glyphon::Color {
    let r = (linear_to_srgb_channel(color.r) * 255.0 + 0.5) as u8;
    let g = (linear_to_srgb_channel(color.g) * 255.0 + 0.5) as u8;
    let b = (linear_to_srgb_channel(color.b) * 255.0 + 0.5) as u8;
    let a = (color.a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    glyphon::Color::rgba(r, g, b, a)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TextFont {
    weight: u16,
    tracking: f32,
    family: Option<&'static str>,
}

fn init_font_system(mut fonts: glyphon::FontSystem) -> RwLock<glyphon::FontSystem> {
    #[cfg(target_os = "android")]
    fonts.db_mut().load_fonts_dir("/system/fonts");
    let tessera_ui::LoadState::Ready(bytes) = crate::res::ROBOTO_REGULAR_TTF.load().get() else {
        panic!("the default font must be embedded");
    };
    let existing = fonts
        .db()
        .faces()
        .filter(|face| face.families.iter().any(|(name, _)| name == "Roboto"))
        .map(|face| face.id)
        .collect::<Vec<_>>();
    for id in existing {
        fonts.db_mut().remove_face(id);
    }
    fonts
        .db_mut()
        .load_font_source(fontdb::Source::Binary(Arc::new(bytes)));
    fonts.db_mut().set_sans_serif_family("Roboto");
    #[cfg(target_family = "wasm")]
    {
        fonts.db_mut().set_serif_family("Noto Sans SC");
        fonts.db_mut().set_monospace_family("Noto Sans SC");
        fonts.db_mut().set_cursive_family("Noto Sans SC");
        fonts.db_mut().set_fantasy_family("Noto Sans SC");
    }
    RwLock::new(fonts)
}

/// Borrows the font system owned by the active runtime session.
pub fn read_font_system() -> ArcRwLockReadGuard<RawRwLock, glyphon::FontSystem> {
    tessera_ui::__private::runtime_resource::<TextResources>()
        .font_system
        .read_arc()
}

/// Mutably borrows the font system owned by the active runtime session.
pub fn write_font_system() -> ArcRwLockWriteGuard<RawRwLock, glyphon::FontSystem> {
    tessera_ui::__private::runtime_resource::<TextResources>()
        .font_system
        .write_arc()
}

/// A text renderer
/// Pipeline for rendering text using the Glyphon engine.
///
/// This struct manages font atlas, cache, viewport, and swash cache for
/// efficient text rendering.
pub struct GlyphonTextRender {
    /// Glyphon font atlas, a heavy-weight, shared resource.
    atlas: glyphon::TextAtlas,
    /// Glyphon cache, a heavy-weight, shared resource.
    cache: glyphon::Cache,
    /// Glyphon swash cache, a CPU-side cache for glyph rasterization.
    swash_cache: glyphon::SwashCache,
    /// Multisample state for anti-aliasing.
    msaa: wgpu::MultisampleState,
    /// Glyphon text renderer, responsible for rendering text.
    renderers: Vec<TextBatchResources>,
    next_renderer: usize,
}

struct TextBatchResources {
    renderer: glyphon::TextRenderer,
    viewport: glyphon::Viewport,
}

impl GlyphonTextRender {
    /// Creates a new text renderer pipeline.
    ///
    /// # Parameters
    ///
    /// - `gpu`: The wgpu device.
    /// - `queue`: The wgpu queue.
    /// - `config`: Surface configuration.
    /// - `sample_count`: Multisample count for anti-aliasing.
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let cache = glyphon::Cache::new(gpu);
        let mut atlas = glyphon::TextAtlas::new(gpu, queue, &cache, config.format);
        let viewport = glyphon::Viewport::new(gpu, &cache);
        let swash_cache = glyphon::SwashCache::new();
        let msaa = wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, gpu, msaa, tessera_ui::ui_stencil_state());

        Self {
            atlas,
            cache,
            swash_cache,
            msaa,
            renderers: vec![TextBatchResources { renderer, viewport }],
            next_renderer: 0,
        }
    }
}

impl DrawablePipeline<TextCommand> for GlyphonTextRender {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.next_renderer = 0;
    }

    fn draw(&mut self, context: &mut DrawContext<TextCommand>) {
        if context.commands.is_empty() {
            return;
        }

        let text_areas = context.commands.iter().map(|(command, geometry)| {
            let origin = geometry.local_to_window.map_point(command.offset);
            command
                .data
                .text_area(origin, geometry.local_to_window.scale())
        });

        // Batches in one submission need distinct buffers because queued
        // uploads precede all of its draw commands. Reuse them across frames.
        if self.next_renderer == self.renderers.len() {
            self.renderers.push(TextBatchResources {
                renderer: glyphon::TextRenderer::new(
                    &mut self.atlas,
                    context.device,
                    self.msaa,
                    tessera_ui::ui_stencil_state(),
                ),
                viewport: glyphon::Viewport::new(context.device, &self.cache),
            });
        }
        let resources = &mut self.renderers[self.next_renderer];
        self.next_renderer += 1;
        resources.viewport.update(
            context.queue,
            glyphon::Resolution {
                width: context.target_size.width.positive(),
                height: context.target_size.height.positive(),
            },
        );
        let renderer = &mut resources.renderer;
        let mut font_system = write_font_system();
        renderer
            .prepare(
                context.device,
                context.queue,
                &mut font_system,
                &mut self.atlas,
                &resources.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .expect("glyphon prepare failed");

        renderer
            .render(&self.atlas, &resources.viewport, context.render_pass)
            .expect("glyphon render failed");
    }
}

/// Text data for rendering, including buffer and size.
#[derive(Debug, Clone)]
pub struct TextData {
    /// glyphon text buffer
    text_buffer: Arc<glyphon::Buffer>,
    /// text area size
    pub size: [u32; 2],
    /// Baseline offset of the first visible line, relative to the text origin.
    pub first_baseline: f32,
    /// Baseline offset of the last visible line, relative to the text origin.
    pub last_baseline: f32,
    /// Number of visible layout lines.
    pub line_count: u32,
    base_color: Color,
    current_color: Color,
    text: String,
    font_size: f32,
    line_height: f32,
    font: TextFont,
    styled_identity: Option<Arc<Vec<StyledLineIdentity>>>,
    base_span_colors: Option<Arc<Vec<Option<glyphon::Color>>>>,
    span_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct StyledLineIdentity {
    text: String,
    attrs: glyphon::AttrsList,
    alignment: Option<glyphon::cosmic_text::Align>,
    layout: Vec<glyphon::LayoutLine>,
}

/// Measurement result returned by `TextData::measure()`.
#[derive(Debug, Clone, PartialEq, Copy)]
#[allow(dead_code)]
pub struct TextMeasureInfo {
    /// The computed size of the text area.
    pub size: [u32; 2],
    /// Baseline offset of the first visible line.
    pub first_baseline: f32,
    /// Baseline offset of the last visible line.
    pub last_baseline: f32,
    /// Number of visible layout lines.
    pub line_count: u32,
}

impl PartialEq for TextData {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
            && self.first_baseline == other.first_baseline
            && self.last_baseline == other.last_baseline
            && self.line_count == other.line_count
            && self.base_color == other.base_color
            && self.current_color == other.current_color
            && self.text == other.text
            && self.font_size == other.font_size
            && self.line_height == other.line_height
            && self.font == other.font
            && self.styled_identity == other.styled_identity
            && self.span_opacity == other.span_opacity
            && self.text_buffer.wrap() == other.text_buffer.wrap()
            && self.text_buffer.ellipsize() == other.text_buffer.ellipsize()
            && self.text_buffer.scroll() == other.text_buffer.scroll()
    }
}

impl TextData {
    /// Shapes text using the active session's font resources.
    pub(crate) fn shape(
        text: String,
        color: Color,
        style: TextStyle,
        constraint: TextConstraint,
    ) -> Self {
        crate::fonts::prepare(&text, style);
        let font_size = style.font_size.to_pixels_f32();
        let line_height = style.line_height.unwrap_or(style.font_size).to_pixels_f32();
        let font = TextFont {
            weight: style.font_weight.clamp(1, 1000),
            tracking: if font_size > 0.0 {
                style.letter_spacing.to_pixels_f32() / font_size
            } else {
                0.0
            },
            family: style.font_family,
        };
        let (text_buffer, bounds, first_baseline, last_baseline, line_count) =
            Self::build_buffer(&text, font_size, line_height, font, &constraint);
        let data = Self {
            text_buffer: Arc::new(text_buffer),
            size: bounds,
            first_baseline,
            last_baseline,
            line_count,
            base_color: color,
            current_color: color,
            text,
            font_size,
            line_height,
            font,
            styled_identity: None,
            base_span_colors: None,
            span_opacity: 1.0,
        };
        data
    }

    /// Builds [`TextData`] directly from a pre-shaped glyphon buffer.
    pub fn from_buffer(text_buffer: glyphon::Buffer) -> Self {
        let styled_identity = text_buffer
            .lines
            .iter()
            .map(|line| StyledLineIdentity {
                text: line.text().to_owned(),
                attrs: line.attrs_list().clone(),
                alignment: line.align(),
                layout: line.layout_opt().cloned().unwrap_or_default(),
            })
            .collect();
        let base_span_colors: Vec<_> = text_buffer
            .lines
            .iter()
            .flat_map(|line| line.layout_opt().into_iter().flatten())
            .flat_map(|line| line.glyphs.iter().map(|glyph| glyph.color_opt))
            .collect();
        let metrics = text_buffer.metrics();
        // Calculate text bounds
        let mut run_width: f32 = 0.0;
        let mut first_baseline = 0.0;
        let mut last_baseline = 0.0;
        let mut line_count: u32 = 0;
        for run in text_buffer.layout_runs() {
            // Take the max. width of all lines.
            run_width = run_width.max(run.line_w);
            if line_count == 0 {
                first_baseline = run.line_y;
            }
            last_baseline = run.line_y;
            line_count += 1;
        }
        if text_buffer.lines.iter().any(|line| {
            matches!(
                line.align(),
                Some(
                    glyphon::cosmic_text::Align::Center
                        | glyphon::cosmic_text::Align::Right
                        | glyphon::cosmic_text::Align::End
                )
            )
        }) {
            run_width = text_buffer.size().0.unwrap_or(run_width);
        }
        let total_height = line_count as f32 * metrics.line_height;
        // build text data
        Self {
            text_buffer: Arc::new(text_buffer),
            size: [run_width as u32, total_height.ceil() as u32],
            first_baseline,
            last_baseline,
            line_count,
            base_color: Color::WHITE,
            current_color: Color::WHITE,
            text: String::new(),
            font_size: metrics.font_size,
            line_height: metrics.line_height,
            font: TextFont {
                weight: 400,
                tracking: 0.0,
                family: None,
            },
            styled_identity: Some(Arc::new(styled_identity)),
            base_span_colors: base_span_colors
                .iter()
                .any(Option::is_some)
                .then(|| Arc::new(base_span_colors)),
            span_opacity: 1.0,
        }
    }

    /// Get the glyphon text area from the text data
    fn text_area(&'_ self, start_pos: PxPosition, scale: f32) -> glyphon::TextArea<'_> {
        let bounds = glyphon::TextBounds {
            left: start_pos.x.floor() as i32,
            top: start_pos.y.floor() as i32,
            right: (start_pos.x + self.size[0] as f32 * scale).ceil() as i32,
            bottom: (start_pos.y + self.size[1] as f32 * scale).ceil() as i32,
        };
        glyphon::TextArea {
            buffer: &self.text_buffer,
            left: start_pos.x - self.text_buffer.scroll().horizontal * scale,
            top: start_pos.y,
            scale,
            bounds,
            default_color: color_to_glyphon(self.current_color),
            custom_glyphs: &[],
        }
    }

    pub(crate) fn set_scroll(&mut self, scroll: glyphon::cosmic_text::Scroll) {
        if self.text_buffer.scroll() == scroll {
            return;
        }
        let buffer = Arc::make_mut(&mut self.text_buffer);
        buffer.set_scroll(scroll);
        buffer.shape_until_scroll(&mut write_font_system(), false);
        let mut width = 0.0_f32;
        self.first_baseline = 0.0;
        self.last_baseline = 0.0;
        self.line_count = 0;
        for run in self.text_buffer.layout_runs() {
            width = width.max(run.line_w);
            if self.line_count == 0 {
                self.first_baseline = run.line_y;
            }
            self.last_baseline = run.line_y;
            self.line_count += 1;
        }
        if self.text_buffer.lines.iter().any(|line| {
            matches!(
                line.align(),
                Some(
                    glyphon::cosmic_text::Align::Center
                        | glyphon::cosmic_text::Align::Right
                        | glyphon::cosmic_text::Align::End
                )
            )
        }) {
            width = self.text_buffer.size().0.unwrap_or(width);
        }
        self.size = [
            width as u32,
            (self.line_count as f32 * self.text_buffer.metrics().line_height).ceil() as u32,
        ];
    }

    fn build_buffer(
        text: &str,
        size: f32,
        line_height: f32,
        font: TextFont,
        constraint: &TextConstraint,
    ) -> (glyphon::Buffer, [u32; 2], f32, f32, u32) {
        let mut text_buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size, line_height),
        );
        if constraint.single_line {
            text_buffer.set_wrap(glyphon::Wrap::None);
            text_buffer.set_ellipsize(glyphon::cosmic_text::Ellipsize::End(
                glyphon::cosmic_text::EllipsizeHeightLimit::Lines(1),
            ));
        } else {
            text_buffer.set_wrap(glyphon::Wrap::WordOrGlyph);
        }
        let single_line = constraint
            .single_line
            .then(|| text.replace(['\r', '\n'], " "));
        text_buffer.set_size(constraint.max_width, constraint.max_height);
        text_buffer.set_text(
            single_line.as_deref().unwrap_or(text),
            &glyphon::Attrs::new()
                .family(
                    font.family
                        .map(fontdb::Family::Name)
                        .unwrap_or(fontdb::Family::SansSerif),
                )
                .weight(glyphon::Weight(font.weight))
                .letter_spacing(font.tracking),
            glyphon::Shaping::Advanced,
            None,
        );
        text_buffer.shape_until_scroll(&mut write_font_system(), false);
        // Calculate text bounds and baselines.
        let mut run_width: f32 = 0.0;
        let metrics = text_buffer.metrics();
        let mut first_baseline = 0.0;
        let mut last_baseline = 0.0;
        let mut line_count: u32 = 0;
        for run in text_buffer.layout_runs() {
            run_width = run_width.max(run.line_w);
            if line_count == 0 {
                first_baseline = run.line_y;
            }
            last_baseline = run.line_y;
            line_count += 1;
        }
        let total_height = line_count as f32 * metrics.line_height;
        (
            text_buffer,
            [run_width.ceil() as u32, total_height.ceil() as u32],
            first_baseline,
            last_baseline,
            line_count,
        )
    }

    pub(crate) fn set_color(&mut self, color: Color) {
        self.base_color = color;
        self.current_color = color;
    }

    pub(crate) fn measure_info(&self) -> TextMeasureInfo {
        TextMeasureInfo {
            size: self.size,
            first_baseline: self.first_baseline,
            last_baseline: self.last_baseline,
            line_count: self.line_count,
        }
    }

    pub(crate) fn apply_opacity(&mut self, opacity: f32) {
        let opacity = opacity.clamp(0.0, 1.0);
        let target_alpha = (self.base_color.a * opacity).clamp(0.0, 1.0);
        let target_color = self.base_color.with_alpha(target_alpha);
        if (target_color.a - self.current_color.a).abs() <= f32::EPSILON
            && (target_color.r - self.current_color.r).abs() <= f32::EPSILON
            && (target_color.g - self.current_color.g).abs() <= f32::EPSILON
            && (target_color.b - self.current_color.b).abs() <= f32::EPSILON
            && self.span_opacity == opacity
        {
            return;
        }

        self.current_color = target_color;
        if let Some(colors) = &self.base_span_colors {
            let mut colors = colors.iter();
            for line in &mut Arc::make_mut(&mut self.text_buffer).lines {
                line.map_layout_glyph_colors(|_| {
                    colors.next().copied().flatten().map(|color| {
                        glyphon::Color::rgba(
                            color.r(),
                            color.g(),
                            color.b(),
                            (f32::from(color.a()) * opacity).round() as u8,
                        )
                    })
                });
            }
        }
        self.span_opacity = opacity;
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{Color, Dp};

    use crate::theme::MaterialTypography;

    use super::{TextConstraint, TextData, read_font_system};

    #[test]
    fn retained_text_copies_share_shapes_and_isolate_scroll_and_opacity() {
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let shaped = TextData::shape(
            "Shared shaped glyphs".into(),
            Color::BLACK,
            MaterialTypography::default().body_large,
            TextConstraint::default(),
        );
        let origin = tessera_ui::PxPosition::from_f32_xy(20.0, 30.0);
        let mut buffer = (*shaped.text_area(origin, 1.0).buffer).clone();
        for line in &mut buffer.lines {
            line.map_layout_glyph_colors(|_| Some(glyphon::Color::rgba(120, 80, 40, 255)));
        }
        let original = TextData::from_buffer(buffer);
        let mut faded = original.clone();
        assert!(
            std::ptr::eq(
                original.text_area(origin, 1.0).buffer,
                faded.text_area(origin, 1.0).buffer
            ),
            "retained command copies must share their shaped glyph storage"
        );
        let mut scrolled = original.clone();
        let mut scroll = scrolled.text_area(origin, 1.0).buffer.scroll();
        scroll.horizontal = 13.5;
        scrolled.set_scroll(scroll);
        assert_eq!(scrolled.text_area(origin, 1.0).left, 6.5);
        assert_eq!(original.text_area(origin, 1.0).left, 20.0);
        faded.apply_opacity(0.5);
        let alpha = |data: &TextData| {
            data.text_area(origin, 1.0)
                .buffer
                .layout_runs()
                .next()
                .unwrap()
                .glyphs[0]
                .color_opt
                .unwrap()
                .a()
        };
        assert_eq!(alpha(&faded), 128);
        assert_eq!(alpha(&original), 255);
        faded.apply_opacity(0.25);
        assert_eq!(alpha(&faded), 64);
        assert_eq!(alpha(&original), 255);
        assert_eq!(alpha(&scrolled), 255);
    }

    #[test]
    fn horizontal_buffer_scroll_moves_glyph_origin_without_moving_clip_bounds() {
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let shaped = TextData::shape(
            "Scrollable editor text".into(),
            Color::BLACK,
            MaterialTypography::default().body_large,
            TextConstraint {
                max_width: Some(60.0),
                max_height: Some(24.0),
                ..Default::default()
            },
        );
        let before = TextData::from_buffer(shaped.text_buffer.as_ref().clone());
        let mut buffer = shaped.text_buffer.as_ref().clone();
        let mut scroll = buffer.scroll();
        scroll.horizontal = 13.5;
        buffer.set_scroll(scroll);
        let after = TextData::from_buffer(buffer);
        let origin = tessera_ui::PxPosition::from_f32_xy(80.25, 50.5);
        let original_area = before.text_area(origin, 1.5);
        let scrolled_area = after.text_area(origin, 1.5);
        assert_eq!(scrolled_area.left, 60.0);
        assert_eq!(scrolled_area.top, 50.5);
        assert_eq!(scrolled_area.bounds, original_area.bounds);
        assert_ne!(
            before, after,
            "scroll changes must invalidate retained text draws"
        );
    }

    #[test]
    fn wrapping_keeps_words_intact_and_falls_back_for_words_wider_than_the_line() {
        let session = tessera_ui::RuntimeSession::new();
        let _session = session.enter();
        let style = MaterialTypography::default().body_medium;
        let alpha = TextData::shape(
            "alpha ".to_string(),
            Color::BLACK,
            style,
            TextConstraint::default(),
        );
        let beta = TextData::shape(
            "beta".to_string(),
            Color::BLACK,
            style,
            TextConstraint::default(),
        );
        let width = alpha.size[0] as f32 + beta.size[0] as f32 / 2.0;
        let phrase = TextData::shape(
            "alpha beta".to_string(),
            Color::BLACK,
            style,
            TextConstraint {
                max_width: Some(width),
                max_height: None,
                ..Default::default()
            },
        );
        let runs: Vec<_> = phrase.text_buffer.layout_runs().collect();
        assert_eq!(runs.len(), 2);
        assert!(runs[0].glyphs.iter().all(|glyph| glyph.end <= 6));
        assert_eq!(runs[1].glyphs[0].start, 6);
        let long = TextData::shape(
            "supercalifragilisticexpialidocious".to_string(),
            Color::BLACK,
            style,
            TextConstraint {
                max_width: Some(width),
                max_height: None,
                ..Default::default()
            },
        );
        let runs: Vec<_> = long.text_buffer.layout_runs().collect();
        assert!(runs.len() > 1);
        assert!(runs.iter().all(|run| run.line_w <= width + 1.0));
    }

    #[test]
    fn bundled_fonts_shape_latin_weights_and_chinese_without_system_fonts() {
        let session = tessera_ui::RuntimeSession::new();
        session.run(|| {
            let db = glyphon::fontdb::Database::new();
            *super::write_font_system() = super::init_font_system(
                glyphon::FontSystem::new_with_locale_and_db("en-US".into(), db),
            )
            .into_inner();
            assert!(
                read_font_system()
                    .db()
                    .faces()
                    .any(|face| face.families.iter().any(|(name, _)| name == "Roboto"))
            );
            let typography = MaterialTypography::default();
            let shape = |content: &str, style| {
                TextData::shape(
                    content.to_owned(),
                    Color::BLACK,
                    style,
                    TextConstraint::default(),
                )
            };
            let regular = shape("Material typography", typography.body_large);
            let medium = shape(
                "Material typography",
                crate::theme::TextStyle {
                    font_weight: 500,
                    ..typography.body_large
                },
            );
            let untracked = shape(
                "Material typography",
                crate::theme::TextStyle {
                    letter_spacing: Dp(0.0),
                    ..typography.body_large
                },
            );
            assert_eq!(regular.size[1], Dp(24.0).to_pixels_f32().ceil() as u32);
            assert!(
                regular.first_baseline > 0.0 && regular.first_baseline < regular.size[1] as f32
            );
            assert!(regular.size[0] > untracked.size[0]);
            let chinese = shape("中文", typography.body_large);
            let named = shape(
                "中文",
                crate::theme::TextStyle {
                    font_family: Some("Noto Sans SC"),
                    ..typography.body_large
                },
            );
            let font_system = read_font_system();
            let chinese_faces = font_system
                .db()
                .faces()
                .filter(|face| face.families.iter().any(|(name, _)| name == "Noto Sans SC"))
                .count();
            assert_eq!(
                chinese_faces, 2,
                "only the two requested Unicode blocks should load"
            );
            let glyphs: Vec<_> = [&chinese, &named]
                .into_iter()
                .flat_map(|data| data.text_buffer.layout_runs().flat_map(|run| run.glyphs))
                .collect();
            assert_eq!(glyphs.len(), 4);
            for glyph in glyphs {
                assert_ne!(glyph.glyph_id, 0);
                assert!(
                    font_system
                        .db()
                        .face(glyph.font_id)
                        .unwrap()
                        .families
                        .iter()
                        .any(|(name, _)| name == "Noto Sans SC")
                );
            }
            for (data, weight) in [(&regular, 400), (&medium, 500)] {
                let glyph = &data.text_buffer.layout_runs().next().unwrap().glyphs[0];
                let face = font_system.db().face(glyph.font_id).unwrap();
                assert_eq!(face.weight.0, weight);
                assert!(face.families.iter().any(|(family, _)| family == "Roboto"));
            }
            drop(font_system);
            let standard = shape("、", typography.body_large);
            let variant = shape("、\u{fe01}", typography.body_large);
            let first =
                |data: &TextData| data.text_buffer.layout_runs().next().unwrap().glyphs[0].glyph_id;
            assert_ne!(
                first(&standard),
                first(&variant),
                "CJK variation sequences must retain their glyphs"
            );
        });
    }
}
