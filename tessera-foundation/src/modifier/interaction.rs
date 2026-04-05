//! Shared interaction configuration types for modifier APIs.
//!
//! ## Usage
//!
//! Configure clickable, toggleable, selectable, and draggable modifier
//! behavior.

use tessera_ui::{
    Callback, CallbackWith, FocusProperties, FocusRequester, Modifier, PointerInput,
    PointerInputModifierNode, Px, PxPosition, PxSize, State, accesskit,
    modifier::ModifierCapabilityExt as _, remember,
};

use crate::gesture::{DragAxis, DragRecognizer, DragSettings};

/// Context for pointer press/release callbacks.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct PointerEventContext {
    /// Pointer position normalized to `[0.0, 1.0]` within the element bounds.
    pub normalized_pos: [f32; 2],
    /// The element size in pixels.
    pub size: PxSize,
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
    /// Horizontal delta in pixels.
    pub x: Px,
    /// Vertical delta in pixels.
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
            slop_px: DragSettings::default().slop_px,
            consume_when_dragging: DragSettings::default().consume_when_dragging,
            on_drag_started: None,
            on_drag_stopped: None,
            interaction_state: None,
        }
    }
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

struct DraggablePointerModifierNode {
    drag_recognizer: State<DragRecognizer>,
    on_drag_delta: DragCallback,
    enabled: bool,
    on_drag_started: Option<Callback>,
    on_drag_stopped: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
}

impl PointerInputModifierNode for DraggablePointerModifierNode {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if !self.enabled {
            return;
        }

        let cursor_position_abs = input.cursor_position_abs();
        let within_bounds = cursor_within_bounds(
            input.cursor_position_rel,
            PxSize::new(input.computed_data.width, input.computed_data.height),
        );
        let (was_dragging, drag_result) = self.drag_recognizer.with_mut(|recognizer| {
            let was_dragging = recognizer.is_dragging();
            let drag_result = recognizer.update(
                input.pass,
                input.pointer_changes,
                cursor_position_abs,
                within_bounds,
            );
            (was_dragging, drag_result)
        });

        if drag_result.started {
            if let Some(interaction_state) = self.interaction_state {
                interaction_state.with_mut(|state| state.set_dragged(true));
            }
            if let Some(on_drag_started) = self.on_drag_started {
                on_drag_started.call();
            }
        }

        if drag_result.updated {
            self.on_drag_delta.call(DragDelta {
                x: drag_result.delta_x,
                y: drag_result.delta_y,
            });
        }

        if drag_result.ended && was_dragging {
            if let Some(interaction_state) = self.interaction_state {
                interaction_state.with_mut(|state| state.set_dragged(false));
            }
            if let Some(on_drag_stopped) = self.on_drag_stopped {
                on_drag_stopped.call();
            }
        }
    }
}

fn cursor_within_bounds(position: Option<PxPosition>, size: PxSize) -> bool {
    let Some(position) = position else {
        return false;
    };

    position.x >= Px::ZERO
        && position.y >= Px::ZERO
        && position.x < size.width
        && position.y < size.height
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

    if !enabled {
        if let Some(interaction_state) = interaction_state {
            interaction_state.with_mut(|state| state.set_dragged(false));
        }
        return base;
    }

    let drag_recognizer = remember(move || {
        DragRecognizer::new(DragSettings {
            slop_px,
            consume_when_dragging,
            axis,
        })
    });
    drag_recognizer.with_mut(|recognizer| {
        recognizer.set_settings(DragSettings {
            slop_px,
            consume_when_dragging,
            axis,
        });
    });

    base.push_pointer_input(DraggablePointerModifierNode {
        drag_recognizer,
        on_drag_delta,
        enabled,
        on_drag_started,
        on_drag_stopped,
        interaction_state,
    })
}
