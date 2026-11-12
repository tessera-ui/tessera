use std::{any::TypeId, rc::Rc};

use tessera_ui::{ModelStore, ModelStoreScope};

use crate::RouterDestination;

struct RouteEntry {
    id: u64,
    destination: Rc<dyn RouterDestination>,
    models: ModelStore,
}

/// Owns the destination stack and the view models of its routes.
///
/// Each route entry provides a [`ModelStore`] to the content rendered for it, so
/// `Model<T>` parameters there live with the route: covered routes keep their
/// models, and popping a route closes them.
pub struct RouterController {
    route_stack: Vec<RouteEntry>,
    next_route_id: u64,
}

impl RouterController {
    /// Creates an empty controller.
    pub fn new() -> Self {
        Self {
            route_stack: Vec::new(),
            next_route_id: 0,
        }
    }

    /// Creates a controller seeded with a root destination.
    pub fn with_root(root: impl RouterDestination) -> Self {
        let mut router = Self::new();
        router.push(root);
        router
    }

    /// Pushes a destination with its own model lifetime.
    pub fn push(&mut self, destination: impl RouterDestination) {
        self.push_route(Rc::new(destination));
    }

    pub(crate) fn push_route(&mut self, destination: Rc<dyn RouterDestination>) {
        self.route_stack.push(RouteEntry {
            id: self.next_route_id,
            destination,
            models: ModelStore::new(),
        });
        self.next_route_id += 1;
    }

    /// Pops the current destination and closes its models.
    pub fn pop(&mut self) -> Option<Rc<dyn RouterDestination>> {
        self.route_stack.pop().map(|removed| removed.destination)
    }

    /// Replaces the current destination, or pushes into an empty stack.
    pub fn replace(
        &mut self,
        destination: impl RouterDestination,
    ) -> Option<Rc<dyn RouterDestination>> {
        let previous = self.pop();
        self.push(destination);
        previous
    }

    /// Returns whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.route_stack.is_empty()
    }

    /// Returns the number of destinations.
    pub fn len(&self) -> usize {
        self.route_stack.len()
    }

    /// Returns the current destination.
    pub fn last(&self) -> Option<&dyn RouterDestination> {
        self.route_stack.last().map(|entry| &*entry.destination)
    }

    /// Returns whether the current destination has type `D`.
    pub fn current_is<D: RouterDestination>(&self) -> bool {
        self.last()
            .is_some_and(|current| current.type_id() == TypeId::of::<D>())
    }

    pub(crate) fn route_at_depth(
        &self,
        depth: usize,
    ) -> Option<(u64, Rc<dyn RouterDestination>, ModelStoreScope)> {
        self.route_stack
            .iter()
            .rev()
            .nth(depth)
            .map(|entry| (entry.id, entry.destination.clone(), entry.models.scope()))
    }

    #[cfg(test)]
    pub(crate) fn route_models(&self, route: u64) -> Option<&ModelStore> {
        self.route_stack
            .iter()
            .find(|entry| entry.id == route)
            .map(|entry| &entry.models)
    }

    /// Clears all destinations and closes their models.
    pub fn clear(&mut self) {
        self.route_stack.clear();
    }

    /// Clears the stack and installs a new root destination.
    pub fn reset(&mut self, root: impl RouterDestination) {
        self.clear();
        self.push(root);
    }
}

impl Default for RouterController {
    fn default() -> Self {
        Self::new()
    }
}
