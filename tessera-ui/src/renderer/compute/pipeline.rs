//! GPU compute pipeline system for Tessera UI framework.
//!
//! This module provides the infrastructure for GPU compute operations in
//! Tessera, enabling advanced visual effects and post-processing operations
//! that would be inefficient or impossible to achieve with traditional
//! CPU-based approaches.
//!
//! # Architecture Overview
//!
//! The compute pipeline system is designed to work seamlessly with the
//! rendering pipeline, using a ping-pong buffer approach for efficient
//! multi-pass operations. Each compute pipeline processes a specific type of
//! compute command and operates on texture data using GPU compute shaders.
//!
//! ## Key Components
//!
//! - [`ComputablePipeline<C>`]: The main trait for implementing custom compute
//!   pipelines
//! - [`ComputePipelineRegistry`]: Manages and dispatches commands to registered
//!   compute pipelines
//! - [`ComputeResourceManager`]: Manages GPU buffers and resources for compute
//!   operations
//!
//! # Design Philosophy
//!
//! The compute pipeline system embraces WGPU's compute shader capabilities to
//! enable:
//!
//! - **Advanced Post-Processing**: Blur, contrast adjustment, color grading,
//!   and other image effects
//! - **Parallel Processing**: Leverage GPU parallelism for computationally
//!   intensive operations
//! - **Real-Time Effects**: Achieve complex visual effects at interactive frame
//!   rates
//! - **Memory Efficiency**: Use GPU memory directly without CPU roundtrips
//!
//! # Ping-Pong Rendering
//!
//! The system uses a ping-pong approach where:
//!
//! 1. **Input Texture**: Contains the result from previous rendering or compute
//!    pass
//! 2. **Output Texture**: Receives the processed result from the current
//!    compute operation
//! 3. **Format Convention**: All textures use `wgpu::TextureFormat::Rgba8Unorm`
//!    for compatibility
//!
//! This approach enables efficient chaining of multiple compute operations
//! without intermediate CPU involvement.
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
//! # Performance Considerations
//!
//! - **Workgroup Size**: Choose workgroup sizes that align with GPU
//!   architecture (typically 8x8 or 16x16)
//! - **Memory Access**: Optimize memory access patterns in shaders for better
//!   cache utilization
//! - **Resource Reuse**: Use the [`ComputeResourceManager`] to reuse buffers
//!   across frames
//! - **Batch Operations**: Combine multiple similar operations when possible
//!
//! # Texture Format Requirements
//!
//! Due to WGPU limitations, compute shaders require specific texture formats:
//!
//! - **Input Textures**: Can be any readable format, typically from render
//!   passes
//! - **Output Textures**: Must use `wgpu::TextureFormat::Rgba8Unorm` for
//!   storage binding
//! - **sRGB Limitation**: sRGB formats cannot be used as storage textures
//!
//! The framework automatically handles format conversions when necessary.

use std::{any::TypeId, collections::HashMap};

use crate::{PxPosition, PxRect, PxSize, compute::resource::ComputeResourceManager};

use super::command::ComputeCommand;

/// Type-erased metadata describing a compute command within a batch.
pub struct ErasedComputeBatchItem<'a> {
    /// The compute command to execute.
    pub command: &'a dyn ComputeCommand,
    /// The measured size of the target region.
    pub size: PxSize,
    /// The absolute position of the target region.
    pub position: PxPosition,
    /// The rectangle of the content that will be written.
    pub target_area: PxRect,
}

/// Strongly typed metadata describing a compute command within a batch.
pub struct ComputeBatchItem<'a, C: ComputeCommand> {
    /// The compute command to execute.
    pub command: &'a C,
    /// The measured size of the target region.
    pub size: PxSize,
    /// The absolute position of the target region.
    pub position: PxPosition,
    /// The rectangle of the content that will be written.
    pub target_area: PxRect,
}

/// Provides comprehensive context for compute operations within a compute pass.
///
/// This struct bundles essential WGPU resources, configuration, and
/// command-specific data required for a compute pipeline to process its
/// commands.
///
/// # Type Parameters
///
/// * `C` - The specific [`ComputeCommand`] type being processed.
///
/// # Fields
///
/// * `device` - The WGPU device, used for creating and managing GPU resources.
/// * `queue` - The WGPU queue, used for submitting command buffers and writing
///   buffer data.
/// * `config` - The current surface configuration, providing information like
///   format and dimensions.
/// * `compute_pass` - The active `wgpu::ComputePass` encoder, used to record
///   compute commands.
/// * `items` - A slice of [`ComputeBatchItem`]s, each containing a compute
///   command and its metadata.
/// * `resource_manager` - A mutable reference to the
///   [`ComputeResourceManager`], used for managing reusable GPU buffers.
/// * `input_view` - A view of the input texture for the compute operation.
/// * `output_view` - A view of the output texture for the compute operation.
pub struct ComputeContext<'a, 'b, 'c, C: ComputeCommand> {
    /// WGPU device used to create and manage GPU resources.
    pub device: &'a wgpu::Device,
    /// Queue for submitting GPU workloads.
    pub queue: &'a wgpu::Queue,
    /// Surface configuration describing output formats and dimensions.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the current compute pass.
    pub target_size: PxSize,
    /// Active compute pass encoder.
    pub compute_pass: &'a mut wgpu::ComputePass<'b>,
    /// Batch of typed compute items to process.
    pub items: &'c [ComputeBatchItem<'c, C>],
    /// Shared resource manager used to reuse GPU buffers.
    pub resource_manager: &'a mut ComputeResourceManager,
    /// Input texture view sampled by the compute pass.
    pub input_view: &'a wgpu::TextureView,
    /// Output texture view written by the compute pass.
    pub output_view: &'a wgpu::TextureView,
}

/// Type-erased context used when dispatching compute pipelines.
pub(crate) struct ErasedDispatchContext<'a, 'b> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub target_size: PxSize,
    pub compute_pass: &'a mut wgpu::ComputePass<'b>,
    pub resource_manager: &'a mut ComputeResourceManager,
    pub input_view: &'a wgpu::TextureView,
    pub output_view: &'a wgpu::TextureView,
}

/// Core trait for implementing GPU compute pipelines.
///
/// This trait defines the interface for compute pipelines that process specific
/// types of compute commands using GPU compute shaders. Each pipeline is
/// responsible for setting up compute resources, managing shader dispatch, and
/// processing texture data.
///
/// # Type Parameters
///
/// * `C` - The specific [`ComputeCommand`] type this pipeline can handle
///
/// # Design Principles
///
/// - **Single Responsibility**: Each pipeline handles one specific type of
///   compute operation
/// - **Stateless Operation**: Pipelines should not maintain state between
///   dispatch calls
/// - **Resource Efficiency**: Reuse GPU resources when possible through the
///   resource manager
/// - **Thread Safety**: All implementations must be `Send + Sync` for parallel
///   execution
///
/// # Integration with Rendering
///
/// Compute pipelines operate within the broader rendering pipeline, typically:
///
/// 1. **After Rendering**: Process the rendered scene for post-effects
/// 2. **Between Passes**: Transform data between different rendering stages
/// 3. **Before Rendering**: Prepare data or textures for subsequent render
///    operations
pub trait ComputablePipeline<C: ComputeCommand>: Send + Sync + 'static {
    /// Dispatches the compute command within an active compute pass.
    ///
    /// This method receives one or more compute commands of the same type.
    /// Implementations may choose to process the batch collectively (e.g.,
    /// by packing data into a single dispatch) or sequentially iterate over
    /// the items. It should set up the necessary GPU resources, bind them
    /// to the compute pipeline, and dispatch the appropriate number of
    /// workgroups to process the input texture.
    ///
    /// # Parameters
    ///
    /// * `context` - The context for the compute pass.
    ///
    /// # Texture Format Requirements
    ///
    /// Due to WGPU limitations, storage textures have specific format
    /// requirements:
    ///
    /// - **Input Texture**: Can be any readable format, typically from render
    ///   passes
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
    /// - **Coverage**: Ensure all pixels are processed by calculating
    ///   appropriate dispatch dimensions
    /// - **Alignment**: Round up dispatch dimensions to cover the entire
    ///   texture
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
    fn dispatch(&mut self, context: &mut ComputeContext<C>);
}

/// Internal trait for type erasure of computable pipelines.
///
/// This trait enables dynamic dispatch of compute commands to their
/// corresponding pipelines without knowing the specific command type at compile
/// time. It's used internally by the [`ComputePipelineRegistry`] and should not
/// be implemented directly by users.
///
/// The type erasure is achieved through the [`Downcast`] trait, which allows
/// downcasting from `&dyn ComputeCommand` to concrete command types.
///
/// # Implementation Note
///
/// This trait is automatically implemented for any type that implements
/// [`ComputablePipeline<C>`] through the [`ComputablePipelineImpl`] wrapper.
pub(crate) trait ErasedComputablePipeline: Send + Sync {
    /// Dispatches a type-erased compute command.
    fn dispatch_erased(
        &mut self,
        context: ErasedDispatchContext<'_, '_>,
        items: &[ErasedComputeBatchItem<'_>],
    );
}

/// A wrapper to implement `ErasedComputablePipeline` for any
/// `ComputablePipeline`.
struct ComputablePipelineImpl<C: ComputeCommand, P: ComputablePipeline<C>> {
    pipeline: P,
    _command: std::marker::PhantomData<C>,
}

impl<C: ComputeCommand + 'static, P: ComputablePipeline<C>> ErasedComputablePipeline
    for ComputablePipelineImpl<C, P>
{
    fn dispatch_erased(
        &mut self,
        context: ErasedDispatchContext<'_, '_>,
        items: &[ErasedComputeBatchItem<'_>],
    ) {
        if items.is_empty() {
            return;
        }

        let mut typed_items: Vec<ComputeBatchItem<'_, C>> = Vec::with_capacity(items.len());
        for item in items {
            let command = item
                .command
                .downcast_ref::<C>()
                .expect("Compute batch contained command of unexpected type");
            typed_items.push(ComputeBatchItem {
                command,
                size: item.size,
                position: item.position,
                target_area: item.target_area,
            });
        }

        self.pipeline.dispatch(&mut ComputeContext {
            device: context.device,
            queue: context.queue,
            config: context.config,
            target_size: context.target_size,
            compute_pass: context.compute_pass,
            items: &typed_items,
            resource_manager: context.resource_manager,
            input_view: context.input_view,
            output_view: context.output_view,
        });
    }
}

/// Registry for managing and dispatching compute pipelines.
///
/// The `ComputePipelineRegistry` serves as the central hub for all compute
/// pipelines in the Tessera framework. It maintains a collection of registered
/// pipelines and handles the dispatch of compute commands to their appropriate
/// pipelines.
///
/// # Architecture
///
/// The registry uses type erasure to store pipelines of different types in a
/// single collection. When a compute command needs to be processed, the
/// registry attempts to dispatch it to all registered pipelines until one
/// handles it successfully.
///
/// # Usage Pattern
///
/// 1. Create a new registry
/// 2. Register all required compute pipelines during application initialization
/// 3. The renderer uses the registry to dispatch commands during frame
///    rendering
///
/// # Performance Considerations
///
/// - Pipeline lookup is O(1) on average due to HashMap implementation.
///
/// # Thread Safety
///
/// The registry and all registered pipelines must be `Send + Sync` to support
/// parallel execution in the rendering system.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    pipelines: HashMap<TypeId, Box<dyn ErasedComputablePipeline>>,
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
    /// This method takes ownership of the pipeline and wraps it in a
    /// type-erased container that can be stored alongside other pipelines
    /// of different types.
    ///
    /// # Type Parameters
    ///
    /// * `C` - The [`ComputeCommand`] type this pipeline handles
    ///
    /// # Parameters
    ///
    /// * `pipeline` - The pipeline instance to register
    ///
    /// # Thread Safety
    ///
    /// The pipeline must implement `Send + Sync` to be compatible with
    /// Tessera's parallel rendering architecture.
    pub fn register<C: ComputeCommand + 'static>(
        &mut self,
        pipeline: impl ComputablePipeline<C> + 'static,
    ) {
        let erased_pipeline = Box::new(ComputablePipelineImpl {
            pipeline,
            _command: std::marker::PhantomData,
        });
        self.pipelines.insert(TypeId::of::<C>(), erased_pipeline);
    }

    /// Dispatches one or more commands to their corresponding registered
    /// pipeline.
    pub(crate) fn dispatch_erased(
        &mut self,
        context: ErasedDispatchContext<'_, '_>,
        items: &[ErasedComputeBatchItem<'_>],
    ) {
        if items.is_empty() {
            return;
        }

        let command_type_id = items[0].command.as_any().type_id();
        if let Some(pipeline) = self.pipelines.get_mut(&command_type_id) {
            pipeline.dispatch_erased(context, items);
        } else {
            panic!(
                "No pipeline found for command {:?}",
                std::any::type_name_of_val(items[0].command)
            );
        }
    }
}
