use tessera_ui_macros::tessera;
use tessera_ui_shard::router::{Router, RouterDestination};

/// The root component of the router, which is the entry point for the routing system.
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
