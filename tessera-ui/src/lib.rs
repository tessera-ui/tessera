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
mod build_tree;
pub mod clip;
pub mod clipboard;
pub mod color;
mod component_call;
mod dispatcher;
mod graphics;
mod work;

pub use dispatcher::UiDispatcher;
pub use graphics::{
    FrameBuffer, GraphicsDevice, GraphicsWork, GraphicsWorkDiagnostics, ScratchFit, ScratchTexture,
    ScratchTextureDescriptor, TrackedBuffer,
};
pub use work::{FrameWork, WorkExecution, WorkPhase, WorkTotals};

mod session;
pub mod system_back;

pub use session::{RuntimeSession, RuntimeSessionGuard, RuntimeSessionId, WeakRuntimeSession};

pub(crate) use session::{local_runtime_resource, runtime_resource};

mod component_tree;
pub mod context;
mod cursor;
pub mod dp;
pub mod entry_point;
pub mod entry_registry;
mod execution_context;
pub mod focus;
pub mod geometry;
#[cfg(all(feature = "headless", not(target_family = "wasm")))]
pub mod headless;
mod ime_state;
mod keyboard_state;
pub mod layout;
pub mod modifier;
pub mod nested_scroll;
pub(crate) mod pipeline_cache;
pub mod pipeline_context;
pub mod plugin;
#[cfg(feature = "profiling")]
pub mod profiler;
mod prop;
pub mod px;
mod read_dependencies;
mod render_graph;
pub mod render_module;
mod render_pass;
pub mod render_scene;
pub mod renderer;
mod runtime;
pub mod scroll;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod thread_utils;
pub mod time;

pub use accesskit;
pub use indextree::{Arena, NodeId};
pub use tessera_macros::{entry, lint, tessera};
pub use wgpu;
pub use winit;

pub use crate::{
    accessibility::{AccessibilityActionHandler, AccessibilityId, AccessibilityNode},
    asset::{Asset, FontAsset, LoadState, Resource, Source},
    clip::{ClipRegion, ClipShape, ui_stencil_state},
    clipboard::{
        ClipboardAction, ClipboardInput, ClipboardInputModifierNode, ClipboardResponse,
        ClipboardWrite, request_clipboard_action,
    },
    color::Color,
    component_tree::{
        AxisConstraint, ComponentTree, ComputedData, Constraint, ImeInput, ImeInputHandlerFn,
        ImeRequest, ImeSession, KeyboardInput, KeyboardInputHandlerFn, MeasurementError,
        ParentConstraint, PointerEventPass, PointerInput, PointerInputHandlerFn,
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
        FocusDirection, FocusGroupNode, FocusManager, FocusProperties, FocusRequester,
        FocusRestorationTarget, FocusScopeNode, FocusState, FocusTraversalPolicy,
        FocusTraversalStrategy,
    },
    geometry::{FloatRect, LayoutCoordinates, LocalToWindow, scaled_box},
    keyboard_state::KeyboardEvent,
    layout::{
        DefaultLayoutNode, DrawModifierHandle, LayoutModifierHandle, LayoutResult,
        NoopDrawModifier, RenderInput, RenderMetadataMut,
    },
    modifier::{
        BuildModifierNode, CursorModifierExt, CursorModifierNode, DrawModifierContent,
        DrawModifierContext, DrawModifierNode, FocusModifierExt, ImeInputModifierNode,
        KeyboardInputModifierNode, LayoutModifierChild, LayoutModifierInput, LayoutModifierNode,
        Modifier, ModifierBuildContext, ParentDataMap, ParentDataModifierNode,
        PlacementModifierNode, PointerInputModifierNode, SemanticsModifierNode,
    },
    pipeline_context::PipelineContext,
    plugin::{
        DesktopPlatformContext, DesktopWindowAction, Plugin, PluginContext, PluginResult,
        register_plugin, register_plugin_boxed, try_with_plugin, try_with_plugin_mut, with_plugin,
        with_plugin_mut,
    },
    prop::{Callback, CallbackWith, RenderSlot, RenderSlotWith, Slot},
    px::{Px, PxPosition, PxRect, PxSize},
    render_graph::{DrawGeometry, RenderFragment, RenderFragmentOp, RenderGraph, RenderGraphOp},
    render_module::RenderModule,
    render_scene::{Command, DrawRegion, PaddingRect, SampleRegion},
    renderer::{
        Renderer,
        compute::{self, ComputablePipeline, ComputeCommand, ComputePipelineRegistry},
        core::OffscreenReadbackError,
        drawer::{self, DrawCommand, DrawablePipeline, PipelineRegistry, command},
    },
    runtime::{
        FrameClockDiagnostics, FrameNanosControl, State, StateOwner, current_frame_nanos,
        current_frame_time, frame_delta, key, receive_frame_nanos, remember, remember_with_key,
        retain, retain_with_key,
    },
    scroll::{PlatformScrollConfig, normalize_platform_scroll_delta, platform_scroll_config},
    thread_utils::SendSync,
};

use ime_state::ImeState;

#[cfg(all(feature = "headless", not(target_family = "wasm")))]
pub use crate::headless::{
    HEADLESS_ENV, HEADLESS_FRAME_TIME_MS_ENV, HEADLESS_HEIGHT_ENV, HEADLESS_WIDTH_ENV,
    HeadlessConfig, HeadlessError, HeadlessRenderer, encode_png_rgba, headless_mode_requested,
    run_headless, write_png_file,
};

#[cfg(target_os = "android")]
pub use {jni, ndk_context, ndk_sys};

/// Window safe areas and keyboard occlusion.
pub mod window_insets;
pub use window_insets::{Insets, WindowInsets, WindowInsetsController};
