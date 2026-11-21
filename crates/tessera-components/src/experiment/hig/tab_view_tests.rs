use tessera_ui::{
    State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::column::column;

use super::{super::text::text, tab_view};

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
