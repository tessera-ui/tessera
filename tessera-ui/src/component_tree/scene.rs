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

use super::{
    ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, NodeRole,
    direct_layout_children, geometry_parent, node_map::DenseNodeMap, record_frame_stage,
};

#[derive(PartialEq)]
enum SceneClip {
    Shape(ClipRegion),
    Rect(PxRect),
}

struct SceneDraw {
    fragment: RenderFragment,
    geometry: DrawGeometry,
    opacity: f32,
    visible: bool,
    bounds: Option<PxRect>,
    partial_redraw: bool,
}

pub(crate) struct SceneRope {
    draw: Option<Arc<SceneDraw>>,
    clip: Option<Arc<SceneClip>>,
    children: SceneChildren,
    bounds: Option<PxRect>,
    partial_redraw: bool,
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
        mut output: impl FnMut(NodeId) -> Option<Arc<SceneRope>>,
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
    children: BranchChildren,
    draw: Option<Arc<SceneDraw>>,
    clip: Option<Arc<SceneClip>>,
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
struct BranchChange {
    own: bool,
    structure: bool,
    visited: bool,
    dirty: bool,
}

impl BranchChange {
    fn affects_pixels(&self) -> bool {
        self.own || self.structure
    }
}

#[derive(Default)]
pub(super) struct FrameScene {
    branches: DenseNodeMap<SceneBranch>,
    geometry_consumers: HashSet<NodeId>,
    structure_changed: HashSet<NodeId>,
    root: Option<Arc<SceneRope>>,
    graph: Option<RenderGraph>,
    viewport: Option<PxSize>,
}

impl FrameScene {
    pub(super) fn invalidate_structure(&mut self, id: NodeId) {
        self.structure_changed.insert(id);
    }

    #[cfg(any(test, feature = "testing"))]
    pub(super) fn clear_structure_changes(&mut self) {
        self.structure_changed.clear();
    }

    fn output(&self, mut node_id: NodeId) -> Option<Arc<SceneRope>> {
        loop {
            let branch = self.branches.get(node_id)?;
            if branch.culled {
                return None;
            }
            // Draw-free, unclipped branches forward the current child output.
            if let Some(child) = branch.transparent_child() {
                node_id = child;
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
        let mut changes = HashMap::<NodeId, BranchChange>::with_capacity_and_hasher(
            self.structure_changed
                .len()
                .max(metadata.paint_changed.len()),
            Default::default(),
        );
        let mut damage = None;
        for node_id in removed {
            let Some(branch) = self.branches.get(*node_id) else {
                continue;
            };
            if let Some(bounds) = branch.output.as_ref().and_then(|rope| rope.bounds) {
                damage = Some(damage.map_or(bounds, |rect: PxRect| rect.union(&bounds)));
            }
            let mut parent = branch.parent;
            if let Some(parent) = parent {
                changes.entry(parent).or_default().structure = true;
            }
            while let Some(id) = parent {
                if std::mem::replace(&mut changes.entry(id).or_default().dirty, true) {
                    break;
                }
                parent = self.branches.get(id).and_then(|branch| branch.parent);
            }
        }
        for node_id in removed {
            if self.branches.remove(*node_id).is_some() {
                self.geometry_consumers.remove(node_id);
            }
        }
        record_frame_stage(&mut stage_timer, "scene_removals");
        for &id in &metadata.paint_changed {
            if is_scene_node(root, tree, id) {
                changes.entry(id).or_default().own = true;
            }
        }
        let viewport_rect = PxRect::enclosing(PxPosition::ZERO, viewport);
        for &node_id in self
            .geometry_consumers
            .intersection(&metadata.geometry_changed)
        {
            let Some(branch) = self.branches.get(node_id) else {
                changes.entry(node_id).or_default().own = true;
                continue;
            };
            let node_metadata = metadata.get(&node_id);
            if branch.culled
                && fully_clipped(tree[node_id].get().role, node_metadata, viewport_rect)
            {
                continue;
            }
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
            changes.entry(node_id).or_default().own = true;
        }
        if self.viewport != Some(viewport) {
            for (id, branch) in self.branches.iter() {
                if branch.culled || branch.draw.is_some() {
                    changes.entry(id).or_default().own = true;
                }
            }
        }
        for id in self.structure_changed.drain() {
            changes.entry(id).or_default().structure = true;
        }
        for &node_id in changed {
            if !is_scene_node(root, tree, node_id) {
                continue;
            }
            let change = changes.entry(node_id).or_default();
            change.own = true;
            // Existing branches receive child changes through the layout journal.
            if self.branches.get(node_id).is_none() {
                change.structure = true;
                if let Some(parent) = geometry_parent(tree, node_id) {
                    changes.entry(parent).or_default().structure = true;
                }
            }
        }
        record_frame_stage(&mut stage_timer, "scene_candidates");
        let candidates: Vec<_> = changes
            .iter()
            .filter_map(|(&id, change)| (change.dirty || change.affects_pixels()).then_some(id))
            .collect();
        let mut ordered = Vec::with_capacity(candidates.len());
        for node_id in candidates {
            if tree.get(node_id).is_none_or(|node| node.is_removed()) {
                continue;
            }
            let start = ordered.len();
            let mut ancestor = Some(node_id);
            while let Some(id) = ancestor {
                let change = changes.entry(id).or_default();
                if std::mem::replace(&mut change.visited, true) {
                    break;
                }
                change.dirty = change.dirty
                    || change.affects_pixels()
                    || self
                        .branches
                        .get(id)
                        .and_then(SceneBranch::transparent_child)
                        .is_none();
                ancestor = geometry_parent(tree, id);
                if change.dirty {
                    ordered.push((id, ancestor));
                }
            }
            // The visited prefix already contains any shared ancestors.
            ordered[start..].reverse();
        }
        record_frame_stage(&mut stage_timer, "scene_ancestry");
        tracing::debug!(
            target: "tessera_ui::frame_cost",
            structural_nodes = changes.values().filter(|change| change.structure).count(),
            geometry_changes = metadata.geometry_changed.len(),
            own_changes = changes.values().filter(|change| change.own).count(),
            updated_branches = ordered.len(),
            "scene_update_work"
        );
        let updated = ordered.len() as u64;
        for (node_id, parent) in ordered.into_iter().rev() {
            let change = &changes[&node_id];
            let node = &tree[node_id];
            let mut branch = self
                .branches
                .get_mut(node_id)
                .map(std::mem::take)
                .unwrap_or_default();
            let affects_pixels = change.affects_pixels();
            if affects_pixels
                && let Some(bounds) = branch.output.as_ref().and_then(|rope| rope.bounds)
            {
                damage = Some(damage.map_or(bounds, |rect: PxRect| rect.union(&bounds)));
            }
            branch.parent = parent;
            if change.structure {
                branch.children.rebuild(
                    direct_layout_children(node_id, tree).iter().copied(),
                    |child| {
                        if let Some(branch) = self.branches.get_mut(child) {
                            branch.parent = Some(node_id);
                        }
                        self.output(child)
                    },
                );
            }
            let was_transparent = branch.transparent_child().is_some();
            if change.own {
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
                            .map(|draw| draw.fragment.clone())
                            .unwrap_or_else(|| metadata.fragment.clone());
                        let geometry = DrawGeometry::new(prepared.size, prepared.local_to_window);
                        let visible = geometry.local_size.width.0 > 0
                            && geometry.local_size.height.0 > 0
                            && !geometry
                                .window_bounds()
                                .outward()
                                .is_orthogonal(&viewport_rect);
                        let bounds = fragment_bounds(&fragment, geometry, viewport_rect);
                        if let Some(draw) = &branch.draw
                            && draw.fragment.shares_ops(&fragment)
                            && draw.geometry == geometry
                            && draw.opacity == prepared.cumulative_opacity
                            && draw.visible == visible
                            && draw.bounds == bounds
                        {
                            return Arc::clone(draw);
                        }
                        Arc::new(SceneDraw {
                            partial_redraw: prepared.cumulative_opacity == 1.0
                                && fragment.supports_partial_redraw(),
                            bounds,
                            fragment,
                            geometry,
                            opacity: prepared.cumulative_opacity,
                            visible,
                        })
                    })
                });
                let clip = prepared
                    .filter(|prepared| prepared.clips_children)
                    .map(|prepared| {
                        node_metadata
                            .and_then(|metadata| metadata.prepared_clip_region())
                            .copied()
                            .map_or_else(
                                || {
                                    SceneClip::Rect(
                                        prepared
                                            .child_clip_bounds
                                            .map(crate::FloatRect::outward)
                                            .unwrap_or(PxRect::ZERO),
                                    )
                                },
                                SceneClip::Shape,
                            )
                    });
                if branch.clip.as_deref() != clip.as_ref() {
                    branch.clip = clip.map(Arc::new);
                }
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
            branch.culled = fully_clipped(node.get().role, metadata.get(&node_id), viewport_rect);
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
                let visible_draw = branch.draw.as_ref().filter(|draw| draw.visible);
                let (bounds, partial_redraw) = children.values().fold(
                    (
                        visible_draw.and_then(|draw| draw.bounds),
                        visible_draw.is_none_or(|draw| draw.partial_redraw),
                    ),
                    |(bounds, partial_redraw), child| {
                        (
                            child.bounds.map_or(bounds, |child_bounds| {
                                Some(
                                    bounds
                                        .map_or(child_bounds, |bounds| bounds.union(&child_bounds)),
                                )
                            }),
                            partial_redraw && child.partial_redraw,
                        )
                    },
                );
                Some(Arc::new(SceneRope {
                    bounds,
                    partial_redraw,
                    draw: branch.draw.clone(),
                    clip: branch.clip.clone(),
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
                if changes.get(&id).is_none_or(|change| !change.dirty) {
                    child = id;
                    parent = geometry_parent(tree, id);
                    continue;
                }
                if let Some(parent) = self.branches.get_mut(id) {
                    parent.children.update_output(child, branch.output.as_ref());
                }
                break;
            }
            self.branches.insert(node_id, branch);
        }
        if !removed.is_empty() {
            self.branches.trim_spare_capacity();
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
            let local_damage = damage.is_some_and(|rect| {
                i64::from(rect.width.0) * i64::from(rect.height.0)
                    < i64::from(viewport_rect.width.0) * i64::from(viewport_rect.height.0) / 2
            });
            self.graph = Some(match &next_root {
                Some(root) if !full && local_damage && root.partial_redraw => {
                    RenderGraph::from_partial_scene(Arc::clone(root))
                }
                root => {
                    let mut builder = RenderGraphBuilder::new();
                    if let Some(root) = root {
                        root.emit(&mut builder, None);
                    }
                    builder.finish()
                }
            });
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

fn is_scene_node(root: NodeId, tree: &ComponentNodeTree, id: NodeId) -> bool {
    tree.get(id).is_some_and(|node| {
        !node.is_removed() && (id == root || node.get().role == NodeRole::Layout)
    })
}

fn fully_clipped(
    role: NodeRole,
    metadata: Option<&ComponentNodeMetaData>,
    viewport: PxRect,
) -> bool {
    (role == NodeRole::Layout && metadata.is_none_or(|node| node.abs_position.is_none()))
        || metadata
            .and_then(|node| node.prepared_layout)
            .and_then(|prepared| prepared.child_clip_bounds)
            .is_some_and(|bounds| {
                bounds.width <= 0.0
                    || bounds.height <= 0.0
                    || bounds.outward().is_orthogonal(&viewport)
            })
}

impl SceneRope {
    pub(crate) fn emit(&self, builder: &mut RenderGraphBuilder, damage: Option<PxRect>) {
        let intersects_damage = |bounds: Option<PxRect>| {
            damage.is_none_or(|damage| {
                bounds.is_some_and(|bounds| bounds.intersection(&damage).is_some())
            })
        };
        if !intersects_damage(self.bounds) {
            return;
        }
        match self.clip.as_deref() {
            Some(SceneClip::Shape(region)) => builder.push_shape_clip_push(*region),
            Some(SceneClip::Rect(rect)) => builder.push_clip_push(*rect),
            None => {}
        }
        if let Some(draw) = &self.draw {
            if draw.visible && intersects_damage(draw.bounds) {
                builder.append_fragment(&draw.fragment, draw.geometry, draw.opacity);
            }
        }
        for child in self.children.values() {
            child.emit(builder, damage);
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
        DrawCommand, Px, RuntimeSession,
        component_tree::{
            ComponentNode, ComponentTree, ComputedData, node::place_node, populate_layout_metadata,
        },
        runtime::TesseraRuntime,
    };

    #[derive(Clone)]
    struct Marker(u32);

    impl DrawCommand for Marker {
        fn supports_partial_redraw(&self) -> bool {
            true
        }

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
    fn local_damage_keeps_ancestor_shape_clips_and_can_expand_the_full_scene() {
        let session = RuntimeSession::new();
        let _runtime_scope = session.enter();
        let mut tree = ComponentTree::new();
        let root = tree.add_node(ComponentNode::new("root", NodeRole::Layout));
        let clip = tree.add_node(ComponentNode::new("clip", NodeRole::Layout));
        let changed = tree.add_node(ComponentNode::new("changed", NodeRole::Layout));
        tree.pop_node();
        tree.pop_node();
        let outside = tree.add_node(ComponentNode::new("outside", NodeRole::Layout));
        tree.pop_node();
        tree.pop_node();

        let zero = PxPosition::ZERO;
        let far = PxPosition::new(Px(64), Px(64));
        for (id, size, position) in [
            (root, 100, zero),
            (clip, 16, zero),
            (changed, 16, zero),
            (outside, 16, far),
        ] {
            let metadata = tree.metadatas.entry_or_default(id);
            metadata.computed_data = Some(ComputedData {
                width: Px(size),
                height: Px(size),
            });
            metadata.rel_position = Some(position);
        }
        let clip_metadata = tree.metadatas.entry_or_default(clip);
        clip_metadata.clips_children = true;
        let original_clip = ClipRegion {
            position: zero,
            size: [16.0, 16.0],
            shape: crate::ClipShape::RoundedRectangle {
                radii: [3.0; 4],
                exponents: [2.0; 4],
            },
        };
        clip_metadata.set_clip_region(Some(original_clip));
        tree.metadatas
            .entry_or_default(changed)
            .fragment
            .push_draw_command(Marker(1));
        tree.metadatas
            .entry_or_default(outside)
            .fragment
            .push_draw_command(Marker(2));

        let viewport = PxSize::new(Px(100), Px(100));
        populate_layout_metadata(root, &tree.tree, &mut tree.metadatas);
        let mut scene = FrameScene::default();
        let (previous, _) = scene.update(
            root,
            &tree.tree,
            &tree.metadatas,
            &root.descendants(&tree.tree).collect(),
            &HashSet::default(),
            viewport,
        );
        tree.metadatas.geometry_changed.clear();
        let fragment = &mut tree.metadatas.entry_or_default(changed).fragment;
        fragment.clear();
        fragment.push_draw_command(Marker(3));
        tree.metadatas.paint_changed.insert(changed);
        let (graph, _) = scene.update(
            root,
            &tree.tree,
            &tree.metadatas,
            &HashSet::default(),
            &HashSet::default(),
            viewport,
        );

        let damage = graph.damage_since(Some(&previous)).expect("local damage");
        let regional = graph.for_damage(damage).expect("regional graph");
        assert!(matches!(
            regional.ops(),
            [
                crate::render_graph::RenderGraphOp {
                    command: Command::ShapeClipPush(_),
                    ..
                },
                crate::render_graph::RenderGraphOp {
                    command: Command::Draw(_),
                    ..
                },
                crate::render_graph::RenderGraphOp {
                    command: Command::ClipPop,
                    ..
                },
            ]
        ));
        assert_eq!(drawing(&regional), [(3, zero)]);
        let new_clip = ClipRegion {
            shape: crate::ClipShape::Ellipse,
            ..original_clip
        };
        tree.metadatas
            .entry_or_default(clip)
            .set_clip_region(Some(new_clip));
        tree.metadatas.mark_geometry_dirty(clip);
        populate_layout_metadata(root, &tree.tree, &mut tree.metadatas);
        let (updated, _) = scene.update(
            root,
            &tree.tree,
            &tree.metadatas,
            &HashSet::default(),
            &HashSet::default(),
            viewport,
        );

        assert_eq!(graph.damage_since(None), None);
        assert!(matches!(
            &graph.ops()[0].command,
            Command::ShapeClipPush(clip) if *clip == original_clip
        ));
        assert!(matches!(
            &updated.ops()[0].command,
            Command::ShapeClipPush(clip) if *clip == new_clip
        ));
        assert_eq!(drawing(&graph), [(3, zero), (2, far)]);
    }

    #[test]
    fn retained_scene_preserves_draw_order_and_geometry_across_child_count_changes() {
        let session = RuntimeSession::new();
        let _runtime_scope = session.enter();
        TesseraRuntime::with_mut(|runtime| {
            let tree = &mut runtime.component_tree;
            let root = tree.add_node(ComponentNode::new("root", NodeRole::Layout));
            let wrapper = tree.add_node(ComponentNode::new("wrapper", NodeRole::Composition));
            let first = tree.add_node(ComponentNode::new("first", NodeRole::Layout));
            tree.pop_node();
            tree.pop_node();
            let second = tree.add_node(ComponentNode::new("second", NodeRole::Layout));
            tree.pop_node();
            tree.pop_node();
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
            let frame = |tree: &mut ComponentTree, changed: &[NodeId], removed: &[NodeId]| {
                populate_layout_metadata(root, &tree.tree, &mut tree.metadatas);
                let (graph, _) = tree.frame_scene.update(
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

            let initial = frame(tree, &[root, wrapper, first, second], &[]);
            let previous_root = Arc::clone(tree.frame_scene.root.as_ref().unwrap());
            let zero = PxPosition::ZERO;
            assert_eq!(drawing(&initial), [(0, zero), (1, zero), (2, zero)]);

            let translated = PxPosition::new(Px(20), Px(0));
            place_node(second, translated, 0, &mut tree.metadatas);
            let moved = frame(tree, &[], &[]);
            assert_eq!(drawing(&moved), [(0, zero), (1, zero), (2, translated)]);

            root.append(wrapper, &mut tree.tree);
            tree.refresh_layout_children(root);
            let reordered_wrapper = frame(tree, &[], &[]);
            assert_eq!(
                drawing(&reordered_wrapper),
                [(0, zero), (2, translated), (1, zero)]
            );

            first.detach(&mut tree.tree);
            tree.refresh_layout_children(wrapper);
            tree.refresh_layout_children(root);
            let one_child = frame(tree, &[], &[first]);
            assert_eq!(drawing(&one_child), [(0, zero), (2, translated)]);

            root.append(first, &mut tree.tree);
            tree.refresh_layout_children(root);
            let reordered = frame(tree, &[first], &[]);
            assert_eq!(drawing(&reordered), [(0, zero), (2, translated), (1, zero)]);

            let offset = PxPosition::new(Px(4), Px(0));
            tree.metadatas.entry_or_default(first).fragment.ops_mut()[0].position_override =
                Some(offset);
            let repainted = frame(tree, &[first], &[]);
            assert_eq!(
                repainted.damage_since(Some(&reordered)),
                Some(PxRect::new(Px::ZERO, Px::ZERO, Px(20), Px(16)))
            );
            assert_eq!(
                drawing(&repainted),
                [(0, zero), (2, translated), (1, offset)]
            );

            tree.metadatas.remove(&first);
            first.remove_subtree(&mut tree.tree);
            let replacement = tree
                .tree
                .new_node(ComponentNode::new("replacement", NodeRole::Layout));
            assert_eq!(usize::from(first), usize::from(replacement));
            assert_ne!(first, replacement);
            root.append(replacement, &mut tree.tree);
            tree.refresh_layout_children(root);
            let metadata = tree.metadatas.entry_or_default(replacement);
            metadata.computed_data = Some(ComputedData {
                width: Px(16),
                height: Px(16),
            });
            metadata.rel_position = Some(zero);
            metadata.fragment.push_draw_command(Marker(3));
            assert!(tree.metadatas.remove(&first).is_none());
            let recycled = frame(tree, &[replacement], &[first]);
            assert_eq!(drawing(&recycled), [(0, zero), (2, translated), (3, zero)]);

            let mut builder = RenderGraphBuilder::new();
            previous_root.emit(&mut builder, None);
            assert_eq!(
                drawing(&builder.finish()),
                [(0, zero), (1, zero), (2, zero)]
            );
        });
    }
}
