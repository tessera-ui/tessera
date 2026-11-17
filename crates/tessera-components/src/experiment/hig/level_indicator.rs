//! Read-only continuous capacity indicators.
//!
//! ## Usage
//!
//! Show storage, battery, or resource levels with warning and critical thresholds.

use std::ops::RangeInclusive;

use tessera_foundation::modifier::ModifierExt as _;
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
#[path = "level_indicator_tests.rs"]
mod tests;

#[derive(Clone, PartialEq)]
struct CapacitySemantics {
    value: f64,
    min: f64,
    max: f64,
    enabled: bool,
    label: Option<String>,
    description: Option<String>,
}

impl SemanticsModifierNode for CapacitySemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.role = Some(Role::Meter);
        node.numeric_value = Some(self.value);
        node.min_numeric_value = Some(self.min);
        node.max_numeric_value = Some(self.max);
        node.disabled = !self.enabled;
        if let Some(label) = &self.label {
            node.label = Some(label.clone());
        }
        if let Some(description) = &self.description {
            node.description = Some(description.clone());
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CapacityLayout;

impl LayoutPolicy for CapacityLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(180.0).into());
        let height = parent.height.clamp(Dp(16.0).into());
        let bar = Px::from(Dp(16.0));
        let cell = Px::from(Dp(18.0));
        let mut result = LayoutResult::new(ComputedData { width, height });
        for child in scope.children() {
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(width), AxisConstraint::exact(bar)),
            )?;
            result.place_child(
                child,
                PxPosition::from_f32_xy(
                    0.0,
                    height.to_f32() - (height - cell).to_f32().max(0.0) * 0.5 - cell.to_f32(),
                ),
            );
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct CapacityFill {
    color: Color,
    fraction: Option<f64>,
    outline: bool,
}

impl DrawModifierNode for CapacityFill {
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
            Dp(Dp::from_pixels_f32(width * fraction as f32).0.round())
                .to_pixels_f32()
                .min(width)
        });
        if fill_width > 0.0 {
            context
                .render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(PerceptualFillCommand {
                    color: self.color,
                    radii: [Dp(2.25); 2],
                    insets: [
                        Dp::ZERO,
                        Dp::from_pixels_f32(width - fill_width),
                        Dp::ZERO,
                        Dp::ZERO,
                    ],
                    clip_radius: None,
                    edge_smoothness: 1.0,
                    stroke: None,
                    line: None,
                    outline_width: if self.outline { Dp(0.5) } else { Dp::ZERO },
                    snap_to_pixels: true,
                    path: None,
                });
        }
        content.draw(context.render_input);
    }
}

#[tessera]
fn capacity_fill(color: Color, fraction: Option<f64>, outline: bool) {
    layout(Stack).modifier(Modifier::new().push_draw(CapacityFill {
        color,
        fraction,
        outline,
    }));
}

/// # capacity_indicator
///
/// Displays a read-only resource level with warning and critical colors.
///
/// ## Usage
///
/// Show remaining or consumed capacity without offering a value-editing control.
///
/// ## Parameters
///
/// - `value`, `range` — finite level and increasing limits; defaults to `0.0..=5.0`.
/// - `warning_value`, `critical_value` — optional inclusive color thresholds in range units; omitted or zero thresholds are disabled.
/// - `enabled` — semantic availability; defaults to true without changing the capacity colors.
/// - `modifier` — sizing, placement, and additional semantics.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::level_indicator::capacity_indicator;
/// use tessera_ui::{remember, tessera, testing::{TestProbe, UiTest}};
/// #[tessera]
/// fn app(probe: TestProbe<Option<tessera_ui::State<f64>>>) {
///     let used = remember(|| 0.25);
///     probe.set(Some(used));
///     capacity_indicator(used.get()).accessibility_label("Storage");
/// }
/// let probe = TestProbe::default();
/// let mut ui = UiTest::new().content(|| app(probe.clone()));
/// ui.frame();
/// probe.get().unwrap().set(0.8);
/// ui.frame();
/// assert_eq!(ui.accessibility_node("Storage").1.numeric_value(), Some(0.8));
/// ```
#[tessera]
pub fn capacity_indicator(
    value: f64,
    range: Option<RangeInclusive<f64>>,
    warning_value: Option<f64>,
    critical_value: Option<f64>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let range = range.unwrap_or(0.0..=5.0);
    let (min, max) = (*range.start(), *range.end());
    assert!(
        min.is_finite() && max.is_finite() && min < max && (max - min).is_finite(),
        "capacity range must be finite and increasing"
    );
    assert!(value.is_finite(), "capacity value must be finite");
    let warning = warning_value.filter(|threshold| *threshold != 0.0);
    let critical = critical_value.filter(|threshold| *threshold != 0.0);
    assert!(
        [warning, critical]
            .into_iter()
            .flatten()
            .all(|threshold| threshold.is_finite() && min <= threshold && threshold <= max)
            && warning
                .zip(critical)
                .is_none_or(|(warning, critical)| warning <= critical),
        "capacity thresholds must be finite, ordered, and inside the range"
    );
    let value = value.clamp(min, max);
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let encoded: [u8; 3] = if critical.is_some_and(|threshold| value >= threshold) {
        if dark { [255, 66, 69] } else { [255, 56, 60] }
    } else if warning.is_some_and(|threshold| value >= threshold) {
        if dark { [255, 214, 0] } else { [255, 204, 0] }
    } else if dark {
        [48, 209, 88]
    } else {
        [52, 199, 89]
    };
    let [r, g, b] = encoded.map(|channel| {
        let channel = f32::from(channel) / 255.0;
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    });
    let fill = Color::from_rgb(r, g, b);
    let track = if dark {
        Color::WHITE.with_alpha(0.1)
    } else {
        Color::BLACK.with_alpha(0.07)
    };
    layout(CapacityLayout)
        .modifier(
            modifier
                .unwrap_or_default()
                .clip_to_bounds()
                .push_semantics(CapacitySemantics {
                    value,
                    min,
                    max,
                    enabled: enabled.unwrap_or(true),
                    label: accessibility_label,
                    description: accessibility_description,
                }),
        )
        .child(move || {
            capacity_fill(track, false);
            capacity_fill(fill, false).fraction((value - min) / (max - min));
            capacity_fill(Color::BLACK.with_alpha(0.13), true);
        });
}
