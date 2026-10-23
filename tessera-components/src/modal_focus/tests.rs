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
    dialog::{basic_dialog, dialog_provider, full_screen_dialog},
    modifier::ModifierExt as _,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider},
    text::text,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Dialog,
    BasicDialog,
    FullScreenDialog,
    BottomSheet,
    SideSheet,
    StandardSheet,
}

#[derive(Clone, PartialEq)]
struct Settings {
    label: String,
    open: bool,
    show_origin: bool,
    origin_enabled: bool,
    background_actions: usize,
    actions: bool,
    confirmed: usize,
    long_content: bool,
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
        label: "Example modal".to_string(),
        open: false,
        show_origin: true,
        origin_enabled: true,
        background_actions: 0,
        actions: true,
        confirmed: 0,
        long_content: false,
    });
    SETTINGS.set(Some(settings));
    material_theme().child(move || match KIND.get() {
        Kind::Dialog => {
            dialog_provider()
                .accessibility_label(settings.get().label)
                .is_open(settings.get().open)
                .on_close_request(move || settings.with_mut(|s| s.open = false))
                .main_content(|| {
                    main_content();
                })
                .dialog_content(|| {
                    modal_content();
                });
        }
        Kind::BasicDialog => {
            dialog_provider()
                .accessibility_label("Basic dialog")
                .is_open(settings.get().open)
                .main_content(main_content)
                .dialog_content(move || {
                    basic_dialog().headline("Title").supporting_text(
                        if settings.get().long_content {
                            "A detailed explanation that wraps within the dialog container. "
                                .repeat(8)
                        } else {
                            "Details".to_owned()
                        },
                    );
                });
        }
        Kind::FullScreenDialog => {
            dialog_provider()
                .full_screen(true)
                .accessibility_label(settings.get().label)
                .is_open(settings.get().open)
                .on_close_request(move || settings.with_mut(|s| s.open = false))
                .main_content(main_content)
                .dialog_content(move || {
                    let mut dialog = full_screen_dialog(tessera_ui::Callback::new(move || {
                        settings.with_mut(|s| s.open = false)
                    }))
                    .title("New event")
                    .content(modal_content);
                    if settings.get().actions {
                        dialog = dialog.on_confirm(move || {
                            settings.with_mut(|s| {
                                s.confirmed += 1;
                                s.open = false;
                            })
                        });
                    }
                    drop(dialog);
                });
        }
        Kind::BottomSheet => {
            bottom_sheet_provider()
                .accessibility_label(settings.get().label)
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
                .accessibility_label(settings.get().label)
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
                .accessibility_label(settings.get().label)
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
    labeled_node(renderer, label, Role::Button)
}

fn labeled_node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some(label) && n.role() == role)
        .unwrap_or_else(|| panic!("Missing {role:?} {label}"))
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
            node(
                renderer,
                if matches!(KIND.get(), Kind::FullScreenDialog) {
                    "Close dialog"
                } else {
                    "First action"
                }
            )
            .0
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
            let dialog_id = dialogs[0].0;
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
            settings.with_mut(|s| s.label = "Updated purpose".to_string());
            renderer.render_frames(3);
            let updated = renderer.accessibility_snapshot().unwrap();
            assert_eq!(
                updated
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == dialog_id)
                    .unwrap()
                    .1
                    .label(),
                Some("Updated purpose"),
                "{kind:?}: accessible names must update without replacing the dialog"
            );
            settings.with_mut(|s| s.label = "Example modal".to_string());
            renderer.render_frames(3);
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

fn check_full_screen(samples: u32) {
    KIND.set(Kind::FullScreenDialog);
    for scale in [1.0, 1.5] {
        for width in [390, 960] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width,
                    height: 640,
                    scale_factor: scale,
                    sample_count: samples,
                    frame_time: Duration::from_millis(20),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            let origin = node(&renderer, "Open modal").0;
            let settings = SETTINGS.get().unwrap();
            open(&mut renderer);
            let dialog = labeled_node(&renderer, "Example modal", Role::Dialog)
                .1
                .bounds()
                .unwrap();
            let expected_width = (560.0 * scale).min(f64::from(width));
            assert!((dialog.width() - expected_width).abs() < 0.01);
            assert!((dialog.x0 - (f64::from(width) - expected_width) * 0.5).abs() < 0.01);
            assert_eq!(dialog.y0, 0.0);
            assert_eq!(dialog.height(), 640.0);
            let close = node(&renderer, "Close dialog").1.bounds().unwrap();
            assert_eq!(close.height(), 48.0 * scale);
            let title = labeled_node(&renderer, "New event", Role::Label)
                .1
                .bounds()
                .unwrap();
            assert!(title.y0 >= 0.0 && title.y1 <= 64.0 * scale);
            let first = node(&renderer, "First action").1.bounds().unwrap();
            assert!((first.x0 - dialog.x0 - 24.0 * scale).abs() < 0.01);
            assert!(first.y0 >= 88.0 * scale);
            for label in ["Save", "First action", "Second action", "Close dialog"] {
                assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    node(&renderer, label).0
                );
            }
            let pixels = renderer.read_pixels().unwrap();
            let pixel = |x: usize, y: usize| {
                &pixels[(y * width as usize + x) * 4..(y * width as usize + x) * 4 + 4]
            };
            assert_eq!(
                pixel(dialog.x0 as usize + 1, 1),
                pixel(((dialog.x0 + dialog.x1) * 0.5) as usize, 638),
                "full-screen corners retain the container fill"
            );
            tap(
                &mut renderer,
                PxPosition::from_f32_xy((dialog.x1 - 16.0 * scale) as f32, 620.0),
            );
            assert!(
                settings.get().open,
                "blank full-screen content must not dismiss the modal"
            );
            action(&mut renderer, "Save", Action::Click);
            renderer.render_frames(24);
            assert_eq!(settings.get().confirmed, 1);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
            open(&mut renderer);
            action(&mut renderer, "Close dialog", Action::Click);
            renderer.render_frames(24);
            assert!(!settings.get().open);
            assert_eq!(settings.get().confirmed, 1);
            assert_eq!(settings.get().background_actions, 0);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
            settings.with_mut(|settings| settings.actions = false);
            open(&mut renderer);
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Save")),
                "omitting the confirm callback must omit its action"
            );
            assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
            renderer.render_frames(3);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                node(&renderer, "Close dialog").0
            );
            action(&mut renderer, "Close dialog", Action::Click);
            renderer.render_frames(24);
            assert_eq!(settings.get().confirmed, 1);
        }
    }
    KIND.set(Kind::BasicDialog);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 960,
            height: 640,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let settings = SETTINGS.get().unwrap();
    settings.with_mut(|settings| settings.open = true);
    renderer.render_frames(24);
    assert_eq!(
        labeled_node(&renderer, "Basic dialog", Role::Dialog)
            .1
            .bounds()
            .unwrap()
            .width(),
        280.0,
        "minimum basic dialog width includes its content padding"
    );
    settings.with_mut(|settings| settings.long_content = true);
    renderer.render_frames(5);
    let width = labeled_node(&renderer, "Basic dialog", Role::Dialog)
        .1
        .bounds()
        .unwrap()
        .width();
    assert!(
        (512.0..=560.0).contains(&width),
        "maximum basic dialog width includes its content padding"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa1() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa1",
        || {
            check(1);
            check_full_screen(1);
        },
    );
}
#[test]
#[ignore = "requires a native GPU adapter"]
fn dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa4() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::dialog_and_sheet_restore_after_exit_and_preserve_input_ownership_msaa4",
        || {
            check(4);
            check_full_screen(4);
        },
    );
}
