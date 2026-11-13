//! Dialog action layout regressions.
//!
//! ## Usage
//!
//! Keep dialog actions visible when available width changes.

use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, Dp, Modifier, State,
    accesskit::{Node, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::md3::{
    button::button,
    dialog::{DialogController, DialogRole, basic_dialog, dialog_provider, full_screen_dialog},
    text::text,
    theme::material_theme,
};

#[tessera]
fn dialog_action_fixture() {
    material_theme().child(|| {
        dialog_provider()
            .accessibility_label("Confirmation")
            .controller(remember(|| DialogController::new(true)))
            .dialog_content(|dialog| {
                basic_dialog()
                    .dialog(dialog)
                    .headline("Confirmation")
                    .supporting_text("Choose an action.")
                    .dismiss_button(|| {
                        button()
                            .text()
                            .modifier(Modifier::new().width(Dp(160.0)).semantics(SemanticsArgs {
                                test_tag: Some("dismiss".into()),
                                ..Default::default()
                            }))
                            .accessibility_label("Dismiss action")
                            .on_click(|| {})
                            .child(|| {
                                text("Dismiss action");
                            });
                    })
                    .confirm_button(|| {
                        button()
                            .text()
                            .modifier(Modifier::new().width(Dp(160.0)).semantics(SemanticsArgs {
                                test_tag: Some("confirm".into()),
                                ..Default::default()
                            }))
                            .accessibility_label("Confirm action")
                            .on_click(|| {})
                            .child(|| {
                                text("Confirm action");
                            });
                    });
            });
    });
}

#[test]
fn dialog_actions_wrap_when_the_row_cannot_fit() {
    for (width, should_wrap) in [(340, true), (800, false)] {
        let snapshot = UiTest::new()
            .content(dialog_action_fixture)
            .viewport(width, 500)
            .frame();
        let dismiss = snapshot.node("dismiss").bounds();
        let confirm = snapshot.node("confirm").bounds();
        if should_wrap {
            assert!(
                confirm.y >= dismiss.y + dismiss.height,
                "actions must wrap without overlap"
            );
        } else {
            assert_eq!(dismiss.y, confirm.y, "actions should share a row");
            assert!(
                dismiss.x + dismiss.width <= confirm.x,
                "actions must not overlap"
            );
        }
    }
}

#[tessera]
fn dialog_accessibility_fixture(observed: TestProbe<Option<State<(u8, bool)>>>) {
    let settings = remember(|| (0_u8, false));
    observed.set(Some(settings));
    material_theme().child(move || {
        let provider = dialog_provider()
            .controller(remember(|| DialogController::new(true)))
            .role(DialogRole::AlertDialog)
            .dialog_content(move |dialog| match settings.get().0 {
                0 | 1 => {
                    let updated = settings.get().0 == 1;
                    basic_dialog()
                        .dialog(dialog)
                        .headline(if updated {
                            "Updated title"
                        } else {
                            "Initial title"
                        })
                        .supporting_text(if updated {
                            "Updated message"
                        } else {
                            "Initial message"
                        });
                }
                2 => {
                    text("Custom content");
                }
                3 => {
                    full_screen_dialog(Callback::new(|| {}))
                        .dialog(dialog)
                        .title("Editor");
                }
                5 => {
                    crate::md3::date_picker::date_picker_dialog()
                        .dialog(dialog)
                        .title("Pick departure");
                }
                6 => {
                    crate::md3::time_picker::time_picker_dialog()
                        .dialog(dialog)
                        .title("Pick time");
                }
                _ => {
                    basic_dialog()
                        .dialog(dialog)
                        .headline("Custom body")
                        .supporting_text("Unused supporting text")
                        .content(|| {
                            text("Custom content");
                        });
                }
            });
        if settings.get().1 {
            provider
                .accessibility_label("Explicit title")
                .accessibility_description("Explicit description");
        }
    });
}

fn dialog_node(renderer: &UiTest) -> Node {
    let nodes = renderer.accessibility_snapshot().unwrap().nodes;
    let mut dialogs = nodes
        .into_iter()
        .map(|(_, node)| node)
        .filter(|node| matches!(node.role(), Role::Dialog | Role::AlertDialog));
    let dialog = dialogs.next().expect("a single dialog must be published");
    assert!(
        dialogs.next().is_none(),
        "standard content must not create nested dialogs"
    );
    dialog
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn standard_dialog_content_names_and_describes_the_modal_across_updates() {
    crate::test_utils::run_isolated_test(
        "md3::dialog::action_tests::standard_dialog_content_names_and_describes_the_modal_across_updates",
        || {
            let observed = TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(600, 500)
                .frame_step(Duration::from_millis(20))
                .rendered(crate::TesseraComponents)
                .content(|| dialog_accessibility_fixture(observed.clone()));
            renderer.frames(4);
            let settings = observed.get().unwrap();
            for (variant, label, description) in [
                (0, Some("Initial title"), Some("Initial message")),
                (1, Some("Updated title"), Some("Updated message")),
                (2, None, None),
                (3, Some("Editor"), None),
                (4, Some("Custom body"), None),
                (5, Some("Pick departure"), None),
                (6, Some("Pick time"), None),
            ] {
                settings.set((variant, false));
                renderer.frames(4);
                let dialog = dialog_node(&renderer);
                assert_eq!(dialog.label(), label);
                assert_eq!(dialog.description(), description);
                assert_eq!(dialog.role(), Role::AlertDialog);
            }
            settings.set((0, true));
            renderer.frames(4);
            let dialog = dialog_node(&renderer);
            assert_eq!(dialog.label(), Some("Explicit title"));
            assert_eq!(dialog.description(), Some("Explicit description"));
            settings.set((1, true));
            renderer.frames(4);
            assert_eq!(dialog_node(&renderer).label(), Some("Explicit title"));
        },
    );
}
