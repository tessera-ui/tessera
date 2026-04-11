//! Component tree build helpers for recomposition and replay.
//!
//! ## Usage
//!
//! Rebuild or partially replay the component tree before layout and rendering.

use std::{cell::RefCell, fmt::Write as _, time::Duration};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
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
    time::Instant,
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

fn missing_replay_snapshot_panic_message(
    instance_key: u64,
    replay_snapshots: &HashMap<u64, crate::runtime::ReplayNodeSnapshot>,
    context_snapshots: &HashMap<u64, crate::context::ContextMap>,
) -> String {
    let mut message = String::new();
    let _ = writeln!(
        message,
        "missing replay snapshot for dirty instance key {instance_key}; this violates replay invariants"
    );
    let _ = writeln!(
        message,
        "previous replay snapshot present: {}",
        replay_snapshots.contains_key(&instance_key)
    );
    let _ = writeln!(
        message,
        "previous context snapshot present: {}",
        context_snapshots.contains_key(&instance_key)
    );

    TesseraRuntime::with(|runtime| {
        let tree = runtime.component_tree.tree();
        let Some(node_id) = runtime
            .component_tree
            .find_node_id_by_instance_key(instance_key)
        else {
            let _ = writeln!(message, "current tree node: <missing>");
            return;
        };

        let Some(node_ref) = tree.get(node_id) else {
            let _ = writeln!(message, "current tree node: <removed>");
            return;
        };

        let node = node_ref.get();
        let _ = writeln!(
            message,
            "current node: fn={}, role={:?}, instance_logic_id={}, has_replay={}, props_unchanged_from_previous={}",
            node.fn_name,
            node.role,
            node.instance_logic_id,
            node.replay.is_some(),
            node.props_unchanged_from_previous,
        );

        let mut nearest_replayable_ancestor = None;
        let mut ancestry = Vec::new();
        let mut cursor = Some(node_id);
        while let Some(current_id) = cursor {
            let Some(current_ref) = tree.get(current_id) else {
                break;
            };
            let current = current_ref.get();
            if nearest_replayable_ancestor.is_none() && current.replay.is_some() {
                nearest_replayable_ancestor = Some((
                    current.instance_key,
                    current.fn_name.clone(),
                    replay_snapshots.contains_key(&current.instance_key),
                    context_snapshots.contains_key(&current.instance_key),
                ));
            }

            ancestry.push(format!(
                "{}[key={}, logic={}, role={:?}, replay={}, prev_replay_snapshot={}, prev_context_snapshot={}]",
                current.fn_name,
                current.instance_key,
                current.instance_logic_id,
                current.role,
                current.replay.is_some(),
                replay_snapshots.contains_key(&current.instance_key),
                context_snapshots.contains_key(&current.instance_key),
            ));

            cursor = current_ref.parent();
        }

        if let Some((ancestor_key, ancestor_name, has_replay_snapshot, has_context_snapshot)) =
            nearest_replayable_ancestor
        {
            let _ = writeln!(
                message,
                "nearest replayable ancestor: {}[key={}, prev_replay_snapshot={}, prev_context_snapshot={}]",
                ancestor_name, ancestor_key, has_replay_snapshot, has_context_snapshot,
            );
        } else {
            let _ = writeln!(message, "nearest replayable ancestor: <none>");
        }

        let _ = writeln!(message, "ancestry (self -> root):");
        for entry in ancestry {
            let _ = writeln!(message, "  - {entry}");
        }
    });

    message
}

fn retain_live_dirty_instance_keys(dirty_instance_keys: &HashSet<u64>) -> HashSet<u64> {
    TesseraRuntime::with(|runtime| {
        dirty_instance_keys
            .iter()
            .copied()
            .filter(|instance_key| {
                runtime
                    .component_tree
                    .find_node_id_by_instance_key(*instance_key)
                    .is_some()
            })
            .collect()
    })
}

fn expand_dirty_instance_keys_for_reuse(dirty_instance_keys: &HashSet<u64>) -> HashSet<u64> {
    TesseraRuntime::with(|runtime| {
        let mut expanded = dirty_instance_keys.clone();
        let tree = runtime.component_tree.tree();

        for instance_key in dirty_instance_keys {
            let Some(node_id) = runtime
                .component_tree
                .find_node_id_by_instance_key(*instance_key)
            else {
                continue;
            };

            let mut parent_id = tree.get(node_id).and_then(|node| node.parent());
            while let Some(pid) = parent_id {
                let Some(parent_node) = tree.get(pid) else {
                    break;
                };
                expanded.insert(parent_node.get().instance_key);
                parent_id = parent_node.parent();
            }
        }

        expanded
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

            let initial_live_dirty_instance_keys =
                retain_live_dirty_instance_keys(&invalidations.dirty_instance_keys);
            let initial_dirty_roots = collect_dirty_replay_roots(&initial_live_dirty_instance_keys);
            if dirty_roots_include_tree_root(&initial_dirty_roots) {
                let result = run_root_recompose();
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
                return result;
            }
            if initial_dirty_roots.is_empty() {
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
            let mut pending_dirty_instance_keys = invalidations.dirty_instance_keys.clone();
            let mut replay_roots_for_debug = Vec::new();
            let mut fallback_to_root_recompose = false;
            #[cfg(feature = "profiling")]
            let mut replayed_nodes = 0_u64;
            #[cfg(feature = "profiling")]
            let total_nodes_before_build =
                TesseraRuntime::with(|runtime| runtime.component_tree.tree().count() as u64);

            while !pending_dirty_instance_keys.is_empty() {
                let live_dirty_instance_keys =
                    retain_live_dirty_instance_keys(&pending_dirty_instance_keys);
                if live_dirty_instance_keys.is_empty() {
                    break;
                }

                let dirty_roots = collect_dirty_replay_roots(&live_dirty_instance_keys);
                if dirty_roots.is_empty() {
                    break;
                }
                if dirty_roots_include_tree_root(&dirty_roots) {
                    fallback_to_root_recompose = true;
                    break;
                }
                if let Some(instance_key) = dirty_roots.iter().copied().find(|instance_key| {
                    !replay_snapshots.contains_key(instance_key)
                        || !context_snapshots.contains_key(instance_key)
                }) {
                    panic!(
                        "{}",
                        missing_replay_snapshot_panic_message(
                            instance_key,
                            &replay_snapshots,
                            &context_snapshots,
                        )
                    );
                }

                replay_roots_for_debug.extend(dirty_roots.iter().copied());

                let reuse_guard_dirty_instance_keys =
                    expand_dirty_instance_keys_for_reuse(&live_dirty_instance_keys);
                let mut round_covered_instance_keys = HashSet::default();

                with_build_dirty_instance_keys(&reuse_guard_dirty_instance_keys, || {
                    for instance_key in &dirty_roots {
                        #[cfg(feature = "profiling")]
                        {
                            replayed_nodes = replayed_nodes
                                .saturating_add(subtree_node_count_by_instance_key(*instance_key));
                        }
                        let replay_snapshot =
                            replay_snapshots.get(instance_key).unwrap_or_else(|| {
                                panic!(
                                    "{}",
                                    missing_replay_snapshot_panic_message(
                                        *instance_key,
                                        &replay_snapshots,
                                        &context_snapshots,
                                    )
                                )
                            });
                        let context_snapshot =
                            context_snapshots.get(instance_key).unwrap_or_else(|| {
                                panic!(
                                    "{}",
                                    missing_replay_snapshot_panic_message(
                                        *instance_key,
                                        &replay_snapshots,
                                        &context_snapshots,
                                    )
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
                        round_covered_instance_keys
                            .extend(replace_result.inserted_instance_keys.iter().copied());
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
                });

                let round_invalidations = take_build_invalidations();
                pending_dirty_instance_keys.extend(round_invalidations.dirty_instance_keys);
                pending_dirty_instance_keys.retain(|instance_key| {
                    !stale_instance_keys.contains(instance_key)
                        && !round_covered_instance_keys.contains(instance_key)
                });
            }

            if fallback_to_root_recompose {
                let result = run_root_recompose();
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
                return result;
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
                let result = result
                    .with_dirty_replay_info(had_invalidations, replay_roots_for_debug.clone());
                result
            }
            #[cfg(not(feature = "profiling"))]
            {
                let result = BuildTreeResult::partial_replay(build_tree_cost);
                #[cfg(feature = "debug-dirty-overlay")]
                let result = result
                    .with_dirty_replay_info(had_invalidations, replay_roots_for_debug.clone());
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
