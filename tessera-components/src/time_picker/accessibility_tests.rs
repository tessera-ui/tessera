//! Published time-picker controls and assistive-technology actions.
//!
//! ## Usage
//!
//! Verify clock values, focus, periods, and manual-entry transitions.

use std::cell::Cell;

use tessera_ui::{
    FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
    remember, tessera,
    winit::event::Ime,
};

use crate::{
    button::button,
    column::column,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    surface::surface,
    text::text,
    theme::material_theme,
};

use super::{
    DayPeriod, TimePickerDisplayMode, TimePickerLayout, TimePickerSelection, TimePickerState,
    time_picker_dialog,
};

thread_local! {
    static STATE: Cell<Option<State<TimePickerState>>> = const { Cell::new(None) };
    static STEPS: Cell<Option<State<(u8, u8)>>> = const { Cell::new(None) };
    static LAYOUT: Cell<Option<State<TimePickerLayout>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let state = remember(|| TimePickerState::new(14, 8, false, TimePickerDisplayMode::Picker));
    let steps = remember(|| (1, 1));
    let arrangement = remember(TimePickerLayout::default);
    STATE.set(Some(state));
    STEPS.set(Some(steps));
    LAYOUT.set(Some(arrangement));
    material_theme().child(move || {
        column().children(move || {
            surface()
                .modifier(Modifier::new().padding(Padding::all(tessera_ui::Dp(24.0))))
                .child(move || {
                    time_picker_dialog()
                        .state(state)
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
                                    text().content("OK");
                                });
                        });
                });
        });
    });
}

fn control(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| panic!("missing {label} ({role:?})"))
}

fn action(renderer: &mut HeadlessRenderer, id: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }));
    renderer.render_frames(8);
}

fn focused_label(renderer: &HeadlessRenderer) -> Option<String> {
    let tree = renderer.accessibility_snapshot().expect("tree");
    control(&renderer, "Meeting time", Role::Group);
    tree.nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .and_then(|(_, node)| node.label().map(str::to_owned))
}

fn tap(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(8);
}

fn save(renderer: &mut HeadlessRenderer, name: &str, samples: u32) {
    if let Some(directory) = std::env::var_os("TESSERA_TIME_QA_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("QA directory");
        let size = renderer.physical_size();
        let pixels = renderer.read_pixels().expect("QA pixels");
        tessera_ui::headless::write_png_file(
            directory.join(format!("{name}-msaa{samples}.png")),
            size.width,
            size.height,
            &pixels,
        )
        .expect("QA image");
    }
}

fn check_accessibility(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 1000,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(8);
    let state = STATE.get().expect("state");
    let steps = STEPS.get().expect("steps");
    let tree = renderer.accessibility_snapshot().expect("tree");
    let slots: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| node.label().is_some_and(|label| label.starts_with("Hour ")))
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
    save(&mut renderer, "portrait", samples);
    let initial = renderer.read_pixels().expect("initial pixels");
    let header = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, header, Action::Focus);
    for label in ["Minute", "PM", "Hour 2 of 12"] {
        FocusManager::current().request_move_focus(FocusDirection::Next);
        renderer.render_frames(8);
        assert_eq!(focused_label(&renderer).as_deref(), Some(label));
        if label == "PM" {
            assert!(renderer.read_pixels().expect("period focus pixels") != initial);
        }
    }
    for label in ["PM", "Minute"] {
        FocusManager::current().request_move_focus(FocusDirection::Previous);
        renderer.render_frames(8);
        assert_eq!(focused_label(&renderer).as_deref(), Some(label));
    }
    assert_eq!(state.with(TimePickerState::hour), 14);
    FocusManager::current().clear_focus(true);
    renderer.render_frames(8);

    let third = control(&renderer, "Hour 3 of 12", Role::Button).0;
    action(&mut renderer, third, Action::Focus);
    assert_eq!(state.with(TimePickerState::hour), 14);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Hour 3 of 12"));
    assert!(renderer.read_pixels().expect("focus pixels") != initial);
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(8);
    assert_eq!(
        focused_label(&renderer).as_deref(),
        Some("Switch to time input")
    );
    FocusManager::current().request_move_focus(FocusDirection::Previous);
    renderer.render_frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Hour 2 of 12"));
    FocusManager::current().clear_focus(true);
    renderer.render_frames(8);
    assert!(renderer.read_pixels().expect("restored focus pixels") == initial);

    action(&mut renderer, third, Action::Click);
    assert_eq!(state.with(TimePickerState::hour), 15);
    assert_eq!(
        state.with(TimePickerState::selection),
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
    renderer.render_frames(8);
    action(&mut renderer, third, Action::Click);
    assert_eq!(
        state.with(TimePickerState::hour),
        15,
        "explicit value actions select their label independently of pointer snapping"
    );
    state.with_mut(|state| state.set_is_24_hour(true));
    renderer.render_frames(8);
    let slots = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.label().is_some_and(|label| label.starts_with("Hour ")))
        .count();
    assert_eq!(slots, 24);
    let last = control(&renderer, "Hour 23 of 24", Role::Button).0;
    action(&mut renderer, last, Action::Click);
    assert_eq!(state.with(TimePickerState::hour), 23);
    state.with_mut(|state| state.set_is_24_hour(false));
    renderer.render_frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Hour 11 of 12"));
    assert_eq!(
        control(&renderer, "Hour 11 of 12", Role::Button)
            .1
            .is_selected(),
        Some(true)
    );
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(8);
    assert_eq!(
        focused_label(&renderer).as_deref(),
        Some("Switch to time input")
    );
    FocusManager::current().request_move_focus(FocusDirection::Previous);
    renderer.render_frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Hour 11 of 12"));
    state.with_mut(|state| state.set_is_24_hour(true));
    renderer.render_frames(8);
    let midnight = control(&renderer, "Hour 0 of 24", Role::Button).0;
    action(&mut renderer, midnight, Action::Click);
    assert_eq!(state.with(TimePickerState::hour), 0);
    save(&mut renderer, "24-hour", samples);

    let minute = control(&renderer, "Minute", Role::Button).0;
    action(&mut renderer, minute, Action::Click);
    assert_eq!(
        state.with(TimePickerState::selection),
        TimePickerSelection::Minute
    );
    let quarter = control(&renderer, "Minute 15 of 60", Role::Button).0;
    action(&mut renderer, quarter, Action::Focus);
    assert_eq!(state.with(TimePickerState::minute), 8);
    action(&mut renderer, quarter, Action::Click);
    assert_eq!(state.with(TimePickerState::minute), 15);
    assert_eq!(
        control(&renderer, "Minute", Role::Button).1.value(),
        Some("15")
    );
    state.with_mut(|state| {
        state.set_is_24_hour(false);
        state.set_hour(14);
    });
    renderer.render_frames(8);
    let am = control(&renderer, "AM", Role::RadioButton).0;
    action(&mut renderer, am, Action::Click);
    assert_eq!(state.with(TimePickerState::period), DayPeriod::Am);
    assert_eq!(state.with(TimePickerState::hour), 2);
    let pm = control(&renderer, "PM", Role::RadioButton).0;
    action(&mut renderer, pm, Action::Click);
    assert_eq!(state.with(TimePickerState::hour), 14);
    state.with_mut(|state| {
        state.set_hour(16);
        state.set_minute(8);
    });
    renderer.render_frames(8);
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
    assert_eq!(state.with(TimePickerState::hour), 14);
    assert_eq!(
        state.with(TimePickerState::selection),
        TimePickerSelection::Minute
    );
    assert_eq!(
        state.with(TimePickerState::minute),
        8,
        "hour release must not also activate a minute slot"
    );
    assert_eq!(focused_label(&renderer).as_deref(), Some("Minute 10 of 60"));
    tap(&mut renderer, point);
    assert_eq!(state.with(TimePickerState::minute), 14);
    assert_eq!(
        control(&renderer, "Minute 15 of 60", Role::Button).0,
        quarter
    );
    action(&mut renderer, quarter, Action::Click);
    assert_eq!(state.with(TimePickerState::minute), 15);
    steps.set((1, 1));
    renderer.render_frames(8);
    let hour = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, hour, Action::Click);
    tap(&mut renderer, point);
    assert_eq!(
        state.with(TimePickerState::hour),
        15,
        "pointer snapping reads updated props"
    );
    assert_eq!(
        state.with(TimePickerState::selection),
        TimePickerSelection::Minute
    );
    assert_eq!(state.with(TimePickerState::minute), 15);

    let toggle = control(&renderer, "Switch to time input", Role::Button).0;
    action(&mut renderer, toggle, Action::Click);
    let hour = control(&renderer, "Hour", Role::TextInput).0;
    control(&renderer, "Minute", Role::TextInput);
    action(&mut renderer, hour, Action::Focus);
    assert_eq!(
        state.with(TimePickerState::selection),
        TimePickerSelection::Hour
    );
    renderer.inject_ime_event(Ime::Commit("7".to_owned()));
    renderer.render_frames(8);
    assert_eq!(state.with(TimePickerState::hour), 19);
    let minute = control(&renderer, "Minute", Role::TextInput).0;
    action(&mut renderer, minute, Action::Focus);
    renderer.inject_ime_event(Ime::Commit("56".to_owned()));
    renderer.render_frames(8);
    assert_eq!(state.with(TimePickerState::minute), 56);
    save(&mut renderer, "input", samples);
    let toggle = control(&renderer, "Switch to clock", Role::Button).0;
    action(&mut renderer, toggle, Action::Click);
    control(&renderer, "Minute 55 of 60", Role::Button);
    state.with_mut(|state| {
        state.set_hour(14);
        state.selection = TimePickerSelection::Hour;
    });
    renderer.render_frames(8);
    let pm = control(&renderer, "PM", Role::RadioButton).0;
    action(&mut renderer, pm, Action::Focus);
    let arrangement = LAYOUT.get().expect("arrangement");
    arrangement.set(TimePickerLayout::Horizontal);
    renderer.resize(1000, 1000);
    renderer.render_frames(8);
    assert_eq!(control(&renderer, "PM", Role::RadioButton).0, pm);
    assert_eq!(focused_label(&renderer).as_deref(), Some("PM"));
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
        state.with(TimePickerState::hour),
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
        state.with(TimePickerState::hour),
        14,
        "the hit target extends below the visual group"
    );
    save(&mut renderer, "horizontal", samples);
    renderer.resize(600, 1000);
    renderer.render_frames(8);
    assert_eq!(control(&renderer, "PM", Role::RadioButton).0, pm);
    assert_eq!(focused_label(&renderer).as_deref(), Some("PM"));
    let bounds = control(&renderer, "PM", Role::RadioButton)
        .1
        .bounds()
        .expect("PM bounds");
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (78.0, 72.0));
    assert_eq!(state.with(TimePickerState::hour), 14);
    assert_eq!(state.with(TimePickerState::minute), 56);
    renderer.resize(1000, 1000);
    renderer.render_frames(8);
    state.with_mut(|state| state.set_is_24_hour(true));
    renderer.render_frames(8);
    let bounds = control(&renderer, "Hour", Role::Button)
        .1
        .bounds()
        .expect("hour bounds");
    assert_eq!(bounds.x1 - bounds.x0, 144.0);
    let hour = control(&renderer, "Hour", Role::Button).0;
    action(&mut renderer, hour, Action::Click);
    let seven = control(&renderer, "Hour 19 of 24", Role::Button).0;
    action(&mut renderer, seven, Action::Click);
    assert_eq!(state.with(TimePickerState::hour), 19);
    save(&mut renderer, "horizontal-24-hour", samples);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_clock_values_periods_focus_and_actions() {
    crate::test_utils::run_isolated_test(
        "time_picker::accessibility_tests::gpu_clock_values_periods_focus_and_actions",
        || check_accessibility(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_clock_values_periods_focus_and_actions_msaa() {
    crate::test_utils::run_isolated_test(
        "time_picker::accessibility_tests::gpu_clock_values_periods_focus_and_actions_msaa",
        || check_accessibility(4),
    );
}
