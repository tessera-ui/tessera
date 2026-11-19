use std::time::Duration;

use glyphon::{Action as GlyphonAction, Edit as _};
use tessera_ui::{
    CallbackWith, ComputedData, Dp, Px, PxPosition, PxSize, accesskit::Role, time::Instant, winit,
};

use crate::text_edit_core::{
    ClickType, ImeComposition, PlannedImeEdit, PlannedImeEvent, RectDef, TextEditorController,
    TextSelection,
};

use super::{
    ClipboardShortcutBehavior, DragSelectionPointerPosition, SingleLineKeyBehavior, TextEditInsets,
    build_ime_request, clipboard_shortcut_for_key, deletion_motion_for_key,
    drag_selection_pointer_position_with_scroll, handle_editor_escape,
    should_expose_submit_accessibility_action, single_line_key_behavior,
    text_input_accessibility_role, text_viewport_size_from_origin,
};

#[test]
fn escape_disposition_leaves_a_plain_caret_unhandled_without_mutating_text() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let mut controller = TextEditorController::new(Dp(16.0), Some(Dp(24.0)));
    controller.set_text_and_selection("café", TextSelection::collapsed(5));
    assert!(!handle_editor_escape(
        &mut controller,
        winit::event::ElementState::Pressed,
        CallbackWith::identity(),
        None,
    ));
    assert_eq!(controller.text(), "café");
    assert_eq!(controller.selection(), TextSelection::collapsed(5));
    assert!(controller.composition().is_none());
}

#[test]
fn escape_disposition_cancels_a_nonempty_selection_once_without_editing_text() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let mut controller = TextEditorController::new(Dp(16.0), Some(Dp(24.0)));
    controller.set_text_and_selection("café", TextSelection { start: 0, end: 3 });
    assert_eq!(controller.selection(), TextSelection { start: 0, end: 3 });
    assert!(handle_editor_escape(
        &mut controller,
        winit::event::ElementState::Pressed,
        CallbackWith::identity(),
        None,
    ));
    assert_eq!(controller.text(), "café");
    assert_eq!(controller.selection(), TextSelection::collapsed(3));
    assert!(controller.composition().is_none());
    assert!(!handle_editor_escape(
        &mut controller,
        winit::event::ElementState::Pressed,
        CallbackWith::identity(),
        None,
    ));
}

#[test]
fn escape_disposition_clears_active_composition_and_selection_before_becoming_unhandled() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let mut controller = TextEditorController::new(Dp(16.0), Some(Dp(24.0)));
    controller.set_text_and_selection("preedit", TextSelection { start: 2, end: 5 });
    controller.set_composition(Some(ImeComposition {
        range: 0..7,
        selection: TextSelection { start: 2, end: 5 },
    }));
    assert!(handle_editor_escape(
        &mut controller,
        winit::event::ElementState::Pressed,
        CallbackWith::identity(),
        None,
    ));
    assert_eq!(controller.text(), "preedit");
    assert_eq!(controller.selection(), TextSelection::collapsed(5));
    assert!(controller.composition().is_none());
    assert!(!handle_editor_escape(
        &mut controller,
        winit::event::ElementState::Pressed,
        CallbackWith::identity(),
        None,
    ));
}

#[test]
fn escape_release_does_not_cancel_an_owned_selection_or_composition() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let mut controller = TextEditorController::new(Dp(16.0), Some(Dp(24.0)));
    let selection = TextSelection { start: 0, end: 3 };
    let composition = ImeComposition {
        range: 0..3,
        selection: selection.clone(),
    };
    controller.set_text_and_selection("abc", selection.clone());
    controller.set_composition(Some(composition.clone()));
    assert!(!handle_editor_escape(
        &mut controller,
        winit::event::ElementState::Released,
        CallbackWith::identity(),
        None,
    ));
    assert_eq!(controller.text(), "abc");
    assert_eq!(controller.selection(), selection);
    assert_eq!(controller.composition(), Some(&composition));
}

fn plan_ime_event_for_state(
    selection: TextSelection,
    composition: Option<ImeComposition>,
    single_line: bool,
    event: &winit::event::Ime,
) -> Option<PlannedImeEvent> {
    let text_len = selection.ordered_range().end.max(
        composition
            .as_ref()
            .map(|composition| composition.range.end)
            .unwrap_or(0),
    );
    let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
    controller.set_text_and_selection(&"x".repeat(text_len), selection);
    controller.set_composition(composition);
    controller.plan_ime_event(single_line, event)
}

fn pointer_position_for_offset(text: &str, offset: usize) -> PxPosition {
    let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
    controller.set_text_and_selection(text, TextSelection::collapsed(offset));
    let (x, y) = controller
        .editor()
        .cursor_position()
        .expect("cursor position should be available");
    PxPosition::new(Px(x), Px(y))
}

#[test]
fn drag_selection_pointer_position_clamps_and_reports_overflow() {
    let text_origin = TextEditInsets {
        horizontal: Dp(8.0),
        vertical: Dp(8.0),
        border: Dp(1.0),
    }
    .origin();
    let size = ComputedData {
        width: Px(120),
        height: Px(40),
    };

    let drag_target = drag_selection_pointer_position_with_scroll(
        PxPosition::new(Px(-4), Px(60)),
        size,
        text_origin,
        Px::ZERO,
        Px::ZERO,
    );

    assert_eq!(
        drag_target,
        DragSelectionPointerPosition {
            position: PxPosition::new(Px::ZERO, Px(22)),
            overflow_x: Px(-13),
            overflow_y: Px(29),
        }
    );
}

#[test]
fn drag_selection_pointer_position_applies_vertical_scroll_to_clamped_position() {
    let text_origin = TextEditInsets {
        horizontal: Dp(8.0),
        vertical: Dp(8.0),
        border: Dp(1.0),
    }
    .origin();
    let size = ComputedData {
        width: Px(120),
        height: Px(40),
    };

    let drag_target = drag_selection_pointer_position_with_scroll(
        PxPosition::new(Px(20), Px(20)),
        size,
        text_origin,
        Px(5),
        Px(11),
    );

    assert_eq!(
        drag_target,
        DragSelectionPointerPosition {
            position: PxPosition::new(Px(16), Px(22)),
            overflow_x: Px::ZERO,
            overflow_y: Px::ZERO,
        }
    );
}

#[test]
fn click_selection_pointer_position_clamps_inside_text_viewport() {
    let text_origin = TextEditInsets {
        horizontal: Dp(8.0),
        vertical: Dp(8.0),
        border: Dp(1.0),
    }
    .origin();
    let size = ComputedData {
        width: Px(120),
        height: Px(40),
    };

    let click_position = drag_selection_pointer_position_with_scroll(
        PxPosition::new(Px(2), Px(4)),
        size,
        text_origin,
        Px::ZERO,
        Px::ZERO,
    )
    .position;

    assert_eq!(click_position, PxPosition::new(Px::ZERO, Px::ZERO));
}

#[test]
fn text_viewport_size_removes_padding_and_border_from_surface() {
    let viewport = text_viewport_size_from_origin(
        ComputedData {
            width: Px(120),
            height: Px(40),
        },
        TextEditInsets {
            horizontal: Dp(8.0),
            vertical: Dp(8.0),
            border: Dp(1.0),
        }
        .origin(),
    );

    assert_eq!(viewport.width, Px(102));
    assert_eq!(viewport.height, Px(22));
}

#[test]
fn plan_ime_edit_preedit_reuses_existing_composition_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let composition = ImeComposition {
        range: 2..5,
        selection: TextSelection::collapsed(5),
    };

    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(5),
            Some(composition),
            false,
            &winit::event::Ime::Preedit("xy".to_string(), Some((1, 1))),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 2..5,
            replacement_text: "xy".to_string(),
            selection: TextSelection::collapsed(3),
            composition_range: Some(2..4),
        }))
    );
}

#[test]
fn plan_ime_edit_empty_preedit_clears_composition_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let composition = ImeComposition {
        range: 2..5,
        selection: TextSelection::collapsed(5),
    };

    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(5),
            Some(composition),
            false,
            &winit::event::Ime::Preedit(String::new(), None),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 2..5,
            replacement_text: String::new(),
            selection: TextSelection::collapsed(2),
            composition_range: None,
        }))
    );
}

#[test]
fn plan_ime_edit_preedit_without_cursor_offset_preserves_previous_composition_selection() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let composition = ImeComposition {
        range: 2..5,
        selection: TextSelection { start: 3, end: 4 },
    };

    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(5),
            Some(composition),
            false,
            &winit::event::Ime::Preedit("xyzq".to_string(), None),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 2..5,
            replacement_text: "xyzq".to_string(),
            selection: TextSelection { start: 3, end: 4 },
            composition_range: Some(2..6),
        }))
    );
}

#[test]
fn plan_ime_edit_commit_replaces_current_composition_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let composition = ImeComposition {
        range: 3..6,
        selection: TextSelection::collapsed(6),
    };

    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(6),
            Some(composition),
            false,
            &winit::event::Ime::Commit("z".to_string()),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 3..6,
            replacement_text: "z".to_string(),
            selection: TextSelection::collapsed(4),
            composition_range: None,
        }))
    );
}

#[test]
fn plan_ime_edit_commit_without_composition_replaces_current_selection_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    assert_eq!(
        plan_ime_event_for_state(
            TextSelection { start: 6, end: 3 },
            None,
            false,
            &winit::event::Ime::Commit("z".to_string()),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 3..6,
            replacement_text: "z".to_string(),
            selection: TextSelection::collapsed(4),
            composition_range: None,
        }))
    );
}

#[test]
fn plan_ime_edit_preedit_tracks_full_composition_range_instead_of_caret_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(1),
            None,
            false,
            &winit::event::Ime::Preedit("abcd".to_string(), Some((2, 2))),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 1..1,
            replacement_text: "abcd".to_string(),
            selection: TextSelection::collapsed(3),
            composition_range: Some(1..5),
        }))
    );
}

#[test]
fn text_input_pointer_sequence_single_click_drag_selects_backward_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let text = "hello world";
    let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
    controller.set_text_and_selection(text, TextSelection::collapsed(6));
    let start_position = pointer_position_for_offset(text, 6);
    let drag_position = pointer_position_for_offset(text, 1);
    let click_time = Instant::now();

    let click_type = controller.handle_click(start_position, start_position, click_time);
    assert!(matches!(click_type, ClickType::Single));
    controller.apply_pointer_action(GlyphonAction::Click {
        x: start_position.x_px().0,
        y: start_position.y_px().0,
    });
    controller.start_drag(click_type);
    controller.apply_pointer_action(GlyphonAction::Drag {
        x: drag_position.x_px().0,
        y: drag_position.y_px().0,
    });

    assert_eq!(controller.selection(), TextSelection { start: 6, end: 1 });
    assert_eq!(controller.cursor_offset(), 1);
}

#[test]
fn text_input_pointer_sequence_double_click_drag_selects_word_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let text = "foo bar baz";
    let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
    controller.set_text_and_selection(text, TextSelection::collapsed(5));
    let first_position = pointer_position_for_offset(text, 5);
    let drag_position = pointer_position_for_offset(text, 9);
    let first_click_time = Instant::now();
    let second_click_time = first_click_time + Duration::from_millis(100);

    let first_click_type =
        controller.handle_click(first_position, first_position, first_click_time);
    assert!(matches!(first_click_type, ClickType::Single));
    controller.apply_pointer_action(GlyphonAction::Click {
        x: first_position.x_px().0,
        y: first_position.y_px().0,
    });

    let second_click_type =
        controller.handle_click(first_position, first_position, second_click_time);
    assert!(matches!(second_click_type, ClickType::Double));
    controller.apply_pointer_action(GlyphonAction::DoubleClick {
        x: first_position.x_px().0,
        y: first_position.y_px().0,
    });
    controller.start_drag(second_click_type);
    controller.apply_pointer_action(GlyphonAction::Drag {
        x: drag_position.x_px().0,
        y: drag_position.y_px().0,
    });

    assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
    assert_eq!(controller.cursor_offset(), 11);
}

#[test]
fn text_input_pointer_sequence_triple_click_drag_selects_line_range() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    let text = "111\n222\n333";
    let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
    controller.set_text_and_selection(text, TextSelection::collapsed(5));
    let line_position = pointer_position_for_offset(text, 5);
    let drag_position = pointer_position_for_offset(text, 9);
    let first_click_time = Instant::now();
    let second_click_time = first_click_time + Duration::from_millis(100);
    let third_click_time = second_click_time + Duration::from_millis(100);

    let first_click_type = controller.handle_click(line_position, line_position, first_click_time);
    assert!(matches!(first_click_type, ClickType::Single));
    controller.apply_pointer_action(GlyphonAction::Click {
        x: line_position.x_px().0,
        y: line_position.y_px().0,
    });

    let second_click_type =
        controller.handle_click(line_position, line_position, second_click_time);
    assert!(matches!(second_click_type, ClickType::Double));
    controller.apply_pointer_action(GlyphonAction::DoubleClick {
        x: line_position.x_px().0,
        y: line_position.y_px().0,
    });

    let third_click_type = controller.handle_click(line_position, line_position, third_click_time);
    assert!(matches!(third_click_type, ClickType::Triple));
    controller.apply_pointer_action(GlyphonAction::TripleClick {
        x: line_position.x_px().0,
        y: line_position.y_px().0,
    });
    controller.start_drag(third_click_type);
    controller.apply_pointer_action(GlyphonAction::Drag {
        x: drag_position.x_px().0,
        y: drag_position.y_px().0,
    });

    assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
    assert_eq!(controller.cursor_offset(), 11);
}

#[test]
fn single_line_enter_uses_submit_behavior() {
    assert_eq!(
        single_line_key_behavior(
            true,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
        ),
        Some(SingleLineKeyBehavior::Submit)
    );
}

#[test]
fn single_line_tab_propagates_focus_navigation() {
    assert_eq!(
        single_line_key_behavior(
            true,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
        ),
        Some(SingleLineKeyBehavior::Propagate)
    );
}

#[test]
fn multi_line_enter_keeps_default_editing_behavior() {
    assert_eq!(
        single_line_key_behavior(
            false,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
        ),
        None
    );
}

#[test]
fn released_enter_does_not_trigger_single_line_submit_behavior() {
    assert_eq!(
        single_line_key_behavior(
            true,
            winit::event::ElementState::Released,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
        ),
        None
    );
}

#[test]
fn single_line_ime_commit_newline_uses_submit_behavior() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(0),
            None,
            true,
            &winit::event::Ime::Commit("\n".to_string()),
        ),
        Some(PlannedImeEvent::Submit)
    );
    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(0),
            None,
            true,
            &winit::event::Ime::Commit("\r\n".to_string()),
        ),
        Some(PlannedImeEvent::Submit)
    );
}

#[test]
fn multi_line_ime_commit_newline_keeps_default_editing_behavior() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(0),
            None,
            false,
            &winit::event::Ime::Commit("\n".to_string()),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 0..0,
            replacement_text: "\n".to_string(),
            selection: TextSelection::collapsed(1),
            composition_range: None,
        }))
    );
}

#[test]
fn single_line_ime_commit_text_does_not_trigger_submit_behavior() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    assert_eq!(
        plan_ime_event_for_state(
            TextSelection::collapsed(0),
            None,
            true,
            &winit::event::Ime::Commit("done".to_string()),
        ),
        Some(PlannedImeEvent::Edit(PlannedImeEdit {
            replacement_range: 0..0,
            replacement_text: "done".to_string(),
            selection: TextSelection::collapsed(4),
            composition_range: None,
        }))
    );
}

#[test]
fn non_submit_single_line_commit_keeps_edit_plan_instead_of_submit_plan() {
    let runtime = tessera_ui::RuntimeSession::new();
    let _scope = runtime.enter();
    assert!(matches!(
        plan_ime_event_for_state(
            TextSelection::collapsed(0),
            Some(ImeComposition {
                range: 1..4,
                selection: TextSelection::collapsed(3),
            }),
            true,
            &winit::event::Ime::Commit("done".to_string()),
        ),
        Some(PlannedImeEvent::Edit(_))
    ));
}

#[test]
fn ctrl_backspace_uses_left_word_deletion_motion() {
    assert_eq!(
        deletion_motion_for_key(
            true,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace),
        ),
        Some(glyphon::cosmic_text::Motion::LeftWord)
    );
}

#[test]
fn ctrl_delete_uses_right_word_deletion_motion() {
    assert_eq!(
        deletion_motion_for_key(
            true,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
        ),
        Some(glyphon::cosmic_text::Motion::RightWord)
    );
}

#[test]
fn ctrl_insert_uses_copy_clipboard_shortcut_behavior() {
    assert_eq!(
        clipboard_shortcut_for_key(
            true,
            false,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
        ),
        Some(ClipboardShortcutBehavior::Copy)
    );
}

#[test]
fn shift_insert_uses_paste_clipboard_shortcut_behavior() {
    assert_eq!(
        clipboard_shortcut_for_key(
            false,
            true,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
        ),
        Some(ClipboardShortcutBehavior::Paste)
    );
}

#[test]
fn shift_delete_uses_cut_clipboard_shortcut_behavior() {
    assert_eq!(
        clipboard_shortcut_for_key(
            false,
            true,
            winit::event::ElementState::Pressed,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
        ),
        Some(ClipboardShortcutBehavior::Cut)
    );
}

#[test]
fn released_insert_does_not_trigger_clipboard_shortcut_behavior() {
    assert_eq!(
        clipboard_shortcut_for_key(
            true,
            false,
            winit::event::ElementState::Released,
            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
        ),
        None
    );
}

#[test]
fn single_line_accessibility_uses_text_input_role() {
    assert_eq!(text_input_accessibility_role(true), Role::TextInput);
}

#[test]
fn multi_line_accessibility_uses_multiline_role() {
    assert_eq!(
        text_input_accessibility_role(false),
        Role::MultilineTextInput
    );
}

#[test]
fn submit_accessibility_action_requires_focused_single_line_handler() {
    assert!(should_expose_submit_accessibility_action(true, true, true,));
    assert!(!should_expose_submit_accessibility_action(
        false, true, true,
    ));
    assert!(!should_expose_submit_accessibility_action(
        true, false, true,
    ));
    assert!(!should_expose_submit_accessibility_action(
        true, true, false
    ));
}

#[test]
fn build_ime_request_uses_active_ime_rect_when_available() {
    let ime_request = build_ime_request(
        PxPosition::new(Px(9), Px(11)),
        ComputedData {
            width: Px(120),
            height: Px(40),
        },
        Some(RectDef {
            x: Px(3),
            y: Px(5),
            width: Px(7),
            height: Px(13),
        }),
    );

    assert_eq!(ime_request.local_position, PxPosition::new(Px(12), Px(16)));
    assert_eq!(ime_request.size, PxSize::new(Px(7), Px(13)));
}

#[test]
fn build_ime_request_falls_back_to_component_bounds_without_active_rect() {
    let ime_request = build_ime_request(
        PxPosition::new(Px(9), Px(11)),
        ComputedData {
            width: Px(120),
            height: Px(40),
        },
        None,
    );

    assert_eq!(ime_request.local_position, PxPosition::ZERO);
    assert_eq!(ime_request.size, PxSize::new(Px(120), Px(40)));
}
