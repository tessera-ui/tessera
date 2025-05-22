use std::sync::Arc;

use glyphon::fontdb;

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

        Self {
            text_renderer,
            font_system,
            atlas,
            cache,
            viewport,
            swash_cache,
        }
    }

    /// Draw the text at the given position with the given color
    pub fn draw(
        &mut self,
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        text: &str,
        position: [f32; 2],
        color: [f32; 3],
        size: f32,
        line_height: f32,
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
        text_buffer.shape_until_scroll(&mut self.font_system, false);
        // Update the viewport
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        );
        // Prepare the text renderer
        let color = glyphon::Color::rgb(color[0] as u8, color[1] as u8, color[2] as u8);
        self.text_renderer
            .prepare(
                gpu,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [glyphon::TextArea {
                    buffer: &text_buffer,
                    left: position[0],
                    top: position[1],
                    scale: 1.0,
                    bounds: glyphon::TextBounds::default(),
                    default_color: color,
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();
        // Do the rendering
        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }
}
