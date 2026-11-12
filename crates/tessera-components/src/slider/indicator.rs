//! Floating value labels for slider handles.
//!
//! ## Usage
//!
//! Help users select exact values while adjusting settings.

use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlotWith, State, key, launch,
    layout::{MeasureScope, PlacementResult, PlacementScope, Stack, layout},
    modifier::FocusModifierExt as _,
    remember, remember_with_key, tessera,
    time::Instant,
    use_context,
};

use crate::{
    alignment::Alignment,
    animation::{Motion, animate_to},
    boxed::boxed,
    progress::cubic_bezier_easing,
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::{MaterialTheme, TextStyle},
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
    pub(super) fn visible(self, interacting: bool, disabled: bool) -> bool {
        !disabled
            && match self {
                Self::Hidden => false,
                Self::OnInteraction => interacting,
                Self::Always => true,
            }
    }
}

#[derive(Clone, PartialEq)]
struct Request {
    anchor: State<tessera_ui::LayoutCoordinates>,
    label: String,
    visible: bool,
    interacting: bool,
    orientation: SliderOrientation,
    background: Color,
    foreground: Color,
    style: TextStyle,
}

/// The overlay registration target supplied to slider content.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliderValueIndicatorHost {
    entries: State<Vec<State<Request>>>,
}

struct Registration {
    entries: State<Vec<State<Request>>>,
    request: State<Request>,
}

impl Drop for Registration {
    fn drop(&mut self) {
        if self.entries.is_alive() {
            self.entries
                .with_mut(|entries| entries.retain(|entry| *entry != self.request));
        }
    }
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
                cubic_bezier_easing(t, 0.3, 0.0, 0.8, 0.15)
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
struct HostLayout;

impl tessera_ui::LayoutPolicy for HostLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let parent = input.parent_constraint();
        let constraint = Constraint::new(parent.width(), parent.height());
        let main = children[0].measure(input, &constraint)?;
        let mut result = LayoutResult::new(main.size());
        result.place_child(children[0], PxPosition::ZERO);
        let constraint = Constraint::new(
            AxisConstraint::exact(main.width),
            AxisConstraint::exact(main.height),
        );
        for child in &children[1..] {
            child.measure(input, &constraint)?;
            result.place_child(*child, PxPosition::ZERO);
        }
        Ok(result)
    }
}

#[derive(Clone)]
struct LabelLayout {
    anchor: State<tessera_ui::LayoutCoordinates>,
    origin: State<tessera_ui::LayoutCoordinates>,
    orientation: SliderOrientation,
    scale: f32,
}

impl PartialEq for LabelLayout {
    fn eq(&self, other: &Self) -> bool {
        self.anchor == other.anchor
            && self.origin == other.origin
            && self.orientation == other.orientation
            && self.scale == other.scale
    }
}

fn position(
    anchor: tessera_ui::FloatRect,
    popup: ComputedData,
    viewport: ComputedData,
    orientation: SliderOrientation,
    scale: f32,
) -> PxPosition {
    let gap = tessera_ui::Dp(4.0).to_pixels_f32();
    let margin = tessera_ui::Dp(8.0).to_pixels_f32();
    let (w, h) = (popup.width.to_f32(), popup.height.to_f32());
    let (width, height) = (viewport.width.to_f32(), viewport.height.to_f32());
    let (left, top) = (anchor.x, anchor.y);
    let (right, bottom) = (anchor.x + anchor.width, anchor.y + anchor.height);
    let (mut x, mut y) = match orientation {
        SliderOrientation::Horizontal => {
            let above = top - gap - h;
            (
                (left + right - w) * 0.5,
                if above >= margin { above } else { bottom + gap },
            )
        }
        SliderOrientation::Vertical => {
            let before = left - gap - w;
            (
                if before >= margin {
                    before
                } else {
                    right + gap
                },
                (top + bottom - h) * 0.5,
            )
        }
    };
    let travel = (1.0 - scale) * tessera_ui::Dp(8.8).to_pixels_f32();
    match orientation {
        SliderOrientation::Horizontal => y += if y < top { travel } else { -travel },
        SliderOrientation::Vertical => x += if x < left { travel } else { -travel },
    }
    PxPosition::from_f32_xy(
        x.clamp(
            margin.min(width),
            (width - w - margin).max(margin.min(width)),
        ),
        y.clamp(
            margin.min(height),
            (height - h - margin).max(margin.min(height)),
        ),
    )
}

impl LabelLayout {
    fn popup_position(&self, popup: ComputedData, size: ComputedData) -> Option<PxPosition> {
        if !self.anchor.is_alive() {
            return None;
        }
        let rect = self.anchor.get();
        let origin = self.origin.get();
        let anchor = origin.local_to_window.inverse_rect(rect.window_bounds());
        if anchor.width <= 0.0
            || anchor.height <= 0.0
            || anchor.x + anchor.width <= 0.0
            || anchor.y + anchor.height <= 0.0
            || anchor.x >= size.width.to_f32()
            || anchor.y >= size.height.to_f32()
        {
            return None;
        }
        Some(position(anchor, popup, size, self.orientation, self.scale))
    }
}

impl tessera_ui::LayoutPolicy for LabelLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = input.parent_constraint();
        let size = ComputedData {
            width: parent.width().clamp(Px::ZERO),
            height: parent.height().clamp(Px::ZERO),
        };
        let mut result = LayoutResult::new(size);
        let Some(child) = input.children().first().copied() else {
            return Ok(result);
        };
        let margin = tessera_ui::Dp(16.0).to_px();
        let constraint = Constraint::new(
            AxisConstraint::at_most((size.width - margin).max(Px::ZERO)),
            AxisConstraint::at_most((size.height - margin).max(Px::ZERO)),
        );
        let popup = child.measure(input, &constraint)?;
        if let Some(position) = input.placement(|| self.popup_position(popup.size(), size)) {
            result.place_child(child, position);
        }
        Ok(result)
    }

    fn place(
        &self,
        scope: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let mut result = LayoutResult::new(scope.size());
        if let Some(child) = scope.children().first().copied()
            && let Some(position) = self.popup_position(child.size(), scope.size())
        {
            result.place_child(child, position);
        }
        Some(result.into_placement_result())
    }
}

#[tessera]
fn label_layer(
    request: Option<State<Request>>,
    origin: Option<State<tessera_ui::LayoutCoordinates>>,
) {
    let request_state = request.expect("slider label request");
    let request = request_state.get();
    let origin = origin.expect("slider label origin");
    let reveal = remember(Reveal::new);
    launch(request.visible, animate_to(reveal, request.visible));
    let scale = reveal.with(|reveal| reveal.value);
    let next = (request.clone(), scale);
    let data = remember(|| next.clone());
    if data.with(|current| current != &next) {
        data.set(next);
    }
    tessera_ui::layout::layout(LabelLayout {
        anchor: request.anchor,
        origin,
        orientation: request.orientation,
        scale,
    })
    .modifier(Modifier::new().semantics(SemanticsArgs {
        hidden: true,
        ..Default::default()
    }))
    .child(move || {
        let (request, scale) = data.get();
        if scale > 0.0 {
            let dp = move |value| tessera_ui::Dp(value * f64::from(scale));
            let modifier = Modifier::new().alpha(scale).constrain(
                Some(AxisConstraint::new(dp(48.0).to_px(), None)),
                Some(AxisConstraint::new(dp(44.0).to_px(), None)),
            );
            #[cfg(test)]
            let modifier = modifier.semantics(SemanticsArgs {
                test_tag: Some("slider-value-label".into()),
                label: Some(request.label.clone()),
                ..Default::default()
            });
            surface()
                .modifier(modifier)
                .shape(Shape::CAPSULE)
                .style(request.background.into())
                .child(move || {
                    let scale = data.with(|(_, scale)| *scale);
                    let dp = move |value| tessera_ui::Dp(value * f64::from(scale));
                    boxed()
                        .modifier(Modifier::new().padding(Padding::symmetric(dp(16.0), dp(12.0))))
                        .alignment(Alignment::Center)
                        .children(move || {
                            let (request, scale) = data.get();
                            let mut style = request.style;
                            style.font_size = tessera_ui::Dp(style.font_size.0 * f64::from(scale));
                            style.line_height = style
                                .line_height
                                .map(|height| tessera_ui::Dp(height.0 * f64::from(scale)));
                            style.letter_spacing =
                                tessera_ui::Dp(style.letter_spacing.0 * f64::from(scale));
                            text(request.label).color(request.foreground).style(style);
                        });
                });
        }
    });
}

/// # slider_value_indicator_provider
///
/// Draw slider value labels above page content and scrolling containers.
///
/// ## Usage
///
/// Wrap a page that contains sliders with optional floating value indicators.
///
/// ## Parameters
///
/// - `modifier` — page placement and bounds; fills the available size by
///   default.
/// - `child` — page content receiving the explicit host for its slider labels.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     slider::{SliderValueIndicatorVisibility, slider, slider_value_indicator_provider},
///     theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
/// #[tessera]
/// fn content() {
///     let value = remember(|| 0.5_f32);
///     material_theme().child(move || {
///         slider_value_indicator_provider().child(move |host| {
///             slider(value.get()).value_indicator_host(host)
///                 .on_change(move |next| value.set(next))
///                 .value_indicator(SliderValueIndicatorVisibility::OnInteraction);
///         });
///     });
///     assert_eq!(value.get(), 0.5);
/// }
/// # tessera_ui::testing::UiTest::new().content(content).frame();
/// ```
#[tessera]
pub fn slider_value_indicator_provider(
    modifier: Option<Modifier>,
    child: Option<RenderSlotWith<SliderValueIndicatorHost>>,
) {
    let entries = remember(Vec::<State<Request>>::new);
    let origin = remember(tessera_ui::LayoutCoordinates::default);
    let next = (
        modifier.unwrap_or_else(|| Modifier::new().fill_max_size()),
        child,
    );
    let child_data = remember(|| next.clone());
    if child_data.get() != next {
        child_data.set(next);
    }
    tessera_ui::layout::layout(HostLayout)
        .modifier(
            child_data
                .with(|(modifier, _)| modifier.clone())
                .on_layout_coordinates(move |coordinates| {
                    origin.set_if_changed(coordinates);
                }),
        )
        .child(move || {
            layout(Stack).child(move || {
                if let Some(child) = child_data.with(|(_, child)| *child) {
                    child.render(SliderValueIndicatorHost { entries });
                }
            });
            let mut requests: Vec<_> = entries
                .get()
                .into_iter()
                .filter(|request| request.is_alive())
                .collect();
            requests.sort_by_key(|request| request.with(|request| request.interacting));
            for request in requests {
                if request.is_alive() {
                    key(request, move || {
                        label_layer().request(request).origin(origin);
                    });
                }
            }
        });
}

#[tessera]
pub(super) fn register_indicator(
    host: Option<SliderValueIndicatorHost>,
    anchor: Option<State<tessera_ui::LayoutCoordinates>>,
    label: Option<String>,
    visible: Option<bool>,
    interacting: Option<bool>,
    orientation: Option<SliderOrientation>,
) {
    let host = host.expect("visible slider indicators require a provider host");
    let theme = use_context::<MaterialTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let next = Request {
        anchor: anchor.expect("slider label anchor"),
        label: label.unwrap_or_default(),
        visible: visible.unwrap_or(false),
        interacting: interacting.unwrap_or(false),
        orientation: orientation.unwrap_or_default(),
        background: theme.color_scheme.inverse_surface,
        foreground: theme.color_scheme.inverse_on_surface,
        style: theme.typography.label_large,
    };
    let request = remember(|| next.clone());
    if request.get() != next {
        request.set(next);
    }
    let _registration = remember_with_key(host, || Registration {
        entries: host.entries,
        request,
    });
    if !host.entries.with(|entries| entries.contains(&request)) {
        host.entries.with_mut(|entries| entries.push(request));
    }
}

#[cfg(test)]
mod tests {
    use super::Reveal;
    use crate::animation::{Motion as _, test_instant};

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
