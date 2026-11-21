use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, Modifier, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::keyboard::NamedKey,
};

use crate::column::column;

use super::{super::text::text, ListStyle, list, list_row, list_section};

#[tessera]
fn fixture(style: ListStyle, probe: TestProbe<Option<State<usize>>>) {
    let selected = remember(|| 0usize);
    probe.set(Some(selected));
    column().children(move || {
        list()
            .style(style)
            .selection(selected.get())
            .on_select(move |tag| selected.set(tag))
            .accessibility_label("Mailboxes")
            .modifier(Modifier::new().size(Dp(200.0), Dp(200.0)))
            .content(move || {
                list_section().header("Favorites").content(move || {
                    for (tag, name) in ["Inbox", "Drafts", "Sent"].into_iter().enumerate() {
                        list_row(tag).accessibility_label(name).content(move || {
                            text(name).modifier(Modifier::new().semantics(SemanticsArgs {
                                test_tag: Some(format!("row-{tag}")),
                                ..Default::default()
                            }));
                        });
                    }
                });
            });
    });
}

fn selected(ui: &UiTest) -> Vec<String> {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .filter(|(_, node)| node.role() == Role::ListBoxOption && node.is_selected() == Some(true))
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

#[test]
fn clicking_and_arrow_keys_select_rows() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 300)
        .content(|| fixture(ListStyle::Automatic, probe.clone()));
    ui.frame();
    let selection = probe.get().expect("fixture state");
    assert_eq!(selected(&ui), ["Inbox"]);
    assert!(
        ui.inspect_accessibility()
            .expect("published semantics")
            .find("Mailboxes", Role::ListBox)
            .is_some()
    );
    let (sent, _) = ui
        .inspect_accessibility()
        .expect("published semantics")
        .find("Sent", Role::ListBoxOption)
        .map(|(id, node)| (id, node.clone()))
        .expect("row");
    assert!(ui.accessibility_action(sent, Action::Click));
    ui.frames(2);
    assert_eq!(selection.get(), 2);
    assert_eq!(selected(&ui), ["Sent"]);
    ui.press_key(NamedKey::ArrowUp);
    ui.frames(2);
    assert_eq!(selection.get(), 1, "the arrow keys move the selection");
}

#[test]
fn sidebar_rows_are_taller() {
    let height = |style| {
        let probe = TestProbe::default();
        let mut ui = UiTest::new()
            .viewport(300, 300)
            .content(move || fixture(style, probe.clone()));
        ui.frame();
        let bounds = ui
            .inspect_accessibility()
            .expect("published semantics")
            .find("Inbox", Role::ListBoxOption)
            .and_then(|(_, node)| node.bounds())
            .expect("row bounds");
        bounds.y1 - bounds.y0
    };
    assert_eq!(height(ListStyle::Inset), 24.0);
    assert_eq!(height(ListStyle::Sidebar), 28.0);
}
