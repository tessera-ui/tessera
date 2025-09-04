use std::collections::HashMap;

pub type ComputeResource = wgpu::Buffer;
pub type ComputeResourceRef = usize;

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
