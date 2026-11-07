use std::any::TypeId;

use downcast_rs::Downcast;
use smallvec::SmallVec;
use tracing::{debug, error, warn};

use crate::{
    DrawCommand, LocalToWindow, Px, PxPosition, PxRect, PxSize, SampleRegion, ScratchFit,
    ScratchTexture, ScratchTextureDescriptor,
    render_graph::{DrawGeometry, RenderGraphOp},
    render_pass::{ClipOps, ComputePlanItem, DrawOrClip, RenderPassGraph, RenderPassPlan},
    renderer::{
        compute::{ErasedComputeBatchItem, pipeline::ErasedDispatchContext},
        drawer::ErasedDrawContext,
    },
    time::Instant,
};

use super::*;

/// Scene pixels available to draws that sample the scene.
enum SceneState {
    /// Draws sample a copy of the offscreen target.
    Offscreen,
    /// A compute pass filtered the scene inside `rect`.
    Filtered {
        texture: ScratchTexture,
        rect: PxRect,
    },
}

struct RenderPassParams<'a> {
    msaa_view: Option<&'a wgpu::TextureView>,
    stencil_view: &'a wgpu::TextureView,
    clipper: &'a mut super::clip::ShapeClipper,
    clear_target: bool,
    encoder: &'a mut wgpu::CommandEncoder,
    write_target: &'a wgpu::TextureView,
    commands_in_pass: &'a mut SmallVec<[DrawOrClip; 32]>,
    scene: Option<(&'a wgpu::TextureView, PxRect)>,
    drawer: &'a mut Drawer,
    graphics: &'a crate::GraphicsDevice,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    target_size: PxSize,
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
    /// Source position of the target's first texel.
    source_origin: PxPosition,
    source_size: PxSize,
}

struct RenderCoreFrameState<'a> {
    graphics: &'a crate::GraphicsDevice,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    targets: &'a FrameTargets,
    pipelines: &'a mut RenderPipelines,
    blit: &'a BlitState,
}

impl RenderCoreFrameState<'_> {
    fn target_size(&self) -> PxSize {
        PxSize::new(Px(self.config.width as i32), Px(self.config.height as i32))
    }
}

struct SubmitContext<'a> {
    graphics: &'a crate::GraphicsDevice,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
}

struct SubmitBatchParams<'a, 'b> {
    drawer: &'a mut Drawer,
    resources: SubmitContext<'a>,
    scene: (&'a wgpu::TextureView, PxRect),
    target_size: PxSize,
    clip_stack: &'b [PxRect],
    current_batch_draw_rect: &'a mut Option<PxRect>,
}

impl RenderCore {
    /// Renders the frame and presents it to the window surface, or keeps it in
    /// the offscreen copy when the core has no surface.
    pub(crate) fn render(
        &mut self,
        ops: Vec<RenderGraphOp>,
        #[cfg(feature = "debug-dirty-overlay")] dirty_overlay_rects: &[PxRect],
    ) {
        if self.surface.is_none() {
            let render_start = Instant::now();
            let output_view = self.offscreen_copy().clone();
            self.render_into_view(
                ops,
                &output_view,
                #[cfg(feature = "debug-dirty-overlay")]
                dirty_overlay_rects,
                render_start,
                std::time::Duration::ZERO,
            );
            return;
        }

        let render_start = Instant::now();
        self.last_render_breakdown = None;
        let acquire_start = Instant::now();
        let (output_frame, reconfigure_after_present) = {
            let surface = self
                .surface
                .as_ref()
                .expect("surface presence checked above");
            match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
                wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                    debug!("Surface suboptimal, rendering frame before reconfiguring...");
                    (frame, true)
                }
                wgpu::CurrentSurfaceTexture::Timeout => {
                    warn!("Surface timeout. Frame will be dropped.");
                    return;
                }
                wgpu::CurrentSurfaceTexture::Occluded => {
                    warn!("Surface occluded. Frame will be dropped.");
                    return;
                }
                wgpu::CurrentSurfaceTexture::Outdated => {
                    debug!("Surface outdated, reconfiguring...");
                    self.resize_surface();
                    return;
                }
                wgpu::CurrentSurfaceTexture::Lost => {
                    warn!("Surface lost, recreating surface...");
                    self.recreate_surface();
                    return;
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    error!("Surface validation error. Frame will be dropped.");
                    return;
                }
            }
        };
        let acquire = acquire_start.elapsed();
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_into_view(
            ops,
            &output_view,
            #[cfg(feature = "debug-dirty-overlay")]
            dirty_overlay_rects,
            render_start,
            acquire,
        );

        let present_start = Instant::now();
        self.queue.present(output_frame);
        if reconfigure_after_present {
            self.resize_surface();
        }
        if let Some(breakdown) = self.last_render_breakdown.as_mut() {
            breakdown.present = present_start.elapsed();
        }
    }

    fn render_into_view(
        &mut self,
        ops: Vec<RenderGraphOp>,
        output_view: &wgpu::TextureView,
        #[cfg(feature = "debug-dirty-overlay")] dirty_overlay_rects: &[PxRect],
        render_start: Instant,
        acquire: std::time::Duration,
    ) {
        let current_frame = self.frame_index;
        self.last_render_breakdown = None;

        let texture_size = wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        };
        let build_start = Instant::now();
        let mut passes = RenderPassGraph::build(ops, texture_size).into_passes();
        self.pipelines.clipper.prepare(
            &self.device,
            &self.queue,
            &mut passes,
            [self.config.width as f32, self.config.height as f32],
        );
        let build_passes = build_start.elapsed();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let encode_start = Instant::now();
        #[cfg(all(
            feature = "profiling",
            feature = "headless",
            not(target_arch = "wasm32")
        ))]
        if let Some(timing) = &mut self.gpu_timing {
            timing.begin(&self.device, &mut encoder, passes.len(), current_frame);
        }

        let mut state = RenderCoreFrameState {
            graphics: &self.graphics,
            device: &self.device,
            queue: &self.queue,
            config: &self.draw_config,
            targets: &self.targets,
            pipelines: &mut self.pipelines,
            blit: &self.blit,
        };
        state.pipelines.drawer.pipeline_registry.begin_all_frames(
            state.device,
            state.queue,
            state.config,
            state.graphics,
        );

        let mut scene = SceneState::Offscreen;
        let mut scene_written = false;
        let mut clip_stack: SmallVec<[PxRect; 16]> = SmallVec::new();
        for pass in &mut passes {
            let timing = tracing::enabled!(tracing::Level::DEBUG).then(Instant::now);
            Self::execute_render_pass(
                &mut state,
                &mut encoder,
                &mut scene,
                &mut scene_written,
                pass,
                &mut clip_stack,
            );
            #[cfg(all(
                feature = "profiling",
                feature = "headless",
                not(target_arch = "wasm32")
            ))]
            if let Some(timing) = &mut self.gpu_timing {
                timing.mark(&mut encoder, || match pass {
                    RenderPassPlan::Draw { reads_scene, .. } => {
                        format!("Draw reads_scene={reads_scene}")
                    }
                    RenderPassPlan::Compute(_) => "Compute".to_owned(),
                });
            }
            if let Some(timing) = timing {
                debug!(
                    elapsed_us = timing.elapsed().as_micros(),
                    frame_nanos = crate::current_frame_nanos(),
                    "render_pass_cpu"
                );
            }
        }
        drop(scene);

        let target_size = state.target_size();
        RenderCore::blit_to_view(BlitParams {
            encoder: &mut encoder,
            device: state.device,
            source: &state.targets.offscreen,
            target: output_view,
            bind_group_layout: &state.blit.bind_group_layout,
            sampler: &state.blit.sampler,
            pipeline: &state.blit.present_pipeline,
            source_origin: PxPosition::ZERO,
            source_size: target_size,
        });
        #[cfg(feature = "debug-dirty-overlay")]
        Self::render_dirty_overlay(
            &mut encoder,
            output_view,
            &state.blit.dirty_overlay_pipeline,
            target_size,
            dirty_overlay_rects,
        );

        state.pipelines.drawer.pipeline_registry.end_all_frames(
            state.device,
            state.queue,
            state.config,
            state.graphics,
        );
        let encode = encode_start.elapsed();

        #[cfg(all(
            feature = "profiling",
            feature = "headless",
            not(target_arch = "wasm32")
        ))]
        if let Some(timing) = &mut self.gpu_timing {
            timing.finish(&mut encoder);
        }

        let submit_start = Instant::now();
        let command_buffer = encoder.finish();
        let finish_time =
            tracing::enabled!(target: "tessera_ui::frame_cost", tracing::Level::DEBUG)
                .then(|| submit_start.elapsed());
        self.queue.submit(
            self.graphics
                .finish_prelude()
                .into_iter()
                .chain([command_buffer]),
        );
        let submit = submit_start.elapsed();
        if let Some(finish_time) = finish_time {
            debug!(
                target: "tessera_ui::frame_cost",
                frame_index = current_frame,
                encoder_finish_ns = finish_time.as_nanos(),
                queue_submit_ns = submit.saturating_sub(finish_time).as_nanos(),
                "render_submit_cpu"
            );
        }

        self.frame_index = self.frame_index.wrapping_add(1);
        self.last_render_breakdown = Some(RenderTimingBreakdown {
            acquire,
            build_passes,
            encode,
            submit,
            present: std::time::Duration::ZERO,
            total: render_start.elapsed(),
        });
    }

    /// Reads the last offscreen-rendered frame back as tightly packed RGBA8
    /// pixels in row-major order with a top-left origin.
    ///
    /// The returned buffer has `width * height * 4` bytes and no row padding.
    /// RGBA8 presentation bytes are sRGB-encoded even when the output texture
    /// uses a non-sRGB format, matching PNG and most image consumers.
    pub fn read_offscreen_rgba(&self) -> Result<Vec<u8>, OffscreenReadbackError> {
        self.read_texture_rgba(self.offscreen_copy().texture())
    }

    fn offscreen_copy(&self) -> &wgpu::TextureView {
        self.targets
            .offscreen_copy
            .as_ref()
            .expect("offscreen cores own a copy target")
    }

    fn read_texture_rgba(
        &self,
        texture: &wgpu::Texture,
    ) -> Result<Vec<u8>, OffscreenReadbackError> {
        let width = self.config.width;
        let height = self.config.height;
        if width == 0 || height == 0 {
            return Err(OffscreenReadbackError::EmptyTarget);
        }

        let unpadded_bytes_per_row = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = (padded_bytes_per_row as u64) * (height as u64);

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Offscreen Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Offscreen Readback Encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|err| OffscreenReadbackError::Poll(err.to_string()))?;
        match rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(err)) => return Err(OffscreenReadbackError::Map(err.to_string())),
            Err(err) => return Err(OffscreenReadbackError::Map(err.to_string())),
        }

        let mut pixels = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
        {
            let mapped = slice
                .get_mapped_range()
                .map_err(|err| OffscreenReadbackError::Map(err.to_string()))?;
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                pixels.extend_from_slice(&mapped[start..end]);
            }
        }
        buffer.unmap();
        Ok(pixels)
    }

    fn execute_render_pass(
        state: &mut RenderCoreFrameState<'_>,
        encoder: &mut wgpu::CommandEncoder,
        scene: &mut SceneState,
        scene_written: &mut bool,
        pass: &mut RenderPassPlan,
        clip_stack: &mut SmallVec<[PxRect; 16]>,
    ) {
        match pass {
            RenderPassPlan::Compute(items) => {
                if let Some(filtered) = Self::filter_scene(state, encoder, std::mem::take(items)) {
                    *scene = filtered;
                }
            }
            RenderPassPlan::Draw { draws, reads_scene } => {
                if draws.is_empty() {
                    return;
                }
                let target_size = state.target_size();
                let sample_copy;
                let sampled = if *reads_scene {
                    Some(match &*scene {
                        SceneState::Filtered { texture, rect } => (&**texture, *rect),
                        SceneState::Offscreen => {
                            let rect =
                                scene_sampling_rect(draws, target_size).unwrap_or_else(|| {
                                    PxRect::from_position_size(PxPosition::ZERO, target_size)
                                });
                            sample_copy = Self::copy_scene_region(state, encoder, rect);
                            (&*sample_copy, rect)
                        }
                    })
                } else {
                    None
                };
                render_current_pass(RenderPassParams {
                    msaa_view: state.targets.msaa_view.as_ref(),
                    stencil_view: &state.targets.stencil,
                    clipper: &mut state.pipelines.clipper,
                    clear_target: !*scene_written,
                    encoder,
                    write_target: &state.targets.offscreen,
                    commands_in_pass: draws,
                    scene: sampled,
                    drawer: &mut state.pipelines.drawer,
                    graphics: state.graphics,
                    device: state.device,
                    queue: state.queue,
                    config: state.config,
                    target_size,
                    clip_stack,
                });
                *scene_written = true;
                *scene = SceneState::Offscreen;
            }
        }
    }

    /// Copies the scene pixels that a sampling pass reads.
    fn copy_scene_region(
        state: &RenderCoreFrameState<'_>,
        encoder: &mut wgpu::CommandEncoder,
        rect: PxRect,
    ) -> ScratchTexture {
        let size = wgpu::Extent3d {
            width: rect.width.positive(),
            height: rect.height.positive(),
            depth_or_array_layers: 1,
        };
        let texture = state.graphics.scratch_texture(ScratchTextureDescriptor {
            width: size.width,
            height: size.height,
            format: state.config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            sample_count: 1,
            fit: ScratchFit::Approx,
        });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: state.targets.offscreen.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: rect.x.positive(),
                    y: rect.y.positive(),
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            texture.texture().as_image_copy(),
            size,
        );
        texture
    }

    /// Runs compute items over the scene region they read and write. Each batch
    /// starts from a copy of the previous result, so pixels a batch does not
    /// write keep their prior values.
    fn filter_scene(
        state: &mut RenderCoreFrameState<'_>,
        encoder: &mut wgpu::CommandEncoder,
        items: Vec<ComputePlanItem>,
    ) -> Option<SceneState> {
        let target_size = state.target_size();
        let region = items
            .iter()
            .map(|item| item.sampling_rect.union(&item.target_rect))
            .reduce(|a, b| a.union(&b))?
            .intersection(&PxRect::from_position_size(PxPosition::ZERO, target_size))?;
        if region.width <= Px::ZERO || region.height <= Px::ZERO {
            return None;
        }
        let origin = PxPosition::new(region.x, region.y);
        let descriptor = ScratchTextureDescriptor {
            width: region.width.positive(),
            height: region.height.positive(),
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            sample_count: 1,
            fit: ScratchFit::Approx,
        };
        let mut front = state.graphics.scratch_texture(descriptor);
        let mut back = state.graphics.scratch_texture(descriptor);
        Self::blit_to_view(BlitParams {
            encoder,
            device: state.device,
            source: &state.targets.offscreen,
            target: &front,
            bind_group_layout: &state.blit.bind_group_layout,
            sampler: &state.blit.sampler,
            pipeline: &state.blit.pipeline_rgba,
            source_origin: origin,
            source_size: target_size,
        });

        let translate = |rect: PxRect| PxRect {
            x: rect.x - region.x,
            y: rect.y - region.y,
            ..rect
        };
        let items: Vec<ComputePlanItem> = items
            .into_iter()
            .map(|item| ComputePlanItem {
                geometry: DrawGeometry::new(
                    item.geometry.local_size,
                    LocalToWindow::new(
                        item.geometry.local_to_window.scale(),
                        item.geometry.local_to_window.translation() - origin,
                    )
                    .expect("finite compute geometry"),
                ),
                target_rect: translate(item.target_rect),
                sampling_rect: translate(item.sampling_rect),
                ..item
            })
            .collect();
        let extent = wgpu::Extent3d {
            width: descriptor.width,
            height: descriptor.height,
            depth_or_array_layers: 1,
        };

        let mut index = 0;
        while index < items.len() {
            let type_id = items[index].command.as_any().type_id();
            let mut batch: SmallVec<[ErasedComputeBatchItem<'_>; 8]> = SmallVec::new();
            let mut sampling: SmallVec<[PxRect; 8]> = SmallVec::new();
            let mut cursor = index;
            // Items in one batch read the same input, so their reads must not
            // depend on each other's writes.
            while let Some(item) = items.get(cursor) {
                if item.command.as_any().type_id() != type_id
                    || (!batch.is_empty()
                        && sampling
                            .iter()
                            .any(|rect| rects_overlap(*rect, item.sampling_rect)))
                {
                    break;
                }
                sampling.push(item.sampling_rect);
                batch.push(ErasedComputeBatchItem {
                    command: &*item.command,
                    geometry: item.geometry,
                    target_area: item.target_rect,
                });
                cursor += 1;
            }

            encoder.copy_texture_to_texture(
                front.texture().as_image_copy(),
                back.texture().as_image_copy(),
                extent,
            );
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });
                state.pipelines.compute_registry.dispatch_erased(
                    ErasedDispatchContext {
                        graphics: state.graphics,
                        device: state.device,
                        queue: state.queue,
                        config: state.config,
                        target_size: PxSize::new(region.width, region.height),
                        compute_pass: &mut compute_pass,
                        input_view: &front,
                        output_view: &back,
                    },
                    &batch,
                );
            }
            std::mem::swap(&mut front, &mut back);
            index = cursor;
        }

        Some(SceneState::Filtered {
            texture: front,
            rect: region,
        })
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
            source_origin,
            source_size,
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
            label: Some("Blit Bind Group"),
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Blit Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    // Replace the destination. Blending against an undefined
                    // or stale target turns opaque scene colors black.
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        // Mapping the whole source onto a shifted viewport copies texels one to
        // one starting at `source_origin`.
        rpass.set_viewport(
            -source_origin.x,
            -source_origin.y,
            source_size.width.to_f32(),
            source_size.height.to_f32(),
            0.0,
            1.0,
        );
        rpass.draw(0..3, 0..1);
    }

    #[cfg(feature = "debug-dirty-overlay")]
    fn render_dirty_overlay(
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        target_size: PxSize,
        dirty_rects: &[PxRect],
    ) {
        if dirty_rects.is_empty() {
            return;
        }

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Dirty Overlay Pass"),
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
        for rect in dirty_rects {
            let Some(clamped) = clamp_rect_to_target(*rect, target_size) else {
                continue;
            };
            if clamped.width <= Px::ZERO || clamped.height <= Px::ZERO {
                continue;
            }
            set_scissor_rect_from_pxrect(&mut rpass, clamped);
            rpass.draw(0..3, 0..1);
        }
    }
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

fn render_current_pass(params: RenderPassParams<'_>) {
    let RenderPassParams {
        msaa_view,
        stencil_view,
        clipper,
        clear_target,
        encoder,
        write_target,
        commands_in_pass,
        scene,
        drawer,
        graphics,
        device,
        queue,
        config,
        target_size,
        clip_stack,
    } = params;

    let (view, resolve_target) = match msaa_view {
        Some(msaa_view) => (msaa_view, Some(write_target)),
        None => (write_target, None),
    };
    let load_ops = if clear_target {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
    } else {
        wgpu::LoadOp::Load
    };

    let timing = tracing::enabled!(tracing::Level::DEBUG).then(Instant::now);
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: stencil_view,
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: wgpu::StoreOp::Discard,
            }),
        }),
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

    clipper.restore(&mut rpass, target_size);

    // Draws that do not sample the scene never read this placeholder.
    let scene = scene.unwrap_or((
        write_target,
        PxRect::from_position_size(PxPosition::ZERO, target_size),
    ));
    let mut buffer: SmallVec<[(&dyn DrawCommand, DrawGeometry); 16]> = SmallVec::new();
    let mut last_command_type_id = None;
    let mut current_batch_draw_rect: Option<PxRect> = None;
    macro_rules! flush {
        () => {
            submit_buffered_commands(
                &mut rpass,
                SubmitBatchParams {
                    drawer: &mut *drawer,
                    resources: SubmitContext {
                        graphics,
                        device,
                        queue,
                        config,
                    },
                    scene,
                    target_size,
                    clip_stack: clip_stack.as_slice(),
                    current_batch_draw_rect: &mut current_batch_draw_rect,
                },
                &mut buffer,
            )
        };
    }
    for cmd in commands_in_pass.iter() {
        let cmd = match cmd {
            DrawOrClip::Clip(clip_ops) => {
                if !buffer.is_empty() {
                    flush!();
                    last_command_type_id = None;
                }
                match *clip_ops {
                    ClipOps::Push(rect) => {
                        clip_stack.push(rect);
                        clipper.push(&mut rpass, target_size, None);
                    }
                    ClipOps::ShapePush { region, index } => {
                        clip_stack.push(region.bounds());
                        clipper.push(&mut rpass, target_size, Some(index));
                    }
                    ClipOps::Pop => {
                        clip_stack.pop();
                        clipper.pop(&mut rpass, target_size);
                    }
                }
                continue;
            }
            DrawOrClip::Draw(cmd) => cmd,
        };

        if !can_merge_into_batch(&last_command_type_id, cmd.type_id) && !buffer.is_empty() {
            flush!();
        }
        buffer.push((cmd.command.as_ref(), cmd.geometry));
        last_command_type_id = Some(cmd.type_id);
        current_batch_draw_rect = Some(merge_batch_rect(current_batch_draw_rect, cmd.draw_rect));
    }
    if !buffer.is_empty() {
        flush!();
    }

    drop(buffer);
    commands_in_pass.clear();
    let record_us = timing.map(|start| start.elapsed().as_micros());
    let finalize_start = timing.map(|_| Instant::now());
    drop(rpass);
    if let Some(finalize_start) = finalize_start {
        debug!(
            record_us,
            finalize_us = finalize_start.elapsed().as_micros(),
            frame_nanos = crate::current_frame_nanos(),
            "draw_pass_cpu"
        );
    }
}

fn submit_buffered_commands(
    rpass: &mut wgpu::RenderPass<'_>,
    params: SubmitBatchParams<'_, '_>,
    buffer: &mut SmallVec<[(&dyn DrawCommand, DrawGeometry); 16]>,
) {
    let SubmitBatchParams {
        drawer,
        resources,
        scene,
        target_size,
        clip_stack,
        current_batch_draw_rect,
    } = params;
    let (current_clip_rect, anything_to_submit) =
        apply_clip_to_batch_rect(clip_stack, current_batch_draw_rect, target_size);
    if anything_to_submit && let Some(rect) = *current_batch_draw_rect {
        set_scissor_rect_from_pxrect(rpass, rect);
        drawer.submit(
            ErasedDrawContext {
                graphics: resources.graphics,
                device: resources.device,
                queue: resources.queue,
                config: resources.config,
                target_size,
                render_pass: rpass,
                scene_texture_view: scene.0,
                scene_rect: scene.1,
                clip_rect: current_clip_rect,
            },
            buffer.as_slice(),
        );
    }
    buffer.clear();
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
    let clipped_rect = if clip_stack.is_empty() {
        None
    } else {
        clip_stack.iter().copied().try_fold(
            PxRect::from_position_size(PxPosition::ZERO, target_size),
            |current, rect| {
                let clamped = clamp_rect_to_target(rect, target_size)?;
                current.intersection(&clamped)
            },
        )
    };

    let Some(current_rect) = current_batch_draw_rect.as_ref() else {
        return (clipped_rect, false);
    };

    let Some(mut final_rect) = clamp_rect_to_target(*current_rect, target_size) else {
        *current_batch_draw_rect = None;
        return (clipped_rect, false);
    };

    if !clip_stack.is_empty() && clipped_rect.is_none() {
        *current_batch_draw_rect = None;
        return (None, false);
    }

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

fn scene_sampling_rect(commands: &[DrawOrClip], target_size: PxSize) -> Option<PxRect> {
    let target = PxRect::new(Px::ZERO, Px::ZERO, target_size.width, target_size.height);
    let mut captured = None;
    for command in commands {
        let DrawOrClip::Draw(command) = command else {
            continue;
        };
        let rect = match command.command.sample_region()? {
            SampleRegion::Global => return Some(target),
            SampleRegion::Absolute(rect) => rect,
            SampleRegion::PaddedLocal(padding) => {
                command.geometry.padded_window_bounds(padding).outward()
            }
        };
        if let Some(rect) = rect.intersection(&target) {
            captured = Some(merge_batch_rect(captured, rect));
        }
    }
    Some(captured.unwrap_or(PxRect::ZERO))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    #[test]
    #[ignore = "requires a GPU adapter"]
    fn unorm_output_encodes_srgb_and_internal_linear_copies_remain_linear() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let core = pollster::block_on(RenderCore::new_offscreen(
            winit::dpi::PhysicalSize::new(1, 1),
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            crate::renderer::DEFAULT_MAX_NON_SAMPLER_BINDINGS,
        ));
        let filtered = core.graphics.scratch_texture(ScratchTextureDescriptor {
            width: 1,
            height: 1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            sample_count: 1,
            fit: ScratchFit::Exact,
        });
        for (target, pipeline, expected) in [
            (&*filtered, &core.blit.pipeline_rgba, 55_u8),
            (core.offscreen_copy(), &core.blit.present_pipeline, 128),
        ] {
            let mut encoder = core.device.create_command_encoder(&Default::default());
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &core.targets.offscreen,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.21404114,
                                g: 0.21404114,
                                b: 0.21404114,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
            }
            RenderCore::blit_to_view(BlitParams {
                encoder: &mut encoder,
                device: &core.device,
                source: &core.targets.offscreen,
                target,
                bind_group_layout: &core.blit.bind_group_layout,
                sampler: &core.blit.sampler,
                pipeline,
                source_origin: PxPosition::ZERO,
                source_size: PxSize::new(Px(1), Px(1)),
            });
            core.queue.submit(Some(encoder.finish()));
            let pixels = core
                .read_texture_rgba(target.texture())
                .expect("GPU pixels");
            for channel in &pixels[..3] {
                assert!(
                    channel.abs_diff(expected) <= 1,
                    "{pixels:?}, expected {expected}"
                );
            }
            assert_eq!(pixels[3], 255);
        }
    }

    #[derive(Clone)]
    struct SampleCommand(Option<SampleRegion>);

    impl DrawCommand for SampleCommand {
        fn sample_region(&self) -> Option<SampleRegion> {
            self.0
        }

        fn apply_opacity(&mut self, _opacity: f32) {}
    }

    fn sampled_draw(region: Option<SampleRegion>, position: PxPosition) -> DrawOrClip {
        let size = PxSize::new(Px(50), Px(50));
        DrawOrClip::Draw(crate::render_pass::DrawCommandWithMetadata {
            command: Box::new(SampleCommand(region)),
            type_id: std::any::TypeId::of::<SampleCommand>(),
            geometry: DrawGeometry::from_position_size(position, size),
            draw_rect: PxRect::from_position_size(position, size),
        })
    }

    #[test]
    fn scene_capture_unions_sampling_regions_without_applying_draw_clips() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let commands = [
            DrawOrClip::Clip(ClipOps::Push(PxRect::new(Px(100), Px(100), Px(1), Px(1)))),
            sampled_draw(
                Some(SampleRegion::uniform_padding_local(Px(10))),
                PxPosition::new(Px(100), Px(100)),
            ),
            sampled_draw(
                Some(SampleRegion::Absolute(PxRect::new(
                    Px(300),
                    Px(100),
                    Px(20),
                    Px(20),
                ))),
                PxPosition::ZERO,
            ),
        ];
        assert_eq!(
            scene_sampling_rect(&commands, PxSize::new(Px(400), Px(400))),
            Some(PxRect::new(Px(90), Px(90), Px(230), Px(70)))
        );
    }

    #[test]
    fn scene_capture_rounds_fractional_sampling_edges_outward() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let commands = [sampled_draw(
            Some(SampleRegion::uniform_padding_local(Px(10))),
            PxPosition::from_f32_xy(100.25, 200.75),
        )];
        assert_eq!(
            scene_sampling_rect(&commands, PxSize::new(Px(400), Px(400))),
            Some(PxRect::new(Px(90), Px(190), Px(71), Px(71)))
        );
    }

    #[test]
    fn scene_capture_clamps_regions_at_screen_edges() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let commands = [sampled_draw(
            Some(SampleRegion::uniform_padding_local(Px(10))),
            PxPosition::from_f32_xy(-5.25, -6.75),
        )];
        assert_eq!(
            scene_sampling_rect(&commands, PxSize::new(Px(400), Px(400))),
            Some(PxRect::new(Px::ZERO, Px::ZERO, Px(55), Px(54)))
        );
    }

    #[test]
    fn scene_capture_preserves_global_and_undeclared_scene_reads() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let target_size = PxSize::new(Px(400), Px(400));
        assert_eq!(
            scene_sampling_rect(
                &[sampled_draw(Some(SampleRegion::Global), PxPosition::ZERO)],
                target_size
            ),
            Some(PxRect::new(Px::ZERO, Px::ZERO, Px(400), Px(400)))
        );
        assert_eq!(
            scene_sampling_rect(&[sampled_draw(None, PxPosition::ZERO)], target_size),
            None
        );
    }

    #[test]
    fn apply_clip_to_batch_rect_intersects_full_clip_stack() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let target_size = PxSize::new(Px(400), Px(400));
        let clip_stack = [
            PxRect::new(Px(0), Px(0), Px(200), Px(200)),
            PxRect::new(Px(50), Px(50), Px(200), Px(200)),
        ];
        let mut batch_rect = Some(PxRect::new(Px(25), Px(25), Px(200), Px(200)));

        let (clip_rect, should_submit) =
            apply_clip_to_batch_rect(&clip_stack, &mut batch_rect, target_size);

        assert!(should_submit);
        assert_eq!(
            clip_rect,
            Some(PxRect::new(Px(50), Px(50), Px(150), Px(150)))
        );
        assert_eq!(
            batch_rect,
            Some(PxRect::new(Px(50), Px(50), Px(150), Px(150)))
        );
    }

    #[test]
    fn apply_clip_to_batch_rect_drops_batch_when_nested_clips_do_not_overlap() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let target_size = PxSize::new(Px(400), Px(400));
        let clip_stack = [
            PxRect::new(Px(0), Px(0), Px(100), Px(100)),
            PxRect::new(Px(200), Px(200), Px(100), Px(100)),
        ];
        let mut batch_rect = Some(PxRect::new(Px(0), Px(0), Px(300), Px(300)));

        let (clip_rect, should_submit) =
            apply_clip_to_batch_rect(&clip_stack, &mut batch_rect, target_size);

        assert!(!should_submit);
        assert_eq!(clip_rect, None);
        assert_eq!(batch_rect, None);
    }
}
