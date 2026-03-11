use std::ptr::NonNull;

use crate::execution_context::{with_execution_context, with_execution_context_mut};

use super::{CompositionRuntime, TesseraRuntime};

pub(crate) fn with_bound_runtime<R>(f: impl FnOnce(&TesseraRuntime) -> R) -> Option<R> {
    with_execution_context(|context| {
        let ptr = context.current_runtime_stack.last().copied()?;
        // SAFETY: The binding guard only stores a pointer that remains valid for
        // the duration of the guarded call on the current thread.
        Some(unsafe { f(ptr.as_ref()) })
    })
}

pub(crate) fn with_bound_runtime_mut<R>(f: impl FnOnce(&mut TesseraRuntime) -> R) -> Option<R> {
    with_execution_context(|context| {
        let ptr = context.current_runtime_stack.last().copied()?;
        // SAFETY: The binding guard only stores a pointer that remains valid for
        // the duration of the guarded call on the current thread.
        Some(unsafe { f(&mut *ptr.as_ptr()) })
    })
}

fn with_bound_composition_runtime<R>(f: impl FnOnce(&CompositionRuntime) -> R) -> Option<R> {
    with_execution_context(|context| {
        let ptr = context.current_composition_runtime_stack.last().copied()?;
        // SAFETY: The binding guard only stores a pointer that remains valid for
        // the duration of the guarded call on the current thread.
        Some(unsafe { f(ptr.as_ref()) })
    })
}

pub(crate) fn with_composition_runtime<R>(f: impl FnOnce(&CompositionRuntime) -> R) -> R {
    let mut f = Some(f);
    if let Some(result) = with_bound_composition_runtime(|runtime| {
        f.take()
            .expect("composition runtime callback should run once")(runtime)
    }) {
        return result;
    }
    if let Some(result) = with_bound_runtime(|runtime| {
        f.take()
            .expect("composition runtime callback should run once")(
            runtime.composition.as_ref()
        )
    }) {
        return result;
    }
    panic!("composition runtime requires an active Tessera runtime session")
}

#[must_use = "composition runtime binding guards must be kept alive for the bound execution scope"]
pub(crate) struct CompositionRuntimeBindingGuard {
    active: bool,
}

impl Drop for CompositionRuntimeBindingGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        with_execution_context_mut(|context| {
            let popped = context.current_composition_runtime_stack.pop();
            debug_assert!(
                popped.is_some(),
                "composition runtime binding stack underflow"
            );
        });
        self.active = false;
    }
}

fn bind_current_composition_runtime(
    runtime: &CompositionRuntime,
) -> CompositionRuntimeBindingGuard {
    with_execution_context_mut(|context| {
        context
            .current_composition_runtime_stack
            .push(NonNull::from(runtime));
    });
    CompositionRuntimeBindingGuard { active: true }
}

#[must_use = "runtime binding guards must be kept alive for the bound execution scope"]
pub(crate) struct RuntimeBindingGuard {
    active: bool,
    _composition_guard: CompositionRuntimeBindingGuard,
}

impl Drop for RuntimeBindingGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        with_execution_context_mut(|context| {
            let popped = context.current_runtime_stack.pop();
            debug_assert!(popped.is_some(), "runtime binding stack underflow");
        });
        self.active = false;
    }
}

pub(crate) fn bind_current_runtime(runtime: &mut TesseraRuntime) -> RuntimeBindingGuard {
    with_execution_context_mut(|context| {
        context
            .current_runtime_stack
            .push(NonNull::from(&mut *runtime));
    });
    let composition_guard = bind_current_composition_runtime(runtime.composition.as_ref());
    RuntimeBindingGuard {
        active: true,
        _composition_guard: composition_guard,
    }
}
