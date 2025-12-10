//! tessera is a cross-platform UI library focused on performance and extensibility.
//!
//! # Guide
//!
//! We recommend reading the [Quick Start](https://tessera-ui.github.io/guide/getting-started.html) to learn how to use tessera.
//!
//! # Component Library
//!
//! The tessera-ui crate itself does not contain built-in components, but the official project provides a [basic component library](https://crates.io/crates/tessera-ui-basic-components).
//!
//! It contains commonly used UI components such as buttons, text boxes, labels, etc., which help developers quickly build user interfaces.
//!
//! # Components
//!
//! To define a component in tessera, write it like this:
//!
//! ```
//! use tessera_ui::tessera;
//!
//! #[tessera]
//! fn my_component() {
//!     // component implementation
//! }
//! ```
//!
//! Functions marked with the `#[tessera]` macro are tessera components.
//!
//! Component functions may contain other component functions, enabling nesting and composition.
//!
//! ```
//! use tessera_ui::tessera;
//!
//! #[tessera]
//! fn child() {
//!     // child component implementation
//! }
//!
//! #[tessera]
//! fn parent() {
//!     child();
//! }
//! ```
//!
//! # Memoized State
//!
//! The `remember` and `remember_with_key` functions can be used to create persistent state across frames within a component.
//!
//! ```
//! use std::sync::atomic::AtomicUsize;
//! use tessera_ui::{remember, tessera};
//!
//! #[tessera]
//! fn counter() {
//!     let mut count = remember(|| AtomicUsize::new(0));
//! }
//! ```
//!
//! Memoized state is implemented via macro-based control-flow analysis and cannot be used outside of functions marked with `#[tessera]`. It also must not be used inside measurement closures or event handler implementations.
//!
//! `remember` handles most control flow situations, but it cannot guarantee stable identity inside loops. If you need to use memoized state within a loop, use `remember_with_key` and provide a stable key.
//!
//! ```
//! use std::sync::atomic::AtomicUsize;
//! use tessera_ui::{tessera, remember_with_key};
//!
//! struct User {
//!     id: i32,
//!     name: String,
//! }
//!
//! #[tessera]
//! fn user_list() {
//!     let users = vec![
//!         User { id: 101, name: "Alice".to_string() },
//!         User { id: 205, name: "Bob".to_string() },
//!         User { id: 33,  name: "Charlie".to_string() },
//!     ];
//!
//!     for user in users.iter() {
//!         // Regardless of the user's position in the list, this `likes` state will follow the user.id
//!         let likes = remember_with_key(user.id, || AtomicUsize::new(0));
//!
//!         /* component implementation */
//!     }
//! }
//! ```
//!
//! Or use the `key` function to influence the `remember` calls inside it.
//!
//! ```
//! use std::sync::atomic::AtomicUsize;
//! use tessera_ui::{key, remember, tessera};
//!
//! #[tessera]
//! fn my_list(items: Vec<String>) {
//!     for item in items {
//!         key(item.clone(), || {
//!             let state = remember(|| AtomicUsize::new(0));
//!         });
//!     }
//! }
//! ```
//!
//! This is equivalent to using `remember_with_key(item.clone(), || AtomicUsize::new(0))`, but it
//! is transparent to child components and necessary for virtual container-like components.
//!
//! However, `remember_with_key` is a litte cheaper than `key` + `remember`, so prefer it
//! in simple cases.
//!
//! # Layout
//!
//! Implement a measure closure to define a component's layout behavior.
//!
//! ```
//! use tessera_ui::tessera;
//!
//! #[tessera]
//! fn component() {
//!     measure(Box::new(|input| {
//!         // measurement and layout implementation
//!         # Ok(tessera_ui::ComputedData {
//!         #     width: tessera_ui::Px::ZERO,
//!         #       height: tessera_ui::Px::ZERO,
//!         # })
//!     }));
//! }
//! ```
//!
//! For more details, see the [Layout Guide](https://tessera-ui.github.io/guide/component.html#measure-place).
#![deny(missing_docs, clippy::unwrap_used)]

pub mod accessibility;
pub mod clipboard;
pub mod color;
mod component_tree;
pub mod context;
mod cursor;
pub mod dp;
pub mod dyn_eq;
pub mod dyn_eq_compute;
pub mod focus_state;
mod ime_state;
mod keyboard_state;
pub(crate) mod pipeline_cache;
pub mod px;
pub mod renderer;
#[doc(hidden)]
pub mod runtime;
mod thread_utils;

#[cfg(feature = "shard")]
pub mod router;

pub use accesskit;
pub use indextree::{Arena, NodeId};
pub use tessera_ui_macros::tessera;
pub use wgpu;
pub use winit;

pub use crate::{
    accessibility::{AccessibilityActionHandler, AccessibilityId, AccessibilityNode},
    clipboard::Clipboard,
    color::Color,
    component_tree::{
        ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree,
        ComponentTree, ComputedData, Constraint, DimensionValue, ImeRequest, InputHandlerFn,
        InputHandlerInput, MeasureFn, MeasureInput, MeasurementError,
    },
    context::{provide_context, use_context},
    cursor::{CursorEvent, CursorEventContent, GestureState, PressKeyEventType, ScrollEventConent},
    dp::Dp,
    focus_state::Focus,
    px::{Px, PxPosition, PxRect, PxSize},
    renderer::{
        BarrierRequirement, Command, Renderer,
        compute::{
            self, ComputablePipeline, ComputeCommand, ComputePipelineRegistry, ComputeResource,
            ComputeResourceManager, ComputeResourceRef,
        },
        drawer::{self, DrawCommand, DrawablePipeline, PipelineRegistry, command},
    },
    runtime::{key, remember, remember_with_key},
};

use ime_state::ImeState;

#[cfg(feature = "shard")]
pub use tessera_ui_macros::shard;
#[cfg(feature = "shard")]
pub use tessera_ui_shard;
