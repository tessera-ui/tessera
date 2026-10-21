//! Pointer targeting and ordered input delivery.
//!
//! ## Usage
//!
//! Route mouse, touch, and hover interactions to placed controls.

use std::cell::{Cell, RefCell};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{NodeId, PxPosition, PxRect, PxSize, modifier::OrderedModifierAction};

use super::{
    ComponentNodeMetaDatas, ComponentNodeTree, NodeRole, PointerDispatchPlan, PointerNodePlan,
    bounds_hit_path, resolve_node_input_context,
};

const CELL_SIZE: f32 = 256.0;
const MAX_CELLS: usize = 256;

struct Entry {
    plan: PointerNodePlan,
    order: Vec<usize>,
    cells: Vec<(i32, i32)>,
}

#[derive(Default)]
pub(super) struct PointerIndex {
    entries: HashMap<NodeId, Entry>,
    cells: HashMap<(i32, i32), HashSet<NodeId>>,
    oversized: HashSet<NodeId>,
    pending: HashSet<NodeId>,
    hovered: HashSet<NodeId>,
    changed: HashSet<NodeId>,
    removed: HashSet<NodeId>,
    hit_cache: RefCell<HashMap<(u32, u32), HashSet<NodeId>>>,
    pub(super) hit_candidates: Cell<u64>,
}

fn cell(position: PxPosition) -> (i32, i32) {
    (
        (position.x / CELL_SIZE).floor() as i32,
        (position.y / CELL_SIZE).floor() as i32,
    )
}

impl PointerIndex {
    pub(super) fn begin_dispatch(&self) {
        self.hit_cache.borrow_mut().clear();
        self.hit_candidates.set(0);
    }
    pub(super) fn invalidate(&mut self, changed: &HashSet<NodeId>, removed: &HashSet<NodeId>) {
        self.changed.extend(changed);
        self.removed.extend(removed);
    }

    pub(super) fn update(
        &mut self,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        order: &HashMap<NodeId, usize>,
        changed: &HashSet<NodeId>,
        removed: &HashSet<NodeId>,
    ) -> usize {
        let mut removed = removed.clone();
        removed.extend(std::mem::take(&mut self.removed));
        let mut changed = changed.clone();
        changed.extend(std::mem::take(&mut self.changed));
        for &id in &removed {
            self.remove(id);
            self.hovered.remove(&id);
            self.pending.remove(&id);
        }
        for &id in &changed {
            self.remove(id);
            let Some(node) = tree.get(id).filter(|node| !node.is_removed()) else {
                continue;
            };
            let node = node.get();
            if node.role != NodeRole::Layout {
                continue;
            }
            let actions = node.modifier.ordered_actions();
            let raw_input = !node.pointer_preview_handlers.is_empty()
                || !node.pointer_handlers.is_empty()
                || !node.pointer_final_handlers.is_empty();
            if !raw_input
                && !actions.iter().any(|action| {
                    matches!(
                        action,
                        OrderedModifierAction::Cursor(_)
                            | OrderedModifierAction::PointerPreviewInput(_)
                            | OrderedModifierAction::PointerInput(_)
                            | OrderedModifierAction::PointerFinalInput(_)
                    )
                })
            {
                continue;
            }
            let Some(metadata) = metadatas.get(&id) else {
                continue;
            };
            if metadata.abs_position.is_none() {
                continue;
            }
            let Some(context) = resolve_node_input_context(tree, metadatas, id) else {
                continue;
            };
            let size = PxSize::new(
                context.node_computed_data.width,
                context.node_computed_data.height,
            );
            let mut bounds = raw_input.then(|| PxRect::from_position_size(context.abs_pos, size));
            let mut position = context.base_abs_pos;
            for action in &actions {
                match action {
                    OrderedModifierAction::Placement(placement) => {
                        position = placement.node().transform_position(position);
                    }
                    OrderedModifierAction::Cursor(_)
                    | OrderedModifierAction::PointerPreviewInput(_)
                    | OrderedModifierAction::PointerInput(_)
                    | OrderedModifierAction::PointerFinalInput(_) => {
                        let rect = PxRect::from_position_size(position, size);
                        bounds = Some(bounds.map_or(rect, |bounds| bounds.union(&rect)));
                    }
                    _ => {}
                }
            }
            let Some(mut bounds) = bounds else { continue };
            let mut exposed = bounds.width.0 > 0 && bounds.height.0 > 0;
            if let Some(clip) = context.event_clip_rect {
                if let Some(clipped) = bounds.intersection(&clip) {
                    bounds = clipped;
                } else {
                    exposed = false;
                }
            }
            let min = cell(PxPosition::new(bounds.x, bounds.y));
            let max = cell(PxPosition::from_f32_xy(
                bounds.x.0 as f32 + bounds.width.0 as f32,
                bounds.y.0 as f32 + bounds.height.0 as f32,
            ));
            let width = (i64::from(max.0) - i64::from(min.0) + 1).max(0);
            let height = (i64::from(max.1) - i64::from(min.1) + 1).max(0);
            let mut cells = Vec::new();
            if exposed && width.saturating_mul(height) > MAX_CELLS as i64 {
                self.oversized.insert(id);
            } else if exposed {
                for x in min.0..=max.0 {
                    for y in min.1..=max.1 {
                        self.cells.entry((x, y)).or_default().insert(id);
                        cells.push((x, y));
                    }
                }
            }
            let mut path: Vec<_> = id
                .ancestors(tree)
                .map(|id| order.get(&id).copied().unwrap_or_default())
                .collect();
            path.reverse();
            self.entries.insert(
                id,
                Entry {
                    plan: PointerNodePlan { context, actions },
                    order: path,
                    cells,
                },
            );
            self.pending.insert(id);
        }
        changed.len()
    }

    fn remove(&mut self, id: NodeId) {
        if let Some(entry) = self.entries.remove(&id) {
            for cell in entry.cells {
                if let Some(nodes) = self.cells.get_mut(&cell) {
                    nodes.remove(&id);
                    if nodes.is_empty() {
                        self.cells.remove(&cell);
                    }
                }
            }
        }
        self.oversized.remove(&id);
    }

    fn hits(
        &self,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        position: Option<PxPosition>,
    ) -> HashSet<NodeId> {
        let Some(position) = position else {
            return HashSet::default();
        };
        let key = (position.x.to_bits(), position.y.to_bits());
        if let Some(hits) = self.hit_cache.borrow().get(&key) {
            return hits.clone();
        }
        let hits: HashSet<_> = self
            .cells
            .get(&cell(position))
            .into_iter()
            .flatten()
            .chain(&self.oversized)
            .copied()
            .filter(|&id| {
                self.hit_candidates.set(self.hit_candidates.get() + 1);
                bounds_hit_path(id, tree, metadatas, position).is_some()
            })
            .collect();
        self.hit_cache.borrow_mut().insert(key, hits.clone());
        hits
    }

    pub(super) fn sample_targets(
        &self,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        position: Option<PxPosition>,
    ) -> HashSet<NodeId> {
        self.hits(tree, metadatas, position)
    }

    pub(super) fn hit_path(
        &self,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        position: Option<PxPosition>,
    ) -> Vec<NodeId> {
        let hits = self.hits(tree, metadatas, position);
        let Some(&leaf) = hits
            .iter()
            .max_by(|a, b| self.entries[a].order.cmp(&self.entries[b].order))
        else {
            return Vec::new();
        };
        let mut path: Vec<_> = leaf
            .ancestors(tree)
            .filter(|id| {
                tree.get(*id)
                    .is_some_and(|node| node.get().role == NodeRole::Layout)
                    && metadatas
                        .get(id)
                        .is_some_and(|metadata| metadata.abs_position.is_some())
            })
            .collect();
        path.reverse();
        path
    }

    pub(super) fn plan(
        &mut self,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        position: Option<PxPosition>,
        paths: impl Iterator<Item = NodeId>,
    ) -> PointerDispatchPlan {
        let hovered = self.hits(tree, metadatas, position);
        let mut selected = std::mem::take(&mut self.pending);
        selected.extend(std::mem::replace(&mut self.hovered, hovered.clone()));
        selected.extend(hovered);
        selected.extend(paths);
        let mut preorder: Vec<_> = selected
            .into_iter()
            .filter(|id| self.entries.contains_key(id))
            .collect();
        preorder.sort_unstable_by(|a, b| self.entries[a].order.cmp(&self.entries[b].order));
        PointerDispatchPlan {
            postorder: preorder.iter().rev().copied().collect(),
            nodes: preorder
                .iter()
                .map(|id| (*id, self.entries[id].plan.clone()))
                .collect(),
            preorder,
        }
    }
}
