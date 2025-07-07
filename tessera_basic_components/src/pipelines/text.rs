mod command;

use std::sync::OnceLock;

use glyphon::fontdb;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tessera::{DrawablePipeline, PxPosition, PxSize, wgpu};

pub use command::{TextCommand, TextConstraint};

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
static FONT_SYSTEM: OnceLock<RwLock<glyphon::FontSystem>> = OnceLock::new();

#[cfg(target_os = "android")]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    let mut font_system = glyphon::FontSystem::new();

    font_system.db_mut().load_fonts_dir("/system/fonts");
    font_system.db_mut().set_sans_serif_family("Roboto");
    font_system.db_mut().set_serif_family("Noto Serif");
    font_system.db_mut().set_monospace_family("Droid Sans Mono");
    font_system.db_mut().set_cursive_family("Dancing Script");
    font_system.db_mut().set_fantasy_family("Dancing Script");

    RwLock::new(font_system)
}

#[cfg(not(target_os = "android"))]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    RwLock::new(glyphon::FontSystem::new())
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
pub struct GlyphonTextRender {
    /// Glypthon text renderer
    text_renderer: glyphon::TextRenderer,
    /// Glypthon font atlas
    atlas: glyphon::TextAtlas,
    /// Glypthon cache
    #[allow(unused)]
    cache: glyphon::Cache,
    /// Glypthon viewport
    viewport: glyphon::Viewport,
    /// Glypthon swash cache
    swash_cache: glyphon::SwashCache,
    /// Buffer for text datas to render
    buffer: Vec<(PxPosition, TextData)>,
}

impl GlyphonTextRender {
    /// Create a new text renderer
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        // Create glyphon cache
        let cache = glyphon::Cache::new(gpu);
        // Create a font atlas
        let mut atlas = glyphon::TextAtlas::new(gpu, queue, &cache, config.format);
        // Create text renderer
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            gpu,
            wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );
        // Create glyphon Viewport
        let viewport = glyphon::Viewport::new(gpu, &cache);
        // Create swash cache
        let swash_cache = glyphon::SwashCache::new();
        // Create a buffer for text datas
        let buffer = Vec::new();

        Self {
            text_renderer,
            atlas,
            cache,
            viewport,
            swash_cache,
            buffer,
        }
    }

    /// Add a text data to the buffer, waiting to be drawn
    pub fn push(&mut self, start_pos: PxPosition, text_data: TextData) {
        self.buffer.push((start_pos, text_data));
    }

    /// Draw the text at the given position with the given color
    pub fn draw_all_to_pass(
        &mut self,
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        // Update the viewport
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        );
        // Prepare the text renderer
        self.text_renderer
            .prepare(
                gpu,
                queue,
                &mut write_font_system(),
                &mut self.atlas,
                &self.viewport,
                self.buffer.iter().map(|(pos, data)| data.text_area(*pos)),
                &mut self.swash_cache,
            )
            .unwrap();
        // Do the rendering
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
        // Clear the buffer
        self.buffer.clear();
    }
}

#[allow(unused_variables)]
impl DrawablePipeline<TextCommand> for GlyphonTextRender {
    fn draw(
        &mut self,
        _gpu: &wgpu::Device,
        _gpu_queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        _render_pass: &mut wgpu::RenderPass<'_>,
        command: &TextCommand,
        _size: PxSize,
        start_pos: PxPosition,
        _scene_texture_view: Option<&wgpu::TextureView>,
        _compute_registry: &mut tessera::renderer::compute::ComputePipelineRegistry,
    ) {
        self.push(start_pos, command.data.clone());
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.draw_all_to_pass(gpu, config, gpu_queue, render_pass);
    }
}

#[derive(Debug, Clone)]
pub struct TextData {
    /// glyphon text buffer
    text_buffer: glyphon::Buffer,
    /// text area size
    pub size: [u32; 2],
}

impl TextData {
    /// Prepare all text datas before rendering
    /// returns the text data buffer
    /// Notice that we must specify the text position
    /// before rendering its return value
    pub fn new(
        text: String,
        color: [u8; 3],
        size: f32,
        line_height: f32,
        constraint: TextConstraint,
    ) -> TextData {
        // Create text buffer
        let mut text_buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size, line_height),
        );
        let color = glyphon::Color::rgb(color[0], color[1], color[2]);
        text_buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
        text_buffer.set_size(
            &mut write_font_system(),
            constraint.max_width,
            constraint.max_height,
        );
        text_buffer.set_text(
            &mut write_font_system(),
            &text,
            &glyphon::Attrs::new()
                .family(fontdb::Family::SansSerif)
                .color(color),
            glyphon::Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut write_font_system(), false);
        // Calculate text bounds
        // Get the layout runs
        let mut run_width: f32 = 0.0;
        // Calculate the line height based on the number of lines
        let line_height =
            text_buffer.layout_runs().count() as f32 * text_buffer.metrics().line_height;
        for run in text_buffer.layout_runs() {
            // Take the max. width of all lines.
            run_width = run_width.max(run.line_w);
        }
        // build text data
        Self {
            text_buffer,
            size: [run_width as u32, line_height as u32],
        }
    }

    pub fn from_buffer(text_buffer: glyphon::Buffer) -> Self {
        // Calculate text bounds
        // Get the layout runs
        let mut run_width: f32 = 0.0;
        // Calculate the line height based on the number of lines
        let line_height =
            text_buffer.layout_runs().count() as f32 * text_buffer.metrics().line_height;
        for run in text_buffer.layout_runs() {
            // Take the max. width of all lines.
            run_width = run_width.max(run.line_w);
        }
        // build text data
        Self {
            text_buffer,
            size: [run_width as u32, line_height as u32],
        }
    }

    /// Get the glyphon text area from the text data
    fn text_area(&'_ self, start_pos: PxPosition) -> glyphon::TextArea<'_> {
        let bounds = glyphon::TextBounds {
            left: start_pos.x.raw(),
            top: start_pos.y.raw(),
            right: start_pos.x.raw() + self.size[0] as i32,
            bottom: start_pos.y.raw() + self.size[1] as i32,
        };
        glyphon::TextArea {
            buffer: &self.text_buffer,
            left: start_pos.x.to_f32(),
            top: start_pos.y.to_f32(),
            scale: 1.0,
            bounds,
            default_color: glyphon::Color::rgb(0, 0, 0), // Black by default
            custom_glyphs: &[],
        }
    }
}
