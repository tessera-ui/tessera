//! Component tree build helpers for recomposition and replay.
//!
//! ## Usage
//!
//! Rebuild or partially replay the component tree before layout and rendering.

use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use rustc_hash::FxHashSet as HashSet;
use tessera_macros::tessera;
use tracing::{debug, instrument};

use crate::{
    component_tree::ReplayReplaceError,
    context::{
        begin_frame_component_context_tracking, begin_recompose_context_slot_epoch,
        drop_context_slots_for_instance_logic_ids, finalize_frame_component_context_tracking,
        finalize_frame_component_context_tracking_partial, previous_component_context_snapshots,
        remove_context_read_dependencies, remove_previous_component_context_snapshots,
        reset_context_read_dependencies, with_context_snapshot,
    },
    runtime::{
        TesseraRuntime, begin_frame_component_replay_tracking, begin_frame_layout_dirty_tracking,
        begin_recompose_slot_epoch, clear_frame_nanos_receivers, drop_slots_for_instance_logic_ids,
        finalize_frame_component_replay_tracking, finalize_frame_component_replay_tracking_partial,
        finalize_frame_layout_dirty_tracking, previous_component_replay_nodes,
        remove_focus_read_dependencies, remove_frame_nanos_receivers,
        remove_previous_component_replay_nodes, remove_render_slot_read_dependencies,
        remove_state_read_dependencies, reset_focus_read_dependencies,
        reset_render_slot_read_dependencies, reset_state_read_dependencies,
        take_build_invalidations, with_build_dirty_instance_keys, with_replay_scope,
    },
};

#[cfg(feature = "profiling")]
use crate::context::live_context_slot_instance_logic_ids;
#[cfg(feature = "profiling")]
use crate::profiler::BuildMode;
#[cfg(feature = "profiling")]
use crate::runtime::live_slot_instance_logic_ids;

#[cfg(not(feature = "profiling"))]
use crate::context::live_context_slot_instance_logic_ids;
#[cfg(not(feature = "profiling"))]
use crate::runtime::live_slot_instance_logic_ids;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuildTreeMode {
    RootRecompose,
    PartialReplay,
    SkipNoInvalidation,
}

#[derive(Clone, Debug)]
pub(crate) struct BuildTreeResult {
    duration: Duration,
    mode: BuildTreeMode,
    #[cfg(feature = "profiling")]
    partial_replay_nodes: Option<u64>,
    #[cfg(feature = "profiling")]
    total_nodes_before_build: Option<u64>,
    #[cfg(feature = "debug-dirty-overlay")]
    had_invalidations: bool,
    #[cfg(feature = "debug-dirty-overlay")]
    dirty_replay_roots: Vec<u64>,
}

impl BuildTreeResult {
    fn root_recompose(duration: Duration) -> Self {
        Self {
            duration,
            mode: BuildTreeMode::RootRecompose,
            #[cfg(feature = "profiling")]
            partial_replay_nodes: None,
            #[cfg(feature = "profiling")]
            total_nodes_before_build: None,
            #[cfg(feature = "debug-dirty-overlay")]
            had_invalidations: false,
            #[cfg(feature = "debug-dirty-overlay")]
            dirty_replay_roots: Vec::new(),
        }
    }

    #[cfg(not(feature = "profiling"))]
    fn partial_replay(duration: Duration) -> Self {
        Self {
            duration,
            mode: BuildTreeMode::PartialReplay,
            #[cfg(feature = "debug-dirty-overlay")]
            had_invalidations: false,
            #[cfg(feature = "debug-dirty-overlay")]
            dirty_replay_roots: Vec::new(),
        }
    }

    #[cfg(feature = "profiling")]
    fn partial_replay(
        duration: Duration,
        partial_replay_nodes: u64,
        total_nodes_before_build: u64,
    ) -> Self {
        Self {
            duration,
            mode: BuildTreeMode::PartialReplay,
            partial_replay_nodes: Some(partial_replay_nodes),
            total_nodes_before_build: Some(total_nodes_before_build),
            #[cfg(feature = "debug-dirty-overlay")]
            had_invalidations: false,
            #[cfg(feature = "debug-dirty-overlay")]
            dirty_replay_roots: Vec::new(),
        }
    }

    fn skip_no_invalidation() -> Self {
        Self {
            duration: Duration::ZERO,
            mode: BuildTreeMode::SkipNoInvalidation,
            #[cfg(feature = "profiling")]
            partial_replay_nodes: None,
            #[cfg(feature = "profiling")]
            total_nodes_before_build: None,
            #[cfg(feature = "debug-dirty-overlay")]
            had_invalidations: false,
            #[cfg(feature = "debug-dirty-overlay")]
            dirty_replay_roots: Vec::new(),
        }
    }

    #[cfg(feature = "debug-dirty-overlay")]
    fn with_dirty_replay_info(
        mut self,
        had_invalidations: bool,
        dirty_replay_roots: Vec<u64>,
    ) -> Self {
        self.had_invalidations = had_invalidations;
        self.dirty_replay_roots = dirty_replay_roots;
        self
    }

    pub(crate) fn mode(&self) -> BuildTreeMode {
        self.mode
    }

    pub(crate) fn duration(&self) -> Duration {
        self.duration
    }

    pub(crate) fn absorb_retry(&mut self, retry: Self) {
        self.duration += retry.duration;
        self.mode = retry.mode;
        #[cfg(feature = "profiling")]
        {
            self.partial_replay_nodes = retry.partial_replay_nodes;
            self.total_nodes_before_build = retry.total_nodes_before_build;
        }
        #[cfg(feature = "debug-dirty-overlay")]
        {
            self.had_invalidations = retry.had_invalidations;
            self.dirty_replay_roots = retry.dirty_replay_roots;
        }
    }

    #[cfg(feature = "debug-dirty-overlay")]
    pub(crate) fn had_invalidations(&self) -> bool {
        self.had_invalidations
    }

    #[cfg(feature = "debug-dirty-overlay")]
    pub(crate) fn dirty_replay_roots(&self) -> &[u64] {
        &self.dirty_replay_roots
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn partial_replay_nodes(&self) -> Option<u64> {
        self.partial_replay_nodes
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn total_nodes_before_build(&self) -> Option<u64> {
        self.total_nodes_before_build
    }
}

#[cfg(feature = "profiling")]
impl BuildTreeResult {
    pub(crate) fn profiler_build_mode(&self) -> BuildMode {
        match self.mode {
            BuildTreeMode::RootRecompose => BuildMode::RootRecompose,
            BuildTreeMode::PartialReplay => BuildMode::PartialReplay,
            BuildTreeMode::SkipNoInvalidation => BuildMode::SkipNoInvalidation,
        }
    }
}

fn collect_dirty_replay_roots(dirty_instance_keys: &HashSet<u64>) -> Vec<u64> {
    TesseraRuntime::with(|runtime| {
        let mut roots = Vec::new();
        let tree = runtime.component_tree.tree();
        for instance_key in dirty_instance_keys {
            let node_id = runtime
                .component_tree
                .find_node_id_by_instance_key(*instance_key)
                .unwrap_or_else(|| {
                    panic!(
                        "missing node for dirty instance key {instance_key}; this violates replay invariants"
                    )
                });

            let mut has_dirty_ancestor = false;
            let mut parent_id = tree.get(node_id).and_then(|node| node.parent());
            while let Some(pid) = parent_id {
                let Some(parent_node) = tree.get(pid) else {
                    break;
                };
                if dirty_instance_keys.contains(&parent_node.get().instance_key) {
                    has_dirty_ancestor = true;
                    break;
                }
                parent_id = parent_node.parent();
            }

            if !has_dirty_ancestor {
                roots.push(*instance_key);
            }
        }
        roots
    })
}

fn dirty_roots_include_tree_root(dirty_roots: &[u64]) -> bool {
    TesseraRuntime::with(|runtime| {
        let tree = runtime.component_tree.tree();
        dirty_roots.iter().any(|instance_key| {
            let node_id = runtime
                .component_tree
                .find_node_id_by_instance_key(*instance_key)
                .unwrap_or_else(|| {
                    panic!(
                        "missing node for dirty instance key {instance_key}; this violates replay invariants"
                    )
                });
            tree.get(node_id)
                .and_then(|node| node.parent())
                .is_none()
        })
    })
}

#[cfg(feature = "profiling")]
fn subtree_node_count_by_instance_key(instance_key: u64) -> u64 {
    TesseraRuntime::with(|runtime| {
        let node_id = runtime
            .component_tree
            .find_node_id_by_instance_key(instance_key)
            .unwrap_or_else(|| {
                panic!(
                    "missing node for dirty instance key {instance_key}; this violates replay invariants"
                )
            });
        let tree = runtime.component_tree.tree();
        let mut stack = vec![node_id];
        let mut count = 0_u64;
        while let Some(current) = stack.pop() {
            count = count.saturating_add(1);
            stack.extend(current.children(tree));
        }
        count
    })
}

#[instrument(level = "debug", skip(entry_point))]
pub(crate) fn build_component_tree<F: Fn()>(entry_point: &F) -> BuildTreeResult {
    with_entry_point_callback(entry_point, || {
        let run_root_recompose = || {
            let recomposed_state_instance_logic_ids = live_slot_instance_logic_ids();
            let recomposed_context_instance_logic_ids = live_context_slot_instance_logic_ids();
            let tree_timer = Instant::now();
            debug!("Building component tree...");
            clear_frame_nanos_receivers();
            reset_focus_read_dependencies();
            reset_render_slot_read_dependencies();
            reset_state_read_dependencies();
            reset_context_read_dependencies();
            TesseraRuntime::with_mut(|runtime| runtime.component_tree.clear());
            begin_frame_component_replay_tracking();
            begin_frame_component_context_tracking();
            begin_frame_layout_dirty_tracking();
            begin_recompose_slot_epoch();
            begin_recompose_context_slot_epoch();
            let _phase_guard = crate::runtime::push_phase(crate::runtime::RuntimePhase::Build);
            entry_wrapper();
            finalize_frame_component_replay_tracking();
            finalize_frame_component_context_tracking();
            finalize_frame_layout_dirty_tracking();
            crate::runtime::recycle_recomposed_slots_for_instance_logic_ids(
                &recomposed_state_instance_logic_ids,
            );
            crate::context::recycle_recomposed_context_slots_for_instance_logic_ids(
                &recomposed_context_instance_logic_ids,
            );
            let build_tree_cost = tree_timer.elapsed();
            debug!("Component tree built in {build_tree_cost:?}");
            BuildTreeResult::root_recompose(build_tree_cost)
        };

        let tree_is_empty = TesseraRuntime::with(|rt| rt.component_tree.tree().count() == 0);
        let invalidations = take_build_invalidations();
        #[cfg(feature = "debug-dirty-overlay")]
        let had_invalidations = !invalidations.dirty_instance_keys.is_empty();
        with_build_dirty_instance_keys(&invalidations.dirty_instance_keys, || {
            if tree_is_empty {
                let result = run_root_recompose();
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
                return result;
            }

            if invalidations.dirty_instance_keys.is_empty() {
                debug!("Skipping component tree build: no invalidations");
                let result = BuildTreeResult::skip_no_invalidation();
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(false, Vec::new());
                return result;
            }

            let dirty_roots = collect_dirty_replay_roots(&invalidations.dirty_instance_keys);
            if dirty_roots_include_tree_root(&dirty_roots) {
                let result = run_root_recompose();
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
                return result;
            }
            if dirty_roots.is_empty() {
                debug!("Skipping component tree build: no dirty replay roots");
                let result = BuildTreeResult::skip_no_invalidation();
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
                return result;
            }

            let replay_snapshots = previous_component_replay_nodes();
            let context_snapshots = previous_component_context_snapshots();

            let tree_timer = Instant::now();
            debug!("Building dirty subtrees with replay...");
            begin_frame_component_replay_tracking();
            begin_frame_component_context_tracking();
            begin_frame_layout_dirty_tracking();
            begin_recompose_slot_epoch();
            begin_recompose_context_slot_epoch();
            let _phase_guard = crate::runtime::push_phase(crate::runtime::RuntimePhase::Build);
            let mut stale_instance_keys = HashSet::default();
            let mut stale_instance_logic_ids = HashSet::default();
            let mut recomposed_instance_logic_ids = HashSet::default();
            #[cfg(feature = "profiling")]
            let mut replayed_nodes = 0_u64;
            #[cfg(feature = "profiling")]
            let total_nodes_before_build =
                TesseraRuntime::with(|runtime| runtime.component_tree.tree().count() as u64);

            for instance_key in &dirty_roots {
                #[cfg(feature = "profiling")]
                {
                    replayed_nodes = replayed_nodes
                        .saturating_add(subtree_node_count_by_instance_key(*instance_key));
                }
                let replay_snapshot = replay_snapshots.get(instance_key).unwrap_or_else(|| {
                    panic!(
                        "missing replay snapshot for dirty instance key {instance_key}; this violates replay invariants"
                    )
                });
                let context_snapshot = context_snapshots.get(instance_key).unwrap_or_else(|| {
                    panic!(
                        "missing context snapshot for dirty instance key {instance_key}; this violates replay invariants"
                    )
                });
                let replay = replay_snapshot.replay.clone();
                let replay_instance_logic_id = replay_snapshot.instance_logic_id;
                let replay_group_path = replay_snapshot.group_path.clone();
                let replay_instance_key_override = replay_snapshot.instance_key_override;

                let replace_context = TesseraRuntime::with_mut(|runtime| {
                    runtime
                        .component_tree
                        .begin_replace_subtree_by_instance_key(*instance_key)
                });
                let replace_context = match replace_context {
                    Ok(context) => context,
                    Err(ReplayReplaceError::RootNodeNotReplaceable) => panic!(
                        "dirty root {} resolved to component tree root; root recomposition must be selected before partial replay",
                        instance_key
                    ),
                    Err(_) => {
                        panic!(
                            "begin_replace_subtree_by_instance_key failed for instance key {instance_key}"
                        )
                    }
                };

                with_context_snapshot(context_snapshot, || {
                    with_replay_scope(
                        replay_instance_logic_id,
                        &replay_group_path,
                        replay_instance_key_override,
                        || {
                            replay.runner.run(replay.props.as_ref());
                        },
                    );
                });

                let replace_result = TesseraRuntime::with_mut(|runtime| {
                    runtime
                        .component_tree
                        .finish_replace_subtree(replace_context)
                });
                let replace_result = replace_result.unwrap_or_else(|_| {
                    panic!("finish_replace_subtree failed for instance key {instance_key}")
                });
                recomposed_instance_logic_ids.extend(
                    replace_result
                        .inserted_instance_logic_ids
                        .difference(&replace_result.reused_instance_logic_ids)
                        .copied(),
                );

                for removed in &replace_result.removed_instance_keys {
                    if !replace_result.inserted_instance_keys.contains(removed) {
                        stale_instance_keys.insert(*removed);
                    }
                }
                for removed in &replace_result.removed_instance_logic_ids {
                    if !replace_result.inserted_instance_logic_ids.contains(removed) {
                        stale_instance_logic_ids.insert(*removed);
                    }
                }
            }
            finalize_frame_component_replay_tracking_partial();
            finalize_frame_component_context_tracking_partial();
            finalize_frame_layout_dirty_tracking();
            remove_previous_component_replay_nodes(&stale_instance_keys);
            remove_frame_nanos_receivers(&stale_instance_keys);
            remove_focus_read_dependencies(&stale_instance_keys);
            remove_render_slot_read_dependencies(&stale_instance_keys);
            remove_state_read_dependencies(&stale_instance_keys);
            crate::runtime::remove_build_invalidations(&stale_instance_keys);
            remove_previous_component_context_snapshots(&stale_instance_keys);
            remove_context_read_dependencies(&stale_instance_keys);
            drop_slots_for_instance_logic_ids(&stale_instance_logic_ids);
            drop_context_slots_for_instance_logic_ids(&stale_instance_logic_ids);
            crate::runtime::recycle_recomposed_slots_for_instance_logic_ids(
                &recomposed_instance_logic_ids,
            );
            crate::context::recycle_recomposed_context_slots_for_instance_logic_ids(
                &recomposed_instance_logic_ids,
            );
            let build_tree_cost = tree_timer.elapsed();
            debug!("Dirty subtree replay finished in {build_tree_cost:?}");
            #[cfg(feature = "profiling")]
            {
                let result = BuildTreeResult::partial_replay(
                    build_tree_cost,
                    replayed_nodes,
                    total_nodes_before_build,
                );
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, dirty_roots.clone());
                result
            }
            #[cfg(not(feature = "profiling"))]
            {
                let result = BuildTreeResult::partial_replay(build_tree_cost);
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, dirty_roots.clone());
                result
            }
        })
    })
}

type EntryPointInvoker = fn(*const ());

#[derive(Clone, Copy)]
struct EntryPointCallback {
    data: *const (),
    invoker: EntryPointInvoker,
}

thread_local! {
    static ENTRY_POINT_CALLBACK: RefCell<Option<EntryPointCallback>> = const { RefCell::new(None) };
}

fn invoke_entry_point<F: Fn()>(data: *const ()) {
    // SAFETY: The pointer is installed by `with_entry_point_callback` and is
    // valid for the guarded call scope.
    let entry_point = unsafe { &*(data as *const F) };
    entry_point();
}

struct EntryPointCallbackGuard;

impl Drop for EntryPointCallbackGuard {
    fn drop(&mut self) {
        ENTRY_POINT_CALLBACK.with(|slot| {
            *slot.borrow_mut() = None;
        });
    }
}

fn with_entry_point_callback<F: Fn(), R>(entry_point: &F, run: impl FnOnce() -> R) -> R {
    ENTRY_POINT_CALLBACK.with(|slot| {
        *slot.borrow_mut() = Some(EntryPointCallback {
            data: entry_point as *const F as *const (),
            invoker: invoke_entry_point::<F>,
        });
    });
    let _guard = EntryPointCallbackGuard;
    run()
}

fn run_entry_point_callback() {
    let callback = ENTRY_POINT_CALLBACK.with(|slot| *slot.borrow());
    let Some(callback) = callback else {
        panic!("entry point callback is not set");
    };
    (callback.invoker)(callback.data);
}

#[tessera(crate)]
fn entry_wrapper() {
    run_entry_point_callback();
}
