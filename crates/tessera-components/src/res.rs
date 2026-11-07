//! Generated asset bindings for tessera-components.
//!
//! ## Usage
//!
//! Pass exported asset constants to components or load them with [`tessera_ui::Resource::load`].
#[allow(missing_docs, clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/tessera_assets.rs"));
}

pub use generated::*;
