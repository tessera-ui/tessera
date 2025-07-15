use std::collections::HashMap;

use uuid::Uuid;

pub type ComputeResource = wgpu::Buffer;
pub type ComputeResourceRef = uuid::Uuid;

#[derive(Debug)]
pub struct ComputeResourceManager {
    resources: HashMap<Uuid, ComputeResource>,
}

impl Default for ComputeResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Clear all resources.
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    /// Move a buffer into the resource manager.
    pub fn push(&mut self, buffer: wgpu::Buffer) -> ComputeResourceRef {
        let id = Uuid::new_v4();
        self.resources.insert(id, buffer);
        id
    }

    /// Access a resource in ref by its ID.
    pub fn get(&self, id: &ComputeResourceRef) -> Option<&ComputeResource> {
        self.resources.get(id)
    }

    /// Remove a resource by its ID.
    pub fn remove(&mut self, id: &ComputeResourceRef) -> Option<ComputeResource> {
        self.resources.remove(id)
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
