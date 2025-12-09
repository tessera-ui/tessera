//! A component for rendering single-style text.
//!
//! ## Usage
//!
//! Use to display labels, headings, or other static or dynamic text content.
use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, DimensionValue, Dp, Px, accesskit::Role, tessera, use_context,
};

use crate::{
    pipelines::text::{
        command::{TextCommand, TextConstraint},
        pipeline::TextData,
    },
    theme::ContentColor,
};

pub use crate::pipelines::text::pipeline::{read_font_system, write_font_system};

/// Configuration arguments for the `text` component.
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TextArgs {
    /// The text content to be rendered.
    #[builder(setter(into))]
    pub text: String,

    /// The color of the text.
    #[builder(default = "use_context::<ContentColor>().current")]
    pub color: Color,

    /// The font size in density-independent pixels (dp).
    #[builder(default = "Dp(25.0)")]
    pub size: Dp,

    /// Optional override for line height in density-independent pixels (dp).
    #[builder(default, setter(strip_option))]
    pub line_height: Option<Dp>,

    /// Optional label announced by assistive technologies. Defaults to the text content.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,

    /// Optional description announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl Default for TextArgs {
    fn default() -> Self {
        TextArgsBuilder::default()
            .text("")
            .build()
            .expect("builder construction failed")
    }
}

impl From<String> for TextArgs {
    fn from(val: String) -> Self {
        TextArgsBuilder::default()
            .text(val)
            .build()
            .expect("builder construction failed")
    }
}

impl From<&str> for TextArgs {
    fn from(val: &str) -> Self {
        TextArgsBuilder::default()
            .text(val.to_string())
            .build()
            .expect("builder construction failed")
    }
}

/// # text
///
/// Renders a block of text with a single, uniform style.
///
/// ## Usage
///
/// Display simple text content. For more complex styling or editing, see other components.
///
/// ## Parameters
///
/// - `args` — configures the text content and styling; see [`TextArgs`]. Can be converted from a `String` or `&str`.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Color, Dp};
/// use tessera_ui_basic_components::text::{TextArgsBuilder, text};
///
/// // Simple text from a string literal
/// text("Hello, world!");
///
/// // Styled text using the builder
/// text(
///     TextArgsBuilder::default()
///         .text("Styled Text")
///         .color(Color::new(0.2, 0.5, 0.8, 1.0))
///         .size(Dp(32.0))
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn text(args: impl Into<TextArgs>) {
    let text_args: TextArgs = args.into();
    let accessibility_label = text_args.accessibility_label.clone();
    let accessibility_description = text_args.accessibility_description.clone();
    let text_for_accessibility = text_args.text.clone();

    input_handler(Box::new(move |input| {
        let mut builder = input.accessibility().role(Role::Label);

        if let Some(label) = accessibility_label.as_ref() {
            builder = builder.label(label.clone());
        } else if !text_for_accessibility.is_empty() {
            builder = builder.label(text_for_accessibility.clone());
        }

        if let Some(description) = accessibility_description.as_ref() {
            builder = builder.description(description.clone());
        }

        builder.commit();
    }));
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
