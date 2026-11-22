use tessera_ui::{
    PxPosition, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::column::column;

use super::{super::presentation::presentation_host, searchable};

#[derive(Clone, Copy)]
struct Controls {
    query: State<String>,
    tokens: State<Vec<String>>,
    scope: State<usize>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let query = remember(String::new);
    let tokens = remember(|| vec!["PDF".to_owned(), "Shared".to_owned()]);
    let scope = remember(|| 0usize);
    probe.set(Some(Controls {
        query,
        tokens,
        scope,
    }));
    presentation_host().child(move || {
        column().children(move || {
            searchable(query.get())
                .tokens(tokens.get())
                .on_tokens_change(move |next| tokens.set(next))
                .suggestions(vec!["Invoices".into(), "Receipts".into()])
                .scopes(vec!["All".into(), "Unread".into()])
                .scope(scope.get())
                .on_scope_change(move |next| scope.set(next))
                .on_change(move |text| query.set(text));
        });
    });
}

fn shows(ui: &UiTest, label: &str, role: Role) -> bool {
    ui.inspect_accessibility()
        .expect("published semantics")
        .find(label, role)
        .is_some_and(|(_, node)| !node.is_hidden())
}

fn focus_field(ui: &mut UiTest) {
    let bounds = ui
        .accessibility_node("Search")
        .1
        .bounds()
        .expect("field bounds");
    ui.click_at(PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    ));
    ui.frame();
}

#[test]
fn typing_shows_suggestions_and_choosing_one_completes_the_query() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    assert!(!shows(&ui, "Invoices", Role::MenuItem));
    assert!(
        !shows(&ui, "Unread", Role::RadioButton),
        "scopes wait for a search"
    );
    focus_field(&mut ui);
    ui.inject_text("inv");
    ui.frames(3);
    assert_eq!(controls.query.get(), "inv");
    assert!(shows(&ui, "Unread", Role::RadioButton));
    assert!(shows(&ui, "Invoices", Role::MenuItem));
    ui.inject_text("o");
    ui.frames(2);
    assert_eq!(
        controls.query.get(),
        "invo",
        "typing continues while suggestions show"
    );
    let (invoices, _) = ui
        .inspect_accessibility()
        .expect("published semantics")
        .find("Invoices", Role::MenuItem)
        .map(|(id, node)| (id, node.clone()))
        .expect("suggestion");
    assert!(ui.accessibility_action(invoices, Action::Click));
    ui.frames(3);
    assert_eq!(controls.query.get(), "Invoices");
    assert!(
        !shows(&ui, "Invoices", Role::MenuItem),
        "choosing closes the suggestions"
    );
}

#[test]
fn tokens_are_removed_when_clicked_and_scopes_change_the_selection() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    let (pdf, node) = ui.accessibility_node("PDF");
    assert_eq!(node.role(), Role::Button);
    assert!(ui.accessibility_action(pdf, Action::Click));
    ui.frames(2);
    assert_eq!(controls.tokens.get(), ["Shared"]);
    controls.query.set("report".into());
    ui.frames(2);
    let (unread, _) = ui
        .inspect_accessibility()
        .expect("published semantics")
        .find("Unread", Role::RadioButton)
        .map(|(id, node)| (id, node.clone()))
        .expect("scope");
    assert!(ui.accessibility_action(unread, Action::Click));
    ui.frames(2);
    assert_eq!(controls.scope.get(), 1);
}
