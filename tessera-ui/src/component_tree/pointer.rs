//! Pointer targeting and ordered input delivery.
//!
//! ## Usage
//!
//! Route mouse, touch, and hover interactions to placed controls.

use std::cell::{Cell, RefCell};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{NodeId, PxPosition, PxSize, modifier::OrderedModifierAction, time::Instant};

use super::{
    ComponentNodeMetaDatas, ComponentNodeTree, NodeRole, PointerDispatchPlan, PointerNodePlan,
    bounds_hit_path, record_frame_stage, resolve_node_input_context,
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
        let mut stage_timer =
            tracing::enabled!(target: "tessera_ui::frame_cost", tracing::Level::DEBUG)
                .then(Instant::now);
        let mut removed = removed.clone();
        removed.extend(std::mem::take(&mut self.removed));
        let structural = std::mem::take(&mut self.changed);
        let geometry_count = changed.len();
        let mut changed = changed.clone();
        changed.extend(structural.iter().copied());
        if tracing::enabled!(target: "tessera_ui::frame_cost", tracing::Level::TRACE) {
            let mut structural_nodes = HashMap::default();
            let mut stale = 0_usize;
            for &id in &structural {
                if let Some(node) = tree.get(id).filter(|node| !node.is_removed()) {
                    *structural_nodes
                        .entry(node.get().fn_name.as_str())
                        .or_insert(0_usize) += 1;
                } else {
                    stale += 1;
                }
            }
            tracing::trace!(
                target: "tessera_ui::frame_cost",
                geometry = geometry_count,
                structural = structural.len(),
                removed = removed.len(),
                stale,
                ?structural_nodes,
                "pointer_index_invalidations"
            );
        }
        for &id in &removed {
            self.remove(id);
            self.hovered.remove(&id);
            self.pending.remove(&id);
        }
        record_frame_stage(&mut stage_timer, "pointer_invalidations");
        for &id in &changed {
            let Some(node) = tree.get(id).filter(|node| !node.is_removed()) else {
                self.remove(id);
                continue;
            };
            let node = node.get();
            if node.role != NodeRole::Layout {
                if self.entries.contains_key(&id) {
                    self.remove(id);
                }
                continue;
            }
            let raw_input = !node.pointer_preview_handlers.is_empty()
                || !node.pointer_handlers.is_empty()
                || !node.pointer_final_handlers.is_empty();
            let Some(metadata) = metadatas.get(&id) else {
                self.remove(id);
                continue;
            };
            if !raw_input && metadata.input_local_positions.is_empty() {
                if self.entries.contains_key(&id) {
                    self.remove(id);
                }
                continue;
            }
            let was_oversized = self.oversized.contains(&id);
            let previous = self.remove(id);
            let was_exposed = was_oversized
                || previous
                    .as_ref()
                    .is_some_and(|entry| !entry.cells.is_empty());
            let retained = previous.filter(|_| !structural.contains(&id));
            let (actions, retained_order) = match retained {
                Some(entry) => (entry.plan.actions, Some(entry.order)),
                None => (node.modifier.ordered_actions(), None),
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
            let mut bounds = raw_input.then(|| {
                context
                    .local_to_window
                    .map_rect(crate::FloatRect::from_position_size(PxPosition::ZERO, size))
                    .outward()
            });
            let mut position = context.base_abs_pos;
            for (index, action) in actions.iter().enumerate() {
                if let Some(transform) = context.input_transforms.get(index) {
                    position = transform.translation();
                }
                match action {
                    OrderedModifierAction::Placement(placement) => {
                        if context.input_transforms.is_empty() {
                            position = placement.node().transform_position(position);
                        }
                    }
                    OrderedModifierAction::Cursor(_)
                    | OrderedModifierAction::PointerPreviewInput(_)
                    | OrderedModifierAction::PointerInput(_)
                    | OrderedModifierAction::PointerFinalInput(_) => {
                        let rect =
                            crate::LocalToWindow::new(context.local_to_window.scale(), position)
                                .unwrap()
                                .map_rect(crate::FloatRect::from_position_size(
                                    PxPosition::ZERO,
                                    size,
                                ))
                                .outward();
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
            let path = retained_order.unwrap_or_else(|| {
                let mut path: Vec<_> = id
                    .ancestors(tree)
                    .map(|id| order.get(&id).copied().unwrap_or_default())
                    .collect();
                path.reverse();
                path
            });
            self.entries.insert(
                id,
                Entry {
                    plan: PointerNodePlan { context, actions },
                    order: path,
                    cells,
                },
            );
            if exposed || was_exposed || structural.contains(&id) {
                self.pending.insert(id);
            }
        }
        record_frame_stage(&mut stage_timer, "pointer_index");
        changed.len()
    }

    fn remove(&mut self, id: NodeId) -> Option<Entry> {
        let entry = self.entries.remove(&id);
        if let Some(entry) = &entry {
            for cell in &entry.cells {
                if let Some(nodes) = self.cells.get_mut(cell) {
                    nodes.remove(&id);
                    if nodes.is_empty() {
                        self.cells.remove(cell);
                    }
                }
            }
        }
        self.oversized.remove(&id);
        entry
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
