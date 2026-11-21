use tessera_ui::{
    State,
    accesskit::{Action, Role, Toggled},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use super::{PickerStyle, picker};

#[derive(Clone, Copy)]
struct Controls {
    selection: State<usize>,
    requests: State<Vec<usize>>,
}

#[tessera]
fn fixture(style: PickerStyle, labeled: bool, probe: TestProbe<Option<Controls>>) {
    let selection = remember(|| 0usize);
    let requests = remember(Vec::new);
    probe.set(Some(Controls {
        selection,
        requests,
    }));
    let control = picker(
        selection.get(),
        vec!["Standard".into(), "Express".into(), "Overnight".into()],
    )
    .style(style)
    .on_select(move |index| {
        requests.with_mut(|requests| requests.push(index));
        selection.set(index);
    });
    if labeled {
        control.label("Delivery");
    } else {
        control.accessibility_label("Delivery");
    }
}

fn exercise(style: PickerStyle) {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 120)
        .content(|| fixture(style, true, probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    assert!(
        snapshot.find("Delivery", Role::RadioGroup).is_some(),
        "the label names the group"
    );
    let (id, node) = snapshot
        .find("Express", Role::RadioButton)
        .expect("option node");
    assert_eq!(node.toggled(), Some(Toggled::False));
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.requests.get(), [1]);
    assert_eq!(controls.selection.get(), 1);
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    assert_eq!(
        snapshot
            .find("Express", Role::RadioButton)
            .map(|(_, node)| node.toggled()),
        Some(Some(Toggled::True))
    );
    assert_eq!(
        snapshot
            .nodes()
            .iter()
            .filter(|(_, node)| !node.is_hidden() && node.label() == Some("Delivery"))
            .count(),
        1,
        "the visible label reads only as the group name"
    );
}

#[test]
fn every_style_publishes_a_named_radio_group_and_requests_selection() {
    for style in [
        PickerStyle::Segmented,
        PickerStyle::RadioGroup,
        PickerStyle::Inline,
        PickerStyle::Palette,
    ] {
        exercise(style);
    }
}

#[test]
fn radio_options_stack_at_swiftui_spacing_after_the_label() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(1200, 360)
        .scale_factor(3.0)
        .content(|| fixture(PickerStyle::RadioGroup, true, probe.clone()));
    ui.frame();
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let bounds = |label: &str| {
        snapshot
            .find(label, Role::RadioButton)
            .and_then(|(_, node)| node.bounds())
            .expect("option bounds")
    };
    let (first, second) = (bounds("Standard"), bounds("Express"));
    assert_eq!(first.y1 - first.y0, 49.0, "16 1/3pt rows at 3x");
    assert_eq!(second.y0 - first.y1, 18.0, "6pt spacing at 3x");
    eprintln!(
        "DBG {:?}",
        snapshot
            .nodes()
            .iter()
            .map(|(_, n)| (
                n.role(),
                n.label().map(str::to_owned),
                n.is_hidden(),
                n.bounds().map(|b| (b.x0, b.y0, b.x1, b.y1))
            ))
            .collect::<Vec<_>>()
    );
    assert!(first.x0 > 8.0, "options follow the leading label");
}

#[test]
fn unlabeled_pickers_start_at_the_control() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 120)
        .content(|| fixture(PickerStyle::RadioGroup, false, probe.clone()));
    ui.frame();
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, group) = snapshot
        .find("Delivery", Role::RadioGroup)
        .expect("group node");
    assert_eq!(group.bounds().map(|bounds| bounds.x0), Some(0.0));
}

#[test]
fn pop_up_buttons_open_with_the_selection_over_the_button_and_choose_from_the_menu() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new().viewport(400, 300).content(|| {
        let probe = probe.clone();
        super::super::presentation::presentation_host().child(move || {
            fixture(PickerStyle::Automatic, true, probe.clone());
        });
    });
    ui.frame();
    let controls = probe.get().expect("fixture state");
    controls.selection.set(1);
    ui.frames(2);
    let (button, node) = ui.accessibility_node("Delivery");
    assert_eq!(node.role(), Role::Button);
    assert_eq!(node.is_expanded(), Some(false));
    let anchor = node.bounds().expect("button bounds");
    assert!(ui.accessibility_action(button, Action::Click));
    ui.frames(3);
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (_, express) = snapshot
        .find("Express", Role::MenuItemCheckBox)
        .expect("selected option");
    assert_eq!(express.toggled(), Some(Toggled::True));
    let row = express.bounds().expect("item bounds");
    assert!(
        (((row.y0 + row.y1) - (anchor.y0 + anchor.y1)) / 2.0).abs() <= 1.0,
        "the selected option opens over the button"
    );
    let (overnight, _) = snapshot
        .find("Overnight", Role::MenuItemCheckBox)
        .expect("other option");
    assert!(ui.accessibility_action(overnight, Action::Click));
    ui.frames(3);
    assert_eq!(controls.requests.get(), [2]);
    assert!(
        ui.inspect_accessibility()
            .expect("published semantics")
            .find("Express", Role::MenuItemCheckBox)
            .is_none(),
        "choosing closes the menu"
    );
}
