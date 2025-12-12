//! Ambient context values shared with descendants during component build.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap as StdHashMap,
    marker::PhantomData,
    sync::Arc,
};

use im::HashMap;

use crate::runtime::ensure_build_phase;

type ContextMap = HashMap<TypeId, u64>;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextMap>> = RefCell::new(vec![ContextMap::new()]);
    static CONTEXT_STORAGE: RefCell<StdHashMap<u64, Arc<dyn Any + Send + Sync>>> =
        RefCell::new(StdHashMap::new());
    static CONTEXT_ID_COUNTER: RefCell<u64> = const { RefCell::new(0) };
}

/// Read-only handle to a context value created by [`provide_context`] and
/// retrieved by [`use_context`].
///
/// # Examples
///
/// ```
/// use tessera_ui::{provide_context, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     color: String,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         Theme {
///             color: "blue".into(),
///         },
///         || {
///             child();
///         },
///     );
/// }
///
/// #[tessera]
/// fn child() {
///     let theme = use_context::<Theme>();
///     theme.with(|t| println!("Color: {}", t.color));
/// }
/// ```
pub struct Context<T: Default> {
    context_id: u64,
    _marker: PhantomData<T>,
}

impl<T: Default> Copy for Context<T> {}

impl<T: Default> Clone for Context<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Default> Context<T> {
    fn new(context_id: u64) -> Self {
        Self {
            context_id,
            _marker: PhantomData,
        }
    }
}

impl<T: Default> Context<T>
where
    T: Send + Sync + 'static,
{
    /// Execute a closure with a shared reference to the context value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let Some(arc) = get_context_value(self.context_id) else {
            return f(&T::default());
        };

        let typed_arc = arc.downcast::<T>().unwrap_or_else(|_| {
            panic!(
                "Context type mismatch for entry {}: expected {}",
                self.context_id,
                std::any::type_name::<T>()
            )
        });

        f(&typed_arc)
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }
}

fn next_context_id() -> u64 {
    CONTEXT_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        let id = *counter;
        *counter = counter.wrapping_add(1);
        id
    })
}

fn store_context_value(value: Arc<dyn Any + Send + Sync>) -> u64 {
    let id = next_context_id();
    CONTEXT_STORAGE.with(|storage| {
        storage.borrow_mut().insert(id, value);
    });
    id
}

pub(crate) fn get_context_value(id: u64) -> Option<Arc<dyn Any + Send + Sync>> {
    CONTEXT_STORAGE.with(|storage| storage.borrow().get(&id).cloned())
}

pub(crate) fn clear_context() {
    CONTEXT_STORAGE.with(|storage| storage.borrow_mut().clear());
    CONTEXT_ID_COUNTER.with(|counter| *counter.borrow_mut() = 0);
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.clear();
        stack.push(ContextMap::new());
    });
}

fn push_context_layer(type_id: TypeId, context_id: u64) {
    CONTEXT_STACK.with(|stack| {
        let mut stack: std::cell::RefMut<'_, Vec<HashMap<TypeId, u64>>> = stack.borrow_mut();
        let parent = stack.last().cloned().unwrap_or_else(ContextMap::new);
        let next = parent.update(type_id, context_id);
        stack.push(next);
    });
}

fn pop_context_layer() {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let popped = stack.pop();
        debug_assert!(popped.is_some(), "Context stack underflow");
        if stack.is_empty() {
            stack.push(ContextMap::new());
        }
    });
}

/// Provides a typed context value for the duration of the given closure.
///
/// This is intended for use inside component build functions only.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, provide_context, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn root() {
///     provide_context(
///         Theme {
///             primary: Color::RED,
///         },
///         || {
///             leaf();
///         },
///     );
/// }
///
/// #[tessera]
/// fn leaf() {
///     let theme = use_context::<Theme>();
///     theme.with(|t| assert_eq!(t.primary, Color::RED));
/// }
/// ```
pub fn provide_context<T, F, R>(value: T, f: F) -> R
where
    T: Send + Sync + 'static,
    F: FnOnce() -> R,
{
    ensure_build_phase();

    // Store value in HashMap and get context ID
    let context_id = store_context_value(Arc::new(value));

    // Push UUID to stack
    push_context_layer(TypeId::of::<T>(), context_id);

    struct ContextScopeGuard;
    impl Drop for ContextScopeGuard {
        fn drop(&mut self) {
            pop_context_layer();
        }
    }
    let guard = ContextScopeGuard;
    let result = f();
    drop(guard);
    result
}

/// Reads a typed context value from the current scope, falling back to
/// Default value when parent did not provide one.
///
/// # Examples
///
/// ```
/// use tessera_ui::{Color, tessera, use_context};
///
/// #[derive(Default)]
/// struct Theme {
///     primary: Color,
/// }
///
/// #[tessera]
/// fn component() {
///     let theme = use_context::<Theme>(); // Default used when no provider
///     theme.with(|t| assert_eq!(t.primary, Color::default()));
/// }
/// ```
pub fn use_context<T>() -> Context<T>
where
    T: Default + Send + Sync + 'static,
{
    ensure_build_phase();

    let context_id = CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let map = stack
            .last_mut()
            .expect("Context stack must always contain at least one layer");

        if let Some(&id) = map.get(&TypeId::of::<T>()) {
            id
        } else {
            // No context provided, create and cache default value
            let id = store_context_value(Arc::new(T::default()));
            *map = map.clone().update(TypeId::of::<T>(), id);
            id
        }
    });

    Context::new(context_id)
}
