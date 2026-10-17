//! Anchored plain and rich contextual tooltips.
//!
//! ## Usage
//!
//! Explain controls on hover, keyboard focus, or touch long press.

use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    AxisConstraint, Callback, ComputedData, CursorEventContent, Dp, FocusDirection, FocusGroupNode,
    FocusRequester, FocusState, FrameNanosControl, GestureState, KeyboardInput,
    KeyboardInputModifierNode, LayoutResult, MOUSE_POINTER_ID, MeasurementError, Modifier,
    PointerInput, PointerInputModifierNode, PressKeyEventType, Px, PxPosition, PxRect, RenderSlot,
    State,
    accesskit::{Live, Role},
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use crate::{
    button::button,
    column::column,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::MaterialTheme,
};

const SHOW_DELAY: u64 = 500_000_000;
const EXIT_DELAY: u64 = 100_000_000;
const TOUCH_DURATION: u64 = 1_500_000_000;

/// Tooltip content and interaction style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipVariant {
    /// A brief non-interactive description with an inverse surface color.
    #[default]
    Plain,
    /// Supporting text with an optional title and action.
    Rich,
}

/// Preferred tooltip position; the opposite side is used when space is limited.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPlacement {
    /// Center the tooltip above the anchor.
    #[default]
    Above,
    /// Center the tooltip below the anchor.
    Below,
    /// Place the tooltip before the anchor in a left-to-right layout.
    Start,
    /// Place the tooltip after the anchor in a left-to-right layout.
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TouchPress {
    pointer: u64,
    origin: PxPosition,
    since: u64,
    triggered: bool,
}

/// Shared tooltip visibility, trigger bounds, and input state.
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipController {
    open: bool,
    suppressed: bool,
    anchor_hovered: bool,
    popup_hovered: bool,
    anchor_focused: bool,
    popup_focused: bool,
    manual: bool,
    ignore_anchor_focus: bool,
    description: String,
    persistent: bool,
    interactive: bool,
    mouse_present: bool,
    touch_focus_suppressed: bool,
    pending_show: Option<u64>,
    pending_hide: Option<u64>,
    touch: Option<TouchPress>,
    anchor: PxRect,
    focus: FocusRequester,
    popup_focus: FocusRequester,
    popup_group: FocusGroupNode,
}

impl Default for TooltipController {
    fn default() -> Self {
        Self {
            open: false,
            suppressed: false,
            anchor_hovered: false,
            popup_hovered: false,
            anchor_focused: false,
            popup_focused: false,
            manual: false,
            ignore_anchor_focus: false,
            description: String::new(),
            persistent: false,
            interactive: false,
            mouse_present: false,
            touch_focus_suppressed: false,
            pending_show: None,
            pending_hide: None,
            touch: None,
            anchor: PxRect::ZERO,
            focus: FocusRequester::new(),
            popup_focus: FocusRequester::new(),
            popup_group: FocusGroupNode::new(),
        }
    }
}

impl TooltipController {
    /// Creates a hidden tooltip.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the tooltip until it is dismissed.
    pub fn show(&mut self) {
        self.popup_hovered = false;
        self.open = true;
        self.manual = true;
        self.suppressed = false;
        self.pending_show = None;
        self.pending_hide = None;
    }

    /// Hides the tooltip until the pointer or focus leaves and re-enters.
    pub fn dismiss(&mut self) {
        self.ignore_anchor_focus = self.popup_focused;
        self.popup_focused = false;
        self.touch = self.touch.filter(|touch| touch.triggered);
        self.open = false;
        self.manual = false;
        self.suppressed = true;
        self.pending_show = None;
        self.pending_hide = None;
    }

    /// Returns whether the tooltip is visible.
    pub fn is_open(&self) -> bool {
        self.open
    }

    fn engaged(&self) -> bool {
        self.anchor_hovered
            || self.anchor_focused
            || (self.open && (self.popup_hovered || self.popup_focused))
    }

    fn hover(&mut self, popup: bool, inside: bool, now: u64) {
        let hovered = if popup {
            &mut self.popup_hovered
        } else {
            &mut self.anchor_hovered
        };
        if *hovered == inside {
            return;
        }
        *hovered = inside;
        if popup && inside && !self.open {
            return;
        }
        if inside {
            if !popup && !self.open {
                self.popup_hovered = false;
            }
            self.suppressed = false;
            self.pending_hide = None;
            if !self.open {
                self.pending_show = Some(now.saturating_add(SHOW_DELAY));
            }
        } else if !self.engaged() && !self.manual {
            self.pending_show = None;
            self.pending_hide = self.open.then_some(now.saturating_add(EXIT_DELAY));
        }
    }

    fn focus(&mut self, popup: bool, focused: bool, now: u64) {
        if !popup && !focused {
            self.touch_focus_suppressed = false;
            self.ignore_anchor_focus = false;
        }
        if !popup && focused && self.touch_focus_suppressed {
            return;
        }
        if focused && self.touch.is_some_and(|touch| !touch.triggered) {
            return;
        }
        let previous = if popup {
            &mut self.popup_focused
        } else {
            &mut self.anchor_focused
        };
        if *previous == focused {
            return;
        }
        *previous = focused;
        if popup && focused && !self.open {
            return;
        }
        if focused {
            if !popup && self.ignore_anchor_focus {
                self.ignore_anchor_focus = false;
                return;
            }
            self.suppressed = false;
            self.open = true;
            self.pending_show = None;
            self.pending_hide = None;
        } else if !self.engaged() && !self.manual {
            self.pending_hide = Some(now.saturating_add(EXIT_DELAY));
        }
    }

    fn needs_frame(&self) -> bool {
        self.pending_show.is_some()
            || self.pending_hide.is_some()
            || self.touch.is_some_and(|touch| !touch.triggered)
    }

    fn advance(&mut self, now: u64) {
        if self.pending_show.is_some_and(|deadline| now >= deadline) {
            self.pending_show = None;
            if !self.suppressed && self.engaged() {
                self.open = true;
            }
        }
        if self
            .touch
            .is_some_and(|touch| !touch.triggered && now.saturating_sub(touch.since) >= SHOW_DELAY)
        {
            if let Some(touch) = &mut self.touch {
                touch.triggered = true;
            }
            self.open = true;
            self.pending_show = None;
            self.pending_hide = None;
        }
        if self.pending_hide.is_some_and(|deadline| now >= deadline) {
            self.pending_hide = None;
            if !self.engaged() && !self.manual {
                self.open = false;
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct TooltipContext {
    controller: State<TooltipController>,
    active: State<Option<State<TooltipController>>>,
}

#[derive(Clone, Copy)]
struct TooltipPointer {
    controller: State<TooltipController>,
    popup: bool,
    persistent: bool,
}

fn within(position: Option<PxPosition>, rect: PxRect) -> bool {
    position.is_some_and(|position| {
        position.x >= rect.x.0 as f32
            && position.y >= rect.y.0 as f32
            && position.x < (rect.x + rect.width).0 as f32
            && position.y < (rect.y + rect.height).0 as f32
    })
}

impl PointerInputModifierNode for TooltipPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let now = current_frame_nanos();
        let rect = PxRect::new(
            Px::ZERO,
            Px::ZERO,
            input.computed_data.width,
            input.computed_data.height,
        );
        let inside = within(input.cursor_position_rel, rect);
        let mouse = self.controller.with(|controller| controller.mouse_present);
        let changed = self.controller.with(|controller| {
            (if self.popup {
                controller.popup_hovered
            } else {
                controller.anchor_hovered
            }) != inside
        });
        if (mouse || input.cursor_position_rel.is_none()) && changed {
            self.controller
                .with_mut(|controller| controller.hover(self.popup, inside, now));
        }
        if self.popup {
            return;
        }
        for event in input
            .pointer_changes
            .iter_mut()
            .filter(|event| event.pointer_id != MOUSE_POINTER_ID)
        {
            let active = self.controller.with(|controller| controller.touch);
            match event.content {
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    if inside && !event.is_consumed() =>
                {
                    if let Some(origin) = input.cursor_position_rel {
                        self.controller.with_mut(|controller| {
                            controller.touch_focus_suppressed = true;
                            controller.anchor_focused = false;
                            controller.popup_focused = false;
                            controller.anchor_hovered = false;
                            controller.popup_hovered = false;
                            controller.pending_show = None;
                            controller.pending_hide = None;
                            controller.open = controller.manual;
                            controller.suppressed = false;
                            controller.touch = Some(TouchPress {
                                pointer: event.pointer_id,
                                origin,
                                since: now,
                                triggered: false,
                            });
                        });
                    }
                }
                CursorEventContent::Moved(_) | CursorEventContent::Scroll(_)
                    if active.is_some_and(|touch| {
                        touch.pointer == event.pointer_id && !touch.triggered
                    }) =>
                {
                    let touch = active.expect("active touch was checked");
                    let moved = input.cursor_position_rel.is_some_and(|position| {
                        (position.x - touch.origin.x).hypot(position.y - touch.origin.y)
                            > Dp(8.0).to_pixels_f32()
                    });
                    if moved || !inside || event.gesture_state == GestureState::Dragged {
                        self.controller
                            .with_mut(|controller| controller.touch = None);
                    }
                }
                CursorEventContent::Released(_) | CursorEventContent::Cancelled(_)
                    if active.is_some_and(|touch| touch.pointer == event.pointer_id) =>
                {
                    let triggered = active.is_some_and(|touch| touch.triggered);
                    let cancelled = matches!(event.content, CursorEventContent::Cancelled(_));
                    if triggered && !cancelled {
                        // Descendant recognizers need cancellation to clear
                        // their active press.
                        event.content = CursorEventContent::Cancelled(PressKeyEventType::Left);
                        event.gesture_state = GestureState::Dragged;
                    }
                    self.controller.with_mut(|controller| {
                        controller.touch = None;
                        if cancelled {
                            controller.dismiss();
                        } else if triggered {
                            controller.manual = self.persistent;
                            controller.pending_hide =
                                (!self.persistent).then_some(now.saturating_add(TOUCH_DURATION));
                        }
                    });
                }
                _ => {}
            }
        }
    }
}

struct TooltipDismissInput {
    controller: State<TooltipController>,
    popup: Arc<RwLock<PxRect>>,
}

impl TooltipDismissInput {
    fn handle_tab(&self, state: ElementState, previous: bool) -> bool {
        let (open, interactive, group, anchor) = self.controller.with(|controller| {
            (
                controller.open,
                controller.interactive,
                controller.popup_group,
                controller.focus,
            )
        });
        if !open || !interactive {
            return false;
        }
        let in_popup = group.has_focus();
        if !in_popup && (!anchor.has_focus() || previous) {
            return false;
        }
        if state == ElementState::Released {
            return true;
        }
        if in_popup {
            if !group.move_focus(if previous {
                FocusDirection::Previous
            } else {
                FocusDirection::Next
            }) {
                if !previous {
                    self.controller.with_mut(|controller| {
                        controller.dismiss();
                        controller.ignore_anchor_focus = true;
                    });
                }
                anchor.request_focus();
            }
            true
        } else {
            group.move_focus(FocusDirection::Next)
        }
    }
}

impl KeyboardInputModifierNode for TooltipDismissInput {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        input.keyboard_events.retain(|event| {
            if event.logical_key == Key::Named(NamedKey::Tab) {
                return !self.handle_tab(event.state, input.key_modifiers.shift_key());
            }
            if event.logical_key != Key::Named(NamedKey::Escape)
                || !self.controller.with(|controller| {
                    controller.open
                        || controller.pending_show.is_some()
                        || controller.touch.is_some()
                })
            {
                return true;
            }
            if event.state == ElementState::Pressed {
                let (group, anchor) = self
                    .controller
                    .with(|controller| (controller.popup_group, controller.focus));
                let restore = group.has_focus();
                self.controller.with_mut(|controller| {
                    controller.dismiss();
                    if restore {
                        controller.ignore_anchor_focus = true;
                    }
                });
                if restore {
                    anchor.request_focus();
                }
            }
            false
        });
    }
}

impl PointerInputModifierNode for TooltipDismissInput {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if let Some(mouse) = input
            .pointer_changes
            .last()
            .map(|event| event.pointer_id == MOUSE_POINTER_ID)
            && self.controller.with(|controller| controller.mouse_present) != mouse
        {
            self.controller.with_mut(|controller| {
                controller.mouse_present = mouse;
                if mouse {
                    controller.touch_focus_suppressed = false;
                }
                if !mouse {
                    controller.hover(false, false, current_frame_nanos());
                    controller.hover(true, false, current_frame_nanos());
                }
            });
        }
        if !self.controller.with(TooltipController::is_open) {
            return;
        }
        let anchor = self.controller.with(|controller| controller.anchor);
        let pressed = input
            .pointer_changes
            .iter()
            .any(|event| matches!(event.content, CursorEventContent::Pressed(_)));
        if pressed
            && !within(input.cursor_position_abs(), anchor)
            && !within(input.cursor_position_abs(), *self.popup.read())
        {
            self.controller.with_mut(TooltipController::dismiss);
        }
    }
}

struct TooltipRestoreInput {
    controller: State<TooltipController>,
}

fn sync_tooltip_focus(controller: State<TooltipController>, popup: bool) {
    let (anchor, group) = controller.with(|controller| (controller.focus, controller.popup_group));
    let focused = if popup {
        group.has_focus()
    } else {
        anchor.has_focus()
    };
    let changed = controller.with(|controller| {
        let previous = if popup {
            controller.popup_focused
        } else {
            controller.anchor_focused
        };
        previous != focused
            || (!popup
                && !focused
                && (controller.touch_focus_suppressed || controller.ignore_anchor_focus))
    });
    if changed {
        controller.with_mut(|controller| controller.focus(popup, focused, current_frame_nanos()));
    }
}

impl TooltipRestoreInput {
    fn restore(&self) {
        let (open, group, anchor) = self
            .controller
            .with(|controller| (controller.open, controller.popup_group, controller.focus));
        if !open && group.has_focus() {
            self.controller
                .with_mut(|controller| controller.ignore_anchor_focus = true);
            anchor.request_focus();
        }
    }
}

impl KeyboardInputModifierNode for TooltipRestoreInput {
    fn on_keyboard_input(&self, _input: KeyboardInput<'_>) {
        self.restore();
    }
}

impl PointerInputModifierNode for TooltipRestoreInput {
    fn on_pointer_input(&self, _input: PointerInput<'_>) {
        self.restore();
    }
}

#[derive(Clone)]
struct TooltipLayout {
    open: bool,
    anchor: PxRect,
    origin: Arc<RwLock<PxRect>>,
    placement: TooltipPlacement,
}

impl PartialEq for TooltipLayout {
    fn eq(&self, other: &Self) -> bool {
        self.open == other.open && self.anchor == other.anchor && self.placement == other.placement
    }
}

fn popup_position(
    anchor: PxRect,
    size: ComputedData,
    available: ComputedData,
    placement: TooltipPlacement,
) -> PxPosition {
    let gap = Dp(4.0).to_px().0;
    let margin = Dp(8.0).to_px().0;
    let width = available.width.0.max(0);
    let height = available.height.0.max(0);
    let above = anchor.y.0 - gap - size.height.0;
    let below = anchor.y.0 + anchor.height.0 + gap;
    let start = anchor.x.0 - gap - size.width.0;
    let end = anchor.x.0 + anchor.width.0 + gap;
    let center_x = anchor.x.0 + (anchor.width.0 - size.width.0) / 2;
    let center_y = anchor.y.0 + (anchor.height.0 - size.height.0) / 2;
    let (x, y) = match placement {
        TooltipPlacement::Above => (center_x, if above >= margin { above } else { below }),
        TooltipPlacement::Below => (
            center_x,
            if below + size.height.0 <= height - margin {
                below
            } else {
                above
            },
        ),
        TooltipPlacement::Start => (if start >= margin { start } else { end }, center_y),
        TooltipPlacement::End => (
            if end + size.width.0 <= width - margin {
                end
            } else {
                start
            },
            center_y,
        ),
    };
    PxPosition::new(
        Px(x.clamp(
            margin.min(width),
            (width - size.width.0 - margin).max(margin.min(width)),
        )),
        Px(y.clamp(
            margin.min(height),
            (height - size.height.0 - margin).max(margin.min(height)),
        )),
    )
}

impl tessera_ui::LayoutModifierNode for TooltipLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let mut result = LayoutResult::default();
        let constraint = input.parent_constraint().without_min();
        let main = children[0].measure(input, &constraint)?;
        result.place_child(children[0], PxPosition::ZERO);
        if self.open {
            let constraint = tessera_ui::Constraint::new(
                AxisConstraint::new(
                    Px::ZERO,
                    Some((main.width - Dp(16.0).to_px()).max(Px::ZERO)),
                ),
                AxisConstraint::new(
                    Px::ZERO,
                    Some((main.height - Dp(16.0).to_px()).max(Px::ZERO)),
                ),
            );
            let popup = children[1].measure(input, &constraint)?;
            let origin = *self.origin.read();
            let anchor = PxRect::new(
                self.anchor.x - origin.x,
                self.anchor.y - origin.y,
                self.anchor.width,
                self.anchor.height,
            );
            let position = popup_position(anchor, popup.size(), main.size(), self.placement);
            result.place_child(children[1], position);
        }
        Ok(result.with_size(main.size()))
    }
}

/// # tooltip_provider
///
/// Host a contextual explanation above page content for a control or action.
///
/// ## Usage
///
/// Wrap a page and place its trigger inside `tooltip_anchor`.
///
/// ## Parameters
///
/// - `message` — supporting text shown in the popup
/// - `variant`, `headline` — plain or rich appearance and optional rich title
/// - `action_label`, `on_action` — optional rich action, which dismisses the
///   popup
/// - `actions` — custom rich actions, replacing the default text action
/// - `controller` — optional shared visibility state for the provider and
///   anchor
/// - `placement` — preferred side, with edge avoidance and opposite-side
///   fallback
/// - `persistent` — keep a touch-triggered popup open until dismissal
/// - `modifier` — modifiers for the page-sized host
/// - `child` — page content, containing one anchor for this provider
///
/// ## Examples
///
/// ```
/// use tessera_components::tooltip::TooltipController;
/// let mut tooltip = TooltipController::new();
/// assert!(!tooltip.is_open());
/// tooltip.show();
/// assert!(tooltip.is_open());
/// tooltip.dismiss();
/// assert!(!tooltip.is_open());
/// ```
#[tessera]
pub fn tooltip_provider(
    #[prop(into)] message: String,
    variant: Option<TooltipVariant>,
    #[prop(into)] headline: Option<String>,
    #[prop(into)] action_label: Option<String>,
    on_action: Option<Callback>,
    actions: Option<RenderSlot>,
    controller: Option<State<TooltipController>>,
    placement: Option<TooltipPlacement>,
    persistent: Option<bool>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let active = if let Some(context) = use_context::<TooltipContext>() {
        context.get().active
    } else {
        remember(|| None::<State<TooltipController>>)
    };
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(TooltipController::new)
    };
    let variant = variant.unwrap_or_default();
    let persistent = persistent.unwrap_or(
        variant == TooltipVariant::Rich && (action_label.is_some() || actions.is_some()),
    );
    let interactive =
        variant == TooltipVariant::Rich && (action_label.is_some() || actions.is_some());
    if controller.with(|controller| {
        controller.description != message
            || controller.persistent != persistent
            || controller.interactive != interactive
    }) {
        controller.with_mut(|controller| {
            controller.description = message.clone();
            controller.persistent = persistent;
            controller.interactive = interactive;
        });
    }
    let child = child.unwrap_or_else(RenderSlot::empty);
    let origin = remember(|| Arc::new(RwLock::new(PxRect::ZERO))).get();
    let popup = remember(|| Arc::new(RwLock::new(PxRect::ZERO))).get();
    if controller.with(TooltipController::needs_frame) {
        receive_frame_nanos(move |now| {
            controller.with_mut(|controller| controller.advance(now));
            if controller.with(TooltipController::needs_frame) {
                FrameNanosControl::Continue
            } else {
                FrameNanosControl::Stop
            }
        });
    }
    let (open, anchor) = controller.with(|controller| (controller.is_open(), controller.anchor));
    if open && active.get() != Some(controller) {
        if let Some(previous) = active.get()
            && previous.is_alive()
        {
            previous.with_mut(TooltipController::dismiss);
        }
        active.set(Some(controller));
    } else if !open && active.get() == Some(controller) {
        active.set(None);
    }
    if !open
        && controller
            .with(|controller| controller.popup_group)
            .has_focus()
    {
        controller.with_mut(|controller| controller.ignore_anchor_focus = true);
        controller
            .with(|controller| controller.focus)
            .request_focus();
    }
    let next = (
        open,
        message,
        headline,
        action_label,
        on_action,
        child,
        variant,
        persistent,
        actions,
    );
    let render_data = remember(|| next.clone());
    if render_data.with(|current| current != &next) {
        render_data.set(next);
    }
    let mut modifier = modifier.unwrap_or_default();
    modifier = modifier
        .push_keyboard_preview_input(TooltipDismissInput {
            controller,
            popup: Arc::clone(&popup),
        })
        .push_pointer_preview_input(TooltipDismissInput {
            controller,
            popup: Arc::clone(&popup),
        })
        .push_keyboard_input(TooltipRestoreInput { controller })
        .push_pointer_final_input(TooltipRestoreInput { controller });
    let bounds = Arc::clone(&origin);
    modifier = modifier.on_globally_positioned(move |rect| *bounds.write() = rect);
    provide_context(
        || TooltipContext { controller, active },
        || {
            layout()
                .modifier(modifier)
                .policy(TooltipLayout {
                    open,
                    anchor,
                    origin,
                    placement: placement.unwrap_or_default(),
                })
                .child(move || {
                    let (
                        open,
                        message,
                        headline,
                        action_label,
                        on_action,
                        child,
                        variant,
                        persistent,
                        actions,
                    ) = render_data.get();
                    child.render();
                    if open {
                        let has_actions = actions.is_some();
                        tooltip_panel(message.clone())
                            .variant(variant)
                            .headline(headline.clone().unwrap_or_default())
                            .action_label(action_label.clone().unwrap_or_default())
                            .on_action_shared(on_action.unwrap_or_default())
                            .controller(controller)
                            .popup(Arc::clone(&popup))
                            .persistent(persistent)
                            .actions_shared(actions.unwrap_or_else(RenderSlot::empty))
                            .has_actions(has_actions);
                    }
                });
        },
    );
}

/// # tooltip_anchor
///
/// Attach hover, focus, and touch explanations to a control without changing
/// its action.
///
/// ## Usage
///
/// Wrap a button or icon in the page content of `tooltip_provider`.
///
/// ## Parameters
///
/// - `controller` — optional shared controller; otherwise use the nearest
///   provider
/// - `modifier` — modifiers for the trigger wrapper
/// - `child` — the control that receives focus and normal activation
///
/// ## Examples
///
/// ```
/// use tessera_components::tooltip::TooltipPlacement;
/// assert_eq!(TooltipPlacement::default(), TooltipPlacement::Above);
/// ```
#[tessera]
pub fn tooltip_anchor(
    controller: Option<State<TooltipController>>,
    modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let context = use_context::<TooltipContext>()
        .expect("tooltip_anchor requires tooltip_provider")
        .get();
    let controller = controller.unwrap_or(context.controller);
    let child = child.unwrap_or_else(RenderSlot::empty);
    let focus = controller.with(|controller| controller.focus);
    let modifier = modifier
        .unwrap_or_default()
        .semantics(SemanticsArgs {
            role: Some(Role::Group),
            merge_descendants: true,
            tooltip: Some(controller.with(|controller| controller.description.clone())),
            ..Default::default()
        })
        .focus_group()
        .focus_requester(focus)
        .on_focus_changed(move |_: FocusState| sync_tooltip_focus(controller, false))
        .on_globally_positioned(move |rect| {
            if controller.with(|controller| controller.anchor) != rect {
                controller.with_mut(|controller| controller.anchor = rect);
            }
        })
        .push_pointer_preview_input(TooltipPointer {
            controller,
            popup: false,
            persistent: controller.with(|controller| controller.persistent),
        });
    layout().modifier(modifier).child(move || child.render());
}

#[tessera]
fn tooltip_panel(
    #[prop(into)] message: String,
    variant: Option<TooltipVariant>,
    headline: Option<String>,
    action_label: Option<String>,
    on_action: Option<Callback>,
    actions: Option<RenderSlot>,
    has_actions: Option<bool>,
    controller: Option<State<TooltipController>>,
    popup: Option<Arc<RwLock<PxRect>>>,
    persistent: Option<bool>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let variant = variant.unwrap_or_default();
    let rich = variant == TooltipVariant::Rich;
    let has_actions = has_actions.unwrap_or(false);
    let actions = actions.unwrap_or_else(RenderSlot::empty);
    let headline = headline.filter(|text| !text.is_empty());
    let action_label = action_label.filter(|text| !text.is_empty());
    let controller = controller.expect("tooltip panel requires controller");
    let popup = popup.expect("tooltip panel requires bounds");
    let max_width = if rich { Dp(320.0) } else { Dp(200.0) };
    let next = (
        theme.clone(),
        message.clone(),
        headline.clone(),
        action_label.clone(),
        rich,
        actions,
        has_actions,
    );
    let render_data = remember(|| next.clone());
    if render_data.with(|current| current != &next) {
        render_data.set(next);
    }
    let modifier = Modifier::new()
        .constrain(
            Some(AxisConstraint::new(
                Dp(40.0).to_px(),
                Some(max_width.to_px()),
            )),
            Some(AxisConstraint::new(Dp(24.0).to_px(), None)),
        )
        .focus_group_with(controller.with(|controller| controller.popup_group))
        .focus_requester(controller.with(|controller| controller.popup_focus))
        .on_focus_changed(move |_: FocusState| sync_tooltip_focus(controller, true))
        .push_keyboard_input(TooltipRestoreInput { controller })
        .push_pointer_final_input(TooltipRestoreInput { controller })
        .on_globally_positioned(move |rect| *popup.write() = rect)
        .push_pointer_preview_input(TooltipPointer {
            controller,
            popup: true,
            persistent: persistent.unwrap_or(false),
        })
        .semantics(SemanticsArgs {
            role: Some(if rich && (action_label.is_some() || has_actions) {
                Role::Dialog
            } else {
                Role::Tooltip
            }),
            label: Some(if rich {
                headline.clone().unwrap_or_else(|| message.clone())
            } else {
                message.clone()
            }),
            live: Some(Live::Polite),
            description: (rich && headline.is_some()).then(|| message.clone()),
            merge_descendants: rich && (action_label.is_some() || has_actions),
            ..Default::default()
        });
    surface()
        .modifier(modifier)
        .shape(Shape::rounded_rectangle(if rich {
            Dp(12.0)
        } else {
            Dp(4.0)
        }))
        .style(
            if rich {
                theme.color_scheme.surface_container
            } else {
                theme.color_scheme.inverse_surface
            }
            .into(),
        )
        .content_color(if rich {
            theme.color_scheme.on_surface_variant
        } else {
            theme.color_scheme.inverse_on_surface
        })
        .elevation(if rich { Dp(3.0) } else { Dp(0.0) })
        .child(move || {
            let (rich, action_row) = render_data.with(|(_, _, _, label, rich, _, has_actions)| {
                (*rich, label.is_some() || *has_actions)
            });
            column()
                .modifier(Modifier::new().padding(if rich {
                    Padding::new(
                        Dp(16.0),
                        Dp(12.0),
                        Dp(16.0),
                        if action_row { Dp(8.0) } else { Dp(16.0) },
                    )
                } else {
                    Padding::symmetric(Dp(8.0), Dp(4.0))
                }))
                .children(move || {
                    let (theme, message, headline, action_label, rich, actions, has_actions) =
                        render_data.get();
                    if rich && let Some(headline) = &headline {
                        text()
                            .content(headline.clone())
                            .style(theme.typography.title_small);
                        crate::spacer::spacer().modifier(Modifier::new().height(Dp(4.0)));
                    }
                    text().content(message.clone()).style(if rich {
                        theme.typography.body_medium
                    } else {
                        theme.typography.body_small
                    });
                    if rich && has_actions {
                        crate::spacer::spacer().modifier(Modifier::new().height(Dp(8.0)));
                        actions.render();
                    } else if rich && action_label.is_some() {
                        crate::spacer::spacer().modifier(Modifier::new().height(Dp(8.0)));
                        button()
                            .text()
                            .accessibility_label(action_label.clone().unwrap_or_default())
                            .on_click(move || {
                                controller.with_mut(TooltipController::dismiss);
                                if let Some(on_action) = on_action {
                                    on_action.call();
                                }
                            })
                            .child(move || {
                                let label = render_data.with(|(_, _, _, label, _, _, _)| {
                                    label.clone().unwrap_or_default()
                                });
                                text().content(label).style(render_data.with(
                                    |(theme, _, _, _, _, _, _)| theme.typography.label_large,
                                ));
                            });
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::headless::{HeadlessConfig, HeadlessRenderer};

    use crate::{boxed::boxed, theme::material_theme};

    use super::*;

    #[test]
    fn hover_delay_gap_crossing_escape_suppression_and_idle_clock() {
        let mut controller = TooltipController::new();
        controller.hover(false, true, 0);
        controller.advance(SHOW_DELAY - 1);
        assert!(!controller.is_open());
        controller.advance(SHOW_DELAY);
        assert!(controller.is_open());
        assert!(!controller.needs_frame());
        controller.hover(false, false, SHOW_DELAY);
        controller.hover(true, true, SHOW_DELAY + EXIT_DELAY / 2);
        controller.advance(SHOW_DELAY + EXIT_DELAY * 2);
        assert!(controller.is_open());
        controller.dismiss();
        controller.hover(true, true, SHOW_DELAY * 2);
        controller.advance(SHOW_DELAY * 3);
        assert!(!controller.is_open());
        assert!(!controller.needs_frame());
        controller.hover(true, false, SHOW_DELAY * 3);
        controller.hover(false, true, SHOW_DELAY * 4);
        controller.advance(SHOW_DELAY * 5);
        assert!(controller.is_open());
    }

    #[test]
    fn focus_keeps_hovered_content_and_escape_restoration_does_not_reopen_it() {
        let mut controller = TooltipController::new();
        controller.focus(false, true, 0);
        assert!(controller.is_open());
        controller.focus(false, false, 1);
        controller.focus(true, true, 2);
        controller.advance(EXIT_DELAY * 2);
        assert!(controller.is_open());
        controller.dismiss();
        controller.focus(true, false, 3);
        controller.focus(false, true, 4);
        controller.advance(EXIT_DELAY * 3);
        assert!(!controller.is_open());
        controller.focus(false, false, EXIT_DELAY * 4);
        controller.focus(false, true, EXIT_DELAY * 5);
        assert!(controller.is_open());
        controller.dismiss();
        controller.ignore_anchor_focus = true;
        controller.focus(false, false, EXIT_DELAY * 6);
        controller.focus(false, true, EXIT_DELAY * 7);
        assert!(controller.is_open());
    }

    #[test]
    fn touch_waits_for_threshold_and_manual_visibility_does_not_run_a_clock() {
        let mut controller = TooltipController::new();
        controller.touch = Some(TouchPress {
            pointer: 1,
            origin: PxPosition::ZERO,
            since: 0,
            triggered: false,
        });
        controller.focus(false, true, 0);
        controller.advance(SHOW_DELAY - 1);
        assert!(!controller.is_open());
        controller.advance(SHOW_DELAY);
        assert!(controller.is_open());
        assert!(!controller.needs_frame());
        controller.show();
        controller.advance(u64::MAX);
        assert!(controller.is_open());
        assert!(!controller.needs_frame());
    }

    #[test]
    fn stale_popup_input_cannot_reopen_a_dismissed_tooltip() {
        let mut controller = TooltipController::new();
        controller.show();
        controller.dismiss();
        controller.focus(true, true, 1);
        controller.hover(true, true, 2);
        controller.advance(SHOW_DELAY * 2);
        assert!(!controller.is_open());
        assert!(!controller.needs_frame());
        controller.focus(true, false, SHOW_DELAY * 3);
        controller.focus(false, true, SHOW_DELAY * 4);
        assert!(controller.is_open());
    }

    #[test]
    fn touch_focus_suppression_clears_when_actual_focus_leaves_the_anchor() {
        use tessera_ui::testing::{
            layout_test, pump_layout_test_session, start_layout_test_session,
        };

        let mut session = start_layout_test_session(
            layout_test(|| {
                fixture();
            })
            .viewport_px(400, 320),
        );
        pump_layout_test_session(&mut session, 0);
        let controller = CONTROLLER.get().expect("controller");
        let anchor = controller.with(|controller| controller.focus);
        controller.with_mut(|controller| {
            controller.touch_focus_suppressed = true;
            controller.anchor_focused = false;
        });
        anchor.request_focus();
        assert!(anchor.has_focus());
        assert!(!controller.with(|controller| controller.anchor_focused));
        anchor.clear_focus();
        assert!(!controller.with(|controller| controller.touch_focus_suppressed));
        anchor.request_focus();
        assert!(controller.with(TooltipController::is_open));
    }

    #[test]
    fn placement_flips_at_each_edge_and_clamps_to_window_margins() {
        let available = ComputedData {
            width: Px(400),
            height: Px(300),
        };
        let popup = ComputedData {
            width: Px(120),
            height: Px(40),
        };
        let cases = [
            (
                TooltipPlacement::Above,
                PxRect::new(Px(150), Px(2), Px(40), Px(40)),
                PxPosition::new(Px(110), Px(46)),
            ),
            (
                TooltipPlacement::Below,
                PxRect::new(Px(150), Px(260), Px(40), Px(40)),
                PxPosition::new(Px(110), Px(216)),
            ),
            (
                TooltipPlacement::Start,
                PxRect::new(Px(2), Px(100), Px(40), Px(40)),
                PxPosition::new(Px(46), Px(100)),
            ),
            (
                TooltipPlacement::End,
                PxRect::new(Px(360), Px(100), Px(40), Px(40)),
                PxPosition::new(Px(236), Px(100)),
            ),
        ];
        for (placement, anchor, expected) in cases {
            assert_eq!(
                popup_position(anchor, popup, available, placement),
                expected
            );
        }
        let left = popup_position(
            PxRect::new(Px(0), Px(100), Px(40), Px(40)),
            popup,
            available,
            TooltipPlacement::Above,
        );
        assert_eq!(left.x, 8.0);
    }

    #[derive(Clone, PartialEq, Default)]
    struct Settings {
        rich: bool,
        changed: bool,
        focus_stress: bool,
        custom_actions: bool,
    }

    thread_local! {
        static CONTROLLER: Cell<Option<State<TooltipController>>> = const { Cell::new(None) };
        static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
        static CLICKS: Cell<Option<State<u32>>> = const { Cell::new(None) };
        static ACTIONS: Cell<Option<State<u32>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture() {
        let controller = remember(TooltipController::new);
        let settings = remember(Settings::default);
        let clicks = remember(|| 0_u32);
        let actions = remember(|| 0_u32);
        let other_tooltip = remember(TooltipController::new);
        // Rebuild the parent boundary when an action also updates sibling
        // content.
        let _ = actions.get();
        CONTROLLER.set(Some(controller));
        SETTINGS.set(Some(settings));
        CLICKS.set(Some(clicks));
        ACTIONS.set(Some(actions));
        material_theme().child(move || {
            let config = settings.get();
            let tooltip = tooltip_provider(if config.changed { "Updated help text with multiple words that wraps within the tooltip's maximum width." } else { "Helpful description" })
                .controller(controller)
                .variant(if config.rich { TooltipVariant::Rich } else { TooltipVariant::Plain })
                .headline("More information")
                .action_label(if config.rich { "Got it" } else { "" })
                .on_action(move || actions.set(actions.get() + 1));
            let tooltip = if config.custom_actions {
                tooltip.actions(move || {
                    crate::row::row().children(move || {
                        button().text().accessibility_label("Not now").on_click(move || controller.with_mut(TooltipController::dismiss)).child(|| { text().content("Not now"); });
                        button().text().accessibility_label("Got it").on_click(move || {
                            controller.with_mut(TooltipController::dismiss);
                            actions.set(actions.get() + 1);
                        }).child(|| { text().content("Got it"); });
                    });
                })
            } else {
                tooltip
            };
            tooltip.child(move || {
                tooltip_provider("Other help").controller(other_tooltip).child(move || {
                    boxed().modifier(Modifier::new().fill_max_size().padding_all(Dp(80.0))).children(move || {
                        tooltip_anchor().controller(controller).child(move || {
                            button().modifier(Modifier::new().width(Dp(120.0))).accessibility_label("Trigger").on_click(move || {
                                clicks.set(clicks.get() + 1);
                                if settings.with(|settings| settings.focus_stress) {
                                    let (anchor, group) = controller.with(|controller| (controller.focus, controller.popup_group));
                                    assert!(group.move_focus(FocusDirection::Next));
                                    anchor.request_focus();
                                    assert!(group.move_focus(FocusDirection::Next));
                                    controller.with_mut(TooltipController::dismiss);
                                    TooltipRestoreInput { controller }.restore();
                                }
                            }).child(|| { text().content("Trigger"); });
                        });
                    });
                });
            });
        });
    }

    fn renderer() -> HeadlessRenderer {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 320,
                ..Default::default()
            },
        )
        .expect("headless renderer");
        renderer.render_frames(3);
        renderer
    }

    fn anchor_center() -> PxPosition {
        let rect = CONTROLLER
            .get()
            .expect("controller")
            .with(|controller| controller.anchor);
        PxPosition::new(rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_hover_restores_pixels_preserves_focus_and_updates_content_and_variant() {
        check_tooltip_interactions(false);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_custom_actions_restore_focus_and_pixels_after_parent_rebuild() {
        check_tooltip_interactions(true);
    }

    fn check_tooltip_interactions(custom_actions: bool) {
        let mut renderer = renderer();
        SETTINGS
            .get()
            .expect("settings")
            .with_mut(|settings| settings.custom_actions = custom_actions);
        renderer.render_frames(2);
        let before = renderer.read_pixels().expect("pixels");
        let focus = renderer.accessibility_snapshot().expect("tree").focus;
        renderer.pointer_move(MOUSE_POINTER_ID, anchor_center());
        renderer.render_frames(35);
        assert!(
            CONTROLLER
                .get()
                .expect("controller")
                .with(TooltipController::is_open),
            "{:?}",
            CONTROLLER.get().expect("controller").get()
        );
        let tree = renderer.accessibility_snapshot().expect("tree");
        assert_eq!(tree.focus, focus);
        let tooltip = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Tooltip)
            .expect("tooltip");
        assert_eq!(tooltip.1.label(), Some("Helpful description"));
        let bounds = tooltip.1.bounds().expect("bounds");
        assert!(bounds.x1 - bounds.x0 <= 200.0);
        assert!(bounds.y1 - bounds.y0 >= 24.0);
        let trigger = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Trigger") && node.role() == Role::Button)
            .expect("trigger");
        assert_eq!(trigger.1.description(), Some("Helpful description"));
        SETTINGS
            .get()
            .expect("settings")
            .with_mut(|settings| settings.changed = true);
        renderer.render_frames(3);
        assert!(
            renderer
                .accessibility_snapshot()
                .expect("tree")
                .nodes
                .iter()
                .any(|(_, node)| node
                    .label()
                    .is_some_and(|label| label.starts_with("Updated help text")))
        );
        SETTINGS.get().expect("settings").with_mut(|settings| {
            settings.rich = true;
            settings.changed = false;
        });
        renderer.render_frames(3);
        let tree = renderer.accessibility_snapshot().expect("tree");
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Dialog)
        );
        let controller = CONTROLLER.get().expect("controller");
        let trigger = controller.with(|controller| controller.focus);
        trigger.request_focus();
        renderer.render_frames(2);
        let navigation = TooltipDismissInput {
            controller,
            popup: Arc::new(RwLock::new(PxRect::ZERO)),
        };
        for _ in 0..2 {
            assert!(navigation.handle_tab(ElementState::Pressed, false));
            if custom_actions {
                assert!(navigation.handle_tab(ElementState::Pressed, false));
            }
            renderer.render_frames(2);
            let focused = renderer.accessibility_snapshot().expect("tree");
            assert_eq!(
                focused
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == focused.focus)
                    .expect("focused action")
                    .1
                    .label(),
                Some("Got it")
            );
            assert!(navigation.handle_tab(ElementState::Pressed, true));
            if custom_actions {
                assert!(navigation.handle_tab(ElementState::Pressed, true));
            }
            renderer.render_frames(2);
            let focused = renderer.accessibility_snapshot().expect("tree");
            assert_eq!(
                focused
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == focused.focus)
                    .expect("focused trigger")
                    .1
                    .label(),
                Some("Trigger")
            );
        }
        assert!(navigation.handle_tab(ElementState::Pressed, false));
        controller.with_mut(TooltipController::dismiss);
        TooltipRestoreInput { controller }.restore();
        assert!(trigger.has_focus());
        assert!(
            !controller
                .with(|controller| controller.popup_group)
                .has_focus()
        );
        renderer.render_frames(2);
        assert!(!controller.with(TooltipController::is_open));
        controller.with_mut(TooltipController::show);
        SETTINGS
            .get()
            .expect("settings")
            .with_mut(|settings| settings.focus_stress = true);
        renderer.render_frames(2);
        renderer.pointer_press(MOUSE_POINTER_ID, anchor_center(), PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_release(MOUSE_POINTER_ID, anchor_center(), PressKeyEventType::Left);
        renderer.render_frames(3);
        assert!(!controller.with(TooltipController::is_open));
        assert!(trigger.has_focus());
        SETTINGS
            .get()
            .expect("settings")
            .with_mut(|settings| settings.focus_stress = false);
        controller.with_mut(TooltipController::show);
        renderer.render_frames(2);
        let action = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Got it"))
            .expect("rich action")
            .1
            .bounds()
            .expect("bounds");
        let position = PxPosition::from_f32_xy(
            ((action.x0 + action.x1) / 2.0) as f32,
            ((action.y0 + action.y1) / 2.0) as f32,
        );
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(3);
        assert_eq!(ACTIONS.get().expect("actions").get(), 1);
        assert!(
            !CONTROLLER
                .get()
                .expect("controller")
                .with(TooltipController::is_open)
        );
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::new(Px(390), Px(310)));
        let focus = CONTROLLER
            .get()
            .expect("controller")
            .with(|controller| controller.focus);
        focus.clear_focus();
        assert!(
            !focus.has_focus(),
            "clear must remove focus from the trigger subtree"
        );
        SETTINGS.get().expect("settings").set(Settings {
            custom_actions,
            ..Settings::default()
        });
        renderer.render_frames(40);
        let (open, anchor_focused, popup_focused, hovered, popup_hovered, manual, ignore) =
            controller.with(|controller| {
                (
                    controller.open,
                    controller.anchor_focused,
                    controller.popup_focused,
                    controller.anchor_hovered,
                    controller.popup_hovered,
                    controller.manual,
                    controller.ignore_anchor_focus,
                )
            });
        assert!(
            !open,
            "tooltip reopened: anchor={anchor_focused}, popup={popup_focused}, hover={hovered}, popup_hover={popup_hovered}, manual={manual}, ignore={ignore}, actual_anchor={}, actual_popup={}",
            focus.has_focus(),
            controller
                .with(|controller| controller.popup_group)
                .has_focus()
        );
        let restored = renderer.read_pixels().expect("restored pixels");
        let changed = before
            .chunks_exact(4)
            .zip(restored.chunks_exact(4))
            .filter(|(before, after)| before != after)
            .count();
        assert!(
            restored == before,
            "dismissed tooltip must restore the original pixels after interaction animations settle ({changed} pixels changed)"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_long_press_consumes_release_and_scrolling_cancels_pending_help() {
        let mut renderer = renderer();
        let position = anchor_center();
        renderer.pointer_press(1, position, PressKeyEventType::Left);
        renderer.render_frames(35);
        assert!(
            CONTROLLER
                .get()
                .expect("controller")
                .with(TooltipController::is_open)
        );
        renderer.pointer_release(1, position, PressKeyEventType::Left);
        renderer.render_frames(3);
        assert_eq!(CLICKS.get().expect("clicks").get(), 0);
        renderer.render_frames(100);
        assert!(
            !CONTROLLER
                .get()
                .expect("controller")
                .with(TooltipController::is_open)
        );
        assert!(
            !CONTROLLER
                .get()
                .expect("controller")
                .with(TooltipController::needs_frame)
        );
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.step_frame();
        assert_eq!(CLICKS.get().expect("clicks").get(), 1);
        let controller = CONTROLLER.get().expect("controller");
        let focus = controller.with(|controller| controller.focus);
        focus.clear_focus();
        controller.with_mut(TooltipController::dismiss);
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::new(Px(390), Px(310)));
        renderer.render_frames(8);
        renderer.pointer_press(2, position, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_move(2, PxPosition::from_f32_xy(position.x + 20.0, position.y));
        renderer.step_frame();
        assert!(
            controller.with(|controller| controller.touch.is_none()),
            "after movement: {:?}",
            controller.get()
        );
        renderer.render_frames(35);
        assert!(
            !controller.with(TooltipController::is_open),
            "after cancellation: {:?}",
            controller.get()
        );
        renderer.pointer_release(2, position, PressKeyEventType::Left);
        renderer.step_frame();
    }
}
