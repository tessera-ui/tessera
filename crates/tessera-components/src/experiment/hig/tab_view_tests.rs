use tessera_ui::{
    State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::column::column;

use super::{super::text::text, TabViewStyle, tab_view};

#[tessera]
fn fixture(probe: TestProbe<Option<State<usize>>>) {
    let page = remember(|| 0usize);
    probe.set(Some(page));
    column().children(move || {
        tab_view(page.get(), vec!["General".into(), "Advanced".into()])
            .on_select(move |index| page.set(index))
            .content(|index| {
                text(if index == 0 {
                    "General page"
                } else {
                    "Advanced page"
                });
            });
    });
}

fn shows(ui: &UiTest, label: &str) -> bool {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .any(|(_, node)| node.label() == Some(label))
}

#[test]
fn choosing_a_tab_shows_its_page_and_the_tabs_sit_on_the_box_edge() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let page = probe.get().expect("fixture state");
    assert!(shows(&ui, "General page"));
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (advanced, tab) = snapshot.find("Advanced", Role::RadioButton).expect("tab");
    let tab = tab.bounds().expect("tab bounds");
    let (_, panel) = snapshot
        .nodes()
        .iter()
        .find(|(_, node)| node.role() == Role::TabPanel)
        .expect("tab panel");
    let panel = panel.bounds().expect("panel bounds");
    assert_eq!(
        (tab.y0 + tab.y1) / 2.0,
        panel.y0,
        "tabs are centred on the box's top edge"
    );
    assert!(ui.accessibility_action(advanced, Action::Click));
    ui.frames(2);
    assert_eq!(page.get(), 1);
    assert!(shows(&ui, "Advanced page"));
    assert!(!shows(&ui, "General page"));
}

#[tessera]
fn styled_fixture(style: TabViewStyle, probe: TestProbe<Option<State<usize>>>) {
    let page = remember(|| 0usize);
    probe.set(Some(page));
    column().children(move || {
        tab_view(page.get(), vec!["General".into(), "Advanced".into()])
            .style(style)
            .on_select(move |index| page.set(index))
            .content(|index| {
                text(if index == 0 {
                    "General page"
                } else {
                    "Advanced page"
                });
            });
    });
}

#[test]
fn tab_bar_tabs_sit_above_the_page_and_sidebar_tabs_are_list_rows() {
    for (style, role) in [
        (TabViewStyle::TabBarOnly, Role::RadioButton),
        (TabViewStyle::SidebarAdaptable, Role::ListBoxOption),
    ] {
        let probe = TestProbe::default();
        let fixture_probe = probe.clone();
        let mut ui = UiTest::new()
            .viewport(500, 300)
            .content(move || styled_fixture(style, fixture_probe.clone()));
        ui.frame();
        let page = probe.get().expect("fixture state");
        let snapshot = ui.inspect_accessibility().expect("published semantics");
        let (advanced, tab) = snapshot.find("Advanced", role).expect("tab");
        let tab = tab.bounds().expect("tab bounds");
        let (_, panel) = snapshot
            .nodes()
            .iter()
            .find(|(_, node)| node.role() == Role::TabPanel)
            .expect("tab panel");
        let panel = panel.bounds().expect("panel bounds");
        if style == TabViewStyle::TabBarOnly {
            assert!(tab.y1 <= panel.y0, "{style:?}: tabs sit above the page");
        } else {
            assert!(tab.x1 <= panel.x0, "{style:?}: tabs sit beside the page");
        }
        assert!(ui.accessibility_action(advanced, Action::Click));
        ui.frames(2);
        assert_eq!(page.get(), 1, "{style:?}");
        assert!(shows(&ui, "Advanced page"), "{style:?}");
    }
}
