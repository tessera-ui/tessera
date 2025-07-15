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
//! - Using `tessera_basic_components` for common UI elements
//! - Basic layout with `row`, `column`, and `surface` components
//!
//! ```rust,ignore
//! use tessera::{Renderer, Color, Dp};
//! use tessera_basic_components::*;
//! use tessera_macros::tessera;
//!
//! #[tessera]
//! fn my_app() {
//!     surface(
//!         SurfaceArgs {
//!             color: Color::WHITE,
//!             padding: Dp(20.0),
//!             ..Default::default()
//!         },
//!         None,
//!         || text("Hello, Tessera!"),
//!     );
//! }
//! ```
//!
//! ### 🟡 **Intermediate Users** - Custom Layout and Interaction
//!
//! For developers who want to create custom components and handle complex layouts:
//!
//! **Essential functions and types:**
//! - [`measure_node`] - Measure child component sizes with constraints
//! - [`place_node`] - Position child components in the layout
//! - [`StateHandlerFn`] - Handle user interactions and state changes
//! - [`Constraint`], [`DimensionValue`] - Layout constraint system
//! - [`ComputedData`] - Return computed size and layout information
//!
//! **Key concepts:**
//! - Understanding the measurement and placement phase
//! - Creating custom layout algorithms
//! - Managing component state through explicit state handlers
//! - Working with the constraint-based layout system
//!
//! ```rust,ignore
//! use tessera::{measure_node, place_node, ComputedData, Constraint, PxPosition};
//! use tessera_macros::tessera;
//!
//! #[tessera]
//! fn custom_layout() {
//!     measure(|input| {
//!         let mut total_width = 0;
//!         for (i, &child_id) in input.children_ids.iter().enumerate() {
//!             let child_size = measure_node(child_id, input.parent_constraint, input.metadatas)?;
//!             place_node(child_id, PxPosition::new(total_width.into(), 0.into()), input.metadatas);
//!             total_width += child_size.width.to_i32();
//!         }
//!         Ok(ComputedData::from_size((total_width.into(), input.parent_constraint.height.min_value())))
//!     });
//!
//!     state_handler(|input| {
//!         // Handle user interactions here
//!     });
//! }
//! ```
//!
//! ### 🔴 **Advanced Users** - Custom Rendering Pipelines
//!
//! For developers building custom visual effects and rendering pipelines:
//!
//! **Advanced rendering modules:**
//! - [`renderer::drawer`] - Custom drawable pipelines and draw commands
//! - [`renderer::compute`] - GPU compute pipelines for advanced effects
//! - [`DrawCommand`], [`ComputeCommand`] - Low-level rendering commands
//! - [`DrawablePipeline`], [`ComputablePipeline`] - Pipeline trait implementations
//! - [`PipelineRegistry`], [`ComputePipelineRegistry`] - Pipeline management
//!
//! **Key concepts:**
//! - Creating custom WGPU shaders and pipelines
//! - Managing GPU resources and compute operations
//! - Understanding the rendering command system
//! - Implementing advanced visual effects like lighting, shadows, and particles
//!
//! ```rust,ignore
//! use tessera::renderer::{DrawCommand, DrawablePipeline};
//! use wgpu::{Device, Queue, RenderPass};
//!
//! struct MyCustomPipeline {
//!     // Your pipeline state
//! }
//!
//! impl DrawablePipeline for MyCustomPipeline {
//!     fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
//!         // Custom rendering logic
//!     }
//! }
//! ```
//!
//! ## Core Modules
//!
//! ### Essential Types and Functions
//! - [`Renderer`] - Main application renderer and lifecycle manager
//! - [`measure_node`], [`place_node`] - Core layout functions
//! - [`Constraint`], [`DimensionValue`] - Layout constraint system
//! - [`Dp`], [`Px`] - Measurement units (device-independent and pixel units)
//! - [`Color`] - Color representation and utilities
//!
//! ### Component System
//! - [`ComponentTree`] - Component tree management
//! - [`ComponentNode`] - Individual component node representation
//! - [`ComputedData`] - Layout computation results
//! - [`StateHandlerFn`] - State management and event handling
//!
//! ### Event Handling
//! - [`CursorEvent`] - Mouse and touch input events
//! - [`Focus`] - Focus management system
//! - [`PressKeyEventType`] - Keyboard input handling
//!
//! ### Rendering System
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

pub mod color;
mod component_tree;
mod cursor;
pub mod dp;
pub mod focus_state;
mod ime_state;
mod keyboard_state;
pub mod px;
pub mod renderer;
mod runtime;
mod thread_utils;
pub mod tokio_runtime;

pub use indextree::{Arena, NodeId};
pub use wgpu;
pub use winit;

pub use crate::{
    color::Color,
    component_tree::{
        ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree,
        ComponentTree, ComputedData, Constraint, DimensionValue, ImeRequest, MeasureFn,
        MeasurementError, StateHandlerFn, StateHandlerInput, measure_node, measure_nodes,
        place_node,
    },
    cursor::{CursorEvent, CursorEventContent, PressKeyEventType, ScrollEventConent},
    dp::Dp,
    focus_state::Focus,
    px::{Px, PxPosition, PxSize},
    renderer::{
        Command, Renderer,
        compute::{
            self, ComputablePipeline, ComputeCommand, ComputePipelineRegistry, ComputeResource,
            ComputeResourceManager, ComputeResourceRef,
        },
        drawer::{
            self, BarrierRequirement, DrawCommand, DrawablePipeline, PipelineRegistry, command,
        },
    },
    runtime::TesseraRuntime,
};

use ime_state::ImeState;
