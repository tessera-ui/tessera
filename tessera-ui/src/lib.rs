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
//! #[tessera]
//! fn my_list(items: Vec<String>) {
//!     for item in items {
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
//! #[tessera]
//! fn scrollable_page(page_id: &str) {
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
//! #[tessera]
//! fn my_list(items: Vec<String>) {
//!     for item in items {
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
//! #[derive(Default, Clone)]
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
//! #[derive(Default, Clone)]
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
mod dyn_traits;
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
pub use tessera_macros::tessera;
pub use wgpu;
pub use winit;

pub use crate::{
    accessibility::{AccessibilityActionHandler, AccessibilityId, AccessibilityNode},
    color::Color,
    component_tree::{
        ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree,
        ComponentTree, ComputedData, Constraint, DimensionValue, ImeRequest, InputHandlerFn,
        InputHandlerInput, MeasurementError, ParentConstraint,
    },
    context::{Context, provide_context, use_context},
    cursor::{CursorEvent, CursorEventContent, GestureState, PressKeyEventType, ScrollEventConent},
    dp::Dp,
    dyn_traits::{DynPartialEqCompute, DynPartialEqDraw},
    entry_registry::{EntryRegistry, TesseraPackage},
    focus_state::Focus,
    layout::{DefaultLayoutSpec, LayoutInput, LayoutOutput, LayoutResult, LayoutSpec, RenderInput},
    modifier::{Modifier, ModifierChild, ModifierWrapper},
    pipeline_context::PipelineContext,
    plugin::{
        Plugin, PluginContext, PluginResult, register_plugin, register_plugin_boxed, with_plugin,
        with_plugin_mut,
    },
    px::{Px, PxPosition, PxRect, PxSize},
    render_graph::{
        RenderFragment, RenderFragmentOp, RenderGraph, RenderGraphOp, RenderGraphParts,
        RenderResource, RenderResourceId, RenderTextureDesc,
    },
    render_module::{RenderMiddleware, RenderMiddlewareContext, RenderModule},
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
    },
    runtime::{State, key, remember, remember_with_key, retain, retain_with_key},
};

use ime_state::ImeState;

#[cfg(feature = "shard")]
pub use tessera_macros::shard;
#[cfg(feature = "shard")]
pub use tessera_shard;

/// Internal helper to enable deadlock detection in debug builds.
#[doc(hidden)]
pub fn __tessera_init_deadlock_detection() {
    #[cfg(debug_assertions)]
    {
        use std::{sync::Once, thread, time::Duration};

        static INIT: Once = Once::new();
        INIT.call_once(|| {
            thread::spawn(|| {
                loop {
                    thread::sleep(Duration::from_secs(10));
                    let deadlocks = parking_lot::deadlock::check_deadlock();
                    if deadlocks.is_empty() {
                        continue;
                    }

                    eprintln!("{} deadlocks detected", deadlocks.len());
                    for (idx, threads) in deadlocks.iter().enumerate() {
                        eprintln!("Deadlock #{}", idx);
                        for thread in threads {
                            eprintln!("Thread Id {:#?}", thread.thread_id());
                            eprintln!("{:?}", thread.backtrace());
                        }
                    }
                }
            });
        });
    }
}

/// Internal helper to initialize tracing subscribers for app entry points.
#[doc(hidden)]
pub fn __tessera_init_tracing() {
    #[cfg(target_os = "android")]
    {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_max_level(tracing::Level::INFO)
            .try_init();
    }

    #[cfg(not(target_os = "android"))]
    {
        let filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
            Ok(filter) => filter,
            Err(_) => match tracing_subscriber::EnvFilter::try_new("error,tessera_ui=info") {
                Ok(filter) => filter,
                Err(_) => tracing_subscriber::EnvFilter::new("error"),
            },
        };

        let _ = tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(filter)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .try_init();
    }
}

/// Defines the Tessera application entry points for desktop and Android.
///
/// This macro registers packages and plugins, then starts the renderer with
/// the provided render modules.
///
/// # Example:
///
/// ```rust,ignore
/// use tessera_components::theme::{MaterialTheme, material_theme};
///
/// fn app() {
///     material_theme(MaterialTheme::default, || {
///         // Your app code here
///     });
/// }
///
/// tessera_ui::entry!(
///     app,
///     packages = [tessera_components::ComponentsPackage::default()],
/// );
/// ```
#[macro_export]
macro_rules! entry {
    ($entry:path $(,)?) => {
        $crate::entry!(
            @parse
            $entry,
            {
                plugins: [],
                modules: [],
                packages: [],
                config: $crate::entry!(@config)
            },
        );
    };
    ($entry:path, $($rest:tt)+) => {
        $crate::entry!(
            @parse
            $entry,
            {
                plugins: [],
                modules: [],
                packages: [],
                config: $crate::entry!(@config)
            },
            $($rest)+
        );
    };
    (@config) => {
        $crate::renderer::TesseraConfig::default()
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        ) => {
        $crate::entry!(@run $entry, $config, [$($modules),*], [$($plugins),*], [$($packages),*]);
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        plugins = [$($plugin:expr),* $(,)?],
        $($rest:tt)+
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugin),*], modules: [$($modules),*], packages: [$($packages),*], config: $config },
            $($rest)+
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        plugins = [$($plugin:expr),* $(,)?],
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugin),*], modules: [$($modules),*], packages: [$($packages),*], config: $config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        plugins = [$($plugin:expr),* $(,)?]
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugin),*], modules: [$($modules),*], packages: [$($packages),*], config: $config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        modules = [$($new_modules:expr),* $(,)?],
        $($rest:tt)+
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($new_modules),*], packages: [$($packages),*], config: $config },
            $($rest)+
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        modules = [$($new_modules:expr),* $(,)?],
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($new_modules),*], packages: [$($packages),*], config: $config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        modules = [$($new_modules:expr),* $(,)?]
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($new_modules),*], packages: [$($packages),*], config: $config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        packages = [$($new_packages:expr),* $(,)?],
        $($rest:tt)+
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($new_packages),*], config: $config },
            $($rest)+
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        packages = [$($new_packages:expr),* $(,)?],
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($new_packages),*], config: $config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        packages = [$($new_packages:expr),* $(,)?]
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($new_packages),*], config: $config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        config = $new_config:expr,
        $($rest:tt)+
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($packages),*], config: $new_config },
            $($rest)+
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        config = $new_config:expr,
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($packages),*], config: $new_config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        config = $new_config:expr
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($packages),*], config: $new_config },
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        , $($rest:tt)*
    ) => {
        $crate::entry!(
            @parse
            $entry,
            { plugins: [$($plugins),*], modules: [$($modules),*], packages: [$($packages),*], config: $config },
            $($rest)*
        );
    };
    (@parse
        $entry:path,
        { plugins: [$($plugins:expr),*], modules: [$($modules:expr),*], packages: [$($packages:expr),*], config: $config:expr },
        $($unexpected:tt)+
    ) => {
        compile_error!("Unsupported argument for tessera_ui::entry!");
    };
    (@run $entry:path, $config:expr, [$($module:expr),*], [$($plugin:expr),*], [$($package:expr),*]) => {
        #[doc(hidden)]
        fn __tessera_modules() -> Vec<Box<dyn $crate::RenderModule>> {
            let mut registry = $crate::EntryRegistry::new();
            $(
                registry.register_plugin($plugin);
            )*
            $(
                registry.register_package($package);
            )*
            $(
                registry.add_module($module);
            )*
            registry.finish()
        }

        #[cfg(target_os = "android")]
        #[unsafe(no_mangle)]
        fn android_main(android_app: $crate::winit::platform::android::activity::AndroidApp) {
            __tessera_entry(android_app);
        }

        #[cfg(target_os = "android")]
        #[allow(dead_code)]
        fn main() {}

        #[cfg(target_os = "android")]
        #[doc(hidden)]
        pub fn __tessera_entry(
            android_app: $crate::winit::platform::android::activity::AndroidApp,
        ) {
            $crate::__tessera_init_tracing();
            $crate::__tessera_init_deadlock_detection();
            if let Err(err) = $crate::Renderer::run_with_config(
                $entry,
                __tessera_modules(),
                android_app,
                $config,
            ) {
                eprintln!("App failed to run: {err}");
            }
        }

        #[cfg(not(target_os = "android"))]
        fn main() {
            __tessera_entry();
        }

        #[cfg(not(target_os = "android"))]
        #[doc(hidden)]
        pub fn __tessera_entry() {
            $crate::__tessera_init_tracing();
            $crate::__tessera_init_deadlock_detection();
            if let Err(err) = $crate::Renderer::run_with_config(
                $entry,
                __tessera_modules(),
                $config,
            ) {
                eprintln!("App failed to run: {err}");
            }
        }
    };
}
