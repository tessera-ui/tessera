//! Dialog action layout regressions.
//!
//! ## Usage
//!
//! Keep dialog actions visible when available width changes.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Callback, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Node, Role},
    remember, tessera,
};

use crate::{
    button::button,
    dialog::{DialogRole, basic_dialog, dialog_provider, full_screen_dialog},
    modifier::ModifierExt as _,
    text::text,
    theme::material_theme,
};

#[tessera]
fn dialog_action_fixture() {
    material_theme().child(|| {
        dialog_provider()
            .accessibility_label("Confirmation")
            .is_open(true)
            .dialog_content(|| {
                basic_dialog()
                    .headline("Confirmation")
                    .supporting_text("Choose an action.")
                    .dismiss_button(|| {
                        button()
                            .text()
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label("Dismiss action")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Dismiss action");
                            });
                    })
                    .confirm_button(|| {
                        button()
                            .text()
                            .modifier(Modifier::new().width(Dp(160.0)))
                            .accessibility_label("Confirm action")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Confirm action");
                            });
                    });
            });
    });
}

fn action_node(renderer: &HeadlessRenderer, label: &str) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::Button && node.label() == Some(label))
        .expect("dialog action must appear in the accessibility tree")
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn dialog_actions_wrap_when_the_row_cannot_fit() {
    crate::test_utils::run_isolated_test(
        "dialog::action_tests::dialog_actions_wrap_when_the_row_cannot_fit",
        || {
            for (width, should_wrap) in [(340, true), (800, false)] {
                let mut renderer = HeadlessRenderer::new(
                    Box::new(dialog_action_fixture),
                    vec![Box::new(crate::TesseraComponents)],
                    HeadlessConfig {
                        width,
                        height: 500,
                        sample_count: 1,
                        frame_time: Duration::from_millis(20),
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(4);
                let dismiss = action_node(&renderer, "Dismiss action").bounds().unwrap();
                let confirm = action_node(&renderer, "Confirm action").bounds().unwrap();
                if should_wrap {
                    assert!(
                        confirm.y0 >= dismiss.y1,
                        "actions must wrap without overlap"
                    );
                } else {
                    assert_eq!(dismiss.y0, confirm.y0, "actions should share a row");
                    assert!(dismiss.x1 <= confirm.x0, "actions must not overlap");
                }
            }
        },
    );
}

thread_local! {
    static CONTENT: Cell<Option<State<(u8, bool)>>> = const { Cell::new(None) };
}

#[tessera]
fn dialog_accessibility_fixture() {
    let settings = remember(|| (0_u8, false));
    CONTENT.set(Some(settings));
    material_theme().child(move || {
        let mut provider = dialog_provider()
            .is_open(true)
            .role(DialogRole::AlertDialog)
            .dialog_content(move || match settings.get().0 {
                0 | 1 => {
                    let updated = settings.get().0 == 1;
                    basic_dialog()
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
                    text().content("Custom content");
                }
                3 => {
                    full_screen_dialog(Callback::new(|| {})).title("Editor");
                }
                _ => {
                    basic_dialog()
                        .headline("Custom body")
                        .supporting_text("Unused supporting text")
                        .content(|| {
                            text().content("Custom content");
                        });
                }
            });
        if settings.get().1 {
            provider = provider
                .accessibility_label("Explicit title")
                .accessibility_description("Explicit description");
        }
        drop(provider);
    });
}

fn dialog_node(renderer: &HeadlessRenderer) -> Node {
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
        "dialog::action_tests::standard_dialog_content_names_and_describes_the_modal_across_updates",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(dialog_accessibility_fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 600,
                    height: 500,
                    sample_count: 1,
                    frame_time: Duration::from_millis(20),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            let settings = CONTENT.get().unwrap();
            for (variant, label, description) in [
                (0, Some("Initial title"), Some("Initial message")),
                (1, Some("Updated title"), Some("Updated message")),
                (2, None, None),
                (3, Some("Editor"), None),
                (4, Some("Custom body"), None),
            ] {
                settings.set((variant, false));
                renderer.render_frames(4);
                let dialog = dialog_node(&renderer);
                assert_eq!(dialog.label(), label);
                assert_eq!(dialog.description(), description);
                assert_eq!(dialog.role(), Role::AlertDialog);
            }
            settings.set((0, true));
            renderer.render_frames(4);
            let dialog = dialog_node(&renderer);
            assert_eq!(dialog.label(), Some("Explicit title"));
            assert_eq!(dialog.description(), Some("Explicit description"));
            settings.set((1, true));
            renderer.render_frames(4);
            assert_eq!(dialog_node(&renderer).label(), Some("Explicit title"));
        },
    );
}
