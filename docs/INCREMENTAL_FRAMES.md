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

Direct layout children are retained through transparent composition nodes.
Structural reconciliation updates that adjacency; measurement reads it through
shared handles. Cached child constraints and sizes are shared, and dirty-child
indexes identify the size dependencies that need checking.
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

- Paint records queued visible nodes. Revealing a dirty hidden node queues its
  first recording.
- Position observers inspect changed geometry and newly attached observers.
- Focus attachments and ordered child indexes update only affected handles.
- Pointer plans persist with a spatial grid. Exact hit tests preserve ancestor
  shape clipping, modifier placement, overflow, and composition order. Dispatch
  includes current and previous hover candidates, sampled hit paths, captures,
  newly attached handlers, and invalidated state readers. Each sample finishes
  all three input passes before the next sample starts.
- The scene retains render-bearing branches and compresses transparent chains.
  An unchanged scene shares its render graph and cached execution ordering.
- Accessibility retains stable semantic identities and child order. Ordinary
  frames publish changed nodes and focus. Initial platform activation and
  explicit headless snapshots publish the complete semantic tree. Inherited
  tooltip and semantic-boundary changes update the affected descendants.

Partial replay reads snapshots for its selected roots. State and context slot
cleanup uses owner indexes and releases table borrows before dropping values.
Persistent focus handles and frame receivers are also removed by owner.

## Work that scales with affected output

Structural replay reconciles its replacement subtree. A changed parent origin
or clip updates affected descendants. Row and Column must revisit sibling
placements when a child's size changes. Focus navigation searches its eligible
scope when navigation is requested. Active frame receivers run their callbacks.
Changed render scenes assemble their command output; GPU pass construction,
uploads, and command encoding process the commands being rendered. Explicit
profiling and inspection snapshots enumerate their requested output.

These paths differ from unconditional component-tree reconstruction on every
frame. They remain important when profiling broad updates and large outputs.

## Verification

`LayoutFrameDiagnostics` reports measurement calls and dirty-child checks,
geometry visits and updates, paint recordings, scene branch updates, pointer
index updates and hit candidates, dispatch selection, and semantic updates.
`HeadlessRenderer::diagnostics` exposes the most recent full frame's counters;
profiling output includes the same fields.

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
