//! Material component directory and specification route.
//!
//! ## Usage
//!
//! Browse Material components and open their interactive specification pages.

use tessera_shard::shard;

#[shard]
pub fn material_components_page(
    selected: Option<tessera_ui::State<Option<usize>>>,
    sessions: Option<crate::app::AppLiveSessions>,
) {
    super::material_spec::material_spec()
        .selected_optional(selected)
        .sessions_optional(sessions);
}
