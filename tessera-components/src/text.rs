//! Single-style text rendering.
//!
//! ## Usage
//!
//! Display labels, headings, and other text content.
use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Color, ComputedData, Dp, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera, use_context,
};

use crate::{
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
/// - `modifier` — modifier chain applied to the text node; caller semantics
///   remain intact. The default label is the content and the default role is a
///   label, unless supplied by the modifier.
/// - `content` — text content to display.
/// - `color` — optional text color override.
/// - `style` — optional text style override for typography presets.
/// - `size` — optional font size override.
/// - `line_height` — optional line height override.
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
///     text()
///         .content("Hello, world!")
///         .color(Color::new(0.2, 0.5, 0.8, 1.0))
///         .style(tessera_components::theme::TextStyle {
///             font_size: Dp(32.0),
///             line_height: Some(Dp(40.0)),
///             ..Default::default()
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn text(
    modifier: Option<Modifier>,
    #[prop(into)] content: Option<String>,
    color: Option<Color>,
    style: Option<TextStyle>,
    size: Option<Dp>,
    line_height: Option<Dp>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let modifier = modifier.unwrap_or_default();
    let content = content.unwrap_or_default();
    let theme = use_context::<MaterialTheme>();
    let inherited_style = use_context::<TextStyle>().map(|s| s.get());
    let resolved_style = style
        .or(inherited_style)
        .or_else(|| theme.map(|t| t.get().typography.body_large))
        .unwrap_or_default();
    let color = color
        .or_else(|| use_context::<ContentColor>().map(|c| c.get().current))
        .or_else(|| theme.map(|t| t.get().color_scheme.on_surface))
        .unwrap_or_else(|| ContentColor::default().current);
    let size = size.unwrap_or(resolved_style.font_size);
    let semantics = TextSemantics {
        content: content.clone(),
        label: accessibility_label,
        description: accessibility_description,
    };
    let line_height = line_height
        .or(resolved_style.line_height)
        .unwrap_or(Dp(size.0 * 1.2));

    let policy = TextLayout {
        text: content.clone(),
        color,
        style: TextStyle {
            font_size: size,
            line_height: Some(line_height),
            ..resolved_style
        },
        // A draw-only rebuild can reuse measurement, so its glyph buffer must
        // survive independently of the newly constructed layout policy.
        shaped: remember(|| Arc::new(Mutex::new(None))).get(),
    };
    layout()
        .modifier(modifier.push_semantics(semantics))
        .policy(policy.clone())
        .draw(policy);
}

#[derive(Clone)]
struct TextSemantics {
    content: String,
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
            accessibility.label = Some(self.content.clone());
        }
        if let Some(description) = &self.description {
            accessibility.description = Some(description.clone());
        }
    }
}

#[derive(Clone)]
struct TextLayout {
    text: String,
    color: Color,
    style: TextStyle,
    shaped: Arc<Mutex<Option<CachedText>>>,
}

struct CachedText {
    text: String,
    style: TextStyle,
    font_size_px: f32,
    constraint: TextConstraint,
    data: TextData,
}

impl PartialEq for TextLayout {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.color == other.color && self.style == other.style
    }
}

impl TextLayout {
    fn shaped_text(&self, constraint: TextConstraint) -> TextData {
        let mut cache = self.shaped.lock();
        let font_size_px = self.style.font_size.to_pixels_f32();
        if cache.as_ref().is_none_or(|cached| {
            cached.text != self.text
                || cached.style != self.style
                || cached.font_size_px != font_size_px
                || cached.constraint != constraint
        }) {
            *cache = Some(CachedText {
                text: self.text.clone(),
                style: self.style,
                font_size_px,
                constraint: constraint.clone(),
                data: TextData::shape(self.text.clone(), self.color, self.style, constraint),
            });
        }
        let mut data = cache.as_ref().expect("text must be shaped").data.clone();
        data.set_color(self.color);
        data
    }
}

impl tessera_ui::LayoutModifierNode for TextLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.text == other.text && self.style == other.style
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let max_width = input.parent_constraint().width().resolve_max();
        let max_height = input.parent_constraint().height().resolve_max();

        let info = self
            .shaped_text(TextConstraint {
                max_width: max_width.map(|px: Px| px.to_f32()),
                max_height: max_height.map(|px: Px| px.to_f32()),
            })
            .measure_info();

        Ok(LayoutResult::new(ComputedData {
            width: info.size[0].into(),
            height: info.size[1].into(),
        }))
    }
}

impl tessera_ui::DrawModifierNode for TextLayout {
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
    use std::cell::Cell;

    use tessera_ui::{
        State,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use crate::{
        boxed::boxed,
        column::column,
        modifier::{ModifierExt as _, SemanticsArgs},
        theme::material_theme,
    };

    use super::*;

    thread_local! { static WIDTH: Cell<Option<State<Dp>>> = const { Cell::new(None) }; }

    #[tessera]
    fn reflow_fixture() {
        let width = remember(|| Dp(120.0));
        WIDTH.set(Some(width));
        material_theme().child(move || {
            column().children(move || {
                boxed()
                    .modifier(Modifier::new().width(width.get()).semantics(SemanticsArgs {
                        test_tag: Some("wrapped-text".into()),
                        ..Default::default()
                    }))
                    .children(|| {
                        text()
                            .content("Material labels stay readable when their containers resize.");
                    });
            });
        });
    }

    #[test]
    fn density_changes_remeasure_retained_content_without_state_writes() {
        crate::test_utils::run_isolated_test(
            "text::tests::density_changes_remeasure_retained_content_without_state_writes",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let mut session = start_layout_test_session(
                    layout_test(|| {
                        reflow_fixture();
                    })
                    .viewport_px(500, 400),
                );
                let original = pump_layout_test_session(&mut session, 0)
                    .node("wrapped-text")
                    .bounds();
                let width = WIDTH.get().expect("fixture state");
                tessera_ui::dp::set_scale_factor(1.5);
                let scaled = pump_layout_test_session(&mut session, 16_666_667)
                    .node("wrapped-text")
                    .bounds();
                assert_eq!(scaled.width, Px::new(180));
                assert!(scaled.height > original.height);
                assert!(WIDTH.get() == Some(width));
                assert_eq!(width.get(), Dp(120.0));
                tessera_ui::dp::set_scale_factor(1.0);
                let restored = pump_layout_test_session(&mut session, 33_333_333)
                    .node("wrapped-text")
                    .bounds();
                assert_eq!(restored, original);
            },
        );
    }

    thread_local! { static COLOR: Cell<Option<State<tessera_ui::Color>>> = const { Cell::new(None) }; }

    #[tessera]
    fn color_fixture() {
        let color = remember(|| tessera_ui::Color::BLACK);
        COLOR.set(Some(color));
        boxed()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("color-text".into()),
                ..Default::default()
            }))
            .children(move || {
                text().content("Color cache").color(color.get());
            });
    }

    #[test]
    fn text_color_updates_retain_measurements_and_window_geometry() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                color_fixture();
            })
            .viewport_px(240, 100),
        );
        let original = pump_layout_test_session(&mut session, 0)
            .node("color-text")
            .bounds();
        COLOR.get().unwrap().set(tessera_ui::Color::WHITE);
        let changed = pump_layout_test_session(&mut session, 16_666_667)
            .node("color-text")
            .bounds();
        assert_eq!(changed, original);
        let diagnostics = session.diagnostics().unwrap();
        assert_eq!(diagnostics.measure_node_calls, 1);
        assert_eq!(diagnostics.cache_store_count, 0);
        assert_eq!(diagnostics.geometry_nodes_updated, 0);
    }

    #[test]
    fn retained_text_reflows_when_its_parent_changes_width() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                reflow_fixture();
            })
            .viewport_px(500, 400),
        );
        let narrow = pump_layout_test_session(&mut session, 0)
            .node("wrapped-text")
            .bounds();
        WIDTH.get().expect("fixture state").set(Dp(280.0));
        let wide = pump_layout_test_session(&mut session, 16_666_667)
            .node("wrapped-text")
            .bounds();
        assert_eq!(narrow.width, Px::new(120));
        assert_eq!(wide.width, Px::new(280));
        assert!(wide.height < narrow.height);
        WIDTH.get().expect("fixture state").set(Dp(120.0));
        let restored = pump_layout_test_session(&mut session, 33_333_333)
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
