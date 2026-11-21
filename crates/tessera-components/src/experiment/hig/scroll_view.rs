//! Scroll views with overlay scrollers.
//!
//! ## Usage
//!
//! Show content larger than its container, such as long forms, documents, and lists.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, CursorEventContent, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, LayoutPolicy, LayoutResult, MeasurementError, Modifier,
    PointerEventPass, PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot, State,
    launch,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera, use_context,
};

use crate::{
    animation::{Delay, animate_to, run_frames},
    column::column,
    pipelines::perceptual_fill::PerceptualFillCommand,
    row::row,
};

use super::{
    motion::Eased,
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "scroll_view_tests.rs"]
mod tests;

/// How long scrollers stay after scrolling stops.
const SCROLLER_HOLD: Duration = Duration::from_secs(1);
/// How long scrollers take to fade out.
const SCROLLER_FADE: Duration = Duration::from_millis(250);

/// The directions a [`scroll_view`] scrolls in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxes {
    /// Up and down.
    #[default]
    Vertical,
    /// Left and right.
    Horizontal,
    /// Both directions.
    Both,
}

impl ScrollAxes {
    fn vertical(self) -> bool {
        self != Self::Horizontal
    }

    fn horizontal(self) -> bool {
        self != Self::Vertical
    }
}

/// How content fades where it scrolls under the top edge of a [`scroll_view`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollEdgeEffectStyle {
    /// The platform default, a soft edge on macOS.
    #[default]
    Automatic,
    /// Content fades into the window background over the top 24pt.
    Soft,
    /// A line separates the edge from content scrolled under it.
    Hard,
}

/// Height of the soft edge effect, and the number of bands it is drawn with.
const SOFT_EDGE: f64 = 24.0;
const SOFT_EDGE_BANDS: usize = 12;

/// Content and viewport sizes from the latest measurement, in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Extents {
    content: (f32, f32),
    viewport: (f32, f32),
}

impl Extents {
    fn max_offset(self) -> (f32, f32) {
        (
            (self.content.0 - self.viewport.0).max(0.0),
            (self.content.1 - self.viewport.1).max(0.0),
        )
    }
}

#[derive(Clone)]
struct ScrollLayout {
    axes: ScrollAxes,
    offset: (f32, f32),
    extents: Arc<Mutex<Extents>>,
}

impl PartialEq for ScrollLayout {
    fn eq(&self, other: &Self) -> bool {
        self.axes == other.axes
            && self.offset == other.offset
            && Arc::ptr_eq(&self.extents, &other.extents)
    }
}

impl LayoutPolicy for ScrollLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let free = |scrolls: bool, axis: AxisConstraint| {
            if scrolls {
                AxisConstraint::new(Px::ZERO, None)
            } else {
                axis
            }
        };
        let child_constraint = Constraint::new(
            free(self.axes.horizontal(), parent.width),
            free(self.axes.vertical(), parent.height),
        );
        // The first child is the content; the second draws scrollers and edge effects above it.
        let children = scope.children();
        let mut content = ComputedData::ZERO;
        if let Some(child) = children.first() {
            content = child.measure(scope, &child_constraint)?.size();
        }
        let viewport = ComputedData {
            width: parent.width.clamp(content.width),
            height: parent.height.clamp(content.height),
        };
        let extents = Extents {
            content: (content.width.to_f32(), content.height.to_f32()),
            viewport: (viewport.width.to_f32(), viewport.height.to_f32()),
        };
        if let Ok(mut current) = self.extents.lock() {
            *current = extents;
        }
        let (max_x, max_y) = extents.max_offset();
        let x = -self.offset.0.clamp(0.0, max_x).round() as i32;
        let y = -self.offset.1.clamp(0.0, max_y).round() as i32;
        let mut result = LayoutResult::new(viewport);
        if let Some(child) = children.first() {
            result.place_child(*child, PxPosition::new(Px(x), Px(y)));
        }
        if let Some(overlay) = children.get(1) {
            overlay.measure(
                scope,
                &Constraint::new(
                    AxisConstraint::exact(viewport.width),
                    AxisConstraint::exact(viewport.height),
                ),
            )?;
            result.place_child(*overlay, PxPosition::ZERO);
        }
        Ok(result)
    }
}

struct ScrollInput {
    axes: ScrollAxes,
    offset: State<(f32, f32)>,
    activity: State<u64>,
    extents: Arc<Mutex<Extents>>,
}

impl PointerInputModifierNode for ScrollInput {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let extents = self
            .extents
            .lock()
            .map(|extents| *extents)
            .unwrap_or_default();
        let (max_x, max_y) = extents.max_offset();
        for change in input.pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            let CursorEventContent::Scroll(scroll) = change.content.clone() else {
                continue;
            };
            let (dx, dy) = scroll.pixel_delta();
            let (x, y) = self.offset.get();
            let x = x.clamp(0.0, max_x);
            let y = y.clamp(0.0, max_y);
            let next_x = if self.axes.horizontal() {
                (x - dx).clamp(0.0, max_x)
            } else {
                x
            };
            let next_y = if self.axes.vertical() {
                (y - dy).clamp(0.0, max_y)
            } else {
                y
            };
            let (used_x, used_y) = (x - next_x, y - next_y);
            if used_x != 0.0 || used_y != 0.0 {
                self.offset.set((next_x, next_y));
                self.activity.set(self.activity.get() + 1);
            }
            let remaining_x = dx - used_x;
            let remaining_y = dy - used_y;
            if remaining_x.abs() < f32::EPSILON && remaining_y.abs() < f32::EPSILON {
                change.consume();
            } else if used_x != 0.0 || used_y != 0.0 {
                let scale_x = if dx != 0.0 { remaining_x / dx } else { 0.0 };
                let scale_y = if dy != 0.0 { remaining_y / dy } else { 0.0 };
                let mut remainder = scroll.clone();
                remainder.delta_x *= scale_x;
                remainder.delta_y *= scale_y;
                change.content = CursorEventContent::Scroll(remainder);
            }
        }
    }
}

#[derive(Clone)]
struct Scrollers {
    axes: ScrollAxes,
    offset: (f32, f32),
    extents: Arc<Mutex<Extents>>,
    color: Color,
    /// The top edge effect and the window background it fades into.
    edge: Option<(ScrollEdgeEffectStyle, Color)>,
}

impl PartialEq for Scrollers {
    fn eq(&self, other: &Self) -> bool {
        self.axes == other.axes
            && self.offset == other.offset
            && self.color == other.color
            && self.edge == other.edge
            && Arc::ptr_eq(&self.extents, &other.extents)
    }
}

fn thumb(color: Color, insets: [Dp; 4]) -> PerceptualFillCommand {
    PerceptualFillCommand {
        outline_width: Dp::ZERO,
        snap_to_pixels: false,
        path: None,
        color,
        radii: [Dp(3.5); 2],
        insets,
        clip_radius: None,
        line: None,
        edge_smoothness: 1.0,
        stroke: None,
    }
}

impl DrawModifierNode for Scrollers {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        content.draw(context.render_input);
        let extents = self
            .extents
            .lock()
            .map(|extents| *extents)
            .unwrap_or_default();
        if let Some((style, background)) = self.edge
            && self.offset.1 > 0.0
        {
            let height = f64::from(Dp::from_pixels_f32(extents.viewport.1).0);
            let mut metadata = context.render_input.metadata_mut();
            let fragment = metadata.fragment_mut();
            if style == ScrollEdgeEffectStyle::Hard {
                fragment.push_draw_command(thumb(
                    Color::BLACK.with_alpha(0.12),
                    [Dp::ZERO, Dp::ZERO, Dp(height - 1.0), Dp::ZERO],
                ));
            } else {
                // Stacked bands approximate a fade from the background to clear.
                let band = SOFT_EDGE / SOFT_EDGE_BANDS as f64;
                for index in 0..SOFT_EDGE_BANDS {
                    let alpha = 0.85 * (1.0 - index as f32 / SOFT_EDGE_BANDS as f32);
                    let top = band * index as f64;
                    fragment.push_draw_command(PerceptualFillCommand {
                        radii: [Dp::ZERO; 2],
                        ..thumb(
                            background.with_alpha(alpha),
                            [
                                Dp(top),
                                Dp::ZERO,
                                Dp((height - top - band).max(0.0)),
                                Dp::ZERO,
                            ],
                        )
                    });
                }
            }
        }
        if self.color.a <= 0.0 {
            return;
        }
        let (max_x, max_y) = extents.max_offset();
        let to_dp = |px: f32| f64::from(Dp::from_pixels_f32(px).0);
        let (width, height) = (to_dp(extents.viewport.0), to_dp(extents.viewport.1));
        let mut metadata = context.render_input.metadata_mut();
        let fragment = metadata.fragment_mut();
        // Thumbs are 7pt thick, 2pt from the edge, and at least 18pt long.
        if self.axes.vertical() && max_y > 0.0 {
            let length = (height * f64::from(extents.viewport.1 / extents.content.1)).max(18.0);
            let travel = height - length - 4.0;
            let top = 2.0 + travel * f64::from(self.offset.1.clamp(0.0, max_y) / max_y);
            fragment.push_draw_command(thumb(
                self.color,
                [Dp(top), Dp(2.0), Dp(height - top - length), Dp(width - 9.0)],
            ));
        }
        if self.axes.horizontal() && max_x > 0.0 {
            let length = (width * f64::from(extents.viewport.0 / extents.content.0)).max(18.0);
            let travel = width - length - 4.0;
            let left = 2.0 + travel * f64::from(self.offset.0.clamp(0.0, max_x) / max_x);
            fragment.push_draw_command(thumb(
                self.color,
                [
                    Dp(height - 9.0),
                    Dp(width - left - length),
                    Dp(2.0),
                    Dp(left),
                ],
            ));
        }
    }
}

/// # scroll_view
///
/// Scrolls content that is larger than the view, with scrollers that appear while scrolling.
///
/// ## Usage
///
/// Show long forms, documents, and other content that does not fit in its container.
///
/// ## Parameters
///
/// - `axes` — directions the content scrolls in; defaults to [`ScrollAxes::Vertical`].
/// - `shows_indicators` — whether scrollers appear while scrolling; defaults to true.
/// - `edge_effect` — how content fades under the top edge once scrolled; defaults to
///   [`ScrollEdgeEffectStyle::Automatic`].
/// - `modifier` — size and placement; the view fills the space its container offers.
/// - `child` — the scrolled content.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{scroll_view::scroll_view, text::text};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// scroll_view().child(|| {
///     for line in 0..50 {
///         text(format!("Line {line}"));
///     }
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn scroll_view(
    axes: Option<ScrollAxes>,
    shows_indicators: Option<bool>,
    edge_effect: Option<ScrollEdgeEffectStyle>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let axes = axes.unwrap_or_default();
    let offset = remember(|| (0.0f32, 0.0f32));
    let activity = remember(|| 0u64);
    let extents = remember(|| Arc::new(Mutex::new(Extents::default()))).get();
    let shown = remember(|| Eased::new(0.0, SCROLLER_FADE));
    let count = activity.get();
    launch(count, async move {
        if count == 0 {
            return;
        }
        shown.set(Eased::new(1.0, SCROLLER_FADE));
        let mut hold = Delay::new(None, SCROLLER_HOLD);
        run_frames(shown, move |_, now| !hold.elapsed(now)).await;
        animate_to(shown, 0.0).await;
    });
    let appearance = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance)
        .unwrap_or_default();
    let dark = appearance == Appearance::Dark;
    let alpha = if shows_indicators.unwrap_or(true) {
        shown.with(Eased::value)
    } else {
        0.0
    };
    let color = if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.5 * alpha);
    let modifier = modifier
        .unwrap_or_default()
        .clip_to_bounds()
        .push_pointer_input(ScrollInput {
            axes,
            offset,
            activity,
            extents: extents.clone(),
        });
    let scrollers = Scrollers {
        axes,
        offset: offset.get(),
        extents: extents.clone(),
        color,
        edge: Some((
            edge_effect.unwrap_or_default(),
            super::toolbar::window_background(appearance),
        )),
    };
    layout(ScrollLayout {
        axes,
        offset: offset.get(),
        extents,
    })
    .modifier(modifier)
    .child(move || {
        let child = child.unwrap_or_else(RenderSlot::empty);
        scroll_content(axes, child);
        scroll_overlay(scrollers.clone());
    });
}

#[tessera]
fn scroll_overlay(scrollers: Scrollers) {
    tessera_ui::layout::layout(tessera_ui::layout::Stack)
        .modifier(Modifier::new().push_draw(scrollers));
}

/// Stacks the content along the scrolled axis.
#[tessera]
fn scroll_content(axes: ScrollAxes, child: RenderSlot) {
    if axes == ScrollAxes::Horizontal {
        row().children_shared(child);
    } else {
        column().children_shared(child);
    }
}
