//! Controller-driven shard routing primitives.
//!
//! ## Usage
//!
//! Mount `shard_home` at the app shell root to render the current shard page.

mod back;
mod controller;
mod destination;
mod home;

use tessera_ui::{State, use_context};

pub use controller::RouterController;
pub use destination::RouterDestination;
pub use home::shard_home;

/// Returns the router rendering the current page.
///
/// Call it during a build below [`shard_home`]; capture the returned handle in
/// callbacks to navigate.
///
/// # Panics
///
/// Panics when called outside [`shard_home`].
pub fn use_router() -> State<RouterController> {
    use_context::<State<RouterController>>()
        .expect("`use_router` requires a page rendered by `shard_home`")
        .get()
}
