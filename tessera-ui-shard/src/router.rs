//! Stack-based routing utilities for shard components.
//!
//! Each `#[shard]` function generates a `*Destination` type that implements
//! [`RouterDestination`]. These destinations are managed in a LIFO stack.
//!
//! # Responsibilities
//!
//! * Maintain an ordered stack (`route_stack`) of active destinations
//! * Expose [`Router::push`] / [`Router::pop`] helpers that also manage shard
//!   state lifetimes
//! * Remove per‑shard state from the registry when a destination whose
//!   lifecycle is `ShardStateLifeCycle::Shard` is popped
//! * Keep routing logic minimal; rendering happens when the top destination's
//!   `exec_component()` is invoked every frame by `router_root`
//!
//! # Related
//!
//! * `#[shard]` macro – generates the `*Destination` structs + optional state
//!   injection
//! * `tessera_ui::router::router_root` – executes the current top destination
//!   each frame
use std::sync::OnceLock;

use parking_lot::RwLock;

use crate::{ShardRegistry, ShardStateLifeCycle};

static ROUTER: OnceLock<RwLock<Router>> = OnceLock::new();

pub struct Router {
    /// Whether the router has been initialized with a default destination
    initialized: bool,
    /// Current route stack
    route_stack: Vec<Box<dyn RouterDestination>>,
}

impl Router {
    fn new() -> Self {
        Self {
            initialized: false,
            route_stack: Vec::new(),
        }
    }

    /// Initialize the router with a default destination if not already done.
    pub fn try_init(defualt_dest: impl RouterDestination + 'static) -> bool {
        Self::with_mut(|router| {
            if router.initialized {
                return false;
            }
            router.push(defualt_dest);
            router.initialized = true;
            true
        })
    }

    /// Execute a closure with exclusive mutable access to the router.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let router = ROUTER.get_or_init(|| RwLock::new(Self::new()));
        let mut router = router.write();
        f(&mut router)
    }

    /// Execute a closure with shared read access to the router.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        let router = ROUTER.get_or_init(|| RwLock::new(Self::new()));
        let router = router.read();
        f(&router)
    }

    /// Push a new route destination onto the stack (internal helper).
    pub fn push<T: RouterDestination + 'static>(&mut self, destination: T) {
        self.route_stack.push(Box::new(destination));
    }

    /// Pop the top route destination from the stack.
    ///
    /// Returns `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<Box<dyn RouterDestination>> {
        let dest = self.route_stack.pop()?;
        // Decide cleanup by destination lifecycle, defaulting to Shard.
        let life_cycle = dest.life_cycle();
        if life_cycle == ShardStateLifeCycle::Shard {
            // Remove per-shard state when destination is discarded
            ShardRegistry::get().shards.remove(dest.shard_id());
        }
        Some(dest)
    }

    /// Whether the router is empty.
    pub fn is_empty(&self) -> bool {
        self.route_stack.is_empty()
    }

    /// Get the current top route destination, used for route component display.
    pub fn last(&self) -> Option<&dyn RouterDestination> {
        self.route_stack.last().map(|v| &**v)
    }

    /// Get the length of the route stack.
    pub fn len(&self) -> usize {
        self.route_stack.len()
    }

    /// Clear all routes from the stack.
    pub fn clear(&mut self) {
        self.route_stack.clear();
    }

    /// Clear all routes from the stack and reset initialization state.
    ///
    /// This allows the root destination to be initialized again.
    pub fn reset(&mut self) {
        self.route_stack.clear();
        self.initialized = false;
    }

    /// Clear all routes from the stack and push a new root destination.
    pub fn reset_with(&mut self, root_dest: impl RouterDestination + 'static) {
        self.route_stack.clear();
        self.push(root_dest);
    }
}

/// A navigation destination produced automatically by the `#[shard]` macro.
///
/// You should not manually implement this trait. Each annotated shard function
/// creates a `*Destination` struct that implements `RouterDestination`.
pub trait RouterDestination: Send + Sync {
    /// Execute the component associated with this destination.
    fn exec_component(&self);
    /// Stable shard identifier used for state registry lookups / cleanup.
    fn shard_id(&self) -> &'static str;
    /// Lifecycle policy for the shard state tied to this destination.
    ///
    /// Default is `Shard`, which means the associated shard state will be
    /// removed from the registry when this destination is popped.
    /// Override in generated implementations to persist for the whole app.
    fn life_cycle(&self) -> ShardStateLifeCycle {
        ShardStateLifeCycle::Shard
    }
}
