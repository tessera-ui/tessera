# Incremental frame maintenance

## Geometry observers

Remeasuring without a size change preserves window geometry. Position,
transform, and clip changes invalidate it through their own journals.

Use `Modifier::layout_coordinate_space` on a figure or container and
`Modifier::on_layout_rect_in_space` on its measured descendants when only
relative geometry is needed. These observers compose local placements up to
the nearest declared space. Common ancestor translation and scaling do not
change the published bounds. Local size, placement, scaling, visibility, and
coordinate-space ancestry changes still publish updated bounds. Without a
declared space, bounds use window units.

Use `Modifier::on_layout_size_changed` for measured dimensions; unplaced
layouts defer notifications until placement resumes.

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
must leave this disabled. Background sampling and compute commands force full
redraw.

Target recreation and a missing rendered predecessor force full redraw. Damage
is relative to the preceding scene update, so it cannot be reused after a dropped
changed frame.

Input and layout-observer state writes settle through bounded build/layout
retries before presentation. Retries do not advance the frame clock or replay
consumed input. Build-time feedback to an already executed boundary remains
deferred until the next frame; nonconverging layout feedback yields after a
retry limit.

## Shared GPU resources

Use `GraphicsDevice::frame_buffer` for transient uniform and storage data. Each
allocation has a distinct aligned range, and each used page uploads once before
frame submission. Its contents are valid only for the current frame.
`cached_bind_group` caches non-array bindings briefly and may keep bound
resources alive.

Scratch texture leases prevent reuse until all consumers have been recorded.
Leases may be larger than requested; use the valid region, not the texture
size. Scene filters operate on region-sized targets, with `scene_rect` mapping
sampled pixels back to window coordinates.

Scene, MSAA, and stencil attachments reuse nearby sizes; a resize still forces
full redraw.

## Diagnostics

`LayoutFrameDiagnostics` reports per-phase work counts.
`HeadlessRenderer::diagnostics` exposes the most recent full frame's counters;
profiling output includes the same fields.

Component build durations include child builds; do not sum nested samples as
exclusive CPU time. Compare frame latency with tracing disabled separately from
work counts. GPU execution requires timestamp-query measurements and cannot be
inferred from CPU submission durations.

With native `profiling` builds, set `TESSERA_GPU_TIMING=1` to capture
`render_gpu_ns` in windowed or headless sessions. Timestamp readback waits for
GPU completion, so use a separate run with timing unset to measure frame
cadence. These timestamps cover frame passes and output blitting, not prelude
uploads, swapchain acquisition, or compositor presentation.
