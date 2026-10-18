//! Floating value labels for slider handles.
//!
//! ## Usage
//!
//! Help users select exact values while adjusting settings.

use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, FrameNanosControl, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, PxRect, RenderSlot, State, current_frame_nanos,
    key,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
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
    anchor: State<PxRect>,
    label: String,
    visible: bool,
    interacting: bool,
    orientation: SliderOrientation,
    background: Color,
    foreground: Color,
    style: TextStyle,
}

#[derive(Clone, PartialEq)]
struct Host {
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
    start: u64,
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
            start: 0,
        }
    }

    fn running(&self) -> bool {
        self.value != f32::from(u8::from(self.target))
    }

    fn advance(&mut self, now: u64) {
        let duration = if self.target {
            400_000_000
        } else {
            150_000_000
        };
        let t = (now.saturating_sub(self.start) as f64 / duration as f64).clamp(0.0, 1.0) as f32;
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

    fn retarget(&mut self, target: bool, now: u64) {
        if self.target == target {
            return;
        }
        self.advance(now);
        self.origin = self.value;
        self.target = target;
        self.start = now;
    }
}

#[derive(Clone, PartialEq)]
struct HostLayout;

impl tessera_ui::LayoutModifierNode for HostLayout {
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
    anchor: State<PxRect>,
    origin: Arc<RwLock<PxRect>>,
    orientation: SliderOrientation,
    scale: f32,
}

impl PartialEq for LabelLayout {
    fn eq(&self, other: &Self) -> bool {
        self.anchor == other.anchor
            && Arc::ptr_eq(&self.origin, &other.origin)
            && self.orientation == other.orientation
            && self.scale == other.scale
    }
}

fn position(
    anchor: PxRect,
    popup: ComputedData,
    viewport: ComputedData,
    orientation: SliderOrientation,
    scale: f32,
) -> PxPosition {
    let gap = tessera_ui::Dp(4.0).to_pixels_f32();
    let margin = tessera_ui::Dp(8.0).to_pixels_f32();
    let (w, h) = (popup.width.to_f32(), popup.height.to_f32());
    let (width, height) = (viewport.width.to_f32(), viewport.height.to_f32());
    let (left, top) = (anchor.x.to_f32(), anchor.y.to_f32());
    let (right, bottom) = (
        (anchor.x + anchor.width).to_f32(),
        (anchor.y + anchor.height).to_f32(),
    );
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

impl tessera_ui::LayoutModifierNode for LabelLayout {
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
        if !self.anchor.is_alive() {
            return Ok(result);
        }
        let rect = self.anchor.get();
        let origin = *self.origin.read();
        let anchor = PxRect::new(
            rect.x - origin.x,
            rect.y - origin.y,
            rect.width,
            rect.height,
        );
        if anchor.width <= Px::ZERO
            || anchor.height <= Px::ZERO
            || anchor.x + anchor.width <= Px::ZERO
            || anchor.y + anchor.height <= Px::ZERO
            || anchor.x >= size.width
            || anchor.y >= size.height
        {
            return Ok(result);
        }
        let margin = tessera_ui::Dp(16.0).to_px();
        let constraint = Constraint::new(
            AxisConstraint::at_most((size.width - margin).max(Px::ZERO)),
            AxisConstraint::at_most((size.height - margin).max(Px::ZERO)),
        );
        let popup = child.measure(input, &constraint)?;
        result.place_child(
            child,
            position(anchor, popup.size(), size, self.orientation, self.scale),
        );
        Ok(result)
    }
}

#[tessera]
fn label_layer(request: Option<State<Request>>, origin: Option<Arc<RwLock<PxRect>>>) {
    #[cfg(test)]
    LABEL_BUILD_COUNT.set(LABEL_BUILD_COUNT.get() + 1);
    let request_state = request.expect("slider label request");
    let request = request_state.get();
    let origin = origin.expect("slider label origin");
    let now = current_frame_nanos();
    let reveal = remember(Reveal::new);
    #[cfg(test)]
    let popup_bounds = remember(|| PxRect::ZERO);
    #[cfg(test)]
    LABELS.with(|labels| {
        let mut labels = labels.borrow_mut();
        labels.retain(|(request, _, _)| request.is_alive());
        if !labels
            .iter()
            .any(|(request, _, _)| *request == request_state)
        {
            labels.push((request_state, popup_bounds, reveal));
        }
    });
    if reveal.with(|reveal| reveal.target != request.visible) {
        reveal.with_mut(|reveal| reveal.retarget(request.visible, now));
    }
    if reveal.with(Reveal::running) {
        receive_frame_nanos(move |now| {
            reveal.with_mut(|reveal| reveal.advance(now));
            if reveal.with(Reveal::running) {
                FrameNanosControl::Continue
            } else {
                FrameNanosControl::Stop
            }
        });
    }
    let scale = reveal.with(|reveal| reveal.value);
    let next = (request.clone(), scale);
    let data = remember(|| next.clone());
    if data.with(|current| current != &next) {
        data.set(next);
    }
    layout()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }))
        .policy(LabelLayout {
            anchor: request.anchor,
            origin,
            orientation: request.orientation,
            scale,
        })
        .child(move || {
            let (request, scale) = data.get();
            if scale > 0.0 {
                let dp = move |value| tessera_ui::Dp(value * f64::from(scale));
                let modifier = Modifier::new().alpha(scale).constrain(
                    Some(AxisConstraint::new(dp(48.0).to_px(), None)),
                    Some(AxisConstraint::new(dp(44.0).to_px(), None)),
                );
                #[cfg(test)]
                let modifier = modifier.on_globally_positioned(move |rect| {
                    if popup_bounds.get() != rect {
                        popup_bounds.set(rect);
                    }
                });
                surface()
                    .modifier(modifier)
                    .shape(Shape::CAPSULE)
                    .style(request.background.into())
                    .child(move || {
                        let scale = data.with(|(_, scale)| *scale);
                        let dp = move |value| tessera_ui::Dp(value * f64::from(scale));
                        boxed()
                            .modifier(
                                Modifier::new().padding(Padding::symmetric(dp(16.0), dp(12.0))),
                            )
                            .alignment(Alignment::Center)
                            .children(move || {
                                let (request, scale) = data.get();
                                let mut style = request.style;
                                style.font_size =
                                    tessera_ui::Dp(style.font_size.0 * f64::from(scale));
                                style.line_height = style
                                    .line_height
                                    .map(|height| tessera_ui::Dp(height.0 * f64::from(scale)));
                                style.letter_spacing =
                                    tessera_ui::Dp(style.letter_spacing.0 * f64::from(scale));
                                text()
                                    .content(request.label)
                                    .color(request.foreground)
                                    .style(style);
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
/// - `child` — page content containing standard, centered, or range sliders.
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
///         slider_value_indicator_provider().child(move || {
///             slider()
///                 .value(value.get())
///                 .on_change(move |next| value.set(next))
///                 .value_indicator(SliderValueIndicatorVisibility::OnInteraction);
///         });
///     });
///     assert_eq!(value.get(), 0.5);
/// }
/// content();
/// ```
#[tessera]
pub fn slider_value_indicator_provider(modifier: Option<Modifier>, child: Option<RenderSlot>) {
    let child = child.unwrap_or_else(RenderSlot::empty);
    if use_context::<Host>().is_some() {
        layout()
            .modifier(modifier.unwrap_or_default())
            .child_shared(child);
        return;
    }
    let entries = remember(Vec::<State<Request>>::new);
    let origin = remember(|| Arc::new(RwLock::new(PxRect::ZERO))).get();
    let bounds = Arc::clone(&origin);
    let next = (
        modifier.unwrap_or_else(|| Modifier::new().fill_max_size()),
        child,
    );
    let child_data = remember(|| next.clone());
    if child_data.get() != next {
        child_data.set(next);
    }
    provide_context(
        || Host { entries },
        move || {
            layout()
                .modifier(
                    child_data
                        .with(|(modifier, _)| modifier.clone())
                        .on_globally_positioned(move |rect| *bounds.write() = rect),
                )
                .policy(HostLayout)
                .child(move || {
                    layout().child(move || child_data.with(|(_, child)| *child).render());
                    let mut requests: Vec<_> = entries
                        .get()
                        .into_iter()
                        .filter(|request| request.is_alive())
                        .collect();
                    requests.sort_by_key(|request| request.with(|request| request.interacting));
                    for request in requests {
                        if request.is_alive() {
                            let origin = Arc::clone(&origin);
                            key(request, move || {
                                label_layer().request(request).origin(Arc::clone(&origin));
                            });
                        }
                    }
                });
        },
    );
}

#[tessera]
pub(super) fn register_indicator(
    anchor: Option<State<PxRect>>,
    label: Option<String>,
    visible: Option<bool>,
    interacting: Option<bool>,
    orientation: Option<SliderOrientation>,
) {
    let host = use_context::<Host>()
        .expect("slider value indicators require slider_value_indicator_provider")
        .get();
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
    let _registration = remember(|| Registration {
        entries: host.entries,
        request,
    });
    if !host.entries.with(|entries| entries.contains(&request)) {
        host.entries.with_mut(|entries| entries.push(request));
    }
}

#[cfg(test)]
type LabelRecord = (State<Request>, State<PxRect>, State<Reveal>);

#[cfg(test)]
thread_local! {
    static LABELS: std::cell::RefCell<Vec<LabelRecord>> = const { std::cell::RefCell::new(Vec::new()) };
    static LABEL_BUILD_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn observed_labels() -> Vec<(String, PxRect)> {
    LABELS.with(|labels| {
        labels
            .borrow()
            .iter()
            .filter_map(|(request, bounds, reveal)| {
                if !request.is_alive()
                    || !bounds.is_alive()
                    || !reveal.is_alive()
                    || reveal.with(|reveal| reveal.value) == 0.0
                {
                    return None;
                }
                Some((request.with(|request| request.label.clone()), bounds.get()))
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::Reveal;

    #[test]
    fn value_label_reveal_reverses_without_a_jump_and_settles_at_material_durations() {
        let mut reveal = Reveal::new();
        reveal.retarget(true, 1_000_000_000);
        reveal.advance(1_035_833_333);
        assert!((reveal.value - 0.0725).abs() < 0.00004);
        let halfway = reveal.value;
        reveal.retarget(false, 1_035_833_333);
        assert_eq!(reveal.value, halfway);
        reveal.advance(1_185_833_333);
        assert_eq!(reveal.value, 0.0);
        assert!(!reveal.running());
        reveal.retarget(true, 2_000_000_000);
        reveal.advance(2_399_000_000);
        assert!(reveal.running());
        reveal.advance(2_400_000_000);
        assert_eq!(reveal.value, 1.0);
        assert!(!reveal.running());
    }
}

#[cfg(test)]
pub(super) fn label_build_count() -> usize {
    LABEL_BUILD_COUNT.get()
}
