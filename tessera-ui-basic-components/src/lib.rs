//! Basic components for the Tessera UI framework.
//!
//! # Usage
//!
//! First, you need to register the pipelines provided by this crate.
//!
//! ```no_run
//! # use tessera_ui::tessera;
//! # #[tessera]
//! # fn component() {
//! use tessera_ui::renderer::Renderer;
//! use tessera_ui_basic_components::pipelines::register_pipelines;
//!
//! Renderer::run(
//!     // ...
//!     # || {}, // Placeholder for root component
//!     |app| {
//!         tessera_ui_basic_components::pipelines::register_pipelines(app);
//!     },
//! );
//! # }
//! # component();
//! ```
//!
//! Then you can use the components in your UI.
//!
//! # Example
//!
//! ```
//! # use tessera_ui::tessera;
//! # #[tessera]
//! # fn component() {
//! use tessera_ui::Dp;
//! use tessera_ui_basic_components::{
//!     button::{ButtonArgs, button},
//!     text::text,
//!     text_editor::{TextEditorArgs, text_editor},
//! };
//!
//! // Button example
//! button(ButtonArgs::filled(|| { /* Handle click */ }), || {
//!     text("Click me".to_string())
//! });
//!
//! // Text editor example
//! text_editor(TextEditorArgs::default());
//! # }
//! # component();
//! ```
#![deny(missing_docs, clippy::unwrap_used)]

mod animation;
mod padding_utils;
mod selection_highlight_rect;

pub mod alignment;
pub mod badge;
pub mod bottom_sheet;
pub mod boxed;
pub mod button;
pub mod button_groups;
pub mod checkbox;
mod checkmark;
pub mod column;
pub mod dialog;
pub mod divider;
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
pub mod material_icons;
pub mod theme;
pub use pipelines::shape::command::{RippleProps, ShadowProps};
pub use ripple_state::RippleState;
pub mod menus;
pub mod navigation_bar;
pub mod pipelines;
pub mod pos_misc;
pub mod progress;
pub mod radio_button;
pub mod ripple_state;
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
