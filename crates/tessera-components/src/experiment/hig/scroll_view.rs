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
        let children = scope.children();
        let mut content = ComputedData::ZERO;
        for child in &children {
            let size = child.measure(scope, &child_constraint)?.size();
            content.width = content.width.max(size.width);
            content.height = content.height.max(size.height);
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
        for child in children {
            result.place_child(child, PxPosition::new(Px(x), Px(y)));
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
}

impl PartialEq for Scrollers {
    fn eq(&self, other: &Self) -> bool {
        self.axes == other.axes
            && self.offset == other.offset
            && self.color == other.color
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
        if self.color.a <= 0.0 {
            return;
        }
        let extents = self
            .extents
            .lock()
            .map(|extents| *extents)
            .unwrap_or_default();
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
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
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
        })
        .push_draw(Scrollers {
            axes,
            offset: offset.get(),
            extents: extents.clone(),
            color,
        });
    layout(ScrollLayout {
        axes,
        offset: offset.get(),
        extents,
    })
    .modifier(modifier)
    .child(move || {
        if let Some(child) = child {
            scroll_content(axes, child);
        }
    });
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
