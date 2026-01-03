//! Single-style text rendering.
//!
//! ## Usage
//!
//! Display labels, headings, and other text content.
use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, Modifier, Px, PxPosition, RenderInput, State, accesskit::Role, remember,
    tessera, use_context,
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
/// use tessera_ui::{Color, Dp, tessera};
/// use tessera_ui_basic_components::text::{TextArgs, text};
///
/// #[tessera]
/// fn demo() {
///     let args = TextArgs::default()
///         .text("Hello, world!")
///         .color(Color::new(0.2, 0.5, 0.8, 1.0))
///         .size(Dp(32.0));
///     assert_eq!(args.text, "Hello, world!");
///     text(args);
/// }
///
/// demo();
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

    let line_height = text_args
        .line_height
        .or(inherited_style.line_height)
        .unwrap_or(Dp(text_args.size.0 * 1.2));
    let cache = remember(TextLayoutCache::default);

    layout(TextLayout {
        text: text_args.text,
        color: text_args.color,
        size: text_args.size,
        line_height,
        cache,
    });
}

#[derive(Clone, Default)]
struct TextLayoutCache {
    data: Option<TextData>,
}

#[derive(Clone)]
struct TextLayout {
    text: String,
    color: Color,
    size: Dp,
    line_height: Dp,
    cache: State<TextLayoutCache>,
}

impl PartialEq for TextLayout {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.color == other.color
            && self.size == other.size
            && self.line_height == other.line_height
    }
}

impl LayoutSpec for TextLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let max_width: Option<Px> = match input.parent_constraint().width() {
            DimensionValue::Fixed(w) => Some(w),
            DimensionValue::Wrap { max, .. } => max,
            DimensionValue::Fill { max, .. } => max,
        };

        let max_height: Option<Px> = match input.parent_constraint().height() {
            DimensionValue::Fixed(h) => Some(h),
            DimensionValue::Wrap { max, .. } => max,
            DimensionValue::Fill { max, .. } => max,
        };

        let text_data = TextData::new(
            self.text.clone(),
            self.color,
            self.size.to_pixels_f32(),
            self.line_height.to_pixels_f32(),
            TextConstraint {
                max_width: max_width.map(|px| px.to_f32()),
                max_height: max_height.map(|px| px.to_f32()),
            },
        );

        let size = text_data.size;
        self.cache.with_mut(|cache| cache.data = Some(text_data));
        Ok(ComputedData {
            width: size[0].into(),
            height: size[1].into(),
        })
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        let text_data = self
            .cache
            .with(|cache| cache.data.clone())
            .expect("Text cache must be populated before record");
        let drawable = TextCommand {
            data: text_data,
            offset: PxPosition::ZERO,
        };
        metadata.push_draw_command(drawable);
    }
}
