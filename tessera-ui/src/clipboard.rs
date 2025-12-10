//! Provides a cross-platform clipboard manager for text manipulation.
//!
//! This module offers a simple, unified interface for interacting with the
//! system clipboard, allowing applications to easily get and set text content.
//! It abstracts platform-specific details, providing a consistent API across
//! different operating systems.
//!
//! # Key Features
//!
//! - **Set Text**: Place a string onto the system clipboard.
//! - **Get Text**: Retrieve the current text content from the system clipboard.
//! - **Cross-platform**: Uses `arboard` for broad platform support (Windows,
//!   macOS, Linux).
//! - **Graceful Fallback**: On unsupported platforms like Android, operations
//!   are no-ops that log a warning, preventing crashes.
//!
//! # Usage
//!
//! The main entry point is the [`Clipboard`] struct, which provides methods to
//! interact with the system clipboard.
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
//! Clipboard operations are currently not supported on Android. Any calls to
//! `set_text` or `get_text` on Android will result in a warning log and will
//! not perform any action.
#[cfg(target_os = "android")]
use jni::{
    JNIEnv,
    objects::{JObject, JString, JValue},
};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// Manages access to the system clipboard for text-based copy and paste
/// operations.
///
/// This struct acts as a handle to the platform's native clipboard, abstracting
/// away the underlying implementation details. It is created using
/// [`Clipboard::new()`].
///
/// All interactions are synchronous. For unsupported platforms (e.g., Android),
/// operations are gracefully handled to prevent runtime errors.
pub struct Clipboard {
    #[cfg(not(target_os = "android"))]
    /// The clipboard manager for handling clipboard operations.
    manager: arboard::Clipboard,
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
}

#[cfg(not(target_os = "android"))]
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
    #[cfg(not(target_os = "android"))]
    /// Creates a new clipboard instance, initializing the connection to the
    /// system clipboard.
    ///
    /// This method may fail if the system clipboard is unavailable, in which
    /// case it will panic.
    ///
    /// # Panics
    ///
    /// Panics if the clipboard provider cannot be initialized. This can happen
    /// in environments without a graphical user interface or due to
    /// system-level permission issues.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tessera_ui::clipboard::Clipboard;
    ///
    /// let clipboard = Clipboard::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: arboard::Clipboard::new().expect("Failed to create clipboard"),
        }
    }

    #[cfg(target_os = "android")]
    /// Creates a new clipboard instance, initializing the connection to the
    /// system clipboard.
    pub fn new(android_app: AndroidApp) -> Self {
        Self { android_app }
    }

    /// Sets the clipboard text, overwriting any previous content.
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
            set_clipboard_text(&self.android_app, text);
        }
    }

    /// Gets the current text content from the clipboard.
    ///
    /// This method retrieves text from the clipboard. If the clipboard is
    /// empty, contains non-text content, or an error occurs, it returns
    /// `None`.
    ///
    /// # Returns
    ///
    /// - `Some(String)` if text is successfully retrieved from the clipboard.
    /// - `None` if the clipboard is empty, contains non-text data, or an error
    ///   occurs.
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
            get_clipboard_text(&self.android_app)
        }
    }

    /// Clears the clipboard content.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tessera_ui::clipboard::Clipboard;
    ///
    /// let mut clipboard = Clipboard::new();
    /// clipboard.set_text("Temporary text"); // "Temporary text" is now in the clipboard
    /// clipboard.clear(); // The clipboard is now cleared
    /// ```
    pub fn clear(&mut self) {
        #[cfg(not(target_os = "android"))]
        {
            let _ = self.manager.clear();
        }
        #[cfg(target_os = "android")]
        {
            clear_clipboard(&self.android_app);
        }
    }
}

/// Helper function: Get ClipboardManager instance
#[cfg(target_os = "android")]
fn get_clipboard_manager<'a>(env: &mut JNIEnv<'a>, activity: &JObject<'a>) -> Option<JObject<'a>> {
    // Get service using "clipboard" string directly
    let service_name = env.new_string("clipboard").ok()?;
    let clipboard_manager = env
        .call_method(
            activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::from(&service_name)],
        )
        .ok()?
        .l()
        .ok()?;
    Some(clipboard_manager)
}

/// Retrieves text from the Android system clipboard.
///
/// ## Parameters
/// - `android_app`: A reference to the Android application context.
///
/// ## Returns
/// - `Some(String)`: If text content is successfully read.
/// - `None`: If the clipboard is empty, the content is not plain text, or any
///   error occurs during the process.
#[cfg(target_os = "android")]
fn get_clipboard_text(android_app: &AndroidApp) -> Option<String> {
    // 1. Get JNI environment and Activity object
    let jvm = unsafe { jni::JavaVM::from_raw(android_app.vm_as_ptr().cast()).ok()? };
    let mut env = jvm.attach_current_thread().ok()?;
    let activity = unsafe { JObject::from_raw(android_app.activity_as_ptr().cast()) };

    // 2. Get ClipboardManager
    let clipboard_manager = get_clipboard_manager(&mut env, &activity)?;

    // 3. Get Primary Clip content
    let clip_data = env
        .call_method(
            &clipboard_manager,
            "getPrimaryClip",
            "()Landroid/content/ClipData;",
            &[],
        )
        .ok()?
        .l()
        .ok()?;

    if clip_data.is_null() {
        return None;
    }

    let item = env
        .call_method(
            &clip_data,
            "getItemAt",
            "(I)Landroid/content/ClipData$Item;",
            &[JValue::from(0)],
        )
        .ok()?
        .l()
        .ok()?;

    if item.is_null() {
        return None;
    }

    // 4. Use coerceToText to force convert item content to text
    let char_seq = env
        .call_method(
            &item,
            "coerceToText",
            "(Landroid/content/Context;)Ljava/lang/CharSequence;",
            &[JValue::from(&activity)],
        )
        .ok()?
        .l()
        .ok()?;

    if char_seq.is_null() {
        return None;
    }

    // 5. Convert CharSequence to Rust String
    let j_string = env
        .call_method(&char_seq, "toString", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;
    let rust_string: String = env.get_string(&JString::from(j_string)).ok()?.into();

    Some(rust_string)
}

/// Sets text to the Android system clipboard.
///
/// ## Parameters
/// - `android_app`: A reference to the Android application context.
/// - `text`: The text to be set to the clipboard.
#[cfg(target_os = "android")]
fn set_clipboard_text(android_app: &AndroidApp, text: &str) {
    let jvm = match unsafe { jni::JavaVM::from_raw(android_app.vm_as_ptr().cast()) } {
        Ok(jvm) => jvm,
        Err(_) => return,
    };
    let mut env = match jvm.attach_current_thread() {
        Ok(env) => env,
        Err(_) => return,
    };
    let activity = unsafe { JObject::from_raw(android_app.activity_as_ptr().cast()) };

    let clipboard_manager = match get_clipboard_manager(&mut env, &activity) {
        Some(manager) => manager,
        None => return,
    };

    let label = match env.new_string("label") {
        Ok(s) => s,
        Err(_) => return,
    };
    let text_to_set = match env.new_string(text) {
        Ok(s) => s,
        Err(_) => return,
    };

    let clip_data = match env.call_static_method(
        "android/content/ClipData",
        "newPlainText",
        "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;",
        &[JValue::from(&label), JValue::from(&text_to_set)],
    ) {
        Ok(c) => match c.l() {
            Ok(clip) => clip,
            Err(_) => return,
        },
        Err(_) => return,
    };

    // Call setPrimaryClip, ignoring return value
    let _ = env.call_method(
        &clipboard_manager,
        "setPrimaryClip",
        "(Landroid/content/ClipData;)V",
        &[JValue::from(&clip_data)],
    );
}

/// Clears the Android system clipboard.
///
/// ## Parameters
/// - `android_app`: A reference to the Android application context.
#[cfg(target_os = "android")]
fn clear_clipboard(android_app: &AndroidApp) {
    let jvm = match unsafe { jni::JavaVM::from_raw(android_app.vm_as_ptr().cast()) } {
        Ok(jvm) => jvm,
        Err(_) => return,
    };
    let mut env = match jvm.attach_current_thread() {
        Ok(env) => env,
        Err(_) => return,
    };
    let activity = unsafe { JObject::from_raw(android_app.activity_as_ptr().cast()) };

    if let Some(clipboard_manager) = get_clipboard_manager(&mut env, &activity) {
        // Call clearPrimaryClip, ignoring return value
        let _ = env.call_method(&clipboard_manager, "clearPrimaryClip", "()V", &[]);
    }
}
