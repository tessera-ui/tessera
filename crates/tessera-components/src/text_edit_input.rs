use glyphon::Action as GlyphonAction;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, ComputedData,
    CursorEventContent, Dp, ImeInput, ImeInputModifierNode, ImeRequest, KeyboardInput,
    KeyboardInputModifierNode, Modifier, PressKeyEventType, Px, PxPosition, PxSize,
    SemanticsModifierNode, State,
    accesskit::{Action, Role},
    clipboard::{ClipboardAction, ClipboardInput, ClipboardInputModifierNode},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    winit,
};

use crate::{
    pos_misc::is_position_inside_bounds,
    text_edit_core::{
        ClickType, ImeEditResult, PlannedImeEvent, RectDef, TextEditorController, TextSelection,
    },
};

#[derive(Clone, PartialEq)]
pub(crate) struct TextEditInput {
    pub(crate) enabled: bool,
    pub(crate) read_only: bool,
    pub(crate) is_secure: bool,
    pub(crate) single_line: bool,
    pub(crate) on_change: CallbackWith<String, String>,
    pub(crate) on_submit: Callback,
    pub(crate) input_transform: Option<CallbackWith<String, String>>,
    pub(crate) accessibility_label: Option<String>,
    pub(crate) accessibility_description: Option<String>,
    pub(crate) content_insets: TextEditInsets,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TextEditInsets {
    pub(crate) horizontal: Dp,
    pub(crate) vertical: Dp,
    pub(crate) border: Dp,
}

impl TextEditInsets {
    fn origin(self) -> PxPosition {
        // The layout rounds the border and padding in separate modifier stages.
        let border = Px(self.border.to_pixels_u32() as i32);
        PxPosition::new(
            Px::from(self.horizontal) + border,
            Px::from(self.vertical) + border,
        )
    }
}

struct TextInputKeyboardModifierNode {
    args: TextEditInput,
    controller: State<TextEditorController>,
}

impl KeyboardInputModifierNode for TextInputKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        handle_text_input_keyboard(
            &mut input,
            &self.args,
            &self.controller,
            &self.args.on_change,
            &self.args.input_transform,
        );
    }
}

struct TextInputImeModifierNode {
    args: TextEditInput,
    controller: State<TextEditorController>,
}

struct TextInputClipboardModifierNode {
    args: TextEditInput,
    controller: State<TextEditorController>,
}

impl ClipboardInputModifierNode for TextInputClipboardModifierNode {
    fn on_clipboard_input(&self, mut input: ClipboardInput<'_>) {
        let controller = self.controller;
        if !self.args.enabled || !controller.with(|c| c.focus_handler().is_focused()) {
            return;
        }
        input.handle();
        match input.action {
            ClipboardAction::Paste(text) => {
                if !self.args.read_only {
                    controller.with_mut(|c| {
                        c.replace_selected_text_with_pipeline(
                            text,
                            self.args.on_change,
                            self.args.input_transform,
                        );
                    });
                }
            }
            ClipboardAction::Copy | ClipboardAction::Cut => {
                if self.args.read_only && matches!(input.action, ClipboardAction::Cut) {
                    return;
                }
                let Some(text) = controller.with(|c| c.selected_text()) else {
                    return;
                };
                if matches!(input.action, ClipboardAction::Cut) {
                    let (version, selection) =
                        controller.with(|c| (c.text_layout_version(), c.selection()));
                    let on_change = self.args.on_change;
                    let input_transform = self.args.input_transform;
                    input.write_text(text, move || {
                        if controller.is_alive()
                            && controller.with(|c| {
                                c.text_layout_version() == version && c.selection() == selection
                            })
                        {
                            controller.with_mut(|c| {
                                c.replace_selected_text_with_pipeline(
                                    "",
                                    on_change,
                                    input_transform,
                                );
                            });
                        }
                    });
                } else {
                    input.write_text(text, || {});
                }
            }
        }
    }
}

impl ImeInputModifierNode for TextInputImeModifierNode {
    fn on_ime_input(&self, mut input: ImeInput<'_>) {
        handle_text_input_ime(
            &mut input,
            &self.args,
            &self.controller,
            &self.args.on_change,
            &self.args.input_transform,
        );
    }
}

pub(crate) fn attach_text_edit_input(
    base: Modifier,
    args: TextEditInput,
    controller: State<TextEditorController>,
) -> Modifier {
    let modifier = if args.enabled {
        base.hover_cursor_icon(winit::window::CursorIcon::Text)
    } else {
        base
    };
    modifier
        .on_focus_changed(move |focus: tessera_ui::FocusState| {
            if !focus.is_focused()
                && controller.is_alive()
                && controller.with(|c| c.composition().is_some())
            {
                controller.with_mut(|c| c.clear_composition());
            }
        })
        .push_semantics(TextInputSemanticsModifierNode {
            args: args.clone(),
            controller,
        })
        .pointer_input((controller, args.clone()), {
            let args = args.clone();
            move |scope| run_text_edit_pointer_input(scope, args.clone(), controller)
        })
        .push_keyboard_input(TextInputKeyboardModifierNode {
            args: args.clone(),
            controller,
        })
        .push_clipboard_input(TextInputClipboardModifierNode {
            args: args.clone(),
            controller,
        })
        .push_ime_input(TextInputImeModifierNode { args, controller })
}

async fn run_text_edit_pointer_input(
    mut scope: tessera_ui::PointerInputScope,
    args: TextEditInput,
    controller: State<TextEditorController>,
) {
    struct SelectionLease(State<TextEditorController>);
    impl Drop for SelectionLease {
        fn drop(&mut self) {
            if self.0.is_alive() {
                self.0.with_mut(|controller| controller.stop_drag());
            }
        }
    }
    let _lease = SelectionLease(controller);
    let mut active: Option<(tessera_ui::PointerId, PxPosition, ClickType, bool)> = None;
    loop {
        let event = scope
            .await_pointer_event(tessera_ui::PointerEventPass::Main)
            .await;
        if event.is_cancelled() {
            return;
        }
        let cursor = event.cursor_position;
        let mut pressed = false;
        let inside = cursor.is_some_and(|position| {
            is_position_inside_bounds(
                ComputedData {
                    width: event.size.width,
                    height: event.size.height,
                },
                position,
            )
        });
        for mut change in event.changes.iter().cloned() {
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    if args.enabled && inside && !change.is_consumed() =>
                {
                    let Some(cursor) = cursor else { continue };
                    if !controller.with(|s| s.focus_handler().is_focused()) {
                        controller.with(|s| *s.focus_handler()).request_focus();
                    }
                    let position = click_selection_pointer_position(
                        cursor,
                        &args,
                        &controller,
                        ComputedData {
                            width: event.size.width,
                            height: event.size.height,
                        },
                    );
                    let window = change
                        .position()
                        .unwrap_or_else(|| event.local_to_window.map_point(cursor));
                    let kind =
                        controller.with_mut(|s| s.handle_click(position, window, change.timestamp));
                    match kind {
                        ClickType::Single if event.key_modifiers.shift_key() => {
                            controller.with_mut(|s| s.extend_selection_to_point(position));
                        }
                        ClickType::Single => controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::Click {
                                x: position.x_px().0,
                                y: position.y_px().0,
                            });
                        }),
                        ClickType::Double => controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::DoubleClick {
                                x: position.x_px().0,
                                y: position.y_px().0,
                            });
                        }),
                        ClickType::Triple => controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::TripleClick {
                                x: position.x_px().0,
                                y: position.y_px().0,
                            });
                        }),
                    }
                    controller.with_mut(|s| s.start_drag(kind));
                    pressed = true;
                    active = Some((change.pointer_id, position, kind, true));
                    change.consume();
                }
                CursorEventContent::Moved(_) if controller.with(|s| s.is_dragging()) => {
                    if let (Some(cursor), Some((pointer, _, _, _))) = (cursor, active)
                        && pointer == change.pointer_id
                    {
                        let target = drag_selection_pointer_position(
                            cursor,
                            &args,
                            &controller,
                            ComputedData {
                                width: event.size.width,
                                height: event.size.height,
                            },
                        );
                        auto_scroll_drag_selection(&target, &controller);
                        let target = drag_selection_pointer_position(
                            cursor,
                            &args,
                            &controller,
                            ComputedData {
                                width: event.size.width,
                                height: event.size.height,
                            },
                        );
                        if controller.with(|s| s.last_click_position()) != Some(target.position) {
                            controller.with_mut(|s| {
                                s.apply_pointer_action(GlyphonAction::Drag {
                                    x: target.position.x_px().0,
                                    y: target.position.y_px().0,
                                });
                                s.update_last_click_position(target.position);
                            });
                        }
                        change.consume();
                    }
                }
                CursorEventContent::Released(_) | CursorEventContent::Cancelled(_)
                    if active.is_some_and(|(pointer, _, _, _)| pointer == change.pointer_id) =>
                {
                    controller.with_mut(|s| s.stop_drag());
                    active = None;
                    change.consume();
                }
                CursorEventContent::Scroll(scroll) if inside && args.enabled => {
                    let (dx, dy) = scroll.pixel_delta();
                    let mut remainder = scroll;
                    if controller.with(|s| s.single_line()) {
                        let requested = if dx.abs() > f32::EPSILON { dx } else { dy };
                        if requested.abs() > f32::EPSILON
                            && controller.with_mut(|s| s.has_horizontal_overflow())
                        {
                            let consumed =
                                controller.with_mut(|s| s.scroll_horizontal_consumed(-requested));
                            if dx.abs() > f32::EPSILON {
                                remainder.delta_x = dx + consumed;
                                remainder.delta_y = dy;
                            } else {
                                remainder.delta_y = dy + consumed;
                                remainder.delta_x = dx;
                            }
                            remainder.unit = tessera_ui::ScrollDeltaUnit::Pixel;
                        }
                    } else if dy.abs() > f32::EPSILON
                        && controller.with_mut(|s| s.has_vertical_overflow())
                    {
                        let consumed = controller.with_mut(|s| s.scroll_vertical_consumed(-dy));
                        remainder.delta_y = dy + consumed;
                        remainder.delta_x = dx;
                        remainder.unit = tessera_ui::ScrollDeltaUnit::Pixel;
                    }
                    event.set_scroll_remainder(change.pointer_id, remainder);
                }
                _ => {}
            }
        }
        let focused = controller.with(|s| s.focus_handler().is_focused());
        if focused && inside {
            event.block_cursor();
        }
        if args.enabled && !args.read_only && focused {
            let request = current_ime_request(
                &args,
                &controller,
                ComputedData {
                    width: event.size.width,
                    height: event.size.height,
                },
            );
            if pressed {
                event.show_ime(request);
            } else {
                event.update_ime(request);
            }
        }
    }
}

fn click_selection_pointer_position(
    cursor_pos: PxPosition,
    args: &TextEditInput,
    controller: &State<TextEditorController>,
    size: ComputedData,
) -> PxPosition {
    drag_selection_pointer_position(cursor_pos, args, controller, size).position
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DragSelectionPointerPosition {
    position: PxPosition,
    overflow_x: Px,
    overflow_y: Px,
}

fn drag_selection_pointer_position(
    cursor_pos: PxPosition,
    args: &TextEditInput,
    controller: &State<TextEditorController>,
    size: ComputedData,
) -> DragSelectionPointerPosition {
    let (horizontal_scroll, vertical_scroll) = controller.with(|s| {
        (
            Px(s.scroll_controller().horizontal().round() as i32),
            Px(s.scroll_controller().vertical().round() as i32),
        )
    });
    let text_origin = args.content_insets.origin();
    drag_selection_pointer_position_with_scroll(
        cursor_pos,
        size,
        text_origin,
        horizontal_scroll,
        vertical_scroll,
    )
}

fn drag_selection_pointer_position_with_scroll(
    cursor_pos: PxPosition,
    size: ComputedData,
    text_origin: PxPosition,
    horizontal_scroll: Px,
    vertical_scroll: Px,
) -> DragSelectionPointerPosition {
    let viewport_size = text_viewport_size_from_origin(size, text_origin);
    let viewport_width = viewport_size.width.0 as f32;
    let viewport_height = viewport_size.height.0 as f32;
    let text_relative_x = cursor_pos.x - text_origin.x;
    let text_relative_y = cursor_pos.y - text_origin.y;
    let overflow_x = if text_relative_x < 0.0 {
        Px(text_relative_x.round() as i32)
    } else if text_relative_x > viewport_width {
        Px((text_relative_x - viewport_width).round() as i32)
    } else {
        Px::ZERO
    };
    let overflow_y = if text_relative_y < 0.0 {
        Px(text_relative_y.round() as i32)
    } else if text_relative_y > viewport_height {
        Px((text_relative_y - viewport_height).round() as i32)
    } else {
        Px::ZERO
    };
    let clamped_x = text_relative_x.clamp(0.0, viewport_width);
    let clamped_y = text_relative_y.clamp(0.0, viewport_height);

    DragSelectionPointerPosition {
        position: PxPosition::from_f32_xy(
            clamped_x + horizontal_scroll.0 as f32,
            clamped_y + vertical_scroll.0 as f32,
        ),
        overflow_x,
        overflow_y,
    }
}

fn text_viewport_size_from_origin(size: ComputedData, text_origin: PxPosition) -> ComputedData {
    let width = (size.width.0 as f32 - text_origin.x * 2.0).round() as i32;
    let height = (size.height.0 as f32 - text_origin.y * 2.0).round() as i32;
    ComputedData {
        width: Px(width).max(Px::ZERO),
        height: Px(height).max(Px::ZERO),
    }
}

fn auto_scroll_drag_selection(
    drag_target: &DragSelectionPointerPosition,
    controller: &State<TextEditorController>,
) {
    controller.with_mut(|s| {
        if s.single_line() {
            if drag_target.overflow_x != Px::ZERO {
                s.scroll_horizontal_by(drag_target.overflow_x.to_f32());
            }
            return;
        }
        if drag_target.overflow_y != Px::ZERO {
            s.scroll_vertical_by(drag_target.overflow_y.to_f32());
        }
    });
}

fn handle_editor_escape(
    controller: &mut TextEditorController,
    state: winit::event::ElementState,
    on_change: CallbackWith<String, String>,
    input_transform: Option<CallbackWith<String, String>>,
) -> bool {
    if state != winit::event::ElementState::Pressed
        || (controller.composition().is_none() && !controller.has_selection())
    {
        return false;
    }
    controller.apply_action_with_pipeline(GlyphonAction::Escape, on_change, input_transform);
    true
}

fn handle_text_input_keyboard(
    input: &mut tessera_ui::KeyboardInput<'_>,
    args: &TextEditInput,
    controller: &State<TextEditorController>,
    on_change: &CallbackWith<String, String>,
    input_transform: &Option<CallbackWith<String, String>>,
) {
    if !args.enabled || !controller.with(|s| s.focus_handler().is_focused()) {
        return;
    }

    let is_ctrl = input.key_modifiers.control_key() || input.key_modifiers.super_key();
    let is_shift = input.key_modifiers.shift_key();
    let mut all_actions = Vec::new();
    let mut should_block_keyboard = false;
    for key_event in input.keyboard_events.iter().cloned() {
        if key_event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) {
            should_block_keyboard |= controller.with_mut(|controller| {
                handle_editor_escape(controller, key_event.state, *on_change, *input_transform)
            });
            continue;
        }
        if let Some(behavior) =
            single_line_key_behavior(args.single_line, key_event.state, &key_event.logical_key)
        {
            match behavior {
                SingleLineKeyBehavior::Submit => {
                    if args.on_submit != Callback::noop() {
                        args.on_submit.call();
                        should_block_keyboard = true;
                    }
                }
                SingleLineKeyBehavior::Propagate => {}
            }
            continue;
        }

        if let Some(behavior) =
            clipboard_shortcut_for_key(is_ctrl, is_shift, key_event.state, &key_event.logical_key)
        {
            should_block_keyboard = true;
            #[cfg(not(target_family = "wasm"))]
            match behavior {
                ClipboardShortcutBehavior::Copy => {
                    controller.with(|s| {
                        s.copy_selection_to_clipboard();
                    });
                }
                ClipboardShortcutBehavior::Cut => {
                    if !args.read_only {
                        controller.with_mut(|s| {
                            s.cut_selection_with_pipeline(*on_change, *input_transform);
                        });
                    }
                }
                ClipboardShortcutBehavior::Paste => {
                    if !args.read_only {
                        controller.with_mut(|s| {
                            s.paste_from_clipboard_with_pipeline(*on_change, *input_transform);
                        });
                    }
                }
            }
            #[cfg(target_family = "wasm")]
            let _ = behavior;
            continue;
        }

        if let Some(motion) =
            deletion_motion_for_key(is_ctrl, key_event.state, &key_event.logical_key)
        {
            should_block_keyboard = true;
            if !args.read_only {
                controller.with_mut(|s| {
                    s.delete_motion_with_pipeline(motion, *on_change, *input_transform);
                });
            }
            continue;
        }

        should_block_keyboard = true;
        let shortcut = if let winit::keyboard::Key::Character(s) = &key_event.logical_key {
            if is_ctrl && key_event.state == winit::event::ElementState::Pressed {
                Some(s.to_lowercase())
            } else {
                None
            }
        } else {
            None
        };

        match shortcut.as_deref() {
            Some("a") => {
                controller.with_mut(|s| s.select_all());
                continue;
            }
            Some("c") => {
                controller.with(|s| {
                    s.copy_selection_to_clipboard();
                });
                continue;
            }
            Some("x") => {
                if !args.read_only {
                    controller.with_mut(|s| {
                        s.cut_selection_with_pipeline(*on_change, *input_transform);
                    });
                }
                continue;
            }
            Some("v") => {
                if !args.read_only {
                    controller.with_mut(|s| {
                        s.paste_from_clipboard_with_pipeline(*on_change, *input_transform);
                    });
                }
                continue;
            }
            _ => {}
        }

        controller.with_mut(|s| {
            if let Some(actions) = s.map_key_event_to_action(key_event, input.key_modifiers) {
                all_actions.extend(actions);
            }
        });
    }
    if !all_actions.is_empty() {
        if args.read_only {
            all_actions.retain(|action| !is_editing_action(action));
        }
        for action in all_actions {
            handle_action(controller, action, *on_change, *input_transform);
        }
    }

    if !args.read_only && controller.with(|s| s.focus_handler().is_focused()) {
        let computed_data = input.computed_data;
        input
            .ime_session()
            .update(current_ime_request(args, controller, computed_data));
    }

    if should_block_keyboard {
        input.block_keyboard();
    }
}

fn handle_text_input_ime(
    input: &mut tessera_ui::ImeInput<'_>,
    args: &TextEditInput,
    controller: &State<TextEditorController>,
    on_change: &CallbackWith<String, String>,
    input_transform: &Option<CallbackWith<String, String>>,
) {
    if !args.enabled || !controller.with(|s| s.focus_handler().is_focused()) {
        return;
    }
    if args.read_only {
        input.block_ime();
        return;
    }

    for event in input.ime_events.drain(..) {
        if !controller.with(|controller| controller.focus_handler().is_focused()) {
            break;
        }
        let direction = match event {
            winit::event::Ime::Action(winit::event::ImeAction::Next) => {
                Some(tessera_ui::FocusDirection::Next)
            }
            winit::event::Ime::Action(winit::event::ImeAction::Previous) => {
                Some(tessera_ui::FocusDirection::Previous)
            }
            _ => None,
        };
        if let Some(direction) = direction {
            tessera_ui::focus::request_move_focus(direction);
            break;
        }
        let Some(plan) = controller.with(|c| c.plan_ime_event(args.single_line, &event)) else {
            continue;
        };
        match plan {
            PlannedImeEvent::Submit => {
                controller.with_mut(|c| c.clear_composition());
                if args.on_submit != Callback::noop() {
                    args.on_submit.call();
                }
            }
            PlannedImeEvent::Edit(plan) => {
                let result = replace_text_range_with_selection(
                    controller,
                    plan.replacement_range.clone(),
                    &plan.replacement_text,
                    plan.selection.clone(),
                    *on_change,
                    *input_transform,
                );
                controller.with_mut(|c| c.commit_ime_edit(&plan, &result));
            }
        }
    }

    let computed_data = input.computed_data;
    input
        .ime_session()
        .update(current_ime_request(args, controller, computed_data));
    input.block_ime();
}

fn current_ime_request(
    args: &TextEditInput,
    controller: &State<TextEditorController>,
    computed_data: ComputedData,
) -> ImeRequest {
    let (ime_rect, state) = controller.with(|c| {
        (
            c.current_ime_rect(),
            winit::event::ImeTextState {
                text: c.text(),
                selection: c.selection().start..c.selection().end,
                composition: c.composition().map(|composition| composition.range.clone()),
            },
        )
    });
    build_ime_request(args.content_insets.origin(), computed_data, ime_rect)
        .with_text_state(state)
        .with_options(winit::window::ImeOptions {
            purpose: if args.is_secure {
                winit::window::ImePurpose::Password
            } else {
                winit::window::ImePurpose::Normal
            },
            action: if args.single_line {
                winit::event::ImeAction::Submit
            } else {
                winit::event::ImeAction::Newline
            },
        })
}

fn build_ime_request(
    text_origin: PxPosition,
    computed_data: ComputedData,
    ime_rect: Option<RectDef>,
) -> ImeRequest {
    ime_rect
        .map(|ime_rect| {
            ImeRequest::new(PxSize::new(ime_rect.width, ime_rect.height))
                .with_local_position(text_origin + PxPosition::new(ime_rect.x, ime_rect.y))
        })
        .unwrap_or_else(|| ImeRequest::new(computed_data.into()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SingleLineKeyBehavior {
    Submit,
    Propagate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClipboardShortcutBehavior {
    Copy,
    Cut,
    Paste,
}

fn deletion_motion_for_key(
    is_ctrl: bool,
    key_state: winit::event::ElementState,
    logical_key: &winit::keyboard::Key,
) -> Option<glyphon::cosmic_text::Motion> {
    if !is_ctrl || key_state != winit::event::ElementState::Pressed {
        return None;
    }

    match logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) => {
            Some(glyphon::cosmic_text::Motion::LeftWord)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete) => {
            Some(glyphon::cosmic_text::Motion::RightWord)
        }
        _ => None,
    }
}

fn clipboard_shortcut_for_key(
    is_ctrl: bool,
    is_shift: bool,
    key_state: winit::event::ElementState,
    logical_key: &winit::keyboard::Key,
) -> Option<ClipboardShortcutBehavior> {
    if key_state != winit::event::ElementState::Pressed {
        return None;
    }

    match logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert) if is_ctrl => {
            Some(ClipboardShortcutBehavior::Copy)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert) if is_shift => {
            Some(ClipboardShortcutBehavior::Paste)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete) if is_shift => {
            Some(ClipboardShortcutBehavior::Cut)
        }
        _ => None,
    }
}

fn single_line_key_behavior(
    single_line: bool,
    key_state: winit::event::ElementState,
    logical_key: &winit::keyboard::Key,
) -> Option<SingleLineKeyBehavior> {
    if !single_line || key_state != winit::event::ElementState::Pressed {
        return None;
    }

    match logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => {
            Some(SingleLineKeyBehavior::Submit)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
            Some(SingleLineKeyBehavior::Propagate)
        }
        _ => None,
    }
}

fn handle_action(
    state: &State<TextEditorController>,
    action: GlyphonAction,
    on_change: CallbackWith<String, String>,
    input_transform: Option<CallbackWith<String, String>>,
) {
    state.with_mut(|c| c.apply_action_with_pipeline(action, on_change, input_transform));
}

fn replace_text_range_with_selection(
    state: &State<TextEditorController>,
    range: std::ops::Range<usize>,
    replacement: &str,
    selection: TextSelection,
    on_change: CallbackWith<String, String>,
    input_transform: Option<CallbackWith<String, String>>,
) -> ImeEditResult {
    state.with_mut(|c| {
        c.replace_text_range_with_pipeline(
            range,
            replacement,
            selection,
            on_change,
            input_transform,
        )
    })
}

struct TextInputSemanticsModifierNode {
    args: TextEditInput,
    controller: State<TextEditorController>,
}

impl SemanticsModifierNode for TextInputSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        apply_text_input_semantics(accessibility, action_handler, &self.args, &self.controller);
    }
}

fn is_editing_action(action: &GlyphonAction) -> bool {
    matches!(
        action,
        GlyphonAction::Insert(_)
            | GlyphonAction::Backspace
            | GlyphonAction::Delete
            | GlyphonAction::Enter
    )
}

fn apply_text_input_semantics(
    accessibility: &mut AccessibilityNode,
    action_handler: &mut Option<AccessibilityActionHandler>,
    args: &TextEditInput,
    state: &State<TextEditorController>,
) {
    let focus = state.with(|c| *c.focus_handler());
    let submit_action_enabled = should_expose_submit_accessibility_action(
        args.single_line,
        focus.is_focused(),
        args.on_submit != Callback::noop(),
    );
    accessibility.role = Some(if args.is_secure {
        Role::PasswordInput
    } else {
        text_input_accessibility_role(args.single_line)
    });
    accessibility.disabled = !args.enabled;
    accessibility.label = args.accessibility_label.clone();
    accessibility.description = args.accessibility_description.clone();

    let current_text = state.with(|c| c.text());
    accessibility.value = (!args.is_secure && !current_text.is_empty()).then_some(current_text);

    accessibility.is_editable_text = args.enabled && !args.read_only;
    accessibility.focusable = args.enabled;
    accessibility.actions.clear();
    if args.enabled && submit_action_enabled {
        accessibility.actions.push(Action::Click);
    }

    if args.enabled {
        let on_submit = args.on_submit;
        *action_handler = Some(Box::new(move |action| match action {
            Action::Focus => focus.request_focus(),
            Action::Blur => focus.clear_focus(),
            Action::Click => {
                if focus.is_focused() && on_submit != Callback::noop() {
                    on_submit.call();
                } else {
                    focus.request_focus();
                }
            }
            _ => {}
        }));
    } else {
        *action_handler = None;
    }
}

fn text_input_accessibility_role(single_line: bool) -> Role {
    if single_line {
        Role::TextInput
    } else {
        Role::MultilineTextInput
    }
}

fn should_expose_submit_accessibility_action(
    single_line: bool,
    focused: bool,
    has_submit_handler: bool,
) -> bool {
    single_line && focused && has_submit_handler
}

#[cfg(test)]
#[path = "text_edit_input_tests.rs"]
mod tests;
