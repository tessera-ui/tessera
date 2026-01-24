//! Basic components for the Tessera UI framework.
//!
//! # Usage
//!
//! First, add the package (recommended) or render module provided by this crate
//! at application entry.
//!
//! ```no_run
//! use tessera_components::theme::{MaterialTheme, material_theme};
//!
//! fn app() {
//!     material_theme(MaterialTheme::default, || {
//!         // Your app code here
//!     });
//! }
//!
//! tessera_ui::entry!(
//!     app,
//!     packages = [tessera_components::ComponentsPackage::default()],
//! );
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
//!     text_input::{TextInputArgs, text_input},
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
//! text_input(TextInputArgs::default());
//! # });
//! # }
//! # component();
//! ```
#![deny(missing_docs, clippy::unwrap_used)]

pub mod alignment;
mod animation;
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
pub mod flow_row;
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
pub mod menus;
pub mod modifier;
pub mod navigation_bar;
pub mod navigation_rail;
mod padding_utils;
pub mod pager;
pub mod pipelines;
pub mod pos_misc;
pub mod progress;
pub mod pull_refresh;
pub mod radio_button;
pub mod ripple_state;
pub mod row;
pub mod scaffold;
pub mod scrollable;
mod selection_highlight_rect;
pub mod shadow;
pub mod shape_def;
pub mod side_bar;
pub mod slider;
pub mod spacer;
pub mod surface;
pub mod switch;
pub mod tabs;
pub mod text;
mod text_edit_core;
pub mod text_field;
pub mod text_input;
pub mod theme;
pub mod time_picker;

use tessera_platform::PlatformPackage;
use tessera_ui::{EntryRegistry, PipelineContext, RenderModule, TesseraPackage};

pub use pipelines::shape::command::RippleProps;
pub use ripple_state::RippleState;

/// Render module for registering all Tessera component pipelines.
#[derive(Default, Clone, Copy)]
struct TesseraComponents;

impl RenderModule for TesseraComponents {
    fn register_pipelines(&self, context: &mut PipelineContext<'_>) {
        pipelines::register_pipelines(context);
    }
}

/// Package that registers the components module and required platform services.
#[derive(Clone, Default, Copy)]
pub struct ComponentsPackage;

impl ComponentsPackage {
    /// Creates a package that registers components and required platform
    /// plugins.
    pub fn new() -> Self {
        Self
    }
}

impl TesseraPackage for ComponentsPackage {
    fn register(self, registry: &mut EntryRegistry) {
        registry.register_package(PlatformPackage);
        registry.add_module(TesseraComponents);
    }
}
