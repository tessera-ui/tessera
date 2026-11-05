//! Snackbar notification and action accessibility regressions.
//!
//! ## Usage
//!
//! Publish transient status without taking focus from the user's current task.

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

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let settings = remember(|| Settings {
        visible: false,
        stacked: false,
        dismiss_label: None,
        actions: 0,
        dismissals: 0,
    });
    observed.with_mut(|handles| handles.settings = Some(settings));
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
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    let (id, node) = snapshot
        .find(label, role)
        .unwrap_or_else(|| panic!("missing {role:?}: {label}"));
    (id, node.clone())
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
        let observed = tessera_ui::testing::TestProbe::default();
        let mut renderer = HeadlessRenderer::new(
            Box::new({
                let observed = observed.clone();
                move || fixture(observed.clone())
            }),
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
        let settings = observed.get().settings.unwrap();
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

mod escape {
    use tessera_ui::{
        Callback, State,
        testing::TestProbe,
        winit::{
            event::ElementState,
            keyboard::{KeyCode, PhysicalKey},
        },
    };

    use super::super::dismiss_key;
    use super::*;

    #[derive(Clone, Copy, Default)]
    struct EscapeHandles {
        dismissals: Option<State<usize>>,
        dismiss: Option<Callback>,
    }

    #[tessera]
    fn escape_fixture(observed: TestProbe<EscapeHandles>) {
        let dismissals = remember(|| 0usize);
        let dismiss = Callback::new(move || dismissals.with_mut(|count| *count += 1));
        observed.set(EscapeHandles {
            dismissals: Some(dismissals),
            dismiss: Some(dismiss),
        });
    }

    #[test]
    fn escape_adapter_dismisses_only_on_pressed_escape() {
        use tessera_ui::testing::{
            layout_test, pump_layout_test_session, start_layout_test_session,
        };
        let observed = TestProbe::default();
        let mut session = start_layout_test_session(
            layout_test({
                let observed = observed.clone();
                move || escape_fixture(observed.clone())
            })
            .viewport_px(200, 100),
        );
        pump_layout_test_session(&mut session, 0);
        let dismissals = observed.get().dismissals.unwrap();
        let dismiss = observed.get().dismiss.unwrap();
        assert!(!dismiss_key(
            PhysicalKey::Code(KeyCode::Enter),
            ElementState::Pressed,
            dismiss
        ));
        assert!(!dismiss_key(
            PhysicalKey::Code(KeyCode::Escape),
            ElementState::Released,
            dismiss
        ));
        assert_eq!(dismissals.get(), 0);
        assert!(dismiss_key(
            PhysicalKey::Code(KeyCode::Escape),
            ElementState::Pressed,
            dismiss
        ));
        assert_eq!(dismissals.get(), 1);
    }
}
