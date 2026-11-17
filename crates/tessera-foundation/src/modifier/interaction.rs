//! Shared interaction configuration and drag behavior.
//!
//! ## Usage
//!
//! Configure clickable, combined-clickable, toggleable, selectable, and draggable modifier
//! behavior.

use tessera_ui::{
    BuildModifierNode, Callback, CallbackWith, FocusProperties, FocusRequester, Modifier,
    ModifierBuildContext, PointerInput, PointerInputModifierNode, Px, PxPosition, PxSize, State,
    accesskit, modifier::ModifierCapabilityExt as _, time::Instant,
};

use crate::pointer_gestures::{DragAxis, DragGestureConfig, detect_drag_gestures};

/// Context for pointer press/release callbacks.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct PointerEventContext {
    /// Pointer position normalized to `[0.0, 1.0]` within the element bounds.
    pub normalized_pos: [f32; 2],
    /// The element size in pixels.
    pub size: PxSize,
    /// When the pointer event that triggered the callback occurred.
    pub timestamp: Instant,
}

type PressCallback = CallbackWith<PointerEventContext, ()>;
type DragCallback = CallbackWith<DragDelta, ()>;

/// Arguments for the `clickable` modifier.
#[derive(Clone)]
pub struct ClickableArgs {
    /// Callback invoked when the element is clicked.
    pub on_click: Callback,
    /// Whether the element is enabled for interaction.
    pub enabled: bool,
    /// Whether to block input propagation when within bounds.
    pub block_input: bool,
    /// Optional press callback with normalized position and element size.
    pub on_press: Option<PressCallback>,
    /// Optional release callback with normalized position and element size.
    pub on_release: Option<PressCallback>,
    /// Optional accessibility role.
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state.
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional externally managed focus requester for this clickable target.
    pub focus_requester: Option<FocusRequester>,
    /// Optional focus properties applied to the clickable target.
    pub focus_properties: Option<FocusProperties>,
}

impl Default for ClickableArgs {
    fn default() -> Self {
        Self {
            on_click: Callback::noop(),
            enabled: true,
            block_input: true,
            on_press: None,
            on_release: None,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            focus_requester: None,
            focus_properties: None,
        }
    }
}

/// Arguments for the `combined_clickable` modifier.
#[derive(Clone)]
pub struct CombinedClickableArgs {
    /// Optional click action; omit it for a long-click-only target.
    pub on_click: Option<Callback>,
    /// Optional long-click action, also available through the context-menu key.
    pub on_long_click: Option<Callback>,
    /// Whether the element accepts interaction.
    pub enabled: bool,
    /// Whether to consume accepted presses and releases.
    pub block_input: bool,
    /// Optional callback when a pointer press is accepted.
    pub on_press: Option<PressCallback>,
    /// Optional callback when the accepted pointer is released.
    pub on_release: Option<PressCallback>,
    /// Optional accessibility role.
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state.
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional externally managed focus requester.
    pub focus_requester: Option<FocusRequester>,
    /// Optional focus properties for the target.
    pub focus_properties: Option<FocusProperties>,
}

impl Default for CombinedClickableArgs {
    fn default() -> Self {
        Self {
            on_click: None,
            on_long_click: None,
            enabled: true,
            block_input: true,
            on_press: None,
            on_release: None,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            focus_requester: None,
            focus_properties: None,
        }
    }
}

/// Arguments for the `toggleable` modifier.
#[derive(Clone)]
pub struct ToggleableArgs {
    /// Current boolean value.
    pub value: bool,
    /// Callback invoked with the new value when changed.
    pub on_value_change: CallbackWith<bool, ()>,
    /// Whether the control is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role.
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state.
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional press callback with normalized position and element size.
    pub on_press: Option<PressCallback>,
    /// Optional release callback with normalized position and element size.
    pub on_release: Option<PressCallback>,
    /// Optional externally managed focus requester for this toggleable target.
    pub focus_requester: Option<FocusRequester>,
}

impl Default for ToggleableArgs {
    fn default() -> Self {
        Self {
            value: false,
            on_value_change: CallbackWith::default_value(),
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            on_press: None,
            on_release: None,
            focus_requester: None,
        }
    }
}

/// Arguments for the `selectable` modifier.
#[derive(Clone)]
pub struct SelectableArgs {
    /// Whether the item is selected.
    pub selected: bool,
    /// Callback invoked when the selectable is activated.
    pub on_click: Callback,
    /// Whether the element is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role.
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state.
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional press callback with normalized position and element size.
    pub on_press: Option<PressCallback>,
    /// Optional release callback with normalized position and element size.
    pub on_release: Option<PressCallback>,
    /// Optional externally managed focus requester for this selectable target.
    pub focus_requester: Option<FocusRequester>,
}

impl Default for SelectableArgs {
    fn default() -> Self {
        Self {
            selected: false,
            on_click: Callback::noop(),
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            on_press: None,
            on_release: None,
            focus_requester: None,
        }
    }
}

/// Pointer delta produced by the `draggable` modifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DragDelta {
    /// Horizontal delta in local physical pixels.
    pub x: Px,
    /// Vertical delta in local physical pixels.
    pub y: Px,
}

/// Arguments for the `draggable` modifier.
#[derive(Clone)]
pub struct DraggableArgs {
    /// Callback invoked with each drag delta update.
    pub on_drag_delta: DragCallback,
    /// Whether dragging is enabled.
    pub enabled: bool,
    /// Optional drag axis lock.
    pub axis: Option<DragAxis>,
    /// Minimum travel before the drag starts.
    pub slop_px: f32,
    /// Whether drag pointer changes should be consumed after dragging starts.
    pub consume_when_dragging: bool,
    /// Optional callback invoked when dragging starts.
    pub on_drag_started: Option<Callback>,
    /// Optional callback invoked when dragging stops after an active drag.
    pub on_drag_stopped: Option<Callback>,
    /// Optional external interaction state updated with the dragged flag.
    pub interaction_state: Option<State<InteractionState>>,
}

impl Default for DraggableArgs {
    fn default() -> Self {
        Self {
            on_drag_delta: CallbackWith::default_value(),
            enabled: true,
            axis: None,
            slop_px: DragGestureConfig::default().slop_px,
            consume_when_dragging: DragGestureConfig::default().consume,
            on_drag_started: None,
            on_drag_stopped: None,
            interaction_state: None,
        }
    }
}

/// A visual interaction used when a control has no active user interaction.
///
/// Presentation changes appearance without requesting focus, dispatching input,
/// or changing accessibility state. Disabled controls ignore presentation
/// state.
///
/// ```
/// use tessera_foundation::modifier::{InteractionState, InteractionVisualState};
///
/// let live = InteractionState::new();
/// let visual = live.with_presentation(Some(InteractionVisualState::Focus), true);
/// assert!(visual.is_focused());
/// assert!(!live.is_focused());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InteractionVisualState {
    /// The resting appearance.
    #[default]
    Rest,
    /// The hovered appearance.
    Hover,
    /// The focused appearance.
    Focus,
    /// The pressed appearance.
    Pressed,
    /// The dragged appearance.
    Dragged,
}

/// Tracks basic interaction flags and derives state-layer alpha.
#[derive(Clone, PartialEq, Copy, Debug, Default)]
pub struct InteractionState {
    is_hovered: bool,
    is_focused: bool,
    is_dragged: bool,
    is_pressed: bool,
}

impl InteractionState {
    /// Resolves appearance while preserving all active user interaction flags.
    ///
    /// Disabled controls return a resting state. Otherwise, any active live
    /// interaction takes precedence over the optional presentation state.
    /// The returned snapshot does not modify the live interaction state.
    pub fn with_presentation(
        self,
        presentation_state: Option<InteractionVisualState>,
        enabled: bool,
    ) -> Self {
        if !enabled {
            return Self::default();
        }
        if self.is_hovered || self.is_focused || self.is_dragged || self.is_pressed {
            return self;
        }
        Self {
            is_hovered: presentation_state == Some(InteractionVisualState::Hover),
            is_focused: presentation_state == Some(InteractionVisualState::Focus),
            is_dragged: presentation_state == Some(InteractionVisualState::Dragged),
            is_pressed: presentation_state == Some(InteractionVisualState::Pressed),
        }
    }

    /// Creates a new interaction state with all flags cleared.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the component as no longer pressed.
    pub fn release(&mut self) {
        self.set_pressed(false);
    }

    /// Sets whether the component is hovered.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    /// Returns whether the component is hovered.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    /// Sets whether the component is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Returns whether the component is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Sets whether the component is dragged.
    pub fn set_dragged(&mut self, dragged: bool) {
        self.is_dragged = dragged;
    }

    /// Returns whether the component is dragged.
    pub fn is_dragged(&self) -> bool {
        self.is_dragged
    }

    /// Sets whether the component is pressed.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    /// Returns whether the component is pressed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns the state-layer alpha derived from the current interactions.
    pub fn state_layer_alpha(&self) -> f32 {
        if self.is_dragged {
            0.16
        } else if self.is_pressed || self.is_focused {
            0.10
        } else if self.is_hovered {
            0.08
        } else {
            0.0
        }
    }
}

async fn async_draggable_detector(
    mut scope: tessera_ui::PointerInputScope,
    settings: DragGestureConfig,
    on_drag_delta: DragCallback,
    on_drag_started: Option<Callback>,
    on_drag_stopped: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
) {
    let remainder = std::cell::Cell::new(PxPosition::ZERO);
    let stop = || {
        if let Some(state) = interaction_state
            && state.is_alive()
        {
            state.with_mut(|state| state.set_dragged(false));
        }
        if let Some(callback) = on_drag_stopped {
            callback.call();
        }
    };
    detect_drag_gestures(
        &mut scope,
        settings,
        |_| {
            remainder.set(PxPosition::ZERO);
            if let Some(state) = interaction_state {
                state.with_mut(|state| state.set_dragged(true));
            }
            if let Some(callback) = on_drag_started {
                callback.call();
            }
        },
        |_, _, delta| {
            let total = delta + remainder.get();
            let x = total.x.round();
            let y = total.y.round();
            remainder.set(PxPosition::from_f32_xy(total.x - x, total.y - y));
            on_drag_delta.call(DragDelta {
                x: Px(x as i32),
                y: Px(y as i32),
            });
        },
        |_, _| stop(),
        || {
            if let Some(state) = interaction_state
                && state.is_alive()
            {
                state.with_mut(|state| state.set_dragged(false));
            }
        },
    )
    .await;
}

pub(crate) fn apply_draggable_modifier(base: Modifier, args: DraggableArgs) -> Modifier {
    let DraggableArgs {
        on_drag_delta,
        enabled,
        axis,
        slop_px,
        consume_when_dragging,
        on_drag_started,
        on_drag_stopped,
        interaction_state,
    } = args;

    let settings = DragGestureConfig {
        slop_px,
        consume: consume_when_dragging,
        axis,
    };
    base.push_build(DraggableBuild {
        settings,
        on_drag_delta,
        enabled,
        on_drag_started,
        on_drag_stopped,
        interaction_state,
    })
}

fn cursor_within_inclusive_bounds(position: Option<PxPosition>, size: PxSize) -> bool {
    position.is_some_and(|position| {
        position.x >= 0.0
            && position.y >= 0.0
            && position.x <= size.width.0 as f32
            && position.y <= size.height.0 as f32
    })
}

struct BlockTouchPropagationPointerModifierNode;

impl PointerInputModifierNode for BlockTouchPropagationPointerModifierNode {
    fn receives_consumed(&self) -> bool {
        true
    }

    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let size = PxSize::new(input.computed_data.width, input.computed_data.height);
        if cursor_within_inclusive_bounds(input.cursor_position_rel, size) {
            input.block_cursor();
        }
    }
}

pub(crate) fn apply_block_touch_propagation_modifier(base: Modifier) -> Modifier {
    base.push_pointer_input(BlockTouchPropagationPointerModifierNode)
}

#[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
async fn async_window_drag_detector(mut scope: tessera_ui::PointerInputScope) {
    loop {
        let Some(mut down) = scope
            .await_first_down(
                tessera_ui::PointerEventPass::Main,
                Some(tessera_ui::PressKeyEventType::Left),
                tessera_ui::DownRequirement::Any,
            )
            .await
        else {
            return;
        };
        down.drag_window();
        down.block_cursor();
        down.change.consume();
        loop {
            let event = scope
                .await_pointer_event(tessera_ui::PointerEventPass::Main)
                .await;
            if event.is_cancelled() {
                return;
            }
            event.block_cursor();
            let mut ended = false;
            for mut change in event.changes {
                if change.pointer_id != down.change.pointer_id {
                    continue;
                }
                match change.content {
                    tessera_ui::CursorEventContent::Released(_)
                    | tessera_ui::CursorEventContent::Cancelled(_) => ended = true,
                    _ => {}
                }
                change.consume();
            }
            if ended {
                break;
            }
        }
    }
}

#[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
struct WindowDragBuild;

#[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
impl BuildModifierNode for WindowDragBuild {
    fn materialize(&self, _context: &mut ModifierBuildContext) -> Option<Modifier> {
        Some(Modifier::new().pointer_input_once(async_window_drag_detector))
    }
}

#[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
pub(crate) fn apply_window_drag_region_modifier(base: Modifier) -> Modifier {
    base.push_build(WindowDragBuild)
}

#[cfg(any(target_family = "wasm", target_os = "android", target_os = "ios"))]
pub(crate) fn apply_window_drag_region_modifier(base: Modifier) -> Modifier {
    base
}

struct DraggableBuild {
    settings: DragGestureConfig,
    on_drag_delta: DragCallback,
    enabled: bool,
    on_drag_started: Option<Callback>,
    on_drag_stopped: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
}

impl BuildModifierNode for DraggableBuild {
    fn materialize(&self, _context: &mut ModifierBuildContext) -> Option<Modifier> {
        if !self.enabled {
            if let Some(state) = self.interaction_state {
                state.with_mut(|state| state.set_dragged(false));
            }
            return Some(Modifier::new());
        }
        let settings = self.settings;
        let on_drag_delta = self.on_drag_delta;
        let on_drag_started = self.on_drag_started;
        let on_drag_stopped = self.on_drag_stopped;
        let interaction_state = self.interaction_state;
        Some(Modifier::new().pointer_input(
            (
                settings,
                on_drag_delta,
                on_drag_started,
                on_drag_stopped,
                interaction_state,
            ),
            move |scope| {
                async_draggable_detector(
                    scope,
                    settings,
                    on_drag_delta,
                    on_drag_started,
                    on_drag_stopped,
                    interaction_state,
                )
            },
        ))
    }
}

#[cfg(test)]
mod presentation_tests {
    use super::{InteractionState, InteractionVisualState};

    #[test]
    fn presentation_preserves_live_combinations_and_disabled_priority() {
        for bits in 0..16 {
            let mut live = InteractionState::new();
            live.set_hovered(bits & 1 != 0);
            live.set_focused(bits & 2 != 0);
            live.set_pressed(bits & 4 != 0);
            live.set_dragged(bits & 8 != 0);
            assert_eq!(live.with_presentation(None, true), live);
            for preview in [
                InteractionVisualState::Rest,
                InteractionVisualState::Hover,
                InteractionVisualState::Focus,
                InteractionVisualState::Pressed,
                InteractionVisualState::Dragged,
            ] {
                assert_eq!(
                    live.with_presentation(Some(preview), false),
                    InteractionState::new(),
                );
                let visual = live.with_presentation(Some(preview), true);
                if bits != 0 {
                    assert_eq!(visual, live);
                } else {
                    assert_eq!(
                        visual.is_hovered(),
                        preview == InteractionVisualState::Hover
                    );
                    assert_eq!(
                        visual.is_focused(),
                        preview == InteractionVisualState::Focus
                    );
                    assert_eq!(
                        visual.is_pressed(),
                        preview == InteractionVisualState::Pressed
                    );
                    assert_eq!(
                        visual.is_dragged(),
                        preview == InteractionVisualState::Dragged
                    );
                }
            }
        }
    }
}
