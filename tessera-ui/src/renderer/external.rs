//! External render textures owned by pipelines.
//!
//! ## Usage
//!
//! Keep persistent atlases and caches across frames for render pipelines.

use std::{collections::HashMap, sync::Arc};

use parking_lot::{RwLock, RwLockWriteGuard};

use crate::{
    PxSize,
    render_graph::{ExternalTextureDesc, RenderTextureDesc},
};

/// Registry of persistent textures that are owned by render pipelines.
#[derive(Clone)]
pub struct ExternalTextureRegistry {
    inner: Arc<RwLock<ExternalTextureRegistryInner>>,
}

impl ExternalTextureRegistry {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExternalTextureRegistryInner::default())),
        }
    }

    /// Allocate a new persistent texture and return its handle.
    pub fn allocate(
        &self,
        device: &wgpu::Device,
        desc: RenderTextureDesc,
        sample_count: u32,
    ) -> ExternalTextureHandle {
        let mut inner = self.inner.write();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        let entry = ExternalTextureEntry::new(device, desc.clone(), sample_count);
        inner.entries.insert(id, entry);
        ExternalTextureHandle {
            id,
            desc,
            sample_count,
            registry: self.clone(),
        }
    }

    /// Ensure the texture backing the handle matches the requested description.
    pub fn ensure(
        &self,
        device: &wgpu::Device,
        handle: &mut ExternalTextureHandle,
        desc: RenderTextureDesc,
        sample_count: u32,
    ) {
        if handle.desc == desc && handle.sample_count == sample_count {
            return;
        }
        let mut inner = self.inner.write();
        if let Some(entry) = inner.entries.get_mut(&handle.id) {
            entry.rebuild(device, desc.clone(), sample_count);
        } else {
            let entry = ExternalTextureEntry::new(device, desc.clone(), sample_count);
            inner.entries.insert(handle.id, entry);
        }
        handle.desc = desc;
        handle.sample_count = sample_count;
    }

    /// Mark an external texture as used in the current frame.
    pub fn mark_used(&self, id: u32, frame_index: u64) {
        if let Some(entry) = self.inner.write().entries.get_mut(&id) {
            entry.last_used_frame = frame_index;
        }
    }

    /// Remove unused textures after a delay once all handles are dropped.
    pub fn collect_garbage(&self, frame_index: u64, delay_frames: u64) {
        let mut inner = self.inner.write();
        inner.entries.retain(|_, entry| {
            if entry.ref_count > 0 {
                return true;
            }
            frame_index <= entry.last_used_frame.saturating_add(delay_frames)
        });
    }

    pub(crate) fn slot(&self, id: u32) -> Option<ExternalTextureSlotGuard<'_>> {
        let guard = self.inner.write();
        guard
            .entries
            .contains_key(&id)
            .then_some(ExternalTextureSlotGuard { guard, id })
    }

    fn add_ref(&self, id: u32) {
        self.inner.write().add_ref(id);
    }

    fn release(&self, id: u32) {
        self.inner.write().release(id);
    }
}

/// Handle to a persistent external texture stored in the registry.
pub struct ExternalTextureHandle {
    id: u32,
    desc: RenderTextureDesc,
    sample_count: u32,
    registry: ExternalTextureRegistry,
}

impl ExternalTextureHandle {
    /// Return the stable id for this texture.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Build a render-graph descriptor for this texture.
    pub fn desc(&self, clear_on_first_use: bool) -> ExternalTextureDesc {
        ExternalTextureDesc {
            handle_id: self.id,
            size: self.desc.size,
            format: self.desc.format,
            sample_count: self.sample_count,
            clear_on_first_use,
        }
    }

    /// Ensure the registry texture matches the provided description.
    pub fn ensure(
        &mut self,
        registry: &ExternalTextureRegistry,
        device: &wgpu::Device,
        desc: RenderTextureDesc,
        sample_count: u32,
    ) {
        registry.ensure(device, self, desc, sample_count);
    }
}

impl Clone for ExternalTextureHandle {
    fn clone(&self) -> Self {
        self.registry.add_ref(self.id);
        Self {
            id: self.id,
            desc: self.desc.clone(),
            sample_count: self.sample_count,
            registry: self.registry.clone(),
        }
    }
}

impl Drop for ExternalTextureHandle {
    fn drop(&mut self) {
        self.registry.release(self.id);
    }
}

pub(crate) struct ExternalTextureSlotGuard<'a> {
    guard: RwLockWriteGuard<'a, ExternalTextureRegistryInner>,
    id: u32,
}

impl ExternalTextureSlotGuard<'_> {
    pub fn size(&self) -> PxSize {
        self.entry().desc.size
    }

    pub fn front_view(&self) -> wgpu::TextureView {
        self.entry().front.clone()
    }

    pub fn back_view(&self) -> wgpu::TextureView {
        self.entry().back.clone()
    }

    pub fn msaa_view(&self) -> Option<wgpu::TextureView> {
        self.entry().msaa_view.clone()
    }

    pub fn swap_front_back(&mut self) {
        let entry = self.entry_mut();
        std::mem::swap(&mut entry.front, &mut entry.back);
    }

    fn entry(&self) -> &ExternalTextureEntry {
        self.guard
            .entries
            .get(&self.id)
            .expect("missing external texture entry")
    }

    fn entry_mut(&mut self) -> &mut ExternalTextureEntry {
        self.guard
            .entries
            .get_mut(&self.id)
            .expect("missing external texture entry")
    }
}

#[derive(Default)]
struct ExternalTextureRegistryInner {
    next_id: u32,
    entries: HashMap<u32, ExternalTextureEntry>,
}

impl ExternalTextureRegistryInner {
    fn add_ref(&mut self, id: u32) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.ref_count = entry.ref_count.saturating_add(1);
        }
    }

    fn release(&mut self, id: u32) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
        }
    }
}

struct ExternalTextureEntry {
    desc: RenderTextureDesc,
    sample_count: u32,
    front: wgpu::TextureView,
    back: wgpu::TextureView,
    msaa_view: Option<wgpu::TextureView>,
    last_used_frame: u64,
    ref_count: u32,
}

impl ExternalTextureEntry {
    fn new(device: &wgpu::Device, desc: RenderTextureDesc, sample_count: u32) -> Self {
        let (front, back, msaa_view) = create_views(device, &desc, sample_count);
        Self {
            desc,
            sample_count,
            front,
            back,
            msaa_view,
            last_used_frame: 0,
            ref_count: 1,
        }
    }

    fn rebuild(&mut self, device: &wgpu::Device, desc: RenderTextureDesc, sample_count: u32) {
        let (front, back, msaa_view) = create_views(device, &desc, sample_count);
        self.desc = desc;
        self.sample_count = sample_count;
        self.front = front;
        self.back = back;
        self.msaa_view = msaa_view;
    }
}

fn create_views(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    sample_count: u32,
) -> (
    wgpu::TextureView,
    wgpu::TextureView,
    Option<wgpu::TextureView>,
) {
    let front = create_texture_view(device, desc, "External Front");
    let back = create_texture_view(device, desc, "External Back");
    let msaa_view = if sample_count > 1 {
        Some(create_msaa_view(device, desc, sample_count))
    } else {
        None
    };
    (front, back, msaa_view)
}

fn create_texture_view(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    label: &str,
) -> wgpu::TextureView {
    let width = desc.size.width.positive().max(1);
    let height = desc.size.height.positive().max(1);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: desc.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_msaa_view(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    sample_count: u32,
) -> wgpu::TextureView {
    let width = desc.size.width.positive().max(1);
    let height = desc.size.height.positive().max(1);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("External MSAA"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: desc.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
