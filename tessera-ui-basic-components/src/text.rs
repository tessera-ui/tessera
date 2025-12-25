//! Single-style text rendering.
//!
//! ## Usage
//!
//! Display labels, headings, and other text content.
use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, DimensionValue, Dp, Modifier, Px, PxPosition, accesskit::Role, tessera,
    use_context,
};

use crate::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pipelines::text::{
        command::{TextCommand, TextConstraint},
        pipeline::TextData,
    },
    theme::{ContentColor, TextStyle},
};

pub use crate::pipelines::text::pipeline::{read_font_system, write_font_system};

/// Configuration arguments for the `text` component.
#[derive(Debug, Setters, Clone)]
pub struct TextArgs {
    /// Optional modifier chain applied to the text.
    pub modifier: Modifier,

    /// The text content to be rendered.
    #[setters(into)]
    pub text: String,

    /// The color of the text.
    pub color: Color,

    /// The font size in density-independent pixels (dp).
    pub size: Dp,

    /// Optional override for line height in density-independent pixels (dp).
    #[setters(strip_option)]
    pub line_height: Option<Dp>,

    /// Optional label announced by assistive technologies. Defaults to the text
    /// content.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,

    /// Optional description announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl Default for TextArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            text: String::new(),
            color: use_context::<ContentColor>().get().current,
            size: use_context::<TextStyle>().get().font_size,
            line_height: None,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
}

impl From<String> for TextArgs {
    fn from(val: String) -> Self {
        TextArgs::default().text(val)
    }
}

impl From<&str> for TextArgs {
    fn from(val: &str) -> Self {
        TextArgs::default().text(val)
    }
}

/// # text
///
/// Renders a block of text with a single, uniform style.
///
/// ## Usage
///
/// Display simple text content. For more complex styling or editing, see other
/// components.
///
/// ## Parameters
///
/// - `args` — configures the text content and styling; see [`TextArgs`]. Can be
///   converted from a `String` or `&str`.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Color, Dp};
/// use tessera_ui_basic_components::text::{TextArgs, text};
///
/// // Simple text from a string literal
/// text("Hello, world!");
///
/// // Styled text using fluent setters
/// text(
///     TextArgs::default()
///         .text("Styled Text")
///         .color(Color::new(0.2, 0.5, 0.8, 1.0))
///         .size(Dp(32.0)),
/// );
/// ```
#[tessera]
pub fn text(args: impl Into<TextArgs>) {
    let text_args: TextArgs = args.into();
    let accessibility_label = text_args
        .accessibility_label
        .clone()
        .or_else(|| (!text_args.text.is_empty()).then(|| text_args.text.clone()));
    let accessibility_description = text_args.accessibility_description.clone();
    let mut semantics = SemanticsArgs::new().role(Role::Label);
    if let Some(label) = accessibility_label {
        semantics = semantics.label(label);
    }
    if let Some(description) = accessibility_description {
        semantics = semantics.description(description);
    }
    text_args.modifier.semantics(semantics).run(move || {
        text_inner(text_args);
    });
}

#[tessera]
fn text_inner(text_args: TextArgs) {
    let inherited_style = use_context::<TextStyle>().get();

    measure(Box::new(move |input| {
        let max_width: Option<Px> = match input.parent_constraint.width() {
            DimensionValue::Fixed(w) => Some(w),
            DimensionValue::Wrap { max, .. } => max, // Use max from Wrap
            DimensionValue::Fill { max, .. } => max, // Use max from Fill
        };

        let max_height: Option<Px> = match input.parent_constraint.height() {
            DimensionValue::Fixed(h) => Some(h),
            DimensionValue::Wrap { max, .. } => max, // Use max from Wrap
            DimensionValue::Fill { max, .. } => max, // Use max from Fill
        };

        let line_height = text_args
            .line_height
            .or(inherited_style.line_height)
            .unwrap_or(Dp(text_args.size.0 * 1.2));

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
        let drawable = TextCommand {
            data: text_data,
            offset: PxPosition::ZERO,
        };

        input.metadata_mut().push_draw_command(drawable);

        Ok(ComputedData {
            width: size[0].into(),
            height: size[1].into(),
        })
    }));
}
