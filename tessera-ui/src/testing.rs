//! Tessera testing helpers for runtime-sensitive tests and documentation
//! examples.
//!
//! ## Usage
//!
//! Wrap examples and tests that need an isolated Tessera session before calling
//! runtime-sensitive APIs. When a build context is also required, declare a
//! hidden `#[tessera]` component inside the closure and call it there. In
//! rustdoc, prefer hidden setup lines (`# ...`) so examples stay focused on the
//! public API being demonstrated. For synchronous component tests, prefer
//! [`#[tessera_ui::test]`](crate::test) and keep `with_tessera(...)` for
//! documentation and ad-hoc runtime harnessing.

use crate::runtime::{TesseraRuntime, bind_current_runtime};

/// # with_tessera
///
/// Run a closure inside an isolated Tessera runtime session for tests and
/// documentation examples.
///
/// ## Usage
///
/// Wrap examples that need a Tessera session. If the example also needs build
/// semantics, declare and call a hidden `#[tessera]` component inside the
/// closure. In rustdoc examples, prefer hidden setup lines (`# ...`) for the
/// `with_tessera(...)` wrapper and helper component declarations. For
/// synchronous component tests, prefer [`#[tessera_ui::test]`](crate::test).
///
/// ## Parameters
///
/// - `f` — closure executed inside an isolated Tessera testing session
///
/// ## Examples
///
/// ```rust
/// use tessera_ui::testing::with_tessera;
///
/// with_tessera(|| {
///     // Runtime-only assertions can run directly inside the testing session.
/// });
/// ```
///
/// ```rust
/// use tessera_ui::testing::with_tessera;
/// use tessera_ui::{Callback, tessera};
///
/// with_tessera(|| {
///     #[tessera]
///     fn demo() {
///         let callback = Callback::new(|| {});
///         callback.call();
///     }
///
///     demo();
/// });
/// ```
pub fn with_tessera<R>(f: impl FnOnce() -> R) -> R {
    let mut runtime = Box::new(TesseraRuntime::default());
    let _binding = bind_current_runtime(runtime.as_mut());
    f()
}

#[cfg(test)]
mod tests {
    #[crate::test(crate)]
    fn tessera_test_runs_inside_build_context() {
        let count = crate::remember(|| 1);
        assert_eq!(count.get(), 1);
        let callback = crate::Callback::new(|| {});
        callback.call();
    }

    #[crate::test(crate)]
    #[should_panic(expected = "boom")]
    fn tessera_test_preserves_test_attributes() {
        panic!("boom");
    }
}
