//! Basic UI components library for Tessera.
//! Internal animation utilities are available via the private `animation` module.

//! Basic components for the Tessera UI framework.
//!
//! # Usage
//!
//! First, you need to register the pipelines provided by this crate.
//!
//! ```rust,ignore
//! use tessera_ui::renderer::WgpuApp;
//! use tessera_ui_basic_components::pipelines::register_pipelines;
//!
//! fn setup(app: &mut WgpuApp) {
//!     register_pipelines(app);
//! }
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
//! let ripple_state = Arc::new(RippleState::new());
//! button(
//!     ButtonArgs {
//!         on_click: Arc::new(|| { /* Handle click */ }),
//!         ..Default::default()
//!     },
//!     ripple_state,
//!     || text("Click me".to_string()),
//! );
//!
//! // Text editor example
//! let editor_state = Arc::new(RwLock::new(TextEditorState::new(Dp(16.0), None)));
//! text_editor(TextEditorArgs::default(), editor_state.clone());
//! ```

mod animation;

pub mod alignment;
pub mod bottom_sheet;
pub mod boxed;
pub mod button;
pub mod checkbox;
pub mod checkmark;
pub mod column;
pub mod dialog;
pub mod fluid_glass;
pub mod glass_button;
pub mod glass_progress;
pub mod glass_slider;
pub mod glass_switch;
pub mod image;
pub mod padding_utils;
pub mod pipelines;
pub mod pos_misc;
pub mod progress;
pub mod ripple_state;
pub use ripple_state::RippleState;
pub mod row;
pub mod scrollable;
pub mod selection_highlight_rect;
pub mod shape_def;
pub mod slider;
pub mod spacer;
pub mod surface;
pub mod switch;
pub mod tabs;
pub mod text;
pub mod text_edit_core;
pub mod text_editor;
