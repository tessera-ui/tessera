//! Render and compute pipelines backing the basic components.
//!
//! ## Usage
//!
//! Register pipelines during renderer initialization before rendering
//! components.

pub(crate) mod blur;
pub(crate) mod checkmark;
pub(crate) mod contrast;
pub(crate) mod fluid_glass;
pub(crate) mod image;
pub(crate) mod image_vector;
pub(crate) mod mean;
pub(crate) mod pos_misc;
pub(crate) mod progress_arc;
pub(crate) mod shadow;
pub(crate) mod shape;
pub(crate) mod simple_rect;
pub(crate) mod text;

mod composite;
mod compute;
mod draw;

/// Register all draw and compute pipelines required by this crate.
pub fn register_pipelines(context: &mut tessera_ui::PipelineContext<'_>) {
    draw::register(context);
    compute::register(context);
    composite::register(context);
}

use std::{cell::RefCell, sync::Arc};

use tessera_ui::wgpu::{self, util::DeviceExt};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UnitQuadVertex {
    position: [f32; 2],
}

/// Shared unit-quad buffers for rectangle-based drawing.
#[derive(Clone)]
pub(crate) struct SharedQuad {
    vertex: Arc<wgpu::Buffer>,
    index: Arc<wgpu::Buffer>,
}

struct SharedQuadSlot {
    device: wgpu::Device,
    quad: SharedQuad,
}

thread_local! {
    static SHARED_QUAD: RefCell<Option<SharedQuadSlot>> = const { RefCell::new(None) };
}

impl SharedQuad {
    /// Returns the shared (0,0)-(1,1) quad buffers for `device`.
    ///
    /// Repeated calls on the same device and thread reuse the buffers.
    pub(crate) fn get(device: &wgpu::Device) -> Self {
        SHARED_QUAD.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(slot) = cache.as_ref()
                && slot.device == *device
            {
                return slot.quad.clone();
            }
            let quad = Self::create(device);
            *cache = Some(SharedQuadSlot {
                device: device.clone(),
                quad: quad.clone(),
            });
            quad
        })
    }

    fn create(device: &wgpu::Device) -> Self {
        let vertices = [
            UnitQuadVertex {
                position: [0.0, 0.0],
            },
            UnitQuadVertex {
                position: [1.0, 0.0],
            },
            UnitQuadVertex {
                position: [1.0, 1.0],
            },
            UnitQuadVertex {
                position: [0.0, 1.0],
            },
        ];
        let vertex = Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shared Unit Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );

        let indices: [u16; 6] = [0, 2, 1, 0, 3, 2];
        let index = Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shared Unit Quad Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
        );

        Self { vertex, index }
    }

    pub(crate) fn vertex(&self) -> &Arc<wgpu::Buffer> {
        &self.vertex
    }

    pub(crate) fn index(&self) -> &Arc<wgpu::Buffer> {
        &self.index
    }
}
