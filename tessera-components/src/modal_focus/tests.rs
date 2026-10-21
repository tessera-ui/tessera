//! Modal focus lifecycle and background input regressions.
//!
//! ## Usage
//!
//! Keep keyboard navigation inside a modal and restore its invoking control.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    remember, tessera,
};

use crate::{
    bottom_sheet::bottom_sheet_provider,
    button::button,
    column::column,
    dialog::dialog_provider,
    modifier::ModifierExt as _,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider},
    text::text,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Dialog,
    BottomSheet,
    SideSheet,
    StandardSheet,
}

#[derive(Clone, PartialEq)]
struct Settings {
    open: bool,
    show_origin: bool,
    origin_enabled: bool,
    background_actions: usize,
    actions: bool,
}

thread_local! {
    static KIND: Cell<Kind> = const { Cell::new(Kind::Dialog) };
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn main_content() {
    let settings = SETTINGS.get().unwrap();
    column()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            if settings.get().show_origin {
                button()
                    .accessibility_label("Open modal")
                    .enabled(settings.get().origin_enabled)
                    .on_click(move || settings.with_mut(|s| s.open = true))
                    .child(|| {
                        text().content("Open");
                    });
            }
            button()
                .accessibility_label("Other task")
                .on_click(move || settings.with_mut(|s| s.background_actions += 1))
                .child(|| {
                    text().content("Other");
                });
        });
}

#[tessera]
fn modal_content() {
    let settings = SETTINGS.get().unwrap();
    column()
        .modifier(Modifier::new().width(Dp(200.0)))
        .children(move || {
            if !settings.get().actions {
                text().content("Modal information");
                return;
            }
            button()
                .accessibility_label("First action")
                .on_click(move || settings.with_mut(|s| s.open = false))
                .child(|| {
                    text().content("Accept");
                });
            button()
                .accessibility_label("Second action")
                .on_click(move || settings.with_mut(|s| s.open = false))
                .child(|| {
                    text().content("Cancel");
                });
        });
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        open: false,
        show_origin: true,
        origin_enabled: true,
        background_actions: 0,
        actions: true,
    });
    SETTINGS.set(Some(settings));
    material_theme().child(move || match KIND.get() {
        Kind::Dialog => {
            dialog_provider()
                .accessibility_label("Example modal")
                .is_open(settings.get().open)
                .on_close_request(move || settings.with_mut(|s| s.open = false))
                .main_content(|| {
                    main_content();
                })
                .dialog_content(|| {
                    modal_content();
                });
        }
        Kind::BottomSheet => {
            bottom_sheet_provider()
                .accessibility_label("Example modal")
                .is_open(settings.get().open)
                .on_close_request(move || settings.with_mut(|s| s.open = false))
                .main_content(|| {
                    main_content();
                })
                .bottom_sheet_content(|| {
                    modal_content();
                });
        }
        Kind::SideSheet => {
            modal_side_sheet_provider()
                .accessibility_label("Example modal")
                .position(SideSheetPosition::End)
                .is_open(settings.get().open)
                .on_close_request(move || settings.with_mut(|s| s.open = false))
                .main_content(|| {
                    main_content();
                })
                .side_sheet_content(|| {
                    modal_content();
                });
        }
        Kind::StandardSheet => {
            standard_side_sheet_provider()
                .accessibility_label("Example modal")
                .position(SideSheetPosition::End)
                .is_open(settings.get().open)
                .on_close_request(move || settings.with_mut(|s| s.open = false))
                .main_content(|| {
                    main_content();
                })
                .side_sheet_content(|| {
                    modal_content();
                });
        }
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
    renderer.render_frames(24);
}

fn tap(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(24);
}

fn open(renderer: &mut HeadlessRenderer) {
    let origin = node(renderer, "Open modal").0;
    let other = node(renderer, "Other task").0;
    let bounds = node(renderer, "Open modal").1.bounds().unwrap();
    tap(
        renderer,
        PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        ),
    );
    assert!(SETTINGS.get().unwrap().get().open);
    if SETTINGS.get().unwrap().get().actions {
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            node(renderer, "First action").0
        );
    } else {
        let focus = renderer.accessibility_snapshot().unwrap().focus;
        assert_ne!(focus, origin);
        assert_ne!(focus, other);
    }
}

fn check(samples: u32) {
    for kind in [
        Kind::Dialog,
        Kind::BottomSheet,
        Kind::SideSheet,
        Kind::StandardSheet,
    ] {
        KIND.set(kind);
        for scale in [1.0, 1.5] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 960,
                    height: 480,
                    scale_factor: scale,
                    sample_count: samples,
                    frame_time: Duration::from_millis(20),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            let settings = SETTINGS.get().unwrap();
            let origin = node(&renderer, "Open modal").0;
            let other_id = node(&renderer, "Other task").0;
            let other_bounds = node(&renderer, "Other task").1.bounds().unwrap();
            open(&mut renderer);
            let snapshot = renderer.accessibility_snapshot().unwrap();
            let dialogs: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, n)| n.role() == Role::Dialog)
                .collect();
            assert_eq!(dialogs.len(), 1, "{kind:?}: exactly one dialog boundary");
            let (_, dialog) = dialogs[0];
            assert_eq!(dialog.label(), Some("Example modal"));
            assert_eq!(dialog.is_modal(), !matches!(kind, Kind::StandardSheet));
            let mut descendants = dialog.children().to_vec();
            let mut cursor = 0;
            while cursor < descendants.len() {
                let id = descendants[cursor];
                if let Some((_, child)) = snapshot.nodes.iter().find(|(key, _)| *key == id) {
                    descendants.extend_from_slice(child.children());
                }
                cursor += 1;
            }
            assert!(descendants.contains(&node(&renderer, "First action").0));
            assert!(descendants.contains(&node(&renderer, "Second action").0));
            if matches!(kind, Kind::Dialog) {
                assert!((dialog.bounds().unwrap().width() - 248.0 * scale).abs() < 0.01);
            }
            if !matches!(kind, Kind::StandardSheet) {
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .filter(|(_, n)| n.role() == Role::Button && !n.is_hidden())
                        .all(|(id, _)| descendants.contains(id)),
                    "{kind:?}: exposed buttons must be inside the dialog"
                );
                assert!(
                    !snapshot
                        .nodes
                        .iter()
                        .any(|(id, _)| *id == origin || *id == other_id)
                );
                for action in [Action::Click, Action::Focus] {
                    assert!(
                        !renderer.dispatch_accessibility_action(ActionRequest {
                            action,
                            target_tree: TreeId::ROOT,
                            target_node: other_id,
                            data: None,
                        }),
                        "{kind:?}: stale background action must be rejected"
                    );
                }
                assert_eq!(settings.get().background_actions, 0);
            }
            if matches!(kind, Kind::StandardSheet) {
                action(&mut renderer, "First action", Action::Click);
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
                open(&mut renderer);
                assert!(FocusManager::current().request_move_focus(FocusDirection::Previous));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    node(&renderer, "Other task").0
                );
                assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    node(&renderer, "First action").0
                );
                let bounds = node(&renderer, "Other task").1.bounds().unwrap();
                tap(
                    &mut renderer,
                    PxPosition::from_f32_xy(
                        ((bounds.x0 + bounds.x1) * 0.5) as f32,
                        ((bounds.y0 + bounds.y1) * 0.5) as f32,
                    ),
                );
                assert_eq!(settings.get().background_actions, 1);
                assert!(settings.get().open);
                let other = node(&renderer, "Other task").0;
                settings.with_mut(|settings| settings.open = false);
                renderer.render_frames(24);
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);
                continue;
            }
            for (direction, label) in [
                (FocusDirection::Next, "Second action"),
                (FocusDirection::Next, "First action"),
                (FocusDirection::Previous, "Second action"),
            ] {
                assert!(FocusManager::current().request_move_focus(direction));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    node(&renderer, label).0,
                    "{kind:?}"
                );
            }
            action(&mut renderer, "First action", Action::Click);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                origin,
                "{kind:?}"
            );
            open(&mut renderer);
            tap(
                &mut renderer,
                PxPosition::from_f32_xy(
                    ((other_bounds.x0 + other_bounds.x1) * 0.5) as f32,
                    ((other_bounds.y0 + other_bounds.y1) * 0.5) as f32,
                ),
            );
            assert!(!settings.get().open);
            assert_eq!(settings.get().background_actions, 0);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                origin,
                "{kind:?}"
            );
            settings.with_mut(|s| s.actions = false);
            open(&mut renderer);
            let empty_focus = renderer.accessibility_snapshot().unwrap().focus;
            for direction in [FocusDirection::Next, FocusDirection::Previous] {
                assert!(FocusManager::current().request_move_focus(direction));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    empty_focus
                );
            }
            settings.with_mut(|s| s.actions = true);
            renderer.render_frames(4);
            assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
            renderer.render_frames(3);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                node(&renderer, "First action").0
            );
            settings.with_mut(|s| s.open = false);
            renderer.render_frames(24);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
            let other = node(&renderer, "Other task").0;
            for remove in [false, true] {
                open(&mut renderer);
                settings.with_mut(|s| {
                    s.open = false;
                    s.origin_enabled = false;
                    s.show_origin = !remove;
                });
                renderer.render_frames(24);
                assert_ne!(renderer.accessibility_snapshot().unwrap().focus, origin);
                action(&mut renderer, "Other task", Action::Focus);
                settings.with_mut(|s| {
                    s.origin_enabled = true;
                    s.show_origin = true;
                });
                renderer.render_frames(4);
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa1() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa1",
        || check(1),
    );
}
#[test]
#[ignore = "requires a native GPU adapter"]
fn dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa4() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa4",
        || check(4),
    );
}
