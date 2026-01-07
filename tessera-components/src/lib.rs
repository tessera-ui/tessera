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
//! use tessera_components::pipelines::register_pipelines;
//! use tessera_ui::renderer::Renderer;
//!
//! Renderer::run(
//!     // ...
//!     # || {}, // Placeholder for root component
//!     |app| {
//!         tessera_components::pipelines::register_pipelines(app);
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
//! use tessera_components::{
//!     button::{ButtonArgs, button},
//!     text::text,
//!     text_editor::{TextEditorArgs, text_editor},
//! };
//! use tessera_ui::Dp;
//! # use tessera_components::theme::{MaterialTheme, material_theme};
//! # material_theme(|| MaterialTheme::default(), || {
//!
//! // Button example
//! button(ButtonArgs::filled(|| { /* Handle click */ }), || {
//!     text("Click me".to_string())
//! });
//!
//! // Text editor example
//! text_editor(TextEditorArgs::default());
//! # });
//! # }
//! # component();
//! ```
#![deny(missing_docs, clippy::unwrap_used)]

mod animation;
mod padding_utils;
mod selection_highlight_rect;

pub mod alignment;
pub mod app_bar;
pub mod badge;
pub mod bottom_sheet;
pub mod boxed;
pub mod button;
pub mod button_groups;
pub mod card;
pub mod checkbox;
mod checkmark;
pub mod chip;
pub mod column;
pub mod date_picker;
pub mod dialog;
pub mod divider;
pub mod floating_action_button;
pub mod flow_column;
pub mod fluid_glass;
pub mod glass_button;
pub mod glass_progress;
pub mod glass_slider;
pub mod glass_switch;
pub mod icon;
pub mod icon_button;
pub mod image;
pub mod image_vector;
pub mod interaction_state;
pub mod lazy_grid;
pub mod lazy_list;
pub mod lazy_staggered_grid;
pub mod material_icons;
pub mod modifier;
pub mod theme;
pub use pipelines::shape::command::RippleProps;
pub use ripple_state::RippleState;
pub mod flow_row;
pub mod menus;
pub mod navigation_bar;
pub mod navigation_rail;
pub mod pager;
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
pub mod time_picker;
