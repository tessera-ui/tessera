//! GPU compute pipeline system for Tessera UI framework.
//!
//! This module provides the infrastructure for GPU compute operations in Tessera,
//! enabling advanced visual effects and post-processing operations that would be
//! inefficient or impossible to achieve with traditional CPU-based approaches.
//!
//! # Architecture Overview
//!
//! The compute pipeline system is designed to work seamlessly with the rendering
//! pipeline, using a ping-pong buffer approach for efficient multi-pass operations.
//! Each compute pipeline processes a specific type of compute command and operates
//! on texture data using GPU compute shaders.
//!
//! ## Key Components
//!
//! - [`ComputablePipeline<C>`]: The main trait for implementing custom compute pipelines
//! - [`ComputePipelineRegistry`]: Manages and dispatches commands to registered compute pipelines
//! - [`ComputeResourceManager`]: Manages GPU buffers and resources for compute operations
//!
//! # Design Philosophy
//!
//! The compute pipeline system embraces WGPU's compute shader capabilities to enable:
//!
//! - **Advanced Post-Processing**: Blur, contrast adjustment, color grading, and other image effects
//! - **Parallel Processing**: Leverage GPU parallelism for computationally intensive operations
//! - **Real-Time Effects**: Achieve complex visual effects at interactive frame rates
//! - **Memory Efficiency**: Use GPU memory directly without CPU roundtrips
//!
//! # Ping-Pong Rendering
//!
//! The system uses a ping-pong approach where:
//!
//! 1. **Input Texture**: Contains the result from previous rendering or compute pass
//! 2. **Output Texture**: Receives the processed result from the current compute operation
//! 3. **Format Convention**: All textures use `wgpu::TextureFormat::Rgba8Unorm` for compatibility
//!
//! This approach enables efficient chaining of multiple compute operations without
//! intermediate CPU involvement.
//!
//! # Implementation Guide
//!
//! ## Creating a Custom Compute Pipeline
//!
//! To create a custom compute pipeline:
//!
//! 1. Define your compute command struct implementing [`ComputeCommand`]
//! 2. Create a pipeline struct implementing [`ComputablePipeline<YourCommand>`]
//! 3. Write a compute shader in WGSL
//! 4. Register the pipeline with [`ComputePipelineRegistry::register`]
//!
//! ## Example: Simple Brightness Adjustment Pipeline
//!
//! ```rust,ignore
//! use tessera_ui::{ComputeCommand, ComputablePipeline, compute::resource::ComputeResourceManager};
//! use wgpu;
//!
//! // 1. Define the compute command
//! #[derive(Debug)]
//! struct BrightnessCommand {
//!     brightness: f32,
//! }
//!
//! impl ComputeCommand for BrightnessCommand {}
//!
//! // 2. Implement the pipeline
//! struct BrightnessPipeline {
//!     compute_pipeline: wgpu::ComputePipeline,
//!     bind_group_layout: wgpu::BindGroupLayout,
//! }
//!
//! impl BrightnessPipeline {
//!     fn new(device: &wgpu::Device) -> Self {
//!         // Create compute shader and pipeline
//!         let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//!             label: Some("Brightness Shader"),
//!             source: wgpu::ShaderSource::Wgsl(include_str!("brightness.wgsl").into()),
//!         });
//!         
//!         // ... setup bind group layout and pipeline ...
//!         # unimplemented!()
//!     }
//! }
//!
//! impl ComputablePipeline<BrightnessCommand> for BrightnessPipeline {
//!     fn dispatch(
//!         &mut self,
//!         device: &wgpu::Device,
//!         queue: &wgpu::Queue,
//!         config: &wgpu::SurfaceConfiguration,
//!         compute_pass: &mut wgpu::ComputePass<'_>,
//!         command: &BrightnessCommand,
//!         resource_manager: &mut ComputeResourceManager,
//!         input_view: &wgpu::TextureView,
//!         output_view: &wgpu::TextureView,
//!     ) {
//!         // Create uniforms buffer with brightness value
//!         let uniforms = [command.brightness];
//!         let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//!             label: Some("Brightness Uniforms"),
//!             contents: bytemuck::cast_slice(&uniforms),
//!             usage: wgpu::BufferUsages::UNIFORM,
//!         });
//!         
//!         // Create bind group with input/output textures and uniforms
//!         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//!             layout: &self.bind_group_layout,
//!             entries: &[
//!                 wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
//!                 wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(input_view) },
//!                 wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(output_view) },
//!             ],
//!             label: Some("brightness_bind_group"),
//!         });
//!         
//!         // Dispatch compute shader
//!         compute_pass.set_pipeline(&self.compute_pipeline);
//!         compute_pass.set_bind_group(0, &bind_group, &[]);
//!         compute_pass.dispatch_workgroups(
//!             (config.width + 7) / 8,
//!             (config.height + 7) / 8,
//!             1
//!         );
//!     }
//! }
//!
//! // 3. Register the pipeline
//! let mut registry = ComputePipelineRegistry::new();
//! let brightness_pipeline = BrightnessPipeline::new(&device);
//! registry.register(brightness_pipeline);
//! ```
//!
//! ## Example WGSL Compute Shader
//!
//! ```wgsl
//! @group(0) @binding(0) var<uniform> brightness: f32;
//! @group(0) @binding(1) var input_texture: texture_2d<f32>;
//! @group(0) @binding(2) var output_texture: texture_storage_2d<rgba8unorm, write>;
//!
//! @compute @workgroup_size(8, 8)
//! fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//!     let coords = vec2<i32>(global_id.xy);
//!     let input_color = textureLoad(input_texture, coords, 0);
//!     let output_color = vec4<f32>(input_color.rgb * brightness, input_color.a);
//!     textureStore(output_texture, coords, output_color);
//! }
//! ```
//!
//! # Integration with Basic Components
//!
//! The `tessera_basic_components` crate provides several compute pipeline implementations:
//!
//! - **BlurPipeline**: Gaussian blur effects for backgrounds and UI elements
//! - **MeanPipeline**: Average color calculation for adaptive UI themes
//! - **ContrastPipeline**: Contrast and saturation adjustments
//!
//! These pipelines demonstrate real-world usage patterns and can serve as references
//! for implementing custom compute operations.
//!
//! # Performance Considerations
//!
//! - **Workgroup Size**: Choose workgroup sizes that align with GPU architecture (typically 8x8 or 16x16)
//! - **Memory Access**: Optimize memory access patterns in shaders for better cache utilization
//! - **Resource Reuse**: Use the [`ComputeResourceManager`] to reuse buffers across frames
//! - **Batch Operations**: Combine multiple similar operations when possible
//!
//! # Texture Format Requirements
//!
//! Due to WGPU limitations, compute shaders require specific texture formats:
//!
//! - **Input Textures**: Can be any readable format, typically from render passes
//! - **Output Textures**: Must use `wgpu::TextureFormat::Rgba8Unorm` for storage binding
//! - **sRGB Limitation**: sRGB formats cannot be used as storage textures
//!
//! The framework automatically handles format conversions when necessary.

use crate::compute::resource::ComputeResourceManager;

use super::command::ComputeCommand;

/// Core trait for implementing GPU compute pipelines.
///
/// This trait defines the interface for compute pipelines that process specific types
/// of compute commands using GPU compute shaders. Each pipeline is responsible for
/// setting up compute resources, managing shader dispatch, and processing texture data.
///
/// # Type Parameters
///
/// * `C` - The specific [`ComputeCommand`] type this pipeline can handle
///
/// # Design Principles
///
/// - **Single Responsibility**: Each pipeline handles one specific type of compute operation
/// - **Stateless Operation**: Pipelines should not maintain state between dispatch calls
/// - **Resource Efficiency**: Reuse GPU resources when possible through the resource manager
/// - **Thread Safety**: All implementations must be `Send + Sync` for parallel execution
///
/// # Integration with Rendering
///
/// Compute pipelines operate within the broader rendering pipeline, typically:
///
/// 1. **After Rendering**: Process the rendered scene for post-effects
/// 2. **Between Passes**: Transform data between different rendering stages
/// 3. **Before Rendering**: Prepare data or textures for subsequent render operations
///
/// # Example Implementation Pattern
///
/// ```rust,ignore
/// impl ComputablePipeline<MyCommand> for MyPipeline {
///     fn dispatch(&mut self, device, queue, config, compute_pass, command,
///                 resource_manager, input_view, output_view) {
///         // 1. Create or retrieve uniform buffer
///         let uniforms = create_uniforms_from_command(command);
///         let uniform_buffer = device.create_buffer_init(...);
///         
///         // 2. Create bind group with textures and uniforms
///         let bind_group = device.create_bind_group(...);
///         
///         // 3. Set pipeline and dispatch
///         compute_pass.set_pipeline(&self.compute_pipeline);
///         compute_pass.set_bind_group(0, &bind_group, &[]);
///         compute_pass.dispatch_workgroups(workgroup_x, workgroup_y, 1);
///     }
/// }
/// ```
pub trait ComputablePipeline<C: ComputeCommand>: Send + Sync + 'static {
    /// Dispatches the compute command within an active compute pass.
    ///
    /// This method is called once for each compute command that needs to be processed.
    /// It should set up the necessary GPU resources, bind them to the compute pipeline,
    /// and dispatch the appropriate number of workgroups to process the input texture.
    ///
    /// # Parameters
    ///
    /// * `device` - The WGPU device for creating GPU resources
    /// * `queue` - The WGPU queue for submitting commands and updating buffers
    /// * `config` - Current surface configuration containing dimensions and format info
    /// * `compute_pass` - The active compute pass to record commands into
    /// * `command` - The specific compute command containing operation parameters
    /// * `resource_manager` - Manager for reusing GPU buffers across operations
    /// * `input_view` - View of the input texture (result from previous pass)
    /// * `output_view` - View of the output texture (target for this operation)
    ///
    /// # Texture Format Requirements
    ///
    /// Due to WGPU limitations, storage textures have specific format requirements:
    ///
    /// - **Input Texture**: Can be any readable format, typically from render passes
    /// - **Output Texture**: Must use `wgpu::TextureFormat::Rgba8Unorm` format
    /// - **sRGB Limitation**: sRGB formats cannot be used as storage textures
    ///
    /// The framework ensures that `output_view` always uses a compatible format
    /// for storage binding operations.
    ///
    /// # Workgroup Dispatch Guidelines
    ///
    /// When dispatching workgroups, consider:
    ///
    /// - **Workgroup Size**: Match your shader's `@workgroup_size` declaration
    /// - **Coverage**: Ensure all pixels are processed by calculating appropriate dispatch dimensions
    /// - **Alignment**: Round up dispatch dimensions to cover the entire texture
    ///
    /// Common dispatch pattern:
    /// ```rust,ignore
    /// let workgroup_size = 8; // Match shader @workgroup_size(8, 8)
    /// let dispatch_x = (config.width + workgroup_size - 1) / workgroup_size;
    /// let dispatch_y = (config.height + workgroup_size - 1) / workgroup_size;
    /// compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
    /// ```
    ///
    /// # Resource Management
    ///
    /// Use the `resource_manager` to:
    /// - Store persistent buffers that can be reused across frames
    /// - Avoid recreating expensive GPU resources
    /// - Manage buffer lifetimes efficiently
    ///
    /// # Error Handling
    ///
    /// This method should handle errors gracefully:
    /// - Validate command parameters before use
    /// - Ensure texture dimensions are compatible
    /// - Handle resource creation failures appropriately
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &C,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    );
}

/// Internal trait for type erasure of computable pipelines.
///
/// This trait enables dynamic dispatch of compute commands to their corresponding pipelines
/// without knowing the specific command type at compile time. It's used internally by
/// the [`ComputePipelineRegistry`] and should not be implemented directly by users.
///
/// The type erasure is achieved through the [`AsAny`] trait, which allows downcasting
/// from `&dyn ComputeCommand` to concrete command types.
///
/// # Implementation Note
///
/// This trait is automatically implemented for any type that implements
/// [`ComputablePipeline<C>`] through the [`ComputablePipelineImpl`] wrapper.
pub(crate) trait ErasedComputablePipeline: Send + Sync {
    /// Dispatches a type-erased compute command.
    fn dispatch_erased(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &dyn ComputeCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    );
}

/// A wrapper to implement `ErasedComputablePipeline` for any `ComputablePipeline`.
struct ComputablePipelineImpl<C: ComputeCommand, P: ComputablePipeline<C>> {
    pipeline: P,
    _command: std::marker::PhantomData<C>,
}

impl<C: ComputeCommand + 'static, P: ComputablePipeline<C>> ErasedComputablePipeline
    for ComputablePipelineImpl<C, P>
{
    fn dispatch_erased(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &dyn ComputeCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        if let Some(command) = command.as_any().downcast_ref::<C>() {
            self.pipeline.dispatch(
                device,
                queue,
                config,
                compute_pass,
                command,
                resource_manager,
                input_view,
                output_view,
            );
        }
    }
}

/// Registry for managing and dispatching compute pipelines.
///
/// The `ComputePipelineRegistry` serves as the central hub for all compute pipelines
/// in the Tessera framework. It maintains a collection of registered pipelines and
/// handles the dispatch of compute commands to their appropriate pipelines.
///
/// # Architecture
///
/// The registry uses type erasure to store pipelines of different types in a single
/// collection. When a compute command needs to be processed, the registry attempts
/// to dispatch it to all registered pipelines until one handles it successfully.
///
/// # Usage Pattern
///
/// 1. Create a new registry
/// 2. Register all required compute pipelines during application initialization
/// 3. The renderer uses the registry to dispatch commands during frame rendering
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::renderer::compute::ComputePipelineRegistry;
///
/// // Create registry and register pipelines
/// let mut registry = ComputePipelineRegistry::new();
/// registry.register(blur_pipeline);
/// registry.register(contrast_pipeline);
/// registry.register(brightness_pipeline);
///
/// // Registry is now ready for use by the renderer
/// ```
///
/// # Performance Considerations
///
/// - All registered pipelines are called for each command until one handles it
/// - Register more commonly used pipelines first for better average performance
/// - Consider the frequency of different compute operations when ordering registrations
///
/// # Thread Safety
///
/// The registry and all registered pipelines must be `Send + Sync` to support
/// parallel execution in the rendering system.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    pipelines: Vec<Box<dyn ErasedComputablePipeline>>,
}

impl ComputePipelineRegistry {
    /// Creates a new empty compute pipeline registry.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui::renderer::compute::ComputePipelineRegistry;
    ///
    /// let registry = ComputePipelineRegistry::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new compute pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a type-erased
    /// container that can be stored alongside other pipelines of different types.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The [`ComputeCommand`] type this pipeline handles
    ///
    /// # Parameters
    ///
    /// * `pipeline` - The pipeline instance to register
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tessera_ui::renderer::compute::ComputePipelineRegistry;
    ///
    /// let mut registry = ComputePipelineRegistry::new();
    ///
    /// // Register custom compute pipelines
    /// let blur_pipeline = BlurPipeline::new(&device);
    /// registry.register(blur_pipeline);
    ///
    /// let contrast_pipeline = ContrastPipeline::new(&device);
    /// registry.register(contrast_pipeline);
    ///
    /// // Register multiple pipelines for different effects
    /// registry.register(BrightnessAdjustmentPipeline::new(&device));
    /// registry.register(ColorGradingPipeline::new(&device));
    /// ```
    ///
    /// # Registration Order
    ///
    /// Unlike drawable pipelines, compute pipelines are dispatched to all registered
    /// pipelines until one handles the command. Consider registering more frequently
    /// used pipelines first for better performance.
    ///
    /// # Thread Safety
    ///
    /// The pipeline must implement `Send + Sync` to be compatible with Tessera's
    /// parallel rendering architecture.
    pub fn register<C: ComputeCommand + 'static>(
        &mut self,
        pipeline: impl ComputablePipeline<C> + 'static,
    ) {
        let erased_pipeline = Box::new(ComputablePipelineImpl {
            pipeline,
            _command: std::marker::PhantomData,
        });
        self.pipelines.push(erased_pipeline);
    }

    /// Dispatches a command to its corresponding registered pipeline.
    pub(crate) fn dispatch_erased(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &dyn ComputeCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.dispatch_erased(
                device,
                queue,
                config,
                compute_pass,
                command,
                resource_manager,
                input_view,
                output_view,
            );
        }
    }
}
