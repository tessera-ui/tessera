//! Indexed storage for live component node values.

use std::num::NonZeroUsize;

use crate::NodeId;

pub(super) struct DenseNodeMap<T> {
    entries: Vec<(NodeId, T)>,
    indices: Vec<Option<NonZeroUsize>>,
}

impl<T> Default for DenseNodeMap<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl<T> DenseNodeMap<T> {
    fn slot(id: NodeId) -> usize {
        usize::from(id) - 1
    }

    fn slot_index(&self, id: NodeId) -> Option<usize> {
        self.indices
            .get(Self::slot(id))?
            .map(|index| index.get() - 1)
    }

    fn index(&self, id: NodeId) -> Option<usize> {
        let index = self.slot_index(id)?;
        (self.entries[index].0 == id).then_some(index)
    }

    pub(super) fn id_at_slot(&self, id: NodeId) -> Option<NodeId> {
        Some(self.entries[self.slot_index(id)?].0)
    }

    pub(super) fn get(&self, id: NodeId) -> Option<&T> {
        Some(&self.entries[self.index(id)?].1)
    }

    pub(super) fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let index = self.index(id)?;
        Some(&mut self.entries[index].1)
    }

    pub(super) fn insert(&mut self, id: NodeId, value: T) -> &mut T {
        if let Some(index) = self.slot_index(id) {
            self.entries[index] = (id, value);
            return &mut self.entries[index].1;
        }
        let slot = Self::slot(id);
        if self.indices.len() <= slot {
            self.indices.resize(slot + 1, None);
        }
        let index = self.entries.len();
        self.entries.push((id, value));
        self.indices[slot] = NonZeroUsize::new(index + 1);
        &mut self.entries[index].1
    }

    pub(super) fn remove(&mut self, id: NodeId) -> Option<T> {
        let index = self.index(id)?;
        self.indices[Self::slot(id)] = None;
        let (_, value) = self.entries.swap_remove(index);
        if let Some((moved, _)) = self.entries.get(index) {
            self.indices[Self::slot(*moved)] = NonZeroUsize::new(index + 1);
        }
        Some(value)
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.indices.clear();
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.entries.iter().map(|(id, value)| (*id, value))
    }

    pub(super) fn trim_spare_capacity(&mut self) {
        while self.indices.last().is_some_and(Option::is_none) {
            self.indices.pop();
        }
        if self.indices.len() < self.indices.capacity() / 4 {
            self.indices.shrink_to(self.indices.len() * 2);
        }
        if self.entries.len() < self.entries.capacity() / 4 {
            self.entries.shrink_to(self.entries.len() * 2);
        }
    }
}
