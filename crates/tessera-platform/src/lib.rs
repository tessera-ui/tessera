//! Platform services for Tessera applications.
//!
//! ## Usage
//!
//! Register platform plugins like clipboard and window access at app startup.
#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

pub mod clipboard;
pub mod window;

use tessera_ui::{EntryRegistry, TesseraPackage};

pub use clipboard::{Clipboard, ClipboardPlugin};
pub use window::WindowPlugin;

/// Package that registers platform plugins.
#[derive(Clone, Debug, Default)]
pub struct PlatformPackage;

impl PlatformPackage {
    /// Creates a platform package.
    pub fn new() -> Self {
        Self
    }
}

impl TesseraPackage for PlatformPackage {
    fn register(self, registry: &mut EntryRegistry) {
        registry.register_plugin(ClipboardPlugin::new());
        registry.register_plugin(WindowPlugin::new());
    }
}
