//! Component prop and replay abstractions.
//!
//! ## Usage
//!
//! Define a `Prop` args type for `#[tessera]` components and pass it by
//! shared reference.

use std::{any::Any, sync::Arc};

/// Stable, comparable slot handle for any shared callable trait object.
///
/// `Slot` compares by identity (`Arc::ptr_eq`) so it can be used in component
/// props without forcing deep closure comparisons.
pub struct Slot<F: ?Sized> {
    inner: Arc<F>,
}

impl<F: ?Sized> Slot<F> {
    /// Create a slot from a shared callable trait object.
    pub fn from_shared(handler: Arc<F>) -> Self {
        Self { inner: handler }
    }

    /// Read the current callable.
    pub fn shared(&self) -> Arc<F> {
        Arc::clone(&self.inner)
    }
}

impl<F: ?Sized> Clone for Slot<F> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<F: ?Sized> PartialEq for Slot<F> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<F: ?Sized> Eq for Slot<F> {}

/// Stable, comparable callback handle for `Fn()`.
///
/// `Callback` compares by identity (`Arc::ptr_eq`) so it can be used in
/// component props without forcing deep closure comparisons.
#[derive(Clone)]
pub struct Callback {
    slot: Slot<dyn Fn() + Send + Sync>,
}

impl Callback {
    /// Create a callback handle from a closure.
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            slot: Slot::from_shared(Arc::new(handler)),
        }
    }

    /// Invoke the callback.
    pub fn call(&self) {
        let handler = self.slot.shared();
        handler();
    }
}

impl<F> From<F> for Callback
where
    F: Fn() + Send + Sync + 'static,
{
    fn from(handler: F) -> Self {
        Self::new(handler)
    }
}

impl Default for Callback {
    fn default() -> Self {
        Self::new(|| {})
    }
}

impl PartialEq for Callback {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
    }
}

impl Eq for Callback {}

/// Stable, comparable callback handle for `Fn(T) -> R`.
///
/// This is useful for value-change handlers and similar one-argument callbacks.
pub struct CallbackWith<T, R = ()> {
    slot: Slot<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> CallbackWith<T, R> {
    /// Create a callback handle from a closure.
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        Self {
            slot: Slot::from_shared(Arc::new(handler)),
        }
    }

    /// Invoke the callback with an argument.
    pub fn call(&self, value: T) -> R {
        let handler = self.slot.shared();
        handler(value)
    }
}

impl<T, R, F> From<F> for CallbackWith<T, R>
where
    F: Fn(T) -> R + Send + Sync + 'static,
{
    fn from(handler: F) -> Self {
        Self::new(handler)
    }
}

impl<T, R> Clone for CallbackWith<T, R> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot.clone(),
        }
    }
}

impl<T, R> PartialEq for CallbackWith<T, R> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
    }
}

impl<T, R> Eq for CallbackWith<T, R> {}

/// Stable, comparable render slot handle.
///
/// `RenderSlot` is semantically distinct from `Callback`, but it has the same
/// identity semantics and is optimized for "call child content later" patterns.
#[derive(Clone)]
pub struct RenderSlot {
    slot: Slot<dyn Fn() + Send + Sync>,
}

impl RenderSlot {
    /// Create a render slot from a closure.
    pub fn new<F>(render: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            slot: Slot::from_shared(Arc::new(render)),
        }
    }

    /// Execute the render closure.
    pub fn render(&self) {
        let render = self.slot.shared();
        render();
    }
}

impl<F> From<F> for RenderSlot
where
    F: Fn() + Send + Sync + 'static,
{
    fn from(render: F) -> Self {
        Self::new(render)
    }
}

impl From<Callback> for RenderSlot {
    fn from(callback: Callback) -> Self {
        Self {
            slot: callback.slot,
        }
    }
}

impl PartialEq for RenderSlot {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
    }
}

impl Eq for RenderSlot {}

/// Stable, comparable render slot handle for `Fn(T) -> R`.
///
/// This is useful for deferred rendering that depends on an input value.
pub struct RenderSlotWith<T, R = ()> {
    slot: Slot<dyn Fn(T) -> R + Send + Sync>,
}

impl<T, R> RenderSlotWith<T, R> {
    /// Create a render slot from a closure.
    pub fn new<F>(render: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        Self {
            slot: Slot::from_shared(Arc::new(render)),
        }
    }

    /// Execute the render closure with an input value.
    pub fn render(&self, value: T) -> R {
        let render = self.slot.shared();
        render(value)
    }
}

impl<T, R, F> From<F> for RenderSlotWith<T, R>
where
    F: Fn(T) -> R + Send + Sync + 'static,
{
    fn from(render: F) -> Self {
        Self::new(render)
    }
}

impl<T, R> From<CallbackWith<T, R>> for RenderSlotWith<T, R> {
    fn from(callback: CallbackWith<T, R>) -> Self {
        Self {
            slot: callback.slot,
        }
    }
}

impl<T, R> Clone for RenderSlotWith<T, R> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot.clone(),
        }
    }
}

impl<T, R> PartialEq for RenderSlotWith<T, R> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
    }
}

impl<T, R> Eq for RenderSlotWith<T, R> {}

/// Component props that can be snapshotted and compared for replay.
pub trait Prop: Clone + Send + Sync + 'static {
    /// Compare current props with another props value.
    fn prop_eq(&self, other: &Self) -> bool;
}

impl<T> Prop for T
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    fn prop_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Type-erased prop value used by the core replay path.
pub trait ErasedProp: Send + Sync {
    /// Access the concrete prop value as `Any`.
    fn as_any(&self) -> &dyn Any;
    /// Clone this erased prop object.
    fn clone_box(&self) -> Box<dyn ErasedProp>;
    /// Compare with another erased prop object.
    fn equals(&self, other: &dyn ErasedProp) -> bool;
}

impl<T> ErasedProp for T
where
    T: Prop,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ErasedProp> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn ErasedProp) -> bool {
        let Some(other) = other.as_any().downcast_ref::<T>() else {
            return false;
        };
        self.prop_eq(other)
    }
}

/// Type-erased component runner used for replay.
pub trait ErasedComponentRunner: Send + Sync {
    /// Execute the component with erased props.
    fn run(&self, props: &dyn ErasedProp);
}

struct ComponentRunner<P: Prop> {
    run_fn: fn(&P),
}

impl<P> ErasedComponentRunner for ComponentRunner<P>
where
    P: Prop,
{
    fn run(&self, props: &dyn ErasedProp) {
        let Some(props) = props.as_any().downcast_ref::<P>() else {
            panic!(
                "component runner props type mismatch: expected {}",
                std::any::type_name::<P>()
            );
        };
        (self.run_fn)(props);
    }
}

/// Build a type-erased runner from a component function.
pub fn make_component_runner<P>(run_fn: fn(&P)) -> Arc<dyn ErasedComponentRunner>
where
    P: Prop,
{
    Arc::new(ComponentRunner { run_fn })
}

/// Snapshot of a replayable component invocation.
#[derive(Clone)]
pub struct ComponentReplayData {
    /// Type-erased component runner.
    pub runner: Arc<dyn ErasedComponentRunner>,
    /// Latest props snapshot.
    pub props: Arc<dyn ErasedProp>,
}

impl ComponentReplayData {
    /// Create replay data from typed props.
    pub fn new<P>(runner: Arc<dyn ErasedComponentRunner>, props: &P) -> Self
    where
        P: Prop,
    {
        Self {
            runner,
            props: Arc::new(props.clone()),
        }
    }
}
