//! Snackbar host focus restoration and queued action regressions.
//!
//! ## Usage
//!
//! Restore a dismissed notification's origin without overriding later
//! navigation.

use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Node, NodeId, Role},
    remember, tessera,
    testing::UiTest,
};

use crate::{button::button, column::column, text::text, theme::material_theme};

use super::{
    SnackbarDuration, SnackbarHostController, SnackbarRequest, SnackbarResult, snackbar_host,
};

#[derive(Clone, PartialEq)]
struct Settings {
    show_origin: bool,
    origin_enabled: bool,
}

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
    host: Option<State<SnackbarHostController>>,
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let settings = remember(|| Settings {
        show_origin: true,
        origin_enabled: true,
    });
    let host = remember(SnackbarHostController::new);
    observed.with_mut(|handles| handles.settings = Some(settings));
    observed.with_mut(|handles| handles.host = Some(host));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().width(Dp(400.0)))
            .children(move || {
                if settings.get().show_origin {
                    button()
                        .accessibility_label("Origin")
                        .enabled(settings.get().origin_enabled)
                        .on_click(|| {})
                        .child(|| {
                            text("Show notification");
                        });
                }
                button()
                    .accessibility_label("Other task")
                    .on_click(|| {})
                    .child(|| {
                        text("Continue elsewhere");
                    });
                snackbar_host().controller(host);
            });
    });
}

fn node(renderer: &UiTest, label: &str) -> (NodeId, Node) {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    let (id, node) = snapshot
        .find(label, Role::Button)
        .unwrap_or_else(|| panic!("missing button: {label}"));
    (id, node.clone())
}

fn action(renderer: &mut UiTest, label: &str, action: Action) {
    assert!(renderer.accessibility_action(node(renderer, label).0, action));
    renderer.frames(3);
}

fn show(
    renderer: &mut UiTest,
    host: State<SnackbarHostController>,
    action_label: Option<&str>,
    duration: SnackbarDuration,
) {
    host.with_mut(|host| {
        let mut request = SnackbarRequest::new("Saved");
        request.with_dismiss_action = true;
        request.action_label = action_label.map(str::to_string);
        request.duration = Some(duration);
        host.show_snackbar(request);
    });
    renderer.frames(3);
}

fn check() {
    for scale in [1.0, 1.5] {
        let observed = tessera_ui::testing::TestProbe::default();
        let mut renderer = UiTest::new()
            .viewport(800, 600)
            .scale_factor(scale)
            .frame_step(Duration::from_millis(100))
            .rendered(crate::TesseraComponents)
            .content(|| fixture(observed.clone()));
        renderer.frames(3);
        let origin = node(&renderer, "Origin").0;
        let other = node(&renderer, "Other task").0;
        let host = observed.get().host.unwrap();
        let settings = observed.get().settings.unwrap();
        action(&mut renderer, "Origin", Action::Focus);
        show(
            &mut renderer,
            host,
            Some("Undo"),
            SnackbarDuration::Indefinite,
        );
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
        action(&mut renderer, "Undo", Action::Focus);
        action(&mut renderer, "Undo", Action::Click);
        assert!(!host.with(SnackbarHostController::is_showing));
        assert_eq!(
            host.with_mut(SnackbarHostController::take_last_result),
            Some(SnackbarResult::ActionPerformed)
        );
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        show(&mut renderer, host, None, SnackbarDuration::Indefinite);
        let bounds = node(&renderer, "Dismiss").1.bounds().unwrap();
        let point = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.frames(1);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.frames(4);
        assert!(!host.with(SnackbarHostController::is_showing));
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        show(&mut renderer, host, None, SnackbarDuration::Indefinite);
        action(&mut renderer, "Dismiss", Action::Focus);
        action(&mut renderer, "Other task", Action::Focus);
        host.with_mut(SnackbarHostController::dismiss_current);
        renderer.frames(3);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);

        action(&mut renderer, "Origin", Action::Focus);
        show(
            &mut renderer,
            host,
            Some("First"),
            SnackbarDuration::Indefinite,
        );
        show(
            &mut renderer,
            host,
            Some("Second"),
            SnackbarDuration::Indefinite,
        );
        action(&mut renderer, "First", Action::Focus);
        action(&mut renderer, "First", Action::Click);
        assert!(host.with(SnackbarHostController::is_showing));
        action(&mut renderer, "Second", Action::Focus);
        action(&mut renderer, "Second", Action::Click);
        assert!(!host.with(SnackbarHostController::is_showing));
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        show(&mut renderer, host, None, SnackbarDuration::Short);
        action(&mut renderer, "Dismiss", Action::Focus);
        renderer.frames(45);
        assert!(!host.with(SnackbarHostController::is_showing));
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        for remove in [false, true] {
            action(&mut renderer, "Origin", Action::Focus);
            show(&mut renderer, host, None, SnackbarDuration::Indefinite);
            action(&mut renderer, "Dismiss", Action::Focus);
            settings.with_mut(|s| {
                s.show_origin = !remove;
                s.origin_enabled = false;
            });
            host.with_mut(SnackbarHostController::dismiss_current);
            renderer.frames(4);
            assert_ne!(renderer.accessibility_snapshot().unwrap().focus, origin);
            action(&mut renderer, "Other task", Action::Focus);
            settings.with_mut(|s| {
                s.show_origin = true;
                s.origin_enabled = true;
            });
            renderer.frames(4);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn host_actions_queue_timeout_and_unavailable_origins_restore_focus() {
    crate::test_utils::run_isolated_test(
        "snackbar::host_tests::host_actions_queue_timeout_and_unavailable_origins_restore_focus",
        check,
    );
}

#[tessera]
fn custom_slot_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let host = remember(SnackbarHostController::new);
    observed.with_mut(|handles| handles.host = Some(host));
    material_theme().child(move || {
        snackbar_host()
            .controller(host)
            .snackbar(|data: super::SnackbarData| {
                text(data.message().to_owned())
                    .accessibility_label(format!("Custom notification: {}", data.message()));
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn custom_host_slot_renders_current_queued_message() {
    crate::test_utils::run_isolated_test(
        "snackbar::host_tests::custom_host_slot_renders_current_queued_message",
        || {
            let observed = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(400, 200)
                .rendered(crate::TesseraComponents)
                .content(|| custom_slot_fixture(observed.clone()));
            renderer.frames(3);
            let host = observed.get().host.unwrap();
            for message in ["Saved", "Uploaded"] {
                host.with_mut(|host| {
                    let mut request = SnackbarRequest::new(message);
                    request.duration = Some(SnackbarDuration::Indefinite);
                    host.show_snackbar(request);
                });
                renderer.frames(3);
                let label = format!("Custom notification: {message}");
                assert!(
                    renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(label.as_str()))
                );
                host.with_mut(SnackbarHostController::dismiss_current);
                renderer.frames(3);
                assert!(
                    !renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(label.as_str()))
                );
            }
        },
    );
}
