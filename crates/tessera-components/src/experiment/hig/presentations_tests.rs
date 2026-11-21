use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Modifier, PxPosition, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::keyboard::NamedKey,
};

use crate::{column::column, spacer::spacer};

use super::{
    super::{
        button::{ButtonRole, button},
        presentation::presentation_host,
        text::text,
    },
    AlertAction, alert, confirmation_dialog, help, inspector, popover, sheet,
};

fn shows(ui: &UiTest, label: &str) -> bool {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .any(|(_, node)| node.label() == Some(label) && !node.is_hidden())
}

#[test]
fn tooltips_appear_after_the_pointer_rests_and_hide_when_it_leaves() {
    let mut ui = UiTest::new().viewport(300, 200).content(|| {
        presentation_host().child(|| {
            column().children(|| {
                help("Share this document").child(|| {
                    button("Share");
                });
            });
        });
    });
    ui.frame();
    let bounds = ui
        .accessibility_node("Share")
        .1
        .bounds()
        .expect("button bounds");
    let inside = PxPosition::from_f32_xy((bounds.x0 + 4.0) as f32, (bounds.y0 + 4.0) as f32);
    ui.pointer_move(tessera_ui::MOUSE_POINTER_ID, inside);
    ui.frames(10);
    assert!(!shows(&ui, "Share this document"), "the tooltip waits");
    ui.frames(60);
    assert!(shows(&ui, "Share this document"));
    ui.pointer_move(
        tessera_ui::MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(250.0, 180.0),
    );
    ui.frames(3);
    assert!(!shows(&ui, "Share this document"));
}

#[derive(Clone, Copy)]
struct Controls {
    shown: State<bool>,
    log: State<Vec<&'static str>>,
}

#[tessera]
fn popover_fixture(probe: TestProbe<Option<Controls>>) {
    let shown = remember(|| true);
    let log = remember(Vec::new);
    probe.set(Some(Controls { shown, log }));
    presentation_host().child(move || {
        column().children(move || {
            popover(shown.get())
                .on_dismiss(move || {
                    log.with_mut(|log| log.push("dismiss"));
                    shown.set(false);
                })
                .content(|| {
                    text("Details");
                })
                .child(|| {
                    button("Info");
                });
        });
    });
}

#[test]
fn popovers_close_on_outside_clicks_and_escape() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 300)
        .content(|| popover_fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().expect("fixture state");
    assert!(shows(&ui, "Details"));
    ui.click_at(PxPosition::from_f32_xy(280.0, 280.0));
    ui.frames(2);
    assert_eq!(controls.log.get(), ["dismiss"]);
    assert!(!shows(&ui, "Details"));
    controls.shown.set(true);
    ui.frames(3);
    ui.press_key(NamedKey::Escape);
    ui.frames(2);
    assert_eq!(controls.log.get(), ["dismiss", "dismiss"]);
}

#[tessera]
fn alert_fixture(confirm: bool, probe: TestProbe<Option<Controls>>) {
    let shown = remember(|| true);
    let log = remember(Vec::new);
    probe.set(Some(Controls { shown, log }));
    presentation_host().child(move || {
        let actions = vec![
            AlertAction::new("Delete", move || log.with_mut(|log| log.push("delete")))
                .role(ButtonRole::Destructive),
        ];
        let dismiss = move || {
            log.with_mut(|log| log.push("dismiss"));
            shown.set(false);
        };
        if confirm {
            confirmation_dialog("Delete file?", shown.get())
                .actions(actions)
                .on_dismiss(dismiss);
        } else {
            alert("Delete file?", shown.get())
                .message("You can’t undo this action.")
                .actions(vec![
                    AlertAction::new("Keep", move || log.with_mut(|log| log.push("cancel")))
                        .role(ButtonRole::Cancel),
                    actions[0].clone(),
                ])
                .on_dismiss(dismiss);
        }
    });
}

#[test]
fn alerts_ignore_outside_clicks_and_run_the_chosen_action() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 400)
        .content(|| alert_fixture(false, probe.clone()));
    ui.frames(3);
    let controls = probe.get().expect("fixture state");
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, dialog) = snapshot
        .find("Delete file?", Role::AlertDialog)
        .expect("alert");
    assert_eq!(dialog.description(), Some("You can’t undo this action."));
    ui.click_at(PxPosition::from_f32_xy(5.0, 5.0));
    ui.frames(2);
    assert!(
        controls.log.get().is_empty(),
        "outside clicks leave alerts open"
    );
    ui.press_key(NamedKey::Escape);
    ui.frames(2);
    assert_eq!(controls.log.get(), ["cancel", "dismiss"]);
    controls.shown.set(true);
    controls.log.set(Vec::new());
    ui.frames(3);
    let (delete, _) = ui.accessibility_node("Delete");
    assert!(ui.accessibility_action(delete, Action::Click));
    ui.frames(2);
    assert_eq!(controls.log.get(), ["delete", "dismiss"]);
}

#[test]
fn confirmation_dialogs_add_a_cancel_button() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 400)
        .content(|| alert_fixture(true, probe.clone()));
    ui.frames(3);
    let controls = probe.get().expect("fixture state");
    let (cancel, _) = ui.accessibility_node("Cancel");
    assert!(ui.accessibility_action(cancel, Action::Click));
    ui.frames(2);
    assert_eq!(controls.log.get(), ["dismiss"]);
}

#[tessera]
fn sheet_fixture(probe: TestProbe<Option<Controls>>) {
    let shown = remember(|| true);
    let log = remember(Vec::new);
    probe.set(Some(Controls { shown, log }));
    presentation_host().child(move || {
        sheet(shown.get())
            .on_dismiss(move || {
                log.with_mut(|log| log.push("dismiss"));
                shown.set(false);
            })
            .content(|| {
                text("Export");
            });
    });
}

#[test]
fn sheets_stay_open_on_outside_clicks_and_close_on_escape() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(600, 400)
        .content(|| sheet_fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().expect("fixture state");
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, export) = snapshot
        .nodes()
        .iter()
        .find(|(_, node)| node.label() == Some("Export"))
        .expect("sheet content");
    assert!(
        export.bounds().expect("bounds").y0 < 40.0,
        "the sheet sits at the top"
    );
    ui.click_at(PxPosition::from_f32_xy(10.0, 390.0));
    ui.frames(2);
    assert!(shows(&ui, "Export"));
    ui.press_key(NamedKey::Escape);
    ui.frames(2);
    assert_eq!(controls.log.get(), ["dismiss"]);
}

#[tessera]
fn fill(tag: String) {
    spacer().modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
        test_tag: Some(tag),
        ..Default::default()
    }));
}

#[tessera]
fn inspector_fixture(probe: TestProbe<Option<State<bool>>>) {
    let shown = remember(|| false);
    probe.set(Some(shown));
    inspector(shown.get())
        .content(|| fill("panel".into()))
        .child(|| fill("main".into()));
}

#[test]
fn inspectors_slide_in_and_narrow_the_content() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(600, 300)
        .content(|| inspector_fixture(probe.clone()));
    let frame = ui.frame();
    assert_eq!(frame.node("main").bounds().width.0, 600);
    probe.get().expect("fixture state").set(true);
    ui.frames(6);
    let partial = ui.frame().node("main").bounds().width.0;
    assert!(
        partial < 600 && partial > 340,
        "the inspector slides in: {partial}"
    );
    ui.advance(Duration::from_millis(400));
    ui.frames(4);
    assert_eq!(ui.frame().node("main").bounds().width.0, 340);
}
