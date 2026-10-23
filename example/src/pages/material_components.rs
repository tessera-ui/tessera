//! Material component directory and specification route.
//!
//! ## Usage
//!
//! Browse Material components and open their interactive specification pages.

use tessera_shard::shard;

#[shard]
pub fn material_components_page() {
    super::material_spec::material_spec();
}
