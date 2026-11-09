//! Component tree build helpers for recomposition and replay.
//!
//! ## Usage
//!
//! Rebuild or partially replay the component tree before layout and rendering.

use std::{fmt::Write as _, sync::Arc, time::Duration};

use rustc_hash::FxHashSet as HashSet;
use tracing::{debug, instrument};

use crate::{
    component_tree::ReplayReplaceResult,
    context::{
        begin_recompose_context_slot_epoch, drop_context_slots_for_instance_logic_ids,
        normalize_context_snapshot, remove_context_read_dependencies,
        reset_context_read_dependencies,
    },
    runtime::{
        TesseraRuntime, begin_frame_component_replay_tracking, begin_frame_layout_dirty_tracking,
        begin_recompose_slot_epoch, clear_frame_nanos_receivers, drop_slots_for_instance_logic_ids,
        finalize_frame_layout_dirty_tracking, previous_component_replay_node,
        release_retired_replay_snapshots, remove_focus_read_dependencies,
        remove_frame_nanos_receivers, remove_layout_state_read_dependencies,
        remove_render_slot_read_dependencies, remove_state_read_dependencies,
        reset_focus_read_dependencies, reset_render_slot_read_dependencies,
        reset_state_read_dependencies, take_build_invalidations, with_build_dirty_instance_keys,
        with_replay_scope,
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

struct ReplayPlan {
    live: Arc<HashSet<u64>>,
    roots: Vec<u64>,
}

fn plan_dirty_replay(dirty_instance_keys: &Arc<HashSet<u64>>) -> ReplayPlan {
    TesseraRuntime::with(|runtime| {
        let component_tree = &runtime.component_tree;
        let mut all_live = true;
        let mut roots = Vec::new();
        for &key in dirty_instance_keys.iter() {
            if !component_tree.has_call(key) {
                all_live = false;
                continue;
            }
            if !component_tree
                .call_ancestors(key)
                .any(|parent| dirty_instance_keys.contains(&parent))
            {
                roots.push(key);
            }
        }
        let live = if all_live {
            Arc::clone(dirty_instance_keys)
        } else {
            Arc::new(
                dirty_instance_keys
                    .iter()
                    .copied()
                    .filter(|&key| component_tree.has_call(key))
                    .collect(),
            )
        };
        ReplayPlan { live, roots }
    })
}

fn missing_replay_snapshot_panic_message(instance_key: u64) -> String {
    let mut message = String::new();
    let _ = writeln!(
        message,
        "missing replay snapshot for dirty instance key {instance_key}; this violates replay invariants"
    );

    TesseraRuntime::with(|runtime| {
        let tree = &runtime.component_tree;
        let _ = writeln!(message, "call ancestry (self -> root):");
        let ancestry = tree
            .has_call(instance_key)
            .then_some(instance_key)
            .into_iter()
            .chain(tree.call_ancestors(instance_key));
        for key in ancestry {
            let _ = writeln!(message, "  - {}[key={key}]", tree.call_name(key));
        }
    });

    message
}

struct BuildRootNodeGuard;

impl Drop for BuildRootNodeGuard {
    fn drop(&mut self) {
        TesseraRuntime::with_mut(|runtime| runtime.component_tree.pop_node());
    }
}

fn push_build_root_node() -> BuildRootNodeGuard {
    TesseraRuntime::with_mut(|runtime| runtime.component_tree.begin_build_root());
    BuildRootNodeGuard
}

#[instrument(level = "debug", skip(entry_point))]
pub(crate) fn build_component_tree<F: Fn() -> Output, Output>(entry_point: &F) -> BuildTreeResult {
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
        begin_frame_layout_dirty_tracking();
        begin_recompose_slot_epoch();
        begin_recompose_context_slot_epoch();
        let _phase_guard = crate::runtime::push_phase(crate::runtime::RuntimePhase::Build);
        let _root_guard = push_build_root_node();
        crate::component_call::with_pending_scope(|| {
            entry_point();
        });
        crate::nested_scroll::flush_attachment_changes();
        release_retired_replay_snapshots();
        finalize_frame_layout_dirty_tracking();
        // Build-time feedback must not become layout-settling work in this frame.
        let deferred = take_build_invalidations();
        crate::runtime::defer_build_invalidations(&deferred.dirty_instance_keys);
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

    let (tree_is_empty, density_changed) = TesseraRuntime::with_mut(|runtime| {
        let density_changed = runtime.component_tree.refresh_scale_factor();
        (runtime.component_tree.tree().is_empty(), density_changed)
    });
    let dirty_instance_keys = Arc::new(take_build_invalidations().dirty_instance_keys);
    #[cfg(feature = "debug-dirty-overlay")]
    let had_invalidations = !dirty_instance_keys.is_empty();
    with_build_dirty_instance_keys(Arc::clone(&dirty_instance_keys), || {
        // Density affects values resolved during build as well as cached
        // measurements.
        if tree_is_empty || density_changed {
            let result = run_root_recompose();
            #[cfg(feature = "debug-dirty-overlay")]
            let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
            return result;
        }

        if dirty_instance_keys.is_empty() {
            debug!("Skipping component tree build: no invalidations");
            let result = BuildTreeResult::skip_no_invalidation();
            #[cfg(feature = "debug-dirty-overlay")]
            let result = result.with_dirty_replay_info(false, Vec::new());
            return result;
        }

        let mut plan = plan_dirty_replay(&dirty_instance_keys);
        if plan.roots.is_empty() {
            debug!("Skipping component tree build: no dirty replay roots");
            let result = BuildTreeResult::skip_no_invalidation();
            #[cfg(feature = "debug-dirty-overlay")]
            let result = result.with_dirty_replay_info(had_invalidations, Vec::new());
            return result;
        }

        let tree_timer = Instant::now();
        debug!("Building dirty subtrees with replay...");
        begin_frame_component_replay_tracking();
        begin_frame_layout_dirty_tracking();
        begin_recompose_slot_epoch();
        begin_recompose_context_slot_epoch();
        let _phase_guard = crate::runtime::push_phase(crate::runtime::RuntimePhase::Build);
        let mut stale_instance_keys = HashSet::default();
        let mut stale_instance_logic_ids = HashSet::default();
        let mut recomposed_instance_logic_ids = HashSet::default();
        let mut pending_dirty_instance_keys = Arc::clone(&dirty_instance_keys);
        #[cfg(feature = "debug-dirty-overlay")]
        let mut replay_roots_for_debug = Vec::new();
        let mut deferred_backwards_instance_keys = HashSet::default();
        let mut executed_instance_keys = HashSet::default();
        let mut replace_result = ReplayReplaceResult::default();
        #[cfg(feature = "profiling")]
        let mut replayed_nodes = 0_u64;
        #[cfg(feature = "profiling")]
        let total_nodes_before_build =
            TesseraRuntime::with(|runtime| runtime.component_tree.tree().len() as u64);

        // Executed boundaries cannot replay again in this frame. Forward slot
        // dependencies may span more rounds than their nesting depth suggests;
        // only backwards writes need to be deferred to the next frame.
        loop {
            let ReplayPlan {
                live: live_dirty_instance_keys,
                roots: dirty_roots,
            } = plan;
            if tracing::enabled!(target: "tessera_ui::replay", tracing::Level::DEBUG) {
                let dirty = TesseraRuntime::with(|runtime| {
                    live_dirty_instance_keys
                        .iter()
                        .map(|&key| (key, runtime.component_tree.call_name(key)))
                        .collect::<Vec<_>>()
                });
                tracing::debug!(target: "tessera_ui::replay", ?dirty, "replay_dirty_boundaries");
            }
            if dirty_roots.is_empty() {
                break;
            }
            #[cfg(feature = "debug-dirty-overlay")]
            replay_roots_for_debug.extend(dirty_roots.iter().copied());

            with_build_dirty_instance_keys(live_dirty_instance_keys, || {
                for instance_key in dirty_roots {
                    let replay_snapshot = previous_component_replay_node(instance_key)
                        .unwrap_or_else(|| {
                            panic!("{}", missing_replay_snapshot_panic_message(instance_key))
                        });
                    #[cfg(feature = "profiling")]
                    {
                        replayed_nodes =
                            replayed_nodes.saturating_add(TesseraRuntime::with(|runtime| {
                                runtime.component_tree.call_subtree_len(instance_key)
                            }));
                    }
                    let replay_trace =
                        tracing::enabled!(target: "tessera_ui::replay", tracing::Level::DEBUG);
                    let trace_name = replay_trace.then(|| {
                        TesseraRuntime::with(|runtime| {
                            runtime.component_tree.call_name(instance_key)
                        })
                    });
                    let detach_timer = replay_trace.then(Instant::now);
                    let replace_context = TesseraRuntime::with_mut(|runtime| {
                        runtime.component_tree.begin_replace_call(instance_key)
                    });
                    let detach_ns = detach_timer.map(|timer| timer.elapsed().as_nanos());

                    let compose_timer = replay_trace.then(Instant::now);
                    with_replay_scope(
                        replay_snapshot.instance_logic_id,
                        replay_snapshot.group_path,
                        replay_snapshot.instance_key_override,
                        normalize_context_snapshot(&replay_snapshot.context),
                        || replay_snapshot.replay.run(),
                    );
                    let compose_ns = compose_timer.map(|timer| timer.elapsed().as_nanos());

                    let attach_timer = replay_trace.then(Instant::now);
                    TesseraRuntime::with_mut(|runtime| {
                        runtime
                            .component_tree
                            .finish_replace_subtree(replace_context, &mut replace_result)
                    });
                    if replay_trace {
                        tracing::debug!(
                            target: "tessera_ui::replay",
                            instance_key,
                            fn_name = trace_name.unwrap_or("<missing>"),
                            detach_ns,
                            compose_ns,
                            attach_ns = attach_timer.map(|timer| timer.elapsed().as_nanos()),
                            inserted = replace_result.inserted_count(),
                            reused = replace_result.retained_count(),
                            "replay_root_cpu"
                        );
                    }
                    executed_instance_keys.extend(replace_result.executed_keys());
                    recomposed_instance_logic_ids.extend(replace_result.recomposed_logic_ids());

                    stale_instance_keys
                        .extend(replace_result.removed_instance_keys.iter().copied());
                    stale_instance_logic_ids
                        .extend(replace_result.removed_instance_logic_ids.iter().copied());
                }
            });

            TesseraRuntime::with_mut(|runtime| runtime.component_tree.finish_replay_round());
            crate::nested_scroll::flush_attachment_changes();
            let round_invalidations = take_build_invalidations();
            // A write from another component after an instance was rebuilt in
            // this frame means that instance observed a stale value. It cannot
            // be replayed again in this frame (its replay snapshot
            // is only recorded once the frame finishes), so it is
            // deferred to the next frame.
            deferred_backwards_instance_keys.extend(
                round_invalidations
                    .cross_instance_dirty_instance_keys
                    .iter()
                    .copied()
                    .filter(|instance_key| executed_instance_keys.contains(instance_key)),
            );
            let still_pending = |instance_key: &u64| {
                !stale_instance_keys.contains(instance_key)
                    && !executed_instance_keys.contains(instance_key)
            };
            if Arc::ptr_eq(&pending_dirty_instance_keys, &dirty_instance_keys)
                && pending_dirty_instance_keys
                    .iter()
                    .chain(&round_invalidations.dirty_instance_keys)
                    .all(|key| !still_pending(key))
            {
                break;
            }
            let pending = Arc::make_mut(&mut pending_dirty_instance_keys);
            pending.extend(round_invalidations.dirty_instance_keys);
            pending.retain(still_pending);
            if pending.is_empty() {
                break;
            }
            plan = plan_dirty_replay(&pending_dirty_instance_keys);
        }

        release_retired_replay_snapshots();
        finalize_frame_layout_dirty_tracking();
        remove_frame_nanos_receivers(&stale_instance_keys);
        remove_focus_read_dependencies(&stale_instance_keys);
        remove_render_slot_read_dependencies(&stale_instance_keys);
        remove_state_read_dependencies(&stale_instance_keys);
        remove_layout_state_read_dependencies(&stale_instance_keys);
        crate::runtime::remove_build_invalidations(&stale_instance_keys);
        deferred_backwards_instance_keys
            .retain(|instance_key| !stale_instance_keys.contains(instance_key));
        crate::runtime::defer_build_invalidations(&deferred_backwards_instance_keys);
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
            let result =
                result.with_dirty_replay_info(had_invalidations, replay_roots_for_debug.clone());
            result
        }
        #[cfg(not(feature = "profiling"))]
        {
            let result = BuildTreeResult::partial_replay(build_tree_cost);
            #[cfg(feature = "debug-dirty-overlay")]
            let result =
                result.with_dirty_replay_info(had_invalidations, replay_roots_for_debug.clone());
            result
        }
    })
}
