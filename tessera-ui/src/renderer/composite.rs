//! Composite command expansion for Tessera render graphs.
//!
//! ## Usage
//!
//! Expand composite commands into draw/compute ops before rendering.

use std::{any::TypeId, collections::HashMap};

use crate::{
    Command, CompositeCommand, PxPosition, PxSize,
    render_graph::{
        ExternalTextureDesc, RenderGraph, RenderGraphOp, RenderGraphParts, RenderResource,
        RenderResourceId,
    },
};

use super::core::RenderResources;
use super::external::ExternalTextureRegistry;

/// Context provided to composite pipelines during expansion.
pub struct CompositeContext<'a> {
    /// Shared GPU resources for pipeline usage.
    pub resources: RenderResources<'a>,
    /// Registry for external persistent textures.
    pub external_textures: ExternalTextureRegistry,
    /// Pixel size of the current frame.
    pub frame_size: PxSize,
    /// Surface format for the current frame.
    pub surface_format: wgpu::TextureFormat,
    /// MSAA sample count for the current frame.
    pub sample_count: u32,
    /// Monotonic frame index.
    pub frame_index: u64,
}

/// Type-erased metadata describing a composite command within a batch.
pub struct ErasedCompositeBatchItem<'a> {
    /// The composite command to expand.
    pub command: &'a dyn CompositeCommand,
    /// Measured size of the target region.
    pub size: PxSize,
    /// Absolute position of the target region.
    pub position: PxPosition,
    /// Opacity multiplier for the command.
    pub opacity: f32,
    /// Original sequence index for ordering.
    pub sequence_index: usize,
    /// Index of the op in the source render graph.
    pub op_index: usize,
}

/// Strongly typed metadata describing a composite command within a batch.
pub struct CompositeBatchItem<'a, C: CompositeCommand> {
    /// The composite command to expand.
    pub command: &'a C,
    /// Measured size of the target region.
    pub size: PxSize,
    /// Absolute position of the target region.
    pub position: PxPosition,
    /// Opacity multiplier for the command.
    pub opacity: f32,
    /// Original sequence index for ordering.
    pub sequence_index: usize,
    /// Index of the op in the source render graph.
    pub op_index: usize,
}

/// Replacement ops emitted for a composite command.
pub struct CompositeReplacement {
    /// Index of the original composite op to replace.
    pub target_op: usize,
    /// Ops that replace the composite command.
    pub ops: Vec<RenderGraphOp>,
}

/// Composite pipeline output for a batch.
pub struct CompositeOutput {
    /// Local resources referenced by prelude and replacement ops.
    pub resources: Vec<RenderResource>,
    /// External resources referenced by prelude and replacement ops.
    pub external_resources: Vec<ExternalTextureDesc>,
    /// Prelude ops emitted before scene ops.
    pub prelude_ops: Vec<RenderGraphOp>,
    /// Replacement ops for composite commands.
    pub replacements: Vec<CompositeReplacement>,
}

impl CompositeOutput {
    /// Creates an empty composite output.
    pub fn empty() -> Self {
        Self {
            resources: Vec::new(),
            external_resources: Vec::new(),
            prelude_ops: Vec::new(),
            replacements: Vec::new(),
        }
    }

    /// Adds an external texture reference and returns its resource id.
    pub fn add_external_texture(&mut self, desc: ExternalTextureDesc) -> RenderResourceId {
        let index = self.external_resources.len() as u32;
        self.external_resources.push(desc);
        RenderResourceId::External(index)
    }
}

/// Trait for pipelines that expand composite commands into graph ops.
pub trait CompositePipeline<C: CompositeCommand>: Send + Sync + 'static {
    /// Expands composite commands into draw/compute ops.
    fn compile(
        &mut self,
        context: &CompositeContext<'_>,
        items: &[CompositeBatchItem<'_, C>],
    ) -> CompositeOutput;
}

/// Type-erased composite pipeline used by the registry.
pub(crate) trait ErasedCompositePipeline: Send + Sync {
    fn compile_erased(
        &mut self,
        context: &CompositeContext<'_>,
        items: &[ErasedCompositeBatchItem<'_>],
    ) -> CompositeOutput;
}

struct CompositePipelineImpl<C: CompositeCommand, P: CompositePipeline<C>> {
    pipeline: P,
    _command: std::marker::PhantomData<C>,
}

impl<C: CompositeCommand + 'static, P: CompositePipeline<C>> ErasedCompositePipeline
    for CompositePipelineImpl<C, P>
{
    fn compile_erased(
        &mut self,
        context: &CompositeContext<'_>,
        items: &[ErasedCompositeBatchItem<'_>],
    ) -> CompositeOutput {
        if items.is_empty() {
            return CompositeOutput::empty();
        }

        let mut typed_items: Vec<CompositeBatchItem<'_, C>> = Vec::with_capacity(items.len());
        for item in items {
            let command = item
                .command
                .downcast_ref::<C>()
                .expect("Composite batch contained command of unexpected type");
            typed_items.push(CompositeBatchItem {
                command,
                size: item.size,
                position: item.position,
                opacity: item.opacity,
                sequence_index: item.sequence_index,
                op_index: item.op_index,
            });
        }

        self.pipeline.compile(context, &typed_items)
    }
}

/// Registry for managing and dispatching composite pipelines.
#[derive(Default)]
pub struct CompositePipelineRegistry {
    pipelines: HashMap<TypeId, Box<dyn ErasedCompositePipeline>>,
}

impl CompositePipelineRegistry {
    /// Creates a new empty composite pipeline registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new composite pipeline for a specific command type.
    pub fn register<C: CompositeCommand + 'static>(
        &mut self,
        pipeline: impl CompositePipeline<C> + 'static,
    ) {
        let erased = Box::new(CompositePipelineImpl {
            pipeline,
            _command: std::marker::PhantomData,
        });
        self.pipelines.insert(TypeId::of::<C>(), erased);
    }

    pub(crate) fn compile_erased(
        &mut self,
        context: &CompositeContext<'_>,
        items: &[ErasedCompositeBatchItem<'_>],
    ) -> CompositeOutput {
        if items.is_empty() {
            return CompositeOutput::empty();
        }

        let command_type_id = items[0].command.as_any().type_id();
        if let Some(pipeline) = self.pipelines.get_mut(&command_type_id) {
            pipeline.compile_erased(context, items)
        } else {
            panic!(
                "No composite pipeline found for command {:?}",
                std::any::type_name_of_val(items[0].command)
            );
        }
    }
}

pub(crate) fn expand_composites(
    scene: RenderGraph,
    context: CompositeContext<'_>,
    registry: &mut CompositePipelineRegistry,
) -> RenderGraph {
    let RenderGraphParts {
        ops,
        resources,
        external_resources,
    } = scene.into_parts();

    let mut type_order: Vec<TypeId> = Vec::new();
    let mut batches: HashMap<TypeId, Vec<ErasedCompositeBatchItem<'_>>> = HashMap::new();

    for (index, op) in ops.iter().enumerate() {
        if let Command::Composite(command) = &op.command {
            let type_id = command.as_any().type_id();
            let entry = batches.entry(type_id).or_insert_with(|| {
                type_order.push(type_id);
                Vec::new()
            });
            entry.push(ErasedCompositeBatchItem {
                command: command.as_ref(),
                size: op.size,
                position: op.position,
                opacity: op.opacity,
                sequence_index: op.sequence_index,
                op_index: index,
            });
        }
    }

    if type_order.is_empty() {
        return RenderGraph::from_parts(RenderGraphParts {
            ops,
            resources,
            external_resources,
        });
    }

    let mut new_resources = resources;
    let mut new_external_resources = external_resources;
    let mut prelude_ops: Vec<RenderGraphOp> = Vec::new();
    let mut replacements: HashMap<usize, Vec<Vec<RenderGraphOp>>> = HashMap::new();

    for type_id in type_order {
        let items = batches
            .get(&type_id)
            .expect("composite batch missing type entry");
        let output = registry.compile_erased(&context, items);

        let resource_map = map_resources(&mut new_resources, &output.resources);
        let external_map =
            map_external_resources(&mut new_external_resources, &output.external_resources);

        let prelude_base = prelude_ops.len();
        let mut mapped_prelude = output.prelude_ops;
        for op in &mut mapped_prelude {
            remap_resources(op, &resource_map, &external_map);
            remap_deps(op, prelude_base);
        }
        prelude_ops.extend(mapped_prelude);

        for replacement in output.replacements {
            let mut mapped_ops = replacement.ops;
            for op in &mut mapped_ops {
                remap_resources(op, &resource_map, &external_map);
            }
            replacements
                .entry(replacement.target_op)
                .or_default()
                .push(mapped_ops);
        }
    }

    let mut new_ops: Vec<RenderGraphOp> = Vec::with_capacity(prelude_ops.len() + ops.len());
    new_ops.extend(prelude_ops);

    for (index, op) in ops.into_iter().enumerate() {
        match op.command {
            Command::Composite(_) => {
                if let Some(mut fragments) = replacements.remove(&index) {
                    for mut fragment_ops in fragments.drain(..) {
                        let base = new_ops.len();
                        for op in &mut fragment_ops {
                            remap_deps(op, base);
                        }
                        new_ops.extend(fragment_ops);
                    }
                }
            }
            _ => new_ops.push(op),
        }
    }

    if !replacements.is_empty() {
        for mut fragments in replacements.into_values() {
            for mut fragment_ops in fragments.drain(..) {
                let base = new_ops.len();
                for op in &mut fragment_ops {
                    remap_deps(op, base);
                }
                new_ops.extend(fragment_ops);
            }
        }
    }

    for (seq, op) in new_ops.iter_mut().enumerate() {
        op.sequence_index = seq;
    }

    RenderGraph::from_parts(RenderGraphParts {
        ops: new_ops,
        resources: new_resources,
        external_resources: new_external_resources,
    })
}

fn map_resources(
    resources: &mut Vec<RenderResource>,
    locals: &[RenderResource],
) -> Vec<RenderResourceId> {
    let mut map = Vec::with_capacity(locals.len());
    for resource in locals {
        let index = resources.len() as u32;
        resources.push(resource.clone());
        map.push(RenderResourceId::Local(index));
    }
    map
}

fn map_external_resources(
    resources: &mut Vec<ExternalTextureDesc>,
    externals: &[ExternalTextureDesc],
) -> Vec<RenderResourceId> {
    let mut map = Vec::with_capacity(externals.len());
    for resource in externals {
        if let Some(index) = resources
            .iter()
            .position(|existing| existing.handle_id == resource.handle_id)
        {
            map.push(RenderResourceId::External(index as u32));
            continue;
        }
        let index = resources.len() as u32;
        resources.push(resource.clone());
        map.push(RenderResourceId::External(index));
    }
    map
}

fn remap_resources(
    op: &mut RenderGraphOp,
    local_map: &[RenderResourceId],
    external_map: &[RenderResourceId],
) {
    op.read = op
        .read
        .map(|resource| map_resource(resource, local_map, external_map));
    op.write = op
        .write
        .map(|resource| map_resource(resource, local_map, external_map));
}

fn map_resource(
    resource: RenderResourceId,
    local_map: &[RenderResourceId],
    external_map: &[RenderResourceId],
) -> RenderResourceId {
    match resource {
        RenderResourceId::Local(index) => local_map
            .get(index as usize)
            .copied()
            .unwrap_or(RenderResourceId::Local(index)),
        RenderResourceId::External(index) => external_map
            .get(index as usize)
            .copied()
            .unwrap_or(RenderResourceId::External(index)),
        other => other,
    }
}

fn remap_deps(op: &mut RenderGraphOp, base: usize) {
    if base == 0 {
        return;
    }
    for dep in op.deps.iter_mut() {
        *dep += base;
    }
}
