//! # Tessera UI Framework
//!
//! Tessera is a declarative, immediate-mode UI framework for Rust that emphasizes performance,
//! flexibility, and extensibility through a functional approach and pluggable shader system.
//!
//! ## Architecture Overview
//!
//! Tessera's architecture is built around several core concepts:
//!
//! - **Declarative Components**: UI components are defined as functions with the `#[tessera]` macro
//! - **Immediate Mode**: The UI is rebuilt every frame, ensuring consistency and simplicity
//! - **Pluggable Shaders**: Custom WGPU shaders are first-class citizens for advanced visual effects
//! - **Parallel Processing**: Core operations like measurement utilize parallel computation
//! - **Explicit State Management**: Components are stateless with explicit state passing
//!
//! ## Getting Started by Developer Level
//!
//! ### 🟢 **Beginner Users** - Building Basic Applications
//!
//! If you're new to Tessera and want to build applications using existing components:
//!
//! **Start with these modules:**
//! - [`renderer`] - Core renderer and application lifecycle management
//! - [`Dp`], [`Px`] - Basic measurement units for layouts
//! - [`Color`] - Color system for styling components
//!
//! **Key concepts to understand:**
//! - How to set up a [`Renderer`] and run your application
//! - Using [`tessera-ui-basic-components`](https://docs.rs/tessera-ui-basic-components/latest/tessera_ui_basic_components/) for common UI elements
//! - Basic layout with `row`, `column`, and `surface` components
//!
//! ### 🟡 **Intermediate Users** - Custom Layout and Interaction
//!
//! For developers who want to create custom components and handle complex layouts:
//!
//! **Essential functions and types:**
//! - [`measure_node`] - Measure child component sizes with constraints
//! - [`place_node`] - Position child components in the layout
//! - [`InputHandlerFn`] - Handle user interactions and state changes
//! - [`Constraint`], [`DimensionValue`] - Layout constraint system
//! - [`ComputedData`] - Return computed size and layout information
//!
//! **Key concepts:**
//! - Understanding the measurement and placement phase
//! - Creating custom layout algorithms
//! - Managing component state through explicit input handlers
//! - Working with the constraint-based layout system
//!
//! ```
//! use tessera_ui::{measure_node, place_node, ComputedData, Constraint, PxPosition, tessera};
//!
//! #[tessera]
//! fn custom_layout(child: impl FnOnce()) {
//!     child();
//!
//!     measure(Box::new(|input| {
//!         // Custom measurement logic here
//! #        Ok(ComputedData::ZERO) // Make doc tests happy
//!     }));
//!
//!     input_handler(Box::new(|input| {
//!         // Handle user interactions here
//!     }));
//! }
//! ```
//!
//! ### 🔴 **Advanced Users** - Custom Rendering Pipelines
//!
//! For developers building custom visual effects and rendering pipelines:
//!
//! **Advanced rendering modules:**
//!
//! - [`renderer::drawer`] - Custom drawable pipelines and draw commands
//! - [`renderer::compute`] - GPU compute pipelines for advanced effects
//! - [`DrawCommand`], [`ComputeCommand`] - Low-level rendering commands
//! - [`DrawablePipeline`], [`ComputablePipeline`] - Pipeline trait implementations
//! - [`PipelineRegistry`], [`ComputePipelineRegistry`] - Pipeline management
//!
//! **Key concepts:**
//!
//! - Creating custom WGPU shaders and pipelines
//! - Managing GPU resources and compute operations
//! - Understanding the rendering command system
//! - Implementing advanced visual effects like lighting, shadows, and particles
//!
//! ## Core Modules
//!
//! ### Essential Types and Functions
//!
//! - [`Renderer`] - Main application renderer and lifecycle manager
//! - [`measure_node`], [`place_node`] - Core layout functions
//! - [`Constraint`], [`DimensionValue`] - Layout constraint system
//! - [`Dp`], [`Px`] - Measurement units (device-independent and pixel units)
//! - [`Color`] - Color representation and utilities
//!
//! ### Component System
//!
//! - [`ComponentTree`] - Component tree management
//! - [`ComponentNode`] - Individual component node representation
//! - [`ComputedData`] - Layout computation results
//! - [`InputHandlerFn`] - State management and event handling
//!
//! ### Event Handling
//!
//! - [`CursorEvent`] - Mouse and touch input events
//! - [`Focus`] - Focus management system
//! - [`PressKeyEventType`] - Keyboard input handling
//!
//! ### Rendering System
//!
//! - [`renderer::drawer`] - Drawing pipeline system
//! - [`renderer::compute`] - Compute pipeline system
//! - [`DrawCommand`], [`ComputeCommand`] - Rendering commands
//!
//! ## Examples
//!
//! Check out the `example` crate in the workspace for comprehensive examples demonstrating:
//! - Basic component usage
//! - Custom layouts and interactions
//! - Advanced shader effects
//! - Cross-platform deployment (Windows, Linux, macOS, Android)
//!
//! ## Performance Considerations
//!
//! Tessera is designed for high performance through:
//! - Parallel measurement computation using Rayon
//! - Efficient GPU utilization through custom shaders
//! - Minimal allocations in hot paths
//! - Optimized component tree traversal

pub mod accessibility;
pub mod clipboard;
pub mod color;
mod component_tree;
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
        InputHandlerInput, MeasureFn, MeasureInput, MeasurementError, measure_node, measure_nodes,
        place_node,
    },
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
    runtime::TesseraRuntime,
};

use ime_state::ImeState;

#[cfg(feature = "shard")]
pub use tessera_ui_macros::shard;
#[cfg(feature = "shard")]
pub use tessera_ui_shard;
