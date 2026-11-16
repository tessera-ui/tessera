//! Determinate horizontal progress bars.
//!
//! ## Usage
//!
//! Show the completed portion of a bounded operation.

use std::ops::RangeInclusive;

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Color, ComputedData, Constraint,
    Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode,
    accesskit::Role,
    layout::{MeasureScope, Stack, layout},
    modifier::ModifierCapabilityExt as _,
    tessera, use_context,
};

use crate::pipelines::perceptual_fill::PerceptualFillCommand;

use super::theme::{Appearance, HigTheme};

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;

#[derive(Clone, PartialEq)]
struct ProgressSemantics {
    value: f64,
    min: f64,
    max: f64,
    label: Option<String>,
    description: Option<String>,
}

impl SemanticsModifierNode for ProgressSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.role = Some(Role::ProgressIndicator);
        node.numeric_value = Some(self.value);
        node.min_numeric_value = Some(self.min);
        node.max_numeric_value = Some(self.max);
        if self.label.is_some() {
            node.label = self.label.clone();
        }
        if self.description.is_some() {
            node.description = self.description.clone();
        }
    }
}

#[derive(Clone, PartialEq)]
struct ProgressFill {
    color: Color,
    fraction: Option<f64>,
}

impl DrawModifierNode for ProgressFill {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let width = context
            .render_input
            .metadata_mut()
            .computed_data()
            .unwrap_or(ComputedData::ZERO)
            .width
            .to_f32();
        let fill_width = self.fraction.map_or(width, |fraction| {
            (width * fraction as f32 - Dp(0.8).to_pixels_f32())
                .max(Dp(4.0).to_pixels_f32())
                .min(width)
        });
        let radius_x = self.fraction.map_or(Dp(4.0), |_| {
            Dp((Dp::from(Px(1)).0 * fill_width as f64 * 0.625).min(4.0))
        });
        let end_inset = Dp(Dp::from(Px(1)).0 * (width - fill_width) as f64);
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                color: self.color,
                radii: [radius_x, Dp(4.0)],
                insets: [Dp(0.0), end_inset, Dp(0.0), Dp(0.0)],
                clip_radius: self.fraction.map(|_| Dp(4.0)),
                edge_smoothness: if self.fraction.is_some() { 1.3 } else { 1.0 },
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ProgressLayout;

impl LayoutPolicy for ProgressLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self == other
    }

    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(160.0).into());
        let height = parent.height.clamp(Dp(20.0).into());
        let bar = Px::from(Dp(8.0)).min(height);
        let inset = Px::from(Dp(0.5)).min(Px(bar.0 / 2)).min(Px(width.0 / 2));
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (index, child) in scope.children().iter().enumerate() {
            let (w, h, x, y) = match index {
                1 => (
                    width - inset * 2,
                    bar - inset * 2,
                    inset.to_f32(),
                    inset.to_f32(),
                ),
                _ => (width, bar, 0.0, 0.0),
            };
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(w), AxisConstraint::exact(h)),
            )?;
            result.place_child(
                *child,
                PxPosition::from_f32_xy(x, (height - bar).to_f32() * 0.5 + y),
            );
        }
        Ok(result)
    }
}

#[tessera]
fn progress_fill(color: Color) {
    layout(Stack).modifier(Modifier::new().push_draw(ProgressFill {
        color,
        fraction: None,
    }));
}

#[tessera]
fn progress_indicator(fraction: f64, color: Color) {
    layout(Stack).modifier(Modifier::new().push_draw(ProgressFill {
        color,
        fraction: Some(fraction),
    }));
}

/// # progress_bar
///
/// Shows determinate completion for downloads and other bounded operations.
///
/// ## Usage
///
/// Publish the current amount of completed work without offering input actions.
///
/// ## Parameters
///
/// - `value`, `range` — finite amount and increasing limits; the range defaults to `0.0..=1.0`.
/// - `window_active` — active accent appearance; defaults to true.
/// - `modifier` — layout and additional semantics.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::progress::progress_bar;
/// use tessera_ui::{remember, tessera, testing::{TestProbe, UiTest}};
/// #[tessera]
/// fn app(probe: TestProbe<Option<tessera_ui::State<f64>>>) {
///     let completed = remember(|| 0.25);
///     probe.set(Some(completed));
///     progress_bar(completed.get()).accessibility_label("Download");
/// }
/// let probe = TestProbe::default();
/// let mut ui = UiTest::new().content(|| app(probe.clone()));
/// ui.frame();
/// probe.get().unwrap().set(0.75);
/// ui.frame();
/// assert_eq!(ui.accessibility_node("Download").1.numeric_value(), Some(0.75));
/// ```
#[tessera]
pub fn progress_bar(
    value: f64,
    range: Option<RangeInclusive<f64>>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let range = range.unwrap_or(0.0..=1.0);
    let (min, max) = (*range.start(), *range.end());
    assert!(
        min.is_finite() && max.is_finite() && min < max && (max - min).is_finite(),
        "progress range must be finite and increasing"
    );
    assert!(value.is_finite(), "progress value must be finite");
    let value = value.clamp(min, max);
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let track = if dark {
        Color::from_rgb(0.02415763, 0.02415763, 0.02415763)
    } else {
        Color::from_rgb(0.76815116, 0.76815116, 0.76815116)
    };
    let fill = if window_active.unwrap_or(true) {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    } else {
        Color::from_rgb(0.40197778, 0.40197778, 0.40197778)
    };
    layout(ProgressLayout)
        .modifier(
            modifier
                .unwrap_or_default()
                .push_semantics(ProgressSemantics {
                    value,
                    min,
                    max,
                    label: accessibility_label,
                    description: accessibility_description,
                }),
        )
        .child(move || {
            progress_fill(tint.with_alpha(if dark { 0.11 } else { 0.13 }));
            progress_fill(track);
            progress_indicator((value - min) / (max - min), fill);
        });
}
