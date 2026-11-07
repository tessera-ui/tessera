//! Retained drawing for interactive scenes.
//!
//! ## Usage
//!
//! Render retained content during animation and interaction.

use std::sync::Arc;

use im::OrdMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    ClipRegion, Command, DrawRegion, NodeId, PxPosition, PxRect, PxSize,
    render_graph::{DrawGeometry, RenderFragment, RenderGraph, RenderGraphBuilder},
    time::Instant,
};

use super::{ComponentNodeMetaDatas, ComponentNodeTree, NodeRole, record_frame_stage};

#[derive(Clone, Copy, PartialEq)]
enum SceneClip {
    Shape(ClipRegion),
    Rect(PxRect),
}

struct SceneDraw {
    fragment: Arc<RenderFragment>,
    geometry: DrawGeometry,
    opacity: f32,
    visible: bool,
    bounds: Option<PxRect>,
}

struct SceneRope {
    draw: Option<Arc<SceneDraw>>,
    clip: Option<SceneClip>,
    children: SceneChildren,
    bounds: Option<PxRect>,
}

enum SceneChildren {
    Empty,
    One(Arc<SceneRope>),
    Many(OrdMap<usize, Arc<SceneRope>>),
}

impl SceneChildren {
    fn values(&self) -> impl Iterator<Item = &Arc<SceneRope>> {
        let (one, many) = match self {
            Self::Empty => (None, None),
            Self::One(child) => (Some(child), None),
            Self::Many(children) => (None, Some(children)),
        };
        one.into_iter()
            .chain(many.into_iter().flat_map(OrdMap::values))
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::One(a), Self::One(b)) => Arc::ptr_eq(a, b),
            (Self::Many(a), Self::Many(b)) => a.ptr_eq(b),
            _ => false,
        }
    }
}

#[derive(Default)]
enum BranchChildren {
    #[default]
    Empty,
    One {
        id: NodeId,
        output: Option<Arc<SceneRope>>,
    },
    Many(Box<WideBranchChildren>),
}

#[derive(Default)]
struct WideBranchChildren {
    indices: HashMap<NodeId, usize>,
    outputs: OrdMap<usize, Arc<SceneRope>>,
}

impl BranchChildren {
    fn rebuild(
        &mut self,
        mut ids: impl Iterator<Item = NodeId>,
        output: impl Fn(NodeId) -> Option<Arc<SceneRope>>,
    ) {
        let Some(first) = ids.next() else {
            *self = Self::Empty;
            return;
        };
        let Some(second) = ids.next() else {
            *self = Self::One {
                id: first,
                output: output(first),
            };
            return;
        };
        let mut many = match std::mem::take(self) {
            Self::Many(mut many) => {
                many.indices.clear();
                many.outputs.clear();
                many
            }
            _ => Box::<WideBranchChildren>::default(),
        };
        for (index, id) in [first, second].into_iter().chain(ids).enumerate() {
            many.indices.insert(id, index);
            if let Some(rope) = output(id) {
                many.outputs.insert(index, rope);
            }
        }
        *self = Self::Many(many);
    }

    fn single(&self) -> Option<NodeId> {
        match self {
            Self::One { id, .. } => Some(*id),
            _ => None,
        }
    }

    fn update_output(&mut self, id: NodeId, output: Option<&Arc<SceneRope>>) {
        match self {
            Self::One {
                id: child,
                output: previous,
            } if *child == id => {
                *previous = output.cloned();
            }
            Self::Many(many) => {
                let Some(&index) = many.indices.get(&id) else {
                    return;
                };
                match output {
                    Some(output)
                        if many
                            .outputs
                            .get(&index)
                            .is_none_or(|previous| !Arc::ptr_eq(previous, output)) =>
                    {
                        many.outputs.insert(index, Arc::clone(output));
                    }
                    None if many.outputs.contains_key(&index) => {
                        many.outputs.remove(&index);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn snapshot(&self) -> SceneChildren {
        match self {
            Self::Empty | Self::One { output: None, .. } => SceneChildren::Empty,
            Self::One {
                output: Some(rope), ..
            } => SceneChildren::One(Arc::clone(rope)),
            Self::Many(many) => match many.outputs.len() {
                0 => SceneChildren::Empty,
                1 => SceneChildren::One(Arc::clone(many.outputs.values().next().unwrap())),
                _ => SceneChildren::Many(many.outputs.clone()),
            },
        }
    }
}

#[derive(Default)]
struct SceneBranch {
    parent: Option<NodeId>,
    depth: usize,
    children: BranchChildren,
    draw: Option<Arc<SceneDraw>>,
    clip: Option<SceneClip>,
    output: Option<Arc<SceneRope>>,
    culled: bool,
}

impl SceneBranch {
    fn transparent_child(&self) -> Option<NodeId> {
        if self.draw.is_none() && self.clip.is_none() {
            self.children.single()
        } else {
            None
        }
    }
}

#[derive(Default)]
pub(super) struct FrameScene {
    branches: HashMap<NodeId, SceneBranch>,
    geometry_consumers: HashSet<NodeId>,
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
            if let Some(child) = branch.transparent_child() {
                node_id = child;
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
        let mut damage = None;
        for node_id in removed {
            if let Some(bounds) = self
                .branches
                .get(node_id)
                .and_then(|branch| branch.output.as_ref())
                .and_then(|rope| rope.bounds)
            {
                damage = Some(damage.map_or(bounds, |rect: PxRect| rect.union(&bounds)));
            }
        }
        for node_id in removed {
            let mut parent = self.branches.get(node_id).and_then(|branch| branch.parent);
            if let Some(parent) = parent {
                structural.insert(parent);
            }
            while let Some(id) = parent {
                if !dirty.insert(id) {
                    break;
                }
                parent = self.branches.get(&id).and_then(|branch| branch.parent);
            }
        }
        for node_id in removed {
            self.branches.remove(node_id);
            self.geometry_consumers.remove(node_id);
        }
        record_frame_stage(&mut stage_timer, "scene_removals");
        let mut own_changes = changed.clone();
        own_changes.extend(metadata.paint_changed.iter().copied());
        let viewport_rect = PxRect::enclosing(PxPosition::ZERO, viewport);
        for &node_id in self
            .geometry_consumers
            .intersection(&metadata.geometry_changed)
        {
            let Some(branch) = self.branches.get(&node_id) else {
                own_changes.insert(node_id);
                continue;
            };
            if branch.culled && fully_clipped(tree, metadata, node_id, viewport_rect) {
                continue;
            }
            let node_metadata = metadata.get(&node_id);
            let prepared = node_metadata.and_then(|node| node.prepared_layout);
            let has_draw = node_metadata.is_some_and(|node| !node.fragment.ops().is_empty());
            if !branch.culled
                && branch.draw.is_none()
                && branch.clip.is_none()
                && !has_draw
                && prepared.is_none_or(|prepared| !prepared.clips_children)
            {
                continue;
            }
            if branch.clip.is_none()
                && branch.draw.as_ref().is_some_and(|draw| !draw.visible)
                && prepared.is_some_and(|prepared| {
                    !prepared.clips_children
                        && DrawGeometry::new(prepared.size, prepared.local_to_window)
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
                let transparent = self
                    .branches
                    .get(&ancestor)
                    .and_then(SceneBranch::transparent_child)
                    .is_some();
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
        tracing::debug!(
            target: "tessera_ui::frame_cost",
            structural_nodes = structural.len(),
            geometry_changes = metadata.geometry_changed.len(),
            own_changes = own_changes.len(),
            updated_branches = ordered.len(),
            "scene_update_work"
        );
        let updated = ordered.len() as u64;
        for (node_id, depth) in ordered {
            let node = &tree[node_id];
            let mut branch = self.branches.remove(&node_id).unwrap_or_default();
            let affects_pixels = own_changes.contains(&node_id) || structural.contains(&node_id);
            if affects_pixels
                && let Some(bounds) = branch.output.as_ref().and_then(|rope| rope.bounds)
            {
                damage = Some(damage.map_or(bounds, |rect: PxRect| rect.union(&bounds)));
            }
            branch.parent = node.parent();
            branch.depth = depth;
            if structural.contains(&node_id) {
                branch
                    .children
                    .rebuild(node_id.children(tree), |child| self.output(child));
            }
            let was_transparent = branch.transparent_child().is_some();
            if own_changes.contains(&node_id) {
                let fragment_changed =
                    changed.contains(&node_id) || metadata.paint_changed.contains(&node_id);
                let node_metadata = metadata.get(&node_id);
                let prepared = node_metadata.and_then(|metadata| metadata.prepared_layout);
                branch.draw = prepared.and_then(|prepared| {
                    let metadata = node_metadata?;
                    (!metadata.fragment.ops().is_empty()).then(|| {
                        let fragment = branch
                            .draw
                            .as_ref()
                            .filter(|_| !fragment_changed)
                            .map(|draw| Arc::clone(&draw.fragment))
                            .unwrap_or_else(|| Arc::new(metadata.fragment.clone()));
                        let geometry = DrawGeometry::new(prepared.size, prepared.local_to_window);
                        let visible = geometry.local_size.width.0 > 0
                            && geometry.local_size.height.0 > 0
                            && !geometry
                                .window_bounds()
                                .outward()
                                .is_orthogonal(&viewport_rect);
                        let bounds = fragment_bounds(&fragment, geometry, viewport_rect);
                        if let Some(draw) = &branch.draw
                            && Arc::ptr_eq(&draw.fragment, &fragment)
                            && draw.geometry == geometry
                            && draw.opacity == prepared.cumulative_opacity
                            && draw.visible == visible
                            && draw.bounds == bounds
                        {
                            return Arc::clone(draw);
                        }
                        Arc::new(SceneDraw {
                            bounds,
                            fragment,
                            geometry,
                            opacity: prepared.cumulative_opacity,
                            visible,
                        })
                    })
                });
                branch.clip = prepared
                    .filter(|prepared| prepared.clips_children)
                    .map(|prepared| {
                        node_metadata
                            .and_then(|metadata| metadata.prepared_clip_region)
                            .map_or_else(
                                || {
                                    SceneClip::Rect(
                                        prepared.child_clip_rect.unwrap_or(PxRect::ZERO),
                                    )
                                },
                                SceneClip::Shape,
                            )
                    });
            }
            let is_transparent = branch.transparent_child().is_some();
            if was_transparent != is_transparent
                && let Some(child) = branch.children.single()
            {
                // Flattened wrappers can bypass child-output maintenance while
                // descendants change visibility. Materializing the wrapper must
                // use the current child output rather than that stale cache.
                branch
                    .children
                    .update_output(child, self.output(child).as_ref());
            }
            let hidden = node.get().role == NodeRole::Layout
                && metadata
                    .get(&node_id)
                    .is_none_or(|metadata| metadata.abs_position.is_none());
            branch.culled = hidden || fully_clipped(tree, metadata, node_id, viewport_rect);
            if branch.draw.is_some() || branch.clip.is_some() || branch.culled {
                self.geometry_consumers.insert(node_id);
            } else {
                self.geometry_consumers.remove(&node_id);
            }
            let children = branch.children.snapshot();
            branch.output = if branch.culled
                || (branch.draw.as_ref().is_none_or(|draw| !draw.visible)
                    && matches!(children, SceneChildren::Empty))
            {
                None
            } else if branch.draw.is_none()
                && branch.clip.is_none()
                && let SceneChildren::One(rope) = children
            {
                Some(rope)
            } else if let Some(previous) = &branch.output
                && previous.clip == branch.clip
                && match (&previous.draw, &branch.draw) {
                    (Some(previous), Some(next)) => Arc::ptr_eq(previous, next),
                    (None, None) => true,
                    _ => false,
                }
                && previous.children.ptr_eq(&children)
            {
                Some(Arc::clone(previous))
            } else {
                Some(Arc::new(SceneRope {
                    bounds: children
                        .values()
                        .filter_map(|rope| rope.bounds)
                        .chain(
                            branch
                                .draw
                                .iter()
                                .filter(|draw| draw.visible)
                                .filter_map(|draw| draw.bounds),
                        )
                        .reduce(|a, b| a.union(&b)),
                    draw: branch.draw.clone(),
                    clip: branch.clip,
                    children,
                }))
            };
            if affects_pixels
                && let Some(bounds) = branch.output.as_ref().and_then(|rope| rope.bounds)
            {
                damage = Some(damage.map_or(bounds, |rect: PxRect| rect.union(&bounds)));
            }
            let mut child = node_id;
            let mut parent = branch.parent;
            while let Some(id) = parent {
                if !dirty.contains(&id) {
                    child = id;
                    parent = tree.get(id).and_then(|node| node.parent());
                    continue;
                }
                if let Some(parent) = self.branches.get_mut(&id) {
                    parent.children.update_output(child, branch.output.as_ref());
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
        let previous_graph = self.graph.clone();
        let full = self.viewport != Some(viewport) || self.graph.is_none();
        if !same || full {
            let mut builder = RenderGraphBuilder::new();
            if let Some(root) = &next_root {
                root.emit(&mut builder);
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
                .clone()
                .with_damage(
                    (!full).then_some(damage.unwrap_or(PxRect::ZERO)),
                    previous_graph.as_ref(),
                ),
            updated,
        )
    }
}

fn fragment_bounds(
    fragment: &RenderFragment,
    geometry: DrawGeometry,
    viewport: PxRect,
) -> Option<PxRect> {
    fragment
        .ops()
        .iter()
        .filter_map(|op| {
            let geometry = DrawGeometry::new(
                op.size_override.unwrap_or(geometry.local_size),
                geometry
                    .local_to_window
                    .with_local_origin(op.position_override.unwrap_or(PxPosition::ZERO)),
            );
            let bounds = match &op.command {
                Command::Draw(command) => match command.draw_region() {
                    DrawRegion::Global => viewport,
                    DrawRegion::Absolute(rect) => rect,
                    DrawRegion::PaddedLocal(padding) => {
                        geometry.padded_window_bounds(padding).outward()
                    }
                },
                Command::Compute(_) => geometry.window_bounds().outward(),
                _ => return None,
            };
            bounds.intersection(&viewport)
        })
        .reduce(|a, b| a.union(&b))
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
    fn emit(&self, builder: &mut RenderGraphBuilder) {
        match self.clip {
            Some(SceneClip::Shape(region)) => builder.push_shape_clip_push(region),
            Some(SceneClip::Rect(rect)) => builder.push_clip_push(rect),
            None => {}
        }
        if let Some(draw) = &self.draw {
            if draw.visible {
                builder.append_fragment(draw.fragment.as_ref(), draw.geometry, draw.opacity);
            }
        }
        for child in self.children.values() {
            child.emit(builder);
        }
        if self.clip.is_some() {
            builder.push_clip_pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        __private::{finish_component_node, register_layout_node},
        DrawCommand, Px, RuntimeSession,
        component_tree::{ComponentTree, ComputedData, node::place_node, populate_layout_metadata},
        runtime::TesseraRuntime,
    };

    #[derive(Clone)]
    struct Marker(u32);

    impl DrawCommand for Marker {
        fn apply_opacity(&mut self, _: f32) {}
    }

    fn drawing(graph: &RenderGraph) -> Vec<(u32, PxPosition)> {
        graph
            .ops()
            .iter()
            .filter_map(|op| {
                let marker = op.command.downcast_draw_ref::<Marker>()?;
                Some((marker.0, op.geometry.window_origin()))
            })
            .collect()
    }

    #[test]
    fn retained_scene_preserves_draw_order_and_geometry_across_child_count_changes() {
        let session = RuntimeSession::new();
        let _runtime_scope = session.enter();
        let root = register_layout_node("root", 0);
        let first = register_layout_node("first", 0);
        finish_component_node();
        let second = register_layout_node("second", 0);
        finish_component_node();
        finish_component_node();

        TesseraRuntime::with_mut(|runtime| {
            let tree = &mut runtime.component_tree;
            let viewport = PxSize::new(Px(100), Px(100));
            for (id, value) in [(root, 0), (first, 1), (second, 2)] {
                let metadata = tree.metadatas.entry_or_default(id);
                metadata.computed_data = Some(ComputedData {
                    width: Px(16),
                    height: Px(16),
                });
                metadata.rel_position = Some(PxPosition::ZERO);
                metadata.fragment.push_draw_command(Marker(value));
            }
            let mut scene = FrameScene::default();
            let frame = |scene: &mut FrameScene,
                         tree: &mut ComponentTree,
                         changed: &[NodeId],
                         removed: &[NodeId]| {
                populate_layout_metadata(root, &tree.tree, &mut tree.metadatas);
                let (graph, _) = scene.update(
                    root,
                    &tree.tree,
                    &tree.metadatas,
                    &changed.iter().copied().collect(),
                    &removed.iter().copied().collect(),
                    viewport,
                );
                tree.metadatas.geometry_changed.clear();
                graph
            };

            let initial = frame(&mut scene, tree, &[root, first, second], &[]);
            let previous_root = Arc::clone(scene.root.as_ref().unwrap());
            let zero = PxPosition::ZERO;
            assert_eq!(drawing(&initial), [(0, zero), (1, zero), (2, zero)]);

            let translated = PxPosition::new(Px(20), Px(0));
            place_node(second, translated, 0, &mut tree.metadatas);
            let moved = frame(&mut scene, tree, &[], &[]);
            assert_eq!(drawing(&moved), [(0, zero), (1, zero), (2, translated)]);

            first.detach(&mut tree.tree);
            let one_child = frame(&mut scene, tree, &[], &[first]);
            assert_eq!(drawing(&one_child), [(0, zero), (2, translated)]);

            root.append(first, &mut tree.tree);
            let reordered = frame(&mut scene, tree, &[first], &[]);
            assert_eq!(drawing(&reordered), [(0, zero), (2, translated), (1, zero)]);
            let mut builder = RenderGraphBuilder::new();
            previous_root.emit(&mut builder);
            assert_eq!(
                drawing(&builder.finish()),
                [(0, zero), (1, zero), (2, zero)]
            );
        });
    }
}
