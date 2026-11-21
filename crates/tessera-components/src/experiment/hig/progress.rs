//! Progress views for bounded operations and ongoing activity.
//!
//! ## Usage
//!
//! Show the completion of a bounded operation or indicate that work is ongoing.

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

use crate::pipelines::{
    perceptual_fill::PerceptualFillCommand,
    progress_arc::command::{ProgressArcCap, ProgressArcCommand},
};

use super::theme::{Appearance, ControlSize, HigTheme};

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
        let single_density = Dp(1.0).to_pixels_f32() == 1.0;
        let mut fill_width = width;
        let mut radii = [Dp(4.0); 2];
        let mut edge_smoothness = 1.0;
        if let Some(fraction) = self.fraction {
            let requested = width * fraction as f32;
            let minimum = Dp(4.0).to_pixels_f32();
            // The minimum cap stays fixed until the nominal fill exceeds the
            // track height. This footprint is calibrated at 1x.
            if single_density && requested <= Dp(8.0).to_pixels_f32() {
                fill_width = minimum.min(width);
                radii = [Dp(3.6), Dp(3.28)];
                edge_smoothness = 1.19;
            } else {
                fill_width = (requested - Dp(0.8).to_pixels_f32())
                    .max(minimum)
                    .min(width);
                radii[0] = Dp((Dp::from(Px(1)).0 * fill_width as f64 * 0.625).min(4.0));
                // At 1x the half-point track inset rounds away; a one-pixel
                // ramp gives the straight top and bottom rows full coverage.
                edge_smoothness = if single_density {
                    // Keep the full vertical radius when the two caps nearly meet.
                    radii[0] = Dp((fill_width as f64 * 0.5).min(4.0));
                    1.0 + 0.25 * (9.5 - fill_width).clamp(0.0, 1.0)
                } else {
                    1.3
                };
            }
        }
        let end_inset = Dp(Dp::from(Px(1)).0 * (width - fill_width) as f64);
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: self.color,
                radii,
                insets: [Dp(0.0), end_inset, Dp(0.0), Dp(0.0)],
                clip_radius: self.fraction.map(|_| Dp(4.0)),
                line: None,
                edge_smoothness,
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

/// Lays out a linear bar of the given thickness, centred in a 20pt tall row.
#[derive(Clone, Copy, PartialEq)]
struct ProgressLayout(Dp);

impl LayoutPolicy for ProgressLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self == other
    }

    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(160.0).into());
        let height = parent.height.clamp(Dp(20.0).into());
        let bar = Px::from(self.0).min(height);
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

/// The appearance of a [`progress_view`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressViewStyle {
    /// A bar for determinate progress and a spinner for indeterminate progress.
    #[default]
    Automatic,
    /// A horizontal bar.
    Linear,
    /// A ring for determinate progress or a spinner for indeterminate progress.
    Circular,
}

/// # progress_view
///
/// Shows how much of an operation is complete, or that work is ongoing.
///
/// ## Usage
///
/// Report downloads, imports, and other long operations without offering input.
///
/// ## Parameters
///
/// - `value`, `total` — completed amount out of `total` (default 1); without a value the view is
///   indeterminate.
/// - `style` — bar, ring, or spinner; defaults to [`ProgressViewStyle::Automatic`].
/// - `control_size` — size of the bar, ring, or spinner; defaults to regular.
/// - `window_active` — active accent appearance; defaults to true.
/// - `modifier` — layout and additional semantics.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::progress::{ProgressViewStyle, progress_view};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// progress_view().value(0.25).accessibility_label("Download");
/// progress_view().value(3.0).total(8.0).style(ProgressViewStyle::Circular);
/// progress_view().accessibility_label("Connecting");
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn progress_view(
    value: Option<f64>,
    total: Option<f64>,
    style: Option<ProgressViewStyle>,
    control_size: Option<ControlSize>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let style = style.unwrap_or_default();
    let size = control_size.unwrap_or_default();
    let modifier = modifier.unwrap_or_default();
    let Some(value) = value else {
        if style == ProgressViewStyle::Linear {
            indeterminate_progress_bar(true)
                .size(IndeterminateBarSize::from(size))
                .window_active_optional(window_active)
                .modifier(modifier)
                .accessibility_label_optional(accessibility_label)
                .accessibility_description_optional(accessibility_description);
        } else {
            progress_spinner(true)
                .size(SpinnerSize::from(size))
                .modifier(modifier)
                .accessibility_label_optional(accessibility_label)
                .accessibility_description_optional(accessibility_description);
        }
        return;
    };
    let total = total.unwrap_or(1.0);
    assert!(
        total.is_finite() && total > 0.0,
        "progress total must be finite and positive"
    );
    assert!(value.is_finite(), "progress value must be finite");
    let value = value.clamp(0.0, total);
    let semantics = ProgressSemantics {
        value,
        min: 0.0,
        max: total,
        label: accessibility_label,
        description: accessibility_description,
    };
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let active = window_active.unwrap_or(true);
    let modifier = modifier.push_semantics(semantics);
    if style == ProgressViewStyle::Circular {
        progress_ring(value / total, size, dark, active, modifier);
    } else {
        progress_bar(value / total, size, dark, active, modifier);
    }
}

#[tessera]
fn progress_bar(fraction: f64, size: ControlSize, dark: bool, active: bool, modifier: Modifier) {
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let track = if dark {
        Color::from_rgb(0.02415763, 0.02415763, 0.02415763)
    } else {
        Color::from_rgb(0.76815116, 0.76815116, 0.76815116)
    };
    let fill = if active {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    } else {
        Color::from_rgb(0.40197778, 0.40197778, 0.40197778)
    };
    let thickness = match size {
        ControlSize::Mini | ControlSize::Small => Dp(6.0),
        _ => Dp(8.0),
    };
    layout(ProgressLayout(thickness))
        .modifier(modifier)
        .child(move || {
            progress_fill(tint.with_alpha(if dark { 0.11 } else { 0.13 }));
            progress_fill(track);
            progress_indicator(fraction, fill);
        });
}

#[derive(Clone, PartialEq)]
struct RingPaint {
    fraction: f64,
    track: Color,
    fill: Color,
}

impl DrawModifierNode for RingPaint {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let stroke = Dp(5.0).to_pixels_f32();
        let mut metadata = context.render_input.metadata_mut();
        let fragment = metadata.fragment_mut();
        fragment.push_draw_command(ProgressArcCommand {
            color: self.track,
            stroke_width_px: stroke,
            start_angle_degrees: 0.0,
            sweep_angle_degrees: 360.0,
            cap: ProgressArcCap::Butt,
        });
        if self.fraction > 0.0 {
            fragment.push_draw_command(ProgressArcCommand {
                color: self.fill,
                stroke_width_px: stroke,
                start_angle_degrees: 270.0,
                sweep_angle_degrees: (self.fraction * 360.0) as f32,
                cap: ProgressArcCap::Round,
            });
        }
        content.draw(context.render_input);
    }
}

/// A determinate ring: 16pt across for mini and small sizes, 32pt otherwise, with a 5pt band.
#[tessera]
fn progress_ring(fraction: f64, size: ControlSize, dark: bool, active: bool, modifier: Modifier) {
    let diameter = match size {
        ControlSize::Mini | ControlSize::Small => Dp(16.0),
        _ => Dp(32.0),
    };
    let (track, fill) = if dark {
        (srgb(48, 48, 48), srgb(23, 105, 230))
    } else {
        (srgb(229, 229, 229), srgb(10, 129, 253))
    };
    let fill = if active {
        fill
    } else {
        Color::from_rgb(0.40197778, 0.40197778, 0.40197778)
    };
    layout(Stack).modifier(modifier.size(diameter, diameter).push_draw(RingPaint {
        fraction,
        track,
        fill,
    }));
}

fn srgb(r: u8, g: u8, b: u8) -> Color {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    Color::from_rgb(channel(r), channel(g), channel(b))
}

use indeterminate_bar::{IndeterminateBarSize, indeterminate_progress_bar};
use spinner::{SpinnerSize, progress_spinner};

#[path = "progress_indeterminate_bar.rs"]
mod indeterminate_bar;
#[path = "progress_spinner.rs"]
mod spinner;
