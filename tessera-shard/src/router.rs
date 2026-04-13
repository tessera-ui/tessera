//! Controller-driven shard routing primitives.
//!
//! ## Usage
//!
//! Mount `shard_home` at the app shell root to render the current shard page.

mod controller;
mod destination;
mod home;
mod state;

pub use controller::RouterController;
pub use destination::RouterDestination;
pub use home::shard_home;

pub(crate) use home::{current_router_controller, with_current_router_shard_state};
pub(crate) use state::{RouteId, RouteShardKey, RouterContext};
