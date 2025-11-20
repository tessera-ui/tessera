//! Compute resource management for sharing GPU buffers.

use std::collections::HashMap;

/// Type alias for compute resources stored in the manager.
pub type ComputeResource = wgpu::Buffer;
/// Opaque identifier for a compute resource.
pub type ComputeResourceRef = usize;

/// Manages reusable GPU buffers for compute workloads.
#[derive(Debug)]
pub struct ComputeResourceManager {
    idx: usize,
    resources: HashMap<usize, ComputeResource>,
}

impl Default for ComputeResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeResourceManager {
    /// Creates an empty compute resource manager.
    pub fn new() -> Self {
        Self {
            idx: 0,
            resources: HashMap::new(),
        }
    }

    /// Clear all resources.
    pub fn clear(&mut self) {
        self.idx = 0;
        self.resources.clear();
    }

    /// Move a buffer into the resource manager.
    pub fn push(&mut self, buffer: wgpu::Buffer) -> ComputeResourceRef {
        self.resources.insert(self.idx, buffer);
        let id = self.idx;
        self.idx += 1;
        id
    }

    /// Access a resource in ref by its ID.
    pub fn get(&self, id: &ComputeResourceRef) -> Option<&ComputeResource> {
        self.resources.get(id)
    }

    /// Check if a resource exists by its ID.
    pub fn contains(&self, id: &ComputeResourceRef) -> bool {
        self.resources.contains_key(id)
    }

    /// Move a resource out of the manager.
    pub fn take(&mut self, id: &ComputeResourceRef) -> Option<ComputeResource> {
        self.resources.remove(id)
    }
}
