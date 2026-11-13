//! Component prop and replay abstractions.
//!
//! ## Usage
//!
//! Internal prop comparison and replay support used by generated component
//! props.

use std::{any::Any, marker::PhantomData, ptr, rc::Rc};

use crate::{
    runtime::{
        FunctorHandle, invoke_callback_handle, invoke_callback_with_handle,
        invoke_render_slot_handle, invoke_render_slot_with_handle, remember_callback_handle,
        remember_callback_with_handle, remember_render_slot_handle,
        remember_render_slot_with_handle, track_render_slot_read_dependency,
    },
    tessera,
};

/// Stable, comparable callback handle for `Fn()`.
///
/// Callback identity remains stable across recomposition. Capture [`State`](crate::State)
/// handles and read their values when the callback runs to observe ongoing
/// changes. Pass the handle through component props or attach it to an action.
///
/// Create callbacks during a Tessera component build.
#[derive(Clone, Copy, Eq, PartialEq)]
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
        F: Fn() + 'static,
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
    F: Fn() + 'static,
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

/// Stable, comparable callback handle for `Fn(T) -> R`.
///
/// The handle remains stable across recomposition. Capture [`State`](crate::State) handles
/// and read their values when the callback runs to observe ongoing changes.
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
        F: Fn(T) -> R + 'static,
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
    F: Fn(T) -> R + 'static,
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

/// Owned UI content expanded by its parent during a Tessera build.
///
/// The parent decides whether, when, and where content expands. Captures must
/// be owned and static; capture stable state handles for changing data.
///
/// Stored content owns its captures. Move local values into its closure:
///
/// ```compile_fail
/// use tessera_ui::{RenderSlot, tessera};
///
/// #[tessera]
/// fn host(child: Option<RenderSlot>) {}
///
/// #[tessera]
/// fn app() {
///     let label = String::from("Title");
///     host().child(|| { let _ = label.len(); });
/// }
/// ```
///
/// A configuration handle cannot be used in place of deferred child content:
///
/// ```compile_fail
/// use tessera_ui::{RenderSlot, tessera};
///
/// #[tessera]
/// fn host(child: Option<RenderSlot>) {}
///
/// #[tessera]
/// fn leaf() {}
///
/// #[tessera]
/// fn app() {
///     host().child(leaf());
/// }
/// ```
///
#[derive(Clone, Copy, Eq, PartialEq)]
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
    pub fn new<F, Output>(render: F) -> Self
    where
        F: Fn() -> Output + 'static,
    {
        Self {
            repr: RenderSlotRepr::Handle(remember_render_slot_handle(move || {
                render();
            })),
        }
    }

    /// Create an empty render slot.
    pub const fn empty() -> Self {
        Self {
            repr: RenderSlotRepr::Empty,
        }
    }

    /// Expands the content at the current build position.
    ///
    /// Call this only from a component or recognized build closure.
    pub fn render(&self) {
        match self.repr {
            RenderSlotRepr::Empty => {}
            RenderSlotRepr::Handle(handle) => {
                render_slot_boundary(handle).props.execute_now();
            }
        }
    }
}

#[tessera]
fn render_slot_boundary(handle: FunctorHandle) {
    track_render_slot_read_dependency(handle);
    invoke_render_slot_handle(handle);
}

impl<F, Output> From<F> for RenderSlot
where
    F: Fn() -> Output + 'static,
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
    pub fn new<F, Output>(render: F) -> Self
    where
        T: 'static,
        F: Fn(T) -> Output + 'static,
    {
        Self {
            handle: remember_render_slot_with_handle(move |value| {
                render(value);
            }),
            marker: PhantomData,
        }
    }

    /// Expands the content with an input at the current build position.
    ///
    /// Call this only from a component or recognized build closure.
    pub fn render(&self, value: T)
    where
        T: Clone + PartialEq + 'static,
    {
        render_slot_with_boundary(self.handle, value)
            .props
            .execute_now();
    }
}

#[tessera]
fn render_slot_with_boundary<T>(handle: FunctorHandle, value: T)
where
    T: Clone + PartialEq + 'static,
{
    track_render_slot_read_dependency(handle);
    invoke_render_slot_with_handle(handle, value)
}

impl<T, F, Output> From<F> for RenderSlotWith<T>
where
    T: 'static,
    F: Fn(T) -> Output + 'static,
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
pub trait Prop: Clone + 'static {
    /// Compare current props with another props value.
    fn prop_eq(&self, other: &Self) -> bool;
    /// Execute the component with these props.
    fn run_component(props: ComponentProps<Self>);
}

/// Props of one component execution: freshly declared, or shared with the
/// replay snapshot.
#[doc(hidden)]
pub enum ComponentProps<P> {
    Owned(P),
    Shared(Rc<P>),
}

impl<P> ComponentProps<P> {
    pub(crate) fn into_shared(self) -> Rc<P> {
        match self {
            Self::Owned(props) => Rc::new(props),
            Self::Shared(props) => props,
        }
    }
}

impl<P> std::ops::Deref for ComponentProps<P> {
    type Target = P;

    fn deref(&self) -> &P {
        match self {
            Self::Owned(props) => props,
            Self::Shared(props) => props,
        }
    }
}

pub(crate) trait ErasedComponent {
    fn as_any(&self) -> &dyn Any;
    fn equals(&self, other: &dyn ErasedComponent) -> bool;
    fn run(self: Rc<Self>);
}

impl<T> ErasedComponent for T
where
    T: Prop,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn ErasedComponent) -> bool {
        let Some(other) = other.as_any().downcast_ref::<T>() else {
            return false;
        };
        self.prop_eq(other)
    }

    fn run(self: Rc<Self>) {
        T::run_component(ComponentProps::Shared(self));
    }
}

/// Declared props taken out of the pending table for one execution.
pub(crate) trait PendingProps {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn run(&mut self);
}

impl<P: Prop> PendingProps for Option<P> {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.as_mut().expect("pending props")
    }

    fn run(&mut self) {
        P::run_component(ComponentProps::Owned(self.take().expect("pending props")));
    }
}
