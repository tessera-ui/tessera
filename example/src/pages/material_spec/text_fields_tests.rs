//! Text field showcase input and validation regressions.
//!
//! ## Usage
//!
//! Verify editing, validation and anatomy in real showcase fields.

use std::{cell::Cell, time::Duration};

use tessera_components::scrollable::ScrollableController;
use tessera_ui::{
    CursorEventContent, GestureState, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    PointerChange, PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    winit::event::Ime,
};

use super::*;

thread_local! {static KIND:Cell<usize>=const {Cell::new(0)};static SCROLL:Cell<Option<State<ScrollableController>>>=const {Cell::new(None)};}
struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}
#[tessera]
fn fixture() {
    material_theme().child(|| {
        column().children(|| match KIND.get() {
            0 | 14 => {
                sample("Regression", KIND.get() == 14)
                    .controls(true)
                    .icons(true)
                    .preview_width(Dp(180.0));
            }
            1 => {
                form_field("Description".into(), false, 6);
            }
            2 => {
                form_field("Password".into(), true, 9);
            }
            4..=7 => {
                let kind = KIND.get();
                field_color_scheme(kind % 2 == 1, kind < 6);
            }
            8..=13 => {
                let kind = KIND.get();
                field_measurements(kind % 2 == 1, (kind - 8) / 2);
            }
            15 | 16 => {
                pointer_clear_fixture(KIND.get() == 15);
            }
            17 => {
                field_icons_diagram();
            }
            18 => {
                field_error_icon_diagram();
            }
            19 => {
                super::super::text_fields_accessibility::labeling();
            }
            22 => {
                super::super::text_fields_accessibility::labeling_figure(6);
            }
            20 | 21 => {
                material_theme()
                    .theme(|| MaterialTheme::from_seed(ACCENT, KIND.get() == 21))
                    .child(|| {
                        specs_anatomy(true);
                    });
            }
            _ => {
                detail(false);
            }
        });
    });
}
fn renderer(kind: usize) -> HeadlessRenderer {
    KIND.set(kind);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: match kind {
                3..=13 | 15 | 16 | 20 | 21 => 1024,
                17..=19 => 676,
                22 => 1363,
                _ => 900,
            },
            height: if kind == 19 { 5000 } else { 1600 },
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    renderer
}
fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
}
fn group_descendants(renderer: &HeadlessRenderer, label: &str) -> Vec<(NodeId, Node)> {
    let nodes = renderer.accessibility_snapshot().unwrap().nodes;
    let root = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Group && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing group {label}"));
    let mut pending = root.1.children().to_vec();
    let mut descendants = Vec::new();
    while let Some(id) = pending.pop() {
        let (_, node) = nodes
            .iter()
            .find(|(current, _)| *current == id)
            .expect("a published child must exist in the accessibility snapshot");
        pending.extend_from_slice(node.children());
        descendants.push((id, node.clone()));
    }
    descendants
}

fn group_node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    let mut matches = group_descendants(renderer, label)
        .into_iter()
        .filter(|(_, node)| node.role() == role);
    let found = matches
        .next()
        .unwrap_or_else(|| panic!("missing {role:?} within {label}"));
    assert!(
        matches.next().is_none(),
        "expected a unique {role:?} within {label}"
    );
    found
}

fn sample_node(renderer: &HeadlessRenderer, group: &str, role: Role) -> (NodeId, Node) {
    let found = group_node(renderer, group, role);
    let label = found
        .1
        .label()
        .expect("the editor must announce its visible label");
    assert!(matches!(label, "Label" | "Label text"));
    assert!(
        group_descendants(renderer, group)
            .iter()
            .any(|(_, node)| { node.role() == Role::Label && node.label() == Some(label) }),
        "the editor name must match visible label text in its own preview"
    );
    found
}

fn act_in_group(renderer: &mut HeadlessRenderer, group: &str, role: Role, action: Action) {
    let id = group_node(renderer, group, role).0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }));
    renderer.render_frames(4);
}

fn group_input_value(renderer: &HeadlessRenderer, group: &str) -> String {
    group_node(renderer, group, Role::TextInput)
        .1
        .value()
        .unwrap_or_default()
        .to_owned()
}

fn exists(renderer: &HeadlessRenderer, label: &str, role: Role) -> bool {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.role() == role && node.label() == Some(label))
}
fn act(renderer: &mut HeadlessRenderer, label: &str, role: Role, action: Action) {
    let id = node(renderer, label, role).0;
    assert!(
        renderer.dispatch_accessibility_action(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None
        }),
        "{action:?} {label}"
    );
    renderer.render_frames(4);
}
fn commit(renderer: &mut HeadlessRenderer, value: &str) {
    for character in value.chars() {
        renderer.inject_ime_event(Ime::Commit(character.to_string()));
    }
    renderer.render_frames(4);
}
fn input_value(renderer: &HeadlessRenderer, label: &str) -> String {
    node(renderer, label, Role::TextInput)
        .1
        .value()
        .unwrap_or_default()
        .to_owned()
}
#[test]
#[ignore = "requires a native GPU adapter"]
fn constrained_showcase_preserves_rapid_ime_clear_focus_and_state_restrictions() {
    for kind in [0, 14] {
        let mut renderer = renderer(kind);
        assert_eq!(
            sample_node(&renderer, "Regression field preview", Role::TextInput)
                .1
                .label(),
            Some("Label")
        );
        act(
            &mut renderer,
            "Clear Regression icon",
            Role::Button,
            Action::Click,
        );
        assert_eq!(group_input_value(&renderer, "Regression field preview"), "");
        assert!(!exists(&renderer, "Clear Regression icon", Role::Button));
        act_in_group(
            &mut renderer,
            "Regression field preview",
            Role::TextInput,
            Action::Focus,
        );
        let container = node(&renderer, "Regression container", Role::Group)
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            container.height(),
            56.0,
            "focused empty single-line field must not grow to wrap its placeholder"
        );
        commit(&mut renderer, "alpha👩‍💻");
        assert_eq!(
            group_input_value(&renderer, "Regression field preview"),
            "alpha👩‍💻"
        );
        assert!(exists(&renderer, "Regression value: alpha👩‍💻", Role::Label));
        assert!(exists(&renderer, "Clear Regression icon", Role::Button));
        act_in_group(
            &mut renderer,
            "Regression field preview",
            Role::TextInput,
            Action::Focus,
        );
        act_in_group(
            &mut renderer,
            "Regression field preview",
            Role::TextInput,
            Action::Click,
        );
        assert!(exists(&renderer, "Regression submissions: 1", Role::Label));
        act(
            &mut renderer,
            "Regression affixes",
            Role::Button,
            Action::Click,
        );
        node(&renderer, "$", Role::Label);
        node(&renderer, ".00", Role::Label);
        assert_eq!(
            group_input_value(&renderer, "Regression field preview"),
            "alpha👩‍💻"
        );
        assert_eq!(
            node(&renderer, "Regression container", Role::Group)
                .1
                .bounds()
                .unwrap()
                .height(),
            56.0,
            "prefix and suffix must preserve the single-line container height"
        );
        act(
            &mut renderer,
            "Regression secure",
            Role::Button,
            Action::Click,
        );
        assert_eq!(
            sample_node(&renderer, "Regression field preview", Role::PasswordInput)
                .1
                .value(),
            None,
            "secure configuration must not publish the editor value"
        );
        assert!(!exists(&renderer, "Regression value: alpha👩‍💻", Role::Label));
        act(
            &mut renderer,
            "Regression secure",
            Role::Button,
            Action::Click,
        );
        assert_eq!(
            group_input_value(&renderer, "Regression field preview"),
            "alpha👩‍💻"
        );
        act(
            &mut renderer,
            "Regression read only",
            Role::Button,
            Action::Click,
        );
        assert!(
            node(&renderer, "Clear Regression", Role::Button)
                .1
                .is_disabled()
        );
        assert!(
            node(&renderer, "Clear Regression icon", Role::Button)
                .1
                .is_disabled()
        );
        act_in_group(
            &mut renderer,
            "Regression field preview",
            Role::TextInput,
            Action::Focus,
        );
        commit(&mut renderer, "ignored");
        assert_eq!(
            group_input_value(&renderer, "Regression field preview"),
            "alpha👩‍💻"
        );
        act(
            &mut renderer,
            "Regression read only",
            Role::Button,
            Action::Click,
        );
        act(
            &mut renderer,
            "Regression enabled",
            Role::Button,
            Action::Click,
        );
        assert!(
            sample_node(&renderer, "Regression field preview", Role::TextInput)
                .1
                .is_disabled()
        );
        assert!(
            node(&renderer, "Clear Regression", Role::Button)
                .1
                .is_disabled()
        );
        commit(&mut renderer, "ignored while disabled");
        assert_eq!(
            group_input_value(&renderer, "Regression field preview"),
            "alpha👩‍💻"
        );
        act(
            &mut renderer,
            "Regression enabled",
            Role::Button,
            Action::Click,
        );
        act(
            &mut renderer,
            "Regression error",
            Role::Button,
            Action::Click,
        );
        node(&renderer, "Please check your input", Role::Alert);
        act(
            &mut renderer,
            "Clear Regression",
            Role::Button,
            Action::Click,
        );
        assert_eq!(group_input_value(&renderer, "Regression field preview"), "");
        assert!(!exists(&renderer, "Clear Regression icon", Role::Button));
        act_in_group(
            &mut renderer,
            "Regression field preview",
            Role::TextInput,
            Action::Focus,
        );
        assert_eq!(
            node(&renderer, "Regression container", Role::Group)
                .1
                .bounds()
                .unwrap()
                .height(),
            56.0
        );
        act(
            &mut renderer,
            "Regression multiline",
            Role::Button,
            Action::Click,
        );
        assert_eq!(
            node(&renderer, "Regression container", Role::Group)
                .1
                .bounds()
                .unwrap()
                .height(),
            96.0,
            "multiline configuration must use its declared minimum height"
        );
        act_in_group(
            &mut renderer,
            "Regression field preview",
            Role::MultilineTextInput,
            Action::Focus,
        );
        commit(&mut renderer, "first\nsecond");
        assert_eq!(
            sample_node(
                &renderer,
                "Regression field preview",
                Role::MultilineTextInput
            )
            .1
            .value(),
            Some("first\nsecond")
        );
        assert!(exists(&renderer, "Regression submissions: 1", Role::Label));
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn counter_overflow_and_password_visibility_keep_real_editor_values() {
    {
        let mut renderer = renderer(1);
        act(&mut renderer, "Description", Role::TextInput, Action::Focus);
        let value = "a".repeat(101);
        commit(&mut renderer, &value);
        assert_eq!(input_value(&renderer, "Description"), value);
        node(&renderer, "Character count: 101 of 100", Role::Label);
        node(&renderer, "Use 100 characters or fewer", Role::Alert);
        assert_eq!(
            node(&renderer, "Description", Role::TextInput)
                .1
                .description(),
            Some("Use 100 characters or fewer")
        );
        // The overflow value is one word, so a real double-click selects it.
        // Recorded intervals keep multi-click recognition independent of GPU
        // time.
        let bounds = node(&renderer, "Description", Role::TextInput)
            .1
            .bounds()
            .unwrap();
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.step_frame();
        let base = tessera_ui::time::Instant::now();
        for (milliseconds, content) in [
            (0, CursorEventContent::Pressed(PressKeyEventType::Left)),
            (10, CursorEventContent::Released(PressKeyEventType::Left)),
            (80, CursorEventContent::Pressed(PressKeyEventType::Left)),
            (90, CursorEventContent::Released(PressKeyEventType::Left)),
        ] {
            renderer.inject_pointer(PointerChange::new(
                base + Duration::from_millis(milliseconds),
                MOUSE_POINTER_ID,
                content,
                GestureState::TapCandidate,
            ));
            renderer.step_frame();
        }
        renderer.inject_ime_event(Ime::Commit("Corrected".into()));
        renderer.render_frames(4);
        assert_eq!(input_value(&renderer, "Description"), "Corrected");
        node(&renderer, "Character count: 9 of 100", Role::Label);
        node(&renderer, "Supporting text", Role::Label);
        assert_eq!(
            node(&renderer, "Description", Role::TextInput)
                .1
                .description(),
            Some("Supporting text")
        );
        assert!(!exists(
            &renderer,
            "Use 100 characters or fewer",
            Role::Alert
        ));
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Alert)
        );
    }
    {
        let mut renderer = renderer(2);
        act(
            &mut renderer,
            "Password",
            Role::PasswordInput,
            Action::Focus,
        );
        commit(&mut renderer, "pāssword");
        assert_eq!(
            node(&renderer, "Password", Role::PasswordInput).1.value(),
            None
        );
        act(&mut renderer, "Show password", Role::Button, Action::Click);
        assert_eq!(input_value(&renderer, "Password"), "pāssword");
        node(&renderer, "Hide password", Role::Button);
        act(&mut renderer, "Hide password", Role::Button, Action::Click);
        assert_eq!(
            node(&renderer, "Password", Role::PasswordInput).1.value(),
            None
        );
        act(&mut renderer, "Show password", Role::Button, Action::Click);
        assert_eq!(input_value(&renderer, "Password"), "pāssword");
    }
    {
        let mut renderer = renderer(18);
        assert_eq!(input_value(&renderer, "Username"), "Sam");
        let password = node(&renderer, "Password", Role::PasswordInput).1;
        assert_eq!(password.value(), None);
        assert_eq!(
            password.description(),
            Some("Include an uppercase letter, a lowercase letter, and a numeric character")
        );
        node(&renderer, "Use at least 6 characters", Role::Alert);
        node(
            &renderer,
            "Include an uppercase letter, a lowercase letter, and a numeric character",
            Role::Alert,
        );
        pointer_click(&mut renderer, "Password", Role::PasswordInput);
        commit(&mut renderer, "9");
        let password = node(&renderer, "Password", Role::PasswordInput);
        assert_eq!(password.1.value(), None);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, password.0);
        assert_eq!(input_value(&renderer, "Username"), "Sam");
        act(&mut renderer, "Username", Role::TextInput, Action::Focus);
        commit(&mut renderer, "uel");
        assert_eq!(input_value(&renderer, "Username"), "uelSam");
        assert_eq!(
            node(&renderer, "Password", Role::PasswordInput).1.value(),
            None
        );
    }
    {
        let mut renderer = renderer(17);
        for number in 1..=6 {
            node(
                &renderer,
                &format!("Text field icon example {number}"),
                Role::Note,
            );
        }
        for (label, value) in [
            ("Calendar icon example", ""),
            ("Error icon example", "Input"),
            ("Clear icon example", "Input text"),
            ("Voice input icon example", ""),
            ("Dropdown icon example", ""),
            ("Payment image example", ""),
        ] {
            assert_eq!(input_value(&renderer, label), value);
        }
        pointer_click(&mut renderer, "Clear icon example text", Role::Button);
        assert_eq!(input_value(&renderer, "Clear icon example"), "");
        assert!(!exists(&renderer, "Clear icon example text", Role::Button));
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            node(&renderer, "Clear icon example", Role::TextInput).0,
            "the native Clear action must retain focus in its own editor"
        );
        commit(&mut renderer, "New");
        assert_eq!(input_value(&renderer, "Clear icon example"), "New");
        node(&renderer, "Clear icon example text", Role::Button);
        pointer_click(&mut renderer, "Choose icon example category", Role::Button);
        pointer_click(&mut renderer, "Work", Role::MenuItem);
        assert_eq!(input_value(&renderer, "Dropdown icon example"), "Work");
        assert!(!exists(&renderer, "Work", Role::MenuItem));
        pointer_click(&mut renderer, "Choose icon example category", Role::Button);
        pointer_click(&mut renderer, "Personal", Role::MenuItem);
        assert_eq!(input_value(&renderer, "Dropdown icon example"), "Personal");
        assert_eq!(input_value(&renderer, "Clear icon example"), "New");
        assert_eq!(input_value(&renderer, "Error icon example"), "Input");
    }
}
#[test]
#[ignore = "requires a native GPU adapter"]
fn accessibility_labeling_diagrams_keep_native_actions_values_and_live_callouts() {
    let mut renderer = renderer(19);
    let email = node(&renderer, "Email", Role::TextInput)
        .1
        .bounds()
        .unwrap();
    let email_callout = node(&renderer, "UI text (“Email”)", Role::Label)
        .1
        .bounds()
        .unwrap();
    assert!(email_callout.x1 < email.x0);
    assert!(email_callout.y0 < email.y1 && email_callout.y1 > email.y0);
    let phone = node(&renderer, "Phone number", Role::TextInput)
        .1
        .bounds()
        .unwrap();
    assert!(phone.y0 > email.y1);
    act(&mut renderer, "Email", Role::TextInput, Action::Focus);
    commit(&mut renderer, "name@example.com");
    assert_eq!(input_value(&renderer, "Email"), "name@example.com");

    act(
        &mut renderer,
        "Password",
        Role::PasswordInput,
        Action::Focus,
    );
    commit(&mut renderer, "sëcret9");
    assert_eq!(
        node(&renderer, "Password", Role::PasswordInput).1.value(),
        None
    );
    let password = node(&renderer, "Password", Role::PasswordInput)
        .1
        .bounds()
        .unwrap();
    let show = node(&renderer, "Show password", Role::Button)
        .1
        .bounds()
        .unwrap();
    assert!(show.y0 < password.y1 && show.y1 > password.y0);
    node(&renderer, "Show Password", Role::Label);
    pointer_click(&mut renderer, "Show password", Role::Button);
    assert_eq!(input_value(&renderer, "Password"), "sëcret9");
    node(&renderer, "Hide Password", Role::Label);
    assert!(!exists(&renderer, "Show Password", Role::Label));
    pointer_click(&mut renderer, "Hide password", Role::Button);
    assert_eq!(
        node(&renderer, "Password", Role::PasswordInput).1.value(),
        None
    );
    node(&renderer, "Show Password", Role::Label);

    pointer_click(&mut renderer, "Choose label option", Role::Button);
    pointer_click(&mut renderer, "Work", Role::MenuItem);
    assert_eq!(input_value(&renderer, "Label options"), "Work");
    assert!(!exists(&renderer, "Work", Role::MenuItem));

    act(&mut renderer, "Amount", Role::TextInput, Action::Focus);
    commit(&mut renderer, "12.50");
    assert_eq!(input_value(&renderer, "Amount"), "12.50");
    let euro = node(&renderer, "Euro", Role::Label).1.bounds().unwrap();
    let amount = node(&renderer, "Amount", Role::TextInput)
        .1
        .bounds()
        .unwrap();
    assert!(euro.x0 < amount.x0);
    assert!(euro.y0 < amount.y1 && euro.y1 > amount.y0);
    node(&renderer, "At gmail dot com", Role::Label);

    let zip = node(&renderer, "ZIP code", Role::TextInput).1;
    assert_eq!(zip.description(), Some("Not a valid ZIP code"));
    let error = node(&renderer, "Not a valid ZIP code", Role::Alert)
        .1
        .bounds()
        .unwrap();
    assert!(error.y0 >= zip.bounds().unwrap().y1);

    assert_eq!(
        group_input_value(&renderer, "Character count labeling example"),
        "Input"
    );
    node(
        &renderer,
        "Character count, 5 of 20 characters entered",
        Role::Label,
    );
    assert_eq!(
        group_node(
            &renderer,
            "Character count labeling example",
            Role::TextInput
        )
        .1
        .label(),
        Some("Label")
    );
    assert!(
        group_descendants(&renderer, "Character count labeling example")
            .iter()
            .any(|(_, node)| node.role() == Role::Label && node.label() == Some("Label"))
    );
    act_in_group(
        &mut renderer,
        "Character count labeling example",
        Role::TextInput,
        Action::Focus,
    );
    commit(&mut renderer, "é中🦀");
    assert_eq!(
        group_input_value(&renderer, "Character count labeling example"),
        "é中🦀Input"
    );
    let counter = node(
        &renderer,
        "Character count, 8 of 20 characters entered",
        Role::Label,
    )
    .1
    .bounds()
    .unwrap();
    let callout = node(
        &renderer,
        "UI text (“Character count, 8 of 20\ncharacters entered”)",
        Role::Label,
    )
    .1
    .bounds()
    .unwrap();
    assert!(callout.x0 > counter.x1);
    assert!(!exists(
        &renderer,
        "Character count, 5 of 20 characters entered",
        Role::Label
    ));

    let supporting = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.role() == Role::TextInput
                && node.label() == Some("Email")
                && node.description() == Some("Please use the company email address")
        })
        .expect("supporting text must describe its own Email editor")
        .1;
    let support = node(
        &renderer,
        "Please use the company email address",
        Role::Label,
    )
    .1
    .bounds()
    .unwrap();
    assert!(support.y0 >= supporting.bounds().unwrap().y1);
    let required = node(&renderer, "Username*", Role::TextInput).1;
    assert_eq!(required.description(), Some("Required"));
    let required_support = node(&renderer, "*required", Role::Label)
        .1
        .bounds()
        .unwrap();
    assert!(required_support.y0 >= required.bounds().unwrap().y1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn outlined_anatomy_uses_one_reference_artboard_and_retains_input_on_resize() {
    for kind in [20, 21] {
        let mut renderer = renderer(kind);
        let (theme, prefix) = if kind == 20 {
            ("Light", "")
        } else {
            ("Dark", "Dark ")
        };
        let panel = node(
            &renderer,
            &format!("{theme} outlined anatomy panel"),
            Role::Group,
        )
        .1
        .bounds()
        .unwrap();
        assert_eq!((panel.width(), panel.height()), (1024.0, 300.0));
        for (label, x) in [("empty", 292.0), ("populated", 552.0)] {
            let bounds = node(
                &renderer,
                &format!("{prefix}Outlined {label} anatomy container"),
                Role::Group,
            )
            .1
            .bounds()
            .unwrap();
            assert_eq!(
                (
                    bounds.x0 - panel.x0,
                    bounds.y0 - panel.y0,
                    bounds.width(),
                    bounds.height()
                ),
                (x, 116.0, 180.0, 56.0),
                "The outlined fields must match the canonical reference geometry"
            );
        }
        for (number, x, y) in [
            (2, 304.0, 78.0),
            (3, 370.0, 78.0),
            (4, 584.0, 63.0),
            (5, 696.0, 79.0),
            (8, 630.0, 197.0),
            (9, 370.0, 206.0),
        ] {
            let bounds = node(
                &renderer,
                &format!("{prefix}Anatomy marker {number}"),
                Role::Note,
            )
            .1
            .bounds()
            .unwrap();
            assert_eq!(
                (
                    bounds.x0 - panel.x0,
                    bounds.y0 - panel.y0,
                    bounds.width(),
                    bounds.height()
                ),
                (x, y, 24.0, 32.0),
                "Outlined anatomy marker {number} must align with its reference part"
            );
        }
        assert!(!exists(
            &renderer,
            &format!(
                "{} outlined anatomy panel",
                if kind == 20 { "Dark" } else { "Light" }
            ),
            Role::Group
        ));
        for number in 1..=9 {
            node(
                &renderer,
                &format!("{prefix}Anatomy marker {number}"),
                Role::Note,
            );
        }
        let marker = node(&renderer, &format!("{prefix}Anatomy marker 1"), Role::Note)
            .1
            .bounds()
            .unwrap();
        let pixels = renderer.read_pixels().unwrap();
        let pixel = |x: f64, y: f64| {
            let offset = (y.floor() as usize * 1024 + x.floor() as usize) * 4;
            &pixels[offset..offset + 4]
        };
        assert_eq!(
            pixel(marker.x0 + 4.0, marker.y0 + 16.0),
            pixel(panel.x0 + 16.0, panel.y0 + 16.0),
            "Anatomy capsules retain their open interior"
        );
        let input = format!("{prefix}Outlined populated anatomy field preview");
        assert_eq!(
            sample_node(&renderer, &input, Role::TextInput).1.label(),
            Some("Label text")
        );
        assert_eq!(group_input_value(&renderer, &input), "Input text");
        pointer_click(
            &mut renderer,
            &format!("Clear {prefix}Outlined populated anatomy icon"),
            Role::Button,
        );
        assert_eq!(group_input_value(&renderer, &input), "");
        renderer.resize(512, 1600);
        renderer.render_frames(4);
        let panel = node(
            &renderer,
            &format!("{theme} outlined anatomy panel"),
            Role::Group,
        )
        .1
        .bounds()
        .unwrap();
        assert_eq!((panel.width(), panel.height()), (512.0, 150.0));
        assert_eq!(group_input_value(&renderer, &input), "");
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn specs_anatomy_keeps_original_inputs_and_publishes_ten_geometry_callouts() {
    let mut renderer = renderer(3);
    for number in 1..=10 {
        node(&renderer, &format!("Anatomy marker {number}"), Role::Note);
    }
    let empty = node(&renderer, "Filled empty anatomy container", Role::Group)
        .1
        .bounds()
        .unwrap();
    let populated = node(&renderer, "Filled populated anatomy container", Role::Group)
        .1
        .bounds()
        .unwrap();
    assert_eq!(empty.height(), 56.0);
    assert_eq!(populated.height(), 56.0);
    assert_eq!(empty.width(), 180.0);
    assert_eq!(populated.width(), 180.0);
    assert!(empty.x1 < populated.x0);
    let panel = node(&renderer, "Light filled anatomy panel", Role::Group)
        .1
        .bounds()
        .unwrap();
    for (bounds, x) in [(empty, 292.0), (populated, 552.0)] {
        assert_eq!(
            (bounds.x0 - panel.x0, bounds.y0 - panel.y0),
            (x, 122.0),
            "Filled anatomy fields must retain their reference placement"
        );
    }
    for (number, x, y) in [
        (1, 248.0, 122.0),
        (2, 304.0, 85.0),
        (3, 370.0, 85.0),
        (4, 628.0, 85.0),
        (5, 696.0, 85.0),
        (6, 751.0, 162.0),
        (7, 664.0, 210.0),
        (8, 630.0, 210.0),
        (9, 370.0, 210.0),
        (10, 248.0, 162.0),
    ] {
        let marker = node(&renderer, &format!("Anatomy marker {number}"), Role::Note)
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (marker.x0 - panel.x0, marker.y0 - panel.y0),
            (x, y),
            "Filled anatomy marker {number} must point to its reference part"
        );
    }
    assert_eq!(
        group_input_value(&renderer, "Filled populated anatomy field preview"),
        "Input text"
    );
    let pixels = renderer.read_pixels().unwrap();
    let width = renderer.physical_size().width as usize;
    let pixel = |x: f64, y: f64| {
        let offset = (y.floor() as usize * width + x.floor() as usize) * 4;
        [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
    };
    let clear_interior = |a: [u8; 3], b: [u8; 3], number: usize| {
        assert!(
            a.into_iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 3),
            "leader must stop before entering anatomy marker {number}"
        );
    };
    let marker = node(&renderer, "Anatomy marker 1", Role::Note)
        .1
        .bounds()
        .unwrap();
    let x = marker.x1 - 4.0;
    let y = (marker.y0 + marker.y1) / 2.0;
    clear_interior(pixel(x, y), pixel(x, y - 4.0), 1);
    for number in [2, 3] {
        let marker = node(&renderer, &format!("Anatomy marker {number}"), Role::Note)
            .1
            .bounds()
            .unwrap();
        let x = (marker.x0 + marker.x1) / 2.0;
        let y = marker.y1 - 4.0;
        clear_interior(pixel(x, y), pixel(x + 4.0, y), number);
    }
    let light = node(&renderer, "Light filled anatomy panel", Role::Group)
        .1
        .bounds()
        .unwrap();
    assert_eq!(light.width(), 1024.0);
    assert_eq!(light.height(), 300.0);
    assert!(!exists(&renderer, "Dark filled anatomy panel", Role::Group));
    pointer_click(
        &mut renderer,
        "Clear Filled populated anatomy icon",
        Role::Button,
    );
    assert_eq!(
        group_input_value(&renderer, "Filled populated anatomy field preview"),
        ""
    );
    assert_eq!(
        group_input_value(&renderer, "Filled empty anatomy field preview"),
        ""
    );
    renderer.resize(512, 1600);
    renderer.render_frames(4);
    let panel = node(&renderer, "Light filled anatomy panel", Role::Group)
        .1
        .bounds()
        .unwrap();
    assert_eq!((panel.width(), panel.height()), (512.0, 150.0));
    let marker = node(&renderer, "Anatomy marker 2", Role::Note)
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        (
            marker.x0 - panel.x0,
            marker.y0 - panel.y0,
            marker.width(),
            marker.height()
        ),
        (152.0, 42.5, 12.0, 16.0),
        "Narrow anatomy must scale its reference coordinates without reflow"
    );
    assert_eq!(
        group_input_value(&renderer, "Filled populated anatomy field preview"),
        ""
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn field_color_panels_use_their_own_theme_surface_and_readable_supporting_text() {
    for kind in 4..=7 {
        let renderer = renderer(kind);
        let outlined = kind % 2 == 1;
        let dark = kind < 6;
        let panel = node(
            &renderer,
            &format!(
                "{} {} color panel",
                if dark { "Dark" } else { "Light" },
                if outlined { "outlined" } else { "filled" }
            ),
            Role::Group,
        )
        .1
        .bounds()
        .unwrap();
        let theme = MaterialTheme::from_seed(ACCENT, dark);
        let pixels = renderer.read_pixels().unwrap();
        let width = renderer.physical_size().width as usize;
        let pixel = |x: usize, y: usize| {
            let offset = (y * width + x) * 4;
            [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
        };
        let background = pixel(
            (panel.x0 + 12.0) as usize,
            ((panel.y0 + panel.y1) / 2.0) as usize,
        );
        let expected = theme.color_scheme.surface_container_low;
        let encode = |value: f32| {
            let value = if value <= 0.0031308 {
                12.92 * value
            } else {
                1.055 * value.powf(1.0 / 2.4) - 0.055
            };
            (value * 255.0).round() as u8
        };
        for (actual, expected) in
            background
                .into_iter()
                .zip([encode(expected.r), encode(expected.g), encode(expected.b)])
        {
            assert!(
                actual.abs_diff(expected) <= 2,
                "color panel must draw its nested theme surface"
            );
        }
        let linear = |value: u8| {
            let value = value as f64 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        let luminance = |pixel: [u8; 3]| {
            0.2126 * linear(pixel[0]) + 0.7152 * linear(pixel[1]) + 0.0722 * linear(pixel[2])
        };
        let background_luminance = luminance(background);
        let support = node(&renderer, "Supporting text", Role::Label)
            .1
            .bounds()
            .unwrap();
        let mut contrast = 1.0_f64;
        for y in support.y0.floor().max(0.0) as usize..support.y1.ceil() as usize {
            for x in support.x0.floor().max(0.0) as usize..support.x1.ceil() as usize {
                let value = luminance(pixel(x, y));
                contrast = contrast.max(
                    (value.max(background_luminance) + 0.05)
                        / (value.min(background_luminance) + 0.05),
                );
            }
        }
        assert!(
            contrast >= 4.5,
            "supporting text must remain readable on its own theme background: {contrast}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn measurement_figures_keep_native_field_icon_support_and_counter_geometry() {
    for kind in 8..=13 {
        let mut renderer = renderer(kind);
        let outlined = kind % 2 == 1;
        let variant = (kind - 8) / 2;
        let bounds = |populated| {
            node(
                &renderer,
                &format!(
                    "{} container",
                    measurement_name(outlined, variant, populated)
                ),
                Role::Group,
            )
            .1
            .bounds()
            .unwrap()
        };
        let empty = bounds(false);
        let pop = bounds(true);
        assert_eq!(empty.height(), 56.0);
        assert_eq!(pop.height(), 56.0);
        assert_eq!(empty.width(), 240.0);
        assert_eq!(pop.width(), 240.0);
        assert_eq!(pop.x0 - empty.x1, 120.0);
        if variant != 2 {
            node(&renderer, "Dimension 56", Role::Note);
        }
        if variant == 1 {
            for (populated, field) in [(false, empty), (true, pop)] {
                let icon = node(
                    &renderer,
                    &format!(
                        "{} leading icon",
                        measurement_name(outlined, variant, populated)
                    ),
                    Role::Image,
                )
                .1
                .bounds()
                .unwrap();
                assert_eq!(icon.width(), 24.0);
                assert_eq!(icon.height(), 24.0);
                assert_eq!(icon.x0 - field.x0, 12.0);
                assert_eq!(icon.y0 - field.y0, 16.0);
                let snapshot = renderer.accessibility_snapshot().unwrap();
                for (label, x, y) in [
                    ("Dimension 24", icon.x0 + 4.0, field.y0 - 62.0),
                    ("Dimension 16", icon.x1, field.y0 - 34.0),
                ] {
                    assert!(
                        snapshot.nodes.iter().any(|(_, annotation)| {
                            annotation.role() == Role::Note
                                && annotation.label() == Some(label)
                                && annotation.bounds().is_some_and(|bounds| {
                                    (bounds.x0 - x).abs() < 0.01 && (bounds.y0 - y).abs() < 0.01
                                })
                        }),
                        "{} must retain its own {label} at ({x}, {y})",
                        measurement_name(outlined, variant, populated),
                    );
                }
            }
            node(&renderer, "Dimension 12", Role::Note);
            node(&renderer, "Dimension 24", Role::Note);
        }
        if variant == 1 {
            let right = measurement_name(outlined, variant, true);
            let left = measurement_name(outlined, variant, false);
            let right_annotation = format!("{right} trailing icon annotation");
            let left_annotation = format!("{left} trailing icon annotation");
            node(&renderer, &right_annotation, Role::Note);
            assert!(!exists(&renderer, &left_annotation, Role::Note));
        }
        if variant == 2 {
            let support = node(
                &renderer,
                &format!(
                    "{} supporting text",
                    measurement_name(outlined, variant, true)
                ),
                Role::Label,
            )
            .1
            .bounds()
            .unwrap();
            let counter = node(
                &renderer,
                "Measurement character count: 0 of 20",
                Role::Label,
            )
            .1
            .bounds()
            .unwrap();
            assert_eq!(support.y0 - pop.y1, 4.0);
            assert_eq!(support.x0 - pop.x0, 16.0);
            assert_eq!(counter.x0 - support.x1, 16.0);
            assert_eq!(pop.x1 - counter.x1, 16.0);
            node(&renderer, "Dimension 4", Role::Note);
        } else {
            let pixels = renderer.read_pixels().unwrap();
            let width = renderer.physical_size().width as usize;
            let pixel = |x: f64, y: f64| {
                let offset = (y as usize * width + x as usize) * 4;
                [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
            };
            let band = pixel(empty.x0 + 4.0, empty.y0 + 28.0);
            let body = pixel(empty.x1 - 48.0, empty.y0 + 28.0);
            assert!(
                band.into_iter()
                    .zip(body)
                    .map(|(a, b)| u32::from(a.abs_diff(b)))
                    .sum::<u32>()
                    > 20,
                "native padding overlay must remain visibly distinct from field content"
            );
        }
        if variant == 1 {
            let right = measurement_name(outlined, variant, true);
            let left = measurement_name(outlined, variant, false);
            act(
                &mut renderer,
                &format!("Clear {right}"),
                Role::Button,
                Action::Click,
            );
            assert_eq!(input_value(&renderer, &right), "");
            assert!(!exists(&renderer, &format!("Clear {right}"), Role::Button));
            assert!(!exists(
                &renderer,
                &format!("{right} trailing icon annotation"),
                Role::Note
            ));
            act(&mut renderer, &left, Role::TextInput, Action::Focus);
            if !outlined {
                assert!(!exists(
                    &renderer,
                    &format!("{right} floating inset annotation"),
                    Role::Note
                ));
                node(
                    &renderer,
                    &format!("{left} floating inset annotation"),
                    Role::Note,
                );
            }
            commit(&mut renderer, "new");
            assert_eq!(input_value(&renderer, &left), "new");
            node(&renderer, &format!("Clear {left}"), Role::Button);
            let annotation = node(
                &renderer,
                &format!("{left} trailing icon annotation"),
                Role::Note,
            )
            .1
            .bounds()
            .unwrap();
            assert_eq!(annotation.width(), 24.0);
            assert_eq!(annotation.height(), 24.0);
            assert!(!exists(
                &renderer,
                &format!("{right} trailing icon annotation"),
                Role::Note
            ));
        }
    }
    for kind in [15, 16] {
        let mut renderer = renderer(kind);
        let outlined = kind == 15;
        pointer_click(&mut renderer, "Earlier Specs control", Role::Button);
        let earlier = node(&renderer, "Earlier Specs control", Role::Button).0;
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, earlier);
        let scroll = SCROLL.get().unwrap();
        scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -600.0)));
        renderer.render_frames(5);
        let right = measurement_name(outlined, 1, true);
        let container_label = format!("{right} container");
        let before = node(&renderer, &container_label, Role::Group)
            .1
            .bounds()
            .unwrap();
        let position = scroll.with(ScrollableController::child_position);
        pointer_click(&mut renderer, &format!("Clear {right}"), Role::Button);
        assert_eq!(input_value(&renderer, &right), "");
        assert!(!exists(&renderer, &format!("Clear {right}"), Role::Button));
        let editor = node(&renderer, &right, Role::TextInput).0;
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            editor,
            "self-removing Clear must hand pointer focus to its own editor"
        );
        assert_eq!(
            scroll.with(ScrollableController::child_position),
            position,
            "clearing must not reveal the earlier Specs focus target"
        );
        assert_eq!(
            node(&renderer, &container_label, Role::Group)
                .1
                .bounds()
                .unwrap(),
            before
        );
    }
}

#[test]
#[ignore = "Requires a native GPU adapter for retained measurement annotations"]
fn translated_measurement_figures_retain_callouts_and_scale_without_reflow_after_resize() {
    for kind in [15, 16] {
        let mut renderer = renderer(kind);
        let outlined = kind == 15;
        let scroll = SCROLL.get().unwrap();
        scroll.with_mut(|scroll| {
            scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -500.0));
        });
        renderer.render_frames(24);
        let label = format!("{} container", measurement_name(outlined, 1, true));
        let before = node(&renderer, &label, Role::Group).1.bounds().unwrap();
        let callouts: Vec<_> = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .filter(|(_, node)| node.role() == Role::Note)
            .map(|(id, node)| (id, node.label().map(str::to_owned), node.bounds().unwrap()))
            .collect();
        assert!(
            !callouts.is_empty(),
            "the actual measurement overlay must be present"
        );
        scroll.with_mut(|scroll| {
            scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -532.0));
        });
        let mut geometry = 0;
        let mut stores = 0;
        for _ in 0..6 {
            renderer.render_frames(1);
            let work = renderer.diagnostics();
            geometry += work.geometry_nodes_updated;
            stores += work.cache_store_count;
        }
        assert!(geometry > 0, "the actual field subtree must move");
        assert_eq!(stores, 0, "translation must reuse field measurements");
        let after = node(&renderer, &label, Role::Group).1.bounds().unwrap();
        assert_eq!(after.y0 - before.y0, -32.0);
        assert_eq!(after.width(), before.width());
        assert_eq!(after.height(), before.height());
        let snapshot = renderer.accessibility_snapshot().unwrap();
        for (id, label, before) in callouts {
            let (_, annotation) = snapshot
                .nodes
                .iter()
                .find(|(current, _)| *current == id)
                .expect("translation must retain each measurement callout");
            assert_eq!(annotation.label(), label.as_deref());
            let after = annotation.bounds().unwrap();
            assert_eq!(after.x0, before.x0);
            assert_eq!(after.y0 - before.y0, -32.0);
            assert_eq!(after.width(), before.width());
            assert_eq!(after.height(), before.height());
        }
        renderer.render_frames(1);
        assert_eq!(renderer.diagnostics().paint_nodes_recorded, 0);
        assert_eq!(renderer.diagnostics().measure_node_calls, 1);

        renderer.resize(640, 1600);
        renderer.render_frames(12);
        scroll.with_mut(|scroll| {
            scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -500.0));
        });
        renderer.render_frames(5);
        let empty = node(
            &renderer,
            &format!("{} container", measurement_name(outlined, 1, false)),
            Role::Group,
        )
        .1
        .bounds()
        .unwrap();
        let populated = node(&renderer, &label, Role::Group).1.bounds().unwrap();
        assert_eq!(empty.width(), 150.0);
        assert_eq!(populated.width(), 150.0);
        assert_eq!(populated.x0 - empty.x1, 75.0);
        assert!(
            (populated.y0 - empty.y0).abs() < 0.01,
            "narrow figures must preserve the reference's paired native layout"
        );
        let snapshot = renderer.accessibility_snapshot().unwrap();
        for (populated, field) in [(false, empty), (true, populated)] {
            let icon = node(
                &renderer,
                &format!("{} leading icon", measurement_name(outlined, 1, populated)),
                Role::Image,
            )
            .1
            .bounds()
            .unwrap();
            assert!(
                snapshot.nodes.iter().any(|(_, annotation)| {
                    annotation.role() == Role::Note
                        && annotation.label() == Some("Dimension 24")
                        && annotation.bounds().is_some_and(|bounds| {
                            (bounds.x0 - icon.x0 - 2.5).abs() < 0.01
                                && (bounds.y0 - field.y0 + 38.75).abs() < 0.01
                        })
                }),
                "resized annotations must follow their own native field"
            );
        }
        let right = measurement_name(outlined, 1, true);
        let position = scroll.with(ScrollableController::child_position);
        let clear = node(&renderer, &format!("Clear {right}"), Role::Button)
            .1
            .bounds()
            .unwrap();
        assert_eq!(clear.width(), 30.0);
        assert_eq!(clear.height(), 30.0);
        pointer_click(&mut renderer, &format!("Clear {right}"), Role::Button);
        assert_eq!(input_value(&renderer, &right), "");
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            node(&renderer, &right, Role::TextInput).0
        );
        assert_eq!(scroll.with(ScrollableController::child_position), position);
    }
}

#[tessera]
fn pointer_clear_fixture(outlined: bool) {
    let scroll = remember(ScrollableController::default);
    SCROLL.set(Some(scroll));
    boxed()
        .modifier(
            Modifier::new()
                .height(Dp(320.0))
                .fill_max_width()
                .vertical_scroll(scroll),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    button()
                        .text()
                        .accessibility_label("Earlier Specs control")
                        .on_click(|| {})
                        .child(|| {
                            text().content("Specs");
                        });
                    spacer().modifier(Modifier::new().height(Dp(600.0)));
                    field_measurements(outlined, 1);
                });
        });
}

fn pointer_click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
    let bounds = node(renderer, label, role).1.bounds().unwrap();
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(5);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn required_labeling_field_accepts_pointer_focus_and_editing_at_scaled_artboard_widths() {
    let mut renderer = renderer(22);
    for width in [1363, 676, 320] {
        renderer.resize(width, 1000);
        renderer.render_frames(8);
        let (id, field) = node(&renderer, "Username*", Role::TextInput);
        let bounds = field.bounds().unwrap();
        let scale = f64::from(width) / 676.0;
        for (actual, expected) in [
            (bounds.x0, 378.0 * scale),
            (bounds.y0, 123.0 * scale),
            (bounds.width(), 295.0 * scale),
            (bounds.height(), 24.0 * scale),
        ] {
            assert!(
                (actual - expected).abs() <= 1.0,
                "semantic input must match the independently mapped canonical inner field at {width}: {bounds:?}, actual {actual}, expected {expected}"
            );
        }
        let before = renderer.read_pixels().unwrap();
        let position = PxPosition::from_f32_xy((500.0 * scale) as f32, (135.0 * scale) as f32);
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(3);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            id,
            "the visible required field receives pointer focus at {width}"
        );
        commit(&mut renderer, "Q");
        assert!(input_value(&renderer, "Username*").contains('Q'));
        let after = renderer.read_pixels().unwrap();
        let mut changed = false;
        for y in (123.0 * scale).floor() as usize..(147.0 * scale).ceil() as usize {
            for x in (378.0 * scale).floor() as usize..(673.0 * scale).ceil() as usize {
                let offset = (y * width as usize + x) * 4;
                changed |= before[offset..offset + 3] != after[offset..offset + 3];
            }
        }
        assert!(
            changed,
            "the independently mapped visible editor must repaint after real input at {width}"
        );
    }
}
