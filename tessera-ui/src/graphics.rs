//! Renderer-owned graphics resources and upload observations.
//!
//! ## Usage
//!
//! Reuse pipeline buffers and inspect actual resource allocations and uploads.

use std::{ops::Deref, sync::Arc};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use wgpu::util::DeviceExt as _;

/// Actual resource operations for a pipeline's declared resource group.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GraphicsWork {
    /// Number of buffers created.
    pub buffer_allocations: usize,
    /// Number of queue buffer writes.
    pub buffer_uploads: usize,
    /// Bytes submitted through queue buffer writes.
    pub buffer_uploaded_bytes: u64,
    /// Number of textures created.
    pub texture_allocations: usize,
    /// Number of queue texture writes.
    pub texture_uploads: usize,
    /// Bytes submitted through queue texture writes.
    pub texture_uploaded_bytes: u64,
}

/// Cumulative graphics work owned by one renderer.
#[derive(Clone, Debug, Default)]
pub struct GraphicsWorkDiagnostics {
    owner: u128,
    groups: FxHashMap<&'static str, GraphicsWork>,
}

impl GraphicsWorkDiagnostics {
    /// Returns operations after an earlier snapshot from the same renderer.
    pub fn since(&self, earlier: &Self) -> Self {
        assert_eq!(
            self.owner, earlier.owner,
            "graphics snapshots belong to different renderers"
        );
        assert!(
            earlier
                .groups
                .keys()
                .all(|name| self.groups.contains_key(name)),
            "graphics baseline is newer than snapshot"
        );
        let mut difference = self.clone();
        for (name, value) in &mut difference.groups {
            let before = earlier.group(name);
            value.buffer_allocations = value
                .buffer_allocations
                .checked_sub(before.buffer_allocations)
                .expect("graphics baseline is newer than snapshot");
            value.buffer_uploads = value
                .buffer_uploads
                .checked_sub(before.buffer_uploads)
                .expect("graphics baseline is newer than snapshot");
            value.buffer_uploaded_bytes = value
                .buffer_uploaded_bytes
                .checked_sub(before.buffer_uploaded_bytes)
                .expect("graphics baseline is newer than snapshot");
            value.texture_allocations = value
                .texture_allocations
                .checked_sub(before.texture_allocations)
                .expect("graphics baseline is newer than snapshot");
            value.texture_uploads = value
                .texture_uploads
                .checked_sub(before.texture_uploads)
                .expect("graphics baseline is newer than snapshot");
            value.texture_uploaded_bytes = value
                .texture_uploaded_bytes
                .checked_sub(before.texture_uploaded_bytes)
                .expect("graphics baseline is newer than snapshot");
        }
        difference
    }
    /// Returns operations for a declared pipeline resource group.
    pub fn group(&self, group: &str) -> GraphicsWork {
        self.groups.get(group).copied().unwrap_or_default()
    }
}

/// A graphics service bound to one renderer's device and upload queue.
#[derive(Clone, Debug)]
pub struct GraphicsDevice {
    device: wgpu::Device,
    queue: wgpu::Queue,
    work: Arc<Mutex<GraphicsWorkDiagnostics>>,
}

impl GraphicsDevice {
    pub(crate) fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            work: Arc::new(Mutex::new(GraphicsWorkDiagnostics {
                owner: uuid::Uuid::new_v4().as_u128(),
                groups: FxHashMap::default(),
            })),
        }
    }

    /// Returns the device used by this renderer.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns the upload queue used by this renderer.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Captures actual operations performed through this graphics service.
    pub fn diagnostics(&self) -> GraphicsWorkDiagnostics {
        self.work.lock().clone()
    }

    /// Creates an owned buffer in a pipeline resource group.
    pub fn create_buffer(
        &self,
        group: &'static str,
        descriptor: &wgpu::BufferDescriptor<'_>,
    ) -> TrackedBuffer {
        let buffer = self.device.create_buffer(descriptor);
        self.work
            .lock()
            .groups
            .entry(group)
            .or_default()
            .buffer_allocations += 1;
        TrackedBuffer {
            buffer,
            group,
            graphics: self.clone(),
        }
    }

    /// Creates a buffer containing its initial data.
    pub fn create_buffer_init(
        &self,
        group: &'static str,
        descriptor: &wgpu::util::BufferInitDescriptor<'_>,
    ) -> TrackedBuffer {
        let buffer = self.device.create_buffer_init(descriptor);
        self.work
            .lock()
            .groups
            .entry(group)
            .or_default()
            .buffer_allocations += 1;
        TrackedBuffer {
            buffer,
            group,
            graphics: self.clone(),
        }
    }

    /// Uploads bytes to a buffer owned by this renderer.
    pub fn write_buffer(&self, buffer: &TrackedBuffer, offset: u64, data: &[u8]) {
        assert!(
            Arc::ptr_eq(&self.work, &buffer.graphics.work),
            "buffer belongs to another renderer"
        );
        self.queue.write_buffer(&buffer.buffer, offset, data);
        let mut work = self.work.lock();
        let group = work.groups.entry(buffer.group).or_default();
        group.buffer_uploads += 1;
        group.buffer_uploaded_bytes += data.len() as u64;
    }

    /// Creates a texture in a pipeline resource group.
    pub fn create_texture(
        &self,
        group: &'static str,
        descriptor: &wgpu::TextureDescriptor<'_>,
    ) -> wgpu::Texture {
        let texture = self.device.create_texture(descriptor);
        self.work
            .lock()
            .groups
            .entry(group)
            .or_default()
            .texture_allocations += 1;
        texture
    }

    /// Uploads texture bytes through this renderer's queue.
    pub fn write_texture(
        &self,
        group: &'static str,
        destination: wgpu::TexelCopyTextureInfo<'_>,
        data: &[u8],
        layout: wgpu::TexelCopyBufferLayout,
        size: wgpu::Extent3d,
    ) {
        self.queue.write_texture(destination, data, layout, size);
        let mut work = self.work.lock();
        let group = work.groups.entry(group).or_default();
        group.texture_uploads += 1;
        group.texture_uploaded_bytes += data.len() as u64;
    }
}

impl Deref for GraphicsDevice {
    type Target = wgpu::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

/// A buffer whose resource group and renderer owner stay attached to uploads.
#[derive(Clone, Debug)]
pub struct TrackedBuffer {
    buffer: wgpu::Buffer,
    group: &'static str,
    graphics: GraphicsDevice,
}

impl Deref for TrackedBuffer {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
