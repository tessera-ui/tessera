//! Single-style text rendering.
//!
//! ## Usage
//!
//! Display labels, headings, and other text content.
use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Color, ComputedData, Dp, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode, accesskit::Role,
    layout::MeasureScope, modifier::ModifierCapabilityExt as _, remember, tessera, use_context,
};

use crate::{
    pipelines::text::{
        command::{TextCommand, TextConstraint},
        pipeline::{TextData, TextFormat, TextMeasureInfo},
    },
    theme::{ContentColor, MaterialTheme, TextStyle},
};

pub use crate::pipelines::text::pipeline::{read_font_system, write_font_system};

/// Shared string content accepted by [`text`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextContent(Arc<str>);

impl<T: Into<String>> From<T> for TextContent {
    fn from(value: T) -> Self {
        Self(value.into().into())
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
/// - `content` — required text content to display.
/// - `modifier` — modifier chain applied to the text node; caller semantics
///   remain intact. The default label is the content and the default role is a
///   label, unless supplied by the modifier.
/// - `color` — optional text color override.
/// - `style` — optional text style override for typography presets.
/// - `size` — optional font size override.
/// - `line_height` — optional line height override.
/// - `single_line` — keep text on one line and ellipsize overflow.
/// - `accessibility_label` — optional label override, taking precedence over
///   modifier semantics and displayed content.
/// - `accessibility_description` — optional description override, taking
///   precedence over modifier semantics.
///
/// ## Examples
///
/// ```
/// use tessera_components::text::text;
/// use tessera_ui::{Color, Dp, tessera};
///
/// #[tessera]
/// fn demo() {
///     text("Hello, world!")
///         .color(Color::new(0.2, 0.5, 0.8, 1.0))
///         .style(tessera_components::theme::TextStyle {
///             font_size: Dp(32.0),
///             line_height: Some(Dp(40.0)),
///             ..Default::default()
///         });
/// }
///
/// # tessera_ui::testing::UiTest::new().content(demo).frame();
/// ```
#[tessera]
pub fn text(
    #[prop(into)] content: TextContent,
    modifier: Option<Modifier>,
    color: Option<Color>,
    style: Option<TextStyle>,
    size: Option<Dp>,
    line_height: Option<Dp>,
    single_line: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let modifier = modifier.unwrap_or_default();
    let theme = if style.is_none() || color.is_none() {
        use_context::<MaterialTheme>()
    } else {
        None
    };
    let resolved_style = if let Some(style) = style {
        style
    } else if let Some(inherited) = use_context::<TextStyle>() {
        inherited.get()
    } else {
        theme
            .map(|theme| theme.with(|theme| theme.typography.body_large))
            .unwrap_or_default()
    };
    let color = if let Some(color) = color {
        color
    } else if let Some(inherited) = use_context::<ContentColor>() {
        inherited.get().current
    } else {
        theme
            .map(|theme| theme.with(|theme| theme.color_scheme.on_surface))
            .unwrap_or_else(|| ContentColor::default().current)
    };
    let size = size.unwrap_or(resolved_style.font_size);
    let TextContent(content) = content;
    let semantics = TextSemantics {
        content: content.clone(),
        label: accessibility_label,
        description: accessibility_description,
    };
    let line_height = line_height
        .or(resolved_style.line_height)
        .unwrap_or(Dp(size.0 * 1.2));

    // A draw-only rebuild can reuse measurement and its glyph buffer.
    let shaped = remember(|| Arc::new(Mutex::new(None))).get();
    tessera_ui::layout::layout(TextLayout {
        text: content,
        single_line: single_line.unwrap_or(false),
        style: TextStyle {
            font_size: size,
            line_height: Some(line_height),
            ..resolved_style
        },
        shaped: shaped.clone(),
    })
    .draw(TextDraw { color, shaped })
    .modifier(modifier.push_semantics(semantics));
}

#[derive(Clone)]
struct TextSemantics {
    content: Arc<str>,
    label: Option<String>,
    description: Option<String>,
}

impl SemanticsModifierNode for TextSemantics {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        _: &mut Option<AccessibilityActionHandler>,
    ) {
        if accessibility.role.is_none() {
            accessibility.role = Some(Role::Label);
        }
        if let Some(label) = &self.label {
            accessibility.label = Some(label.clone());
        } else if accessibility.label.is_none() && !self.content.is_empty() {
            accessibility.label = Some(self.content.to_string());
        }
        if let Some(description) = &self.description {
            accessibility.description = Some(description.clone());
        }
    }
}

#[derive(Clone)]
struct TextLayout {
    text: Arc<str>,
    single_line: bool,
    style: TextStyle,
    shaped: Arc<Mutex<Option<CachedText>>>,
}

struct CachedText {
    fonts: Option<u64>,
    constraint: TextConstraint,
    data: TextData,
}

impl PartialEq for TextLayout {
    fn eq(&self, other: &Self) -> bool {
        tessera_ui::LayoutPolicy::measure_eq(self, other)
    }
}

impl TextLayout {
    fn measure_text(&self, constraint: TextConstraint) -> TextMeasureInfo {
        let fonts = crate::fonts::prepare(&self.text, self.style);
        let format = TextFormat::from(self.style);
        let mut cache = self.shaped.lock();
        match cache.as_mut() {
            Some(cached)
                if cached.fonts == fonts
                    && cached.data.matches_text(&self.text, format)
                    && cached.constraint.single_line == constraint.single_line =>
            {
                if cached.constraint != constraint {
                    cached.data.relayout(&constraint);
                    cached.constraint = constraint;
                }
            }
            _ => {
                *cache = Some(CachedText {
                    fonts,
                    constraint: constraint.clone(),
                    data: TextData::shape_prepared(
                        self.text.clone(),
                        Color::WHITE,
                        format,
                        constraint,
                    ),
                });
            }
        }
        cache
            .as_ref()
            .expect("text must be shaped")
            .data
            .measure_info()
    }
}

impl tessera_ui::LayoutPolicy for TextLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.style == other.style
            && self.single_line == other.single_line
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let max_width = input.parent_constraint().width().resolve_max();
        let max_height = input.parent_constraint().height().resolve_max();

        let info = self.measure_text(TextConstraint {
            max_width: max_width.map(|px: Px| px.to_f32()),
            max_height: max_height.map(|px: Px| px.to_f32()),
            single_line: self.single_line,
        });

        Ok(LayoutResult::new(ComputedData {
            width: input.parent_constraint().width().clamp(info.size[0].into()),
            height: input
                .parent_constraint()
                .height()
                .clamp(info.size[1].into()),
        }))
    }
}

#[derive(Clone)]
struct TextDraw {
    color: Color,
    shaped: Arc<Mutex<Option<CachedText>>>,
}

impl PartialEq for TextDraw {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color && Arc::ptr_eq(&self.shaped, &other.shaped)
    }
}

impl tessera_ui::DrawModifierNode for TextDraw {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        let Some(mut text_data) = self
            .shaped
            .lock()
            .as_ref()
            .filter(|cached| cached.fonts.is_some())
            .map(|cached| cached.data.clone())
        else {
            return;
        };
        text_data.set_color(self.color);

        let drawable = TextCommand {
            data: text_data,
            offset: PxPosition::ZERO,
        };
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(drawable);
        content.draw(input);
    }
}

#[cfg(test)]
mod tests {
    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::{
        State, provide_context,
        testing::WorkPhase,
        testing::{TestProbe, UiTest},
    };

    use crate::{boxed::boxed, column::column, row::row, theme::material_theme};

    use super::*;

    #[tessera]
    fn weighted_labels_fixture() {
        boxed().children(|| {
            row()
                .modifier(Modifier::new().width(Dp(300.0)))
                .children(|| {
                    for index in 0..3 {
                        text("Cell").modifier(Modifier::new().weight(1.0).semantics(
                            SemanticsArgs {
                                test_tag: Some(format!("weighted-label-{index}")),
                                ..Default::default()
                            },
                        ));
                    }
                });
        });
    }

    #[test]
    fn weighted_text_respects_allocated_column_width() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "text::tests::weighted_text_respects_allocated_column_width",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let snapshot = UiTest::new()
                    .content(weighted_labels_fixture)
                    .viewport(400, 100)
                    .frame();
                for index in 0..3 {
                    let bounds = snapshot.node(&format!("weighted-label-{index}")).bounds();
                    assert_eq!(bounds.width, Px(100));
                    assert_eq!(bounds.x, Px(index * 100));
                }
            },
        );
    }

    #[tessera]
    fn reflow_fixture(observed: TestProbe<Option<State<Dp>>>) {
        let width = remember(|| Dp(120.0));
        observed.set(Some(width));
        material_theme().child(move || {
            column().children(move || {
                boxed()
                    .modifier(Modifier::new().width(width.get()).semantics(SemanticsArgs {
                        test_tag: Some("wrapped-text".into()),
                        ..Default::default()
                    }))
                    .children(|| {
                        text("Material labels stay readable when their containers resize.");
                    });
            });
        });
    }

    #[test]
    fn density_changes_remeasure_retained_content_without_state_writes() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "text::tests::density_changes_remeasure_retained_content_without_state_writes",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let observed = TestProbe::default();
                let mut session = UiTest::new()
                    .content(|| reflow_fixture(observed.clone()))
                    .viewport(500, 400);
                let original = session.frame().node("wrapped-text").bounds();
                let width = observed.get().expect("fixture state");
                tessera_ui::dp::set_scale_factor(1.5);
                let scaled = session.frames(1).node("wrapped-text").bounds();
                assert_eq!(scaled.width, Px::new(180));
                assert!(scaled.height > original.height);
                assert!(observed.get() == Some(width));
                assert_eq!(width.get(), Dp(120.0));
                tessera_ui::dp::set_scale_factor(1.0);
                let restored = session
                    .advance(std::time::Duration::from_nanos(16_666_666))
                    .node("wrapped-text")
                    .bounds();
                assert_eq!(restored, original);
            },
        );
    }

    #[tessera]
    fn color_fixture(observed: TestProbe<Option<State<tessera_ui::Color>>>) {
        let color = remember(|| tessera_ui::Color::BLACK);
        observed.set(Some(color));
        boxed()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("color-text".into()),
                ..Default::default()
            }))
            .children(move || {
                text("Color cache").color(color.get());
            });
    }

    #[test]
    fn text_color_updates_retain_measurements_and_window_geometry() {
        let observed = TestProbe::default();
        let mut session = UiTest::new()
            .content(|| color_fixture(observed.clone()))
            .viewport(240, 100);
        let original = session.frame().node("color-text").bounds();
        observed.get().unwrap().set(tessera_ui::Color::WHITE);
        let changed = session.frames(1).node("color-text").bounds();
        assert_eq!(changed, original);
        let diagnostics = session.diagnostics();
        assert_eq!(
            session
                .work()
                .phase(tessera_ui::testing::WorkPhase::Measure)
                .count(),
            0
        );
        assert_eq!(diagnostics.geometry_nodes_updated, 0);
    }

    #[tessera]
    fn inherited_defaults_fixture(
        observed: TestProbe<Option<(State<TextStyle>, State<ContentColor>)>>,
    ) {
        let style = remember(TextStyle::default);
        let color = remember(ContentColor::default);
        observed.set(Some((style, color)));
        provide_context(style.get(), move || {
            provide_context(color.get(), move || {
                text("Inherited");
                text("Explicit")
                    .style(TextStyle::default())
                    .color(Color::BLACK);
            });
        });
    }

    #[test]
    fn explicit_text_defaults_ignore_inherited_updates() {
        let observed = TestProbe::default();
        let mut session = UiTest::new().content(|| inherited_defaults_fixture(observed.clone()));
        session.frame();
        assert_eq!(session.work().count("text", WorkPhase::Build), 2);
        let (style, color) = observed.get().unwrap();

        style.with_mut(|style| style.font_size = Dp(30.0));
        session.frames(1);
        assert_eq!(session.work().count("text", WorkPhase::Build), 1);

        color.set(ContentColor {
            current: Color::WHITE,
        });
        session.frames(1);
        assert_eq!(session.work().count("text", WorkPhase::Build), 1);
    }

    #[test]
    fn retained_text_reflows_when_its_parent_changes_width() {
        let observed = TestProbe::default();
        let mut session = UiTest::new()
            .content(|| reflow_fixture(observed.clone()))
            .viewport(500, 400);
        let narrow = session.frame().node("wrapped-text").bounds();
        observed.get().expect("fixture state").set(Dp(280.0));
        let wide = session.frames(1).node("wrapped-text").bounds();
        assert_eq!(narrow.width, Px::new(120));
        assert_eq!(wide.width, Px::new(280));
        assert!(wide.height < narrow.height);
        observed.get().expect("fixture state").set(Dp(120.0));
        let restored = session
            .advance(std::time::Duration::from_nanos(16_666_666))
            .node("wrapped-text")
            .bounds();
        assert_eq!(restored.height, narrow.height);
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "text/drawing_tests.rs"]
mod drawing_tests;

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "text/semantics_tests.rs"]
mod semantics_tests;
