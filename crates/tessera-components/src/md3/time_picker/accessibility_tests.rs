use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    FocusDirection, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Node, NodeId, Role, Toggled},
    focus, remember, tessera,
    testing::{UiTest, text_matches, visible_text},
    winit::event::Ime,
};

use crate::{
    column::column,
    md3::{button::button, surface::surface, text::text, theme::material_theme},
};

use super::{
    DayPeriod, TimePickerController, TimePickerDisplayMode, TimePickerLayout, TimePickerSelection,
    time_picker_dialog,
};

#[tessera]
fn fixture(
    output: tessera_ui::testing::TestProbe<
        Option<(
            State<TimePickerController>,
            State<(u8, u8)>,
            State<TimePickerLayout>,
        )>,
    >,
) {
    let state = remember(|| TimePickerController::new(14, 8, false, TimePickerDisplayMode::Picker));
    let steps = remember(|| (1, 1));
    let arrangement = remember(TimePickerLayout::default);
    output.set(Some((state, steps, arrangement)));
    material_theme().child(move || {
        column().children(move || {
            surface()
                .modifier(Modifier::new().padding(Padding::all(tessera_ui::Dp(24.0))))
                .child(move || {
                    time_picker_dialog()
                        .controller(state)
                        .picker_modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some("Meeting time".to_owned()),
                            merge_descendants: true,
                            ..Default::default()
                        }))
                        .picker_hour_step(steps.get().0)
                        .picker_minute_step(steps.get().1)
                        .picker_layout(arrangement.get())
                        .confirm_button(|| {
                            button()
                                .text()
                                .accessibility_label("Confirm time")
                                .on_click(|| {})
                                .child(|| {
                                    text("OK");
                                });
                        });
                });
        });
    });
}

fn control(renderer: &UiTest, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| text_matches(node.label(), label) && node.role() == role)
        .unwrap_or_else(|| panic!("missing {label} ({role:?})"))
}

fn action(renderer: &mut UiTest, id: NodeId, action: Action) {
    assert!(renderer.accessibility_action(id, action));
    renderer.frames(8);
}

fn focused_label(renderer: &UiTest) -> Option<String> {
    let tree = renderer.accessibility_snapshot().expect("tree");
    control(&renderer, "Meeting time", Role::Group);
    tree.nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .and_then(|(_, node)| node.label().map(str::to_owned))
}

fn tap(renderer: &mut UiTest, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(1);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(8);
}

fn save(renderer: &mut UiTest, name: &str) {
    if let Some(directory) = std::env::var_os("TESSERA_TIME_QA_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("QA directory");
        let size = renderer.physical_size();
        let pixels = renderer.read_pixels();
        tessera_ui::headless::write_png_file(
            directory.join(format!("{name}.png")),
            size.width,
            size.height,
            &pixels,
        )
        .expect("QA image");
    }
}

fn check_accessibility() {
    let output = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(600, 1000)
        .scale_factor(1.5)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(output.clone()));
    renderer.frames(8);
    let state = output.get().unwrap().0;
    let steps = output.get().unwrap().1;
    let tree = renderer.accessibility_snapshot().expect("tree");
    let slots: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| {
            node.label()
                .map(visible_text)
                .is_some_and(|label| label.starts_with("Hour "))
        })
        .collect();
    assert_eq!(slots.len(), 12);
    assert_eq!(
        slots
            .iter()
            .filter(|(_, node)| node.is_selected() == Some(true))
            .count(),
        1
    );
    for (_, node) in slots {
        assert_eq!(node.role(), Role::Button);
        assert!(node.supports_action(Action::Click));
        assert!(node.supports_action(Action::Focus));
        assert_eq!(node.toggled(), None);
        let bounds = node.bounds().expect("slot bounds");
        assert!((bounds.x1 - bounds.x0 - 72.0).abs() <= 1.0);
        assert!((bounds.y1 - bounds.y0 - 72.0).abs() <= 1.0);
    }
    assert_eq!(
        control(&renderer, "Hour", Role::Button).1.value(),
        Some("02")
    );
    assert_eq!(
        control(&renderer, "Minute", Role::Button).1.value(),
        Some("08")
    );
    assert_eq!(
        control(&renderer, "AM", Role::RadioButton).1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        control(&renderer, "PM", Role::RadioButton).1.toggled(),
        Some(Toggled::True)
    );
    save(&mut renderer, "portrait");
    let initial = renderer.read_pixels();
    let header = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, header, Action::Focus);
    for label in ["Minute", "PM", "Hour 2 of 12"] {
        focus::request_move_focus(FocusDirection::Next);
        renderer.frames(8);
        assert!(text_matches(focused_label(&renderer).as_deref(), label));
        if label == "PM" {
            assert!(renderer.read_pixels() != initial);
        }
    }
    for label in ["PM", "Minute"] {
        focus::request_move_focus(FocusDirection::Previous);
        renderer.frames(8);
        assert!(text_matches(focused_label(&renderer).as_deref(), label));
    }
    assert_eq!(state.with(TimePickerController::hour), 14);
    focus::clear_focus(true);
    renderer.frames(8);

    let third = control(&renderer, "Hour 3 of 12", Role::Button).0;
    action(&mut renderer, third, Action::Focus);
    assert_eq!(state.with(TimePickerController::hour), 14);
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Hour 3 of 12"
    ));
    assert!(renderer.read_pixels() != initial);
    focus::request_move_focus(FocusDirection::Next);
    renderer.frames(8);
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Switch to time input"
    ));
    focus::request_move_focus(FocusDirection::Previous);
    renderer.frames(8);
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Hour 2 of 12"
    ));
    focus::clear_focus(true);
    renderer.frames(8);
    assert!(renderer.read_pixels() == initial);

    action(&mut renderer, third, Action::Click);
    assert_eq!(state.with(TimePickerController::hour), 15);
    assert_eq!(
        state.with(TimePickerController::selection),
        TimePickerSelection::Hour
    );
    assert_eq!(control(&renderer, "Hour 3 of 12", Role::Button).0, third);
    assert_eq!(
        control(&renderer, "Hour 3 of 12", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    assert_eq!(
        control(&renderer, "Hour 2 of 12", Role::Button)
            .1
            .is_selected(),
        Some(false)
    );
    steps.set((2, 7));
    renderer.frames(8);
    action(&mut renderer, third, Action::Click);
    assert_eq!(
        state.with(TimePickerController::hour),
        15,
        "explicit value actions select their label independently of pointer snapping"
    );
    state.with_mut(|state| state.set_is_24_hour(true));
    renderer.frames(8);
    let slots = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| {
            node.label()
                .map(visible_text)
                .is_some_and(|label| label.starts_with("Hour "))
        })
        .count();
    assert_eq!(slots, 24);
    let last = control(&renderer, "Hour 23 of 24", Role::Button).0;
    action(&mut renderer, last, Action::Click);
    assert_eq!(state.with(TimePickerController::hour), 23);
    state.with_mut(|state| state.set_is_24_hour(false));
    renderer.frames(8);
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Hour 11 of 12"
    ));
    assert_eq!(
        control(&renderer, "Hour 11 of 12", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    focus::request_move_focus(FocusDirection::Next);
    renderer.frames(8);
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Switch to time input"
    ));
    focus::request_move_focus(FocusDirection::Previous);
    renderer.frames(8);
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Hour 11 of 12"
    ));
    state.with_mut(|state| state.set_is_24_hour(true));
    renderer.frames(8);
    let midnight = control(&renderer, "Hour 0 of 24", Role::Button).0;
    action(&mut renderer, midnight, Action::Click);
    assert_eq!(state.with(TimePickerController::hour), 0);
    save(&mut renderer, "24-hour");

    let minute = control(&renderer, "Minute", Role::Button).0;
    action(&mut renderer, minute, Action::Click);
    assert_eq!(
        state.with(TimePickerController::selection),
        TimePickerSelection::Minute
    );
    let quarter = control(&renderer, "Minute 15 of 60", Role::Button).0;
    action(&mut renderer, quarter, Action::Focus);
    assert_eq!(state.with(TimePickerController::minute), 8);
    action(&mut renderer, quarter, Action::Click);
    assert_eq!(state.with(TimePickerController::minute), 15);
    assert_eq!(
        control(&renderer, "Minute", Role::Button).1.value(),
        Some("15")
    );
    state.with_mut(|state| {
        state.set_is_24_hour(false);
        state.set_hour(14);
    });
    renderer.frames(8);
    let am = control(&renderer, "AM", Role::RadioButton).0;
    action(&mut renderer, am, Action::Click);
    assert_eq!(state.with(TimePickerController::period), DayPeriod::Am);
    assert_eq!(state.with(TimePickerController::hour), 2);
    let pm = control(&renderer, "PM", Role::RadioButton).0;
    action(&mut renderer, pm, Action::Click);
    assert_eq!(state.with(TimePickerController::hour), 14);
    state.with_mut(|state| {
        state.set_hour(16);
        state.set_minute(8);
    });
    renderer.frames(8);
    let hour = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, hour, Action::Click);
    let clock = control(&renderer, "Hours", Role::Group)
        .1
        .bounds()
        .expect("clock");
    let point = PxPosition::from_f32_xy(
        ((clock.x0 + clock.x1) / 2.0 + 151.5) as f32,
        ((clock.y0 + clock.y1) / 2.0) as f32,
    );
    tap(&mut renderer, point);
    assert_eq!(state.with(TimePickerController::hour), 14);
    assert_eq!(
        state.with(TimePickerController::selection),
        TimePickerSelection::Minute
    );
    assert_eq!(
        state.with(TimePickerController::minute),
        8,
        "hour release must not also activate a minute slot"
    );
    assert!(text_matches(
        focused_label(&renderer).as_deref(),
        "Minute 10 of 60"
    ));
    tap(&mut renderer, point);
    assert_eq!(state.with(TimePickerController::minute), 14);
    assert_eq!(
        control(&renderer, "Minute 15 of 60", Role::Button).0,
        quarter
    );
    action(&mut renderer, quarter, Action::Click);
    assert_eq!(state.with(TimePickerController::minute), 15);
    steps.set((1, 1));
    renderer.frames(8);
    let hour = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, hour, Action::Click);
    tap(&mut renderer, point);
    assert_eq!(
        state.with(TimePickerController::hour),
        15,
        "pointer snapping reads updated props"
    );
    assert_eq!(
        state.with(TimePickerController::selection),
        TimePickerSelection::Minute
    );
    assert_eq!(state.with(TimePickerController::minute), 15);

    let toggle = control(&renderer, "Switch to time input", Role::Button).0;
    action(&mut renderer, toggle, Action::Click);
    let hour = control(&renderer, "Hour", Role::TextInput).0;
    control(&renderer, "Minute", Role::TextInput);
    action(&mut renderer, hour, Action::Focus);
    assert_eq!(
        state.with(TimePickerController::selection),
        TimePickerSelection::Hour
    );
    renderer.inject_ime_event(Ime::Commit("7".to_owned()));
    renderer.frames(8);
    assert_eq!(state.with(TimePickerController::hour), 19);
    let minute = control(&renderer, "Minute", Role::TextInput).0;
    action(&mut renderer, minute, Action::Focus);
    renderer.inject_ime_event(Ime::Commit("56".to_owned()));
    renderer.frames(8);
    assert_eq!(state.with(TimePickerController::minute), 56);
    save(&mut renderer, "input");
    let toggle = control(&renderer, "Switch to clock", Role::Button).0;
    action(&mut renderer, toggle, Action::Click);
    control(&renderer, "Minute 55 of 60", Role::Button);
    state.with_mut(|state| {
        state.set_hour(14);
        state.selection = TimePickerSelection::Hour;
    });
    renderer.frames(8);
    let pm = control(&renderer, "PM", Role::RadioButton).0;
    action(&mut renderer, pm, Action::Focus);
    let arrangement = output.get().unwrap().2;
    arrangement.set(TimePickerLayout::Horizontal);
    renderer.resize(1000, 1000);
    renderer.frames(8);
    assert_eq!(control(&renderer, "PM", Role::RadioButton).0, pm);
    assert!(text_matches(focused_label(&renderer).as_deref(), "PM"));
    let bounds = control(&renderer, "PM", Role::RadioButton)
        .1
        .bounds()
        .expect("PM bounds");
    assert_eq!(
        (bounds.x1 - bounds.x0, bounds.y1 - bounds.y0),
        (162.0, 72.0)
    );
    let clock = control(&renderer, "Hours", Role::Group)
        .1
        .bounds()
        .expect("clock");
    assert_eq!(clock.x0, 438.0);
    let am_bounds = control(&renderer, "AM", Role::RadioButton)
        .1
        .bounds()
        .expect("AM bounds");
    tap(
        &mut renderer,
        PxPosition::from_f32_xy(
            ((am_bounds.x0 + am_bounds.x1) / 2.0) as f32,
            (am_bounds.y0 + 1.0) as f32,
        ),
    );
    assert_eq!(
        state.with(TimePickerController::hour),
        2,
        "the hit target extends above the visual group"
    );
    tap(
        &mut renderer,
        PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            (bounds.y1 - 1.0) as f32,
        ),
    );
    assert_eq!(
        state.with(TimePickerController::hour),
        14,
        "the hit target extends below the visual group"
    );
    save(&mut renderer, "horizontal");
    renderer.resize(600, 1000);
    renderer.frames(8);
    assert_eq!(control(&renderer, "PM", Role::RadioButton).0, pm);
    assert!(text_matches(focused_label(&renderer).as_deref(), "PM"));
    let bounds = control(&renderer, "PM", Role::RadioButton)
        .1
        .bounds()
        .expect("PM bounds");
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (78.0, 72.0));
    assert_eq!(state.with(TimePickerController::hour), 14);
    assert_eq!(state.with(TimePickerController::minute), 56);
    renderer.resize(1000, 1000);
    renderer.frames(8);
    state.with_mut(|state| state.set_is_24_hour(true));
    renderer.frames(8);
    let bounds = control(&renderer, "Hour", Role::Button)
        .1
        .bounds()
        .expect("hour bounds");
    assert_eq!(bounds.x1 - bounds.x0, 144.0);
    let hour = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, hour, Action::Click);
    let seven = control(&renderer, "Hour 19 of 24", Role::Button).0;
    action(&mut renderer, seven, Action::Click);
    assert_eq!(state.with(TimePickerController::hour), 19);
    save(&mut renderer, "horizontal-24-hour");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_clock_values_periods_focus_and_actions() {
    crate::test_utils::run_isolated_test(
        "md3::time_picker::accessibility_tests::gpu_clock_values_periods_focus_and_actions",
        check_accessibility,
    );
}
