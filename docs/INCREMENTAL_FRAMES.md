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

Custom `DrawCommand` implementations are conservative by default. Override
`supports_partial_redraw` only if every visual change invalidates the command's
paint fragment or geometry, and `draw_region` bounds every pixel it can affect.
Resources updated independently of paint, including deferred image uploads,
must leave this disabled. Background sampling and compute commands currently
require full redraw, including the first frame after their removal.

Target recreation and a missing rendered predecessor force full redraw. Damage
is relative to the preceding scene update, so it cannot be reused after a dropped
changed frame. This target-pixel reuse is not an independently compositable
subtree cache and does not change per-command opacity into group opacity.

Queued runtime wakes are ignored if their invalidations have already been
consumed. Pending layout, paint, animation, dispatcher, clipboard, and focus
work still requests another frame.

## Retained runtime storage

Context snapshots share their bindings until replay needs to remap or remove a
binding. Modifier snapshots share immutable chain links. The scene stores empty
and single-child branches inline and uses persistent ordered maps for wider
branches so updating one child can preserve the previous frame's snapshot.

Nodes allocate interaction data and accessibility descriptions only when used.
Pointer modifier positions are shared between the node and dispatch plans, and
nodes without pointer modifiers need no position buffer.

## Shared GPU resources

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
payloads. For a partial redraw, pass planning keeps the clip stack and only
copies draws that intersect the damage rectangle.

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
