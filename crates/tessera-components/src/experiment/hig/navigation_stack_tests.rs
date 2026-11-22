use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, KeyboardEvent, Modifier, State,
    accesskit::{Action, Role},
    modifier::FocusModifierExt as _,
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{
        event::ElementState,
        keyboard::{Key, KeyCode, KeyLocation, ModifiersState, PhysicalKey},
    },
};

use crate::spacer::spacer;

use super::{super::toolbar::toolbar_button, navigation_stack};

#[derive(Clone, Copy)]
struct Controls {
    path: State<Vec<String>>,
    sidebar: State<u32>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let path = remember(Vec::<String>::new);
    let sidebar = remember(|| 0u32);
    probe.set(Some(Controls { path, sidebar }));
    navigation_stack("Library")
        .path(path.get())
        .on_back(move || {
            path.with_mut(|path| {
                path.pop();
            });
        })
        .leading(move || {
            toolbar_button()
                .label("Sidebar")
                .on_click(move || sidebar.set(sidebar.get() + 1));
        })
        .content(|| {
            spacer().modifier(
                Modifier::new()
                    .size(Dp(100.0), Dp(40.0))
                    .focusable()
                    .semantics(SemanticsArgs {
                        role: Some(Role::Button),
                        label: Some("Page".into()),
                        focusable: true,
                        ..Default::default()
                    }),
            );
        });
}

fn has(ui: &UiTest, label: &str, role: Role) -> bool {
    ui.inspect_accessibility()
        .expect("published semantics")
        .find(label, role)
        .is_some()
}

fn button(ui: &UiTest, label: &str) -> tessera_ui::accesskit::NodeId {
    ui.inspect_accessibility()
        .expect("published semantics")
        .find(label, Role::Button)
        .map(|(id, _)| id)
        .expect("button")
}

fn shows_title(ui: &UiTest, title: &str) -> bool {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .any(|(_, node)| node.label() == Some(title) && node.role() != Role::Button)
}

#[test]
fn the_toolbar_titles_the_current_page_and_goes_back_along_the_path() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(500, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    assert!(shows_title(&ui, "Library"));
    assert!(!has(&ui, "Back", Role::Button));
    let sidebar = button(&ui, "Sidebar");
    assert!(ui.accessibility_action(sidebar, Action::Click));
    ui.frames(2);
    assert_eq!(
        controls.sidebar.get(),
        1,
        "the first page shows the leading items"
    );

    controls.path.set(vec!["Albums".into(), "Summer".into()]);
    ui.frames(2);
    assert!(shows_title(&ui, "Summer"));
    assert!(!has(&ui, "Sidebar", Role::Button));
    let back = button(&ui, "Back");
    assert!(ui.accessibility_action(back, Action::Click));
    ui.frames(2);
    assert_eq!(controls.path.get(), ["Albums"]);
    assert!(shows_title(&ui, "Albums"));
}

#[test]
fn command_bracket_goes_back_from_inside_the_page() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(500, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    controls.path.set(vec!["Albums".into()]);
    ui.frames(2);
    let (page, _) = ui.accessibility_node("Page");
    assert!(ui.accessibility_action(page, Action::Focus));
    ui.frames(2);
    ui.set_key_modifiers(ModifiersState::SUPER);
    ui.inject_key_event(KeyboardEvent {
        physical_key: PhysicalKey::Code(KeyCode::BracketLeft),
        logical_key: Key::Character("[".into()),
        text: None,
        location: KeyLocation::Standard,
        state: ElementState::Pressed,
        repeat: false,
    });
    ui.frames(2);
    assert!(controls.path.get().is_empty());
    assert!(shows_title(&ui, "Library"));
}
