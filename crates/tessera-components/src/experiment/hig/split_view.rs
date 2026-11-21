//! Split views with a draggable divider between two panes.
//!
//! ## Usage
//!
//! Show a source list beside its detail, or an editor above its console.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, CursorEventContent, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, LayoutPolicy, LayoutResult, MeasurementError, Modifier,
    PointerEventPass, PointerInput, PointerInputModifierNode, PressKeyEventType, Px, PxPosition,
    RenderSlot, State,
    accesskit::Role,
    layout::{MeasureScope, Stack, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::pipelines::perceptual_fill::PerceptualFillCommand;

use super::theme::{Appearance, HigTheme};

#[cfg(test)]
#[path = "split_view_tests.rs"]
mod tests;

/// Thickness of the divider and its drag target.
const DIVIDER: f64 = 1.0;
const DRAG_TARGET: f64 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SplitAxis {
    Horizontal,
    Vertical,
}

/// Places the first pane, the divider, and the second pane along the axis. `first` is the
/// first pane's extent in points, kept within `min` of both edges.
#[derive(Clone, Copy, PartialEq)]
struct SplitLayout {
    axis: SplitAxis,
    first: f64,
    min: f64,
    collapsed: bool,
}

impl SplitLayout {
    fn first_extent(&self, total: Px) -> Px {
        if self.collapsed {
            return Px::ZERO;
        }
        let total = Dp::from_pixels_f32(total.to_f32()).0;
        let upper = (total - DIVIDER - self.min).max(self.min);
        Px::from(Dp(self.first.clamp(self.min.min(upper), upper)))
    }
}

impl LayoutPolicy for SplitLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.resolve_max().unwrap_or(Px::from(Dp(600.0)));
        let height = parent.height.resolve_max().unwrap_or(Px::from(Dp(400.0)));
        let horizontal = self.axis == SplitAxis::Horizontal;
        let total = if horizontal { width } else { height };
        let first = self.first_extent(total);
        let divider = if self.collapsed {
            Px::ZERO
        } else {
            Px::from(Dp(DIVIDER))
        };
        let second = (total - first - divider).max(Px::ZERO);
        // Children are the two panes and then the divider, whose drag target overlaps both.
        let target = if self.collapsed {
            Px::ZERO
        } else {
            Px::from(Dp(DRAG_TARGET))
        };
        let target_start = first + divider / 2 - target / 2;
        let slots = [
            (Px::ZERO, first),
            (first + divider, second),
            (target_start, target),
        ];
        let children = scope.children();
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (child, (offset, extent)) in children.into_iter().zip(slots) {
            let constraint = if horizontal {
                Constraint::new(AxisConstraint::exact(extent), AxisConstraint::exact(height))
            } else {
                Constraint::new(AxisConstraint::exact(width), AxisConstraint::exact(extent))
            };
            child.measure(scope, &constraint)?;
            let position = if horizontal {
                PxPosition::new(offset, Px::ZERO)
            } else {
                PxPosition::new(Px::ZERO, offset)
            };
            result.place_child(child, position);
        }
        Ok(result)
    }
}

/// Drags the divider: moves `first` with the pointer while pressed.
struct DividerDrag {
    axis: SplitAxis,
    first: State<f64>,
    anchor: State<Option<(f32, f64)>>,
}

impl PointerInputModifierNode for DividerDrag {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let Some(position) = input.cursor_position_abs() else {
            return;
        };
        let along = if self.axis == SplitAxis::Horizontal {
            position.x
        } else {
            position.y
        };
        for change in input.pointer_changes.iter_mut() {
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left) if !change.is_consumed() => {
                    self.anchor.set(Some((along, self.first.get())));
                    change.consume();
                }
                CursorEventContent::Released(PressKeyEventType::Left) => {
                    if self.anchor.get().is_some() {
                        self.anchor.set(None);
                        change.consume();
                    }
                }
                _ => {}
            }
        }
        if let Some((start, first)) = self.anchor.get() {
            let moved = f64::from(Dp::from_pixels_f32(along - start).0);
            if (self.first.get() - (first + moved)).abs() > f64::EPSILON {
                self.first.set(first + moved);
            }
        }
    }
}

/// The one-point line centred in the divider's drag target.
#[derive(Clone, PartialEq)]
struct DividerLine(Color, SplitAxis);

impl DrawModifierNode for DividerLine {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: true,
                path: None,
                color: self.0,
                radii: [Dp::ZERO; 2],
                insets: {
                    let side = Dp((DRAG_TARGET - DIVIDER) / 2.0);
                    match self.1 {
                        SplitAxis::Horizontal => [Dp::ZERO, side, Dp::ZERO, side],
                        SplitAxis::Vertical => [side, Dp::ZERO, side, Dp::ZERO],
                    }
                },
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

/// Two panes with a divider; `first` is the first pane's width or height in points.
#[tessera]
pub(super) fn split(
    axis: SplitAxis,
    first: State<f64>,
    min: f64,
    collapsed: bool,
    first_pane: RenderSlot,
    second_pane: RenderSlot,
    modifier: Option<Modifier>,
) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let anchor = remember(|| None);
    let color = if dark {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::BLACK.with_alpha(0.12)
    };
    layout(SplitLayout {
        axis,
        first: first.get(),
        min,
        collapsed,
    })
    .modifier(modifier.unwrap_or_default().clip_to_bounds())
    .child(move || {
        pane(first_pane);
        pane(second_pane);
        divider(axis, first, anchor, color, collapsed);
    });
}

#[tessera]
fn pane(content: RenderSlot) {
    layout(Stack)
        .modifier(Modifier::new().clip_to_bounds())
        .child(content);
}

#[tessera]
fn divider(
    axis: SplitAxis,
    first: State<f64>,
    anchor: State<Option<(f32, f64)>>,
    color: Color,
    collapsed: bool,
) {
    let mut modifier = Modifier::new()
        .push_draw(DividerLine(color, axis))
        .semantics(SemanticsArgs {
            role: Some(Role::Splitter),
            ..Default::default()
        });
    if !collapsed {
        modifier = modifier
            .push_pointer_input(DividerDrag {
                axis,
                first,
                anchor,
            })
            .hover_cursor_icon(match axis {
                SplitAxis::Horizontal => CursorIcon::ColResize,
                SplitAxis::Vertical => CursorIcon::RowResize,
            });
    }
    layout(Stack).modifier(modifier);
}

/// # h_split_view
///
/// Shows two panes side by side with a draggable divider between them.
///
/// ## Usage
///
/// Place a browser beside its preview, or a list beside its inspector.
///
/// ## Parameters
///
/// - `leading`, `trailing` — the two panes.
/// - `leading_width` — initial width of the leading pane; defaults to 200pt.
/// - `modifier` — size and placement; the view fills the space its container offers.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{split_view::h_split_view, text::text};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// h_split_view()
///     .leading(|| {
///         text("Files");
///     })
///     .trailing(|| {
///         text("Preview");
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn h_split_view(
    leading: Option<RenderSlot>,
    trailing: Option<RenderSlot>,
    leading_width: Option<Dp>,
    modifier: Option<Modifier>,
) {
    let width = remember(|| leading_width.unwrap_or(Dp(200.0)).0);
    split(
        SplitAxis::Horizontal,
        width,
        60.0,
        false,
        slot(leading),
        slot(trailing),
    )
    .modifier_optional(modifier);
}

/// # v_split_view
///
/// Shows two panes stacked vertically with a draggable divider between them.
///
/// ## Usage
///
/// Place an editor above its console, or a message list above the open message.
///
/// ## Parameters
///
/// - `top`, `bottom` — the two panes.
/// - `top_height` — initial height of the top pane; defaults to 150pt.
/// - `modifier` — size and placement; the view fills the space its container offers.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{split_view::v_split_view, text::text};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// v_split_view()
///     .top(|| {
///         text("Messages");
///     })
///     .bottom(|| {
///         text("Message");
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn v_split_view(
    top: Option<RenderSlot>,
    bottom: Option<RenderSlot>,
    top_height: Option<Dp>,
    modifier: Option<Modifier>,
) {
    let height = remember(|| top_height.unwrap_or(Dp(150.0)).0);
    split(
        SplitAxis::Vertical,
        height,
        40.0,
        false,
        slot(top),
        slot(bottom),
    )
    .modifier_optional(modifier);
}

fn slot(content: Option<RenderSlot>) -> RenderSlot {
    content.unwrap_or_else(RenderSlot::empty)
}
