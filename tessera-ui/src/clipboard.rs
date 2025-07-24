//! Provides a cross-platform clipboard manager for text manipulation.
//!
//! This module offers a simple, unified interface for interacting with the system clipboard,
//! allowing applications to easily get and set text content. It abstracts platform-specific
//! details, providing a consistent API across different operating systems.
//!
//! # Key Features
//!
//! - **Set Text**: Place a string onto the system clipboard.
//! - **Get Text**: Retrieve the current text content from the system clipboard.
//! - **Cross-platform**: Uses `arboard` for broad platform support (Windows, macOS, Linux).
//! - **Graceful Fallback**: On unsupported platforms like Android, operations are no-ops
//!   that log a warning, preventing crashes.
//!
//! # Usage
//!
//! The main entry point is the [`Clipboard`] struct, which provides methods to interact
//! with the system clipboard.
//!
//! ```no_run
//! use tessera_ui::clipboard::Clipboard;
//!
//! // Create a new clipboard instance.
//! let mut clipboard = Clipboard::new();
//!
//! // Set text to the clipboard.
//! let text_to_set = "Hello, Tessera!";
//! clipboard.set_text(text_to_set);
//!
//! // Get text from the clipboard.
//! if let Some(text_from_clipboard) = clipboard.get_text() {
//!     assert_eq!(text_from_clipboard, text_to_set);
//!     println!("Clipboard text: {}", text_from_clipboard);
//! } else {
//!     println!("Could not retrieve text from clipboard.");
//! }
//! ```
//!
//! # Note on Android
//!
//! Clipboard operations are currently not supported on Android. Any calls to `set_text` or
//! `get_text` on Android will result in a warning log and will not perform any action.

/// Manages access to the system clipboard for text-based copy and paste operations.
///
/// This struct acts as a handle to the platform's native clipboard, abstracting away the
/// underlying implementation details. It is created using [`Clipboard::new()`].
///
/// All interactions are synchronous. For unsupported platforms (e.g., Android),
/// operations are gracefully handled to prevent runtime errors.
pub struct Clipboard {
    #[cfg(not(target_os = "android"))]
    /// The clipboard manager for handling clipboard operations.
    manager: arboard::Clipboard,
}

impl Default for Clipboard {
    /// Creates a new `Clipboard` instance using default settings.
    ///
    /// This is equivalent to calling [`Clipboard::new()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tessera_ui::clipboard::Clipboard;
    ///
    /// let clipboard = Clipboard::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    /// Creates a new clipboard instance, initializing the connection to the system clipboard.
    ///
    /// This method may fail if the system clipboard is unavailable, in which case it will panic.
    ///
    /// # Panics
    ///
    /// Panics if the clipboard provider cannot be initialized. This can happen in environments
    /// without a graphical user interface or due to system-level permission issues.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tessera_ui::clipboard::Clipboard;
    ///
    /// let clipboard = Clipboard::new();
    /// ```
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "android"))]
            manager: arboard::Clipboard::new().expect("Failed to create clipboard"),
        }
    }

    /// Sets the clipboard text, overwriting any previous content.
    ///
    /// On unsupported platforms like Android, this operation is a no-op and will log a warning.
    ///
    /// # Arguments
    ///
    /// * `text` - The string slice to be copied to the clipboard.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tessera_ui::clipboard::Clipboard;
    ///
    /// let mut clipboard = Clipboard::new();
    /// clipboard.set_text("Hello, world!");
    /// ```
    pub fn set_text(&mut self, text: &str) {
        #[cfg(not(target_os = "android"))]
        {
            let _ = self.manager.set_text(text.to_string());
        }
        #[cfg(target_os = "android")]
        {
            // Android-specific clipboard handling can be implemented here
            // For now, we do nothing as clipboard is not supported on Android
            log::warn!("Clipboard operations are not supported on Android");
        }
    }

    /// Gets the current text content from the clipboard.
    ///
    /// This method retrieves text from the clipboard. If the clipboard is empty, contains
    /// non-text content, or an error occurs, it returns `None`.
    ///
    /// On unsupported platforms like Android, this always returns `None` and logs a warning.
    ///
    /// # Returns
    ///
    /// - `Some(String)` if text is successfully retrieved from the clipboard.
    /// - `None` if the clipboard is empty, contains non-text data, or an error occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tessera_ui::clipboard::Clipboard;
    ///
    /// let mut clipboard = Clipboard::new();
    /// clipboard.set_text("Hello, Tessera!");
    ///
    /// if let Some(text) = clipboard.get_text() {
    ///     println!("Retrieved from clipboard: {}", text);
    /// } else {
    ///     println!("Clipboard was empty or contained non-text content.");
    /// }
    /// ```
    pub fn get_text(&mut self) -> Option<String> {
        #[cfg(not(target_os = "android"))]
        {
            self.manager.get_text().ok()
        }
        #[cfg(target_os = "android")]
        {
            // Android-specific clipboard handling can be implemented here
            // For now, we return None as clipboard is not supported on Android
            log::warn!("Clipboard operations are not supported on Android");
            None
        }
    }
}
