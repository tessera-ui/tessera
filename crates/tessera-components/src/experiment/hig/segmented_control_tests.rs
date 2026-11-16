use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Role, Toggled},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::{column::column, modifier::ModifierExt as _};

use super::{
    super::button::{button, button_scope},
    Segment, segmented_control,
};

#[derive(Clone, Copy)]
struct Controls {
    segments: State<Vec<Segment>>,
    selected: State<Option<usize>>,
    enabled: State<bool>,
    accept: State<bool>,
    width: State<Dp>,
    requests: State<Vec<usize>>,
    before: FocusRequester,
    after: FocusRequester,
    submits: State<u32>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let segments = remember(|| {
        vec![
            Segment::new("Day"),
            Segment::new("Week"),
            Segment::new("Month"),
        ]
    });
    let selected = remember(|| Some(0usize));
    let enabled = remember(|| true);
    let accept = remember(|| true);
    let width = remember(|| Dp(240.0));
    let requests = remember(Vec::new);
    let before = remember(FocusRequester::new).get();
    let after = remember(FocusRequester::new).get();
    let submits = remember(|| 0);
    probe.set(Some(Controls {
        segments,
        selected,
        enabled,
        accept,
        width,
        requests,
        before,
        after,
        submits,
    }));
    button_scope().child(move |scope| {
        column()
            .modifier(Modifier::new().background(Color::WHITE))
            .children(move || {
                button("Before").focus_requester(before);
                segmented_control(segments.get())
                    .selected_optional(selected.get())
                    .enabled(enabled.get())
                    .accessibility_label("Period")
                    .modifier(Modifier::new().width(width.get()))
                    .on_select(move |index| {
                        requests.with_mut(|requests| requests.push(index));
                        if accept.get() {
                            selected.set(Some(index));
                        }
                    });
                button("Save")
                    .scope(scope)
                    .default_action(true)
                    .focus_requester(after)
                    .on_click(move || submits.set(submits.get() + 1));
            });
    });
}

fn center(ui: &UiTest<'_>, label: &str) -> PxPosition {
    let (_, node) = ui.accessibility_node(label);
    let bounds = node.bounds().unwrap();
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    )
}

#[test]
fn pointer_selection_reselection_and_disabled_segments_use_caller_owned_values() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 160)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let week = center(&ui, "Week");
    ui.click_at(week);
    ui.click_at(week);
    assert_eq!(controls.requests.get(), [1, 1]);
    assert_eq!(controls.selected.get(), Some(1));
    let month = center(&ui, "Month");
    ui.pointer_press(MOUSE_POINTER_ID, month, PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.requests.get(), [1, 1]);
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(390.0, 150.0),
        PressKeyEventType::Left,
    );
    ui.frames(1);
    assert_eq!(controls.requests.get(), [1, 1]);
    controls.accept.set(false);
    controls.selected.set(None);
    ui.frames(1);
    ui.click_at(center(&ui, "Month"));
    assert_eq!(controls.selected.get(), None);
    assert_eq!(controls.requests.get(), [1, 1, 2]);
    assert_eq!(
        ui.accessibility_node("Month").1.toggled(),
        Some(Toggled::False)
    );
    controls.segments.with_mut(|items| items[1].enabled = false);
    ui.frames(1);
    ui.click_at(center(&ui, "Week"));
    assert_eq!(controls.requests.get(), [1, 1, 2]);
    let (_, week) = ui.accessibility_node("Week");
    assert!(week.is_disabled());
    assert!(!week.supports_action(Action::Click));
    controls.enabled.set(false);
    ui.frames(1);
    ui.click_at(center(&ui, "Day"));
    assert_eq!(controls.requests.get(), [1, 1, 2]);
    assert!(ui.accessibility_node("Period").1.is_disabled());
    assert!(ui.accessibility_node("Day").1.is_disabled());
}

fn check_shrinking_and_replacing_segments() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 160)
        .scale_factor(1.5)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    controls.width.set(Dp(241.0));
    ui.frames(1);
    let bounds = |ui: &UiTest<'_>, name| ui.accessibility_node(name).1.bounds().unwrap();
    let day = bounds(&ui, "Day");
    let week = bounds(&ui, "Week");
    let month = bounds(&ui, "Month");
    assert_eq!(day.x1, week.x0);
    assert_eq!(week.x1, month.x0);
    assert_eq!(month.x1, bounds(&ui, "Period").x1);
    controls.segments.set(vec![Segment::new("Only")]);
    ui.frames(1);
    let label_bounds = |ui: &UiTest<'_>| {
        ui.inspect_components()
            .nodes()
            .iter()
            .find(|node| {
                node.name() == "text_node"
                    && node
                        .semantics()
                        .is_some_and(|semantics| semantics.label.as_deref() == Some("Only"))
            })
            .unwrap()
            .bounds()
            .unwrap()
    };
    let label = label_bounds(&ui);
    controls.width.set(Dp((label.width + 49.0) as f64 / 1.5));
    ui.frames(1);
    let frame = ui.frames(1);
    let group = frame.node("Period").bounds();
    let label = label_bounds(&ui);
    assert_eq!(label.x - group.x.to_f32(), 25.0);
    controls.segments.set(vec![
        Segment::new("Day"),
        Segment::new("Week"),
        Segment::new("Month"),
    ]);
    controls.width.set(Dp(0.0));
    ui.frames(1);
    assert_eq!(ui.frames(1).node("Period").bounds().width.0, 0);
    controls.width.set(Dp(240.0));
    controls.segments.with_mut(|items| {
        items[0].label = "Daytime".into();
        items.truncate(2);
    });
    controls.selected.set(Some(2));
    ui.frames(1);
    assert_eq!(
        ui.accessibility_node("Daytime").1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        ui.accessibility_node("Week").1.toggled(),
        Some(Toggled::False)
    );
    ui.click_at(center(&ui, "Daytime"));
    assert_eq!(controls.requests.get(), [0]);
    assert_eq!(controls.selected.get(), Some(0));
    controls.segments.set(Vec::new());
    ui.frames(1);
    assert_eq!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::RadioButton)
            .count(),
        0
    );
}

#[test]
fn shrinking_and_replacing_segments_preserve_contiguous_bounds_and_current_actions() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::segmented_control::tests::shrinking_and_replacing_segments_preserve_contiguous_bounds_and_current_actions",
        check_shrinking_and_replacing_segments,
    );
}

fn key_up(ui: &mut UiTest<'_>, key: NamedKey) {
    ui.inject_key_event(KeyboardEvent::named(key, ElementState::Released));
    ui.frames(1);
}

fn check_keyboard_and_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 160)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().unwrap();
    controls.segments.with_mut(|items| items[1].enabled = false);
    ui.frames(1);
    let (month_id, _) = ui.accessibility_node("Month");
    let initial = ui.read_pixels();
    ui.run(|| controls.before.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Tab);
    ui.frames(1);
    key_up(&mut ui, NamedKey::Tab);
    assert_eq!(
        ui.accessibility_snapshot().unwrap().focus,
        ui.accessibility_node("Day").0
    );
    assert!(controls.requests.get().is_empty());
    assert_ne!(initial, ui.read_pixels());
    ui.press_key(NamedKey::ArrowRight);
    ui.frames(1);
    key_up(&mut ui, NamedKey::ArrowRight);
    assert_eq!(ui.accessibility_snapshot().unwrap().focus, month_id);
    assert_eq!(controls.selected.get(), Some(2));
    assert_eq!(controls.requests.get(), [2]);
    assert_ne!(initial, ui.read_pixels());
    ui.press_key(NamedKey::ArrowLeft);
    ui.frames(1);
    key_up(&mut ui, NamedKey::ArrowLeft);
    assert_eq!(controls.selected.get(), Some(0));
    ui.run(|| controls.after.request_focus());
    ui.frames(1);
    let count = controls.requests.get().len();
    assert!(ui.accessibility_action(month_id, Action::Focus));
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), count);
    assert_eq!(controls.selected.get(), Some(0));
    let focused = ui.read_pixels();
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), count);
    assert_ne!(focused, ui.read_pixels());
    let mut repeated = KeyboardEvent::named(NamedKey::Space, ElementState::Pressed);
    repeated.repeat = true;
    ui.inject_key_event(repeated);
    ui.frames(1);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.selected.get(), Some(2));
    assert_eq!(controls.requests.get().len(), count + 1);
    assert!(ui.accessibility_action(month_id, Action::Click));
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), count + 2);
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.submits.get(), 1);
    key_up(&mut ui, NamedKey::Enter);
    assert_eq!(controls.requests.get().len(), count + 2);
    ui.press_key(NamedKey::Tab);
    ui.frames(1);
    key_up(&mut ui, NamedKey::Tab);
    assert!(ui.run(|| controls.after.is_focused()));
    assert!(ui.accessibility_action(month_id, Action::Focus));
    ui.frames(1);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    controls.segments.with_mut(|items| {
        items.pop();
    });
    ui.frames(1);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get().len(), count + 2);
    assert_eq!(controls.selected.get(), Some(2));
    assert_eq!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::RadioButton)
            .count(),
        2
    );
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and pixels"]
fn keyboard_navigation_focus_actions_and_removal_use_the_segmented_control() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::segmented_control::tests::keyboard_navigation_focus_actions_and_removal_use_the_segmented_control",
        check_keyboard_and_pixels,
    );
}
