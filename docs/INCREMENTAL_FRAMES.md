# Incremental frame maintenance

Tessera retains layout adjacency, measured layouts, window geometry, draw
fragments, and frame-consumer indexes across frames. A clean frame reuses the
root measurement and scene without walking unchanged component subtrees.

## Invalidation boundaries

State reads are attributed to their execution phase. Build reads invalidate
composition. Measure reads invalidate size dependencies. Placement reads can
reposition children using cached measurements. Placement-modifier reads update
window transforms. Paint reads record draw fragments. Pointer-handler reads
schedule delivery to the subscribed input node even when it is outside the
current hover path.

A changed child size propagates through parents that depend on that size. A
dirty child whose size stays unchanged does not require restoring its parent's
cached placements. Intrinsic measurements retain their separate metadata and
dependency rules. A placement-only change can still require measurement when a
layout policy does not provide a compatible placement result.

Measurement invalidations in unplaced subtrees must outlive the current frame.
After layout publishes its placements, measurement and placement invalidations
retain their phase and direct child paths within unplaced branches, stopping at
the first placed layout. This also covers mutations in the frame that hides a
subtree. Hidden branches remain idle; their next measurement checks only these
pending paths. Placement updates retain measured dimensions. Hidden transform
updates invalidate local placement until geometry is published again.

Direct layout children are retained through transparent composition nodes.
Structural reconciliation updates that adjacency; measurement reads it through
shared handles. Cached child constraints and sizes are shared, and dirty-child
indexes identify the size dependencies that need checking.
Replay propagates adjacency changes through transparent ancestors only until a
layout boundary or unchanged adjacency. Profiling subtree counts propagate
separately as size deltas.
When parent constraints change, layout wrappers can reuse their content if the
effective content constraints and final size remain unchanged. This prevents a
fixed-size sibling from remeasuring merely because a flex container offers it a
different remaining maximum size.

## Geometry and frame consumers

Geometry dirtiness identifies affected ancestor paths. If an ancestor's window
geometry stays unchanged, only its dirty child branches are visited. Changed
origins, inherited clipping, opacity, placement order, and visibility propagate
to their affected descendants. Unplaced layout subtrees lose input, semantics,
drawing, and focus eligibility.
Propagation stops when a child's window transform, rectangular clipping,
opacity, and clip ancestry remain unchanged, including counter-translated
pinned content and size-only changes in unclipped parents. Independently dirty
descendants still update. Ancestor shape changes propagate even when their
bounding rectangles are unchanged, preserving current descendant hit tests.
An already hidden, unchanged subtree does not republish descendant geometry
when a visible ancestor moves. First hiding, structural changes, and revealing
still update the affected consumers.

Every affected layout retains current window coordinates, including passive
containers. Geometry change notifications include only nodes with drawing,
clipping, semantics, input, focus, or position-observer capabilities. Passive
container movement still propagates to its descendants without notifying
unrelated frame consumers. First hiding continues to publish affected subtree
removals, and newly attached capabilities receive their initial update.

Use `Modifier::layout_coordinate_space` on a figure or container and
`Modifier::on_layout_rect_in_space` on its measured descendants when only
relative geometry is needed. These observers compose local placements up to
the nearest declared space. Common ancestor translation and scaling do not
change the published bounds or invalidate annotation composition. Local size,
placement, scaling, visibility, and coordinate-space ancestry changes still
publish updated bounds. Without a declared space, bounds use window units.
Window-coordinate observers continue to receive ancestor movement normally.
The observer index tracks each declared space's subscribers separately from
window-geometry notifications. Counter-translated descendants therefore update
their relative observations even when cached window geometry stays unchanged.

Use `Modifier::on_layout_size_changed` for consumers of measured dimensions.
Size observations have a separate change journal. Size-only observers are
excluded from window-position indexing, so common translations and ancestor
scaling do not visit them when local dimensions stay unchanged. Newly attached
observers receive the current size;
unplaced layouts defer notifications until placement resumes. Window and
relative-coordinate observers retain their own geometry dependencies.

Geometry propagation uses retained direct layout adjacency, skipping transparent
composition wrappers. An inherited translation reuses unchanged local placement
results and their state dependencies; local placement changes still reevaluate
the affected modifier chain.
Read-only geometry traversal borrows cached child lists without acquiring an
additional shared reference for each visited node.
Input action positions remain local and share their storage. Input consumers
map requested positions through the current parent transform without allocating
a world-transform array for each translated node.

- Paint records queued visible nodes. Revealing a dirty hidden node queues its
  first recording.
- Position observers intersect changed geometry with retained registrations and
  inspect newly attached observers. Hidden registrations remain eligible when
  their layout is revealed again.
- Focus attachments and ordered child indexes update only affected handles.
  Placement-order changes have their own journal. Pure translations update
  focus rectangles without rebuilding traversal paths; transparent composition
  wrappers do not override the order of placed layout children.
- Pointer plans persist with a spatial grid. Exact hit tests preserve ancestor
  shape clipping, modifier placement, overflow, and composition order. Dispatch
  includes current and previous hover candidates, sampled hit paths, captures,
  newly attached handlers, and invalidated state readers. Each sample finishes
  all three input passes before the next sample starts.
  Geometry updates retain unchanged handlers and composition order. Moving an
  already clipped handler does not schedule input delivery by itself; captures,
  hover exits, and explicit state invalidations still schedule delivery.
  Geometry journals are intersected with registered pointer capabilities before
  updating the spatial index; decorative layout changes do not enter it.
  Pointer entries retain their rectangular grid coverage. Position updates
  refresh input geometry without changing shared cells; boundary crossings
  insert and remove only the differing cells. Hidden and oversized entries
  transition between coverage modes without allocating per-entry cell lists.
  Dispatch shares cached handler plans instead of copying their action lists.
  Geometry updates reuse uniquely owned plans and function-name buffers;
  retained copies stay immutable, and structural changes replace handlers.
- The scene retains render-bearing branches and compresses transparent chains.
  An unchanged scene shares its render graph and cached execution ordering.
  Transparent unary branches forward their current descendant output without
  rebuilding every wrapper. Fully clipped output and draws that remain outside
  the viewport retain their geometry until they can contribute again.
  Invisible draw leaves do not occupy the output tree. Updating their paint
  retains unchanged visible output; revealing them publishes the latest paint.
  Ancestor branches reuse their output when their draw, clip, and child outputs
  remain unchanged.
  Geometry changes share immutable paint fragments. Frame assembly applies
  opacity to its command output without mutating the retained fragment.
  Assembly borrows fragment operations directly and remaps contiguous resource
  IDs without allocating intermediate fragment or resource-mapping vectors.
  Renderer batches borrow the pass's owned commands and reuse reference scratch
  storage across pipeline and clip boundaries, including overflow capacity.
  Shape batches append instances to retained storage-buffer blocks. Cached
  transforms and uncached shape parameters use separate pools; each draw keeps
  its own instance range across clip and pass boundaries. Frame-end uploads
  publish the used blocks before the frame's command buffer is submitted.
  Geometry journals are intersected with drawing, clipping, and hidden scene
  branches before evaluating scene candidates.
  Draw visibility is retained until geometry or viewport dimensions change.
  Text commands share shaped glyph buffers; scrolling and span-color opacity
  changes copy shared buffers before mutation.
- Accessibility retains stable semantic identities and child order. Ordinary
  frames publish changed nodes and focus. Initial platform activation and
  explicit headless snapshots publish the complete semantic tree. Inherited
  tooltip and semantic-boundary changes update the affected descendants.
  Position-only changes update existing semantic bounds without rebuilding
  labels, actions, parent relationships, or child ordering.
  Reading order follows placed layout children across transparent composition
  wrappers, independently of overlay paint order. Placement-order changes
  relink affected semantic children without replacing their identities or focus.
  Platform publication starts after accessibility activation. Without an active
  consumer, bounded change journals coalesce updates until an explicit semantic
  snapshot or activation requests the current tree. Removed unpublished nodes
  leave these journals instead of accumulating across navigation.

Partial replay reads snapshots for its selected roots. Reusing an unchanged
subtree journals its root without republishing every descendant to input,
focus, accessibility, and scene consumers. Replay compares the retained root's
ancestor path before and after attachment. Changes in sibling order, semantic
boundaries or inherited tooltips, and focus ancestry also update that root's
descendants. Capturing these paths costs the depth of each retained root,
rather than another traversal of its unchanged descendants.

A dirty descendant does not force an equal-prop ancestor to execute. Replay
retains that ancestor and schedules the remaining dirty boundary separately,
including invalidations created while replaying its parent. Only boundaries
actually executed in the frame count as covered. Forward invalidations finish
before layout without a fixed replay-round limit, so nested slots do not add
frames of stale content. Each boundary executes at most once per frame.
Writes targeting an already
executed boundary are deferred to the next frame, preserving the validity of
its replay snapshot. Replacement cleanup enumerates abandoned nodes rather
than every descendant of a reused subtree.

State and context slot cleanup uses owner indexes and releases table borrows
before dropping values. Persistent focus handles and frame receivers are also
removed by owner.

## Work that scales with affected output

Structural replay reconciles its replacement subtree. A changed parent origin
or clip updates affected descendants. Row and Column must revisit sibling
placements when a child's size changes. Focus navigation searches its eligible
scope when navigation is requested. Active frame receivers run their callbacks.
Changed render scenes assemble their command output; GPU pass construction,
uploads, and command encoding process the commands being rendered. Explicit
profiling and inspection snapshots enumerate their requested output.
Shape pipelines retain separate storage-buffer pools for cached and uncached
batches. Draws append disjoint instance ranges and upload each used block once
at the frame boundary. Warmed batches reuse capacity; larger batches replace
only an unused block, preserving all earlier draws in the same frame.
Shape cache heat advances once per rendered frame, independently of draw-batch
count. Repeated batches cannot prematurely warm a transient shape or expire
another stable shape within the same frame.
Vector atlas samples share aligned uniform-buffer blocks with dynamic offsets.
Each active block uploads once before frame submission; unused retained blocks
do not participate in that frame's upload work.
Short typed draw batches use inline scratch storage and spill to the heap only
when their command count exceeds that capacity.

These paths differ from unconditional component-tree reconstruction on every
frame. They remain important when profiling broad updates and large outputs.

## Verification

`LayoutFrameDiagnostics` reports measurement calls and dirty-child checks,
geometry visits and updates, paint recordings, scene branch updates, pointer
index updates and hit candidates, dispatch selection, and semantic updates.
`HeadlessRenderer::diagnostics` exposes the most recent full frame's counters;
profiling output includes the same fields.
Semantic publication can flush after drawing; explicit headless semantic
snapshots materialize current updates even when rendering had no active
accessibility consumer. Layout test sessions flush semantics synchronously.
Profiling maintains subtree counts during structural updates instead of walking
reused subtrees to count them each frame. Optional tracing targets
`tessera_ui::replay`, `tessera_ui::build_cost`, and `tessera_ui::frame_cost` expose
replay boundaries, component build durations, and frame-consumer stage costs.
At debug level, `scene_update_work` distinguishes structural candidates,
geometry notifications, directly changed nodes, and rebuilt scene branches.
`position_observer_work` counts callbacks scheduled by geometry publication.
It also reports candidate counts and entries in the size-change journal.
Compare these counters with replay boundaries to distinguish geometry-only
scroll updates from composition triggered by position observers. Component
build durations include child builds; do not sum nested samples as exclusive
CPU time. Detailed tracing also adds overhead, so compare frame latency with
tracing disabled separately from work counts.
At debug level, `tessera_ui::frame_cost` also reports `render_submit_cpu` with
separate `encoder_finish_ns` and `queue_submit_ns` fields. Both are CPU-side
durations; the profiling submission metric includes both operations. GPU
execution requires timestamp-query measurements and must not be inferred from
either CPU duration.

Run deterministic core and component regressions with:

```text
cargo test -p tessera-ui --all-features --lib
cargo test -p tessera-components --lib
cargo test --workspace --doc
```

The full-frame GPU regression requires an offscreen adapter and runs in an
isolated child process so its density cannot affect parallel tests:

```text
cargo test -p tessera-components --lib layout_flex::drawing_tests -- --ignored
```

It verifies pixels and semantic bounds alongside work limits for 128 rendered,
interactive semantic children, including idle frames, color changes,
translation, size changes, hover exit, and off-hover input-state invalidation.
