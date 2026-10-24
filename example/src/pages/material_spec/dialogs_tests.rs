//! Dialog article retained draft and discard confirmation regressions.
//!
//! ## Usage
//!
//! Verify real event edits survive cancellation and reset after completion.

use std::time::Duration;

use tessera_components::theme::material_theme;
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
};

use super::*;

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        dialogs_spec();
    });
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

fn exists(renderer: &HeadlessRenderer, label: &str, role: Role) -> bool {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.role() == role && node.label() == Some(label))
}

fn action(renderer: &mut HeadlessRenderer, label: &str, role: Role, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer, label, role).0,
        data: None,
    }));
    renderer.render_frames(20);
}

fn edit(renderer: &mut HeadlessRenderer, label: &str, value: &str) {
    action(renderer, label, Role::TextInput, Action::Focus);
    for character in value.chars() {
        renderer.inject_text(&character.to_string());
    }
    renderer.render_frames(3);
}

fn assert_default_draft(renderer: &HeadlessRenderer) {
    for (label, value) in [
        ("Event name", ""),
        ("From", "Mon, May 18"),
        ("To", "Mon, May 18"),
        ("Time zone", "GMT-07:00"),
    ] {
        assert_eq!(
            node(renderer, label, Role::TextInput)
                .1
                .value()
                .unwrap_or_default(),
            value
        );
    }
    assert_eq!(
        node(renderer, "All-day", Role::CheckBox).1.toggled(),
        Some(Toggled::False)
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn event_dialog_keeps_edits_when_discard_is_cancelled_and_resets_after_discard_or_save() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 1000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    action(&mut renderer, "Overview", Role::Tab, Action::Click);
    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Focus,
    );
    let origin = node(&renderer, "Open full-screen dialog", Role::Button).0;
    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Click,
    );
    node(&renderer, "New event", Role::Dialog);
    assert_default_draft(&renderer);
    edit(&mut renderer, "Event name", "Planning 🦀");
    for (label, prefix) in [
        ("From", "Start: "),
        ("To", "End: "),
        ("Time zone", "Zone: "),
    ] {
        edit(&mut renderer, label, prefix);
    }
    action(&mut renderer, "All-day", Role::CheckBox, Action::Click);
    action(&mut renderer, "Close dialog", Role::Button, Action::Focus);
    let close = node(&renderer, "Close dialog", Role::Button).0;
    action(&mut renderer, "Close dialog", Role::Button, Action::Click);
    node(&renderer, "Discard changes?", Role::AlertDialog);
    action(&mut renderer, "Keep editing", Role::Button, Action::Click);
    assert!(!exists(&renderer, "Discard changes?", Role::AlertDialog));
    node(&renderer, "New event", Role::Dialog);
    assert_eq!(
        node(&renderer, "Event name", Role::TextInput).1.value(),
        Some("Planning 🦀")
    );
    for (label, value) in [
        ("From", "Start: Mon, May 18"),
        ("To", "End: Mon, May 18"),
        ("Time zone", "Zone: GMT-07:00"),
    ] {
        assert_eq!(
            node(&renderer, label, Role::TextInput).1.value(),
            Some(value)
        );
    }
    assert_eq!(
        node(&renderer, "All-day", Role::CheckBox).1.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, close);
    action(&mut renderer, "Close dialog", Role::Button, Action::Click);
    node(&renderer, "Discard changes?", Role::AlertDialog);
    action(&mut renderer, "Discard", Role::Button, Action::Click);
    assert!(!exists(&renderer, "New event", Role::Dialog));
    assert!(!exists(&renderer, "Changes saved", Role::Label));
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Click,
    );
    assert_default_draft(&renderer);
    edit(&mut renderer, "Event name", "Release review");
    for (label, prefix) in [
        ("From", "Start: "),
        ("To", "End: "),
        ("Time zone", "Zone: "),
    ] {
        edit(&mut renderer, label, prefix);
    }
    action(&mut renderer, "All-day", Role::CheckBox, Action::Click);
    action(&mut renderer, "Save", Role::Button, Action::Click);
    assert!(!exists(&renderer, "New event", Role::Dialog));
    node(&renderer, "Changes saved", Role::Label);
    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Click,
    );
    assert_default_draft(&renderer);
    action(&mut renderer, "Close dialog", Role::Button, Action::Click);
    assert!(!exists(&renderer, "Discard changes?", Role::AlertDialog));
    assert!(!exists(&renderer, "New event", Role::Dialog));
}
