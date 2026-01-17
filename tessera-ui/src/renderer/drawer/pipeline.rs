//! Graphics rendering pipeline system for Tessera UI framework.
//!
//! This module provides the core infrastructure for pluggable graphics
//! rendering pipelines in Tessera. The design philosophy emphasizes flexibility
//! and extensibility, allowing developers to create custom rendering effects
//! without being constrained by built-in drawing primitives.
//!
//! # Architecture Overview
//!
//! The pipeline system uses a trait-based approach with type erasure to support
//! dynamic dispatch of rendering commands. Each pipeline is responsible for
//! rendering a specific type of draw command, such as shapes, text, images, or
//! custom visual effects.
//!
//! ## Key Components
//!
//! - [`DrawablePipeline<T>`]: The main trait for implementing custom rendering
//!   pipelines
//! - [`PipelineRegistry`]: Manages and dispatches commands to registered
//!   pipelines
//!
//! # Design Philosophy
//!
//! Unlike traditional UI frameworks that provide built-in "brush" or drawing
//! primitives, Tessera treats shaders as first-class citizens. This approach
//! offers several advantages:
//!
//! - **Modern GPU Utilization**: Leverages WGPU and WGSL for efficient,
//!   cross-platform rendering
//! - **Advanced Visual Effects**: Enables complex effects like neumorphic
//!   design, lighting, shadows, reflections, and bloom that are difficult to
//!   achieve with traditional approaches
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
//! # Integration with Basic Components
//!
//! The `tessera_basic_components` crate demonstrates real-world pipeline
//! implementations:
//!
//! - **ShapePipeline**: Renders rounded rectangles, circles, and complex shapes
//!   with shadows and ripple effects
//! - **TextPipeline**: Handles text rendering with font management and glyph
//!   caching
//! - **ImagePipeline**: Displays images with various scaling and filtering
//!   options
//! - **FluidGlassPipeline**: Creates advanced glass effects with distortion and
//!   transparency
//!
//! These pipelines are registered in `tessera_components::init(...)`.
//!
//! # Performance Considerations
//!
//! - **Batch Similar Commands**: Group similar draw commands to minimize
//!   pipeline switches
//! - **Resource Management**: Reuse buffers and textures when possible
//! - **Shader Optimization**: Write efficient shaders optimized for your target
//!   platforms
//! - **State Changes**: Minimize render state changes within the draw method
//!
//! # Advanced Features
//!
//! ## Barrier Requirements
//!
//! Some rendering effects need to sample from previously rendered content
//! (e.g., blur effects). Implement [`DrawCommand::barrier()`] to return
//! `SampleBackground` requirements for such commands.
//!
//! ## Multi-Pass Rendering
//!
//! Use `begin_pass()` and `end_pass()` for pipelines that require multiple
//! rendering passes or complex setup/teardown operations.
//!
//! ## Scene Texture Access
//!
//! The `scene_texture_view` parameter provides access to the current scene
//! texture, enabling effects that sample from the background or perform
//! post-processing.

use std::{any::TypeId, collections::HashMap};

use crate::{
    px::{PxPosition, PxRect, PxSize},
    renderer::DrawCommand,
};

/// Provides context for operations that occur once per frame.
///
/// This struct bundles essential WGPU resources and configuration that are
/// relevant for the entire rendering frame, but are not specific to a single
/// render pass.
pub struct FrameContext<'a> {
    /// The WGPU device.
    pub device: &'a wgpu::Device,
    /// The WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// The current surface configuration.
    pub config: &'a wgpu::SurfaceConfiguration,
}

/// Provides context for operations within a single render pass.
///
/// This struct bundles WGPU resources and configuration specific to a render
/// pass, including the active render pass encoder and the scene texture view
/// for sampling.
pub struct PassContext<'a, 'b> {
    /// The WGPU device.
    pub device: &'a wgpu::Device,
    /// The WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// The current surface configuration.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the current pass.
    pub target_size: PxSize,
    /// The active render pass encoder.
    pub render_pass: &'a mut wgpu::RenderPass<'b>,
    /// A view of the current scene texture.
    pub scene_texture_view: &'a wgpu::TextureView,
}

/// Provides comprehensive context for drawing operations within a render pass.
///
/// This struct extends `PassContext` with information specific to individual
/// draw calls, including the commands to be rendered and an optional clipping
/// rectangle.
///
/// # Type Parameters
///
/// * `T` - The specific [`DrawCommand`] type being processed.
///
/// # Fields
///
/// * `device` - The WGPU device, used for creating and managing GPU resources.
/// * `queue` - The WGPU queue, used for submitting command buffers and writing
///   buffer data.
/// * `config` - The current surface configuration, providing information like
///   format and dimensions.
/// * `render_pass` - The active `wgpu::RenderPass` encoder, used to record
///   rendering commands.
/// * `commands` - A slice of tuples, each containing a draw command, its size,
///   and its position.
/// * `scene_texture_view` - A view of the current scene texture, useful for
///   effects that sample from the background.
/// * `clip_rect` - An optional rectangle defining the clipping area for the
///   draw call.
pub struct DrawContext<'a, 'b, 'c, T> {
    /// The WGPU device.
    pub device: &'a wgpu::Device,
    /// The WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// The current surface configuration.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the current pass.
    pub target_size: PxSize,
    /// The active render pass encoder.
    pub render_pass: &'a mut wgpu::RenderPass<'b>,
    /// The draw commands to be processed.
    pub commands: &'c [(&'c T, PxSize, PxPosition)],
    /// A view of the current scene texture.
    pub scene_texture_view: &'a wgpu::TextureView,
    /// An optional clipping rectangle for the draw call.
    pub clip_rect: Option<PxRect>,
}

/// Type-erased context used for dispatching draw pipelines.
pub struct ErasedDrawContext<'a, 'b> {
    /// WGPU device used for pipeline resource access.
    pub device: &'a wgpu::Device,
    /// WGPU queue used for submissions.
    pub queue: &'a wgpu::Queue,
    /// Current surface configuration for the render target.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the current pass.
    pub target_size: PxSize,
    /// Active render pass that receives draw calls.
    pub render_pass: &'a mut wgpu::RenderPass<'b>,
    /// Scene texture view available for sampling.
    pub scene_texture_view: &'a wgpu::TextureView,
    /// Optional clipping rectangle applied to the submission.
    pub clip_rect: Option<PxRect>,
}

/// Core trait for implementing custom graphics rendering pipelines.
///
/// This trait defines the interface for rendering pipelines that process
/// specific types of draw commands. Each pipeline is responsible for setting up
/// GPU resources, managing render state, and executing the actual drawing
/// operations.
///
/// # Type Parameters
///
/// * `T` - The specific [`DrawCommand`] type this pipeline can handle
///
/// # Lifecycle Methods
///
/// The pipeline system provides five lifecycle hooks, executed in the following
/// order:
///
/// 1. [`begin_frame()`](Self::begin_frame): Called once at the start of a new
///    frame, before any render passes.
/// 2. [`begin_pass()`](Self::begin_pass): Called at the start of each render
///    pass that involves this pipeline.
/// 3. [`draw()`](Self::draw): Called for each command of type `T` within a
///    render pass.
/// 4. [`end_pass()`](Self::end_pass): Called at the end of each render pass
///    that involved this pipeline.
/// 5. [`end_frame()`](Self::end_frame): Called once at the end of the frame,
///    after all render passes are complete.
///
/// Typically, `begin_pass`, `draw`, and `end_pass` are used for the core
/// rendering logic within a pass, while `begin_frame` and `end_frame` are used
/// for setup and teardown that spans the entire frame.
///
/// # Implementation Notes
///
/// - Only the [`draw()`](Self::draw) method is required; others have default
///   empty implementations.
/// - Pipelines should be stateless between frames when possible
/// - Resource management should prefer reuse over recreation
/// - Consider batching multiple commands for better performance
///
/// # Example
///
/// See the module-level documentation for a complete implementation example.
#[allow(unused_variables)]
pub trait DrawablePipeline<T: DrawCommand> {
    /// Called once at the beginning of the frame, before any render passes.
    ///
    /// This method is the first hook in the pipeline's frame lifecycle. It's
    /// invoked after a new `CommandEncoder` has been created but before any
    /// rendering occurs. It's ideal for per-frame setup that is not tied to
    /// a specific `wgpu::RenderPass`.
    ///
    /// Since this method is called outside a render pass, it cannot be used for
    /// drawing commands. However, it can be used for operations like:
    ///
    /// - Updating frame-global uniform buffers (e.g., with time or resolution
    ///   data) using [`wgpu::Queue::write_buffer`].
    /// - Preparing or resizing buffers that will be used throughout the frame.
    /// - Performing CPU-side calculations needed for the frame.
    ///
    /// # Parameters
    ///
    /// * `context` - The context for the frame.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing.
    fn begin_frame(&mut self, context: &FrameContext<'_>) {}

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
    /// * `context` - The context for the render pass.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing, which is suitable for most
    /// pipelines.
    fn begin_pass(&mut self, context: &mut PassContext<'_, '_>) {}

    /// Renders a batch of draw commands.
    ///
    /// This is the core method where the actual rendering happens. It's called
    /// once for a batch of draw commands of type `T` that need to be rendered.
    ///
    /// # Parameters
    ///
    /// * `context` - The context for drawing, including the render pass and
    ///   commands.
    ///
    /// # Implementation Guidelines
    ///
    /// - Iterate over the `context.commands` slice to process each command.
    /// - Update buffers (e.g., instance buffers, storage buffers) with data
    ///   from the command batch.
    /// - Set the appropriate render pipeline.
    /// - Bind necessary resources (textures, buffers, bind groups).
    /// - Issue one or more draw calls (e.g., an instanced draw call) to render
    ///   the entire batch.
    /// - If `context.clip_rect` is `Some`, use
    ///   `context.render_pass.set_scissor_rect()` to clip rendering.
    /// - Avoid expensive operations like buffer creation; prefer reusing and
    ///   updating existing resources.
    ///
    /// # Scene Texture Usage
    ///
    /// The `context.scene_texture_view` provides access to the current rendered
    /// scene, enabling effects that sample from the background.
    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, T>);

    /// Called once at the end of the render pass.
    ///
    /// Use this method to perform cleanup operations or finalize rendering
    /// for all draw commands of this type in the current frame. This is useful
    /// for:
    ///
    /// - Cleaning up temporary resources
    /// - Finalizing multi-pass rendering operations
    /// - Submitting batched draw calls
    ///
    /// # Parameters
    ///
    /// * `context` - The context for the render pass.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing, which is suitable for most
    /// pipelines.
    fn end_pass(&mut self, context: &mut PassContext<'_, '_>) {}

    /// Called once at the end of the frame, after all render passes are
    /// complete.
    ///
    /// This method is the final hook in the pipeline's frame lifecycle. It's
    /// invoked after all `begin_pass`, `draw`, and `end_pass` calls for the
    /// frame have completed, but before the frame's command buffer is
    /// submitted to the GPU.
    ///
    /// It's suitable for frame-level cleanup or finalization tasks, such as:
    ///
    /// - Reading data back from the GPU (though this can be slow and should be
    ///   used sparingly).
    /// - Cleaning up temporary resources created in `begin_frame`.
    /// - Preparing data for the next frame.
    ///
    /// # Parameters
    ///
    /// * `context` - The context for the frame.
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing.
    fn end_frame(&mut self, context: &FrameContext<'_>) {}
}

/// Internal trait for type erasure of drawable pipelines.
///
/// This trait enables dynamic dispatch of draw commands to their corresponding
/// pipelines without knowing the specific command type at compile time. It's
/// used internally by the [`PipelineRegistry`] and should not be implemented
/// directly by users.
///
/// The type erasure is achieved through the [`AsAny`] trait, which allows
/// downcasting from `&dyn DrawCommand` to concrete command types.
///
/// # Implementation Note
///
/// This trait is automatically implemented for any type that implements
/// [`DrawablePipeline<T>`] through the [`DrawablePipelineImpl`] wrapper.
pub(crate) trait ErasedDrawablePipeline {
    /// Called at the beginning of a frame to prepare pipeline resources.
    fn begin_frame(&mut self, context: &FrameContext<'_>);
    /// Called at the end of a frame for cleanup or readback.
    fn end_frame(&mut self, context: &FrameContext<'_>);
    /// Invoked before a render pass starts to bind shared pass state.
    fn begin_pass(&mut self, context: &mut PassContext<'_, '_>);
    /// Invoked after a render pass ends to finalize pass-level resources.
    fn end_pass(&mut self, context: &mut PassContext<'_, '_>);
    /// Draws a batch of commands with type-erased dispatch.
    fn draw_erased(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, PxSize, PxPosition)],
    ) -> bool;
}

struct DrawablePipelineImpl<T: DrawCommand, P: DrawablePipeline<T>> {
    pipeline: P,
    _marker: std::marker::PhantomData<T>,
}

impl<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static> ErasedDrawablePipeline
    for DrawablePipelineImpl<T, P>
{
    fn begin_frame(&mut self, context: &FrameContext<'_>) {
        self.pipeline.begin_frame(context);
    }

    fn end_frame(&mut self, context: &FrameContext<'_>) {
        self.pipeline.end_frame(context);
    }

    fn begin_pass(&mut self, context: &mut PassContext<'_, '_>) {
        self.pipeline.begin_pass(context);
    }

    fn end_pass(&mut self, context: &mut PassContext<'_, '_>) {
        self.pipeline.end_pass(context);
    }

    fn draw_erased(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, PxSize, PxPosition)],
    ) -> bool {
        if commands.is_empty() {
            return true;
        }

        let ErasedDrawContext {
            device,
            queue,
            config,
            target_size,
            render_pass,
            scene_texture_view,
            clip_rect,
        } = context;

        if commands[0].0.as_any().is::<T>() {
            let typed_commands: Vec<(&T, PxSize, PxPosition)> = commands
                .iter()
                .map(|(cmd, size, pos)| {
                    (
                        cmd.as_any().downcast_ref::<T>().expect(
                            "FATAL: A command in a batch has a different type than the first one.",
                        ),
                        *size,
                        *pos,
                    )
                })
                .collect();

            self.pipeline.draw(&mut DrawContext {
                device,
                queue,
                config,
                target_size,
                render_pass,
                commands: &typed_commands,
                scene_texture_view,
                clip_rect,
            });
            true
        } else {
            false
        }
    }
}

/// Registry for managing and dispatching drawable pipelines.
///
/// The `PipelineRegistry` serves as the central hub for all rendering pipelines
/// in the Tessera framework. It maintains a collection of registered pipelines
/// and handles the dispatch of draw commands to their appropriate pipelines.
///
/// # Architecture
///
/// The registry uses type erasure to store pipelines of different types in a
/// single collection. When a draw command needs to be rendered, the registry
/// iterates through all registered pipelines until it finds one that can handle
/// the command type.
///
/// # Usage Pattern
///
/// 1. Create a new registry
/// 2. Register all required pipelines during application initialization
/// 3. The renderer uses the registry to dispatch commands during frame
///    rendering
pub struct PipelineRegistry {
    pub(crate) pipelines: HashMap<TypeId, Box<dyn ErasedDrawablePipeline>>,
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
    /// ```
    /// use tessera_ui::renderer::drawer::PipelineRegistry;
    ///
    /// let registry = PipelineRegistry::new();
    /// ```
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    /// Registers a new drawable pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a
    /// type-erased container that can be stored alongside other pipelines
    /// of different types.
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
    pub fn register<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static>(
        &mut self,
        pipeline: P,
    ) {
        let erased = Box::new(DrawablePipelineImpl::<T, P> {
            pipeline,
            _marker: std::marker::PhantomData,
        });
        self.pipelines.insert(TypeId::of::<T>(), erased);
    }

    pub(crate) fn begin_all_passes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        scene_texture_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.values_mut() {
            pipeline.begin_pass(&mut PassContext {
                device,
                queue,
                config,
                target_size,
                render_pass,
                scene_texture_view,
            });
        }
    }

    pub(crate) fn end_all_passes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        scene_texture_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.values_mut() {
            pipeline.end_pass(&mut PassContext {
                device,
                queue,
                config,
                target_size,
                render_pass,
                scene_texture_view,
            });
        }
    }

    pub(crate) fn begin_all_frames(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        for pipeline in self.pipelines.values_mut() {
            pipeline.begin_frame(&FrameContext {
                device,
                queue,
                config,
            });
        }
    }

    pub(crate) fn end_all_frames(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        for pipeline in self.pipelines.values_mut() {
            pipeline.end_frame(&FrameContext {
                device,
                queue,
                config,
            });
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, PxSize, PxPosition)],
    ) {
        if commands.is_empty() {
            return;
        }

        let command_type_id = commands[0].0.as_any().type_id();
        if let Some(pipeline) = self.pipelines.get_mut(&command_type_id) {
            if !pipeline.draw_erased(context, commands) {
                panic!(
                    "FATAL: A command in a batch has a different type than the first one. This should not happen."
                )
            }
        } else {
            panic!(
                "No pipeline found for command {:?}",
                std::any::type_name_of_val(commands[0].0)
            );
        }
    }
}
