use tessera_ui::{
    State,
    accesskit::{Action, Role, Toggled},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use super::{ToggleStyle, toggle};

#[derive(Clone, Copy)]
struct Controls {
    on: State<bool>,
    mixed: State<bool>,
    requests: State<Vec<bool>>,
}

#[tessera]
fn fixture(style: ToggleStyle, probe: TestProbe<Option<Controls>>) {
    let on = remember(|| false);
    let mixed = remember(|| false);
    let requests = remember(Vec::new);
    probe.set(Some(Controls {
        on,
        mixed,
        requests,
    }));
    toggle(on.get())
        .label("Option")
        .style(style)
        .mixed(mixed.get())
        .on_value_change(move |next| {
            requests.with_mut(|requests| requests.push(next));
            on.set(next);
        });
}

fn activate(ui: &mut UiTest, role: Role) -> Option<Toggled> {
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (id, node) = snapshot.find("Option", role).expect("toggle node");
    let before = node.toggled();
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    before
}

fn exercise(style: ToggleStyle, role: Role, mixed_reads_as: Toggled) {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 80)
        .content(|| fixture(style, probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    assert_eq!(activate(&mut ui, role), Some(Toggled::False));
    assert_eq!(activate(&mut ui, role), Some(Toggled::True));
    controls.mixed.set(true);
    ui.frames(1);
    assert_eq!(activate(&mut ui, role), Some(mixed_reads_as));
    assert_eq!(controls.requests.get(), [true, false, true]);
    assert!(controls.on.get());
}

#[test]
fn every_style_publishes_its_role_and_a_mixed_value_requests_on() {
    exercise(ToggleStyle::Automatic, Role::CheckBox, Toggled::Mixed);
    exercise(ToggleStyle::Checkbox, Role::CheckBox, Toggled::Mixed);
    exercise(ToggleStyle::Switch, Role::Switch, Toggled::Mixed);
    exercise(ToggleStyle::Button, Role::Button, Toggled::False);
}

#[test]
fn switch_labels_lead_the_switch_and_read_as_one_control() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 80)
        .content(|| fixture(ToggleStyle::Switch, probe.clone()));
    ui.frame();
    let text = ui.node("text_node").bounds();
    let switch = ui
        .inspect_accessibility()
        .expect("published semantics")
        .find("Option", Role::Switch)
        .and_then(|(_, node)| node.bounds())
        .expect("switch bounds");
    assert_eq!(switch.x0, f64::from(text.x.0 + text.width.0) + 8.0);
    assert_eq!(switch.x1 - switch.x0, 54.0);
    assert_eq!(
        ui.inspect_accessibility()
            .expect("published semantics")
            .nodes()
            .iter()
            .filter(|(_, node)| !node.is_hidden() && node.label() == Some("Option"))
            .count(),
        1
    );
}
