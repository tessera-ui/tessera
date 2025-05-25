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
    /// returns the text data buffer
    /// Notice that we must specify the text position
    /// before rendering its return value
    pub fn build_text_data(
        &mut self,
        text: &str,
        color: [u8; 3],
        size: f32,
        line_height: f32,
        constraint: TextConstraint,
    ) -> TextData {
        // Create text buffer
        let mut text_buffer = glyphon::Buffer::new(
            &mut self.font_system,
            glyphon::Metrics::new(size, line_height),
        );
        let color = glyphon::Color::rgb(color[0], color[1], color[2]);
        text_buffer.set_text(
            &mut self.font_system,
            text,
            &glyphon::Attrs::new()
                .family(fontdb::Family::SansSerif)
                .color(color),
            glyphon::Shaping::Advanced,
        );
        text_buffer.set_size(
            &mut self.font_system,
            constraint.max_width,
            constraint.max_height,
        );
        text_buffer.shape_until_scroll(&mut self.font_system, false);
        // Calculate text bounds
        // Get the layout runs
        let layout_runs = text_buffer.layout_runs();
        let mut run_width: f32 = 0.0;
        let line_height = text_buffer.lines.len() as f32 * text_buffer.metrics().line_height;
        for run in layout_runs {
            // Take the max. width of all lines.
            run_width = run_width.max(run.line_w);
        }
        // build text data
        TextData::new(
            text_buffer,
            None,
            color,
            [run_width as u32, line_height as u32],
        )
    }

    /// Add a text data to the buffer, waiting to be drawn
    pub fn push(&mut self, text_data: TextData) {
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

#[derive(Debug)]
pub struct TextData {
    /// glyphon text buffer
    text_buffer: glyphon::Buffer,
    // we need fields below to construct a glyphon::TextArea
    // don't ask me why we cannot just save a glyphon::TextArea, ask borrow checker
    /// Position of the text, none if it is not computed yet
    pub position: Option<[u32; 2]>,
    /// Color of the text
    color: glyphon::Color,
    /// Text area size
    pub size: [u32; 2],
}

impl PartialEq for TextData {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.color == other.color && self.size == other.size
    }
}

impl TextData {
    /// Create a new text data
    pub fn new(
        text_buffer: glyphon::Buffer,
        position: Option<[u32; 2]>,
        color: glyphon::Color,
        size: [u32; 2],
    ) -> Self {
        Self {
            text_buffer,
            position,
            color,
            size,
        }
    }

    /// Get the glyphon text area from the text data
    fn text_area(&self) -> glyphon::TextArea {
        let bounds = glyphon::TextBounds {
            left: self.position.unwrap()[0] as i32,
            top: self.position.unwrap()[1] as i32,
            right: self.position.unwrap()[0] as i32 + self.size[0] as i32,
            bottom: self.position.unwrap()[1] as i32 + self.size[1] as i32,
        };
        glyphon::TextArea {
            buffer: &self.text_buffer,
            left: self.position.unwrap()[0] as f32,
            top: self.position.unwrap()[1] as f32,
            scale: 1.0,
            bounds,
            default_color: self.color,
            custom_glyphs: &[],
        }
    }
}
