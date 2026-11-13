//! Official components for the Tessera UI framework.
//!
//! The crate root provides layout and content primitives shared by every
//! design system. Design-system components live in their own modules:
//! [`md3`] implements Material Design 3, and [`experiment::hig`] contains
//! experimental Apple Human Interface Guidelines components.
//!
//! Built-in labels follow the language registered with the application entry.
//!
//! # Usage
//!
//! First, add the package (recommended) or render module provided by this crate
//! at application entry.
//!
//! ```no_run
//! use tessera_components::md3::theme::{MaterialTheme, material_theme};
//!
//! #[tessera_ui::tessera]
//! fn app() {
//!     material_theme()
//!         .theme(|| MaterialTheme::default())
//!         .child(|| {
//!             // Your app code here
//!         });
//! }
//!
//! use tessera_ui::EntryPoint;
//!
//! #[tessera_ui::entry]
//! pub fn run() -> EntryPoint {
//!     EntryPoint::new(app).package(tessera_components::ComponentsPackage)
//! }
//!
//! fn main() {
//!     let _ = run().run_desktop();
//! }
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
//! use tessera_components::md3::{button::button, text::text, text_input::text_input};
//! # use tessera_components::md3::theme::{MaterialTheme, material_theme};
//! # material_theme()
//! #     .theme(|| MaterialTheme::default())
//! #     .child(|| {
//!
//! // Button example
//! button().filled().on_click(|| {}).child(|| {
//!     text("Click me");
//! });
//!
//! // Text editor example
//! text_input();
//! #     });
//! # }
//! # tessera_ui::testing::UiTest::new().content(component).frame();
//! ```
#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]
// Resolving `Send`/`Sync` for the GPU handles stored by pipeline types recurses
// deep enough inside `wgpu` to hit the solver's recursion limit. Raise the limit
// so auto traits resolve without falling back to the depth guard.
#![recursion_limit = "512"]

mod animation;
pub mod boxed;
pub mod column;
pub mod experiment;
pub mod flow_column;
pub mod flow_row;
pub mod fonts;
pub mod image;
pub mod image_vector;
pub mod interaction_state;
mod layout_flex;
mod lazy_shared;
pub mod md3;
mod modal_focus;
pub mod modifier;
mod nested_scroll;
mod overlay_semantics;
mod padding_utils;
pub mod pager;
pub mod painter;
pub mod pipelines;
pub mod pos_misc;
pub mod res;
pub mod row;
pub mod spacer;

#[cfg(test)]
mod test_utils;

use tessera_platform::PlatformPackage;
use tessera_ui::{
    EntryRegistry, TesseraPackage,
    render::{PipelineContext, RenderModule},
};

pub use tessera_foundation::{alignment, shape_def};

/// Render module for registering all Tessera component pipelines.
#[derive(Default, Clone, PartialEq, Copy)]
struct TesseraComponents;

impl RenderModule for TesseraComponents {
    fn register_pipelines(&self, context: &mut PipelineContext<'_>) {
        pipelines::register_pipelines(context);
    }
}

/// Package that registers the components module and required platform services.
#[derive(Clone, PartialEq, Default, Copy)]
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
        pipelines::text::pipeline::preload_system_fonts();
        registry.register_package(PlatformPackage);
        registry.add_module(TesseraComponents);
    }
}
