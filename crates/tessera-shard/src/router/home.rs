use std::rc::Rc;

use tessera_ui::{State, provide_context, remember, tessera};

use crate::router::{RouterController, RouterDestination, back::route_display};

/// Mounts a router at the app root and renders its active destination.
///
/// On Android, system back pops one route and a predictive gesture previews
/// the previous route. Android handles back at the root of the stack.
///
/// Set `.root(...)` to the initial destination unless supplying a controller.
/// Pages can access the controller with [`crate::use_router`].
#[tessera]
pub fn shard_home(
    #[prop(skip_setter)] root: Option<Rc<dyn RouterDestination>>,
    controller: Option<State<RouterController>>,
) {
    let internal_controller = remember({
        let root = root.clone();
        move || {
            let mut router = RouterController::new();
            if let Some(root) = root {
                router.push_route(root);
            }
            router
        }
    });
    let controller = controller.unwrap_or(internal_controller);

    if root.is_none()
        && controller == internal_controller
        && controller.with(RouterController::is_empty)
    {
        panic!("shard_home requires `root` when `controller` is not provided");
    }

    provide_context(controller, || route_display(controller));
}

impl ShardHomeBuilder {
    /// Sets the initial destination used when `controller` is omitted.
    pub fn root(&self, root: impl RouterDestination) -> Self {
        let value: Option<Rc<dyn RouterDestination>> = Some(Rc::new(root));
        self.props
            .configure("root", move |props| props.root = value);
        Self { props: self.props }
    }
}
