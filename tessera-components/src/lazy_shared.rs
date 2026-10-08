//! Shared helpers for the lazy scrolling components.

use tessera_ui::Px;

/// Clamps the main-axis size a lazy component reports to its parent.
///
/// The reported size never exceeds the viewport, and is further limited by the
/// parent constraint for the main axis, falling back to `fallback_limit` when
/// the parent does not constrain that axis.
pub(crate) fn clamp_reported_main(
    total_main: Px,
    viewport_span: Px,
    constraint_max: Option<Px>,
    fallback_limit: Option<Px>,
) -> Px {
    let viewport = viewport_span.max(Px::ZERO);
    let mut report = total_main.min(viewport);
    if let Some(max_value) = constraint_max.or(fallback_limit) {
        report = report.min(max_value.max(Px::ZERO));
    }
    report
}
