//! Gauges that show a value within a range.
//!
//! ## Usage
//!
//! Show storage, battery, or other levels as bars, dials, or rings.

use std::ops::RangeInclusive;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Color, ComputedData, Constraint,
    Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, PxPosition, SemanticsModifierNode,
    accesskit::Role,
    layout::{MeasureScope, Stack, layout},
    modifier::ModifierCapabilityExt as _,
    tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    pipelines::perceptual_fill::{DialShape, PerceptualDialCommand, PerceptualFillCommand},
    spacer::spacer,
};

use super::{
    progress::progress_view,
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "gauge_tests.rs"]
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

/// Lays the capacity bar out at 16pt tall across the available width, or 180pt when the width
/// is unbounded.
#[derive(Clone, Copy, PartialEq)]
struct CapacityLayout;

impl LayoutPolicy for CapacityLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent
            .width
            .resolve_max()
            .unwrap_or_else(|| parent.width.clamp(Dp(180.0).into()));
        let height = parent.height.clamp(Dp(16.0).into());
        let mut result = LayoutResult::new(ComputedData { width, height });
        for child in scope.children() {
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(width), AxisConstraint::exact(height)),
            )?;
            result.place_child(child, PxPosition::ZERO);
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

/// The appearance of a [`gauge`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GaugeStyle {
    /// The platform default, a linear capacity bar on macOS.
    #[default]
    Automatic,
    /// A 16pt bar filled up to the value, under a centered label.
    LinearCapacity,
    /// A thin capsule with a marker at the value.
    AccessoryLinear,
    /// An 8pt bar filled up to the value, under a leading label.
    AccessoryLinearCapacity,
    /// An open dial with a marker at the value and a small label below.
    AccessoryCircular,
    /// A ring filled up to the value around a large label.
    AccessoryCircularCapacity,
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

/// # gauge
///
/// Shows a value within a range as a bar, dial, or ring.
///
/// ## Usage
///
/// Show storage, battery, or other levels that the user reads but does not edit.
///
/// ## Parameters
///
/// - `value`, `range` — finite level and increasing limits; the range defaults to `0.0..=1.0`.
/// - `label` — describes the gauge and is its default accessible name.
/// - `style` — bar, dial, or ring appearance; defaults to [`GaugeStyle::Automatic`].
/// - `tint` — fill color; defaults to green for capacity bars and the accent color for the
///   accessory capacity bar.
/// - `modifier` — sizing, placement, and additional semantics.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::gauge::{GaugeStyle, gauge};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// gauge(0.4).label("Storage");
/// gauge(72.0).range(0.0..=100.0).label("Battery").style(GaugeStyle::AccessoryCircularCapacity);
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn gauge(
    value: f64,
    range: Option<RangeInclusive<f64>>,
    #[prop(into)] label: Option<String>,
    style: Option<GaugeStyle>,
    tint: Option<Color>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let range = range.unwrap_or(0.0..=1.0);
    let (min, max) = (*range.start(), *range.end());
    assert!(
        min.is_finite() && max.is_finite() && min < max && (max - min).is_finite(),
        "gauge range must be finite and increasing"
    );
    assert!(value.is_finite(), "gauge value must be finite");
    let value = value.clamp(min, max);
    let fraction = (value - min) / (max - min);
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let dark = theme.appearance == Appearance::Dark;
    let ink = theme.label_color();
    let semantics = CapacitySemantics {
        value,
        min,
        max,
        enabled: true,
        label: accessibility_label.or_else(|| label.clone()),
        description: accessibility_description,
    };
    let modifier = modifier.unwrap_or_default().push_semantics(semantics);
    let label = label.filter(|label| !label.is_empty());
    match style.unwrap_or_default() {
        GaugeStyle::Automatic | GaugeStyle::LinearCapacity => {
            let fill = tint.unwrap_or_else(|| {
                if dark {
                    srgb(48, 209, 88)
                } else {
                    srgb(52, 199, 89)
                }
            });
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(modifier)
                .children(move || {
                    if let Some(label) = label.clone() {
                        spacer().modifier(Modifier::new().height(Dp(0.5)));
                        gauge_label(label, 13.0, 16.0, 400, ink);
                        spacer().modifier(Modifier::new().height(Dp(8.0)));
                    }
                    capacity_bar(fraction, fill, dark);
                    if label.is_some() {
                        spacer().modifier(Modifier::new().height(Dp(0.5)));
                    }
                });
        }
        GaugeStyle::AccessoryLinearCapacity => {
            column().modifier(modifier).children(move || {
                if let Some(label) = label.clone() {
                    gauge_label(label, 13.0, 16.0, 400, ink);
                }
                let bar = progress_view().value(fraction).modifier(
                    Modifier::new()
                        .fill_max_width()
                        .clear_and_set_semantics(Default::default()),
                );
                if let Some(tint) = tint {
                    bar.tint(tint);
                }
            });
        }
        GaugeStyle::AccessoryLinear => {
            layout(Stack).modifier(modifier.fill_max_width().height(Dp(8.0)).push_draw(
                LinearDial {
                    fraction,
                    color: tint.unwrap_or(ink),
                },
            ));
        }
        GaugeStyle::AccessoryCircular => {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(modifier.size(Dp(58.0), Dp(58.0)).push_draw(CircularDial {
                    fraction,
                    color: tint.unwrap_or(ink),
                }))
                .children(move || {
                    if let Some(label) = label.clone() {
                        boxed()
                            .alignment(Alignment::TopCenter)
                            .modifier(Modifier::new().width(Dp(58.0)).offset(Dp::ZERO, Dp(41.0)))
                            .children(move || gauge_label(label.clone(), 11.0, 16.5, 600, ink));
                    }
                });
        }
        GaugeStyle::AccessoryCircularCapacity => {
            boxed()
                .alignment(Alignment::Center)
                .modifier(modifier.size(Dp(58.0), Dp(58.0)).push_draw(CapacityRing {
                    fraction,
                    color: tint.unwrap_or(ink),
                    track: if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.2965),
                }))
                .children(move || {
                    if let Some(label) = label.clone() {
                        gauge_label(label, 25.0, 29.0, 500, ink);
                    }
                });
        }
    }
}

#[tessera]
fn gauge_label(content: String, size: f64, line_height: f64, weight: u16, color: Color) {
    text(content)
        .style(TextStyle {
            font_size: Dp(size),
            line_height: Dp(line_height),
            font_weight: weight,
            ..Default::default()
        })
        .color(color)
        .single_line(true)
        .modifier(Modifier::new().clear_and_set_semantics(
            tessera_foundation::modifier::SemanticsArgs {
                hidden: true,
                ..Default::default()
            },
        ));
}

/// The 16pt capacity bar: a track, the fill, and a 0.5pt inner outline.
#[tessera]
fn capacity_bar(fraction: f64, fill: Color, dark: bool) {
    let track = if dark {
        Color::WHITE.with_alpha(0.1)
    } else {
        Color::BLACK.with_alpha(0.07)
    };
    layout(CapacityLayout)
        .modifier(Modifier::new().clip_to_bounds())
        .child(move || {
            capacity_fill(track, false);
            capacity_fill(fill, false).fraction(fraction);
            capacity_fill(Color::BLACK.with_alpha(0.13), true);
        });
}

/// Stroke width of the accessory dials and rings, scaled with their 58pt size.
const DIAL_STROKE: f64 = 58.0 / 10.5;

fn dial(color: Color, shape: DialShape, marker: [Dp; 4]) -> PerceptualDialCommand {
    PerceptualDialCommand {
        color,
        shape,
        stroke: Dp(DIAL_STROKE),
        marker,
    }
}

#[derive(Clone, PartialEq)]
struct LinearDial {
    fraction: f64,
    color: Color,
}

impl DrawModifierNode for LinearDial {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let mut metadata = context.render_input.metadata_mut();
        let size = metadata.computed_data().unwrap_or(ComputedData::ZERO);
        let width = Dp::from_pixels_f32(size.width.to_f32()).0;
        let height = Dp::from_pixels_f32(size.height.to_f32()).0;
        let radius = height * 0.5;
        let marker = radius + (width - height) * self.fraction - width * 0.5;
        metadata.fragment_mut().push_draw_command(dial(
            self.color,
            DialShape::Capsule,
            [Dp(marker), Dp::ZERO, Dp(8.0), Dp(radius)],
        ));
        content.draw(context.render_input);
    }
}

#[derive(Clone, PartialEq)]
struct CircularDial {
    fraction: f64,
    color: Color,
}

impl DrawModifierNode for CircularDial {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let mut metadata = context.render_input.metadata_mut();
        let size = metadata.computed_data().unwrap_or(ComputedData::ZERO);
        let extent = Dp::from_pixels_f32(size.width.to_f32().min(size.height.to_f32())).0;
        let radius = extent * 0.5 - DIAL_STROKE * 0.5;
        let angle = (135.0 + 270.0 * self.fraction).to_radians();
        metadata.fragment_mut().push_draw_command(dial(
            self.color,
            DialShape::Arc {
                start: 135.0,
                sweep: 270.0,
            },
            [
                Dp(radius * angle.cos()),
                Dp(radius * angle.sin()),
                Dp(13.45 * 0.5),
                Dp(DIAL_STROKE * 0.5),
            ],
        ));
        content.draw(context.render_input);
    }
}

#[derive(Clone, PartialEq)]
struct CapacityRing {
    fraction: f64,
    color: Color,
    track: Color,
}

impl DrawModifierNode for CapacityRing {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let mut metadata = context.render_input.metadata_mut();
        let fragment = metadata.fragment_mut();
        fragment.push_draw_command(dial(self.track, DialShape::Ring, [Dp::ZERO; 4]));
        if self.fraction > 0.0 {
            fragment.push_draw_command(dial(
                self.color,
                DialShape::Arc {
                    start: 270.0,
                    sweep: (self.fraction * 360.0) as f32,
                },
                [Dp::ZERO; 4],
            ));
        }
        content.draw(context.render_input);
    }
}
