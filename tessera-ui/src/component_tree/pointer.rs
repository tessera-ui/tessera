//! Pointer targeting and ordered input delivery.
//!
//! ## Usage
//!
//! Route mouse, touch, and hover interactions to placed controls.

use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{NodeId, PxPosition, PxSize, modifier::OrderedModifierAction, time::Instant};

use super::{
    ComponentNodeMetaDatas, ComponentNodeTree, NodeRole, PointerDispatchPlan, PointerNodePlan,
    bounds_hit_path, record_frame_stage, resolve_node_input_context_with_name,
};

const CELL_SIZE: f32 = 256.0;
const MAX_CELLS: usize = 256;

struct Entry {
    plan: Arc<PointerNodePlan>,
    order: Vec<usize>,
    coverage: Coverage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct CellRange {
    min: (i32, i32),
    max: (i32, i32),
}

impl CellRange {
    fn contains(self, (x, y): (i32, i32)) -> bool {
        x >= self.min.0 && x <= self.max.0 && y >= self.min.1 && y <= self.max.1
    }

    fn iter(self) -> impl Iterator<Item = (i32, i32)> {
        (self.min.0..=self.max.0).flat_map(move |x| (self.min.1..=self.max.1).map(move |y| (x, y)))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Coverage {
    Hidden,
    Grid(CellRange),
    Oversized,
}

impl Coverage {
    fn contains(self, cell: (i32, i32)) -> bool {
        matches!(self, Self::Grid(range) if range.contains(cell))
    }
}

#[derive(Default)]
pub(super) struct PointerIndex {
    eligible: HashSet<NodeId>,
    entries: HashMap<NodeId, Entry>,
    cells: HashMap<(i32, i32), HashSet<NodeId>>,
    oversized: HashSet<NodeId>,
    pending: HashSet<NodeId>,
    hovered: HashSet<NodeId>,
    changed: HashSet<NodeId>,
    removed: HashSet<NodeId>,
    hit_cache: RefCell<HashMap<(u32, u32), HashSet<NodeId>>>,
    pub(super) hit_candidates: Cell<u64>,
    grid_additions: usize,
    grid_removals: usize,
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
        self.grid_additions = 0;
        self.grid_removals = 0;
        let mut stage_timer =
            tracing::enabled!(target: "tessera_ui::frame_cost", tracing::Level::DEBUG)
                .then(Instant::now);
        let mut removed = removed.clone();
        removed.extend(std::mem::take(&mut self.removed));
        let structural = std::mem::take(&mut self.changed);
        let geometry_count = changed.len();
        // Retire removals before registering current structural nodes.
        // A tree reset can reuse identities present in both journals.
        for &id in &removed {
            self.eligible.remove(&id);
            self.remove(id);
            self.hovered.remove(&id);
            self.pending.remove(&id);
        }
        for &id in &structural {
            let eligible = tree
                .get(id)
                .filter(|node| !node.is_removed())
                .is_some_and(|node| {
                    let node = node.get();
                    node.role == NodeRole::Layout
                        && (!node.pointer_preview_handlers.is_empty()
                            || !node.pointer_handlers.is_empty()
                            || !node.pointer_final_handlers.is_empty()
                            || node.modifier.has_pointer_input())
                });
            if eligible {
                self.eligible.insert(id);
            } else {
                self.eligible.remove(&id);
            }
        }
        let mut updates = structural.clone();
        if self.eligible.len() < changed.len() {
            updates.extend(
                self.eligible
                    .iter()
                    .filter(|id| changed.contains(id))
                    .copied(),
            );
        } else {
            updates.extend(
                changed
                    .iter()
                    .filter(|id| self.eligible.contains(id))
                    .copied(),
            );
        }
        let changed = updates;
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
            if metadata.abs_position.is_none() {
                self.remove(id);
                continue;
            }
            let previous = self.entries.remove(&id);
            let previous_coverage = previous
                .as_ref()
                .map_or(Coverage::Hidden, |entry| entry.coverage);
            let was_exposed = previous_coverage != Coverage::Hidden;
            let retained = previous.filter(|_| !structural.contains(&id));
            let (mut retained_plan, retained_order) = match retained {
                Some(entry) => (Some(entry.plan), Some(entry.order)),
                None => (None, None),
            };
            let fn_name = retained_plan.as_mut().map_or_else(String::new, |plan| {
                std::mem::take(&mut Arc::make_mut(plan).context.fn_name)
            });
            let Some(context) = resolve_node_input_context_with_name(tree, metadatas, id, fn_name)
            else {
                self.update_coverage(id, previous_coverage, Coverage::Hidden);
                continue;
            };
            let plan = if let Some(mut plan) = retained_plan {
                Arc::make_mut(&mut plan).context = context;
                plan
            } else {
                Arc::new(PointerNodePlan {
                    context,
                    actions: node.modifier.ordered_actions(),
                })
            };
            let context = &plan.context;
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
            for (index, action) in plan.actions.iter().enumerate() {
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
            let Some(mut bounds) = bounds else {
                self.update_coverage(id, previous_coverage, Coverage::Hidden);
                continue;
            };
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
            let coverage = if !exposed {
                Coverage::Hidden
            } else if width.saturating_mul(height) > MAX_CELLS as i64 {
                Coverage::Oversized
            } else {
                Coverage::Grid(CellRange { min, max })
            };
            self.update_coverage(id, previous_coverage, coverage);
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
                    plan,
                    order: path,
                    coverage,
                },
            );
            if exposed || was_exposed || structural.contains(&id) {
                self.pending.insert(id);
            }
        }
        record_frame_stage(&mut stage_timer, "pointer_index");
        tracing::debug!(target: "tessera_ui::frame_cost", additions = self.grid_additions, removals = self.grid_removals, "pointer_grid_changes");
        changed.len()
    }

    fn remove(&mut self, id: NodeId) -> Option<Entry> {
        let entry = self.entries.remove(&id);
        if let Some(entry) = &entry {
            self.update_coverage(id, entry.coverage, Coverage::Hidden);
        }
        self.oversized.remove(&id);
        entry
    }

    fn update_coverage(&mut self, id: NodeId, previous: Coverage, next: Coverage) {
        if previous == next {
            return;
        }
        if let Coverage::Grid(range) = previous {
            for cell in range.iter().filter(|&cell| !next.contains(cell)) {
                if let Some(nodes) = self.cells.get_mut(&cell) {
                    if nodes.remove(&id) {
                        self.grid_removals += 1;
                    }
                    if nodes.is_empty() {
                        self.cells.remove(&cell);
                    }
                }
            }
        }
        if let Coverage::Grid(range) = next {
            for cell in range.iter().filter(|&cell| !previous.contains(cell)) {
                if self.cells.entry(cell).or_default().insert(id) {
                    self.grid_additions += 1;
                }
            }
        }
        if previous == Coverage::Oversized {
            self.oversized.remove(&id);
        }
        if next == Coverage::Oversized {
            self.oversized.insert(id);
        }
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ComputedData, Constraint, LayoutResult, Modifier, Px, PxPosition, State,
        layout::{MeasureScope, layout},
        modifier::{LayoutModifierNode, PlacementModifierNode, PointerInputModifierNode},
        remember,
        runtime::TesseraRuntime,
        tessera,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    thread_local! {
        static POSITION: Cell<Option<State<PxPosition>>> = const { Cell::new(None) };
        static SIZE: Cell<Option<State<ComputedData>>> = const { Cell::new(None) };
        static PLACED: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static REBUILD: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static OBSERVED_HANDLER: Cell<bool> = const { Cell::new(false) };
    }

    #[derive(Clone, PartialEq)]
    struct Canvas(State<bool>);

    impl LayoutModifierNode for Canvas {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let child = scope.children()[0].measure(scope, &Constraint::NONE)?;
            let mut result = LayoutResult::new(ComputedData {
                width: Px(8000),
                height: Px(8000),
            });
            if self.0.get() {
                result.place_child(child, PxPosition::ZERO);
            }
            Ok(result)
        }
    }

    #[derive(Clone, PartialEq)]
    struct ControlSize(State<ComputedData>);

    impl LayoutModifierNode for ControlSize {
        fn measure(
            &self,
            _: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            Ok(LayoutResult::new(self.0.get()))
        }
    }

    #[derive(PartialEq)]
    struct Position(State<PxPosition>);

    impl PlacementModifierNode for Position {
        fn transform_position(&self, position: PxPosition) -> PxPosition {
            position + self.0.get()
        }
    }

    struct Input(bool);

    impl PointerInputModifierNode for Input {
        fn on_pointer_input(&self, input: crate::PointerInput<'_>) {
            if input.pass == crate::PointerEventPass::Main {
                OBSERVED_HANDLER.set(self.0);
            }
        }
    }

    #[tessera(crate)]
    fn moving_control() {
        let position = remember(|| PxPosition::new(Px(10), Px(10)));
        let size = remember(|| ComputedData {
            width: Px(20),
            height: Px(20),
        });
        let placed = remember(|| true);
        let rebuild = remember(|| false);
        POSITION.set(Some(position));
        SIZE.set(Some(size));
        PLACED.set(Some(placed));
        REBUILD.set(Some(rebuild));
        let generation = rebuild.get();
        layout().policy(Canvas(placed)).child(move || {
            layout().policy(ControlSize(size)).modifier(
                Modifier::new()
                    .push_placement(Position(position))
                    .push_pointer_input(Input(generation)),
            );
        });
    }

    #[test]
    fn pointer_grid_reuses_overlapping_cells_and_updates_hits_across_visibility_and_size_changes() {
        let mut session =
            start_layout_test_session(layout_test(moving_control).viewport_px(8000, 8000));
        pump_layout_test_session(&mut session, 0);
        let target = TesseraRuntime::with(|runtime| {
            let index = &runtime.component_tree.pointer_index;
            assert_eq!(index.eligible.len(), 1);
            *index.eligible.iter().next().unwrap()
        });
        let check = |additions, removals, hit: PxPosition, expected| {
            TesseraRuntime::with(|runtime| {
                let tree = &runtime.component_tree;
                let index = &tree.pointer_index;
                assert_eq!(
                    (index.grid_additions, index.grid_removals),
                    (additions, removals)
                );
                assert_eq!(
                    index
                        .sample_targets(tree.tree(), tree.metadatas(), Some(hit))
                        .contains(&target),
                    expected
                );
            });
        };
        check(1, 0, PxPosition::new(Px(15), Px(15)), true);
        let position = POSITION.get().unwrap();
        position.set(PxPosition::new(Px(40), Px(10)));
        pump_layout_test_session(&mut session, 16_666_667);
        check(0, 0, PxPosition::new(Px(15), Px(15)), false);
        check(0, 0, PxPosition::new(Px(45), Px(15)), true);
        position.set(PxPosition::new(Px(250), Px(10)));
        pump_layout_test_session(&mut session, 33_333_334);
        check(1, 0, PxPosition::new(Px(265), Px(15)), true);
        position.set(PxPosition::new(Px(270), Px(10)));
        pump_layout_test_session(&mut session, 50_000_001);
        check(0, 1, PxPosition::new(Px(265), Px(15)), false);
        check(0, 1, PxPosition::new(Px(275), Px(15)), true);

        let placed = PLACED.get().unwrap();
        placed.set(false);
        pump_layout_test_session(&mut session, 66_666_668);
        check(0, 1, PxPosition::new(Px(275), Px(15)), false);
        placed.set(true);
        pump_layout_test_session(&mut session, 83_333_335);
        check(1, 0, PxPosition::new(Px(275), Px(15)), true);

        let size = SIZE.get().unwrap();
        size.set(ComputedData {
            width: Px(5000),
            height: Px(5000),
        });
        pump_layout_test_session(&mut session, 100_000_002);
        check(0, 1, PxPosition::new(Px(500), Px(500)), true);
        size.set(ComputedData {
            width: Px(20),
            height: Px(20),
        });
        pump_layout_test_session(&mut session, 116_666_669);
        check(1, 0, PxPosition::new(Px(500), Px(500)), false);
        check(1, 0, PxPosition::new(Px(275), Px(15)), true);

        TesseraRuntime::with_mut(|runtime| {
            let tree = &mut runtime.component_tree;
            let canvas = super::super::geometry_parent(tree.tree(), target).unwrap();
            tree.metadatas.entry_or_default(canvas).clips_children = true;
            tree.metadatas.mark_geometry_dirty(canvas);
        });
        position.set(PxPosition::new(Px(8100), Px(10)));
        pump_layout_test_session(&mut session, 133_333_336);
        check(0, 1, PxPosition::new(Px(8105), Px(15)), false);
        TesseraRuntime::with(|runtime| {
            assert!(
                runtime
                    .component_tree
                    .pointer_index
                    .entries
                    .contains_key(&target)
            );
        });
        position.set(PxPosition::new(Px(270), Px(10)));
        pump_layout_test_session(&mut session, 150_000_003);
        check(1, 0, PxPosition::new(Px(275), Px(15)), true);
        pump_layout_test_session(&mut session, 166_666_670);
        check(0, 0, PxPosition::new(Px(275), Px(15)), true);

        OBSERVED_HANDLER.set(false);
        REBUILD.get().unwrap().set(true);
        pump_layout_test_session(&mut session, 183_333_337);
        assert!(
            OBSERVED_HANDLER.get(),
            "replay must replace the cached input handler"
        );
        TesseraRuntime::with(|runtime| {
            let tree = &runtime.component_tree;
            let current = *tree.pointer_index.eligible.iter().next().unwrap();
            assert!(
                tree.pointer_index
                    .sample_targets(
                        tree.tree(),
                        tree.metadatas(),
                        Some(PxPosition::new(Px(275), Px(15)))
                    )
                    .contains(&current)
            );
        });
    }
}
