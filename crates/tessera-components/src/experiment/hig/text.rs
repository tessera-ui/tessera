//! HIG text labels and headings.
//!
//! ## Usage
//!
//! Display readable labels, descriptions, and headings throughout an interface.

use std::sync::Arc;

use tessera_ui::{Color, Dp, Modifier, tessera, use_context};

use crate::text_shared::{TextMetrics, text_node};

pub use crate::pipelines::text::pipeline::{read_font_system, write_font_system};

use super::theme::{ContentColor, HigTheme, TextStyle};

/// Shared string content accepted by [`text`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextContent(Arc<str>);

impl<T: Into<String>> From<T> for TextContent {
    fn from(value: T) -> Self {
        Self(value.into().into())
    }
}

pub(super) fn metrics(style: TextStyle, size: Option<Dp>, line_height: Option<Dp>) -> TextMetrics {
    let font_size = size.unwrap_or(style.font_size);
    TextMetrics {
        font_size,
        line_height: line_height.unwrap_or(style.line_height),
        weight: style.font_weight,
        letter_spacing: style.letter_spacing,
        family: style.font_family,
        optical_size: Some(font_size.0.max(17.0) as f32),
        point_size: Some(font_size.0 as f32),
    }
}

/// # text
///
/// Displays a HIG label or heading with inherited typography and foreground color.
///
/// ## Usage
///
/// Describe controls, content, and sections using noneditable text.
///
/// ## Parameters
///
/// - `content` — displayed text.
/// - `modifier` — layout and semantics; existing caller semantics remain intact.
/// - `style` — typography override, otherwise inherited or 13dp system text.
/// - `color` — foreground override, otherwise inherited or the theme's label color.
/// - `size`, `line_height` — individual typography overrides.
/// - `single_line` — ellipsize overflow instead of wrapping.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::text::text;
/// use tessera_ui::{Modifier, tessera};
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
///
/// #[tessera]
/// fn app() {
///     text("Settings").modifier(Modifier::new().semantics(SemanticsArgs {
///         test_tag: Some("title".into()),
///         ..Default::default()
///     }));
/// }
/// let frame = tessera_ui::testing::UiTest::new().content(app).frame();
/// assert!(frame.node("title").bounds().height.0 > 0);
/// ```
#[tessera]
pub fn text(
    #[prop(into)] content: TextContent,
    modifier: Option<Modifier>,
    style: Option<TextStyle>,
    color: Option<Color>,
    size: Option<Dp>,
    line_height: Option<Dp>,
    single_line: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let style = if let Some(style) = style {
        style
    } else {
        use_context::<TextStyle>()
            .map(|style| style.get())
            .unwrap_or_default()
    };
    let color = if let Some(color) = color {
        color
    } else if let Some(color) = use_context::<ContentColor>() {
        color.get().0
    } else {
        use_context::<HigTheme>()
            .map(|theme| theme.get())
            .unwrap_or_default()
            .label_color()
    };
    text_node(
        content.0,
        modifier.unwrap_or_default(),
        color,
        metrics(style, size, line_height),
        single_line.unwrap_or(false),
        true,
    )
    .label_optional(accessibility_label)
    .description_optional(accessibility_description);
}

#[cfg(test)]
mod tests {
    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::{
        State, provide_context, remember,
        testing::{TestProbe, UiTest, WorkPhase},
    };

    use crate::column::column;

    use super::*;

    #[tessera]
    fn inherited_fixture(probe: TestProbe<Option<(State<TextStyle>, State<ContentColor>)>>) {
        let style = remember(TextStyle::default);
        let color = remember(|| ContentColor(Color::BLACK));
        probe.set(Some((style, color)));
        provide_context(style.get(), move || {
            provide_context(color.get(), move || {
                column().children(|| {
                    text("Inherited label").modifier(Modifier::new().semantics(SemanticsArgs {
                        test_tag: Some("inherited".into()),
                        ..Default::default()
                    }));
                    text("Explicit label")
                        .style(TextStyle::default())
                        .color(Color::BLACK)
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some("explicit".into()),
                            ..Default::default()
                        }));
                });
            });
        });
    }

    #[test]
    fn inherited_typography_remeasures_without_changing_explicit_text() {
        let probe = TestProbe::default();
        let mut ui = UiTest::new().content(|| inherited_fixture(probe.clone()));
        let before = ui.frame();
        let inherited = before.node("inherited").bounds();
        let explicit = before.node("explicit").bounds();
        let (style, _) = probe.get().expect("fixture state");
        style.with_mut(|style| {
            style.font_size = Dp(26.0);
            style.line_height = Dp(32.0);
        });
        let after = ui.frames(1);
        assert!(after.node("inherited").bounds().height > inherited.height);
        assert_eq!(after.node("explicit").bounds().height, explicit.height);
        assert_eq!(ui.work().count("text", WorkPhase::Build), 1);
    }

    #[test]
    fn inherited_foreground_updates_do_not_invalidate_text_measurement() {
        let probe = TestProbe::default();
        let mut ui = UiTest::new().content(|| inherited_fixture(probe.clone()));
        let before = ui.frame().node("inherited").bounds();
        probe
            .get()
            .expect("fixture state")
            .1
            .set(ContentColor(Color::WHITE));
        assert_eq!(ui.frames(1).node("inherited").bounds(), before);
        assert_eq!(ui.work().phase(WorkPhase::Measure).count(), 0);
        assert_eq!(ui.work().count("text", WorkPhase::Build), 1);
    }
}
