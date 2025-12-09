//! Ambient context values shared with descendants during component build.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    sync::Arc,
};

use im::HashMap;

use crate::runtime::ensure_build_phase;

type ContextMap = HashMap<TypeId, Arc<dyn Any + Send + Sync>>;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextMap>> = RefCell::new(vec![ContextMap::new()]);
}

fn push_context_layer(type_id: TypeId, value: Arc<dyn Any + Send + Sync>) {
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let parent = stack.last().cloned().unwrap_or_else(ContextMap::new);
        let mut next = parent;
        next.insert(type_id, value);
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
/// use tessera_ui::{Color, provide_context, tessera};
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
///     let theme = tessera_ui::use_context::<Theme>();
///     assert_eq!(theme.primary, Color::RED);
/// }
/// ```
pub fn provide_context<T, F, R>(value: T, f: F) -> R
where
    T: Send + Sync + 'static,
    F: FnOnce() -> R,
{
    ensure_build_phase();
    push_context_layer(TypeId::of::<T>(), Arc::new(value));
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

/// Reads a typed context value from the current scope, falling back to `T::default()` when missing.
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
///     assert_eq!(theme.primary, Color::default());
/// }
/// ```
pub fn use_context<T>() -> Arc<T>
where
    T: Default + Send + Sync + 'static,
{
    ensure_build_phase();
    CONTEXT_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let map = stack
            .last_mut()
            .expect("Context stack must always contain at least one layer");
        if let Some(value) = map.get(&TypeId::of::<T>()) {
            return Arc::downcast::<T>(value.clone())
                .expect("Context type mismatch; multiple types mapped to same TypeId");
        }
        let value = Arc::new(T::default());
        map.insert(TypeId::of::<T>(), value.clone());
        value
    })
}
