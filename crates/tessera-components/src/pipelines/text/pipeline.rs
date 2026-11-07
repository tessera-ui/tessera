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

#[cfg(not(target_family = "wasm"))]
static PRELOADED_FONTS: std::sync::Mutex<Option<std::thread::JoinHandle<glyphon::FontSystem>>> =
    std::sync::Mutex::new(None);

pub(crate) fn preload_system_fonts() {
    #[cfg(not(target_family = "wasm"))]
    {
        let mut preloaded = PRELOADED_FONTS.lock().expect("font preload lock");
        if preloaded.is_none() {
            *preloaded = std::thread::Builder::new()
                .name("tessera-fonts".into())
                .spawn(|| {
                    let mut fonts = system_fonts();
                    // Matching examines installed variable-font axes. Prepare common UI
                    // weights while the renderer initializes instead of on first layout.
                    for weight in [
                        fontdb::Weight::NORMAL,
                        fontdb::Weight::MEDIUM,
                        fontdb::Weight::BOLD,
                    ] {
                        fonts.get_font_matches(&glyphon::Attrs::new().weight(weight));
                    }
                    fonts
                })
                .ok();
        }
    }
}

fn system_fonts() -> glyphon::FontSystem {
    use glyphon::cosmic_text::{Fallback, PlatformFallback};

    let mut fonts = glyphon::FontSystem::new();
    let db = fonts.db_mut();
    let fallback = PlatformFallback.common_fallback().first().copied();
    #[cfg(target_os = "android")]
    db.load_fonts_dir("/system/fonts");
    #[cfg(target_os = "android")]
    let fallback = fallback.or(Some("Roboto"));
    #[cfg(target_os = "ios")]
    let fallback = [".SF UI", ".SF NS", "Helvetica Neue", "Helvetica", "Arial"]
        .into_iter()
        .find(|name| {
            db.query(&fontdb::Query {
                families: &[fontdb::Family::Name(name)],
                ..Default::default()
            })
            .is_some()
        })
        .or(fallback);
    if let Some(family) = fallback {
        db.set_sans_serif_family(family);
    }
    fonts
}

impl Default for TextResources {
    fn default() -> Self {
        #[cfg(not(target_family = "wasm"))]
        let preloaded = PRELOADED_FONTS
            .lock()
            .expect("font preload lock")
            .take()
            .and_then(|scan| scan.join().ok());
        #[cfg(target_family = "wasm")]
        let preloaded = None;
        Self {
            font_system: Arc::new(RwLock::new(preloaded.unwrap_or_else(system_fonts))),
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

/// Renders shaped text with a shared glyph atlas.
pub struct GlyphonTextRender {
    atlas: glyphon::TextAtlas,
    cache: glyphon::Cache,
    swash_cache: glyphon::SwashCache,
    msaa: wgpu::MultisampleState,
    renderers: Vec<TextBatchResources>,
    next_renderer: usize,
}

struct TextBatchResources {
    renderer: glyphon::TextRenderer,
    viewport: glyphon::Viewport,
}

impl GlyphonTextRender {
    /// Creates a text pipeline for the target format and sample count.
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let cache = glyphon::Cache::new(gpu);
        let atlas = glyphon::TextAtlas::new(gpu, queue, &cache, config.format);
        let swash_cache = glyphon::SwashCache::new();
        let msaa = wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let mut pipeline = Self {
            atlas,
            cache,
            swash_cache,
            msaa,
            renderers: Vec::new(),
            next_renderer: 0,
        };
        // Compiles the text render pipeline now rather than on the first text draw.
        pipeline.push_renderer(gpu);
        pipeline
    }

    fn push_renderer(&mut self, device: &wgpu::Device) {
        self.renderers.push(TextBatchResources {
            renderer: glyphon::TextRenderer::new(
                &mut self.atlas,
                device,
                self.msaa,
                tessera_ui::ui_stencil_state(),
            ),
            viewport: glyphon::Viewport::new(device, &self.cache),
        });
    }
}

impl DrawablePipeline<TextCommand> for GlyphonTextRender {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.next_renderer = 0;
        self.atlas.trim();
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
            self.push_renderer(context.device);
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
        let text_buffer = Self::build_buffer(&text, font_size, line_height, font, &constraint);
        let mut data = Self {
            text_buffer: Arc::new(text_buffer),
            size: [0; 2],
            first_baseline: 0.0,
            last_baseline: 0.0,
            line_count: 0,
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
        data.measure_layout();
        data
    }

    /// Lays out the shaped text again for a new constraint without reshaping.
    pub(crate) fn relayout(&mut self, constraint: &TextConstraint) {
        let buffer = Arc::make_mut(&mut self.text_buffer);
        buffer.set_size(constraint.max_width, constraint.max_height);
        buffer.shape_until_scroll(&mut write_font_system(), false);
        self.measure_layout();
    }

    fn measure_layout(&mut self) {
        let buffer = &self.text_buffer;
        let mut width = 0.0_f32;
        self.first_baseline = 0.0;
        self.last_baseline = 0.0;
        self.line_count = 0;
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            if self.line_count == 0 {
                self.first_baseline = run.line_y;
            }
            self.last_baseline = run.line_y;
            self.line_count += 1;
        }
        if buffer.lines.iter().any(|line| {
            matches!(
                line.align(),
                Some(
                    glyphon::cosmic_text::Align::Center
                        | glyphon::cosmic_text::Align::Right
                        | glyphon::cosmic_text::Align::End
                )
            )
        }) {
            width = buffer.size().0.unwrap_or(width);
        }
        self.size = [
            width.ceil() as u32,
            (self.line_count as f32 * buffer.metrics().line_height).ceil() as u32,
        ];
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
        let mut data = Self {
            text_buffer: Arc::new(text_buffer),
            size: [0; 2],
            first_baseline: 0.0,
            last_baseline: 0.0,
            line_count: 0,
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
        };
        data.measure_layout();
        data
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
        self.measure_layout();
    }

    fn build_buffer(
        text: &str,
        size: f32,
        line_height: f32,
        font: TextFont,
        constraint: &TextConstraint,
    ) -> glyphon::Buffer {
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
        text_buffer
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
    use tessera_ui::Color;

    use crate::theme::MaterialTypography;

    use super::{TextConstraint, TextData};

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
    fn wrapping_keeps_words_intact_relays_out_like_fresh_shaping_and_breaks_long_words() {
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
        let unconstrained = TextData::shape(
            "alpha beta".to_string(),
            Color::BLACK,
            style,
            TextConstraint::default(),
        );
        let mut relaid = unconstrained.clone();
        relaid.relayout(&TextConstraint {
            max_width: Some(width),
            max_height: None,
            ..Default::default()
        });
        assert_eq!(
            relaid.measure_info(),
            phrase.measure_info(),
            "relayout must wrap like a fresh shape"
        );
        relaid.relayout(&TextConstraint::default());
        assert_eq!(relaid.measure_info(), unconstrained.measure_info());
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
}
