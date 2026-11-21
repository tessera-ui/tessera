use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, FocusRequester, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::keyboard::NamedKey,
};

use crate::{column::column, spacer::spacer};

use super::{
    super::{button::button, presentation::presentation_host},
    context_menu, menu, menu_divider, menu_item,
};

#[derive(Clone, Copy)]
struct Controls {
    chosen: State<Vec<&'static str>>,
    behind: State<u32>,
    focus: FocusRequester,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let chosen = remember(Vec::new);
    let behind = remember(|| 0);
    let focus = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        chosen,
        behind,
        focus,
    }));
    presentation_host().child(move || {
        column().children(move || {
            menu("Actions").content(move || {
                menu_item("Copy").on_click(move || chosen.with_mut(|c| c.push("Copy")));
                menu_item("Rename").enabled(false);
                menu_divider();
                menu_item("Delete").on_click(move || chosen.with_mut(|c| c.push("Delete")));
            });
            spacer().modifier(Modifier::new().height(Dp(120.0)));
            button("Behind")
                .focus_requester(focus)
                .on_click(move || behind.set(behind.get() + 1));
        });
    });
}

fn items(ui: &UiTest) -> Vec<String> {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .filter(|(_, node)| node.role() == Role::MenuItem && !node.is_hidden())
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

fn focused(ui: &UiTest) -> Option<String> {
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    snapshot
        .get(snapshot.focus())
        .and_then(|node| node.label().map(str::to_owned))
}

fn open(ui: &mut UiTest) {
    let (id, _) = ui.accessibility_node("Actions");
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(3);
}

#[test]
fn choosing_an_item_runs_it_closes_the_menu_and_returns_focus_to_the_button() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    assert!(items(&ui).is_empty());
    assert_eq!(
        ui.accessibility_node("Actions").1.is_expanded(),
        Some(false)
    );
    open(&mut ui);
    assert_eq!(items(&ui), ["Copy", "Rename", "Delete"]);
    assert_eq!(ui.accessibility_node("Actions").1.is_expanded(), Some(true));
    let (rename, node) = ui.accessibility_node("Rename");
    assert!(node.is_disabled());
    assert!(!ui.accessibility_action(rename, Action::Click));
    let (delete, _) = ui.accessibility_node("Delete");
    assert!(ui.accessibility_action(delete, Action::Click));
    ui.frames(3);
    assert_eq!(controls.chosen.get(), ["Delete"]);
    assert!(items(&ui).is_empty());
    assert_eq!(focused(&ui).as_deref(), Some("Actions"));
}

#[test]
fn escape_and_outside_clicks_dismiss_without_reaching_the_content_behind() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    open(&mut ui);
    ui.press_key(NamedKey::Escape);
    ui.frames(2);
    assert!(items(&ui).is_empty());

    open(&mut ui);
    let behind = ui
        .accessibility_node("Behind")
        .1
        .bounds()
        .expect("button bounds");
    ui.click_at(PxPosition::from_f32_xy(
        ((behind.x0 + behind.x1) / 2.0) as f32,
        ((behind.y0 + behind.y1) / 2.0) as f32,
    ));
    ui.frames(2);
    assert!(items(&ui).is_empty());
    assert_eq!(controls.behind.get(), 0, "the dismissing click is consumed");
    assert!(!ui.run(|| controls.focus.is_focused()));
}

#[test]
fn arrow_keys_move_between_enabled_items_and_return_chooses() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    open(&mut ui);
    assert_eq!(focused(&ui).as_deref(), Some("Copy"));
    ui.press_key(NamedKey::ArrowDown);
    ui.frames(2);
    assert_eq!(
        focused(&ui).as_deref(),
        Some("Delete"),
        "disabled items are skipped"
    );
    ui.press_key(NamedKey::Enter);
    ui.frames(3);
    assert_eq!(controls.chosen.get(), ["Delete"]);
}

#[tessera]
fn context_fixture(probe: TestProbe<Option<State<Vec<&'static str>>>>) {
    let chosen = remember(Vec::new);
    probe.set(Some(chosen));
    presentation_host().child(move || {
        context_menu()
            .child(|| {
                spacer().modifier(Modifier::new().size(Dp(200.0), Dp(100.0)));
            })
            .menu(move || {
                menu_item("Open").on_click(move || chosen.with_mut(|c| c.push("Open")));
            });
    });
}

#[test]
fn secondary_clicks_and_the_show_menu_action_open_the_context_menu_at_the_pointer() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| context_fixture(probe.clone()));
    ui.frame();
    let chosen = probe.get().expect("fixture state");
    let point = PxPosition::from_f32_xy(60.0, 40.0);
    ui.click_at(point);
    ui.frames(2);
    assert!(items(&ui).is_empty(), "primary clicks do not open the menu");
    ui.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Right);
    ui.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Right);
    ui.frames(3);
    let (open, node) = ui.accessibility_node("Open");
    let bounds = node.bounds().expect("item bounds");
    assert!(
        (bounds.x0 - 60.0).abs() <= 1.0 && bounds.y0 > 40.0 && bounds.y0 < 40.0 + 10.0,
        "the menu opens at the pointer: {bounds:?}"
    );
    assert!(ui.accessibility_action(open, Action::Click));
    ui.frames(3);
    assert_eq!(chosen.get(), ["Open"]);
    assert!(items(&ui).is_empty());

    let target = ui
        .inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .find(|(_, node)| node.supports_action(Action::ShowContextMenu))
        .map(|(id, _)| *id)
        .expect("context menu target");
    assert!(ui.accessibility_action(target, Action::ShowContextMenu));
    ui.frames(3);
    assert_eq!(items(&ui), ["Open"]);
}

/// Rows below the menu button, where only the menu draws.
fn panel_rows(pixels: &[u8]) -> &[u8] {
    &pixels[40 * 400 * 4..200 * 400 * 4]
}

fn check_menu_fades() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let before = ui.read_pixels();
    let (button, _) = ui.accessibility_node("Actions");
    assert!(ui.accessibility_action(button, Action::Click));
    ui.frames(2);
    let early = ui.read_pixels();
    ui.frames(20);
    let open = ui.read_pixels();
    assert_ne!(early, open, "the menu fades in");
    ui.frames(2);
    assert_eq!(open, ui.read_pixels(), "the open menu rests");
    ui.press_key(NamedKey::Escape);
    ui.frames(2);
    let closing = ui.read_pixels();
    assert_ne!(
        panel_rows(&closing),
        panel_rows(&before),
        "the menu fades out instead of vanishing"
    );
    ui.frames(24);
    assert_eq!(panel_rows(&ui.read_pixels()), panel_rows(&before));
}

#[test]
#[ignore = "requires a native GPU adapter for menu pixels"]
fn menus_fade_in_and_out() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::menu::tests::menus_fade_in_and_out",
        check_menu_fades,
    );
}
