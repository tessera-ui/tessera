use std::{any::TypeId, mem, time::Instant};

use downcast_rs::Downcast;
use smallvec::SmallVec;

use crate::{
    DrawCommand, Px, PxPosition, PxRect, PxSize,
    render_graph::{
        ExternalTextureDesc, RenderGraphExecution, RenderResource, RenderResourceId,
        RenderTextureDesc,
    },
    render_pass::{
        ClipOps, ComputePlanItem, DrawOrClip, RenderPassGraph, RenderPassKind, RenderPassPlan,
    },
    renderer::{
        compute::{ErasedComputeBatchItem, pipeline::ErasedDispatchContext},
        drawer::ErasedDrawContext,
        external::{ExternalTextureRegistry, ExternalTextureSlotGuard},
    },
};

use super::*;

fn compute_last_use_passes(passes: &[RenderPassPlan], local_count: usize) -> Vec<usize> {
    let mut last_use = vec![0usize; local_count];
    for (pass_index, pass) in passes.iter().enumerate() {
        if let Some(RenderResourceId::Local(index)) = pass.read_resource {
            last_use[index as usize] = pass_index;
        }
        if let RenderResourceId::Local(index) = pass.write_resource {
            last_use[index as usize] = pass_index;
        }
        for draw in pass.draws.iter() {
            if let DrawOrClip::Draw(cmd) = draw
                && let Some(RenderResourceId::Local(index)) = cmd.read_resource
            {
                last_use[index as usize] = pass_index;
            }
        }
    }
    last_use
}

struct RenderPassParams<'a, 'b> {
    msaa_view: Option<wgpu::TextureView>,
    clear_target: bool,
    encoder: &'a mut wgpu::CommandEncoder,
    write_target: wgpu::TextureView,
    commands_in_pass: &'a mut SmallVec<[DrawOrClip; 32]>,
    scene_texture_view: wgpu::TextureView,
    drawer: &'a mut Drawer,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    target_size: PxSize,
    clip_stack: &'a mut SmallVec<[PxRect; 16]>,
    apply_clip: bool,
    resources: &'a mut FrameResources<'b>,
}

struct RenderPassExecParams<'a, 'b> {
    encoder: &'a mut wgpu::CommandEncoder,
    scene_texture_view: &'a mut wgpu::TextureView,
    scene_source: &'a mut SceneSource,
    resources: &'a mut FrameResources<'b>,
    clear_state: &'a mut RenderPassClearState,
    pass: &'a mut RenderPassPlan,
    clip_stack: &'a mut SmallVec<[PxRect; 16]>,
}

struct BlitParams<'a> {
    encoder: &'a mut wgpu::CommandEncoder,
    device: &'a wgpu::Device,
    source: &'a wgpu::TextureView,
    target: &'a wgpu::TextureView,
    bind_group_layout: &'a wgpu::BindGroupLayout,
    sampler: &'a wgpu::Sampler,
    pipeline: &'a wgpu::RenderPipeline,
    target_size: PxSize,
    scissor_rect: Option<PxRect>,
}

struct FrameResources<'a> {
    locals: Vec<LocalResourceState>,
    pool: &'a mut LocalTexturePool,
    device: &'a wgpu::Device,
    sample_count: u32,
    current_frame: u64,
    external: ExternalTextureRegistry,
    external_descs: &'a [ExternalTextureDesc],
}

struct FrameResourcesParams<'a> {
    pool: &'a mut LocalTexturePool,
    device: &'a wgpu::Device,
    resources: &'a [RenderResource],
    external_descs: &'a [ExternalTextureDesc],
    sample_count: u32,
    current_frame: u64,
    last_use_passes: &'a [usize],
    external: ExternalTextureRegistry,
}

struct LocalResourceState {
    desc: RenderTextureDesc,
    slot: Option<usize>,
    last_use_pass: usize,
}

impl<'a> FrameResources<'a> {
    fn new(params: FrameResourcesParams<'a>) -> Self {
        let FrameResourcesParams {
            pool,
            device,
            resources,
            external_descs,
            sample_count,
            current_frame,
            last_use_passes,
            external,
        } = params;
        let locals = resources
            .iter()
            .enumerate()
            .map(|(index, resource)| match resource {
                RenderResource::Texture(desc) => LocalResourceState {
                    desc: desc.clone(),
                    slot: None,
                    last_use_pass: *last_use_passes.get(index).unwrap_or(&0),
                },
            })
            .collect();
        Self {
            locals,
            pool,
            device,
            sample_count,
            current_frame,
            external,
            external_descs,
        }
    }

    fn local_slot(&mut self, id: RenderResourceId) -> Option<&LocalTextureSlot> {
        let RenderResourceId::Local(index) = id else {
            return None;
        };
        let local = self.locals.get_mut(index as usize)?;
        let slot_index = match local.slot {
            Some(slot) => slot,
            None => {
                let slot = self.pool.allocate(
                    self.device,
                    &local.desc,
                    self.sample_count,
                    self.current_frame,
                );
                local.slot = Some(slot);
                slot
            }
        };
        self.pool.slot(slot_index)
    }

    fn local_slot_mut(&mut self, id: RenderResourceId) -> Option<&mut LocalTextureSlot> {
        let RenderResourceId::Local(index) = id else {
            return None;
        };
        let local = self.locals.get_mut(index as usize)?;
        let slot_index = match local.slot {
            Some(slot) => slot,
            None => {
                let slot = self.pool.allocate(
                    self.device,
                    &local.desc,
                    self.sample_count,
                    self.current_frame,
                );
                local.slot = Some(slot);
                slot
            }
        };
        self.pool.slot_mut(slot_index)
    }

    fn external_slot(&self, id: RenderResourceId) -> Option<ExternalTextureSlotGuard<'_>> {
        let RenderResourceId::External(index) = id else {
            return None;
        };
        let handle_id = self.external_descs.get(index as usize)?.handle_id;
        self.external.slot(handle_id)
    }

    fn external_slot_mut(&self, id: RenderResourceId) -> Option<ExternalTextureSlotGuard<'_>> {
        self.external_slot(id)
    }

    fn release_for_pass(&mut self, pass_index: usize) {
        for local in &mut self.locals {
            if local.last_use_pass == pass_index
                && let Some(slot) = local.slot.take()
                && let Some(slot_ref) = self.pool.slot_mut(slot)
            {
                slot_ref.in_use = false;
            }
        }
    }
}

struct RenderCoreFrameState<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    targets: &'a mut FrameTargets,
    pipelines: &'a mut RenderPipelines,
    compute: &'a mut ComputeState,
    blit: &'a BlitState,
}

struct RenderPassClearState {
    scene_written: bool,
    locals_written: Vec<bool>,
    externals_written: Vec<bool>,
    externals_should_clear: Vec<bool>,
}

impl RenderPassClearState {
    fn new(local_count: usize, external_resources: &[ExternalTextureDesc]) -> Self {
        Self {
            scene_written: false,
            locals_written: vec![false; local_count],
            externals_written: vec![false; external_resources.len()],
            externals_should_clear: external_resources
                .iter()
                .map(|desc| desc.clear_on_first_use)
                .collect(),
        }
    }

    fn should_clear(&mut self, resource: RenderResourceId) -> bool {
        match resource {
            RenderResourceId::SceneColor => {
                let clear = !self.scene_written;
                self.scene_written = true;
                clear
            }
            RenderResourceId::Local(index) => {
                let slot = self
                    .locals_written
                    .get_mut(index as usize)
                    .unwrap_or_else(|| panic!("missing clear state for local resource {index}"));
                let clear = !*slot;
                *slot = true;
                clear
            }
            RenderResourceId::External(index) => {
                let should_clear = self
                    .externals_should_clear
                    .get(index as usize)
                    .copied()
                    .unwrap_or(true);
                if !should_clear {
                    return false;
                }
                let slot = self
                    .externals_written
                    .get_mut(index as usize)
                    .unwrap_or_else(|| panic!("missing clear state for external resource {index}"));
                let clear = !*slot;
                *slot = true;
                clear
            }
            RenderResourceId::SceneDepth => false,
        }
    }
}

trait ComputeTargets {
    fn views(&self) -> (wgpu::TextureView, wgpu::TextureView);
    fn swap(&mut self);
}

struct SceneComputeTargets<'a> {
    front: &'a mut wgpu::TextureView,
    back: &'a mut wgpu::TextureView,
}

impl ComputeTargets for SceneComputeTargets<'_> {
    fn views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        (self.front.clone(), self.back.clone())
    }

    fn swap(&mut self) {
        std::mem::swap(self.front, self.back);
    }
}

struct LocalComputeTargets<'a> {
    slot: &'a mut LocalTextureSlot,
}

impl ComputeTargets for LocalComputeTargets<'_> {
    fn views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        (
            self.slot.front_view().clone(),
            self.slot.back_view().clone(),
        )
    }

    fn swap(&mut self) {
        self.slot.swap_front_back();
    }
}

struct ExternalComputeTargets<'a> {
    slot: ExternalTextureSlotGuard<'a>,
}

impl ComputeTargets for ExternalComputeTargets<'_> {
    fn views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        (self.slot.front_view(), self.slot.back_view())
    }

    fn swap(&mut self) {
        self.slot.swap_front_back();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SceneSource {
    Offscreen,
    Compute,
}

struct SubmitContext<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
}

struct SubmitBatchParams<'a, 'b> {
    drawer: &'a mut Drawer,
    resources: SubmitContext<'a>,
    scene_texture_view: &'a wgpu::TextureView,
    target_size: PxSize,
    clip_stack: &'b [PxRect],
    current_batch_draw_rect: &'a mut Option<PxRect>,
}

struct ComputePassParams<'a> {
    encoder: &'a mut wgpu::CommandEncoder,
    commands: Vec<ComputePlanItem>,
    compute_pipeline_registry: &'a mut ComputePipelineRegistry,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    resource_manager: &'a mut ComputeResourceManager,
}

impl RenderCore {
    /// Render the surface using the unified command system.
    ///
    /// This method processes a stream of commands (both draw and compute) and
    /// renders them to the surface using a multi-pass rendering approach
    /// with offscreen texture. Commands that require barriers will trigger
    /// texture copies between passes.
    ///
    /// # Arguments
    ///
    /// * `ops` - Ordered render ops for the current frame.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if rendering succeeds
    /// * `Err(wgpu::SurfaceError)` if there are issues with the surface
    pub(crate) fn render(
        &mut self,
        execution: RenderGraphExecution,
    ) -> Result<(), wgpu::SurfaceError> {
        let render_start = Instant::now();
        let current_frame = self.frame_index;
        self.last_render_breakdown = None;
        let acquire_start = Instant::now();
        let output_frame = self.surface.get_current_texture()?;
        let acquire = acquire_start.elapsed();

        let texture_size = wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        };

        let RenderGraphExecution {
            ops,
            resources,
            external_resources,
        } = execution;
        for resource in &external_resources {
            self.external_textures
                .mark_used(resource.handle_id, current_frame);
        }
        let build_start = Instant::now();
        let graph = RenderPassGraph::build(ops, &resources, &external_resources, texture_size);
        let mut passes = graph.into_passes();
        let last_use_passes = compute_last_use_passes(&passes, resources.len());
        let build_passes = build_start.elapsed();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let device = &self.device;
        let queue = &self.queue;
        let config = &self.config;
        let blit = &self.blit;
        let pipelines = &mut self.pipelines;
        let targets = &mut self.targets;
        let compute = &mut self.compute;
        let local_textures = &mut self.local_textures;

        let encode_start = Instant::now();
        local_textures.begin_frame(current_frame);

        // Frame-level begin for all pipelines
        pipelines
            .drawer
            .pipeline_registry
            .begin_all_frames(device, queue, config);

        let mut scene_texture_view = targets.offscreen.clone();
        let mut scene_source = SceneSource::Offscreen;
        let mut clip_stack: SmallVec<[PxRect; 16]> = SmallVec::new();
        let mut frame_resources = FrameResources::new(FrameResourcesParams {
            pool: local_textures,
            device,
            resources: &resources,
            external_descs: &external_resources,
            sample_count: targets.sample_count,
            current_frame,
            last_use_passes: &last_use_passes,
            external: self.external_textures.clone(),
        });

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut clear_state = RenderPassClearState::new(resources.len(), &external_resources);

        let mut frame_state = RenderCoreFrameState {
            device,
            queue,
            config,
            targets,
            pipelines,
            compute,
            blit,
        };

        for (pass_index, pass) in passes.iter_mut().enumerate() {
            Self::execute_render_pass(
                &mut frame_state,
                RenderPassExecParams {
                    encoder: &mut encoder,
                    scene_texture_view: &mut scene_texture_view,
                    scene_source: &mut scene_source,
                    resources: &mut frame_resources,
                    clear_state: &mut clear_state,
                    pass,
                    clip_stack: &mut clip_stack,
                },
            );
            frame_resources.release_for_pass(pass_index);
        }

        let target_size = PxSize::new(Px(self.config.width as i32), Px(self.config.height as i32));
        RenderCore::blit_to_view(BlitParams {
            encoder: &mut encoder,
            device,
            source: &scene_texture_view,
            target: &output_view,
            bind_group_layout: &blit.bind_group_layout,
            sampler: &blit.sampler,
            pipeline: &blit.pipeline,
            target_size,
            scissor_rect: None,
        });

        // Frame-level end for all pipelines
        frame_state
            .pipelines
            .drawer
            .pipeline_registry
            .end_all_frames(device, queue, config);
        let encode = encode_start.elapsed();

        let submit_start = Instant::now();
        queue.submit(Some(encoder.finish()));
        let submit = submit_start.elapsed();

        let present_start = Instant::now();
        output_frame.present();
        let present = present_start.elapsed();
        self.external_textures.collect_garbage(current_frame, 2);
        self.frame_index = self.frame_index.wrapping_add(1);
        self.last_render_breakdown = Some(RenderTimingBreakdown {
            acquire,
            build_passes,
            encode,
            submit,
            present,
            total: render_start.elapsed(),
        });

        Ok(())
    }

    fn execute_render_pass(
        state: &mut RenderCoreFrameState<'_>,
        params: RenderPassExecParams<'_, '_>,
    ) {
        let RenderPassExecParams {
            encoder,
            scene_texture_view,
            scene_source,
            resources,
            clear_state,
            pass,
            clip_stack,
        } = params;

        if pass.draws.is_empty() && pass.compute.is_empty() {
            return;
        }

        match pass.kind {
            RenderPassKind::Compute => {
                if pass.compute.is_empty() {
                    return;
                }

                let read_resource = pass.read_resource.unwrap_or(RenderResourceId::SceneColor);
                let write_resource = pass.write_resource;

                let input_view = match read_resource {
                    RenderResourceId::SceneColor => Some(scene_texture_view.clone()),
                    RenderResourceId::Local(_) => resources
                        .local_slot(read_resource)
                        .map(|slot| slot.front_view().clone()),
                    RenderResourceId::External(_) => resources
                        .external_slot(read_resource)
                        .map(|slot| slot.front_view()),
                    RenderResourceId::SceneDepth => Some(scene_texture_view.clone()),
                };

                let input_follows_front = read_resource == write_resource;
                let compute_to_run = std::mem::take(&mut pass.compute);

                let final_view = match write_resource {
                    RenderResourceId::SceneColor => {
                        let mut targets = SceneComputeTargets {
                            front: &mut state.compute.target_a,
                            back: &mut state.compute.target_b,
                        };
                        let needs_scene_prefill = read_resource == RenderResourceId::SceneColor
                            && !matches!(*scene_source, SceneSource::Compute);
                        if needs_scene_prefill {
                            let (front_view, _) = targets.views();
                            Self::blit_to_view(BlitParams {
                                encoder,
                                device: state.device,
                                source: scene_texture_view,
                                target: &front_view,
                                bind_group_layout: &state.blit.bind_group_layout,
                                sampler: &state.blit.sampler,
                                pipeline: &state.blit.pipeline_rgba,
                                target_size: PxSize::new(
                                    Px(state.config.width as i32),
                                    Px(state.config.height as i32),
                                ),
                                scissor_rect: None,
                            });
                        }
                        let params = ComputePassParams {
                            encoder,
                            commands: compute_to_run,
                            compute_pipeline_registry: &mut state.pipelines.compute_registry,
                            device: state.device,
                            queue: state.queue,
                            config: state.config,
                            resource_manager: &mut state.compute.resource_manager.write(),
                        };
                        let texture_size = wgpu::Extent3d {
                            width: state.config.width,
                            height: state.config.height,
                            depth_or_array_layers: 1,
                        };
                        do_compute_with_targets(
                            params,
                            if read_resource == RenderResourceId::SceneColor {
                                None
                            } else {
                                input_view
                            },
                            &mut targets,
                            texture_size,
                            input_follows_front,
                        )
                    }
                    RenderResourceId::Local(_) => {
                        let params = ComputePassParams {
                            encoder,
                            commands: compute_to_run,
                            compute_pipeline_registry: &mut state.pipelines.compute_registry,
                            device: state.device,
                            queue: state.queue,
                            config: state.config,
                            resource_manager: &mut state.compute.resource_manager.write(),
                        };
                        let Some(slot) = resources.local_slot_mut(write_resource) else {
                            return;
                        };
                        let texture_size = wgpu::Extent3d {
                            width: slot.desc.size.width.positive().max(1),
                            height: slot.desc.size.height.positive().max(1),
                            depth_or_array_layers: 1,
                        };
                        let mut targets = LocalComputeTargets { slot };
                        do_compute_with_targets(
                            params,
                            input_view,
                            &mut targets,
                            texture_size,
                            input_follows_front,
                        )
                    }
                    RenderResourceId::External(_) => {
                        let params = ComputePassParams {
                            encoder,
                            commands: compute_to_run,
                            compute_pipeline_registry: &mut state.pipelines.compute_registry,
                            device: state.device,
                            queue: state.queue,
                            config: state.config,
                            resource_manager: &mut state.compute.resource_manager.write(),
                        };
                        let Some(slot) = resources.external_slot_mut(write_resource) else {
                            return;
                        };
                        let size = slot.size();
                        let texture_size = wgpu::Extent3d {
                            width: size.width.positive().max(1),
                            height: size.height.positive().max(1),
                            depth_or_array_layers: 1,
                        };
                        let mut targets = ExternalComputeTargets { slot };
                        do_compute_with_targets(
                            params,
                            input_view,
                            &mut targets,
                            texture_size,
                            input_follows_front,
                        )
                    }
                    RenderResourceId::SceneDepth => {
                        drop(compute_to_run);
                        return;
                    }
                };

                if write_resource == RenderResourceId::SceneColor {
                    *scene_texture_view = final_view;
                    *scene_source = SceneSource::Compute;
                }
            }
            RenderPassKind::Draw => {
                if pass.draws.is_empty() {
                    return;
                }

                let write_resource = pass.write_resource;
                let reads_scene = pass.read_resource == Some(RenderResourceId::SceneColor);
                let (write_target, msaa_view, target_size) = match write_resource {
                    RenderResourceId::SceneColor => (
                        state.targets.offscreen.clone(),
                        state.targets.msaa_view.clone(),
                        PxSize::new(
                            Px(state.config.width as i32),
                            Px(state.config.height as i32),
                        ),
                    ),
                    RenderResourceId::Local(_) => {
                        let Some(slot) = resources.local_slot(write_resource) else {
                            return;
                        };
                        (
                            slot.front_view().clone(),
                            slot.msaa_view.clone(),
                            slot.desc.size,
                        )
                    }
                    RenderResourceId::External(_) => {
                        let Some(slot) = resources.external_slot(write_resource) else {
                            return;
                        };
                        (slot.front_view(), slot.msaa_view(), slot.size())
                    }
                    RenderResourceId::SceneDepth => {
                        return;
                    }
                };

                let mut scene_view = if reads_scene {
                    scene_texture_view.clone()
                } else {
                    write_target.clone()
                };
                if reads_scene
                    && write_resource == RenderResourceId::SceneColor
                    && matches!(*scene_source, SceneSource::Offscreen)
                {
                    let copy_view = state.targets.offscreen_copy.clone();
                    Self::blit_to_view(BlitParams {
                        encoder,
                        device: state.device,
                        source: scene_texture_view,
                        target: &copy_view,
                        bind_group_layout: &state.blit.bind_group_layout,
                        sampler: &state.blit.sampler,
                        pipeline: &state.blit.pipeline,
                        target_size: PxSize::new(
                            Px(state.config.width as i32),
                            Px(state.config.height as i32),
                        ),
                        scissor_rect: None,
                    });
                    scene_view = copy_view;
                }
                let clear_target = clear_state.should_clear(write_resource);

                render_current_pass(RenderPassParams {
                    msaa_view,
                    clear_target,
                    encoder,
                    write_target: write_target.clone(),
                    commands_in_pass: &mut pass.draws,
                    scene_texture_view: scene_view,
                    drawer: &mut state.pipelines.drawer,
                    device: state.device,
                    queue: state.queue,
                    config: state.config,
                    target_size,
                    clip_stack,
                    apply_clip: write_resource == RenderResourceId::SceneColor,
                    resources,
                });

                if write_resource == RenderResourceId::SceneColor {
                    *scene_texture_view = write_target.clone();
                    *scene_source = SceneSource::Offscreen;
                }
            }
        }
    }

    fn blit_to_view(params: BlitParams<'_>) {
        let BlitParams {
            encoder,
            device,
            source,
            target,
            bind_group_layout,
            sampler,
            pipeline,
            target_size,
            scissor_rect,
        } = params;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("Compute Copy Bind Group"),
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Compute Copy Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        if let Some(rect) = scissor_rect {
            let Some(clamped) = clamp_rect_to_target(rect, target_size) else {
                return;
            };
            rpass.set_scissor_rect(
                clamped.x.0.max(0) as u32,
                clamped.y.0.max(0) as u32,
                clamped.width.0.max(0) as u32,
                clamped.height.0.max(0) as u32,
            );
        }
        rpass.draw(0..3, 0..1);
    }
}

fn do_compute_with_targets<T: ComputeTargets>(
    params: ComputePassParams<'_>,
    fixed_input_view: Option<wgpu::TextureView>,
    targets: &mut T,
    texture_size: wgpu::Extent3d,
    input_follows_front: bool,
) -> wgpu::TextureView {
    if params.commands.is_empty() {
        let (front_view, _) = targets.views();
        return front_view;
    }

    let commands = &params.commands;
    let mut index = 0;
    while index < commands.len() {
        let command = &commands[index];
        let type_id = command.command.as_any().type_id();

        let mut batch_items: SmallVec<[ErasedComputeBatchItem<'_>; 8]> = SmallVec::new();
        let mut batch_sampling_rects: SmallVec<[PxRect; 8]> = SmallVec::new();
        let mut cursor = index;

        while cursor < commands.len() {
            let candidate = &commands[cursor];
            if candidate.command.as_any().type_id() != type_id {
                break;
            }

            let sampling_area = candidate.sampling_rect;

            if batch_sampling_rects
                .iter()
                .any(|existing| rects_overlap(*existing, sampling_area))
            {
                break;
            }

            batch_sampling_rects.push(sampling_area);
            batch_items.push(ErasedComputeBatchItem {
                command: &*candidate.command,
                size: candidate.size,
                position: candidate.start_pos,
                target_area: candidate.target_rect,
            });
            cursor += 1;
        }

        if batch_items.is_empty() {
            batch_sampling_rects.push(command.sampling_rect);
            batch_items.push(ErasedComputeBatchItem {
                command: &*command.command,
                size: command.size,
                position: command.start_pos,
                target_area: command.target_rect,
            });
            cursor = index + 1;
        }

        let (front_view, back_view) = targets.views();
        let use_fixed_input = input_follows_front && fixed_input_view.is_some() && index == 0;
        let input_view = if use_fixed_input {
            fixed_input_view
                .as_ref()
                .expect("compute pass missing input view")
                .clone()
        } else if input_follows_front {
            front_view.clone()
        } else {
            fixed_input_view
                .as_ref()
                .expect("compute pass missing input view")
                .clone()
        };

        let copy_source = if use_fixed_input {
            fixed_input_view
                .as_ref()
                .expect("compute pass missing input view")
        } else {
            &front_view
        };
        params.encoder.copy_texture_to_texture(
            copy_source.texture().as_image_copy(),
            back_view.texture().as_image_copy(),
            texture_size,
        );

        {
            let mut cpass = params
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

            params.compute_pipeline_registry.dispatch_erased(
                ErasedDispatchContext {
                    device: params.device,
                    queue: params.queue,
                    config: params.config,
                    target_size: PxSize::new(
                        Px(texture_size.width as i32),
                        Px(texture_size.height as i32),
                    ),
                    compute_pass: &mut cpass,
                    resource_manager: params.resource_manager,
                    input_view: &input_view,
                    output_view: &back_view,
                },
                &batch_items,
            );
        }

        targets.swap();
        index = cursor;
    }

    let (front_view, _) = targets.views();
    front_view
}

fn rects_overlap(a: PxRect, b: PxRect) -> bool {
    let a_left = a.x.0;
    let a_top = a.y.0;
    let a_right = a_left + a.width.0;
    let a_bottom = a_top + a.height.0;

    let b_left = b.x.0;
    let b_top = b.y.0;
    let b_right = b_left + b.width.0;
    let b_bottom = b_top + b.height.0;

    !(a_right <= b_left || b_right <= a_left || a_bottom <= b_top || b_bottom <= a_top)
}

fn render_current_pass(params: RenderPassParams<'_, '_>) {
    let RenderPassParams {
        msaa_view,
        clear_target,
        encoder,
        write_target,
        commands_in_pass,
        scene_texture_view,
        drawer,
        device,
        queue,
        config,
        target_size,
        clip_stack,
        apply_clip,
        resources,
    } = params;

    let (view, resolve_target) = if let Some(msaa_view) = msaa_view.as_ref() {
        (msaa_view, Some(&write_target))
    } else {
        (&write_target, None)
    };

    let load_ops = if clear_target {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
    } else {
        wgpu::LoadOp::Load
    };

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target,
            ops: wgpu::Operations {
                load: load_ops,
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });

    drawer.begin_pass(
        device,
        queue,
        config,
        target_size,
        &mut rpass,
        &scene_texture_view,
    );

    // Prepare buffered submission state
    let mut buffer: Vec<(Box<dyn DrawCommand>, PxSize, PxPosition)> = Vec::new();
    let mut last_command_type_id = None;
    let mut current_batch_draw_rect: Option<PxRect> = None;
    let mut current_batch_read: Option<RenderResourceId> = None;
    for cmd in mem::take(commands_in_pass).into_iter() {
        let cmd = match cmd {
            DrawOrClip::Clip(clip_ops) => {
                // Must flush any existing buffered commands before changing clip state
                if !buffer.is_empty() {
                    let scene_view = resolve_scene_view(
                        current_batch_read,
                        &scene_texture_view,
                        &write_target,
                        resources,
                    );
                    submit_buffered_commands(
                        &mut rpass,
                        SubmitBatchParams {
                            drawer,
                            resources: SubmitContext {
                                device,
                                queue,
                                config,
                            },
                            scene_texture_view: &scene_view,
                            target_size,
                            clip_stack: if apply_clip {
                                clip_stack.as_slice()
                            } else {
                                &[]
                            },
                            current_batch_draw_rect: &mut current_batch_draw_rect,
                        },
                        &mut buffer,
                    );
                    last_command_type_id = None; // Reset batch type after flush
                    current_batch_read = None;
                }
                // Update clip stack
                match clip_ops {
                    ClipOps::Push(rect) => {
                        clip_stack.push(rect);
                    }
                    ClipOps::Pop => {
                        clip_stack.pop();
                    }
                }
                // continue to next command
                continue;
            }
            DrawOrClip::Draw(cmd) => cmd, // Proceed with draw commands
        };

        // If the incoming command cannot be merged into the current batch, flush first.
        let read_resource = cmd.read_resource;
        if (!can_merge_into_batch(&last_command_type_id, cmd.type_id)
            || current_batch_read != read_resource)
            && !buffer.is_empty()
        {
            let scene_view = resolve_scene_view(
                current_batch_read,
                &scene_texture_view,
                &write_target,
                resources,
            );
            submit_buffered_commands(
                &mut rpass,
                SubmitBatchParams {
                    drawer,
                    resources: SubmitContext {
                        device,
                        queue,
                        config,
                    },
                    scene_texture_view: &scene_view,
                    target_size,
                    clip_stack: if apply_clip {
                        clip_stack.as_slice()
                    } else {
                        &[]
                    },
                    current_batch_draw_rect: &mut current_batch_draw_rect,
                },
                &mut buffer,
            );
        }

        // Add the command to the buffer and update the current batch rect (extracted
        // merge helper).
        buffer.push((cmd.command, cmd.size, cmd.start_pos));
        last_command_type_id = Some(cmd.type_id);
        current_batch_read = read_resource;
        current_batch_draw_rect = Some(merge_batch_rect(current_batch_draw_rect, cmd.draw_rect));
    }

    // If there are any remaining commands in the buffer, submit them
    if !buffer.is_empty() {
        let scene_view = resolve_scene_view(
            current_batch_read,
            &scene_texture_view,
            &write_target,
            resources,
        );
        submit_buffered_commands(
            &mut rpass,
            SubmitBatchParams {
                drawer,
                resources: SubmitContext {
                    device,
                    queue,
                    config,
                },
                scene_texture_view: &scene_view,
                target_size,
                clip_stack: if apply_clip {
                    clip_stack.as_slice()
                } else {
                    &[]
                },
                current_batch_draw_rect: &mut current_batch_draw_rect,
            },
            &mut buffer,
        );
    }

    drawer.end_pass(
        device,
        queue,
        config,
        target_size,
        &mut rpass,
        &scene_texture_view,
    );
}

fn resolve_scene_view(
    read_resource: Option<RenderResourceId>,
    scene_texture_view: &wgpu::TextureView,
    write_target: &wgpu::TextureView,
    resources: &mut FrameResources<'_>,
) -> wgpu::TextureView {
    match read_resource {
        Some(RenderResourceId::SceneColor) => scene_texture_view.clone(),
        Some(resource_id @ RenderResourceId::Local(_)) => resources
            .local_slot(resource_id)
            .map(|slot| slot.front_view().clone())
            .unwrap_or_else(|| scene_texture_view.clone()),
        Some(resource_id @ RenderResourceId::External(_)) => resources
            .external_slot(resource_id)
            .map(|slot| slot.front_view())
            .unwrap_or_else(|| scene_texture_view.clone()),
        Some(RenderResourceId::SceneDepth) => scene_texture_view.clone(),
        None => write_target.clone(),
    }
}

fn submit_buffered_commands(
    rpass: &mut wgpu::RenderPass<'_>,
    params: SubmitBatchParams<'_, '_>,
    buffer: &mut Vec<(Box<dyn DrawCommand>, PxSize, PxPosition)>,
) {
    let SubmitBatchParams {
        drawer,
        resources,
        scene_texture_view,
        target_size,
        clip_stack,
        current_batch_draw_rect,
    } = params;
    // Take the buffered commands and convert to the transient representation
    // expected by drawer.submit
    let commands = mem::take(buffer);
    let commands = commands
        .iter()
        .map(|(cmd, sz, pos)| (&**cmd, *sz, *pos))
        .collect::<Vec<_>>();

    // Apply clipping to the current batch rectangle; if nothing remains, abort
    // early.
    let (current_clip_rect, anything_to_submit) =
        apply_clip_to_batch_rect(clip_stack, current_batch_draw_rect, target_size);
    if !anything_to_submit {
        return;
    }

    let Some(rect) = *current_batch_draw_rect else {
        return;
    };
    set_scissor_rect_from_pxrect(rpass, rect);

    drawer.submit(
        ErasedDrawContext {
            device: resources.device,
            queue: resources.queue,
            config: resources.config,
            target_size,
            render_pass: rpass,
            scene_texture_view,
            clip_rect: current_clip_rect,
        },
        &commands,
    );
    *current_batch_draw_rect = None;
}

fn set_scissor_rect_from_pxrect(rpass: &mut wgpu::RenderPass<'_>, rect: PxRect) {
    rpass.set_scissor_rect(
        rect.x.positive(),
        rect.y.positive(),
        rect.width.positive(),
        rect.height.positive(),
    );
}

fn clamp_rect_to_target(rect: PxRect, target_size: PxSize) -> Option<PxRect> {
    let target_rect = PxRect::from_position_size(PxPosition::ZERO, target_size);
    rect.intersection(&target_rect)
}

/// Apply clip_stack to current_batch_draw_rect. Returns false if intersection
/// yields nothing (meaning there is nothing to submit), true otherwise.
///
/// Also returns the current clipping rectangle (if any) for potential use by
/// the caller.
fn apply_clip_to_batch_rect(
    clip_stack: &[PxRect],
    current_batch_draw_rect: &mut Option<PxRect>,
    target_size: PxSize,
) -> (Option<PxRect>, bool) {
    let clipped_rect = clip_stack
        .last()
        .copied()
        .and_then(|rect| clamp_rect_to_target(rect, target_size));

    let Some(current_rect) = current_batch_draw_rect.as_ref() else {
        return (clipped_rect, false);
    };

    let Some(mut final_rect) = clamp_rect_to_target(*current_rect, target_size) else {
        *current_batch_draw_rect = None;
        return (clipped_rect, false);
    };

    if let Some(clip_rect) = clipped_rect {
        if let Some(intersection) = final_rect.intersection(&clip_rect) {
            final_rect = intersection;
        } else {
            *current_batch_draw_rect = None;
            return (Some(clip_rect), false);
        }
    }

    *current_batch_draw_rect = Some(final_rect);
    (clipped_rect, true)
}

/// Determine whether `next_type_id` (with potential clipping) can be merged
/// into the current batch. Equivalent to the negation of the original flush
/// condition: merge allowed when last_command_type_id == Some(next_type_id) or
/// last_command_type_id is None.
fn can_merge_into_batch(last_command_type_id: &Option<TypeId>, next_type_id: TypeId) -> bool {
    match last_command_type_id {
        Some(l) => *l == next_type_id,
        None => true,
    }
}

/// Merge the existing optional batch rect with a new command rect.
fn merge_batch_rect(current: Option<PxRect>, next: PxRect) -> PxRect {
    current.map(|dr| dr.union(&next)).unwrap_or(next)
}
