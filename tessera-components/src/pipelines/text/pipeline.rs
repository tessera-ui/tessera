//! Text Rendering Pipeline for UI Components
//!
//! This module implements the GPU pipeline and related utilities for efficient
//! text rendering in Tessera UI components. It leverages the Glyphon engine for
//! font management, shaping, and rasterization, providing high-quality and
//! performant text output. Typical use cases include rendering static labels,
//! paragraphs, and editable text fields within the UI.
//!
//! The pipeline is designed to be reusable and efficient, sharing a static font
//! system across the application to minimize resource usage. It exposes APIs
//! for preparing, measuring, and rendering text, supporting advanced features
//! such as font fallback, shaping, and multi-line layout.
//!
//! This module is intended for integration into custom UI components and
//! rendering flows that require flexible and robust text display.

use std::{num::NonZero, sync::OnceLock};

use glyphon::fontdb;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tessera_ui::{
    Color, PxPosition,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use crate::theme::TextStyle;

use super::command::{TextCommand, TextConstraint};

#[cfg(target_family = "wasm")]
use crate::res;
#[cfg(target_family = "wasm")]
use std::io;
#[cfg(target_family = "wasm")]
use tessera_ui::AssetExt;

const LRU_CAPACITY: usize = 1024;

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
static FONT_SYSTEM: OnceLock<RwLock<glyphon::FontSystem>> = OnceLock::new();

/// Create TextData is a heavy operation, so we provide a lru cache to store
/// recently used TextData.
static TEXT_DATA_CACHE: OnceLock<RwLock<lru::LruCache<LruKey, TextData>>> = OnceLock::new();

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

struct LruKey {
    text: String,
    font_size: f32,
    line_height: f32,
    /// The final computed bounds, used as the cache key instead of constraint.
    bounds: [u32; 2],
    font: TextFont,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TextFont {
    weight: u16,
    tracking: f32,
    family: Option<&'static str>,
}

impl PartialEq for LruKey {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.font_size.to_bits() == other.font_size.to_bits()
            && self.line_height.to_bits() == other.line_height.to_bits()
            && self.bounds == other.bounds
            && self.font == other.font
    }
}

impl Eq for LruKey {}

impl std::hash::Hash for LruKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_size.to_bits().hash(state);
        self.line_height.to_bits().hash(state);
        self.bounds.hash(state);
        self.font.weight.hash(state);
        self.font.tracking.to_bits().hash(state);
        self.font.family.hash(state);
    }
}

fn write_lru_cache() -> RwLockWriteGuard<'static, lru::LruCache<LruKey, TextData>> {
    TEXT_DATA_CACHE
        .get_or_init(|| {
            RwLock::new(lru::LruCache::new(
                NonZero::new(LRU_CAPACITY).expect("text cache size must be non-zero"),
            ))
        })
        .write()
}

fn load_material_fonts(db: &mut fontdb::Database) {
    let existing: Vec<_> = db
        .faces()
        .filter(|face| face.families.iter().any(|(name, _)| name == "Roboto"))
        .map(|face| face.id)
        .collect();
    for id in existing {
        db.remove_face(id);
    }
    #[cfg(not(target_family = "wasm"))]
    for bytes in [
        include_bytes!("../../../assets/Roboto-Regular.ttf").as_slice(),
        include_bytes!("../../../assets/Roboto-Medium.ttf").as_slice(),
        include_bytes!("../../../assets/Roboto-Bold.ttf").as_slice(),
    ] {
        db.load_font_data(bytes.to_vec());
    }
    #[cfg(target_family = "wasm")]
    for asset in [
        res::ROBOTO_REGULAR_TTF,
        res::ROBOTO_MEDIUM_TTF,
        res::ROBOTO_BOLD_TTF,
    ] {
        if let Err(err) = load_asset_font_data(db, asset) {
            eprintln!("wasm fontdb: failed to load Roboto asset: {err}");
        }
    }
    db.set_sans_serif_family("Roboto");
}

#[cfg(target_os = "android")]
fn is_android_emoji_face(face: &fontdb::FaceInfo) -> bool {
    let post_script = face.post_script_name.as_str();
    if post_script.contains("NotoColorEmoji") {
        return true;
    }

    face.families
        .iter()
        .any(|(name, _)| name.contains("Noto Color Emoji"))
}

#[cfg(target_os = "android")]
fn remove_system_emoji_faces(db: &mut fontdb::Database) -> usize {
    let ids: Vec<_> = db
        .faces()
        .filter(|face| {
            matches!(
                &face.source,
                fontdb::Source::File(_) | fontdb::Source::SharedFile(_, _)
            ) && is_android_emoji_face(face)
        })
        .map(|face| face.id)
        .collect();

    // Android ships COLRv1 emoji fonts, which swash cannot rasterize yet.
    for id in &ids {
        db.remove_face(*id);
    }

    ids.len()
}

#[cfg(target_os = "android")]
fn load_embedded_android_emoji_font(db: &mut fontdb::Database) -> usize {
    let before = db.len();
    let data = include_bytes_zstd::include_bytes_zstd!("assets/NotoColorEmoji_COLRv0.ttf", 19);
    if data.is_empty() {
        return 0;
    }
    db.load_font_data(data);
    db.len().saturating_sub(before)
}

#[cfg(target_os = "android")]
fn ensure_android_emoji_font(db: &mut fontdb::Database) {
    // Use the bundled COLRv0 emoji font to avoid COLRv1 rendering gaps.
    remove_system_emoji_faces(db);
    load_embedded_android_emoji_font(db);
}

#[cfg(target_os = "android")]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    let mut font_system = glyphon::FontSystem::new();

    {
        let db = font_system.db_mut();
        db.load_fonts_dir("/system/fonts");
        ensure_android_emoji_font(db);
        load_material_fonts(db);
        db.set_sans_serif_family("Roboto");
        db.set_serif_family("Noto Serif");
        db.set_monospace_family("Droid Sans Mono");
        db.set_cursive_family("Dancing Script");
        db.set_fantasy_family("Dancing Script");
    }

    RwLock::new(font_system)
}

#[cfg(target_family = "wasm")]
fn load_asset_font_data(db: &mut fontdb::Database, asset: res::Asset) -> io::Result<usize> {
    let before = db.len();
    let bytes = asset.read()?;
    db.load_font_data(bytes.as_ref().to_vec());
    Ok(db.len().saturating_sub(before))
}

#[cfg(target_family = "wasm")]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    let mut font_system = glyphon::FontSystem::new();

    {
        let db = font_system.db_mut();
        if let Err(err) = load_asset_font_data(db, res::NOTOSANSSC_REGULAR_OTF) {
            eprintln!("wasm fontdb: failed to load Noto Sans SC asset: {err}");
        }
        if let Err(err) = load_asset_font_data(db, res::NOTOCOLOREMOJI_COLRV0_TTF) {
            eprintln!("wasm fontdb: failed to load emoji font asset: {err}");
        }
        load_material_fonts(db);
        db.set_serif_family("Noto Sans SC");
        db.set_monospace_family("Noto Sans SC");
        db.set_cursive_family("Noto Sans SC");
        db.set_fantasy_family("Noto Sans SC");
    }

    RwLock::new(font_system)
}

#[cfg(not(target_os = "android"))]
#[cfg(not(target_family = "wasm"))]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    let mut font_system = glyphon::FontSystem::new();
    load_material_fonts(font_system.db_mut());
    RwLock::new(font_system)
}

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
/// This function returns a read lock of the font system.
pub fn read_font_system() -> RwLockReadGuard<'static, glyphon::FontSystem> {
    FONT_SYSTEM.get_or_init(init_font_system).read()
}

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
/// This function returns a write lock of the font system.
pub fn write_font_system() -> RwLockWriteGuard<'static, glyphon::FontSystem> {
    FONT_SYSTEM.get_or_init(init_font_system).write()
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

        let text_areas = context.commands.iter().map(|(command, _size, start_pos)| {
            let start_pos = PxPosition::from_f32_xy(
                start_pos.x + command.offset.x,
                start_pos.y + command.offset.y,
            );
            command.data.text_area(start_pos)
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
    text_buffer: glyphon::Buffer,
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
    }
}

impl TextData {
    /// Shapes text once and stores the shaped buffer in the global LRU.
    pub(crate) fn shape(
        text: String,
        color: Color,
        style: TextStyle,
        constraint: TextConstraint,
    ) -> Self {
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
        let key = LruKey {
            text: text.clone(),
            font_size,
            line_height,
            bounds,
            font,
        };
        let data = Self {
            text_buffer,
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
        };
        write_lru_cache().put(key, data.clone());
        data
    }

    /// Builds [`TextData`] directly from a pre-shaped glyphon buffer.
    pub fn from_buffer(text_buffer: glyphon::Buffer) -> Self {
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
            text_buffer,
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
        }
    }

    /// Get the glyphon text area from the text data
    fn text_area(&'_ self, start_pos: PxPosition) -> glyphon::TextArea<'_> {
        let bounds = glyphon::TextBounds {
            left: start_pos.x_px().raw(),
            top: start_pos.y_px().raw(),
            right: start_pos.x_px().raw() + self.size[0] as i32,
            bottom: start_pos.y_px().raw() + self.size[1] as i32,
        };
        glyphon::TextArea {
            buffer: &self.text_buffer,
            left: start_pos.x,
            top: start_pos.y,
            scale: 1.0,
            bounds,
            default_color: color_to_glyphon(self.current_color),
            custom_glyphs: &[],
        }
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
        text_buffer.set_wrap(glyphon::Wrap::WordOrGlyph);
        text_buffer.set_size(constraint.max_width, constraint.max_height);
        text_buffer.set_text(
            text,
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
        let target_alpha = (self.base_color.a * opacity).clamp(0.0, 1.0);
        let target_color = self.base_color.with_alpha(target_alpha);
        if (target_color.a - self.current_color.a).abs() <= f32::EPSILON
            && (target_color.r - self.current_color.r).abs() <= f32::EPSILON
            && (target_color.g - self.current_color.g).abs() <= f32::EPSILON
            && (target_color.b - self.current_color.b).abs() <= f32::EPSILON
        {
            return;
        }

        self.current_color = target_color;
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{Color, Dp};

    use crate::theme::MaterialTypography;

    use super::{TextConstraint, TextData, read_font_system};

    #[test]
    fn wrapping_keeps_words_intact_and_falls_back_for_words_wider_than_the_line() {
        let style = MaterialTypography::default().body_medium;
        let alpha = TextData::shape(
            "alpha ".to_string(),
            Color::BLACK,
            style,
            TextConstraint {
                max_width: None,
                max_height: None,
            },
        );
        let beta = TextData::shape(
            "beta".to_string(),
            Color::BLACK,
            style,
            TextConstraint {
                max_width: None,
                max_height: None,
            },
        );
        let width = alpha.size[0] as f32 + beta.size[0] as f32 / 2.0;
        let phrase = TextData::shape(
            "alpha beta".to_string(),
            Color::BLACK,
            style,
            TextConstraint {
                max_width: Some(width),
                max_height: None,
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
            },
        );
        let runs: Vec<_> = long.text_buffer.layout_runs().collect();
        assert!(runs.len() > 1);
        assert!(runs.iter().all(|run| run.line_w <= width + 1.0));
    }

    #[test]
    fn material_text_uses_bundled_weight_and_tracking_with_exact_line_height() {
        let typography = MaterialTypography::default();
        let shape = |style| {
            TextData::shape(
                "Material typography".to_owned(),
                Color::BLACK,
                style,
                TextConstraint {
                    max_width: None,
                    max_height: None,
                },
            )
        };
        let regular = shape(typography.body_large);
        let medium = shape(crate::theme::TextStyle {
            font_weight: 500,
            ..typography.body_large
        });
        let untracked = shape(crate::theme::TextStyle {
            letter_spacing: Dp(0.0),
            ..typography.body_large
        });
        assert_eq!(regular.size[1], Dp(24.0).to_pixels_f32().ceil() as u32);
        assert!(regular.first_baseline > 0.0 && regular.first_baseline < regular.size[1] as f32);
        assert!(regular.size[0] > untracked.size[0]);
        let font_system = read_font_system();
        for (data, weight) in [(&regular, 400), (&medium, 500)] {
            let glyph = &data.text_buffer.layout_runs().next().unwrap().glyphs[0];
            let face = font_system.db().face(glyph.font_id).unwrap();
            assert_eq!(face.weight.0, weight);
            assert!(face.families.iter().any(|(family, _)| family == "Roboto"));
        }
    }
}
