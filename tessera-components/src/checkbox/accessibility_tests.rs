//! Checkbox and switch state and accessibility regressions.
//!
//! ## Usage
//!
//! Preserve disabled controls, controlled values, and cancelled activation.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, FocusRequester, HeadlessConfig, HeadlessRenderer,
    MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
    remember, tessera,
};

use crate::{
    column::column, modifier::ModifierExt as _, radio_button::radio_button, switch::switch,
    theme::material_theme,
};

use super::{CheckboxController, checkbox};

thread_local! {
    static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CHECKED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CALLS: Cell<Option<State<[u32; 2]>>> = const { Cell::new(None) };
    static MIXED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static REQUESTERS: Cell<Option<[FocusRequester;3]>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let enabled = remember(|| false);
    let checked = remember(|| true);
    let calls = remember(|| [0u32; 2]);
    let mixed = remember(|| false);
    let requesters = remember(|| {
        [
            FocusRequester::new(),
            FocusRequester::new(),
            FocusRequester::new(),
        ]
    })
    .get();
    REQUESTERS.set(Some(requesters));
    let internal = remember(|| {
        let mut controller = CheckboxController::new(false);
        controller.set_indeterminate(true);
        controller
    });
    ENABLED.set(Some(enabled));
    CHECKED.set(Some(checked));
    CALLS.set(Some(calls));
    MIXED.set(Some(mixed));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().padding_all(Dp(16.0)))
            .children(move || {
                radio_button()
                    .focus_requester(requesters[2])
                    .accessibility_label("Before");
                checkbox()
                    .focus_requester(requesters[0])
                    .checked(checked.get())
                    .indeterminate(mixed.get())
                    .disabled(!enabled.get())
                    .accessibility_label("Checkbox")
                    .accessibility_description("Checkbox description")
                    .on_toggle(move |value| {
                        mixed.set(false);
                        checked.set(value);
                        calls.with_mut(|calls| calls[0] += 1);
                    });
                switch()
                    .focus_requester(requesters[1])
                    .checked(checked.get())
                    .enabled(enabled.get())
                    .accessibility_label("Switch")
                    .accessibility_description("Switch description")
                    .on_toggle(move |value| {
                        checked.set(value);
                        calls.with_mut(|calls| calls[1] += 1);
                    });
                radio_button().accessibility_label("After");
                checkbox()
                    .controller(internal)
                    .checkmark_color(Color::RED)
                    .accessibility_label("Internal mixed");
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

fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) -> bool {
    let target_node = node(renderer, label).0;
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    })
}

fn center(node: &Node) -> PxPosition {
    let rect = node.bounds().expect("bounds");
    PxPosition::from_f32_xy(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    )
}

fn assert_values(renderer: &HeadlessRenderer, checked: bool) {
    for label in ["Checkbox", "Switch"] {
        assert_eq!(
            node(renderer, label).1.toggled(),
            Some(if checked {
                Toggled::True
            } else {
                Toggled::False
            }),
            "{label}"
        );
    }
}

fn check_controls(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 320,
            height: 480,
            scale_factor: 1.5,
            sample_count: samples,
            frame_time: Duration::from_millis(16),
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(12);
    let allocations = crate::pipelines::checkmark::pipeline::ALLOCATIONS
        .load(std::sync::atomic::Ordering::Relaxed);
    let identities = [node(&renderer, "Checkbox").0, node(&renderer, "Switch").0];
    for (label, role, description, width) in [
        ("Checkbox", Role::CheckBox, "Checkbox description", 72.0),
        ("Switch", Role::Switch, "Switch description", 78.0),
    ] {
        let (_, control) = node(&renderer, label);
        assert_eq!(control.role(), role);
        assert_eq!(control.description(), Some(description));
        assert!(control.is_disabled());
        assert!(!control.supports_action(Action::Click));
        assert!(!control.supports_action(Action::Focus));
        let rect = control.bounds().unwrap();
        assert_eq!(rect.x1 - rect.x0, width);
        assert_eq!(rect.y1 - rect.y0, 72.0);
        assert!(!action(&mut renderer, label, Action::Click));
        let point = center(&control);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    }
    renderer.render_frames(12);
    assert_values(&renderer, true);
    assert_eq!(CALLS.get().unwrap().get(), [0, 0]);
    assert!(action(&mut renderer, "Before", Action::Focus));
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(12);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        node(&renderer, "After").0
    );
    CHECKED.get().unwrap().set(false);
    renderer.render_frames(30);
    assert_values(&renderer, false);
    ENABLED.get().unwrap().set(true);
    renderer.render_frames(12);
    for (requester, label) in REQUESTERS
        .get()
        .unwrap()
        .into_iter()
        .zip(["Checkbox", "Switch", "Before"])
    {
        requester.request_focus();
        renderer.render_frames(3);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            node(&renderer, label).0,
            "external focus must target the control semantics"
        );
        assert_eq!(
            CALLS.get().unwrap().get(),
            [0, 0],
            "focus requests must not activate controls"
        );
        assert_values(&renderer, false);
        assert_eq!(
            node(&renderer, "Before").1.toggled(),
            Some(Toggled::False),
            "requesting radio focus must not select it"
        );
    }
    for (index, label) in ["Checkbox", "Switch"].into_iter().enumerate() {
        let (id, control) = node(&renderer, label);
        assert_eq!(id, identities[index]);
        assert!(!control.is_disabled());
        assert!(control.supports_action(Action::Click));
        assert!(control.supports_action(Action::Focus));
        assert_eq!(
            control.bounds().unwrap().y1 - control.bounds().unwrap().y0,
            72.0
        );
    }
    let point = center(&node(&renderer, "Checkbox").1);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_values(&renderer, true);
    assert_eq!(CALLS.get().unwrap().get(), [1, 0]);
    assert!(action(&mut renderer, "Switch", Action::Click));
    renderer.render_frames(30);
    assert_values(&renderer, false);
    assert_eq!(CALLS.get().unwrap().get(), [1, 1]);
    let point = center(&node(&renderer, "Switch").1);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    ENABLED.get().unwrap().set(false);
    renderer.render_frames(12);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(30);
    assert_values(&renderer, false);
    assert_eq!(CALLS.get().unwrap().get(), [1, 1]);
    CHECKED.get().unwrap().set(true);
    renderer.render_frames(30);
    assert_values(&renderer, true);
    assert_eq!(CALLS.get().unwrap().get(), [1, 1]);
    ENABLED.get().unwrap().set(true);
    renderer.render_frames(12);
    assert!(action(&mut renderer, "Switch", Action::Click));
    renderer.render_frames(30);
    assert_values(&renderer, false);
    assert_eq!(CALLS.get().unwrap().get(), [1, 2]);
    MIXED.get().unwrap().set(true);
    renderer.render_frames(30);
    let (mixed_id, mixed_node) = node(&renderer, "Checkbox");
    assert_eq!(mixed_id, identities[0]);
    assert_eq!(mixed_node.toggled(), Some(Toggled::Mixed));
    let point = center(&mixed_node);
    let pixels = renderer.read_pixels().expect("mixed pixels");
    let pixel = |x: u32, y: u32| {
        let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
        &pixels[offset..offset + 3]
    };
    let x = point.x as u32;
    let y = point.y as u32;
    for x in [x - 4, x, x + 4] {
        assert!(
            pixel(x, y).iter().all(|channel| *channel > 200),
            "the partial-selection stroke must be horizontal at ({x}, {y}), got {:?}",
            pixel(x, y)
        );
    }
    assert!(
        pixel(x, y)[1] > pixel(x, y - 4)[1].saturating_add(20),
        "the dash must stay near the glyph center"
    );
    let mixed_green = pixel(x, y)[1];
    let internal_center = center(&node(&renderer, "Internal mixed").1);
    let internal_ink = pixel(internal_center.x as u32, internal_center.y as u32);
    assert!(internal_ink[0] > internal_ink[1].saturating_add(80));
    assert!(internal_ink[0] > internal_ink[2].saturating_add(80));
    ENABLED.get().unwrap().set(false);
    renderer.render_frames(12);
    assert_eq!(
        node(&renderer, "Checkbox").1.toggled(),
        Some(Toggled::Mixed)
    );
    assert!(!action(&mut renderer, "Checkbox", Action::Click));
    assert_eq!(CALLS.get().unwrap().get(), [1, 2]);
    ENABLED.get().unwrap().set(true);
    renderer.render_frames(12);
    assert!(action(&mut renderer, "Checkbox", Action::Click));
    renderer.render_frames(30);
    assert_values(&renderer, true);
    assert!(!MIXED.get().unwrap().get());
    assert_eq!(CALLS.get().unwrap().get(), [2, 2]);
    let checked_pixels = renderer.read_pixels().expect("checked pixels");
    let offset = ((y * renderer.physical_size().width + x + 4) * 4) as usize;
    assert!(
        checked_pixels[offset + 1].saturating_add(20) < mixed_green,
        "the selected glyph must morph from the dash to a checkmark"
    );
    MIXED.get().unwrap().set(true);
    renderer.render_frames(30);
    assert_eq!(
        node(&renderer, "Checkbox").1.toggled(),
        Some(Toggled::Mixed)
    );
    assert!(action(&mut renderer, "Checkbox", Action::Click));
    renderer.render_frames(30);
    assert_values(&renderer, true);
    assert_eq!(CALLS.get().unwrap().get(), [3, 2]);
    assert_eq!(
        node(&renderer, "Internal mixed").1.toggled(),
        Some(Toggled::Mixed)
    );
    assert!(action(&mut renderer, "Internal mixed", Action::Click));
    renderer.render_frames(30);
    assert_eq!(
        node(&renderer, "Internal mixed").1.toggled(),
        Some(Toggled::True)
    );
    assert!(action(&mut renderer, "Internal mixed", Action::Click));
    renderer.render_frames(30);
    assert_eq!(
        node(&renderer, "Internal mixed").1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        crate::pipelines::checkmark::pipeline::ALLOCATIONS
            .load(std::sync::atomic::Ordering::Relaxed),
        allocations,
        "selection changes must reuse instance pages across frames"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_disabled_boolean_controls_retain_semantics_values_and_targets_msaa1() {
    crate::test_utils::run_isolated_test(
        "checkbox::accessibility_tests::native_disabled_boolean_controls_retain_semantics_values_and_targets_msaa1",
        || check_controls(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_disabled_boolean_controls_retain_semantics_values_and_targets_msaa4() {
    crate::test_utils::run_isolated_test(
        "checkbox::accessibility_tests::native_disabled_boolean_controls_retain_semantics_values_and_targets_msaa4",
        || check_controls(4),
    );
}
