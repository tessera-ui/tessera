//! Snackbar host focus restoration and queued action regressions.
//!
//! ## Usage
//!
//! Restore a dismissed notification's origin without overriding later
//! navigation.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    remember, tessera,
};

use crate::{
    button::button, column::column, modifier::ModifierExt as _, text::text, theme::material_theme,
};

use super::{SnackbarDuration, SnackbarHostState, SnackbarRequest, SnackbarResult, snackbar_host};

#[derive(Clone, PartialEq)]
struct Settings {
    show_origin: bool,
    origin_enabled: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static HOST: Cell<Option<State<SnackbarHostState>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        show_origin: true,
        origin_enabled: true,
    });
    let host = remember(SnackbarHostState::new);
    SETTINGS.set(Some(settings));
    HOST.set(Some(host));
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
                            text().content("Show notification");
                        });
                }
                button()
                    .accessibility_label("Other task")
                    .on_click(|| {})
                    .child(|| {
                        text().content("Continue elsewhere");
                    });
                snackbar_host().state(host);
            });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some(label) && n.role() == Role::Button)
        .unwrap_or_else(|| panic!("Missing button {label}"))
}

fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer, label).0,
        data: None,
    }));
    renderer.render_frames(3);
}

fn show(renderer: &mut HeadlessRenderer, action_label: Option<&str>, duration: SnackbarDuration) {
    HOST.get().unwrap().with_mut(|host| {
        let mut request = SnackbarRequest::new("Saved");
        request.with_dismiss_action = true;
        request.action_label = action_label.map(str::to_string);
        request.duration = Some(duration);
        host.show_snackbar(request);
    });
    renderer.render_frames(3);
}

fn check(samples: u32) {
    for scale in [1.0, 1.5] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 800,
                height: 600,
                scale_factor: scale,
                sample_count: samples,
                frame_time: Duration::from_millis(100),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let origin = node(&renderer, "Origin").0;
        let other = node(&renderer, "Other task").0;
        let host = HOST.get().unwrap();
        let settings = SETTINGS.get().unwrap();
        action(&mut renderer, "Origin", Action::Focus);
        show(&mut renderer, Some("Undo"), SnackbarDuration::Indefinite);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
        action(&mut renderer, "Undo", Action::Focus);
        action(&mut renderer, "Undo", Action::Click);
        assert!(!host.with(SnackbarHostState::is_showing));
        assert_eq!(
            host.with_mut(SnackbarHostState::take_last_result),
            Some(SnackbarResult::ActionPerformed)
        );
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        show(&mut renderer, None, SnackbarDuration::Indefinite);
        let bounds = node(&renderer, "Dismiss").1.bounds().unwrap();
        let point = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(1);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(4);
        assert!(!host.with(SnackbarHostState::is_showing));
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        show(&mut renderer, None, SnackbarDuration::Indefinite);
        action(&mut renderer, "Dismiss", Action::Focus);
        action(&mut renderer, "Other task", Action::Focus);
        host.with_mut(SnackbarHostState::dismiss_current);
        renderer.render_frames(3);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);

        action(&mut renderer, "Origin", Action::Focus);
        show(&mut renderer, Some("First"), SnackbarDuration::Indefinite);
        show(&mut renderer, Some("Second"), SnackbarDuration::Indefinite);
        action(&mut renderer, "First", Action::Focus);
        action(&mut renderer, "First", Action::Click);
        assert!(host.with(SnackbarHostState::is_showing));
        action(&mut renderer, "Second", Action::Focus);
        action(&mut renderer, "Second", Action::Click);
        assert!(!host.with(SnackbarHostState::is_showing));
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        show(&mut renderer, None, SnackbarDuration::Short);
        action(&mut renderer, "Dismiss", Action::Focus);
        renderer.render_frames(45);
        assert!(!host.with(SnackbarHostState::is_showing));
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

        for remove in [false, true] {
            action(&mut renderer, "Origin", Action::Focus);
            show(&mut renderer, None, SnackbarDuration::Indefinite);
            action(&mut renderer, "Dismiss", Action::Focus);
            settings.with_mut(|s| {
                s.show_origin = !remove;
                s.origin_enabled = false;
            });
            host.with_mut(SnackbarHostState::dismiss_current);
            renderer.render_frames(4);
            assert_ne!(renderer.accessibility_snapshot().unwrap().focus, origin);
            action(&mut renderer, "Other task", Action::Focus);
            settings.with_mut(|s| {
                s.show_origin = true;
                s.origin_enabled = true;
            });
            renderer.render_frames(4);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn host_actions_queue_timeout_and_unavailable_origins_restore_focus_msaa1() {
    crate::test_utils::run_isolated_test(
        "snackbar::host_tests::host_actions_queue_timeout_and_unavailable_origins_restore_focus_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn host_actions_queue_timeout_and_unavailable_origins_restore_focus_msaa4() {
    crate::test_utils::run_isolated_test(
        "snackbar::host_tests::host_actions_queue_timeout_and_unavailable_origins_restore_focus_msaa4",
        || check(4),
    );
}
