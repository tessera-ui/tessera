//! Component prop and replay abstractions.
//!
//! ## Usage
//!
//! Internal prop comparison and replay support used by generated component
//! props.

use std::{any::Any, marker::PhantomData, ptr, sync::Arc};

use crate::{
    runtime::{
        FunctorHandle, invoke_callback_handle, invoke_callback_with_handle,
        invoke_render_slot_handle, invoke_render_slot_with_handle, remember_callback_handle,
        remember_callback_with_handle, remember_render_slot_handle,
        remember_render_slot_with_handle, track_render_slot_read_dependency,
    },
    tessera,
};

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
/// `Callback` is a stable handle to the latest callback closure created at the
/// same build call site.
///
/// Callback identity does not change when the captured closure changes during
/// recomposition. As a result, callback updates do not force prop mismatches or
/// replay invalidation. Event handlers always invoke the latest closure stored
/// behind the handle.
///
/// Create callbacks only during a Tessera component build.
#[derive(Clone, Copy)]
pub struct Callback {
    repr: CallbackRepr,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CallbackRepr {
    Noop,
    Handle(FunctorHandle),
}

impl Callback {
    /// Create a callback handle from a closure.
    ///
    /// This must be called during a component build.
    #[track_caller]
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            repr: CallbackRepr::Handle(remember_callback_handle(handler)),
        }
    }

    /// Create an empty callback.
    pub const fn noop() -> Self {
        Self {
            repr: CallbackRepr::Noop,
        }
    }

    /// Invoke the callback.
    pub fn call(&self) {
        match self.repr {
            CallbackRepr::Noop => {}
            CallbackRepr::Handle(handle) => invoke_callback_handle(handle),
        }
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
        Self::noop()
    }
}

impl PartialEq for Callback {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Eq for Callback {}

/// Stable, comparable callback handle for `Fn(T) -> R`.
///
/// This follows the same stability rules as [`Callback`]: the handle remains
/// stable across recomposition, while calls always observe the latest closure.
///
/// This is useful for value-change handlers and similar one-argument callbacks.
pub struct CallbackWith<T, R = ()> {
    repr: CallbackWithRepr<T, R>,
}

enum CallbackWithRepr<T, R> {
    Handle(FunctorHandle),
    Static(fn(T) -> R),
}

impl<T, R> Copy for CallbackWithRepr<T, R> {}

impl<T, R> Clone for CallbackWithRepr<T, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, R> CallbackWith<T, R>
where
    T: 'static,
    R: 'static,
{
    /// Create a callback handle from a closure.
    ///
    /// This must be called during a component build.
    #[track_caller]
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        Self {
            repr: CallbackWithRepr::Handle(remember_callback_with_handle(handler)),
        }
    }

    /// Invoke the callback with an argument.
    pub fn call(&self, value: T) -> R {
        match self.repr {
            CallbackWithRepr::Handle(handle) => invoke_callback_with_handle(handle, value),
            CallbackWithRepr::Static(handler) => handler(value),
        }
    }

    fn from_static(handler: fn(T) -> R) -> Self {
        Self {
            repr: CallbackWithRepr::Static(handler),
        }
    }
}

impl<T, R, F> From<F> for CallbackWith<T, R>
where
    T: 'static,
    R: 'static,
    F: Fn(T) -> R + Send + Sync + 'static,
{
    fn from(handler: F) -> Self {
        Self::new(handler)
    }
}

impl<T, R> Copy for CallbackWith<T, R> {}

impl<T, R> Clone for CallbackWith<T, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, R> PartialEq for CallbackWith<T, R> {
    fn eq(&self, other: &Self) -> bool {
        match (self.repr, other.repr) {
            (CallbackWithRepr::Handle(lhs), CallbackWithRepr::Handle(rhs)) => lhs == rhs,
            (CallbackWithRepr::Static(lhs), CallbackWithRepr::Static(rhs)) => {
                ptr::fn_addr_eq(lhs, rhs)
            }
            _ => false,
        }
    }
}

impl<T, R> Eq for CallbackWith<T, R> {}

impl<T, R> CallbackWith<T, R>
where
    T: 'static,
    R: Default + 'static,
{
    /// Create a callback that ignores its input and returns
    /// [`Default::default`].
    pub fn default_value() -> Self {
        fn default_value_impl<T, R: Default>(_: T) -> R {
            R::default()
        }

        Self::from_static(default_value_impl::<T, R>)
    }
}

impl<T> CallbackWith<T, T>
where
    T: 'static,
{
    /// Create a callback that returns its input unchanged.
    pub fn identity() -> Self {
        fn identity_impl<T>(value: T) -> T {
            value
        }

        Self::from_static(identity_impl::<T>)
    }
}

/// Stable, comparable render slot handle.
///
/// `RenderSlot` is a stable handle to deferred UI content.
///
/// Like [`Callback`], the handle stays stable across recomposition. Unlike
/// callbacks, updating a render slot's closure invalidates component instances
/// that rendered the slot, so slot content changes can trigger replayed
/// components to rebuild with the latest UI.
///
/// Create render slots only during a Tessera component build.
#[derive(Clone, Copy)]
pub struct RenderSlot {
    repr: RenderSlotRepr,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum RenderSlotRepr {
    Empty,
    Handle(FunctorHandle),
}

impl RenderSlot {
    /// Create a render slot from a closure.
    ///
    /// This must be called during a component build.
    #[track_caller]
    pub fn new<F>(render: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            repr: RenderSlotRepr::Handle(remember_render_slot_handle(render)),
        }
    }

    /// Create an empty render slot.
    pub const fn empty() -> Self {
        Self {
            repr: RenderSlotRepr::Empty,
        }
    }

    /// Execute the render closure.
    pub fn render(&self) {
        match self.repr {
            RenderSlotRepr::Empty => {}
            RenderSlotRepr::Handle(handle) => {
                render_slot_boundary(handle);
            }
        }
    }
}

#[tessera(crate)]
fn render_slot_boundary(handle: FunctorHandle) {
    track_render_slot_read_dependency(handle);
    invoke_render_slot_handle(handle);
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
        Self::new(move || callback.call())
    }
}

impl Default for RenderSlot {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for RenderSlot {
    fn eq(&self, other: &Self) -> bool {
        match (&self.repr, &other.repr) {
            (RenderSlotRepr::Empty, RenderSlotRepr::Empty) => true,
            (RenderSlotRepr::Handle(lhs), RenderSlotRepr::Handle(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Eq for RenderSlot {}

/// Stable, comparable render slot handle for `Fn(T)`.
///
/// This follows the same invalidation rules as [`RenderSlot`], while supporting
/// deferred rendering that depends on an input value.
pub struct RenderSlotWith<T> {
    handle: FunctorHandle,
    marker: PhantomData<fn(T)>,
}

impl<T> RenderSlotWith<T> {
    /// Create a render slot from a closure.
    ///
    /// This must be called during a component build.
    #[track_caller]
    pub fn new<F>(render: F) -> Self
    where
        T: 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        Self {
            handle: remember_render_slot_with_handle(render),
            marker: PhantomData,
        }
    }

    /// Execute the render closure with an input value.
    pub fn render(&self, value: T)
    where
        T: Clone + PartialEq + Send + Sync + 'static,
    {
        render_slot_with_boundary(self.handle, value);
    }
}

#[tessera(crate)]
fn render_slot_with_boundary<T>(handle: FunctorHandle, value: T)
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    track_render_slot_read_dependency(handle);
    invoke_render_slot_with_handle(handle, value)
}

impl<T, F> From<F> for RenderSlotWith<T>
where
    T: 'static,
    F: Fn(T) + Send + Sync + 'static,
{
    fn from(render: F) -> Self {
        Self::new(render)
    }
}

impl<T> From<CallbackWith<T>> for RenderSlotWith<T>
where
    T: 'static,
{
    fn from(callback: CallbackWith<T>) -> Self {
        Self::new(move |value| {
            callback.call(value);
        })
    }
}

impl<T> Clone for RenderSlotWith<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RenderSlotWith<T> {}

impl<T> PartialEq for RenderSlotWith<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<T> Eq for RenderSlotWith<T> {}

/// Internal component props contract used by replay and prop comparison.
pub trait Prop: Clone + Send + Sync + 'static {
    /// Compare current props with another props value.
    fn prop_eq(&self, other: &Self) -> bool;
}

impl Prop for () {
    fn prop_eq(&self, _other: &Self) -> bool {
        true
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
