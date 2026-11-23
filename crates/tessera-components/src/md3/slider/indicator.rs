use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, Dp, Modifier, PxPosition, launch,
    popup::{PopupPositionInput, PopupPositionPolicy, popup},
    remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    alignment::Alignment,
    animation::{Motion, animate_to},
    boxed::boxed,
    md3::{progress::cubic_bezier_easing, surface::surface, text::text, theme::MaterialTheme},
    shape_def::Shape,
};

use super::SliderOrientation;

/// When a slider's floating value label is visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderValueIndicatorVisibility {
    /// Keep the label hidden.
    #[default]
    Hidden,
    /// Show the label while its handle is focused or being dragged.
    OnInteraction,
    /// Show the label whenever its enabled handle is on screen.
    Always,
}

impl SliderValueIndicatorVisibility {
    pub(super) fn indicator(
        self,
        interacting: bool,
        disabled: bool,
        label: impl FnOnce() -> String,
    ) -> Option<ValueIndicator> {
        (self != Self::Hidden).then(|| ValueIndicator {
            label: label(),
            visible: !disabled && (self == Self::Always || interacting),
        })
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct ValueIndicator {
    label: String,
    visible: bool,
}

#[derive(Clone, Copy)]
struct Reveal {
    value: f32,
    origin: f32,
    target: bool,
    start: Option<Instant>,
}

fn emphasized(progress: f32) -> f32 {
    if progress <= 1.0 / 6.0 {
        cubic_bezier_easing(progress * 6.0, 0.3, 0.0, 0.8, 0.15) * 0.4
    } else {
        let t = (progress - 1.0 / 6.0) * 1.2;
        0.4 + cubic_bezier_easing(t, 0.05, 0.7, 0.1, 1.0) * 0.6
    }
}

impl Reveal {
    fn new() -> Self {
        Self {
            value: 0.0,
            origin: 0.0,
            target: false,
            start: None,
        }
    }

    fn running(&self) -> bool {
        self.value != f32::from(u8::from(self.target))
    }

    fn tick(&mut self, now: Instant) {
        let duration = if self.target { 0.4 } else { 0.15 };
        let t = self.start.map_or(1.0, |start| {
            (now.saturating_duration_since(start).as_secs_f32() / duration).clamp(0.0, 1.0)
        });
        let target = f32::from(u8::from(self.target));
        if t >= 1.0 {
            self.value = target;
        } else {
            let t = if self.target {
                emphasized(t)
            } else {
                cubic_bezier_easing(t, 0.3, 0.0, 0.8, 1.0)
            };
            self.value = self.origin + (target - self.origin) * t;
        }
    }
}

impl Motion for Reveal {
    type Target = bool;

    fn is_settled_at(&self, &target: &bool) -> bool {
        self.target == target && !self.running()
    }

    fn retarget(&mut self, target: bool, now: Instant) {
        self.tick(now);
        if self.target != target {
            self.origin = self.value;
            self.target = target;
            self.start = Some(now);
        }
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.tick(now);
        self.running()
    }
}

#[derive(Clone, PartialEq)]
struct IndicatorPosition {
    orientation: SliderOrientation,
    scale: f32,
}

impl PopupPositionPolicy for IndicatorPosition {
    fn position(&self, input: &PopupPositionInput) -> PxPosition {
        let gap = Dp(4.0).to_pixels_f32();
        let margin = Dp(8.0).to_pixels_f32();
        let bounds = input.bounds;
        let anchor = input.anchor.window_bounds();
        let (w, h) = (
            input.popup_size.width.to_f32(),
            input.popup_size.height.to_f32(),
        );
        let margin_x = margin.min((bounds.width - w).max(0.0) * 0.5);
        let margin_y = margin.min((bounds.height - h).max(0.0) * 0.5);
        let (left, top) = (anchor.x, anchor.y);
        let (right, bottom) = (anchor.x + anchor.width, anchor.y + anchor.height);
        let travel = (1.0 - self.scale) * Dp(8.8).to_pixels_f32();
        let (x, y) = match self.orientation {
            SliderOrientation::Horizontal => {
                let above = top - gap - h;
                (
                    (left + right - w) * 0.5,
                    if above >= bounds.y + margin_y {
                        above + travel
                    } else {
                        bottom + gap - travel
                    },
                )
            }
            SliderOrientation::Vertical => {
                let before = left - gap - w;
                (
                    if before >= bounds.x + margin_x {
                        before + travel
                    } else {
                        right + gap - travel
                    },
                    (top + bottom - h) * 0.5,
                )
            }
        };
        PxPosition::from_f32_xy(
            x.clamp(
                bounds.x + margin_x,
                (bounds.x + bounds.width - w - margin_x).max(bounds.x + margin_x),
            ),
            y.clamp(
                bounds.y + margin_y,
                (bounds.y + bounds.height - h - margin_y).max(bounds.y + margin_y),
            ),
        )
    }
}

#[tessera]
pub(super) fn value_indicator(
    indicator: ValueIndicator,
    orientation: SliderOrientation,
    anchor_visible: bool,
) {
    let reveal = remember(Reveal::new);
    launch(indicator.visible, animate_to(reveal, indicator.visible));
    let scale = reveal.with(|reveal| reveal.value);
    if !anchor_visible || scale <= 0.0 {
        return;
    }
    let theme = use_context::<MaterialTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let label = indicator.label;
    let dp = move |value| Dp(value * f64::from(scale));
    let mut style = theme.typography.label_large;
    style.font_size = dp(style.font_size.0);
    style.line_height = style.line_height.map(|height| dp(height.0));
    style.letter_spacing = dp(style.letter_spacing.0);
    let foreground = theme.color_scheme.inverse_on_surface;
    let background = theme.color_scheme.inverse_surface;
    popup(IndicatorPosition { orientation, scale })
        .modifier(Modifier::new().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }))
        .child(move || {
            let modifier = Modifier::new().alpha(scale).constrain(
                Some(AxisConstraint::new(dp(48.0).to_px(), None)),
                Some(AxisConstraint::new(dp(44.0).to_px(), None)),
            );
            #[cfg(test)]
            let modifier = modifier.semantics(SemanticsArgs {
                test_tag: Some("slider-value-label".into()),
                label: Some(label.clone()),
                ..Default::default()
            });
            let label = label.clone();
            surface()
                .modifier(modifier)
                .shape(Shape::CAPSULE)
                .style(background.into())
                .child(move || {
                    let label = label.clone();
                    boxed()
                        .modifier(Modifier::new().padding(Padding::symmetric(dp(16.0), dp(12.0))))
                        .alignment(Alignment::Center)
                        .children(move || text(label.clone()).color(foreground).style(style));
                });
        });
}

#[cfg(test)]
mod tests {
    use crate::animation::{Motion as _, test_instant};

    use super::Reveal;

    #[test]
    fn value_label_reveal_reverses_without_a_jump_and_settles_at_material_durations() {
        let mut reveal = Reveal::new();
        reveal.retarget(true, test_instant(1_000_000_000));
        reveal.tick(test_instant(1_035_833_333));
        assert!((reveal.value - 0.0725).abs() < 0.00004);
        let halfway = reveal.value;
        reveal.retarget(false, test_instant(1_035_833_333));
        assert_eq!(reveal.value, halfway);
        reveal.tick(test_instant(1_185_833_333));
        assert_eq!(reveal.value, 0.0);
        assert!(!reveal.running());
        reveal.retarget(true, test_instant(2_000_000_000));
        reveal.tick(test_instant(2_399_000_000));
        assert!(reveal.running());
        reveal.tick(test_instant(2_400_000_000));
        assert_eq!(reveal.value, 1.0);
        assert!(!reveal.running());
    }
}
