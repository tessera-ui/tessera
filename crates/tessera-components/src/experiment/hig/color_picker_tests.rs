use tessera_ui::{
    Color, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::column::column;

use super::{super::presentation::presentation_host, color_picker, palette};

#[tessera]
fn fixture(probe: TestProbe<Option<State<Color>>>) {
    let color = remember(|| Color::WHITE);
    probe.set(Some(color));
    presentation_host().child(move || {
        column().children(move || {
            color_picker(color.get())
                .label("Highlight")
                .on_change(move |next| color.set(next));
        });
    });
}

#[test]
fn choosing_a_palette_color_changes_the_selection_and_closes_the_palette() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let color = probe.get().expect("fixture state");
    let (well, node) = ui.accessibility_node("Highlight");
    assert_eq!(node.role(), Role::ColorWell);
    assert!(ui.accessibility_action(well, Action::Click));
    ui.frames(3);
    let (blue, _) = ui.accessibility_node("Blue");
    assert!(ui.accessibility_action(blue, Action::Click));
    ui.frames(3);
    let expected = palette()
        .into_iter()
        .find(|(name, _)| *name == "Blue")
        .map(|(_, color)| color);
    assert_eq!(Some(color.get()), expected);
    assert!(
        ui.inspect_accessibility()
            .expect("published semantics")
            .find("Blue", Role::Button)
            .is_none(),
        "the palette closes"
    );
}
