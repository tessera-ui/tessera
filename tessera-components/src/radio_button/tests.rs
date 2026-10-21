//! Radio semantics and directional navigation regressions.
//!
//! ## Usage
//!
//! Verify radio semantics, exclusive selection, and keyboard navigation.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, FocusProperties, HeadlessConfig, HeadlessRenderer,
    MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Orientation, Role, Toggled, TreeId},
    modifier::FocusModifierExt as _,
    remember, tessera,
};

use crate::{
    column::column,
    modifier::ModifierExt as _,
    theme::{MaterialTheme, material_theme},
};

use super::{RadioButtonController, RadioGroupOrientation, radio_button, radio_group};

thread_local! {
    static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
    static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CALLS: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static INK: Cell<Option<State<Option<Color>>>> = const { Cell::new(None) };
    static ORIENTATION: Cell<Option<State<RadioGroupOrientation>>> = const { Cell::new(None) };
    static SKIP_ALPHA: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static SHOW_MANAGED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static MANAGED_CALLS: Cell<Option<State<[u32; 3]>>> = const { Cell::new(None) };
    static PRESET: Cell<Option<State<RadioButtonController>>> = const { Cell::new(None) };
}

#[tessera]
fn managed_fixture() {
    let show = remember(|| true);
    let calls = remember(|| [0u32; 3]);
    let preset = remember(|| RadioButtonController::new(true));
    SHOW_MANAGED.set(Some(show));
    MANAGED_CALLS.set(Some(calls));
    PRESET.set(Some(preset));
    material_theme().child(move || {
        column().children(move || {
            radio_group()
                .accessibility_label("Managed choices")
                .content(move || {
                    column().children(move || {
                        radio_button()
                            .accessibility_label("One")
                            .on_select(move |_| calls.with_mut(|calls| calls[0] += 1));
                        if show.get() {
                            radio_button()
                                .accessibility_label("Two")
                                .on_select(move |_| calls.with_mut(|calls| calls[1] += 1));
                        }
                        radio_button()
                            .accessibility_label("Preset")
                            .controller(preset)
                            .on_select(move |_| calls.with_mut(|calls| calls[2] += 1));
                    });
                });
            radio_group()
                .accessibility_label("Independent choices")
                .content(|| {
                    column().children(|| {
                        radio_button().accessibility_label("Other");
                        radio_button().accessibility_label("Another");
                    });
                });
        });
    });
}

fn assert_checked(renderer: &HeadlessRenderer, labels: &[&str]) {
    let checked: Vec<_> = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| {
            node.role() == Role::RadioButton && node.toggled() == Some(Toggled::True)
        })
        .map(|(_, node)| node.label().expect("radio label").to_owned())
        .collect();
    assert_eq!(checked, labels);
}

fn check_managed_group(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(managed_fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 320,
            height: 400,
            scale_factor: 1.25,
            sample_count: samples,
            frame_time: Duration::from_millis(16),
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(15);
    assert_checked(&renderer, &["Preset"]);
    let bounds = node(&renderer, "One").1.bounds().expect("one bounds");
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_checked(&renderer, &["One"]);
    assert_eq!(MANAGED_CALLS.get().unwrap().get(), [1, 0, 0]);
    let two = node(&renderer, "Two").0;
    assert!(action(&mut renderer, two, Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Two"]);
    assert!(action(&mut renderer, two, Action::Click));
    renderer.render_frames(12);
    assert_eq!(MANAGED_CALLS.get().unwrap().get(), [1, 1, 0]);
    assert!(action_by_label(&mut renderer, "Other", Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Two", "Other"]);
    SHOW_MANAGED.get().unwrap().set(false);
    renderer.render_frames(15);
    assert_checked(&renderer, &["Other"]);
    assert!(action_by_label(&mut renderer, "One", Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["One", "Other"]);
    SHOW_MANAGED.get().unwrap().set(true);
    renderer.render_frames(15);
    assert_checked(&renderer, &["One", "Other"]);
    assert!(action_by_label(&mut renderer, "Two", Action::Focus));
    renderer.render_frames(12);
    assert_checked(&renderer, &["One", "Other"]);
    traverse(&mut renderer, FocusDirection::Down, "Preset");
    assert_checked(&renderer, &["Preset", "Other"]);
    assert_eq!(MANAGED_CALLS.get().unwrap().get(), [2, 1, 1]);
    assert!(action_by_label(&mut renderer, "Two", Action::Click));
    renderer.render_frames(30);
    PRESET
        .get()
        .unwrap()
        .with_mut(|controller| controller.set_selected(true));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Preset", "Other"]);
    assert_eq!(MANAGED_CALLS.get().unwrap().get(), [2, 2, 1]);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_radio_uncontrolled_groups_are_exclusive_and_release_removed_choices_msaa1() {
    crate::test_utils::run_isolated_test(
        "radio_button::tests::native_radio_uncontrolled_groups_are_exclusive_and_release_removed_choices_msaa1",
        || check_managed_group(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_radio_uncontrolled_groups_are_exclusive_and_release_removed_choices_msaa4() {
    crate::test_utils::run_isolated_test(
        "radio_button::tests::native_radio_uncontrolled_groups_are_exclusive_and_release_removed_choices_msaa4",
        || check_managed_group(4),
    );
}

#[tessera]
fn fixture() {
    let selected = remember(|| 1usize);
    let enabled = remember(|| false);
    let calls = remember(|| 0u32);
    let ink = remember(|| None::<Color>);
    let orientation = remember(RadioGroupOrientation::default);
    let skip_alpha = remember(|| false);
    SELECTED.set(Some(selected));
    ENABLED.set(Some(enabled));
    CALLS.set(Some(calls));
    INK.set(Some(ink));
    ORIENTATION.set(Some(orientation));
    SKIP_ALPHA.set(Some(skip_alpha));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.primary = Color::RED;
            theme.color_scheme.on_surface = Color::BLUE;
            theme.color_scheme.on_surface_variant = Color::GREEN;
            theme
        })
        .child(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(Color::WHITE)
                        .padding_all(Dp(16.0)),
                )
                .children(move || {
                    radio_button().selected(false).accessibility_label("Before");
                    radio_group()
                        .orientation(orientation.get())
                        .accessibility_label("Choices")
                        .accessibility_description("Choose one option")
                        .content(move || {
                            column().children(move || {
                                for (index, label) in
                                    ["Alpha", "Beta", "Gamma"].into_iter().enumerate()
                                {
                                    radio_button()
                                        .modifier(if index == 0 && skip_alpha.get() {
                                            Modifier::new().focusable().focus_properties(
                                                FocusProperties::new().skip_traversal(true),
                                            )
                                        } else {
                                            Modifier::new()
                                        })
                                        .selected(selected.get() == index)
                                        .enabled(index != 1 || enabled.get())
                                        .accessibility_label(label)
                                        .accessibility_description("Option description")
                                        .on_select(move |_| {
                                            selected.set(index);
                                            calls.with_mut(|calls| *calls += 1);
                                        });
                                }
                            });
                        });
                    radio_button()
                        .selected(false)
                        .unselected_color_optional(ink.get())
                        .accessibility_label("Standalone");
                });
        });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn action(renderer: &mut HeadlessRenderer, target: NodeId, action: Action) -> bool {
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: target,
        data: None,
    })
}

fn action_by_label(renderer: &mut HeadlessRenderer, label: &str, requested: Action) -> bool {
    let target = node(renderer, label).0;
    action(renderer, target, requested)
}

fn traverse(renderer: &mut HeadlessRenderer, direction: FocusDirection, label: &str) {
    FocusManager::current().request_move_focus(direction);
    renderer.render_frames(30);
    assert_eq!(
        renderer.accessibility_snapshot().expect("focus").focus,
        node(renderer, label).0
    );
}

fn center_pixel(renderer: &HeadlessRenderer, node: &Node) -> [u8; 4] {
    let bounds = node.bounds().expect("radio bounds");
    let x = ((bounds.x0 + bounds.x1) / 2.0) as u32;
    let y = ((bounds.y0 + bounds.y1) / 2.0) as u32;
    let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
    renderer.read_pixels().expect("pixels")[offset..offset + 4]
        .try_into()
        .unwrap()
}

fn check_radios(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 300,
            height: 440,
            scale_factor: 1.5,
            sample_count: samples,
            frame_time: Duration::from_millis(16),
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(12);
    let (group_id, group) = node(&renderer, "Choices");
    assert_eq!(group.role(), Role::RadioGroup);
    assert_eq!(group.orientation(), Some(Orientation::Vertical));
    assert_eq!(group.description(), Some("Choose one option"));
    let (alpha, _) = node(&renderer, "Alpha");
    let (beta, disabled) = node(&renderer, "Beta");
    let (gamma, _) = node(&renderer, "Gamma");
    assert_eq!(group.children(), &[alpha, beta, gamma]);
    assert_eq!(disabled.role(), Role::RadioButton);
    assert_eq!(disabled.description(), Some("Option description"));
    assert_eq!(disabled.toggled(), Some(Toggled::True));
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    assert!(!disabled.supports_action(Action::Focus));
    let bounds = disabled.bounds().expect("disabled target");
    assert_eq!(bounds.x1 - bounds.x0, 72.0);
    assert_eq!(bounds.y1 - bounds.y0, 72.0);
    let disabled_ink = center_pixel(&renderer, &disabled);
    assert_eq!(disabled_ink, [206, 206, 255, 255]);
    assert!(!action(&mut renderer, beta, Action::Click));
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(12);
    assert_eq!(CALLS.get().unwrap().get(), 0);
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    assert!(action(&mut renderer, alpha, Action::Focus));
    renderer.render_frames(12);
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    assert_eq!(CALLS.get().unwrap().get(), 0);
    assert!(action(&mut renderer, alpha, Action::Click));
    renderer.render_frames(30);
    assert_eq!(SELECTED.get().unwrap().get(), 0);
    assert_eq!(
        center_pixel(&renderer, &node(&renderer, "Alpha").1),
        [255, 0, 0, 255]
    );
    assert_eq!(node(&renderer, "Beta").1.toggled(), Some(Toggled::False));
    traverse(&mut renderer, FocusDirection::Down, "Gamma");
    assert_eq!(SELECTED.get().unwrap().get(), 2);
    traverse(&mut renderer, FocusDirection::Down, "Alpha");
    assert_eq!(SELECTED.get().unwrap().get(), 0);
    traverse(&mut renderer, FocusDirection::Up, "Gamma");
    assert_eq!(SELECTED.get().unwrap().get(), 2);
    traverse(&mut renderer, FocusDirection::Right, "Alpha");
    assert_eq!(SELECTED.get().unwrap().get(), 0);
    traverse(&mut renderer, FocusDirection::Left, "Gamma");
    assert_eq!(SELECTED.get().unwrap().get(), 2);
    ORIENTATION
        .get()
        .unwrap()
        .set(RadioGroupOrientation::Horizontal);
    renderer.render_frames(12);
    assert_eq!(
        node(&renderer, "Choices").1.orientation(),
        Some(Orientation::Horizontal)
    );
    traverse(&mut renderer, FocusDirection::Right, "Alpha");
    traverse(&mut renderer, FocusDirection::Left, "Gamma");
    traverse(&mut renderer, FocusDirection::Down, "Alpha");
    traverse(&mut renderer, FocusDirection::Up, "Gamma");
    ENABLED.get().unwrap().set(true);
    renderer.render_frames(12);
    let (restored_id, restored) = node(&renderer, "Beta");
    assert_eq!(restored_id, beta);
    assert_eq!(node(&renderer, "Choices").0, group_id);
    assert!(!restored.is_disabled());
    assert!(restored.supports_action(Action::Focus));
    assert!(restored.supports_action(Action::Click));
    let before_drag = CALLS.get().unwrap().get();
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    assert_eq!(
        SELECTED.get().unwrap().get(),
        2,
        "pointer focus must wait for click completion"
    );
    let dragged = PxPosition::from_f32_xy(250.0, point.y);
    renderer.pointer_move(MOUSE_POINTER_ID, dragged);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, dragged, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_eq!(SELECTED.get().unwrap().get(), 2);
    assert_eq!(
        CALLS.get().unwrap().get(),
        before_drag,
        "a cancelled pointer click must not select"
    );
    assert!(action(&mut renderer, gamma, Action::Focus));
    renderer.render_frames(30);
    traverse(&mut renderer, FocusDirection::Up, "Beta");
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    assert_eq!(node(&renderer, "Beta").1.toggled(), Some(Toggled::True));
    let calls = CALLS.get().unwrap().get();
    assert!(action(&mut renderer, beta, Action::Click));
    renderer.render_frames(12);
    assert_eq!(
        CALLS.get().unwrap().get(),
        calls,
        "selected radios do not activate twice"
    );
    let (standalone, unselected) = node(&renderer, "Standalone");
    let bounds = unselected.bounds().expect("standalone bounds");
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let ring_offset =
        ((point.y as u32 * renderer.physical_size().width + point.x as u32 + 13) * 4) as usize;
    let pixels = renderer.read_pixels().expect("unselected ring");
    assert_eq!(&pixels[ring_offset..ring_offset + 4], &[0, 255, 0, 255]);
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.render_frames(30);
    let pixels = renderer.read_pixels().expect("hovered ring");
    assert_eq!(&pixels[ring_offset..ring_offset + 4], &[0, 0, 255, 255]);
    assert!(action(&mut renderer, standalone, Action::Focus));
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(290.0, 310.0));
    renderer.render_frames(30);
    let pixels = renderer.read_pixels().expect("focused ring");
    assert_eq!(&pixels[ring_offset..ring_offset + 4], &[0, 0, 255, 255]);
    assert_eq!(
        node(&renderer, "Standalone").1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    INK.get().unwrap().set(Some(Color::RED));
    renderer.render_frames(30);
    let pixels = renderer.read_pixels().expect("custom focused ring");
    assert_eq!(&pixels[ring_offset..ring_offset + 4], &[255, 0, 0, 255]);
    let bounds = node(&renderer, "Alpha").1.bounds().expect("alpha bounds");
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let before_click = CALLS.get().unwrap().get();
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_eq!(SELECTED.get().unwrap().get(), 0);
    assert_eq!(CALLS.get().unwrap().get(), before_click + 1);

    SELECTED.get().unwrap().set(2);
    renderer.render_frames(30);
    let before = node(&renderer, "Before").0;
    assert!(action(&mut renderer, before, Action::Focus));
    renderer.render_frames(12);
    traverse(&mut renderer, FocusDirection::Next, "Gamma");
    traverse(&mut renderer, FocusDirection::Next, "Standalone");
    traverse(&mut renderer, FocusDirection::Previous, "Gamma");
    traverse(&mut renderer, FocusDirection::Previous, "Before");
    SELECTED.get().unwrap().set(usize::MAX);
    renderer.render_frames(30);
    traverse(&mut renderer, FocusDirection::Next, "Alpha");
    assert_eq!(SELECTED.get().unwrap().get(), usize::MAX);
    traverse(&mut renderer, FocusDirection::Right, "Beta");
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    ENABLED.get().unwrap().set(false);
    renderer.render_frames(30);
    assert_eq!(
        renderer
            .accessibility_snapshot()
            .expect("recovered focus")
            .focus,
        alpha
    );
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    traverse(&mut renderer, FocusDirection::Next, "Standalone");
    SKIP_ALPHA.get().unwrap().set(true);
    renderer.render_frames(12);
    assert!(action(&mut renderer, before, Action::Focus));
    renderer.render_frames(12);
    traverse(&mut renderer, FocusDirection::Next, "Gamma");
    assert_eq!(SELECTED.get().unwrap().get(), 1);
    assert_eq!(FocusManager::current().focus_change_direction(), None);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_radio_disabled_semantics_and_wrapped_navigation_msaa1() {
    crate::test_utils::run_isolated_test(
        "radio_button::tests::native_radio_disabled_semantics_and_wrapped_navigation_msaa1",
        || check_radios(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_radio_disabled_semantics_and_wrapped_navigation_msaa4() {
    crate::test_utils::run_isolated_test(
        "radio_button::tests::native_radio_disabled_semantics_and_wrapped_navigation_msaa4",
        || check_radios(4),
    );
}
