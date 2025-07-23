//! Text component module for Tessera UI.
//!
//! This module provides the [`text`] component and its configuration types for rendering styled, single-style text within the Tessera UI framework.
//! It is designed for displaying static or dynamic text content with customizable color, font size, and line height, supporting Unicode and DPI scaling.
//!
//! Typical use cases include labels, headings, captions, and any UI element requiring straightforward text rendering.
//! The component is stateless and integrates with Tessera's layout and rendering systems, automatically adapting to parent constraints and device pixel density.
//!
//! The builder-pattern [`TextArgs`] struct allows ergonomic and flexible configuration, with sensible defaults for most properties.
//! Conversions from `String` and `&str` are supported for convenience.
//!
//! # Examples
//!
//! Basic usage:
//! ```
//! use tessera_ui_basic_components::text::{text, TextArgs};
//! text("Hello, Tessera!");
//! ```
//!
//! Custom styling:
//! ```
//! use tessera_ui_basic_components::text::{text, TextArgsBuilder};
//! use tessera_ui::{Color, Dp};
//! let args = TextArgsBuilder::default()
//!     .text("Styled".to_string())
//!     .color(Color::from_rgb(0.2, 0.4, 0.8))
//!     .size(Dp(32.0))
//!     .build()
//!     .unwrap();
//! text(args);
//! ```
use derive_builder::Builder;
use tessera_ui::{Color, ComputedData, DimensionValue, Dp, Px};
use tessera_ui_macros::tessera;

use crate::pipelines::{TextCommand, TextConstraint, TextData};

/// Configuration arguments for the `text` component.
///
/// `TextArgs` defines the visual properties and content for rendering text in the Tessera UI framework.
/// It uses the builder pattern for convenient construction and provides sensible defaults for all styling properties.
///
/// # Fields
///
/// - `text`: The string content to be displayed
/// - `color`: Text color (defaults to black)
/// - `size`: Font size in density-independent pixels (defaults to 25.0 dp)
/// - `line_height`: Optional line height override (defaults to 1.2 × font size)
///
/// # Builder Pattern
///
/// This struct uses the `derive_builder` crate to provide a fluent builder API. All fields except `text`
/// have sensible defaults, making it easy to create text with minimal configuration.
///
/// # Examples
///
/// ## Basic text with defaults
/// ```
/// use tessera_ui_basic_components::text::{TextArgs, TextArgsBuilder};
///
/// let args = TextArgsBuilder::default()
///     .text("Hello, World!".to_string())
///     .build()
///     .unwrap();
/// // Uses: black color, 25.0 dp size, 30.0 dp line height (1.2 × size)
/// ```
///
/// ## Customized text styling
/// ```
/// use tessera_ui_basic_components::text::{TextArgs, TextArgsBuilder};
/// use tessera_ui::{Color, Dp};
///
/// let args = TextArgsBuilder::default()
///     .text("Styled Text".to_string())
///     .color(Color::from_rgb(0.2, 0.4, 0.8)) // Blue color
///     .size(Dp(32.0))                        // Larger font
///     .line_height(Dp(40.0))                 // Custom line height
///     .build()
///     .unwrap();
/// ```
///
/// ## Using automatic line height calculation
/// ```
/// use tessera_ui_basic_components::text::{TextArgs, TextArgsBuilder};
/// use tessera_ui::Dp;
///
/// let args = TextArgsBuilder::default()
///     .text("Auto Line Height".to_string())
///     .size(Dp(50.0))
///     // line_height will automatically be Dp(60.0) (1.2 × 50.0)
///     .build()
///     .unwrap();
/// ```
///
/// ## Converting from string types
/// ```
/// use tessera_ui_basic_components::text::TextArgs;
///
/// // From String
/// let args1: TextArgs = "Hello".to_string().into();
///
/// // From &str
/// let args2: TextArgs = "World".into();
/// ```
#[derive(Debug, Default, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TextArgs {
    /// The text content to be rendered.
    ///
    /// This is the actual string that will be displayed on screen. It can contain
    /// Unicode characters and will be rendered using the specified font properties.
    pub text: String,

    /// The color of the text.
    ///
    /// Defaults to `Color::BLACK` if not specified. The color is applied uniformly
    /// to all characters in the text string.
    #[builder(default = "Color::BLACK")]
    pub color: Color,

    /// The font size in density-independent pixels (dp).
    ///
    /// Defaults to `Dp(25.0)` if not specified. This size is automatically scaled
    /// based on the device's pixel density to ensure consistent visual appearance
    /// across different screen densities.
    #[builder(default = "Dp(25.0)")]
    pub size: Dp,

    /// Optional override for line height in density-independent pixels (dp).
    ///
    /// If not specified (None), the line height will automatically be calculated as
    /// 1.2 times the font size, which provides good readability for most text.
    ///
    /// Set this to a specific value if you need precise control over line spacing,
    /// such as for dense layouts or specific design requirements.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::text::TextArgsBuilder;
    /// use tessera_ui::Dp;
    ///
    /// // Automatic line height (1.2 × size)
    /// let auto = TextArgsBuilder::default()
    ///     .text("Auto spacing".to_string())
    ///     .size(Dp(20.0))  // line_height will be Dp(24.0)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Custom line height
    /// let custom = TextArgsBuilder::default()
    ///     .text("Custom spacing".to_string())
    ///     .size(Dp(20.0))
    ///     .line_height(Dp(30.0))  // Explicit line height
    ///     .build()
    ///     .unwrap();
    /// ```
    #[builder(default, setter(strip_option))]
    pub line_height: Option<Dp>,
}

/// Converts a [`String`] into [`TextArgs`] using the builder pattern.
///
/// This allows convenient usage of string literals or owned strings as text arguments
/// for the [`text`] component.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::text::TextArgs;
///
/// let args: TextArgs = "Hello, Tessera!".to_string().into();
/// ```
impl From<String> for TextArgs {
    fn from(val: String) -> Self {
        TextArgsBuilder::default().text(val).build().unwrap()
    }
}

/// Converts a string slice (`&str`) into [`TextArgs`] using the builder pattern.
///
/// This enables ergonomic conversion from string literals for the [`text`] component.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::text::TextArgs;
///
/// let args: TextArgs = "Quick text".into();
/// ```
impl From<&str> for TextArgs {
    fn from(val: &str) -> Self {
        TextArgsBuilder::default()
            .text(val.to_string())
            .build()
            .unwrap()
    }
}

/// Basic text component.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::text::{text, TextArgs, TextArgsBuilder};
/// use tessera_ui::Dp;
/// // a simple hello world text, in black
/// let args = TextArgsBuilder::default()
///     .text("Hello, World!".to_string())
///     .size(Dp(50.0)) // Example using Dp
///     // line_height will be Dp(60.0) (1.2 * size) by default
///     .build()
///     .unwrap();
/// text(args);
/// ```
#[tessera]
pub fn text(args: impl Into<TextArgs>) {
    let text_args: TextArgs = args.into();
    measure(Box::new(move |input| {
        let max_width: Option<Px> = match input.parent_constraint.width {
            DimensionValue::Fixed(w) => Some(w),
            DimensionValue::Wrap { max, .. } => max, // Use max from Wrap
            DimensionValue::Fill { max, .. } => max, // Use max from Fill
        };

        let max_height: Option<Px> = match input.parent_constraint.height {
            DimensionValue::Fixed(h) => Some(h),
            DimensionValue::Wrap { max, .. } => max, // Use max from Wrap
            DimensionValue::Fill { max, .. } => max, // Use max from Fill
        };

        let line_height = text_args.line_height.unwrap_or(Dp(text_args.size.0 * 1.2));

        let text_data = TextData::new(
            text_args.text.clone(),
            text_args.color,
            text_args.size.to_pixels_f32(),
            line_height.to_pixels_f32(),
            TextConstraint {
                max_width: max_width.map(|px| px.to_f32()),
                max_height: max_height.map(|px| px.to_f32()),
            },
        );

        let size = text_data.size;
        let drawable = TextCommand { data: text_data };

        // Use the new unified command system to add the text rendering command
        input.metadata_mut().push_draw_command(drawable);

        Ok(ComputedData {
            width: size[0].into(),
            height: size[1].into(),
        })
    }));
}
