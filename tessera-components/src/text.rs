//! Single-style text rendering.
//!
//! ## Usage
//!
//! Display labels, headings, and other text content.
use tessera_ui::{
    Color, ComputedData, Dp, LayoutInput, LayoutOutput, LayoutPolicy, MeasurementError, Modifier,
    Px, PxPosition, RenderInput, RenderPolicy, accesskit::Role, layout::layout_primitive, tessera,
    use_context,
};

use crate::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pipelines::text::{
        command::{TextCommand, TextConstraint},
        pipeline::TextData,
    },
    theme::{ContentColor, MaterialTheme, TextStyle},
};

pub use crate::pipelines::text::pipeline::{read_font_system, write_font_system};

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
/// - `modifier` — modifier chain applied to the text node.
/// - `content` — text content to display.
/// - `color` — optional text color override.
/// - `size` — optional font size override.
/// - `line_height` — optional line height override.
/// - `accessibility_label` — optional accessibility label override.
/// - `accessibility_description` — optional accessibility description override.
///
/// ## Examples
///
/// ```
/// use tessera_components::text::text;
/// use tessera_ui::{Color, Dp, tessera};
///
/// #[tessera]
/// fn demo() {
///     text()
///         .content("Hello, world!")
///         .color(Color::new(0.2, 0.5, 0.8, 1.0))
///         .size(Dp(32.0));
/// }
///
/// demo();
/// ```
#[tessera]
pub fn text(
    modifier: Modifier,
    #[prop(into)] content: String,
    color: Option<Color>,
    size: Option<Dp>,
    line_height: Option<Dp>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let theme = use_context::<MaterialTheme>();
    let color = color
        .or_else(|| use_context::<ContentColor>().map(|c| c.get().current))
        .or_else(|| theme.map(|t| t.get().color_scheme.on_surface))
        .unwrap_or_else(|| ContentColor::default().current);
    let size = size
        .or_else(|| use_context::<TextStyle>().map(|s| s.get().font_size))
        .or_else(|| theme.map(|t| t.get().typography.body_large.font_size))
        .unwrap_or_else(|| TextStyle::default().font_size);
    let accessibility_label = accessibility_label
        .clone()
        .or_else(|| (!content.is_empty()).then(|| content.clone()));
    let semantics = SemanticsArgs {
        role: Some(Role::Label),
        label: accessibility_label,
        description: accessibility_description,
        ..Default::default()
    };
    let inherited_style = use_context::<TextStyle>().map(|s| s.get());
    let line_height = line_height
        .or_else(|| inherited_style.and_then(|style| style.line_height))
        .unwrap_or(Dp(size.0 * 1.2));

    let policy = TextLayout {
        text: content.clone(),
        color,
        size,
        line_height,
    };
    layout_primitive()
        .modifier(modifier.semantics(semantics))
        .layout_policy(policy.clone())
        .render_policy(policy);
}

#[derive(Clone)]
struct TextLayout {
    text: String,
    color: Color,
    size: Dp,
    line_height: Dp,
}

impl PartialEq for TextLayout {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.color == other.color
            && self.size == other.size
            && self.line_height == other.line_height
    }
}

impl LayoutPolicy for TextLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let max_width = input.parent_constraint().width().resolve_max();
        let max_height = input.parent_constraint().height().resolve_max();

        let info = TextData::measure(
            self.text.clone(),
            self.color,
            self.size.to_pixels_f32(),
            self.line_height.to_pixels_f32(),
            TextConstraint {
                max_width: max_width.map(|px: Px| px.to_f32()),
                max_height: max_height.map(|px: Px| px.to_f32()),
            },
        );

        Ok(ComputedData {
            width: info.size[0].into(),
            height: info.size[1].into(),
        })
    }
}

impl RenderPolicy for TextLayout {
    fn record(&self, input: &RenderInput<'_>) {
        let metadata = input.metadata_mut();
        let computed = metadata
            .computed_data()
            .expect("ComputedData must exist during record");
        drop(metadata);

        // Use TextData::get() with the computed bounds to retrieve cached data
        let text_data = TextData::get(
            self.text.clone(),
            self.color,
            self.size.to_pixels_f32(),
            self.line_height.to_pixels_f32(),
            [computed.width.raw() as u32, computed.height.raw() as u32],
        );

        let drawable = TextCommand {
            data: text_data,
            offset: PxPosition::ZERO,
        };
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(drawable);
    }
}
