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
    render_graph::{RenderFragment, RenderGraph, RenderGraphBuilder},
};

use super::{ComponentNodeMetaDatas, ComponentNodeTree, NodeRole};

#[derive(Clone, Copy)]
enum SceneClip {
    Shape(ClipRegion),
    Rect(PxRect),
}

struct SceneDraw {
    fragment: RenderFragment,
    size: PxSize,
    position: PxPosition,
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
    child_indices: HashMap<NodeId, usize>,
    children: OrdMap<usize, Arc<SceneRope>>,
    draw: Option<Arc<SceneDraw>>,
    clip: Option<SceneClip>,
    output: Option<Arc<SceneRope>>,
}

#[derive(Default)]
pub(super) struct FrameScene {
    branches: HashMap<NodeId, SceneBranch>,
    root: Option<Arc<SceneRope>>,
    graph: Option<RenderGraph>,
    viewport: Option<PxSize>,
}

impl FrameScene {
    pub(super) fn update(
        &mut self,
        root: NodeId,
        tree: &ComponentNodeTree,
        metadata: &ComponentNodeMetaDatas,
        changed: &HashSet<NodeId>,
        removed: &HashSet<NodeId>,
        viewport: PxSize,
    ) -> (RenderGraph, u64) {
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
        let mut own_changes = changed.clone();
        own_changes.extend(metadata.geometry_changed.iter().copied());
        own_changes.extend(metadata.paint_changed.iter().copied());
        for &node_id in changed {
            if let Some(parent) = tree.get(node_id).and_then(|node| node.parent()) {
                structural.insert(parent);
            }
        }
        for &node_id in own_changes.iter().chain(structural.iter()) {
            if tree.get(node_id).is_none_or(|node| node.is_removed()) {
                continue;
            }
            for ancestor in node_id.ancestors(tree) {
                if !dirty.insert(ancestor) {
                    break;
                }
            }
        }
        let mut ordered = dirty
            .into_iter()
            .filter(|id| tree.get(*id).is_some_and(|node| !node.is_removed()))
            .collect::<Vec<_>>();
        ordered.sort_by_cached_key(|id| std::cmp::Reverse(id.ancestors(tree).count()));
        let updated = ordered.len() as u64;
        for node_id in ordered {
            let node = &tree[node_id];
            let mut branch = self.branches.remove(&node_id).unwrap_or_default();
            branch.parent = node.parent();
            if structural.contains(&node_id) {
                branch.child_indices.clear();
                branch.children.clear();
                for (index, child) in node_id.children(tree).enumerate() {
                    branch.child_indices.insert(child, index);
                    if let Some(output) = self
                        .branches
                        .get(&child)
                        .and_then(|child| child.output.clone())
                    {
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
                            size: prepared.size,
                            position: prepared.self_position,
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
            branch.output = if hidden || (branch.draw.is_none() && branch.children.is_empty()) {
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
            if let Some(parent) = branch
                .parent
                .and_then(|parent| self.branches.get_mut(&parent))
            {
                if let Some(&index) = parent.child_indices.get(&node_id) {
                    if let Some(output) = &branch.output {
                        parent.children.insert(index, Arc::clone(output));
                    } else {
                        parent.children.remove(&index);
                    }
                }
            }
            self.branches.insert(node_id, branch);
        }
        let next_root = self
            .branches
            .get(&root)
            .and_then(|branch| branch.output.clone());
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

impl SceneRope {
    fn emit(&self, builder: &mut RenderGraphBuilder, viewport: PxRect) {
        match self.clip {
            Some(SceneClip::Shape(region)) => builder.push_shape_clip_push(region),
            Some(SceneClip::Rect(rect)) => builder.push_clip_push(rect),
            None => {}
        }
        if let Some(draw) = &self.draw {
            if draw.size.width.0 > 0
                && draw.size.height.0 > 0
                && !PxRect::enclosing(draw.position, draw.size).is_orthogonal(&viewport)
            {
                builder.append_fragment(
                    draw.fragment.clone(),
                    draw.size,
                    draw.position,
                    draw.opacity,
                );
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
