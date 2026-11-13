//! Checkbox and switch state and accessibility regressions.
//!
//! ## Usage
//!
//! Preserve disabled controls, controlled values, and cancelled activation.

use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, FocusDirection, FocusRequester, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Node, Role, Toggled},
    focus, remember, tessera,
    testing::UiTest,
};

use crate::{
    column::column,
    md3::{radio_button::radio_button, switch::switch, theme::material_theme},
};

use super::checkbox;

#[derive(Clone, Default)]
struct FixtureHandles {
    enabled: Option<State<bool>>,
    checked: Option<State<bool>>,
    calls: Option<State<[u32; 2]>>,
    mixed: Option<State<bool>>,
    requesters: Option<[FocusRequester; 3]>,
}

#[tessera]
fn fixture(probe: tessera_ui::testing::TestProbe<FixtureHandles>) {
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
    probe.with_mut(|handles| handles.requesters = Some(requesters));
    let internal = remember(|| (false, true));
    probe.with_mut(|handles| handles.enabled = Some(enabled));
    probe.with_mut(|handles| handles.checked = Some(checked));
    probe.with_mut(|handles| handles.calls = Some(calls));
    probe.with_mut(|handles| handles.mixed = Some(mixed));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().padding_all(Dp(16.0)))
            .children(move || {
                radio_button(false)
                    .focus_requester(requesters[2])
                    .accessibility_label("Before");
                checkbox(checked.get())
                    .focus_requester(requesters[0])
                    .indeterminate(mixed.get())
                    .disabled(!enabled.get())
                    .accessibility_label("Checkbox")
                    .accessibility_description("Checkbox description")
                    .on_toggle(move |value| {
                        mixed.set(false);
                        checked.set(value);
                        calls.with_mut(|calls| calls[0] += 1);
                    });
                switch(checked.get())
                    .focus_requester(requesters[1])
                    .enabled(enabled.get())
                    .accessibility_label("Switch")
                    .accessibility_description("Switch description")
                    .on_toggle(move |value| {
                        checked.set(value);
                        calls.with_mut(|calls| calls[1] += 1);
                    });
                radio_button(false).accessibility_label("After");
                checkbox(internal.get().0)
                    .indeterminate(internal.get().1)
                    .on_toggle(move |checked| internal.set((checked, false)))
                    .checkmark_color(Color::RED)
                    .accessibility_label("Internal mixed");
            });
    });
}

fn action(renderer: &mut UiTest, label: &str, action: Action) -> bool {
    let target_node = renderer.accessibility_node(label).0;
    renderer.accessibility_action(target_node, action)
}

fn center(node: &Node) -> PxPosition {
    let rect = node.bounds().expect("bounds");
    PxPosition::from_f32_xy(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    )
}

fn assert_values(renderer: &UiTest, checked: bool) {
    for label in ["Checkbox", "Switch"] {
        assert_eq!(
            renderer.accessibility_node(label).1.toggled(),
            Some(if checked {
                Toggled::True
            } else {
                Toggled::False
            }),
            "{label}"
        );
    }
}

fn check_controls() {
    let probe = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(320, 480)
        .scale_factor(1.5)
        .frame_step(Duration::from_millis(16))
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    renderer.frames(12);
    let allocations = renderer
        .graphics_work()
        .group("checkmark_instances")
        .buffer_allocations;
    let identities = [
        renderer.accessibility_node("Checkbox").0,
        renderer.accessibility_node("Switch").0,
    ];
    for (label, role, description, width) in [
        ("Checkbox", Role::CheckBox, "Checkbox description", 72.0),
        ("Switch", Role::Switch, "Switch description", 78.0),
    ] {
        let (_, control) = renderer.accessibility_node(label);
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
    renderer.frames(12);
    assert_values(&renderer, true);
    assert_eq!(probe.get().calls.unwrap().get(), [0, 0]);
    assert!(action(&mut renderer, "Before", Action::Focus));
    focus::request_move_focus(FocusDirection::Next);
    renderer.frames(12);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        renderer.accessibility_node("After").0
    );
    probe.get().checked.unwrap().set(false);
    renderer.frames(30);
    assert_values(&renderer, false);
    probe.get().enabled.unwrap().set(true);
    renderer.frames(12);
    for (requester, label) in probe
        .get()
        .requesters
        .unwrap()
        .into_iter()
        .zip(["Checkbox", "Switch", "Before"])
    {
        requester.request_focus();
        renderer.frames(3);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            renderer.accessibility_node(label).0,
            "external focus must target the control semantics"
        );
        assert_eq!(
            probe.get().calls.unwrap().get(),
            [0, 0],
            "focus requests must not activate controls"
        );
        assert_values(&renderer, false);
        assert_eq!(
            renderer.accessibility_node("Before").1.toggled(),
            Some(Toggled::False),
            "requesting radio focus must not select it"
        );
    }
    for (index, label) in ["Checkbox", "Switch"].into_iter().enumerate() {
        let (id, control) = renderer.accessibility_node(label);
        assert_eq!(id, identities[index]);
        assert!(!control.is_disabled());
        assert!(control.supports_action(Action::Click));
        assert!(control.supports_action(Action::Focus));
        assert_eq!(
            control.bounds().unwrap().y1 - control.bounds().unwrap().y0,
            72.0
        );
    }
    let point = center(&renderer.accessibility_node("Checkbox").1);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(30);
    assert_values(&renderer, true);
    assert_eq!(probe.get().calls.unwrap().get(), [1, 0]);
    assert!(action(&mut renderer, "Switch", Action::Click));
    renderer.frames(30);
    assert_values(&renderer, false);
    assert_eq!(probe.get().calls.unwrap().get(), [1, 1]);
    let point = center(&renderer.accessibility_node("Switch").1);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(1);
    probe.get().enabled.unwrap().set(false);
    renderer.frames(12);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(30);
    assert_values(&renderer, false);
    assert_eq!(probe.get().calls.unwrap().get(), [1, 1]);
    probe.get().checked.unwrap().set(true);
    renderer.frames(30);
    assert_values(&renderer, true);
    assert_eq!(probe.get().calls.unwrap().get(), [1, 1]);
    probe.get().enabled.unwrap().set(true);
    renderer.frames(12);
    assert!(action(&mut renderer, "Switch", Action::Click));
    renderer.frames(30);
    assert_values(&renderer, false);
    assert_eq!(probe.get().calls.unwrap().get(), [1, 2]);
    probe.get().mixed.unwrap().set(true);
    renderer.frames(30);
    let (mixed_id, mixed_node) = renderer.accessibility_node("Checkbox");
    assert_eq!(mixed_id, identities[0]);
    assert_eq!(mixed_node.toggled(), Some(Toggled::Mixed));
    let point = center(&mixed_node);
    let pixels = renderer.read_pixels();
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
    let internal_center = center(&renderer.accessibility_node("Internal mixed").1);
    let internal_ink = pixel(internal_center.x as u32, internal_center.y as u32);
    assert!(internal_ink[0] > internal_ink[1].saturating_add(80));
    assert!(internal_ink[0] > internal_ink[2].saturating_add(80));
    probe.get().enabled.unwrap().set(false);
    renderer.frames(12);
    assert_eq!(
        renderer.accessibility_node("Checkbox").1.toggled(),
        Some(Toggled::Mixed)
    );
    assert!(!action(&mut renderer, "Checkbox", Action::Click));
    assert_eq!(probe.get().calls.unwrap().get(), [1, 2]);
    probe.get().enabled.unwrap().set(true);
    renderer.frames(12);
    assert!(action(&mut renderer, "Checkbox", Action::Click));
    renderer.frames(30);
    assert_values(&renderer, true);
    assert!(!probe.get().mixed.unwrap().get());
    assert_eq!(probe.get().calls.unwrap().get(), [2, 2]);
    let checked_pixels = renderer.read_pixels();
    let offset = ((y * renderer.physical_size().width + x + 4) * 4) as usize;
    assert!(
        checked_pixels[offset + 1].saturating_add(20) < mixed_green,
        "the selected glyph must morph from the dash to a checkmark"
    );
    probe.get().mixed.unwrap().set(true);
    renderer.frames(30);
    assert_eq!(
        renderer.accessibility_node("Checkbox").1.toggled(),
        Some(Toggled::Mixed)
    );
    assert!(action(&mut renderer, "Checkbox", Action::Click));
    renderer.frames(30);
    assert_values(&renderer, true);
    assert_eq!(probe.get().calls.unwrap().get(), [3, 2]);
    assert_eq!(
        renderer.accessibility_node("Internal mixed").1.toggled(),
        Some(Toggled::Mixed)
    );
    assert!(action(&mut renderer, "Internal mixed", Action::Click));
    renderer.frames(30);
    assert_eq!(
        renderer.accessibility_node("Internal mixed").1.toggled(),
        Some(Toggled::True)
    );
    assert!(action(&mut renderer, "Internal mixed", Action::Click));
    renderer.frames(30);
    assert_eq!(
        renderer.accessibility_node("Internal mixed").1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        renderer
            .graphics_work()
            .group("checkmark_instances")
            .buffer_allocations,
        allocations,
        "selection changes must reuse instance pages across frames"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_disabled_boolean_controls_retain_semantics_values_and_targets() {
    crate::test_utils::run_isolated_test(
        "md3::checkbox::accessibility_tests::native_disabled_boolean_controls_retain_semantics_values_and_targets",
        check_controls,
    );
}
