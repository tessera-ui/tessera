//! Retained drawing for interactive scenes.
//!
//! ## Usage
//!
//! Render retained content during animation and interaction.

use std::sync::Arc;

use im::OrdMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    ClipRegion, NodeId, PxPosition, PxRect, PxSize,
    render_graph::{DrawGeometry, RenderFragment, RenderGraph, RenderGraphBuilder},
    time::Instant,
};

use super::{ComponentNodeMetaDatas, ComponentNodeTree, NodeRole, record_frame_stage};

#[derive(Clone, Copy)]
enum SceneClip {
    Shape(ClipRegion),
    Rect(PxRect),
}

struct SceneDraw {
    fragment: RenderFragment,
    geometry: DrawGeometry,
    opacity: f32,
}

struct SceneRope {
    draw: Option<Arc<SceneDraw>>,
    clip: Option<SceneClip>,
    children: OrdMap<usize, Arc<SceneRope>>,
}

#[derive(Default)]
struct SceneBranch {
    parent: Option<NodeId>,
    depth: usize,
    child_indices: HashMap<NodeId, usize>,
    children: OrdMap<usize, Arc<SceneRope>>,
    draw: Option<Arc<SceneDraw>>,
    clip: Option<SceneClip>,
    output: Option<Arc<SceneRope>>,
    culled: bool,
}

#[derive(Default)]
pub(super) struct FrameScene {
    branches: HashMap<NodeId, SceneBranch>,
    root: Option<Arc<SceneRope>>,
    graph: Option<RenderGraph>,
    viewport: Option<PxSize>,
}

impl FrameScene {
    fn output(&self, mut node_id: NodeId) -> Option<Arc<SceneRope>> {
        loop {
            let branch = self.branches.get(&node_id)?;
            // Transparent composition wrappers have no geometry journal of
            // their own.
            if branch.draw.is_none() && branch.clip.is_none() && branch.child_indices.len() == 1 {
                node_id = *branch.child_indices.keys().next()?;
            } else if branch.culled {
                return None;
            } else {
                return branch.output.clone();
            }
        }
    }

    pub(super) fn update(
        &mut self,
        root: NodeId,
        tree: &ComponentNodeTree,
        metadata: &ComponentNodeMetaDatas,
        changed: &HashSet<NodeId>,
        removed: &HashSet<NodeId>,
        viewport: PxSize,
    ) -> (RenderGraph, u64) {
        let mut stage_timer =
            tracing::enabled!(target: "tessera_ui::frame_cost", tracing::Level::DEBUG)
                .then(Instant::now);
        let mut dirty = HashSet::default();
        let mut structural = changed.clone();
        for node_id in removed {
            let mut parent = self.branches.get(node_id).and_then(|branch| branch.parent);
            if let Some(parent) = parent {
                structural.insert(parent);
            }
            while let Some(id) = parent {
                dirty.insert(id);
                parent = self.branches.get(&id).and_then(|branch| branch.parent);
            }
        }
        for node_id in removed {
            self.branches.remove(node_id);
        }
        record_frame_stage(&mut stage_timer, "scene_removals");
        let mut own_changes = changed.clone();
        own_changes.extend(metadata.paint_changed.iter().copied());
        let viewport_rect = PxRect::enclosing(PxPosition::ZERO, viewport);
        for &node_id in &metadata.geometry_changed {
            let Some(branch) = self.branches.get(&node_id) else {
                own_changes.insert(node_id);
                continue;
            };
            if branch.culled && fully_clipped(tree, metadata, node_id, viewport_rect) {
                continue;
            }
            let prepared = metadata.get(&node_id).and_then(|node| node.prepared_layout);
            let has_draw = metadata
                .get(&node_id)
                .is_some_and(|node| !node.fragment.ops().is_empty());
            if !branch.culled
                && branch.draw.is_none()
                && branch.clip.is_none()
                && !has_draw
                && prepared.is_none_or(|prepared| !prepared.clips_children)
            {
                continue;
            }
            if branch.clip.is_none()
                && prepared.is_some_and(|prepared| {
                    !prepared.clips_children
                        && DrawGeometry::new(prepared.size, prepared.local_to_window)
                            .window_bounds()
                            .outward()
                            .is_orthogonal(&viewport_rect)
                })
                && branch.draw.as_ref().is_some_and(|draw| {
                    draw.geometry
                        .window_bounds()
                        .outward()
                        .is_orthogonal(&viewport_rect)
                })
            {
                continue;
            }
            own_changes.insert(node_id);
        }
        if self.viewport != Some(viewport) {
            own_changes.extend(self.branches.iter().filter_map(|(&id, branch)| {
                (branch.culled || branch.draw.is_some()).then_some(id)
            }));
        }
        for &node_id in changed {
            if let Some(parent) = tree.get(node_id).and_then(|node| node.parent()) {
                structural.insert(parent);
            }
        }
        record_frame_stage(&mut stage_timer, "scene_candidates");
        let mut visited = HashSet::default();
        for &node_id in own_changes.iter().chain(structural.iter()) {
            if tree.get(node_id).is_none_or(|node| node.is_removed()) {
                continue;
            }
            for ancestor in node_id.ancestors(tree) {
                if !visited.insert(ancestor) {
                    break;
                }
                let transparent = self.branches.get(&ancestor).is_some_and(|branch| {
                    branch.draw.is_none()
                        && branch.clip.is_none()
                        && branch.child_indices.len() == 1
                });
                if !transparent || own_changes.contains(&ancestor) || structural.contains(&ancestor)
                {
                    dirty.insert(ancestor);
                }
            }
        }
        record_frame_stage(&mut stage_timer, "scene_ancestry");
        let mut ordered = dirty
            .iter()
            .copied()
            .filter(|id| tree.get(*id).is_some_and(|node| !node.is_removed()))
            .map(|id| {
                let depth = self
                    .branches
                    .get(&id)
                    .filter(|_| !changed.contains(&id))
                    .map_or_else(|| id.ancestors(tree).count(), |branch| branch.depth);
                (id, depth)
            })
            .collect::<Vec<_>>();
        ordered.sort_unstable_by_key(|&(_, depth)| std::cmp::Reverse(depth));
        record_frame_stage(&mut stage_timer, "scene_order");
        let updated = ordered.len() as u64;
        for (node_id, depth) in ordered {
            let node = &tree[node_id];
            let mut branch = self.branches.remove(&node_id).unwrap_or_default();
            branch.parent = node.parent();
            branch.depth = depth;
            if structural.contains(&node_id) {
                branch.child_indices.clear();
                branch.children.clear();
                for (index, child) in node_id.children(tree).enumerate() {
                    branch.child_indices.insert(child, index);
                    if let Some(output) = self.output(child) {
                        branch.children.insert(index, output);
                    }
                }
            }
            if own_changes.contains(&node_id) {
                let prepared = metadata
                    .get(&node_id)
                    .and_then(|metadata| metadata.prepared_layout);
                branch.draw = prepared.and_then(|prepared| {
                    let metadata = metadata.get(&node_id)?;
                    (!metadata.fragment.ops().is_empty()).then(|| {
                        Arc::new(SceneDraw {
                            fragment: metadata.fragment.clone(),
                            geometry: DrawGeometry::new(prepared.size, prepared.local_to_window),
                            opacity: prepared.cumulative_opacity,
                        })
                    })
                });
                branch.clip = prepared
                    .filter(|prepared| prepared.clips_children)
                    .map(|prepared| {
                        prepared.clip_region.map_or_else(
                            || SceneClip::Rect(prepared.child_clip_rect.unwrap_or(PxRect::ZERO)),
                            SceneClip::Shape,
                        )
                    });
            }
            let hidden = node.get().role == NodeRole::Layout
                && metadata
                    .get(&node_id)
                    .is_none_or(|metadata| metadata.abs_position.is_none());
            branch.culled = hidden || fully_clipped(tree, metadata, node_id, viewport_rect);
            branch.output = if branch.culled
                || (branch.draw.is_none() && branch.children.is_empty())
            {
                None
            } else if branch.draw.is_none() && branch.clip.is_none() && branch.children.len() == 1 {
                branch.children.values().next().cloned()
            } else {
                Some(Arc::new(SceneRope {
                    draw: branch.draw.clone(),
                    clip: branch.clip,
                    children: branch.children.clone(),
                }))
            };
            let mut child = node_id;
            let mut parent = branch.parent;
            while let Some(id) = parent {
                if !dirty.contains(&id) {
                    child = id;
                    parent = tree.get(id).and_then(|node| node.parent());
                    continue;
                }
                if let Some(parent) = self.branches.get_mut(&id)
                    && let Some(&index) = parent.child_indices.get(&child)
                {
                    if let Some(output) = &branch.output {
                        parent.children.insert(index, Arc::clone(output));
                    } else {
                        parent.children.remove(&index);
                    }
                }
                break;
            }
            self.branches.insert(node_id, branch);
        }
        record_frame_stage(&mut stage_timer, "scene_branches");
        let next_root = self.output(root);
        let same = match (&self.root, &next_root) {
            (Some(previous), Some(next)) => Arc::ptr_eq(previous, next),
            (None, None) => true,
            _ => false,
        };
        if !same || self.viewport != Some(viewport) || self.graph.is_none() {
            let mut builder = RenderGraphBuilder::new();
            if let Some(root) = &next_root {
                root.emit(&mut builder, PxRect::enclosing(PxPosition::ZERO, viewport));
            }
            self.graph = Some(builder.finish());
        }
        record_frame_stage(&mut stage_timer, "scene_emit");
        self.root = next_root;
        self.viewport = Some(viewport);
        (
            self.graph
                .as_ref()
                .expect("frame scene is initialized")
                .clone(),
            updated,
        )
    }
}

fn fully_clipped(
    tree: &ComponentNodeTree,
    metadata: &ComponentNodeMetaDatas,
    node_id: NodeId,
    viewport: PxRect,
) -> bool {
    for id in node_id.ancestors(tree) {
        if let Some(prepared) = metadata.get(&id).and_then(|node| node.prepared_layout) {
            return prepared.child_clip_bounds.is_some_and(|bounds| {
                bounds.width <= 0.0
                    || bounds.height <= 0.0
                    || bounds.outward().is_orthogonal(&viewport)
            });
        }
        if tree[id].get().role == NodeRole::Layout {
            return true;
        }
    }
    false
}

impl SceneRope {
    fn emit(&self, builder: &mut RenderGraphBuilder, viewport: PxRect) {
        match self.clip {
            Some(SceneClip::Shape(region)) => builder.push_shape_clip_push(region),
            Some(SceneClip::Rect(rect)) => builder.push_clip_push(rect),
            None => {}
        }
        if let Some(draw) = &self.draw {
            if draw.geometry.local_size.width.0 > 0
                && draw.geometry.local_size.height.0 > 0
                && !draw
                    .geometry
                    .window_bounds()
                    .outward()
                    .is_orthogonal(&viewport)
            {
                builder.append_fragment(draw.fragment.clone(), draw.geometry, draw.opacity);
            }
        }
        for child in self.children.values() {
            child.emit(builder, viewport);
        }
        if self.clip.is_some() {
            builder.push_clip_pop();
        }
    }
}
