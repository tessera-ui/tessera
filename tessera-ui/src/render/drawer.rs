//! Graphics rendering pipeline management.

mod command;
mod pipeline;

use crate::render_graph::DrawGeometry;

pub use command::DrawCommand;
pub use pipeline::{DrawContext, DrawablePipeline, FrameContext, PipelineRegistry};

pub(crate) use pipeline::ErasedDrawContext;

/// Drawer manages graphics pipelines and processes draw commands.
#[derive(Default)]
pub(crate) struct Drawer {
    /// Registry containing all registered graphics pipelines
    pub pipeline_registry: PipelineRegistry,
}

impl Drawer {
    /// Create a new drawer
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit a draw command to the appropriate pipeline for rendering.
    ///
    /// # Arguments
    ///
    /// * `context` - Render-pass resources and the active clipping rectangle.
    /// * `commands` - Commands with local dimensions and precise target
    ///   mappings.
    pub fn submit(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, DrawGeometry)],
    ) {
        self.pipeline_registry.dispatch(context, commands);
    }
}
