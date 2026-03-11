//! This module provides the global runtime state management for tessera.

mod build_scope;
mod composition;
mod session;
mod slot_table;

use std::sync::{Arc, OnceLock};

use parking_lot::RwLock;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    NodeId,
    component_tree::{ComponentNode, ComponentTree},
    focus::{
        FocusDirection, FocusGroupNode, FocusHandleId, FocusNode, FocusProperties,
        FocusRegistration, FocusRegistrationKind, FocusRequester, FocusRevealRequest,
        FocusScopeNode, FocusState, FocusTraversalPolicy,
    },
    layout::{LayoutSpec, LayoutSpecDyn},
    prop::{CallbackWith, ComponentReplayData, ErasedComponentRunner, Prop},
};

pub use self::{
    build_scope::{
        CurrentComponentInstanceGuard, GroupGuard, InstanceKeyGuard, NodeContextGuard,
        PathGroupGuard, PhaseGuard, RuntimePhase, current_instance_key, current_instance_logic_id,
        current_node_id, key, push_current_component_instance_key, push_current_node,
        push_current_node_with_instance_logic_id, push_phase,
    },
    composition::{
        FrameNanosControl, current_frame_nanos, current_frame_time, frame_delta,
        persistent_focus_group_for_current_instance,
        persistent_focus_requester_for_current_instance,
        persistent_focus_scope_for_current_instance, persistent_focus_target_for_current_instance,
        receive_frame_nanos,
    },
    slot_table::{State, remember, remember_with_key, retain, retain_with_key},
};

pub(crate) use self::{
    build_scope::{
        compute_context_slot_key, current_component_instance_key_from_scope, current_group_path,
        current_instance_key_override, current_phase, ensure_build_phase,
        is_instance_key_build_dirty, with_build_dirty_instance_keys, with_replay_scope,
    },
    composition::{
        CompositionRuntime, ContextReadDependencyKey, ContextReadDependencyTracker,
        ContextSlotEntry, ContextSlotKey, ContextSlotTable, ContextSnapshotTracker,
        ReplayNodeSnapshot, begin_frame_clock, begin_frame_component_replay_tracking,
        clear_frame_nanos_receivers, clear_persistent_focus_handles, clear_redraw_waker,
        finalize_frame_component_replay_tracking, finalize_frame_component_replay_tracking_partial,
        focus_read_subscribers, focus_requester_read_subscribers, has_pending_build_invalidations,
        has_pending_frame_nanos_receivers, has_persistent_focus_handle, install_redraw_waker,
        previous_component_replay_nodes, record_component_invalidation_for_instance_key,
        remove_build_invalidations, remove_focus_read_dependencies, remove_frame_nanos_receivers,
        remove_previous_component_replay_nodes, remove_render_slot_read_dependencies,
        remove_state_read_dependencies, reset_build_invalidations, reset_component_replay_tracking,
        reset_focus_read_dependencies, reset_frame_clock, reset_render_slot_read_dependencies,
        reset_state_read_dependencies, retain_persistent_focus_handles, take_build_invalidations,
        tick_frame_nanos_receivers, track_focus_read_dependency,
        track_focus_requester_read_dependency, track_render_slot_read_dependency,
    },
    session::{
        bind_current_runtime, with_bound_runtime, with_bound_runtime_mut, with_composition_runtime,
    },
    slot_table::{
        FunctorHandle, SlotHandle, begin_recompose_slot_epoch, drop_slots_for_instance_logic_ids,
        invoke_callback_handle, invoke_callback_with_handle, invoke_render_slot_handle,
        invoke_render_slot_with_handle, live_slot_instance_logic_ids,
        recycle_recomposed_slots_for_instance_logic_ids, remember_callback_handle,
        remember_callback_with_handle, remember_render_slot_handle,
        remember_render_slot_with_handle, reset_slots,
    },
};

#[derive(Default)]
struct LayoutDirtyTracker {
    previous_layout_specs_by_node: HashMap<u64, Box<dyn LayoutSpecDyn>>,
    frame_layout_specs_by_node: HashMap<u64, Box<dyn LayoutSpecDyn>>,
    pending_self_dirty_nodes: HashSet<u64>,
    ready_self_dirty_nodes: HashSet<u64>,
    previous_children_by_node: HashMap<u64, Vec<u64>>,
}

#[derive(Default)]
pub(crate) struct StructureReconcileResult {
    pub changed_nodes: HashSet<u64>,
    pub removed_nodes: HashSet<u64>,
}

static LAYOUT_DIRTY_TRACKER: OnceLock<RwLock<LayoutDirtyTracker>> = OnceLock::new();

fn layout_dirty_tracker() -> &'static RwLock<LayoutDirtyTracker> {
    LAYOUT_DIRTY_TRACKER.get_or_init(|| RwLock::new(LayoutDirtyTracker::default()))
}

fn record_layout_spec_dirty(instance_key: u64, layout_spec: &dyn LayoutSpecDyn) {
    if current_phase() != Some(RuntimePhase::Build) {
        return;
    }
    let mut tracker = layout_dirty_tracker().write();
    let (changed, next_layout_spec) =
        match tracker.previous_layout_specs_by_node.remove(&instance_key) {
            Some(previous) => {
                if previous.dyn_eq(layout_spec) {
                    (false, previous)
                } else {
                    (true, layout_spec.clone_box())
                }
            }
            None => (true, layout_spec.clone_box()),
        };
    if changed {
        tracker.pending_self_dirty_nodes.insert(instance_key);
    }
    tracker
        .frame_layout_specs_by_node
        .insert(instance_key, next_layout_spec);
}

pub(crate) fn begin_frame_layout_dirty_tracking() {
    let mut tracker = layout_dirty_tracker().write();
    tracker.frame_layout_specs_by_node.clear();
    tracker.pending_self_dirty_nodes.clear();
}

pub(crate) fn finalize_frame_layout_dirty_tracking() {
    let mut tracker = layout_dirty_tracker().write();
    tracker.ready_self_dirty_nodes = std::mem::take(&mut tracker.pending_self_dirty_nodes);
    tracker.previous_layout_specs_by_node = std::mem::take(&mut tracker.frame_layout_specs_by_node);
}

pub(crate) fn take_layout_self_dirty_nodes() -> HashSet<u64> {
    std::mem::take(&mut layout_dirty_tracker().write().ready_self_dirty_nodes)
}

pub(crate) fn reset_layout_dirty_tracking() {
    *layout_dirty_tracker().write() = LayoutDirtyTracker::default();
}

fn record_component_replay_snapshot(runtime: &TesseraRuntime, node_id: NodeId) {
    let Some(node) = runtime.component_tree.get(node_id) else {
        return;
    };
    let Some(replay) = node.replay.clone() else {
        return;
    };

    let snapshot = ReplayNodeSnapshot {
        instance_key: node.instance_key,
        instance_logic_id: node.instance_logic_id,
        group_path: current_group_path(),
        instance_key_override: current_instance_key_override(),
        replay,
    };
    runtime
        .composition
        .record_component_replay_snapshot(snapshot);
}

pub(crate) fn reconcile_layout_structure(
    current_children_by_node: &HashMap<u64, Vec<u64>>,
) -> StructureReconcileResult {
    let mut tracker = layout_dirty_tracker().write();
    let previous_children_by_node = &tracker.previous_children_by_node;

    let mut changed_nodes = HashSet::default();
    let mut removed_nodes = HashSet::default();

    for (node, current_children) in current_children_by_node {
        match previous_children_by_node.get(node) {
            Some(previous_children) if previous_children == current_children => {}
            _ => {
                changed_nodes.insert(*node);
            }
        }
    }

    for node in previous_children_by_node.keys().copied() {
        if !current_children_by_node.contains_key(&node) {
            changed_nodes.insert(node);
            removed_nodes.insert(node);
        }
    }

    tracker.previous_children_by_node = current_children_by_node.clone();
    StructureReconcileResult {
        changed_nodes,
        removed_nodes,
    }
}

pub struct TesseraRuntime {
    /// Composition execution state for the current UI session.
    pub(crate) composition: Arc<CompositionRuntime>,
    /// Hierarchical structure of all UI components in the application.
    pub component_tree: ComponentTree,
    /// Current window dimensions in physical pixels.
    pub(crate) window_size: [u32; 2],
    /// Cursor icon change request from UI components.
    pub cursor_icon_request: Option<winit::window::CursorIcon>,
    /// Whether the window is currently minimized.
    pub(crate) window_minimized: bool,
}

impl Default for TesseraRuntime {
    fn default() -> Self {
        Self {
            composition: Arc::new(CompositionRuntime::default()),
            component_tree: ComponentTree::default(),
            window_size: Default::default(),
            cursor_icon_request: Default::default(),
            window_minimized: Default::default(),
        }
    }
}

impl TesseraRuntime {
    /// Executes a closure with a shared, read-only reference to the runtime.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        with_bound_runtime(f).expect("TesseraRuntime::with requires an active runtime session")
    }

    /// Executes a closure with an exclusive, mutable reference to the runtime.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        with_bound_runtime_mut(f)
            .expect("TesseraRuntime::with_mut requires an active runtime session")
    }

    /// Get the current window size in physical pixels.
    pub fn window_size(&self) -> [u32; 2] {
        self.window_size
    }
}

impl TesseraRuntime {
    fn current_node(&self) -> Option<&ComponentNode> {
        self.component_tree.current_node()
    }

    fn with_current_node_mut<R>(
        &mut self,
        missing_message: &'static str,
        f: impl FnOnce(&mut ComponentNode) -> R,
    ) -> Option<R> {
        let Some(node) = self.component_tree.current_node_mut() else {
            debug_assert!(false, "{missing_message}");
            return None;
        };
        Some(f(node))
    }

    fn with_current_focus_registration_mut<R>(
        &mut self,
        missing_node_message: &'static str,
        missing_registration_message: &'static str,
        f: impl FnOnce(&mut FocusRegistration) -> R,
    ) -> Option<R> {
        self.with_current_node_mut(missing_node_message, |current| {
            let Some(registration) = current.focus_registration.as_mut() else {
                debug_assert!(false, "{missing_registration_message}");
                return None;
            };
            Some(f(registration))
        })
        .flatten()
    }

    fn current_focus_handle<H>(
        &self,
        kind: FocusRegistrationKind,
        map: impl FnOnce(FocusHandleId) -> H,
    ) -> Option<H> {
        let registration = self.current_node()?.focus_registration?;
        (registration.kind == kind).then(|| map(registration.id))
    }
}

impl TesseraRuntime {
    /// Sets identity fields for the current component node.
    #[doc(hidden)]
    pub fn set_current_node_identity(&mut self, instance_key: u64, instance_logic_id: u64) {
        let _ = self.with_current_node_mut(
            "set_current_node_identity must be called inside a component build",
            |node| {
                node.instance_key = instance_key;
                node.instance_logic_id = instance_logic_id;
            },
        );
    }

    /// Stores replay metadata for the current component node.
    #[doc(hidden)]
    pub fn set_current_component_replay<P>(
        &mut self,
        runner: Arc<dyn ErasedComponentRunner>,
        props: &P,
    ) -> bool
    where
        P: Prop,
    {
        let current_node_info = self
            .current_node()
            .map(|node| (node.instance_key, node.instance_logic_id));
        let previous_replay = current_node_info.and_then(|(instance_key, instance_logic_id)| {
            let previous = self
                .composition
                .previous_component_replay_node(instance_key)?;
            if previous.instance_logic_id != instance_logic_id {
                return None;
            }
            if previous.replay.props.equals(props) {
                Some(previous.replay.clone())
            } else {
                None
            }
        });

        let pending_dirty = current_node_info
            .map(|(instance_key, _)| {
                self.composition
                    .consume_pending_build_invalidation(instance_key)
            })
            .unwrap_or(false);

        let Some((instance_key, instance_logic_id)) = current_node_info else {
            debug_assert!(
                false,
                "set_current_component_replay must be called inside a component build"
            );
            return false;
        };

        if let Some(replay) = previous_replay.clone()
            && !is_instance_key_build_dirty(instance_key)
            && !pending_dirty
            && self
                .component_tree
                .try_reuse_current_subtree(instance_key, instance_logic_id)
        {
            if let Some(node) = self.component_tree.current_node_mut() {
                node.replay = Some(replay);
                node.props_unchanged_from_previous = true;
            }
            return true;
        }

        let _ = self.with_current_node_mut(
            "set_current_component_replay must be called inside a component build",
            |node| {
                if let Some(replay) = previous_replay {
                    node.replay = Some(replay);
                    node.props_unchanged_from_previous = true;
                } else {
                    node.replay = Some(ComponentReplayData::new(runner, props));
                    node.props_unchanged_from_previous = false;
                }
            },
        );
        if let Some(node_id) = current_node_id() {
            record_component_replay_snapshot(self, node_id);
        }
        false
    }

    /// Sets the layout spec for the current component node.
    #[doc(hidden)]
    pub fn set_current_layout_spec<S>(&mut self, spec: S)
    where
        S: LayoutSpec,
    {
        let _ = self.with_current_node_mut(
            "set_current_layout_spec must be called inside a component build",
            |node| {
                node.layout_spec = Box::new(spec) as Box<dyn LayoutSpecDyn>;
            },
        );
    }

    /// Binds a focus requester to the current component node.
    #[doc(hidden)]
    pub fn bind_current_focus_requester(&mut self, requester: FocusRequester) {
        let _ = self.with_current_node_mut(
            "bind_current_focus_requester must be called inside a component build",
            |current| {
                current.focus_requester_binding = Some(requester);
            },
        );
    }

    /// Registers the current component node as a focus target.
    #[doc(hidden)]
    pub fn register_current_focus_target(&mut self, node: FocusNode) {
        let _ = self.with_current_node_mut(
            "register_current_focus_target must be called inside a component build",
            |current| {
                current.focus_registration = Some(FocusRegistration::target(node));
            },
        );
    }

    /// Ensures the current component node has a focus target registration.
    #[doc(hidden)]
    pub fn ensure_current_focus_target(&mut self, node: FocusNode) {
        let _ = self.with_current_node_mut(
            "ensure_current_focus_target must be called inside a component build",
            |current| {
                if current.focus_registration.is_none() {
                    current.focus_registration = Some(FocusRegistration::target(node));
                }
            },
        );
    }

    /// Registers the current component node as a focus scope.
    #[doc(hidden)]
    pub fn register_current_focus_scope(&mut self, scope: FocusScopeNode) {
        let _ = self.with_current_node_mut(
            "register_current_focus_scope must be called inside a component build",
            |current| {
                current.focus_registration = Some(FocusRegistration::scope(scope));
            },
        );
    }

    /// Ensures the current component node has a focus scope registration.
    #[doc(hidden)]
    pub fn ensure_current_focus_scope(&mut self, scope: FocusScopeNode) {
        let _ = self.with_current_node_mut(
            "ensure_current_focus_scope must be called inside a component build",
            |current| {
                if current.focus_registration.is_none() {
                    current.focus_registration = Some(FocusRegistration::scope(scope));
                }
            },
        );
    }

    /// Registers the current component node as a focus traversal group.
    #[doc(hidden)]
    pub fn register_current_focus_group(&mut self, group: FocusGroupNode) {
        let _ = self.with_current_node_mut(
            "register_current_focus_group must be called inside a component build",
            |current| {
                current.focus_registration = Some(FocusRegistration::group(group));
            },
        );
    }

    /// Ensures the current component node has a focus traversal group
    /// registration.
    #[doc(hidden)]
    pub fn ensure_current_focus_group(&mut self, group: FocusGroupNode) {
        let _ = self.with_current_node_mut(
            "ensure_current_focus_group must be called inside a component build",
            |current| {
                if current.focus_registration.is_none() {
                    current.focus_registration = Some(FocusRegistration::group(group));
                }
            },
        );
    }

    /// Returns the current component node's registered focus target, if any.
    #[doc(hidden)]
    pub fn current_focus_target_handle(&self) -> Option<FocusNode> {
        self.current_focus_handle(FocusRegistrationKind::Target, FocusNode::from_handle_id)
    }

    /// Returns the current component node's registered focus scope, if any.
    #[doc(hidden)]
    pub fn current_focus_scope_handle(&self) -> Option<FocusScopeNode> {
        self.current_focus_handle(FocusRegistrationKind::Scope, FocusScopeNode::from_handle_id)
    }

    /// Returns the current component node's registered focus group, if any.
    #[doc(hidden)]
    pub fn current_focus_group_handle(&self) -> Option<FocusGroupNode> {
        self.current_focus_handle(FocusRegistrationKind::Group, FocusGroupNode::from_handle_id)
    }

    /// Updates focus properties for the current component node registration.
    #[doc(hidden)]
    pub fn set_current_focus_properties(&mut self, properties: FocusProperties) {
        let _ = self.with_current_focus_registration_mut(
            "set_current_focus_properties must be called inside a component build",
            "set_current_focus_properties requires focus_target, focus_scope, or focus_group first",
            |registration| {
                registration.properties = properties;
            },
        );
    }

    /// Updates the traversal policy for the current focus scope or group.
    #[doc(hidden)]
    pub fn set_current_focus_traversal_policy(&mut self, policy: FocusTraversalPolicy) {
        let _ = self.with_current_node_mut(
            "set_current_focus_traversal_policy must be called inside a component build",
            |current| {
                if current.focus_registration.is_some_and(|registration| {
                    matches!(
                        registration.kind,
                        FocusRegistrationKind::Scope | FocusRegistrationKind::Group
                    )
                }) {
                    current.focus_traversal_policy = Some(policy);
                } else {
                    debug_assert!(
                        false,
                        "set_current_focus_traversal_policy requires focus_scope or focus_group first"
                    );
                }
            },
        );
    }

    /// Registers a focus-changed callback on the current component node.
    #[doc(hidden)]
    pub fn set_current_focus_changed_handler(&mut self, handler: CallbackWith<FocusState>) {
        let _ = self.with_current_node_mut(
            "set_current_focus_changed_handler must be called inside a component build",
            |current| {
                current.focus_changed_handler = Some(handler);
            },
        );
    }

    /// Registers a focus-event callback on the current component node.
    #[doc(hidden)]
    pub fn set_current_focus_event_handler(&mut self, handler: CallbackWith<FocusState>) {
        let _ = self.with_current_node_mut(
            "set_current_focus_event_handler must be called inside a component build",
            |current| {
                current.focus_event_handler = Some(handler);
            },
        );
    }

    /// Registers a beyond-bounds focus callback on the current component node.
    #[doc(hidden)]
    pub fn set_current_focus_beyond_bounds_handler(
        &mut self,
        handler: CallbackWith<FocusDirection, bool>,
    ) {
        let _ = self.with_current_node_mut(
            "set_current_focus_beyond_bounds_handler must be called inside a component build",
            |current| {
                current.focus_beyond_bounds_handler = Some(handler);
            },
        );
    }

    /// Registers a focus reveal callback on the current component node.
    #[doc(hidden)]
    pub fn set_current_focus_reveal_handler(
        &mut self,
        handler: CallbackWith<FocusRevealRequest, bool>,
    ) {
        let _ = self.with_current_node_mut(
            "set_current_focus_reveal_handler must be called inside a component build",
            |current| {
                current.focus_reveal_handler = Some(handler);
            },
        );
    }

    /// Registers a restorer fallback on the current focus scope node.
    #[doc(hidden)]
    pub fn set_current_focus_restorer_fallback(&mut self, fallback: FocusRequester) {
        let _ = self.with_current_node_mut(
            "set_current_focus_restorer_fallback must be called inside a component build",
            |current| {
                if current
                    .focus_registration
                    .is_some_and(|registration| registration.kind == FocusRegistrationKind::Scope)
                {
                    current.focus_restorer_fallback = Some(fallback);
                } else {
                    debug_assert!(
                        false,
                        "set_current_focus_restorer_fallback requires focus_scope or focus_restorer first"
                    );
                }
            },
        );
    }

    /// Records the final layout spec snapshot for the current node.
    #[doc(hidden)]
    pub fn finalize_current_layout_spec_dirty(&mut self) {
        if let Some(node) = self.current_node() {
            record_layout_spec_dirty(node.instance_key, node.layout_spec.as_ref());
        } else {
            debug_assert!(
                false,
                "finalize_current_layout_spec_dirty must be called inside a component build"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Instant;

    use super::*;
    use crate::execution_context::{with_execution_context, with_execution_context_mut};
    use crate::prop::{Callback, RenderSlot};
    use crate::runtime::build_scope::take_next_node_instance_logic_id_override;
    use crate::runtime::slot_table::{SlotEntry, SlotKey, SlotTable, slot_table};
    use crate::testing::with_tessera;

    fn with_test_component_scope<R>(component_type_id: u64, f: impl FnOnce() -> R) -> R {
        let mut arena = crate::Arena::<()>::new();
        let node_id = arena.new_node(());
        let _phase_guard = push_phase(RuntimePhase::Build);
        let _node_guard = push_current_node(node_id, component_type_id, "test_component");
        let _instance_guard = push_current_component_instance_key(current_instance_key());
        f()
    }

    #[test]
    fn frame_receiver_uses_component_scope_instance_key() {
        with_tessera(|| {
            let _instance_guard = push_current_component_instance_key(7);
            assert_eq!(current_component_instance_key_from_scope(), Some(7));
        });
    }

    #[test]
    fn receive_frame_nanos_panics_without_component_scope() {
        with_tessera(|| {
            reset_frame_clock();
            begin_frame_clock(Instant::now());

            let result = std::panic::catch_unwind(|| {
                receive_frame_nanos(|_| FrameNanosControl::Continue);
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn receive_frame_nanos_panics_in_input_phase() {
        with_tessera(|| {
            let _phase_guard = push_phase(RuntimePhase::Input);
            let result = std::panic::catch_unwind(|| {
                receive_frame_nanos(|_| FrameNanosControl::Continue);
            });
            assert!(result.is_err());
        });
    }

    #[test]
    fn tick_frame_nanos_receivers_removes_stopped_receivers() {
        with_tessera(|| {
            reset_frame_clock();
            begin_frame_clock(Instant::now());

            with_test_component_scope(123, || {
                receive_frame_nanos(|_| FrameNanosControl::Stop);
            });

            tick_frame_nanos_receivers();
            assert!(!has_pending_frame_nanos_receivers());
        });
    }

    #[test]
    fn with_build_dirty_instance_keys_marks_current_scope() {
        with_tessera(|| {
            let mut outer = HashSet::default();
            outer.insert(7);

            assert!(!is_instance_key_build_dirty(7));
            with_build_dirty_instance_keys(&outer, || {
                assert!(is_instance_key_build_dirty(7));
                assert!(!is_instance_key_build_dirty(8));

                let mut inner = HashSet::default();
                inner.insert(8);
                with_build_dirty_instance_keys(&inner, || {
                    assert!(!is_instance_key_build_dirty(7));
                    assert!(is_instance_key_build_dirty(8));
                });

                assert!(is_instance_key_build_dirty(7));
                assert!(!is_instance_key_build_dirty(8));
            });
            assert!(!is_instance_key_build_dirty(7));
        });
    }

    #[test]
    fn with_build_dirty_instance_keys_restores_on_panic() {
        with_tessera(|| {
            let mut dirty = HashSet::default();
            dirty.insert(11);

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_build_dirty_instance_keys(&dirty, || {
                    assert!(is_instance_key_build_dirty(11));
                    panic!("expected panic");
                });
            }));
            assert!(result.is_err());
            assert!(!is_instance_key_build_dirty(11));
        });
    }

    #[test]
    fn with_replay_scope_restores_group_path_and_override() {
        with_tessera(|| {
            with_execution_context_mut(|context| {
                context.group_path_stack = vec![1, 2, 3];
            });
            with_execution_context_mut(|context| {
                context.instance_key_stack = vec![5];
            });
            with_execution_context_mut(|context| {
                context.next_node_instance_logic_id_override = Some(9);
            });

            with_replay_scope(42, &[7, 8], Some(11), || {
                assert_eq!(current_group_path(), vec![7, 8]);
                assert_eq!(current_instance_key_override(), Some(11));
                assert_eq!(take_next_node_instance_logic_id_override(), Some(42));
                assert_eq!(take_next_node_instance_logic_id_override(), None);
            });

            assert_eq!(current_group_path(), vec![1, 2, 3]);
            assert_eq!(current_instance_key_override(), Some(5));
            let restored_override =
                with_execution_context(|context| context.next_node_instance_logic_id_override);
            assert_eq!(restored_override, Some(9));
        });
    }

    #[test]
    fn with_replay_scope_restores_on_panic() {
        with_tessera(|| {
            with_execution_context_mut(|context| {
                context.group_path_stack = vec![5];
            });
            with_execution_context_mut(|context| {
                context.instance_key_stack = vec![13];
            });
            with_execution_context_mut(|context| {
                context.next_node_instance_logic_id_override = None;
            });

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_replay_scope(77, &[10], Some(17), || {
                    assert_eq!(current_group_path(), vec![10]);
                    assert_eq!(current_instance_key_override(), Some(17));
                    panic!("expected panic");
                });
            }));
            assert!(result.is_err());

            assert_eq!(current_group_path(), vec![5]);
            assert_eq!(current_instance_key_override(), Some(13));
            let restored_override =
                with_execution_context(|context| context.next_node_instance_logic_id_override);
            assert_eq!(restored_override, None);
        });
    }

    #[test]
    fn group_local_remember_does_not_shift_following_slots() {
        with_tessera(|| {
            reset_slots();

            begin_recompose_slot_epoch();
            with_test_component_scope(1001, || {
                let stable_state = remember(|| 1usize);
                stable_state.set(41);
            });

            begin_recompose_slot_epoch();
            with_test_component_scope(1001, || {
                {
                    let _group_guard = GroupGuard::new(7);
                    let _branch_state = remember(|| 10usize);
                }
                let stable_state = remember(|| 1usize);
                assert_eq!(stable_state.get(), 41);
            });
        });
    }

    #[test]
    fn conditional_frame_receiver_does_not_shift_following_remember_slots() {
        with_tessera(|| {
            reset_slots();
            reset_frame_clock();
            begin_frame_clock(Instant::now());

            begin_recompose_slot_epoch();
            with_test_component_scope(1002, || {
                let stable_state = remember(|| 1usize);
                stable_state.set(99);
            });

            begin_recompose_slot_epoch();
            with_test_component_scope(1002, || {
                {
                    let _group_guard = GroupGuard::new(9);
                    receive_frame_nanos(|_| FrameNanosControl::Stop);
                }
                let stable_state = remember(|| 1usize);
                assert_eq!(stable_state.get(), 99);
            });
        });
    }

    #[test]
    fn callback_handle_stays_stable_and_invokes_latest_closure() {
        with_tessera(|| {
            reset_slots();

            let calls = Arc::new(AtomicUsize::new(0));

            begin_recompose_slot_epoch();
            let first = with_test_component_scope(11001, || {
                let calls = Arc::clone(&calls);
                Callback::new(move || {
                    calls.store(1, Ordering::SeqCst);
                })
            });

            begin_recompose_slot_epoch();
            let second = with_test_component_scope(11001, || {
                let calls = Arc::clone(&calls);
                Callback::new(move || {
                    calls.store(2, Ordering::SeqCst);
                })
            });

            assert!(first == second);
            first.call();
            assert_eq!(calls.load(Ordering::SeqCst), 2);
        });
    }

    #[test]
    fn render_slot_update_invalidates_reader_instance() {
        with_tessera(|| {
            reset_slots();
            reset_render_slot_read_dependencies();
            reset_build_invalidations();

            begin_recompose_slot_epoch();
            let first = with_test_component_scope(11002, || RenderSlot::new(|| {}));

            let reader_instance_key = with_test_component_scope(11003, || {
                let instance_key = current_component_instance_key_from_scope()
                    .expect("reader must have instance key");
                first.render();
                instance_key
            });

            assert!(!has_pending_build_invalidations());

            begin_recompose_slot_epoch();
            let second = with_test_component_scope(11002, || RenderSlot::new(|| {}));

            assert!(first == second);
            assert!(has_pending_build_invalidations());

            let invalidations = take_build_invalidations();
            let mut expected = HashSet::default();
            expected.insert(reader_instance_key);
            assert_eq!(invalidations.dirty_instance_keys, expected);
        });
    }

    #[test]
    fn group_local_child_identity_does_not_shift_following_siblings() {
        with_tessera(|| {
            fn stable_child_instance_logic_id(with_group_child: bool) -> u64 {
                let mut arena = crate::Arena::<()>::new();
                let root_node = arena.new_node(());
                let stable_child_node = arena.new_node(());
                let group_child_node = arena.new_node(());

                let _phase_guard = push_phase(RuntimePhase::Build);
                let _root_guard = push_current_node(root_node, 2001, "root_component");
                let _root_instance_guard =
                    push_current_component_instance_key(current_instance_key());

                if with_group_child {
                    let _group_guard = GroupGuard::new(5);
                    let _group_child_guard =
                        push_current_node(group_child_node, 2002, "group_child_component");
                    let _group_child_instance_guard =
                        push_current_component_instance_key(current_instance_key());
                    let _ = current_instance_logic_id();
                }

                let _stable_child_guard =
                    push_current_node(stable_child_node, 2003, "stable_child_component");
                current_instance_logic_id()
            }

            assert_eq!(
                stable_child_instance_logic_id(false),
                stable_child_instance_logic_id(true)
            );
        });
    }

    #[test]
    fn child_components_in_different_groups_get_distinct_instance_logic_ids() {
        with_tessera(|| {
            let mut arena = crate::Arena::<()>::new();
            let root_node = arena.new_node(());
            let first_child_node = arena.new_node(());
            let second_child_node = arena.new_node(());

            let _phase_guard = push_phase(RuntimePhase::Build);
            let _root_guard = push_current_node(root_node, 3001, "root_component");
            let _root_instance_guard = push_current_component_instance_key(current_instance_key());

            let first_id = {
                let _group_guard = GroupGuard::new(11);
                let _child_guard = push_current_node(first_child_node, 3002, "grouped_child");
                current_instance_logic_id()
            };

            let second_id = {
                let _group_guard = GroupGuard::new(12);
                let _child_guard = push_current_node(second_child_node, 3002, "grouped_child");
                current_instance_logic_id()
            };

            assert_ne!(first_id, second_id);
        });
    }

    #[test]
    fn child_components_in_repeated_path_groups_keep_distinct_instance_logic_ids() {
        with_tessera(|| {
            let mut arena = crate::Arena::<()>::new();
            let root_node = arena.new_node(());
            let first_child_node = arena.new_node(());
            let second_child_node = arena.new_node(());

            let _phase_guard = push_phase(RuntimePhase::Build);
            let _root_guard = push_current_node(root_node, 4001, "root_component");
            let _root_instance_guard = push_current_component_instance_key(current_instance_key());

            let first_id = {
                let _group_guard = PathGroupGuard::new(21);
                let _child_guard = push_current_node(first_child_node, 4002, "loop_child");
                current_instance_logic_id()
            };

            let second_id = {
                let _group_guard = PathGroupGuard::new(21);
                let _child_guard = push_current_node(second_child_node, 4002, "loop_child");
                current_instance_logic_id()
            };

            assert_ne!(first_id, second_id);
        });
    }

    #[test]
    fn drop_slots_for_instance_logic_ids_keeps_retained_entries() {
        with_tessera(|| {
            let mut table = SlotTable::default();
            let keep_key = SlotKey {
                instance_logic_id: 7,
                slot_hash: 11,
                type_id: TypeId::of::<i32>(),
            };
            let drop_key = SlotKey {
                instance_logic_id: 7,
                slot_hash: 12,
                type_id: TypeId::of::<i32>(),
            };

            let keep_slot = table.entries.insert(SlotEntry {
                key: keep_key,
                generation: 1,
                value: Some(Arc::new(RwLock::new(10_i32))),
                last_alive_epoch: 0,
                retained: true,
            });
            let drop_slot = table.entries.insert(SlotEntry {
                key: drop_key,
                generation: 1,
                value: Some(Arc::new(RwLock::new(20_i32))),
                last_alive_epoch: 0,
                retained: false,
            });
            table.key_to_slot.insert(keep_key, keep_slot);
            table.key_to_slot.insert(drop_key, drop_slot);
            *slot_table().write() = table;

            let mut stale = HashSet::default();
            stale.insert(7_u64);
            drop_slots_for_instance_logic_ids(&stale);

            let slot_table = slot_table();
            let table = slot_table.read();
            assert!(table.entries.get(keep_slot).is_some());
            assert!(table.key_to_slot.contains_key(&keep_key));
            assert!(table.entries.get(drop_slot).is_none());
            assert!(!table.key_to_slot.contains_key(&drop_key));
        });
    }
}
