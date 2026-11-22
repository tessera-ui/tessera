use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Color, ComputedData, Dp, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
};

use crate::pipelines::text::{
    command::{TextCommand, TextConstraint},
    pipeline::{TextData, TextFormat, TextMeasureInfo, perceptual::PerceptualTextCommand},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TextMetrics {
    pub font_size: Dp,
    pub line_height: Dp,
    pub weight: u16,
    pub letter_spacing: Dp,
    pub family: Option<&'static str>,
    pub optical_size: Option<f32>,
    pub point_size: Option<f32>,
}

#[tessera]
pub(crate) fn text_node(
    content: Arc<str>,
    modifier: Modifier,
    color: Color,
    metrics: TextMetrics,
    single_line: bool,
    perceptual: bool,
    underline: Option<(Dp, Dp)>,
    label: Option<String>,
    description: Option<String>,
) {
    let semantics = TextSemantics {
        content: content.clone(),
        label,
        description,
    };
    // A draw-only rebuild can reuse measurement and its glyph buffer.
    let shaped = remember(|| Arc::new(Mutex::new(None))).get();
    layout(TextLayout {
        text: content,
        single_line,
        perceptual,
        metrics,
        shaped: shaped.clone(),
    })
    .draw(TextDraw {
        color,
        shaped,
        perceptual,
        underline,
    })
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
    perceptual: bool,
    metrics: TextMetrics,
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
        let fonts = crate::fonts::prepare(&self.text, self.metrics.family);
        let format = TextFormat::new(
            self.metrics.font_size,
            self.metrics.line_height,
            self.metrics.weight,
            self.metrics.letter_spacing,
            self.metrics.family,
        )
        .with_optical_size(self.metrics.optical_size)
        .with_point_size(self.metrics.point_size)
        .with_native_rasterization(self.perceptual);
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
            && self.metrics == other.metrics
            && self.single_line == other.single_line
            && self.perceptual == other.perceptual
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
    perceptual: bool,
    underline: Option<(Dp, Dp)>,
}

impl PartialEq for TextDraw {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.underline == other.underline
            && self.perceptual == other.perceptual
            && Arc::ptr_eq(&self.shaped, &other.shaped)
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
        if let Some((offset, thickness)) = self.underline {
            let info = text_data.measure_info();
            let mut metadata = input.metadata_mut();
            if let Some(size) = metadata.computed_data() {
                let end_x = size.width.to_f32().min(info.size[0] as f32);
                let top = (info.first_baseline + offset.to_pixels_f32()).round();
                let bottom =
                    (top + thickness.to_pixels_f32().round().max(1.0)).min(size.height.to_f32());
                if info.line_count == 1 && end_x > 0.0 && top >= 0.0 && top < bottom {
                    let dp = |px: f32| Dp::from_pixels_f64(px as f64);
                    metadata.fragment_mut().push_draw_command(
                        crate::pipelines::perceptual_fill::PerceptualFillCommand {
                            color: self.color,
                            radii: [Dp::ZERO; 2],
                            insets: [
                                dp(top),
                                dp(size.width.to_f32() - end_x),
                                dp(size.height.to_f32() - bottom),
                                Dp::ZERO,
                            ],
                            clip_radius: None,
                            edge_smoothness: 1.0,
                            stroke: None,
                            path: None,
                            line: None,
                            outline_width: Dp::ZERO,
                            snap_to_pixels: false,
                        },
                    );
                }
            }
        }
        if self.perceptual {
            input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(PerceptualTextCommand {
                    data: text_data,
                    tint: None,
                });
        } else {
            input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(TextCommand {
                    data: text_data,
                    offset: PxPosition::ZERO,
                });
        }
        content.draw(input);
    }
}
