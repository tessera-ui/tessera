//! Graphics rendering pipeline system for Tessera UI framework.
//!
//! This module provides the core infrastructure for pluggable graphics rendering pipelines
//! in Tessera. The design philosophy emphasizes flexibility and extensibility, allowing
//! developers to create custom rendering effects without being constrained by built-in
//! drawing primitives.
//!
//! # Architecture Overview
//!
//! The pipeline system uses a trait-based approach with type erasure to support dynamic
//! dispatch of rendering commands. Each pipeline is responsible for rendering a specific
//! type of draw command, such as shapes, text, images, or custom visual effects.
//!
//! ## Key Components
//!
//! - [`DrawablePipeline<T>`]: The main trait for implementing custom rendering pipelines
//! - [`PipelineRegistry`]: Manages and dispatches commands to registered pipelines
//! - [`ErasedDrawablePipeline`]: Internal trait for type erasure and dynamic dispatch
//!
//! # Design Philosophy
//!
//! Unlike traditional UI frameworks that provide built-in "brush" or drawing primitives,
//! Tessera treats shaders as first-class citizens. This approach offers several advantages:
//!
//! - **Modern GPU Utilization**: Leverages WGPU and WGSL for efficient, cross-platform rendering
//! - **Advanced Visual Effects**: Enables complex effects like neumorphic design, lighting,
//!   shadows, reflections, and bloom that are difficult to achieve with traditional approaches
//! - **Flexibility**: Custom shaders allow for unlimited creative possibilities
//! - **Performance**: Direct GPU programming eliminates abstraction overhead
//!
//! # Pipeline Lifecycle
//!
//! Each pipeline follows a three-phase lifecycle during rendering:
//!
//! 1. **Begin Pass**: Setup phase for initializing pipeline-specific resources
//! 2. **Draw**: Main rendering phase where commands are processed
//! 3. **End Pass**: Cleanup phase for finalizing rendering operations
//!
//! # Implementation Guide
//!
//! ## Creating a Custom Pipeline
//!
//! To create a custom rendering pipeline:
//!
//! 1. Define your draw command struct implementing [`DrawCommand`]
//! 2. Create a pipeline struct implementing [`DrawablePipeline<YourCommand>`]
//! 3. Register the pipeline with [`PipelineRegistry::register`]
//!
//! ## Example: Simple Rectangle Pipeline
//!
//! ```rust,ignore
//! use tessera_ui::{DrawCommand, DrawablePipeline, PxPosition, PxSize};
//! use wgpu;
//!
//! // 1. Define the draw command
//! #[derive(Debug)]
//! struct RectangleCommand {
//!     color: [f32; 4],
//!     corner_radius: f32,
//! }
//!
//! impl DrawCommand for RectangleCommand {
//!     // Most commands don't need barriers
//!     fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
//!         None
//!     }
//! }
//!
//! // 2. Implement the pipeline
//! struct RectanglePipeline {
//!     render_pipeline: wgpu::RenderPipeline,
//!     uniform_buffer: wgpu::Buffer,
//!     bind_group: wgpu::BindGroup,
//! }
//!
//! impl RectanglePipeline {
//!     fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
//!         // Create shader, pipeline, buffers, etc.
//!         // ... implementation details ...
//!         # unimplemented!()
//!     }
//! }
//!
//! impl DrawablePipeline<RectangleCommand> for RectanglePipeline {
//!     fn draw(
//!         &mut self,
//!         gpu: &wgpu::Device,
//!         gpu_queue: &wgpu::Queue,
//!         config: &wgpu::SurfaceConfiguration,
//!         render_pass: &mut wgpu::RenderPass<'_>,
//!         command: &RectangleCommand,
//!         size: PxSize,
//!         start_pos: PxPosition,
//!         scene_texture_view: &wgpu::TextureView,
//!     ) {
//!         // Update uniforms with command data
//!         // Set pipeline and draw
//!         render_pass.set_pipeline(&self.render_pipeline);
//!         render_pass.set_bind_group(0, &self.bind_group, &[]);
//!         render_pass.draw(0..6, 0..1); // Draw quad
//!     }
//! }
//!
//! // 3. Register the pipeline
//! let mut registry = PipelineRegistry::new();
//! let rectangle_pipeline = RectanglePipeline::new(&device, &config, sample_count);
//! registry.register(rectangle_pipeline);
//! ```
//!
//! # Integration with Basic Components
//!
//! The `tessera_basic_components` crate demonstrates real-world pipeline implementations:
//!
//! - **ShapePipeline**: Renders rounded rectangles, circles, and complex shapes with shadows and ripple effects
//! - **TextPipeline**: Handles text rendering with font management and glyph caching
//! - **ImagePipeline**: Displays images with various scaling and filtering options
//! - **FluidGlassPipeline**: Creates advanced glass effects with distortion and transparency
//!
//! These pipelines are registered in `tessera_ui_basic_components::pipelines::register_pipelines()`.
//!
//! # Performance Considerations
//!
//! - **Batch Similar Commands**: Group similar draw commands to minimize pipeline switches
//! - **Resource Management**: Reuse buffers and textures when possible
//! - **Shader Optimization**: Write efficient shaders optimized for your target platforms
//! - **State Changes**: Minimize render state changes within the draw method
//!
//! # Advanced Features
//!
//! ## Barrier Requirements
//!
//! Some rendering effects need to sample from previously rendered content (e.g., blur effects).
//! Implement [`DrawCommand::barrier()`] to return [`BarrierRequirement::SampleBackground`]
//! for such commands.
//!
//! ## Multi-Pass Rendering
//!
//! Use `begin_pass()` and `end_pass()` for pipelines that require multiple rendering passes
//! or complex setup/teardown operations.
//!
//! ## Scene Texture Access
//!
//! The `scene_texture_view` parameter provides access to the current scene texture,
//! enabling effects that sample from the background or perform post-processing.

use crate::{PxPosition, px::PxSize, renderer::DrawCommand};

/// Core trait for implementing custom graphics rendering pipelines.
///
/// This trait defines the interface for rendering pipelines that process specific types
/// of draw commands. Each pipeline is responsible for setting up GPU resources,
/// managing render state, and executing the actual drawing operations.
///
/// # Type Parameters
///
/// * `T` - The specific [`DrawCommand`] type this pipeline can handle
///
/// # Lifecycle Methods
///
/// The pipeline system provides three lifecycle hooks:
///
/// - [`begin_pass()`](Self::begin_pass): Called once at the start of the render pass
/// - [`draw()`](Self::draw): Called for each command of type `T`
/// - [`end_pass()`](Self::end_pass): Called once at the end of the render pass
///
/// # Implementation Notes
///
/// - Only the [`draw()`](Self::draw) method is required; others have default empty implementations
/// - Pipelines should be stateless between frames when possible
/// - Resource management should prefer reuse over recreation
/// - Consider batching multiple commands for better performance
///
/// # Example
///
/// See the module-level documentation for a complete implementation example.
#[allow(unused_variables)]
pub trait DrawablePipeline<T: DrawCommand> {
    /// Called once at the beginning of the render pass.
    ///
    /// Use this method to perform one-time setup operations that apply to all
    /// draw commands of this type in the current frame. This is ideal for:
    ///
    /// - Setting up shared uniform buffers
    /// - Binding global resources
    /// - Configuring render state that persists across multiple draw calls
    ///
    /// # Parameters
    ///
    /// * `gpu` - The WGPU device for creating resources
    /// * `gpu_queue` - The WGPU queue for submitting commands
    /// * `config` - Current surface configuration
    /// * `render_pass` - The active render pass
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing, which is suitable for most pipelines.
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
    }

    /// Called once at the end of the render pass.
    ///
    /// Use this method to perform cleanup operations or finalize rendering
    /// for all draw commands of this type in the current frame. This is useful for:
    ///
    /// - Cleaning up temporary resources
    /// - Finalizing multi-pass rendering operations
    /// - Submitting batched draw calls
    ///
    /// # Parameters
    ///
    /// * `gpu` - The WGPU device for creating resources
    /// * `gpu_queue` - The WGPU queue for submitting commands
    /// * `config` - Current surface configuration
    /// * `render_pass` - The active render pass
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing, which is suitable for most pipelines.
    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
    }

    /// Renders a single draw command.
    ///
    /// This is the core method where the actual rendering happens. It's called
    /// once for each draw command of type `T` that needs to be rendered.
    ///
    /// # Parameters
    ///
    /// * `gpu` - The WGPU device for creating resources
    /// * `gpu_queue` - The WGPU queue for submitting commands and updating buffers
    /// * `config` - Current surface configuration containing format and size information
    /// * `render_pass` - The active render pass to record draw commands into
    /// * `command` - The specific draw command to render
    /// * `size` - The size of the rendering area in pixels
    /// * `start_pos` - The top-left position where rendering should begin
    /// * `scene_texture_view` - View of the current scene texture for background sampling
    ///
    /// # Implementation Guidelines
    ///
    /// - Update any per-command uniforms or push constants
    /// - Set the appropriate render pipeline
    /// - Bind necessary resources (textures, buffers, bind groups)
    /// - Issue draw calls (typically `draw()`, `draw_indexed()`, or `draw_indirect()`)
    /// - Avoid expensive operations like buffer creation; prefer reusing resources
    ///
    /// # Scene Texture Usage
    ///
    /// The `scene_texture_view` provides access to the current rendered scene,
    /// enabling effects that sample from the background. This is commonly used for:
    ///
    /// - Blur and post-processing effects
    /// - Glass and transparency effects
    /// - Distortion and refraction
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn draw(&mut self, gpu: &wgpu::Device, gpu_queue: &wgpu::Queue,
    ///         config: &wgpu::SurfaceConfiguration, render_pass: &mut wgpu::RenderPass<'_>,
    ///         command: &MyCommand, size: PxSize, start_pos: PxPosition,
    ///         scene_texture_view: &wgpu::TextureView) {
    ///     // Update uniforms with command-specific data
    ///     let uniforms = MyUniforms {
    ///         color: command.color,
    ///         position: [start_pos.x as f32, start_pos.y as f32],
    ///         size: [size.width as f32, size.height as f32],
    ///     };
    ///     gpu_queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    ///     
    ///     // Set pipeline and resources
    ///     render_pass.set_pipeline(&self.render_pipeline);
    ///     render_pass.set_bind_group(0, &self.bind_group, &[]);
    ///     
    ///     // Draw a quad (two triangles)
    ///     render_pass.draw(0..6, 0..1);
    /// }
    /// ```
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &T,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: &wgpu::TextureView,
    );
}

/// Internal trait for type erasure of drawable pipelines.
///
/// This trait enables dynamic dispatch of draw commands to their corresponding pipelines
/// without knowing the specific command type at compile time. It's used internally by
/// the [`PipelineRegistry`] and should not be implemented directly by users.
///
/// The type erasure is achieved through the [`AsAny`] trait, which allows downcasting
/// from `&dyn DrawCommand` to concrete command types.
///
/// # Implementation Note
///
/// This trait is automatically implemented for any type that implements
/// [`DrawablePipeline<T>`] through the [`DrawablePipelineImpl`] wrapper.
pub trait ErasedDrawablePipeline {
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    );

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    );

    fn draw_erased(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: &wgpu::TextureView,
    ) -> bool;
}

struct DrawablePipelineImpl<T: DrawCommand, P: DrawablePipeline<T>> {
    pipeline: P,
    _marker: std::marker::PhantomData<T>,
}

impl<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static> ErasedDrawablePipeline
    for DrawablePipelineImpl<T, P>
{
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.pipeline
            .begin_pass(gpu, gpu_queue, config, render_pass);
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.pipeline.end_pass(gpu, gpu_queue, config, render_pass);
    }

    fn draw_erased(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: &wgpu::TextureView,
    ) -> bool {
        if let Some(cmd) = command.as_any().downcast_ref::<T>() {
            self.pipeline.draw(
                gpu,
                gpu_queue,
                config,
                render_pass,
                cmd,
                size,
                start_pos,
                scene_texture_view,
            );
            true
        } else {
            false
        }
    }
}

/// Registry for managing and dispatching drawable pipelines.
///
/// The `PipelineRegistry` serves as the central hub for all rendering pipelines in the
/// Tessera framework. It maintains a collection of registered pipelines and handles
/// the dispatch of draw commands to their appropriate pipelines.
///
/// # Architecture
///
/// The registry uses type erasure to store pipelines of different types in a single
/// collection. When a draw command needs to be rendered, the registry iterates through
/// all registered pipelines until it finds one that can handle the command type.
///
/// # Usage Pattern
///
/// 1. Create a new registry
/// 2. Register all required pipelines during application initialization
/// 3. The renderer uses the registry to dispatch commands during frame rendering
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::renderer::drawer::PipelineRegistry;
///
/// // Create registry and register pipelines
/// let mut registry = PipelineRegistry::new();
/// registry.register(my_shape_pipeline);
/// registry.register(my_text_pipeline);
/// registry.register(my_image_pipeline);
///
/// // Registry is now ready for use by the renderer
/// ```
///
/// # Performance Considerations
///
/// - Pipeline lookup is O(n) where n is the number of registered pipelines
/// - Register frequently used pipelines first for better average performance
/// - Consider the order of registration based on command frequency
pub struct PipelineRegistry {
    pub(crate) pipelines: Vec<Box<dyn ErasedDrawablePipeline>>,
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineRegistry {
    /// Creates a new empty pipeline registry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tessera_ui::renderer::drawer::PipelineRegistry;
    ///
    /// let registry = PipelineRegistry::new();
    /// ```
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    /// Registers a new drawable pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a type-erased
    /// container that can be stored alongside other pipelines of different types.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The [`DrawCommand`] type this pipeline handles
    /// * `P` - The pipeline implementation type
    ///
    /// # Parameters
    ///
    /// * `pipeline` - The pipeline instance to register
    ///
    /// # Panics
    ///
    /// This method does not panic, but the registry will panic during dispatch
    /// if no pipeline is found for a given command type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::renderer::drawer::PipelineRegistry;
    ///
    /// let mut registry = PipelineRegistry::new();
    ///
    /// // Register a custom pipeline
    /// let my_pipeline = MyCustomPipeline::new(&device, &config, sample_count);
    /// registry.register(my_pipeline);
    ///
    /// // Register multiple pipelines
    /// registry.register(ShapePipeline::new(&device, &config, sample_count));
    /// registry.register(TextPipeline::new(&device, &config, sample_count));
    /// ```
    ///
    /// # Registration Order
    ///
    /// The order of registration can affect performance since pipeline lookup
    /// is performed linearly. Consider registering more frequently used pipelines first.
    pub fn register<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static>(
        &mut self,
        pipeline: P,
    ) {
        let erased = Box::new(DrawablePipelineImpl::<T, P> {
            pipeline,
            _marker: std::marker::PhantomData,
        });
        self.pipelines.push(erased);
    }

    pub(crate) fn begin_all_passes(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.begin_pass(gpu, gpu_queue, config, render_pass);
        }
    }

    pub(crate) fn end_all_passes(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.end_pass(gpu, gpu_queue, config, render_pass);
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        cmd: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            if pipeline.draw_erased(
                gpu,
                gpu_queue,
                config,
                render_pass,
                cmd,
                size,
                start_pos,
                scene_texture_view,
            ) {
                return;
            }
        }

        panic!(
            "No pipeline found for command {:?}",
            std::any::type_name_of_val(cmd)
        );
    }
}
