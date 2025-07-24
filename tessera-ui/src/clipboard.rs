pub struct Clipboard {
    #[cfg(not(target_os = "android"))]
    /// The clipboard manager for handling clipboard operations.
    manager: arboard::Clipboard,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    /// Creates a new clipboard instance.
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "android"))]
            manager: arboard::Clipboard::new().expect("Failed to create clipboard"),
        }
    }

    /// Sets the clipboard text.
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

    /// Gets the clipboard text.
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
