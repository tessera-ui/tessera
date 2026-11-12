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
