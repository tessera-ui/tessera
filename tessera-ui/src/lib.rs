//! Tessera is a cross-platform declarative & functional UI library for rust,
//! focused on performance and extensibility.
//!
//! # Guide
//!
//! We recommend reading the [Quick Start](https://tessera-ui.github.io/guide/getting-started.html) to learn how to use tessera.
//!
//! # Component Library
//!
//! The tessera-ui crate itself does not contain built-in components, but the official project provides a [basic component library](https://crates.io/crates/tessera-components).
//!
//! It contains commonly used UI components such as buttons, text boxes, labels,
//! etc., which help developers quickly build user interfaces.
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
//! Component functions may contain other component functions, enabling nesting
//! and composition.
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
//! # Memoized state
//!
//! Components in tessera are functions. To persist state across frames within a
//! component, use the memoization API.
//!
//! There are two primary primitives for memoized state depending on lifetime:
//! `remember` and `retain`. The following sections describe their behavior and
//! usage.
//!
//! ## remember
//!
//! The `remember` and `remember_with_key` functions can be used to create
//! persistent state across frames within a component.
//!
//! ```
//! use tessera_ui::{remember, tessera};
//!
//! #[tessera]
//! fn counter() {
//!     let count = remember(|| 0);
//!     count.with_mut(|c| *c += 1);
//! }
//! ```
//!
//! Memoized state is implemented via macro-based control-flow analysis and
//! cannot be used outside of functions marked with `#[tessera]`. It also must
//! not be used inside layout specs or event handler implementations.
//!
//! `remember` handles most control flow situations, but it cannot guarantee
//! stable identity inside loops. If you need to use memoized state within a
//! loop, use `remember_with_key` and provide a stable key.
//!
//! ```
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
//!         let likes = remember_with_key(user.id, || 0);
//!
//!         /* component implementation */
//!     }
//! }
//! ```
//!
//! Or use the `key` function to influence the `remember` calls inside it.
//!
//! ```
//! use tessera_ui::{key, remember, tessera};
//!
//! #[derive(Clone, PartialEq)]
//! struct MyListArgs {
//!     items: Vec<String>,
//! }
//!
//! #[tessera]
//! fn my_list(args: &MyListArgs) {
//!     for item in args.items.iter() {
//!         key(item.clone(), || {
//!             let state = remember(|| 0);
//!         });
//!     }
//! }
//! ```
//!
//! This is equivalent to using `remember_with_key(item.clone(), || 0)`, but it
//! is transparent to child components and necessary for virtual container-like
//! components.
//!
//! However, `remember_with_key` is a litte cheaper than `key` + `remember`, so
//! prefer it in simple cases.
//!
//! ## retain
//!
//! `retain` just work as `remember` but is not dropped when a component becomes
//! invisible; use it for state that should persist for the lifetime of the
//! process (for example, scroll position).
//!
//! It has the same API and typical usage as `remember`, except that its value
//! is retained across the entire lifetime of the process.
//!
//! ```
//! use tessera_ui::{retain, tessera};
//!
//! #[derive(Clone, PartialEq)]
//! struct ScrollablePageArgs {
//!     page_id: String,
//! }
//!
//! #[tessera]
//! fn scrollable_page(_args: &ScrollablePageArgs) {
//!     // Scroll position persists even when navigating away and returning
//!     let scroll_offset = retain(|| 0.0f32);
//!
//!     /* component implementation */
//! }
//! ```
//!
//! There is also a `key` variant for `retain`, called `retain_with_key`. Use it
//! when you need to retain state in a loop or similar scenarios.
//!
//! ```
//! use tessera_ui::{tessera, retain_with_key};
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
//!         let likes = retain_with_key(user.id, || 0);
//!
//!         /* component implementation */
//!     }
//! }
//! ```
//!
//! Or use the `key` function to influence the `retain` calls inside it.
//!
//! ```
//! use tessera_ui::{key, retain, tessera};
//!
//! #[derive(Clone, PartialEq)]
//! struct MyListArgs {
//!     items: Vec<String>,
//! }
//!
//! #[tessera]
//! fn my_list(args: &MyListArgs) {
//!     for item in args.items.iter() {
//!         key(item.clone(), || {
//!             let state = retain(|| 0);
//!         });
//!     }
//! }
//! ```
//!
//! # Context
//!
//! The context mechanism is used to pass data down the component tree, avoiding
//! the need to thread it through parameters.
//!
//! ```
//! use tessera_ui::{Color, provide_context, tessera, use_context};
//!
//! #[derive(Clone, PartialEq)]
//! struct Theme {
//!     color: Color,
//! }
//!
//! #[tessera]
//! fn parent() {
//!     provide_context(
//!         || Theme { color: Color::RED },
//!         || {
//!             child();
//!         },
//!     );
//! }
//!
//! #[tessera]
//! fn child() {
//!     let theme = use_context::<Theme>().expect("Theme must be provided");
//!     theme.with(|t| assert_eq!(t.color, Color::RED));
//! }
//! ```
//!
//! A context corresponds to a type. In the component tree, a component will
//! receive the nearest parent-provided context of the same type.
//! ```
//! use tessera_ui::{Color, provide_context, tessera, use_context};
//!
//! #[derive(Clone, PartialEq)]
//! struct Theme {
//!     color: Color,
//! }
//!
//! #[tessera]
//! fn parent() {
//!     provide_context(
//!         || Theme { color: Color::RED },
//!         || {
//!             child();
//!         },
//!     );
//! }
//!
//! #[tessera]
//! fn child() {
//!     let theme = use_context::<Theme>().expect("Theme must be provided");
//!     theme.with(|t| assert_eq!(t.color, Color::RED));
//!     provide_context(
//!         || Theme {
//!             color: Color::GREEN,
//!         },
//!         || {
//!             grandchild();
//!         },
//!     );
//! }
//!
//! #[tessera]
//! fn grandchild() {
//!     let theme = use_context::<Theme>().expect("Theme must be provided");
//!     theme.with(|t| assert_eq!(t.color, Color::GREEN));
//! }
//! ```
//!
//! # Layout
//!
//! Implement a layout spec to define a component's layout behavior.
//! ```
//! use tessera_ui::{
//!     Constraint, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px, PxPosition,
//!     tessera,
//! };
//!
//! #[derive(Clone, PartialEq)]
//! struct DefaultLayout;
//!
//! impl LayoutSpec for DefaultLayout {
//!     fn measure(
//!         &self,
//!         input: &LayoutInput<'_>,
//!         output: &mut LayoutOutput<'_>,
//!     ) -> Result<tessera_ui::ComputedData, MeasurementError> {
//!         let parent_constraint = Constraint::new(
//!             input.parent_constraint().width(),
//!             input.parent_constraint().height(),
//!         );
//!         if input.children_ids().is_empty() {
//!             return Ok(tessera_ui::ComputedData::min_from_constraint(
//!                 &parent_constraint,
//!             ));
//!         }
//!         let nodes_to_measure = input
//!             .children_ids()
//!             .iter()
//!             .map(|&child_id| (child_id, parent_constraint))
//!             .collect();
//!         let sizes = input.measure_children(nodes_to_measure)?;
//!         let mut final_width = Px(0);
//!         let mut final_height = Px(0);
//!         for (child_id, size) in sizes {
//!             output.place_child(child_id, PxPosition::ZERO);
//!             final_width = final_width.max(size.width);
//!             final_height = final_height.max(size.height);
//!         }
//!         Ok(tessera_ui::ComputedData {
//!             width: final_width,
//!             height: final_height,
//!         })
//!     }
//! }
//!
//! #[tessera]
//! fn component() {
//!     layout(DefaultLayout);
//! }
//! ```
//!
//! For more details, see the [Layout Guide](https://tessera-ui.github.io/guide/component.html#layout).
#![deny(missing_docs, clippy::unwrap_used)]

pub mod accessibility;
#[cfg(target_os = "android")]
pub mod android;
pub mod color;
mod component_tree;
pub mod context;
mod cursor;
pub mod dp;
pub mod entry_point;
pub mod entry_registry;
pub mod focus_state;
mod ime_state;
mod keyboard_state;
pub mod layout;
pub mod modifier;
pub(crate) mod pipeline_cache;
pub mod pipeline_context;
pub mod plugin;
#[cfg(feature = "profiling")]
pub mod profiler;
#[doc(hidden)]
pub mod prop;
pub mod px;
mod render_graph;
pub mod render_module;
mod render_pass;
pub mod render_scene;
pub mod renderer;
#[doc(hidden)]
pub mod runtime;
mod thread_utils;

#[cfg(feature = "shard")]
pub mod router;

pub use accesskit;
pub use indextree::{Arena, NodeId};
pub use tessera_macros::{entry, tessera};
pub use wgpu;
pub use winit;

pub use crate::{
    accessibility::{AccessibilityActionHandler, AccessibilityId, AccessibilityNode},
    color::Color,
    component_tree::{
        ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree,
        ComponentTree, ComputedData, Constraint, DimensionValue, ImeRequest, InputHandlerFn,
        InputHandlerInput, MeasurementError, ParentConstraint, WindowAction, WindowRequests,
    },
    context::{Context, provide_context, use_context},
    cursor::{
        CursorEvent, CursorEventContent, GestureState, PressKeyEventType, ScrollEventContent,
        ScrollEventSource,
    },
    dp::Dp,
    entry_point::EntryPoint,
    entry_registry::{EntryRegistry, TesseraPackage},
    focus_state::Focus,
    layout::{DefaultLayoutSpec, LayoutInput, LayoutOutput, LayoutResult, LayoutSpec, RenderInput},
    modifier::{Modifier, ModifierChild, ModifierWrapper},
    pipeline_context::PipelineContext,
    plugin::{
        Plugin, PluginContext, PluginResult, register_plugin, register_plugin_boxed, with_plugin,
        with_plugin_mut,
    },
    prop::{Callback, CallbackWith, Prop, RenderSlot, RenderSlotWith, Slot},
    px::{Px, PxPosition, PxRect, PxSize},
    render_graph::{
        ExternalTextureDesc, RenderFragment, RenderFragmentOp, RenderGraph, RenderGraphOp,
        RenderGraphParts, RenderResource, RenderResourceId, RenderTextureDesc,
    },
    render_module::RenderModule,
    render_scene::{Command, CompositeCommand, DrawRegion, PaddingRect, SampleRegion},
    renderer::{
        Renderer,
        composite::{
            self, CompositeBatchItem, CompositeContext, CompositeOutput, CompositePipeline,
            CompositePipelineRegistry, CompositeReplacement,
        },
        compute::{
            self, ComputablePipeline, ComputeCommand, ComputePipelineRegistry, ComputeResource,
            ComputeResourceManager, ComputeResourceRef,
        },
        drawer::{self, DrawCommand, DrawablePipeline, PipelineRegistry, command},
        external::{ExternalTextureHandle, ExternalTextureRegistry},
    },
    runtime::{
        FrameNanosControl, State, current_frame_time, frame_delta, key, receive_frame_nanos,
        remember, remember_with_key, retain, retain_with_key,
    },
};

use ime_state::ImeState;

#[cfg(feature = "shard")]
pub use tessera_macros::shard;
#[cfg(feature = "shard")]
pub use tessera_shard;
