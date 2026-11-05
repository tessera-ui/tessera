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

## Diagnostics

`LayoutFrameDiagnostics` reports measurement calls and dirty-child checks,
geometry visits and updates, paint recordings, scene branch updates, pointer
index updates and hit candidates, dispatch selection, and semantic updates.
`HeadlessRenderer::diagnostics` exposes the most recent full frame's counters;
profiling output includes the same fields.

Component build durations include child builds; do not sum nested samples as
exclusive CPU time. Compare frame latency with tracing disabled separately from
work counts. GPU execution requires timestamp-query measurements and cannot be
inferred from CPU submission durations.
