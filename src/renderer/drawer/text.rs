use std::sync::Arc;

use glyphon::fontdb;

use super::command::TextConstraint;

/// A text renderer
pub struct GlyphonTextRender {
    /// Glypthon text renderer
    text_renderer: glyphon::TextRenderer,
    /// Glypthon font system
    font_system: glyphon::FontSystem,
    /// Glypthon font atlas
    atlas: glyphon::TextAtlas,
    /// Glypthon cache
    cache: glyphon::Cache,
    /// Glypthon viewport
    viewport: glyphon::Viewport,
    /// Glypthon swash cache
    swash_cache: glyphon::SwashCache,
    /// Buffer for text datas to render
    buffer: Vec<TextData>,
}

impl GlyphonTextRender {
    /// Create a new text renderer
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        // Load fonts from system and assets
        let font_system = glyphon::FontSystem::new_with_fonts([fontdb::Source::Binary(Arc::new(
            include_bytes!("../assets/fonts/NotoSansSC-Regular.otf"),
        ))]);
        // Create glyphon cache
        let cache = glyphon::Cache::new(gpu);
        // Create a font atlas
        let mut atlas = glyphon::TextAtlas::new(gpu, queue, &cache, config.format);
        // Create text renderer
        let text_renderer =
            glyphon::TextRenderer::new(&mut atlas, gpu, wgpu::MultisampleState::default(), None);
        // Create glyphon Viewport
        let viewport = glyphon::Viewport::new(gpu, &cache);
        // Create swash cache
        let swash_cache = glyphon::SwashCache::new();
        // Create a buffer for text datas
        let buffer = Vec::new();

        Self {
            text_renderer,
            font_system,
            atlas,
            cache,
            viewport,
            swash_cache,
            buffer,
        }
    }

    /// Prepare all text datas before rendering
    /// We must call [Self::draw] after all [Self::prepare] calls
    /// to render the text
    pub fn prepare(
        &mut self,
        text: &str,
        position: [u32; 2],
        color: [f32; 3],
        size: f32,
        line_height: f32,
        constraint: TextConstraint,
    ) {
        // Create text buffer
        let mut text_buffer = glyphon::Buffer::new(
            &mut self.font_system,
            glyphon::Metrics::new(size, line_height),
        );
        text_buffer.set_text(
            &mut self.font_system,
            text,
            &glyphon::Attrs::new().family(fontdb::Family::SansSerif),
            glyphon::Shaping::Advanced,
        );
        text_buffer.set_size(
            &mut self.font_system,
            Some(constraint.max_width as f32),
            Some(constraint.max_height as f32),
        );
        text_buffer.shape_until_scroll(&mut self.font_system, false);
        // Calculate text bounds
        let bound = glyphon::TextBounds {
            left: position[0] as i32,
            top: position[1] as i32,
            right: position[0] as i32 + constraint.max_width as i32,
            bottom: position[1] as i32 + constraint.max_height as i32,
        };
        // build text data and push it to the buffer for later rendering
        let text_data = TextData::new(text_buffer, position, color, bound);
        self.buffer.push(text_data);
    }

    /// Draw the text at the given position with the given color
    pub fn draw(
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
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                self.buffer.iter().map(|t| t.text_area()),
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

struct TextData {
    /// glyphon text buffer
    text_buffer: glyphon::Buffer,
    // we need fields below to construct a glyphon::TextArea
    // don't ask me why we cannot just save a glyphon::TextArea, ask borrow checker
    /// Position of the text
    position: [u32; 2],
    /// Color of the text
    color: glyphon::Color,
    /// Text bounds of the text
    bounds: glyphon::TextBounds,
}

impl TextData {
    /// Create a new text data
    pub fn new(
        text_buffer: glyphon::Buffer,
        position: [u32; 2],
        color: [f32; 3],
        text_bounds: glyphon::TextBounds,
    ) -> Self {
        Self {
            text_buffer,
            position,
            color: glyphon::Color::rgb(color[0] as u8, color[1] as u8, color[2] as u8),
            bounds: text_bounds,
        }
    }

    /// Get the glyphon text area from the text data
    fn text_area(&self) -> glyphon::TextArea {
        glyphon::TextArea {
            buffer: &self.text_buffer,
            left: self.position[0] as f32,
            top: self.position[1] as f32,
            scale: 1.0,
            bounds: self.bounds,
            default_color: self.color,
            custom_glyphs: &[],
        }
    }
}
