//! Basic components for the Tessera UI framework.
//!
//! # Usage
//!
//! First, you need to register the pipelines provided by this crate.
//!
//! ```no_run
//! use tessera_ui::renderer::Renderer;
//! use tessera_ui_basic_components::pipelines::register_pipelines;
//!
//! Renderer::run(
//!     // ...
//!     # || {}, // Placeholder for root component
//!     |app| {
//!         tessera_ui_basic_components::pipelines::register_pipelines(app);
//!     }
//! );
//! ```
//!
//! Then you can use the components in your UI.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//!
//! use tessera_ui::Dp;
//! use tessera_ui_basic_components::{
//!     button::{button, ButtonArgs},
//!     text::text,
//!     text_editor::{text_editor, TextEditorArgs, TextEditorState},
//!     RippleState,
//! };
//!
//! // Button example
//! let ripple_state = RippleState::new();
//! button(
//!     ButtonArgs {
//!         on_click: Some(Arc::new(|| { /* Handle click */ })),
//!         ..Default::default()
//!     },
//!     ripple_state.clone(),
//!     || text("Click me".to_string()),
//! );
//!
//! // Text editor example
//! let editor_state = TextEditorState::new(Dp(16.0), None);
//! text_editor(TextEditorArgs::default(), editor_state.clone());
//! ```
#![deny(missing_docs, clippy::unwrap_used)]

mod animation;
mod padding_utils;
mod selection_highlight_rect;

pub mod alignment;
pub mod bottom_sheet;
pub mod boxed;
pub mod button;
pub mod checkbox;
mod checkmark;
pub mod column;
pub mod dialog;
pub mod fluid_glass;
pub mod glass_button;
pub mod glass_progress;
pub mod glass_slider;
pub mod glass_switch;
pub mod icon;
pub mod icon_button;
pub mod image;
pub mod image_vector;
pub mod lazy_list;
pub mod pipelines;
pub mod pos_misc;
pub mod progress;
pub mod ripple_state;
pub use ripple_state::RippleState;
pub mod bottom_nav_bar;
pub mod row;
pub mod scrollable;
pub mod shape_def;
pub mod side_bar;
pub mod slider;
pub mod spacer;
pub mod surface;
pub mod switch;
pub mod tabs;
pub mod text;
mod text_edit_core;
pub mod text_editor;
