//! Modal focus lifecycle and background input regressions.
//!
//! ## Usage
//!
//! Keep keyboard navigation inside a modal and restore its invoking control.

use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, FocusDirection, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Node, NodeId, Role},
    focus, remember, tessera,
    testing::UiTest,
};

use crate::{
    bottom_sheet::{BottomSheetController, bottom_sheet_provider},
    button::button,
    column::column,
    dialog::{DialogController, DialogRole, basic_dialog, dialog_provider, full_screen_dialog},
    side_sheet::{
        SideSheetController, SideSheetPosition, modal_side_sheet_provider,
        standard_side_sheet_provider,
    },
    test_utils::OverlayController,
    text::text,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug, PartialEq)]
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
    role: DialogRole,
    open: OverlayController,
    show_origin: bool,
    origin_enabled: bool,
    background_actions: usize,
    actions: bool,
    confirmed: usize,
    long_content: bool,
}

#[tessera]
fn long_dialog_fixture(
    output: tessera_ui::testing::TestProbe<Option<(State<DialogController>, State<usize>)>>,
) {
    let dialog = remember(|| DialogController::new(true));
    let confirmed = remember(|| 0_usize);
    output.set(Some((dialog, confirmed)));
    material_theme().child(move || {
        dialog_provider()
            .accessibility_label("Long message")
            .controller(dialog)
            .on_close_request(move || dialog.with_mut(DialogController::close))
            .dialog_content(move |scope| {
                basic_dialog()
                    .dialog(scope)
                    .headline("Long message")
                    .content(|| {
                        column().children(|| {
                            for index in 0..30 {
                                text(format!("Message line {index:02}"));
                            }
                        });
                    })
                    .dismiss_button(move || {
                        button()
                            .text()
                            .accessibility_label("Keep editing")
                            .on_click(move || dialog.with_mut(DialogController::close))
                            .child(|| {
                                text("Keep editing");
                            });
                    })
                    .confirm_button(move || {
                        button()
                            .text()
                            .accessibility_label("Confirm message")
                            .on_click(move || {
                                dialog.with_mut(DialogController::close);
                                confirmed.with_mut(|confirmed| *confirmed += 1);
                            })
                            .child(|| {
                                text("Confirm");
                            });
                    });
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn basic_dialog_scrolls_long_body_while_headline_and_actions_remain_visible() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::basic_dialog_scrolls_long_body_while_headline_and_actions_remain_visible",
        || {
            let output = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(640, 360)
                .frame_step(Duration::from_millis(20))
                .rendered(crate::TesseraComponents)
                .content(|| long_dialog_fixture(output.clone()));
            renderer.frames(4);
            let dialog = labeled_node(&renderer, "Long message", Role::Dialog)
                .1
                .bounds()
                .unwrap();
            let title = labeled_node(&renderer, "Long message", Role::Label)
                .1
                .bounds()
                .unwrap();
            let dismiss = node(&renderer, "Keep editing").1.bounds().unwrap();
            let confirm = node(&renderer, "Confirm message").1.bounds().unwrap();
            assert!(
                title.y0 >= dialog.y0 && title.y1 <= dialog.y1,
                "headline remains within the dialog"
            );
            assert!(
                dismiss.y0 >= dialog.y0 && dismiss.y1 <= dialog.y1,
                "dismiss action remains within the dialog"
            );
            assert!(
                confirm.y0 >= dialog.y0 && confirm.y1 <= dialog.y1,
                "confirm action remains within the dialog"
            );
            let first = labeled_node(&renderer, "Message line 03", Role::Label)
                .1
                .bounds()
                .unwrap();
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(
                    ((dialog.x0 + dialog.x1) * 0.5) as f32,
                    ((title.y1 + confirm.y0) * 0.5) as f32,
                ),
                0.0,
                -36.0,
            );
            renderer.frames(4);
            let shifted = labeled_node(&renderer, "Message line 03", Role::Label)
                .1
                .bounds()
                .unwrap();
            assert!(
                shifted.y0 < first.y0,
                "the actual body responds to wheel input"
            );
            assert_eq!(
                labeled_node(&renderer, "Long message", Role::Label)
                    .1
                    .bounds(),
                Some(title)
            );
            assert_eq!(node(&renderer, "Keep editing").1.bounds(), Some(dismiss));
            assert_eq!(node(&renderer, "Confirm message").1.bounds(), Some(confirm));
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(
                    ((dialog.x0 + dialog.x1) * 0.5) as f32,
                    ((title.y1 + confirm.y0) * 0.5) as f32,
                ),
                0.0,
                -10_000.0,
            );
            renderer.frames(24);
            let last = labeled_node(&renderer, "Message line 29", Role::Label)
                .1
                .bounds()
                .unwrap();
            assert!(
                last.y0 >= title.y1 && last.y1 <= confirm.y0,
                "scrolling reaches the final body line above the pinned actions"
            );
            assert_eq!(
                labeled_node(&renderer, "Long message", Role::Label)
                    .1
                    .bounds(),
                Some(title)
            );
            assert_eq!(node(&renderer, "Confirm message").1.bounds(), Some(confirm));
            action(&mut renderer, "Confirm message", Action::Click);
            let (dialog, confirmed) = output.get().unwrap();
            assert!(!dialog.with(DialogController::is_open));
            assert_eq!(confirmed.get(), 1);
        },
    );
}

#[tessera]
fn main_content(settings: State<Settings>) {
    column()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            if settings.get().show_origin {
                button()
                    .accessibility_label("Open modal")
                    .enabled(settings.get().origin_enabled)
                    .on_click(move || settings.with(|s| s.open).set(true))
                    .child(|| {
                        text("Open");
                    });
            }
            button()
                .accessibility_label("Other task")
                .on_click(move || settings.with_mut(|s| s.background_actions += 1))
                .child(|| {
                    text("Other");
                });
        });
}

#[tessera]
fn modal_content(settings: State<Settings>) {
    column()
        .modifier(Modifier::new().width(Dp(200.0)))
        .children(move || {
            if !settings.get().actions {
                text("Modal information");
                return;
            }
            button()
                .accessibility_label("First action")
                .on_click(move || settings.with(|s| s.open).set(false))
                .child(|| {
                    text("Accept");
                });
            button()
                .accessibility_label("Second action")
                .on_click(move || settings.with(|s| s.open).set(false))
                .child(|| {
                    text("Cancel");
                });
        });
}

#[tessera]
fn fixture(kind: Kind, output: tessera_ui::testing::TestProbe<Option<State<Settings>>>) {
    let open = match kind {
        Kind::Dialog | Kind::BasicDialog | Kind::FullScreenDialog => {
            OverlayController::Dialog(remember(DialogController::default))
        }
        Kind::BottomSheet => OverlayController::Bottom(remember(BottomSheetController::default)),
        Kind::SideSheet | Kind::StandardSheet => {
            OverlayController::Side(remember(SideSheetController::default))
        }
    };
    let settings = remember(|| Settings {
        label: "Example modal".to_string(),
        role: DialogRole::Dialog,
        open,
        show_origin: true,
        origin_enabled: true,
        background_actions: 0,
        actions: true,
        confirmed: 0,
        long_content: false,
    });
    output.set(Some(settings));
    material_theme().child(move || match (kind, open) {
        (Kind::Dialog, OverlayController::Dialog(controller)) => {
            dialog_provider()
                .accessibility_label(settings.get().label)
                .role(settings.get().role)
                .controller(controller)
                .on_close_request(move || settings.with(|s| s.open).set(false))
                .main_content(move || {
                    main_content(settings);
                })
                .dialog_content(move |_| {
                    modal_content(settings);
                });
        }
        (Kind::BasicDialog, OverlayController::Dialog(controller)) => {
            dialog_provider()
                .accessibility_label("Basic dialog")
                .controller(controller)
                .main_content(move || {
                    main_content(settings);
                })
                .dialog_content(move |scope| {
                    basic_dialog()
                        .dialog(scope)
                        .headline("Title")
                        .supporting_text(if settings.get().long_content {
                            "A detailed explanation that wraps within the dialog container. "
                                .repeat(8)
                        } else {
                            "Details".to_owned()
                        });
                });
        }
        (Kind::FullScreenDialog, OverlayController::Dialog(controller)) => {
            dialog_provider()
                .full_screen(true)
                .accessibility_label(settings.get().label)
                .controller(controller)
                .on_close_request(move || settings.with(|s| s.open).set(false))
                .main_content(move || {
                    main_content(settings);
                })
                .dialog_content(move |scope| {
                    let dialog = full_screen_dialog(tessera_ui::Callback::new(move || {
                        settings.with(|s| s.open).set(false)
                    }))
                    .dialog(scope)
                    .title("New event")
                    .content(move || {
                        modal_content(settings);
                    });
                    if settings.get().actions {
                        dialog.on_confirm(move || {
                            settings.with_mut(|s| s.confirmed += 1);
                            settings.with(|s| s.open).set(false);
                        });
                    }
                });
        }
        (Kind::BottomSheet, OverlayController::Bottom(controller)) => {
            bottom_sheet_provider()
                .accessibility_label(settings.get().label)
                .controller(controller)
                .on_close_request(move || settings.with(|s| s.open).set(false))
                .main_content(move || {
                    main_content(settings);
                })
                .bottom_sheet_content(move || {
                    modal_content(settings);
                });
        }
        (Kind::SideSheet, OverlayController::Side(controller)) => {
            modal_side_sheet_provider()
                .accessibility_label(settings.get().label)
                .position(SideSheetPosition::End)
                .controller(controller)
                .on_close_request(move || settings.with(|s| s.open).set(false))
                .main_content(move || {
                    main_content(settings);
                })
                .side_sheet_content(move || {
                    modal_content(settings);
                });
        }
        (Kind::StandardSheet, OverlayController::Side(controller)) => {
            standard_side_sheet_provider()
                .accessibility_label(settings.get().label)
                .position(SideSheetPosition::End)
                .controller(controller)
                .on_close_request(move || settings.with(|s| s.open).set(false))
                .main_content(move || {
                    main_content(settings);
                })
                .side_sheet_content(move || {
                    modal_content(settings);
                });
        }
        _ => unreachable!("fixture controllers match their kind"),
    });
}

fn node(renderer: &UiTest, label: &str) -> (NodeId, Node) {
    labeled_node(renderer, label, Role::Button)
}

fn labeled_node(renderer: &UiTest, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some(label) && n.role() == role)
        .unwrap_or_else(|| panic!("Missing {role:?} {label}"))
}

fn action(renderer: &mut UiTest, label: &str, action: Action) {
    assert!(renderer.accessibility_action(node(renderer, label).0, action));
    renderer.frames(24);
}

fn tap(renderer: &mut UiTest, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(24);
}

fn open(renderer: &mut UiTest, kind: Kind, settings: State<Settings>) {
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
    assert!(settings.get().open.get());
    if settings.get().actions {
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            node(
                renderer,
                if matches!(kind, Kind::FullScreenDialog) {
                    "Close dialog"
                } else if matches!(kind, Kind::BottomSheet) {
                    "Drag handle"
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

fn check() {
    for kind in [
        Kind::Dialog,
        Kind::BottomSheet,
        Kind::SideSheet,
        Kind::StandardSheet,
    ] {
        for scale in [1.0, 1.5] {
            let output = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(960, 480)
                .scale_factor(scale)
                .frame_step(Duration::from_millis(20))
                .rendered(crate::TesseraComponents)
                .content(|| fixture(kind, output.clone()));
            renderer.frames(4);
            let settings = output.get().unwrap();
            let origin = node(&renderer, "Open modal").0;
            let other_id = node(&renderer, "Other task").0;
            let other_bounds = node(&renderer, "Other task").1.bounds().unwrap();
            open(&mut renderer, kind, output.get().unwrap());
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
                        !renderer.accessibility_action(other_id, action),
                        "{kind:?}: stale background action must be rejected"
                    );
                }
                assert_eq!(settings.get().background_actions, 0);
            }
            settings.with_mut(|s| s.label = "Updated purpose".to_string());
            renderer.frames(3);
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
            renderer.frames(3);
            if matches!(kind, Kind::StandardSheet) {
                action(&mut renderer, "First action", Action::Click);
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
                open(&mut renderer, kind, output.get().unwrap());
                assert!(focus::request_move_focus(FocusDirection::Previous));
                renderer.frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    node(&renderer, "Other task").0
                );
                assert!(focus::request_move_focus(FocusDirection::Next));
                renderer.frames(3);
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
                assert!(settings.get().open.get());
                let other = node(&renderer, "Other task").0;
                renderer.run(|| settings.with(|s| s.open).set(false));
                renderer.frames(24);
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);
                continue;
            }
            let traversal = if matches!(kind, Kind::BottomSheet) {
                vec![
                    (FocusDirection::Next, "First action"),
                    (FocusDirection::Next, "Second action"),
                    (FocusDirection::Next, "Drag handle"),
                    (FocusDirection::Previous, "Second action"),
                ]
            } else {
                vec![
                    (FocusDirection::Next, "Second action"),
                    (FocusDirection::Next, "First action"),
                    (FocusDirection::Previous, "Second action"),
                ]
            };
            for (direction, label) in traversal {
                assert!(focus::request_move_focus(direction));
                renderer.frames(3);
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
            open(&mut renderer, kind, output.get().unwrap());
            tap(
                &mut renderer,
                PxPosition::from_f32_xy(
                    ((other_bounds.x0 + other_bounds.x1) * 0.5) as f32,
                    ((other_bounds.y0 + other_bounds.y1) * 0.5) as f32,
                ),
            );
            assert!(!settings.get().open.get());
            assert_eq!(settings.get().background_actions, 0);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                origin,
                "{kind:?}"
            );
            settings.with_mut(|s| s.actions = false);
            open(&mut renderer, kind, output.get().unwrap());
            let empty_focus = renderer.accessibility_snapshot().unwrap().focus;
            for direction in [FocusDirection::Next, FocusDirection::Previous] {
                assert!(focus::request_move_focus(direction));
                renderer.frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    empty_focus
                );
            }
            settings.with_mut(|s| s.actions = true);
            renderer.frames(4);
            assert!(focus::request_move_focus(FocusDirection::Next));
            renderer.frames(3);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                node(&renderer, "First action").0
            );
            renderer.run(|| settings.with(|s| s.open).set(false));
            renderer.frames(24);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
            let other = node(&renderer, "Other task").0;
            for remove in [false, true] {
                open(&mut renderer, kind, output.get().unwrap());
                renderer.run(|| settings.with(|s| s.open).set(false));
                settings.with_mut(|s| {
                    s.origin_enabled = false;
                    s.show_origin = !remove;
                });
                renderer.frames(24);
                assert_ne!(renderer.accessibility_snapshot().unwrap().focus, origin);
                action(&mut renderer, "Other task", Action::Focus);
                settings.with_mut(|s| {
                    s.origin_enabled = true;
                    s.show_origin = true;
                });
                renderer.frames(4);
                assert_eq!(renderer.accessibility_snapshot().unwrap().focus, other);
            }
        }
    }
}

fn check_full_screen() {
    let kind = Kind::FullScreenDialog;
    for scale in [1.0, 1.5] {
        for width in [390, 960] {
            let output = tessera_ui::testing::TestProbe::default();
            let probe = output.clone();
            let mut renderer = UiTest::new()
                .viewport(width, 640)
                .scale_factor(scale)
                .frame_step(Duration::from_millis(20))
                .rendered(crate::TesseraComponents)
                .content(move || fixture(kind, probe.clone()));
            renderer.frames(4);
            let origin = node(&renderer, "Open modal").0;
            let settings = output.get().unwrap();
            open(&mut renderer, kind, output.get().unwrap());
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
                assert!(focus::request_move_focus(FocusDirection::Next));
                renderer.frames(3);
                assert_eq!(
                    renderer.accessibility_snapshot().unwrap().focus,
                    node(&renderer, label).0
                );
            }
            let pixels = renderer.read_pixels();
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
                settings.get().open.get(),
                "blank full-screen content must not dismiss the modal"
            );
            action(&mut renderer, "Save", Action::Click);
            renderer.frames(24);
            assert_eq!(settings.get().confirmed, 1);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
            open(&mut renderer, kind, output.get().unwrap());
            action(&mut renderer, "Close dialog", Action::Click);
            renderer.frames(24);
            assert!(!settings.get().open.get());
            assert_eq!(settings.get().confirmed, 1);
            assert_eq!(settings.get().background_actions, 0);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
            settings.with_mut(|settings| settings.actions = false);
            open(&mut renderer, kind, output.get().unwrap());
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Save")),
                "omitting the confirm callback must omit its action"
            );
            assert!(focus::request_move_focus(FocusDirection::Next));
            renderer.frames(3);
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                node(&renderer, "Close dialog").0
            );
            action(&mut renderer, "Close dialog", Action::Click);
            renderer.frames(24);
            assert_eq!(settings.get().confirmed, 1);
        }
    }
    let kind = Kind::BasicDialog;
    let output = tessera_ui::testing::TestProbe::default();
    let probe = output.clone();
    let mut renderer = UiTest::new()
        .viewport(960, 640)
        .rendered(crate::TesseraComponents)
        .content(move || fixture(kind, probe.clone()));
    renderer.frames(4);
    let settings = output.get().unwrap();
    renderer.run(|| settings.with(|s| s.open).set(true));
    renderer.frames(24);
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
    renderer.frames(5);
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
fn explicit_alert_dialog_role_preserves_modal_controls_and_focus_restoration() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::explicit_alert_dialog_role_preserves_modal_controls_and_focus_restoration",
        || {
            let kind = Kind::Dialog;
            let output = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(640, 360)
                .frame_step(Duration::from_millis(20))
                .rendered(crate::TesseraComponents)
                .content(|| fixture(kind, output.clone()));
            renderer.frames(4);
            let state = output.get().unwrap();
            state.with_mut(|state| state.role = DialogRole::AlertDialog);
            let origin = node(&renderer, "Open modal").0;
            open(&mut renderer, kind, output.get().unwrap());
            let (id, alert) = labeled_node(&renderer, "Example modal", Role::AlertDialog);
            assert!(alert.is_modal());
            assert_eq!(
                renderer.accessibility_snapshot().unwrap().focus,
                node(&renderer, "First action").0
            );
            state.with_mut(|state| state.label = "Updated alert".to_owned());
            renderer.frames(3);
            assert_eq!(
                labeled_node(&renderer, "Updated alert", Role::AlertDialog).0,
                id
            );
            action(&mut renderer, "First action", Action::Click);
            assert!(!state.get().open.get());
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
        },
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn dialog_and_sheet_restore_after_exit_and_preserve_input_ownership() {
    crate::test_utils::run_isolated_test(
        "modal_focus::tests::dialog_and_sheet_restore_after_exit_and_preserve_input_ownership",
        || {
            check();
            check_full_screen();
        },
    );
}
