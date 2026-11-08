# Incremental frame maintenance

## Geometry observers

Use `Modifier::layout_coordinate_space` on a figure or container and
`Modifier::on_layout_rect_in_space` on its measured descendants when only
relative geometry is needed. These observers compose local placements up to
the nearest declared space. Common ancestor translation and scaling do not
change the published bounds or invalidate annotation composition. Local size,
placement, scaling, visibility, and coordinate-space ancestry changes still
publish updated bounds. Without a declared space, bounds use window units.
Window-coordinate observers continue to receive ancestor movement normally.

Use `Modifier::on_layout_size_changed` for measured dimensions. Newly attached
observers receive the current size; unplaced layouts defer notifications until
placement resumes.

## Retained scene pixels

The renderer preserves its scene target between frames. Scene journals collect
the old and new drawing bounds of changed or removed branches. Eligible frames
clear that damage rectangle and replay intersecting commands in scene order,
with the same shape clips, blending, and MSAA as a full redraw. Unchanged frames
do not upload or draw scene batches; presentation still copies the scene target.
Small damage regions can build their draw list directly from intersecting scene
branches, retaining ancestor clips. The full draw list remains available on demand.

The retained scene follows the layout tree. Changes to direct layout children
update the parent branch, including when composition wrappers reorder retained
children. Repainting a retained branch preserves its child links and limits
damage to the changed drawing bounds. The walk over changed branches and their
ancestors builds the child-before-parent update order as it goes.

Custom `DrawCommand` implementations are conservative by default. Override
`supports_partial_redraw` only if every visual change invalidates the command's
paint fragment or geometry, and `draw_region` bounds every pixel it can affect.
Resources updated independently of paint, including deferred image uploads,
must leave this disabled. Background sampling and compute commands currently
require full redraw, including the first frame after their removal.

Target recreation and a missing rendered predecessor force full redraw. Damage
is relative to the preceding scene update, so it cannot be reused after a dropped
changed frame. The damage history keeps the predecessor's identity without
holding its draw commands alive. This target-pixel reuse is not an independently
compositable subtree cache and does not change per-command opacity into group
opacity.

Queued runtime wakes are ignored if their invalidations have already been
consumed. Pending layout, paint, animation, dispatcher, clipboard, and focus
work still requests another frame.

## Retained runtime storage

The session owns its core runtime tables directly and initializes them on first access.
Replay snapshots share finalized props and inherited context maps. Empty
contexts need no binding storage. Context and keyed scopes restore their prior
bindings when they exit. Control-flow scopes extend and restore a compact path
identity, which pending calls and replay snapshots store by value.
Pending calls install the live bindings captured at declaration. Replay
snapshots resolve bindings that can change across frames.
Replay snapshot records are contiguous, with a separate index by instance key.
Partial replay leaves unchanged snapshots in place and publishes all replacements
before releasing retired props.
Modifier snapshots share immutable chain links. The scene stores empty and
single-child branches inline and uses persistent ordered maps for wider branches
so updating one child can preserve the previous frame's snapshot.
Scene branches and layout metadata use dense node tables with generation checks.
Scene culling uses the inherited clip bounds computed by layout.

Nodes allocate interaction data, accessibility descriptions, and shape clip
regions only when used.
Descriptive semantics retain their text and inheritance fields and expand
them when publishing a snapshot. Rebuilt nodes reuse semantic storage while
the description keeps its representation.
Rectangular clips retain fractional bounds and derive outward pixel bounds as needed.
Scene branches and retained draw snapshots share clip records.
Pointer modifier positions and their transform metadata are allocated on demand.
The node and its dispatch plans share the position buffer.
The accessibility index starts from the live tree on first publication and
tracks only semantic nodes and the window root. A deactivated adapter releases
its index and stops collecting changes until publication resumes.

Composition wrappers without semantics need no geometry metadata outside the
root. Component nodes and input dispatch plans borrow static function names.
The layout primitive and built-in layout components can use the same node for
replay and geometry. Layout input
snapshots share the authored policy with component props, and each layout node
measures with its own policy clone. Cached child measurements share
their storage across placement updates, and leaf layouts need no child storage.
Layout adjacency stores a single child inline. Parents with a single composition
child can share its wider flattened layout child list.
The same ancestor walk marks dirty layout paths and records the affected children
for parents that can reuse their cached measurement.
State, context, focus, and render-slot reads share a dependency index with inline
storage for the first reader and dependency.
Functor lookup keeps one slot per order buffer inline and reuses overflow
storage in proportion to the previous build's order.
An order hit reuses the slot record directly. Order and key lookups share the
same update path.
Component slot ownership shares the same inline set used for dependency readers.
The slot key index stores handles and compares full keys in their slot records.

Partial replay preserves the node and its layout metadata when the component
identity and role match. Children declared in the same order keep their tree
links. Reordered children remain eligible for reuse until the replacement
finishes. Retained descendants compare their old and new consumer ancestry
before the frame updates focus, semantics, and position observers.

Each replay round refreshes affected parent layout lists from the deepest
parent outward. Sibling replacements attach at their original positions and
share that refresh. Consumers compare retained ancestry after the round's order
is available.
Replay records rebuilt identities and retained roots as they attach. Removal
checks index the rebuilt identities only when a large replacement has old
branches to remove.
Replay planning shares the invalidation set when all keys are live and filters
expired keys before execution. Pending work is copied only when the first
round leaves invalidations to process.
Disjoint replay roots keep their planned node IDs and load their snapshots just
before execution.
Component replay uses the same props allocation prepared by the registered call.
Immediate setters take only the pending configuration while the declaration
scope stays registered.
Subtree replacements reuse the build queue and bounded temporary identity sets.
Detached subtrees transfer reusable geometry through the live instance index.
Retained subtrees attach directly without allocating a temporary tree node.
Focus, pointer dispatch, and accessibility read sibling order from the same
indices stored on the component nodes.
Frame structure journals reuse capacity in proportion to recent changes and the
live tree after all consumers have read them.
Focus reads the geometry and structure journals directly when it has attached
targets or active focus.
The pointer index reads the geometry journal directly and refreshes plans for
nodes with pointer handlers or cursor modifiers.
Rebuilt layout leaves with a matching instance keep their node IDs and cached
geometry. Changes to input geometry or the parent invalidate placement.

## Shared GPU resources

Component draw and compute pipelines are created on first use. Factories
receive the renderer's resources before that frame starts its pipeline hooks.

Use `GraphicsDevice::frame_buffer` for transient uniform and storage data. Each
allocation has a distinct aligned range, and each used page uploads once before
frame submission. Its contents are valid only for the current frame. Use
`cached_bind_group` for repeated buffer, texture, and sampler bindings; array
bindings bypass the cache. The binding cache retains at most 512 entries for up
to two unused frames and can keep bound resources alive during that interval.

Scratch texture leases prevent reuse until all consumers have been recorded.
Idle leases expire after two unused frames and share a 64 MiB budget; active
leases are not evicted. Approximate-fit requests share size buckets, so shaders
must use the actual texture dimensions and the valid region separately. Scene
filters operate on region-sized targets, with `scene_rect` mapping sampled
pixels back to window coordinates.

Vector atlas pages retain recently used sizes for up to 120 unused frames.
The cache evicts oldest inactive pages above 32 MiB; pages sampled by the current
frame are never evicted to satisfy that budget.

Native component-package registration starts system-font scanning and matching
for normal, medium, and bold text on a worker, overlapping renderer startup.
The first text layout waits only if that preparation has not completed; custom
font registration still invalidates the affected matching caches normally.
Plain text shares immutable content across component props, layout, and semantics.
Text measurement and drawing share the shaped buffer, and only drawing keeps
the text color.
The measurement cache compares the text and pixel format stored in the shaped
data.
Initial layout and reflow share live glyph buffers and measurements for identical
short text and constraints within a runtime session. A mutable font-system borrow
clears the bounded weak lookup and invalidates older buffers as reflow sources.

Scene, MSAA, and stencil attachments reuse nearby allocation sizes while layout,
clipping, and drawing retain the exact viewport dimensions. Capacity grows in
128-pixel steps with shrink hysteresis: each axis keeps less than 256 pixels of
slack and at most twice the requested extent (with a 128-pixel floor). A resize
still forces a full scene redraw. Offscreen output and readback remain exact-sized.
Windows D3D12 DirectComposition surfaces also reuse capacity and present unscaled
pixels clipped by the client window; other presentation systems remain exact-sized.
Forced surface recovery reconfigures even when capacity already fits.

## Diagnostics

`LayoutFrameDiagnostics` reports measurement calls and dirty-child checks,
geometry visits and updates, paint recordings, scene branch updates, pointer
index updates and hit candidates, dispatch selection, and semantic updates.
`HeadlessRenderer::diagnostics` exposes the most recent full frame's counters;
profiling output includes the same fields.

Render-graph ordering is deferred until the target needs scene work and is
included in `render_build_passes_ns`; `render_prepare_ns` is not emitted for a
separate pre-render ordering step.

Draw order can be reused across scene updates when the authoring sequence,
pipeline types, clip transitions, and ordering bounds still match. The order
plan contains indices into the current commands and does not retain prior paint
payloads. Scene snapshots share recorded fragments until the fragment changes.
Pass planning borrows commands from the graph. A partial redraw keeps the clip
stack and only plans draws that intersect the damage rectangle.

Windowed `frame_total_ns` includes pending surface and render-target resize work.

Component build durations include child builds; do not sum nested samples as
exclusive CPU time. Compare frame latency with tracing disabled separately from
work counts. GPU execution requires timestamp-query measurements and cannot be
inferred from CPU submission durations.

With native `profiling` builds, set `TESSERA_GPU_TIMING=1` to capture
`render_gpu_ns` in windowed or headless sessions. The existing
`TESSERA_HEADLESS_GPU_TIMING` name remains supported. Timestamp readback waits for
GPU completion, so use a separate run with both flags unset to measure frame
cadence. These timestamps cover frame passes and output blitting, not prelude
uploads, swapchain acquisition, or compositor presentation.
