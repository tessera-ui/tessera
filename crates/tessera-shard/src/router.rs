//! Controller-driven shard routing primitives.
//!
//! ## Usage
//!
//! Mount `shard_home` at the app shell root to render the current shard page.

mod back;
mod controller;
mod destination;
mod home;

pub use controller::RouterController;
pub use destination::RouterDestination;
pub use home::shard_home;

#[cfg(test)]
pub(crate) use crate::__private::with_current_router_shard_state;
