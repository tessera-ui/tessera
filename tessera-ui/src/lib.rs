//! Declarative UI runtime and extension APIs.
//!
//! ## Usage
//!
//! Compose with [`tessera`], defer child content with [`RenderSlot`], and own data with [`State`].
#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

extern crate self as tessera_ui;

#[doc(hidden)]
pub mod __private;
pub mod accessibility;
#[cfg(target_os = "android")]
pub mod android;
pub mod asset;
mod async_dispatch;
mod build_tree;
mod clip;
pub mod clipboard;
pub mod color;
mod component_call;
mod dispatcher;
mod graphics;
mod work;

pub use async_dispatch::{Dispatcher, Task, on, spawn, with_frame_time};

mod session;
pub mod system_back;

pub use session::{
    RuntimeSession, RuntimeSessionGuard, RuntimeSessionId, WeakRuntimeSession, runtime_resource,
};
pub(crate) use session::{local_runtime_resource, shared_runtime_resource};

mod component_tree;
pub mod context;
mod cursor;
pub mod dp;
pub mod entry_point;
pub mod entry_registry;
mod execution_context;
pub mod focus;
mod gap_buffer;
pub mod geometry;
#[cfg(all(feature = "headless", not(target_family = "wasm")))]
pub mod headless;
mod ime_state;
mod inline_set;
mod keyboard_state;
pub mod layout;
#[cfg(test)]
mod layout_policy_tests;
pub mod modifier;
pub mod nested_scroll;
pub(crate) mod pipeline_cache;
mod pipeline_context;
pub mod plugin;
#[cfg(feature = "profiling")]
pub mod profiler;
mod prop;
pub mod px;
mod read_dependencies;
pub mod render;
mod render_graph;
mod render_module;
mod render_pass;
mod render_scene;
pub mod renderer;
mod runtime;
pub mod scroll;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod thread_utils;
pub mod time;

pub use accesskit;
pub use tessera_macros::{entry, lint, tessera};

pub(crate) use crate::thread_utils::SendSync;
pub(crate) use indextree::NodeId;
pub use wgpu;
pub use winit;

pub use crate::{
    accessibility::{AccessibilityActionHandler, AccessibilityId, AccessibilityNode},
    asset::{Asset, FontAsset, LoadState, Resource, Source},
    clipboard::{
        ClipboardAction, ClipboardInput, ClipboardInputModifierNode, ClipboardResponse,
        ClipboardWrite, request_clipboard_action,
    },
    color::Color,
    component_tree::{
        AxisConstraint, ComputedData, Constraint, ImeInput, ImeRequest, ImeSession, KeyboardInput,
        MeasurementError, ParentConstraint, PointerEventPass, PointerInput,
    },
    context::{Context, provide_context, use_context},
    cursor::{
        CursorEventContent, GestureState, MOUSE_POINTER_ID, PointerChange, PointerId,
        PressKeyEventType, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource,
    },
    dp::Dp,
    entry_point::EntryPoint,
    entry_registry::{EntryRegistry, TesseraPackage},
    focus::{
        FocusDirection, FocusGroupNode, FocusProperties, FocusRequester, FocusRestorationTarget,
        FocusScopeNode, FocusState, FocusTraversalPolicy, FocusTraversalStrategy,
    },
    geometry::{FloatRect, LayoutCoordinates, LocalToWindow, scaled_box},
    keyboard_state::KeyboardEvent,
    layout::{
        DrawModifierHandle, LayoutResult, NoopDrawModifier, RenderInput, RenderMetadataMut, Stack,
    },
    modifier::{
        BuildModifierNode, CursorModifierNode, DrawModifierContent, DrawModifierContext,
        DrawModifierNode, ImeInputModifierNode, KeyboardInputModifierNode, LayoutModifier,
        LayoutModifierChild, LayoutModifierInput, LayoutPolicy, Modifier, ModifierBuildContext,
        ParentDataMap, ParentDataModifierNode, PlacementModifierNode, PointerInputModifierNode,
        SemanticsModifierNode, StatefulLayoutPolicy,
    },
    plugin::{DesktopPlatformContext, Plugin, PluginContext, PluginResult, with_plugin},
    prop::{Callback, CallbackWith, RenderSlot, RenderSlotWith},
    px::{Px, PxPosition, PxRect, PxSize},
    renderer::Renderer,
    runtime::{
        Model, ModelInit, ModelStore, ModelStoreScope, State, key, launch, launch_once, remember,
        remember_with_key, retain, retain_with_key,
    },
};

use ime_state::ImeState;

#[cfg(target_os = "android")]
pub use {jni, ndk_context, ndk_sys};

/// Window safe areas and keyboard occlusion.
pub mod window_insets;
pub use window_insets::{Insets, WindowInsets, WindowInsetsController};
