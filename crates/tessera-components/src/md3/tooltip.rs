//! Anchored plain and rich contextual tooltips.
//!
//! ## Usage
//!
//! Explain controls on hover, keyboard focus, or touch long press.

use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    Callback, ComputedData, Constraint, Dp, FloatRect, FocusDirection, FocusGroupNode,
    FocusRequester, FocusState, KeyboardInput, KeyboardInputModifierNode, LayoutPolicy,
    LayoutResult, MOUSE_POINTER_ID, MeasurementError, Modifier, PointerInput,
    PointerInputModifierNode, PressKeyEventType, Px, PxPosition, PxRect, PxSize, RenderSlot, State,
    accesskit::{Live, Role},
    launch_once,
    layout::{MeasureScope, Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    popup::{PopupPositionInput, PopupPositionPolicy, popup},
    remember, tessera,
    time::Instant,
    use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use crate::{
    animation::{Delay, run_frames},
    column::column,
    md3::{button::button, surface::surface, text::text, theme::MaterialTheme},
    shape_def::Shape,
};

const SHOW_DELAY: Duration = Duration::from_millis(500);
const EXIT_DELAY: Duration = Duration::from_millis(100);
const TOUCH_DURATION: Duration = Duration::from_millis(1500);

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
    triggered: bool,
}

/// Shared tooltip visibility and interaction state.
#[derive(Clone, PartialEq)]
pub struct TooltipController {
    open: bool,
    suppressed: bool,
    anchor_hovered: bool,
    popup_hovered: bool,
    anchor_focused: bool,
    popup_focused: bool,
    manual: bool,
    ignore_anchor_focus: bool,
    persistent: bool,
    interactive: bool,
    mouse_present: bool,
    touch_focus_suppressed: bool,
    pending_show: Option<Delay>,
    pending_hide: Option<Delay>,
    touch: Option<TouchPress>,
    focus: FocusRequester,
    popup_focus: FocusRequester,
    popup_group: FocusGroupNode,
}

impl std::fmt::Debug for TooltipController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TooltipController")
            .field("open", &self.open)
            .field("suppressed", &self.suppressed)
            .field("persistent", &self.persistent)
            .field("interactive", &self.interactive)
            .finish_non_exhaustive()
    }
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
            persistent: false,
            interactive: false,
            mouse_present: false,
            touch_focus_suppressed: false,
            pending_show: None,
            pending_hide: None,
            touch: None,
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

    fn hover(&mut self, popup: bool, inside: bool, at: Option<Instant>) {
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
                self.pending_show = Some(Delay::new(at, SHOW_DELAY));
            }
        } else if !self.engaged() && !self.manual {
            self.pending_show = None;
            self.pending_hide = self.open.then_some(Delay::new(at, EXIT_DELAY));
        }
    }

    fn focus(&mut self, popup: bool, focused: bool) {
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
            self.pending_hide = Some(Delay::new(None, EXIT_DELAY));
        }
    }

    fn needs_frame(&self) -> bool {
        self.pending_show.is_some() || self.pending_hide.is_some()
    }

    fn advance(&mut self, now: Instant) -> bool {
        if self
            .pending_show
            .as_mut()
            .is_some_and(|delay| delay.elapsed(now))
        {
            self.pending_show = None;
            if !self.suppressed && self.engaged() {
                self.open = true;
            }
        }
        if self
            .pending_hide
            .as_mut()
            .is_some_and(|delay| delay.elapsed(now))
        {
            self.pending_hide = None;
            if !self.engaged() && !self.manual {
                self.open = false;
            }
        }
        self.needs_frame()
    }
}

#[derive(Clone, Copy)]
struct TooltipPointer {
    controller: State<TooltipController>,
    popup: bool,
}

fn within(position: Option<PxPosition>, rect: impl Into<tessera_ui::FloatRect>) -> bool {
    let rect = rect.into();
    position.is_some_and(|position| rect.contains(position))
}

impl PointerInputModifierNode for TooltipPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let rect = PxRect::new(
            Px::ZERO,
            Px::ZERO,
            input.computed_data.width,
            input.computed_data.height,
        );
        let inside = within(input.cursor_position_rel, rect);
        let at = input
            .pointer_changes
            .iter()
            .map(|change| change.timestamp)
            .max();

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
                .with_mut(|controller| controller.hover(self.popup, inside, at));
        }
    }
}

async fn tooltip_touch_detector(
    mut scope: tessera_ui::PointerInputScope,
    controller: State<TooltipController>,
) {
    struct TouchLease(State<TooltipController>);
    impl Drop for TouchLease {
        fn drop(&mut self) {
            if self.0.is_alive() {
                self.0.with_mut(|controller| controller.touch = None);
            }
        }
    }
    // Touch observes Initial so its long-press release preempts descendant clicks.
    while let Some(down) = scope
        .await_first_down(
            tessera_ui::PointerEventPass::Initial,
            Some(PressKeyEventType::Left),
            tessera_ui::DownRequirement::Unconsumed,
        )
        .await
    {
        if down.change.pointer_id == MOUSE_POINTER_ID {
            continue;
        }
        let _lease = TouchLease(controller);
        controller.with_mut(|controller| {
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
                pointer: down.change.pointer_id,
                triggered: false,
            });
        });
        if !matches!(
            scope
                .await_long_press_or_cancellation(
                    tessera_ui::PointerEventPass::Initial,
                    &down,
                    SHOW_DELAY,
                    Dp(8.0).to_pixels_f32()
                )
                .await,
            tessera_ui::LongPressResult::Triggered { .. }
        ) {
            continue;
        }
        if !controller.with(|controller| {
            controller
                .touch
                .is_some_and(|touch| touch.pointer == down.change.pointer_id)
        }) {
            continue;
        }
        controller.with_mut(|controller| {
            if let Some(touch) = &mut controller.touch {
                touch.triggered = true;
            }
            controller.open = true;
            controller.pending_show = None;
            controller.pending_hide = None;
        });
        let end = scope
            .consume_until_up(
                tessera_ui::PointerEventPass::Initial,
                down.change.pointer_id,
            )
            .await;
        controller.with_mut(|controller| match end {
            tessera_ui::GestureEnd::Released(up) => {
                controller.manual = controller.persistent;
                controller.pending_hide = (!controller.persistent)
                    .then_some(Delay::new(Some(up.timestamp), TOUCH_DURATION));
            }
            tessera_ui::GestureEnd::Cancelled => controller.dismiss(),
        });
    }
}

struct TooltipNavigation {
    controller: State<TooltipController>,
}

impl TooltipNavigation {
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

impl KeyboardInputModifierNode for TooltipNavigation {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        input.keyboard_events.retain(|event| {
            event.logical_key != Key::Named(NamedKey::Tab)
                || !self.handle_tab(event.state, input.key_modifiers.shift_key())
        });
    }
}

impl PointerInputModifierNode for TooltipNavigation {
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
                    let at = input
                        .pointer_changes
                        .iter()
                        .map(|change| change.timestamp)
                        .max();
                    controller.hover(false, false, at);
                    controller.hover(true, false, at);
                }
            });
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
        controller.with_mut(|controller| controller.focus(popup, focused));
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

fn popup_position(
    anchor: FloatRect,
    size: PxSize,
    available: FloatRect,
    placement: TooltipPlacement,
) -> PxPosition {
    let gap = Dp(4.0).to_pixels_f32();
    let margin = Dp(8.0).to_pixels_f32();
    let anchor = FloatRect {
        x: anchor.x - available.x,
        y: anchor.y - available.y,
        ..anchor
    };
    let width = available.width.max(0.0);
    let height = available.height.max(0.0);
    let popup_width = size.width.to_f32();
    let popup_height = size.height.to_f32();
    let margin_x = margin.min((width - popup_width).max(0.0) * 0.5);
    let margin_y = margin.min((height - popup_height).max(0.0) * 0.5);
    let above = anchor.y - gap - popup_height;
    let below = anchor.y + anchor.height + gap;
    let start = anchor.x - gap - popup_width;
    let end = anchor.x + anchor.width + gap;
    let center_x = anchor.x + (anchor.width - popup_width) * 0.5;
    let center_y = anchor.y + (anchor.height - popup_height) * 0.5;
    let (x, y) = match placement {
        TooltipPlacement::Above => (center_x, if above >= margin_y { above } else { below }),
        TooltipPlacement::Below => (
            center_x,
            if below + popup_height <= height - margin_y {
                below
            } else {
                above
            },
        ),
        TooltipPlacement::Start => (if start >= margin_x { start } else { end }, center_y),
        TooltipPlacement::End => (
            if end + popup_width <= width - margin_x {
                end
            } else {
                start
            },
            center_y,
        ),
    };
    PxPosition::from_f32_xy(
        available.x + x.clamp(margin_x, (width - popup_width - margin_x).max(margin_x)),
        available.y + y.clamp(margin_y, (height - popup_height - margin_y).max(margin_y)),
    )
}

/// Measures the tooltip inside the window minus its edge margins.
#[derive(Clone, PartialEq)]
struct WindowMargin;

impl LayoutPolicy for WindowMargin {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let margins = Px::from(Dp(16.0));
        let parent = scope.parent_constraint();
        let constraint = Constraint {
            width: parent.width().without_min() - margins,
            height: parent.height().without_min() - margins,
        };
        let mut result = LayoutResult::new(ComputedData::ZERO);
        for child in scope.children() {
            let size = child.measure(scope, &constraint)?.size();
            result.size = size;
            result.place_child(child, PxPosition::ZERO);
        }
        Ok(result)
    }
}

impl PopupPositionPolicy for TooltipPlacement {
    fn position(&self, input: &PopupPositionInput) -> PxPosition {
        popup_position(
            input.anchor.window_bounds(),
            input.popup_size,
            input.bounds,
            *self,
        )
    }
}

/// # tooltip_box
///
/// Attach a contextual explanation to a control without changing its action.
///
/// ## Usage
///
/// Explain an icon button on hover, keyboard focus, or touch long press.
///
/// ## Parameters
///
/// - `message` — supporting text shown in the popup
/// - `variant`, `headline` — plain or rich appearance and optional rich title
/// - `action_label`, `on_action` — optional rich action, which dismisses the
///   popup
/// - `actions` — custom rich actions, replacing the default text action
/// - `controller` — optional state for programmatic visibility control
/// - `group` — optional shared state to show only one tooltip at a time
/// - `placement` — preferred side, with edge avoidance and opposite-side
///   fallback
/// - `persistent` — keep a touch-triggered popup open until dismissal
/// - `modifier` — modifiers for the trigger wrapper
/// - `popup_modifier` — modifiers for the tooltip container
/// - `child` — the trigger that receives focus and normal activation
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::{button::button, tooltip::{TooltipController, tooltip_box}};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn save() {
///     let tooltip = remember(TooltipController::new);
///     tooltip_box("Save the current draft").controller(tooltip).child(|| {
///         button().accessibility_label("Save");
///     });
/// }
///
/// let mut tooltip = TooltipController::new();
/// assert!(!tooltip.is_open());
/// tooltip.show();
/// assert!(tooltip.is_open());
/// tooltip.dismiss();
/// assert!(!tooltip.is_open());
/// ```
#[tessera]
pub fn tooltip_box(
    #[prop(into)] message: String,
    variant: Option<TooltipVariant>,
    #[prop(into)] headline: Option<String>,
    #[prop(into)] action_label: Option<String>,
    on_action: Option<Callback>,
    actions: Option<RenderSlot>,
    controller: Option<State<TooltipController>>,
    group: Option<State<Option<State<TooltipController>>>>,
    placement: Option<TooltipPlacement>,
    persistent: Option<bool>,
    modifier: Option<Modifier>,
    popup_modifier: Option<Modifier>,
    child: Option<RenderSlot>,
) {
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(TooltipController::new)
    };
    let variant = variant.unwrap_or_default();
    let action_label = action_label.filter(|label| !label.is_empty());
    let interactive =
        variant == TooltipVariant::Rich && (action_label.is_some() || actions.is_some());
    let persistent = persistent.unwrap_or(interactive);
    if controller.with(|controller| {
        controller.persistent != persistent || controller.interactive != interactive
    }) {
        controller.with_mut(|controller| {
            controller.persistent = persistent;
            controller.interactive = interactive;
        });
    }
    if controller.with(TooltipController::needs_frame) {
        launch_once(run_frames(controller, TooltipController::advance));
    }
    let open = controller.with(TooltipController::is_open);
    if let Some(active) = group {
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
    }
    TooltipRestoreInput { controller }.restore();
    let focus = controller.with(|controller| controller.focus);
    let modifier = modifier
        .unwrap_or_default()
        .semantics(SemanticsArgs {
            role: Some(Role::Group),
            merge_descendants: true,
            tooltip: Some(message.clone()),
            ..Default::default()
        })
        .focus_group(None)
        .focus_requester(focus)
        .on_focus_changed(move |_: FocusState| sync_tooltip_focus(controller, false))
        .push_keyboard_preview_input(TooltipNavigation { controller })
        .push_pointer_preview_input(TooltipNavigation { controller })
        .push_pointer_preview_input(TooltipPointer {
            controller,
            popup: false,
        })
        .pointer_input(controller, move |scope| {
            tooltip_touch_detector(scope, controller)
        });
    let next = (
        open,
        message,
        headline,
        action_label,
        on_action,
        child,
        variant,
        actions,
        popup_modifier,
        placement.unwrap_or_default(),
    );
    let render_data = remember(|| next.clone());
    if render_data.with(|current| current != &next) {
        render_data.set(next);
    }
    layout(Stack).modifier(modifier).child(move || {
        let (
            open,
            message,
            headline,
            action_label,
            on_action,
            child,
            variant,
            actions,
            popup_modifier,
            placement,
        ) = render_data.get();
        if let Some(child) = child {
            child.render();
        }
        if open {
            let panel = tooltip_popup(placement, controller, message)
                .modifier_optional(popup_modifier)
                .variant(variant)
                .headline_optional(headline)
                .action_label_optional(action_label);
            if let Some(on_action) = on_action {
                panel.on_action_shared(on_action);
            }
            if let Some(actions) = actions {
                panel.actions_shared(actions);
            }
        }
    });
}

#[tessera]
fn tooltip_popup(
    placement: TooltipPlacement,
    controller: State<TooltipController>,
    message: String,
    variant: Option<TooltipVariant>,
    headline: Option<String>,
    action_label: Option<String>,
    on_action: Option<Callback>,
    actions: Option<RenderSlot>,
    modifier: Option<Modifier>,
) {
    popup(placement)
        .on_dismiss_request(move || controller.with_mut(TooltipController::dismiss))
        .child(move || {
            let (message, modifier, headline, action_label, actions) = (
                message.clone(),
                modifier.clone(),
                headline.clone(),
                action_label.clone(),
                actions.clone(),
            );
            tooltip_margin(move || {
                let panel = tooltip_panel(message.clone(), controller)
                    .modifier_optional(modifier.clone())
                    .variant_optional(variant)
                    .headline_optional(headline.clone())
                    .action_label_optional(action_label.clone());
                if let Some(on_action) = on_action {
                    panel.on_action_shared(on_action);
                }
                if let Some(actions) = &actions {
                    panel.actions_shared(actions.clone());
                }
            });
        });
}

#[tessera]
fn tooltip_margin(content: RenderSlot) {
    layout(WindowMargin).child(content);
}

#[tessera]
fn tooltip_panel(
    message: String,
    controller: State<TooltipController>,
    variant: Option<TooltipVariant>,
    headline: Option<String>,
    action_label: Option<String>,
    on_action: Option<Callback>,
    actions: Option<RenderSlot>,
    modifier: Option<Modifier>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let variant = variant.unwrap_or_default();
    let rich = variant == TooltipVariant::Rich;
    let has_actions = actions.is_some();
    let actions = actions.unwrap_or_else(RenderSlot::empty);
    let headline = headline.filter(|text| !text.is_empty());
    let action_label = action_label.filter(|text| !text.is_empty());
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
    let modifier = modifier
        .unwrap_or_default()
        .size_in(Some(Dp(40.0)), Some(max_width), Some(Dp(24.0)), None)
        .focus_group(Some(controller.with(|controller| controller.popup_group)))
        .focus_requester(controller.with(|controller| controller.popup_focus))
        .on_focus_changed(move |_: FocusState| sync_tooltip_focus(controller, true))
        .push_keyboard_input(TooltipRestoreInput { controller })
        .push_pointer_final_input(TooltipRestoreInput { controller })
        .push_pointer_preview_input(TooltipPointer {
            controller,
            popup: true,
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
                        Dp::ZERO,
                        Dp(12.0),
                        Dp::ZERO,
                        if action_row { Dp(8.0) } else { Dp(16.0) },
                    )
                } else {
                    Padding::symmetric(Dp(8.0), Dp(4.0))
                }))
                .children(move || {
                    let (theme, message, headline, action_label, rich, actions, has_actions) =
                        render_data.get();
                    if rich {
                        column()
                            .modifier(
                                Modifier::new().padding(Padding::symmetric(Dp(16.0), Dp::ZERO)),
                            )
                            .children(move || {
                                if let Some(headline) = &headline {
                                    text(headline.clone()).style(theme.typography.title_small);
                                    crate::spacer::spacer()
                                        .modifier(Modifier::new().height(Dp(4.0)));
                                }
                                text(message.clone()).style(theme.typography.body_medium);
                            });
                    } else {
                        text(message.clone()).style(theme.typography.body_small);
                    }
                    if rich && has_actions {
                        crate::spacer::spacer().modifier(Modifier::new().height(Dp(12.0)));
                        actions.render();
                    } else if rich && action_label.is_some() {
                        crate::spacer::spacer().modifier(Modifier::new().height(Dp(12.0)));
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
                                text(label).style(render_data.with(|(theme, _, _, _, _, _, _)| {
                                    theme.typography.label_large
                                }));
                            });
                    }
                });
        });
}

#[cfg(test)]
mod tests {

    use tessera_ui::testing::UiTest;

    use crate::{animation::test_instant, boxed::boxed, md3::theme::material_theme};

    use super::*;

    fn at(offset: Duration) -> Instant {
        test_instant(0) + offset
    }

    #[test]
    fn hover_delay_gap_crossing_escape_suppression_and_idle_clock() {
        let mut controller = TooltipController::new();
        controller.hover(false, true, Some(at(Duration::ZERO)));
        controller.advance(at(SHOW_DELAY - Duration::from_nanos(1)));
        assert!(!controller.is_open());
        controller.advance(at(SHOW_DELAY));
        assert!(controller.is_open());
        assert!(!controller.needs_frame());
        controller.hover(false, false, Some(at(SHOW_DELAY)));
        controller.hover(true, true, Some(at(SHOW_DELAY + EXIT_DELAY / 2)));
        controller.advance(at(SHOW_DELAY + EXIT_DELAY * 2));
        assert!(controller.is_open());
        controller.dismiss();
        controller.hover(true, true, Some(at(SHOW_DELAY * 2)));
        controller.advance(at(SHOW_DELAY * 3));
        assert!(!controller.is_open());
        assert!(!controller.needs_frame());
        controller.hover(true, false, Some(at(SHOW_DELAY * 3)));
        controller.hover(false, true, Some(at(SHOW_DELAY * 4)));
        controller.advance(at(SHOW_DELAY * 5));
        assert!(controller.is_open());
    }

    #[test]
    fn focus_keeps_hovered_content_and_escape_restoration_does_not_reopen_it() {
        let mut controller = TooltipController::new();
        controller.focus(false, true);
        assert!(controller.is_open());
        controller.focus(false, false);
        controller.focus(true, true);
        controller.advance(at(EXIT_DELAY * 2));
        assert!(controller.is_open());
        controller.dismiss();
        controller.focus(true, false);
        controller.focus(false, true);
        controller.advance(at(EXIT_DELAY * 3));
        assert!(!controller.is_open());
        controller.focus(false, false);
        controller.focus(false, true);
        assert!(controller.is_open());
        controller.dismiss();
        controller.ignore_anchor_focus = true;
        controller.focus(false, false);
        controller.focus(false, true);
        assert!(controller.is_open());
    }

    #[test]
    fn touch_waits_for_threshold_and_manual_visibility_does_not_run_a_clock() {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| fixture(observed.clone()))
            .viewport(400, 320);
        session.frame();
        let controller = observed.get().controller.expect("controller");
        let point = anchor_center(&observed);
        session.pointer_press(1, point, PressKeyEventType::Left);
        session.frame();
        session.advance(SHOW_DELAY - Duration::from_nanos(1));
        assert!(!controller.with(TooltipController::is_open));
        session.advance(Duration::from_nanos(1));
        assert!(controller.with(TooltipController::is_open));
        session.run(|| controller.with_mut(TooltipController::show));
        session.advance(SHOW_DELAY * 100);
        assert!(controller.with(TooltipController::is_open));
        assert!(!controller.with(TooltipController::needs_frame));
    }

    #[test]
    fn stale_popup_input_cannot_reopen_a_dismissed_tooltip() {
        let mut controller = TooltipController::new();
        controller.show();
        controller.dismiss();
        controller.focus(true, true);
        controller.hover(true, true, Some(at(Duration::from_nanos(2))));
        controller.advance(at(SHOW_DELAY * 2));
        assert!(!controller.is_open());
        assert!(!controller.needs_frame());
        controller.focus(true, false);
        controller.focus(false, true);
        assert!(controller.is_open());
    }

    #[test]
    fn touch_focus_suppression_clears_when_actual_focus_leaves_the_anchor() {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| fixture(observed.clone()))
            .viewport(400, 320);
        session.frame();
        let controller = observed.get().controller.expect("controller");
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
        let available = tessera_ui::FloatRect {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        };
        let popup = PxSize::new(Px(120), Px(40));
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
            (
                TooltipPlacement::Below,
                PxRect::new(Px(150), Px(100), Px(41), Px(40)),
                PxPosition::from_f32_xy(110.5, 144.0),
            ),
        ];
        for (placement, anchor, expected) in cases {
            assert_eq!(
                popup_position(anchor.into(), popup, available, placement),
                expected
            );
        }
        let left = popup_position(
            PxRect::new(Px(0), Px(100), Px(40), Px(40)).into(),
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
        custom_actions: bool,
        without_headline: bool,
        popup_width: Option<Dp>,
    }

    #[derive(Clone, Copy, Default)]
    struct FixtureHandles {
        anchor_bounds: Option<FloatRect>,
        popup_bounds: Option<tessera_ui::FloatRect>,
        controller: Option<State<TooltipController>>,
        settings: Option<State<Settings>>,
        clicks: Option<State<u32>>,
        actions: Option<State<u32>>,
    }

    #[tessera]
    fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
        let controller = remember(TooltipController::new);
        let settings = remember(Settings::default);
        let clicks = remember(|| 0_u32);
        let actions = remember(|| 0_u32);
        // Rebuild the parent boundary when an action also updates sibling
        // content.
        let _ = actions.get();
        observed.with_mut(|handles| handles.controller = Some(controller));
        observed.with_mut(|handles| handles.settings = Some(settings));
        observed.with_mut(|handles| handles.clicks = Some(clicks));
        observed.with_mut(|handles| handles.actions = Some(actions));
        material_theme().child(move || {
            let config = settings.get();
            let popup_modifier = Modifier::new()
                .on_globally_positioned_precise({ let observed = observed.clone(); move |bounds| observed.with_mut(|handles| handles.popup_bounds = Some(bounds)) });
            let popup_modifier = if let Some(width) = config.popup_width {
                popup_modifier.width(width)
            } else {
                popup_modifier
            };
            let observed = observed.clone();
            boxed().modifier(Modifier::new().fill_max_size().padding_all(Dp(80.0))).children(move || {
            let tooltip = tooltip_box(if config.changed { "Updated help text with multiple words that wraps within the tooltip's maximum width." } else { "Helpful description" })
                .modifier(Modifier::new().on_globally_positioned_precise({ let observed = observed.clone(); move |bounds| observed.with_mut(|handles| handles.anchor_bounds = Some(bounds)) }))
                .popup_modifier(popup_modifier.clone())
                .controller(controller)
                .variant(if config.rich { TooltipVariant::Rich } else { TooltipVariant::Plain })
                .headline(if config.without_headline { "" } else { "More information" })
                .action_label(if config.rich { "Got it" } else { "" })
                .on_action(move || actions.set(actions.get() + 1));
            let tooltip = if config.custom_actions {
                tooltip.actions(move || {
                    crate::row::row().children(move || {
                        button().text().accessibility_label("Not now").on_click(move || controller.with_mut(TooltipController::dismiss)).child(|| { text("Not now"); });
                        button().text().accessibility_label("Got it").on_click(move || {
                            controller.with_mut(TooltipController::dismiss);
                            actions.set(actions.get() + 1);
                        }).child(|| { text("Got it"); });
                    });
                })
            } else {
                tooltip
            };
            tooltip.child(move || {
                            button().modifier(Modifier::new().width(Dp(120.0))).accessibility_label("Trigger").on_click(move || {
                                clicks.set(clicks.get() + 1);
                            }).child(|| { text("Trigger"); });
            });
            });
        });
    }

    fn renderer() -> (
        UiTest<'static>,
        tessera_ui::testing::TestProbe<FixtureHandles>,
    ) {
        let observed = tessera_ui::testing::TestProbe::default();
        let fixture_observed = observed.clone();
        let mut renderer = UiTest::new()
            .viewport(400, 320)
            .rendered(crate::TesseraComponents)
            .content(move || fixture(fixture_observed.clone()));
        renderer.frames(3);
        (renderer, observed)
    }

    fn anchor_center(observed: &tessera_ui::testing::TestProbe<FixtureHandles>) -> PxPosition {
        let rect = observed.get().anchor_bounds.expect("anchor bounds");
        PxPosition::from_f32_xy(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
    }

    #[test]
    fn tooltip_escape_restores_trigger_focus_and_outside_press_still_activates_trigger() {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut ui = UiTest::new()
            .viewport(400, 320)
            .content(|| fixture(observed.clone()));
        ui.frame();
        let handles = observed.get();
        handles
            .settings
            .unwrap()
            .with_mut(|settings| settings.rich = true);
        let controller = handles.controller.unwrap();
        let trigger = controller.with(|controller| controller.focus);
        trigger.request_focus();
        ui.frames(2);
        ui.press_key(NamedKey::Tab);
        ui.frame();
        assert!(controller.with(|controller| controller.popup_group.has_focus()));
        ui.press_key(NamedKey::Escape);
        ui.frames(2);
        assert!(!controller.with(TooltipController::is_open));
        assert!(trigger.has_focus());

        controller.with_mut(TooltipController::show);
        ui.frame();
        let point = anchor_center(&observed);
        ui.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        ui.frame();
        assert!(!controller.with(TooltipController::is_open));
        ui.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        ui.frame();
        assert_eq!(handles.clicks.unwrap().get(), 1);
    }

    #[tessera]
    fn clipped_host_fixture(scale: f32, observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
        let controller = remember(|| {
            let mut controller = TooltipController::new();
            controller.show();
            controller
        });
        observed.with_mut(|handles| handles.controller = Some(controller));
        tessera_ui::scaled_box(scale).child(move || {
            let observed = observed.clone();
            material_theme().child(move || {
                let observed = observed.clone();
                boxed()
                    .modifier(
                        Modifier::new()
                            .clip_to_bounds()
                            .required_size(Dp(400.0), Dp(300.0)),
                    )
                    .children(move || {
                        tooltip_box("This explanation must remain visible above the trigger.")
                            .controller(controller)
                            .variant(TooltipVariant::Rich)
                            .action_label("Got it")
                            .placement(TooltipPlacement::Above)
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(120.0), Dp(0.0))
                                    .on_globally_positioned_precise({
                                        let observed = observed.clone();
                                        move |bounds| {
                                            observed.with_mut(|handles| {
                                                handles.anchor_bounds = Some(bounds)
                                            })
                                        }
                                    }),
                            )
                            .child(|| {
                                button().child(|| crate::md3::text::text("Trigger"));
                            });
                    });
            });
        });
    }

    #[test]
    fn rich_tooltip_flips_at_window_edge_without_inheriting_anchor_scale() {
        for scale in [1.0, 0.75] {
            let observed = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(400, 320)
                .content(|| clipped_host_fixture(scale, observed.clone()));
            renderer.frames(6);
            let tree = renderer.accessibility_snapshot().unwrap();
            let popup = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Dialog)
                .expect("placed rich tooltip")
                .1
                .bounds()
                .unwrap();
            let anchor = observed.get().anchor_bounds.expect("anchor bounds");
            assert!(
                (popup.y0 - f64::from(anchor.y + anchor.height) - 4.0).abs() <= 1.0,
                "a preferred-above popup must flip below the top-edge trigger with an unscaled gap: {popup:?}, {anchor:?}",
            );
            renderer.resize(400, 200);
            renderer.frames(6);
            let tree = renderer.accessibility_snapshot().unwrap();
            let resized = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Dialog)
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                resized.y1 <= 200.0 - 8.0 + 1.0,
                "window resize must update popup bounds: {resized:?}"
            );
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Got it"))
                .expect("tooltip action")
                .1
                .bounds()
                .expect("action bounds");
            let point = PxPosition::from_f32_xy(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.frame();
            renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.frames(2);
            assert!(
                !observed
                    .get()
                    .controller
                    .unwrap()
                    .with(TooltipController::is_open)
            );
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_hover_restores_pixels_preserves_focus_and_updates_content_and_variant() {
        for without_headline in [false, true] {
            check_tooltip_interactions(false, without_headline);
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_custom_actions_restore_focus_and_pixels_after_parent_rebuild() {
        for without_headline in [false, true] {
            check_tooltip_interactions(true, without_headline);
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_popup_modifier_preserves_requested_width_and_restores_natural_bounds() {
        let (mut renderer, observed) = renderer();
        let settings = observed.get().settings.expect("settings");
        settings.with_mut(|settings| {
            settings.rich = true;
            settings.popup_width = Some(Dp(312.0));
        });
        observed
            .get()
            .controller
            .expect("controller")
            .with_mut(TooltipController::show);
        renderer.frames(4);
        let requested = renderer.accessibility_snapshot().expect("tree");
        let requested = requested
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Dialog)
            .expect("rich tooltip")
            .1
            .bounds()
            .expect("popup bounds");
        assert_eq!(requested.width(), 312.0);
        assert_eq!(
            observed.get().popup_bounds.expect("observed popup").width,
            312.0
        );

        settings.with_mut(|settings| settings.popup_width = None);
        renderer.frames(4);
        let natural = renderer.accessibility_snapshot().expect("tree");
        let natural = natural
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Dialog)
            .expect("rich tooltip")
            .1
            .bounds()
            .expect("popup bounds");
        assert!(natural.width() >= 40.0 && natural.width() <= 320.0);
        assert!(natural.width() < requested.width());
        assert_eq!(
            observed.get().popup_bounds.expect("observed popup").width as f64,
            natural.width()
        );
    }

    fn assert_rich_action_insets(renderer: &UiTest, custom_actions: bool, without_headline: bool) {
        let tree = renderer.accessibility_snapshot().expect("tree");
        let bounds = |role: Role, label: &str| {
            tree.nodes
                .iter()
                .find(|(_, node)| node.role() == role && node.label() == Some(label))
                .unwrap_or_else(|| panic!("missing {label} ({role:?})"))
                .1
                .bounds()
                .expect("semantic bounds")
        };
        let popup = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Dialog)
            .expect("rich tooltip")
            .1
            .bounds()
            .expect("popup bounds");
        let body = bounds(Role::Label, "Helpful description");
        assert_eq!(body.x0 - popup.x0, 16.0);
        if without_headline {
            assert!(
                !tree.nodes.iter().any(|(_, node)| node.role() == Role::Label
                    && node.label() == Some("More information"))
            );
            assert_eq!(body.y0 - popup.y0, 12.0);
        } else {
            let headline = bounds(Role::Label, "More information");
            assert_eq!(headline.x0 - popup.x0, 16.0);
            assert_eq!(headline.y0 - popup.y0, 12.0);
            assert_eq!(body.y0 - headline.y1, 4.0);
        }
        let first_label = if custom_actions { "Not now" } else { "Got it" };
        let first = bounds(Role::Button, first_label);
        let first_text = bounds(Role::Label, first_label);
        assert_eq!(first_text.x0 - popup.x0, 16.0);
        assert_eq!(first.y0 - body.y1, 12.0);
        assert_eq!(popup.y1 - first.y1, 8.0);
        let labels: &[&str] = if custom_actions {
            &["Not now", "Got it"]
        } else {
            &["Got it"]
        };
        for label in labels {
            let button = bounds(Role::Button, label);
            let text = bounds(Role::Label, label);
            assert!(button.width() >= 48.0 && button.height() >= 48.0);
            assert!(
                text.x0 >= button.x0
                    && text.x1 <= button.x1
                    && text.y0 >= button.y0
                    && text.y1 <= button.y1
            );
            assert_eq!(text.x0 - button.x0, 16.0);
        }
        if custom_actions {
            let second = bounds(Role::Button, "Got it");
            assert_eq!(second.x0, first.x1);
            assert_eq!(second.y0, first.y0);
            assert_eq!(second.y1, first.y1);
        }
    }

    fn check_tooltip_interactions(custom_actions: bool, without_headline: bool) {
        let (mut renderer, observed) = renderer();
        observed
            .get()
            .settings
            .expect("settings")
            .with_mut(|settings| {
                settings.custom_actions = custom_actions;
                settings.without_headline = without_headline;
            });
        renderer.frames(2);
        let before = renderer.read_pixels();
        let focus = renderer.accessibility_snapshot().expect("tree").focus;
        renderer.pointer_move(MOUSE_POINTER_ID, anchor_center(&observed));
        renderer.frames(35);
        assert!(
            observed
                .get()
                .controller
                .expect("controller")
                .with(TooltipController::is_open),
            "{:?}",
            observed.get().controller.expect("controller").get()
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
        let popup_bounds = observed
            .get()
            .popup_bounds
            .expect("popup modifier observes the actual container");
        assert_eq!(popup_bounds.x as f64, bounds.x0);
        assert_eq!(popup_bounds.y as f64, bounds.y0);
        assert_eq!(popup_bounds.width as f64, bounds.width());
        assert_eq!(popup_bounds.height as f64, bounds.height());
        assert!(bounds.x1 - bounds.x0 <= 200.0);
        assert!(bounds.y1 - bounds.y0 >= 24.0);
        let trigger = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Trigger") && node.role() == Role::Button)
            .expect("trigger");
        assert_eq!(trigger.1.description(), Some("Helpful description"));
        observed
            .get()
            .settings
            .expect("settings")
            .with_mut(|settings| settings.changed = true);
        renderer.frames(3);
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
        observed
            .get()
            .settings
            .expect("settings")
            .with_mut(|settings| {
                settings.rich = true;
                settings.changed = false;
            });
        renderer.frames(3);
        let tree = renderer.accessibility_snapshot().expect("tree");
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Dialog)
        );
        let body = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Label && node.label() == Some("Helpful description")
            })
            .expect("rich supporting text")
            .1
            .bounds()
            .expect("supporting text bounds");
        let action = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Got it"))
            .expect("rich action")
            .1
            .bounds()
            .expect("action bounds");
        assert_eq!(
            action.y0 - body.y1,
            12.0,
            "rich tooltip separates supporting text from actual actions by 12dp"
        );
        assert_rich_action_insets(&renderer, custom_actions, without_headline);
        let controller = observed.get().controller.expect("controller");
        let trigger = controller.with(|controller| controller.focus);
        trigger.request_focus();
        renderer.frames(2);
        let navigation = TooltipNavigation { controller };
        for _ in 0..2 {
            assert!(navigation.handle_tab(ElementState::Pressed, false));
            if custom_actions {
                assert!(navigation.handle_tab(ElementState::Pressed, false));
            }
            renderer.frames(2);
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
            renderer.frames(2);
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
        renderer.press_key(NamedKey::Escape);
        renderer.frames(2);
        assert!(trigger.has_focus());
        assert!(
            !controller
                .with(|controller| controller.popup_group)
                .has_focus()
        );
        assert!(!controller.with(TooltipController::is_open));
        controller.with_mut(TooltipController::show);
        renderer.frames(2);
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
        renderer.frames(1);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.frames(3);
        assert_eq!(observed.get().actions.expect("actions").get(), 1);
        assert!(
            !observed
                .get()
                .controller
                .expect("controller")
                .with(TooltipController::is_open)
        );
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::new(Px(390), Px(310)));
        let focus = observed
            .get()
            .controller
            .expect("controller")
            .with(|controller| controller.focus);
        focus.clear_focus();
        assert!(
            !focus.has_focus(),
            "clear must remove focus from the trigger subtree"
        );
        observed.get().settings.expect("settings").set(Settings {
            custom_actions,
            ..Settings::default()
        });
        renderer.frames(40);
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
        let restored = renderer.read_pixels();
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
        let (mut renderer, observed) = renderer();
        let position = anchor_center(&observed);
        renderer.pointer_press(1, position, PressKeyEventType::Left);
        renderer.frames(35);
        assert!(
            observed
                .get()
                .controller
                .expect("controller")
                .with(TooltipController::is_open)
        );
        renderer.pointer_release(1, position, PressKeyEventType::Left);
        renderer.frames(3);
        assert_eq!(observed.get().clicks.expect("clicks").get(), 0);
        renderer.frames(100);
        assert!(
            !observed
                .get()
                .controller
                .expect("controller")
                .with(TooltipController::is_open)
        );
        assert!(
            !observed
                .get()
                .controller
                .expect("controller")
                .with(TooltipController::needs_frame)
        );
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.frames(1);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.frames(1);
        assert_eq!(observed.get().clicks.expect("clicks").get(), 1);
        let controller = observed.get().controller.expect("controller");
        let focus = controller.with(|controller| controller.focus);
        focus.clear_focus();
        controller.with_mut(TooltipController::dismiss);
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::new(Px(390), Px(310)));
        renderer.frames(8);
        renderer.pointer_press(2, position, PressKeyEventType::Left);
        renderer.frames(1);
        renderer.pointer_move(2, PxPosition::from_f32_xy(position.x + 20.0, position.y));
        renderer.frames(1);
        assert!(
            controller.with(|controller| controller.touch.is_none()),
            "after movement: {:?}",
            controller.get()
        );
        renderer.frames(35);
        assert!(
            !controller.with(TooltipController::is_open),
            "after cancellation: {:?}",
            controller.get()
        );
        renderer.pointer_release(2, position, PressKeyEventType::Left);
        renderer.frames(1);
    }
}
