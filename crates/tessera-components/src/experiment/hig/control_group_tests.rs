use tessera_ui::{
    State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{alignment::Alignment, boxed::boxed};

use super::{
    super::{button::button, presentation::presentation_host},
    ControlGroupStyle, control_group,
};

#[tessera]
fn fixture(style: ControlGroupStyle, probe: TestProbe<Option<State<Vec<&'static str>>>>) {
    let pressed = remember(Vec::new);
    probe.set(Some(pressed));
    presentation_host().child(move || {
        boxed().alignment(Alignment::TopStart).children(move || {
            control_group()
                .label("Navigation")
                .style(style)
                .content(move || {
                    button("Back").on_click(move || pressed.with_mut(|p| p.push("Back")));
                    button("Forward").on_click(move || pressed.with_mut(|p| p.push("Forward")));
                });
        });
    });
}

#[test]
fn joined_groups_publish_a_named_group_of_independent_buttons() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 200)
        .content(|| fixture(ControlGroupStyle::Automatic, probe.clone()));
    ui.frame();
    let pressed = probe.get().expect("fixture state");
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, group) = snapshot
        .find("Navigation", Role::Group)
        .expect("named group");
    let group = group.bounds().expect("group bounds");
    let (back, back_node) = snapshot.find("Back", Role::Button).expect("back");
    let (_, forward_node) = snapshot.find("Forward", Role::Button).expect("forward");
    let (back_bounds, forward_bounds) = (
        back_node.bounds().expect("back bounds"),
        forward_node.bounds().expect("forward bounds"),
    );
    assert_eq!(back_bounds.x1, forward_bounds.x0, "the buttons are joined");
    assert_eq!((group.x0, group.x1), (back_bounds.x0, forward_bounds.x1));
    assert!(ui.accessibility_action(back, Action::Click));
    assert_eq!(pressed.get(), ["Back"]);
}

#[test]
fn menu_groups_show_their_buttons_as_menu_items() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 200)
        .content(|| fixture(ControlGroupStyle::Menu, probe.clone()));
    ui.frame();
    let pressed = probe.get().expect("fixture state");
    let (button, node) = ui.accessibility_node("Navigation");
    assert_eq!(node.role(), Role::Button);
    assert!(ui.accessibility_action(button, Action::Click));
    ui.frames(3);
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (forward, _) = snapshot
        .find("Forward", Role::MenuItem)
        .expect("button shown as a menu item");
    assert!(ui.accessibility_action(forward, Action::Click));
    ui.frames(2);
    assert_eq!(pressed.get(), ["Forward"]);
}
