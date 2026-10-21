//! Snackbar notification and action accessibility regressions.
//!
//! ## Usage
//!
//! Publish transient status without taking focus from the user's current task.

use std::cell::Cell;

use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Action, ActionRequest, Live, Node, NodeId, Role, TreeId},
    remember, tessera,
};

use crate::{
    button::button, column::column, modifier::ModifierExt as _, text::text, theme::material_theme,
};

use super::snackbar;

#[derive(Clone, PartialEq)]
struct Settings {
    visible: bool,
    stacked: bool,
    dismiss_label: Option<String>,
    actions: usize,
    dismissals: usize,
}

thread_local! { static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) }; }

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        visible: false,
        stacked: false,
        dismiss_label: None,
        actions: 0,
        dismissals: 0,
    });
    SETTINGS.set(Some(settings));
    material_theme().child(move || {
        column().children(move || {
            button()
                .accessibility_label("Current task")
                .on_click(|| {})
                .child(|| {
                    text().content("Continue editing");
                });
            if settings.get().visible {
                snackbar()
                    .modifier(Modifier::new().width(Dp(400.0)))
                    .message("Changes saved")
                    .action_label("Undo")
                    .with_dismiss_action(true)
                    .action_on_new_line(settings.get().stacked)
                    .dismiss_action_label_optional(settings.get().dismiss_label)
                    .on_action(move || settings.with_mut(|s| s.actions += 1))
                    .on_dismiss(move || settings.with_mut(|s| s.dismissals += 1));
            }
        });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| panic!("Missing {role:?}: {label}"))
}

fn action(renderer: &mut HeadlessRenderer, target: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: target,
        data: None,
    }));
    renderer.render_frames(2);
}

fn check(samples: u32) {
    for scale in [1.0, 1.5] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 800,
                height: 420,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let task = node(&renderer, "Current task", Role::Button).0;
        action(&mut renderer, task, Action::Focus);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, task);
        let settings = SETTINGS.get().unwrap();
        settings.with_mut(|s| s.visible = true);
        renderer.render_frames(3);
        for stacked in [false, true] {
            settings.with_mut(|s| s.stacked = stacked);
            renderer.render_frames(3);
            let (_, status) = node(&renderer, "Changes saved", Role::Status);
            assert_eq!(status.live(), Some(Live::Polite));
            assert!(!status.supports_action(Action::Focus));
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, task);
            let undo = node(&renderer, "Undo", Role::Button).0;
            let dismiss = node(&renderer, "Dismiss", Role::Button).0;
            action(&mut renderer, undo, Action::Click);
            action(&mut renderer, dismiss, Action::Click);
            action(&mut renderer, task, Action::Focus);
        }
        assert_eq!(settings.get().actions, 2);
        assert_eq!(settings.get().dismissals, 2);
        settings.with_mut(|s| s.dismiss_label = Some("Close notification".into()));
        renderer.render_frames(3);
        let dismiss = node(&renderer, "Close notification", Role::Button).0;
        action(&mut renderer, dismiss, Action::Click);
        assert_eq!(settings.get().dismissals, 3);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn snackbar_status_and_named_actions_preserve_existing_focus_msaa1() {
    crate::test_utils::run_isolated_test(
        "snackbar::accessibility_tests::snackbar_status_and_named_actions_preserve_existing_focus_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn snackbar_status_and_named_actions_preserve_existing_focus_msaa4() {
    crate::test_utils::run_isolated_test(
        "snackbar::accessibility_tests::snackbar_status_and_named_actions_preserve_existing_focus_msaa4",
        || check(4),
    );
}
