//! Root routing entry utilities.
//!
//! This module re‑exports [`push`] and [`pop`] for manipulating the navigation stack
//! and provides the [`router_root`] component which drives per‑frame execution of
//! the current (top) destination.
//!
//! Core flow:
//! * On the first frame, the supplied `root_dest` is pushed if the stack is empty.
//! * On every frame, the top destination's `exec_component()` is invoked.
//!
//! The actual stack and destination logic live in `tessera_ui_shard::router`.
//!
//! # Typical Minimal Usage
//!
//! ```
//! use tessera_ui::{tessera, shard, router::{router_root, self}};
//!
//! #[shard]
//! #[tessera]
//! fn home_screen() { /* ... */ }
//!
//! // In your app's root layout:
//! router_root(HomeScreenDestination {});
//!
//! ##[shard]
//! ##[tessera]
//! # fn settings_screen() { /* ... */ }
//!
//! // Somewhere inside an event (e.g. button click) to navigate:
//! router::push(SettingsScreenDestination {});
//!
//! // To go back:
//! router::pop();
//! ```
//!
//! # Behavior
//!
//! * `router_root` is idempotent regarding the initial destination: it only pushes
//!   `root_dest` when the stack is empty.
//! * Subsequent frames never push automatically; they only execute the current top.
//! * If the stack is externally cleared (not typical), `router_root` will push again.
//!
//! # Panics
//!
//! Panics if after internal logic the stack is still empty (indicates an unexpected
//! mutation from user code while in the execution closure).
//!
//! # See Also
//!
//! * [`tessera_ui_shard::router::RouterDestination`]
//! * `#[shard]` macro which generates `*Destination` structs.
use tessera_ui_macros::tessera;
use tessera_ui_shard::router::{Router, RouterDestination};

pub use tessera_ui_shard::router::{pop, push};

/// Root component that drives the shard router stack each frame.
///
/// See module‑level docs for detailed usage patterns.
///
/// # Parameters
/// * `root_dest` - The destination pushed only once when the stack is empty.
///
/// # Notes
/// Keep `router_root` exactly once at the point in your tree that should display
/// the active routed component. Wrapping it multiple times would still execute
/// only one top destination but wastes layout work.
#[tessera(crate)]
pub fn router_root(root_dest: impl RouterDestination + 'static) {
    Router::with_mut(|router| {
        if router.is_empty() {
            router.push(root_dest);
        }
        router
            .last()
            .expect("Router stack should not be empty")
            .exec_component();
    });
}
