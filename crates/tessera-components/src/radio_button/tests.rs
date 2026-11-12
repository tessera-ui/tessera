//! Radio semantics and directional navigation regressions.
//!
//! ## Usage
//!
//! Verify radio semantics, exclusive selection, and keyboard navigation.

use std::time::Duration;

use tessera_ui::{
    Color, Dp, FocusDirection, FocusProperties, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Orientation, Role, Toggled, TreeId},
    focus,
    headless::{HeadlessConfig, HeadlessRenderer},
    modifier::FocusModifierExt as _,
    remember, tessera,
    testing::TestProbe,
};

use crate::{
    column::column,
    modifier::ModifierExt as _,
    theme::{MaterialTheme, material_theme},
};

use super::{RadioGroupOrientation, radio_button, radio_group};

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    selected: Option<State<usize>>,
    enabled: Option<State<bool>>,
    calls: Option<State<u32>>,
    ink: Option<State<Option<Color>>>,
    orientation: Option<State<RadioGroupOrientation>>,
    skip_alpha: Option<State<bool>>,
    show_managed: Option<State<bool>>,
    managed_calls: Option<State<[u32; 3]>>,
    managed_selection: Option<State<Option<usize>>>,
}

#[tessera]
fn managed_fixture(observed: TestProbe<FixtureHandles>) {
    let show = remember(|| true);
    let calls = remember(|| [0u32; 3]);
    let selected = remember(|| Some(2usize));
    let other = remember(|| None::<usize>);
    observed.with_mut(|handles| handles.show_managed = Some(show));
    observed.with_mut(|handles| handles.managed_calls = Some(calls));
    observed.with_mut(|handles| handles.managed_selection = Some(selected));
    material_theme().child(move || {
        column().children(move || {
            radio_group()
                .accessibility_label("Managed choices")
                .content(move |group| {
                    column().children(move || {
                        let select = move |index: usize| {
                            selected.set(Some(index));
                            calls.with_mut(|calls| calls[index] += 1);
                        };
                        radio_button(selected.get() == Some(0))
                            .group(group)
                            .accessibility_label("One")
                            .on_select(move |_| select(0));
                        if show.get() {
                            radio_button(selected.get() == Some(1))
                                .group(group)
                                .accessibility_label("Two")
                                .on_select(move |_| select(1));
                        }
                        radio_button(selected.get() == Some(2))
                            .group(group)
                            .accessibility_label("Preset")
                            .on_select(move |_| select(2));
                    });
                });
            radio_group()
                .accessibility_label("Independent choices")
                .content(move |group| {
                    column().children(move || {
                        for (index, label) in ["Other", "Another"].into_iter().enumerate() {
                            radio_button(other.get() == Some(index))
                                .group(group)
                                .accessibility_label(label)
                                .on_select(move |_| other.set(Some(index)));
                        }
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
    let observed = TestProbe::default();
    let fixture_observed = observed.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            managed_fixture(fixture_observed.clone());
        }),
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
    assert_eq!(observed.get().managed_calls.unwrap().get(), [1, 0, 0]);
    let two = node(&renderer, "Two").0;
    assert!(action(&mut renderer, two, Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Two"]);
    assert!(action(&mut renderer, two, Action::Click));
    renderer.render_frames(12);
    assert_eq!(observed.get().managed_calls.unwrap().get(), [1, 1, 0]);
    assert!(action_by_label(&mut renderer, "Other", Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Two", "Other"]);
    observed.get().show_managed.unwrap().set(false);
    renderer.render_frames(15);
    assert_checked(&renderer, &["Other"]);
    assert!(action_by_label(&mut renderer, "One", Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["One", "Other"]);
    observed.get().show_managed.unwrap().set(true);
    renderer.render_frames(15);
    assert_checked(&renderer, &["One", "Other"]);
    assert!(action_by_label(&mut renderer, "Two", Action::Focus));
    renderer.render_frames(12);
    assert_checked(&renderer, &["One", "Other"]);
    traverse(&mut renderer, FocusDirection::Down, "Preset");
    assert_checked(&renderer, &["Preset", "Other"]);
    assert_eq!(observed.get().managed_calls.unwrap().get(), [2, 1, 1]);
    assert!(action_by_label(&mut renderer, "Two", Action::Click));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Two", "Other"]);
    observed.get().managed_selection.unwrap().set(Some(2));
    renderer.render_frames(30);
    assert_checked(&renderer, &["Preset", "Other"]);
    assert_eq!(observed.get().managed_calls.unwrap().get(), [2, 2, 1]);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_radio_groups_follow_caller_selection_and_release_removed_choices_msaa1() {
    crate::test_utils::run_isolated_test(
        "radio_button::tests::native_radio_groups_follow_caller_selection_and_release_removed_choices_msaa1",
        || check_managed_group(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_radio_groups_follow_caller_selection_and_release_removed_choices_msaa4() {
    crate::test_utils::run_isolated_test(
        "radio_button::tests::native_radio_groups_follow_caller_selection_and_release_removed_choices_msaa4",
        || check_managed_group(4),
    );
}

#[tessera]
fn fixture(observed: TestProbe<FixtureHandles>) {
    let selected = remember(|| 1usize);
    let enabled = remember(|| false);
    let calls = remember(|| 0u32);
    let ink = remember(|| None::<Color>);
    let orientation = remember(RadioGroupOrientation::default);
    let skip_alpha = remember(|| false);
    observed.with_mut(|handles| handles.selected = Some(selected));
    observed.with_mut(|handles| handles.enabled = Some(enabled));
    observed.with_mut(|handles| handles.calls = Some(calls));
    observed.with_mut(|handles| handles.ink = Some(ink));
    observed.with_mut(|handles| handles.orientation = Some(orientation));
    observed.with_mut(|handles| handles.skip_alpha = Some(skip_alpha));
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
                    radio_button(false).accessibility_label("Before");
                    radio_group()
                        .orientation(orientation.get())
                        .accessibility_label("Choices")
                        .accessibility_description("Choose one option")
                        .content(move |group| {
                            column().children(move || {
                                for (index, label) in
                                    ["Alpha", "Beta", "Gamma"].into_iter().enumerate()
                                {
                                    radio_button(selected.get() == index)
                                        .group(group)
                                        .modifier(if index == 0 && skip_alpha.get() {
                                            Modifier::new().focusable().focus_properties(
                                                FocusProperties::new().skip_traversal(true),
                                            )
                                        } else {
                                            Modifier::new()
                                        })
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
                    radio_button(false)
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
    focus::request_move_focus(direction);
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
    let observed = TestProbe::default();
    let fixture_observed = observed.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            fixture(fixture_observed.clone());
        }),
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
    assert_eq!(observed.get().calls.unwrap().get(), 0);
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    assert!(action(&mut renderer, alpha, Action::Focus));
    renderer.render_frames(12);
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    assert_eq!(observed.get().calls.unwrap().get(), 0);
    assert!(action(&mut renderer, alpha, Action::Click));
    renderer.render_frames(30);
    assert_eq!(observed.get().selected.unwrap().get(), 0);
    assert_eq!(
        center_pixel(&renderer, &node(&renderer, "Alpha").1),
        [255, 0, 0, 255]
    );
    assert_eq!(node(&renderer, "Beta").1.toggled(), Some(Toggled::False));
    traverse(&mut renderer, FocusDirection::Down, "Gamma");
    assert_eq!(observed.get().selected.unwrap().get(), 2);
    traverse(&mut renderer, FocusDirection::Down, "Alpha");
    assert_eq!(observed.get().selected.unwrap().get(), 0);
    traverse(&mut renderer, FocusDirection::Up, "Gamma");
    assert_eq!(observed.get().selected.unwrap().get(), 2);
    traverse(&mut renderer, FocusDirection::Right, "Alpha");
    assert_eq!(observed.get().selected.unwrap().get(), 0);
    traverse(&mut renderer, FocusDirection::Left, "Gamma");
    assert_eq!(observed.get().selected.unwrap().get(), 2);
    observed
        .get()
        .orientation
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
    observed.get().enabled.unwrap().set(true);
    renderer.render_frames(12);
    let (restored_id, restored) = node(&renderer, "Beta");
    assert_eq!(restored_id, beta);
    assert_eq!(node(&renderer, "Choices").0, group_id);
    assert!(!restored.is_disabled());
    assert!(restored.supports_action(Action::Focus));
    assert!(restored.supports_action(Action::Click));
    let before_drag = observed.get().calls.unwrap().get();
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    assert_eq!(
        observed.get().selected.unwrap().get(),
        2,
        "pointer focus must wait for click completion"
    );
    let dragged = PxPosition::from_f32_xy(250.0, point.y);
    renderer.pointer_move(MOUSE_POINTER_ID, dragged);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, dragged, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_eq!(observed.get().selected.unwrap().get(), 2);
    assert_eq!(
        observed.get().calls.unwrap().get(),
        before_drag,
        "a cancelled pointer click must not select"
    );
    assert!(action(&mut renderer, gamma, Action::Focus));
    renderer.render_frames(30);
    traverse(&mut renderer, FocusDirection::Up, "Beta");
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    assert_eq!(node(&renderer, "Beta").1.toggled(), Some(Toggled::True));
    let calls = observed.get().calls.unwrap().get();
    assert!(action(&mut renderer, beta, Action::Click));
    renderer.render_frames(12);
    assert_eq!(
        observed.get().calls.unwrap().get(),
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
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    observed.get().ink.unwrap().set(Some(Color::RED));
    renderer.render_frames(30);
    let pixels = renderer.read_pixels().expect("custom focused ring");
    assert_eq!(&pixels[ring_offset..ring_offset + 4], &[255, 0, 0, 255]);
    let bounds = node(&renderer, "Alpha").1.bounds().expect("alpha bounds");
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let before_click = observed.get().calls.unwrap().get();
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_eq!(observed.get().selected.unwrap().get(), 0);
    assert_eq!(observed.get().calls.unwrap().get(), before_click + 1);

    observed.get().selected.unwrap().set(2);
    renderer.render_frames(30);
    let before = node(&renderer, "Before").0;
    assert!(action(&mut renderer, before, Action::Focus));
    renderer.render_frames(12);
    traverse(&mut renderer, FocusDirection::Next, "Gamma");
    traverse(&mut renderer, FocusDirection::Next, "Standalone");
    traverse(&mut renderer, FocusDirection::Previous, "Gamma");
    traverse(&mut renderer, FocusDirection::Previous, "Before");
    observed.get().selected.unwrap().set(usize::MAX);
    renderer.render_frames(30);
    traverse(&mut renderer, FocusDirection::Next, "Alpha");
    assert_eq!(observed.get().selected.unwrap().get(), usize::MAX);
    traverse(&mut renderer, FocusDirection::Right, "Beta");
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    observed.get().enabled.unwrap().set(false);
    renderer.render_frames(30);
    assert_eq!(
        renderer
            .accessibility_snapshot()
            .expect("recovered focus")
            .focus,
        alpha
    );
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    traverse(&mut renderer, FocusDirection::Next, "Standalone");
    observed.get().skip_alpha.unwrap().set(true);
    renderer.render_frames(12);
    assert!(action(&mut renderer, before, Action::Focus));
    renderer.render_frames(12);
    traverse(&mut renderer, FocusDirection::Next, "Gamma");
    assert_eq!(observed.get().selected.unwrap().get(), 1);
    assert_eq!(focus::focus_change_direction(), None);
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
