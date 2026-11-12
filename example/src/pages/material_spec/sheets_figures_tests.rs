use std::time::Duration;

use tessera_components::theme::material_theme;
use tessera_ui::{
    MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Role, Toggled},
    headless::{HeadlessConfig, HeadlessRenderer},
};

use super::*;

#[tessera]
fn music_fixture() {
    material_theme().child(|| {
        column().children(|| {
            music_context();
        });
    });
}

#[tessera]
fn filters_fixture() {
    material_theme().child(|| {
        column().children(|| {
            filter_context();
        });
    });
}

#[tessera]
fn tokens_fixture() {
    material_theme().child(|| {
        column().children(|| {
            token_browser();
        });
    });
}

fn renderer<Output: 'static>(fixture: fn() -> Output) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 676,
            height: 700,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    renderer
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    snapshot
        .find(&crate::res::I18N.text(label), role)
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
        .1
        .clone()
}

fn click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
    let bounds = node(renderer, label, role).bounds().unwrap();
    assert!(bounds.width() >= 48.0 && bounds.height() >= 48.0);
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(4);
}

fn labelled(renderer: &HeadlessRenderer, label: &str) -> bool {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| crate::has_label(&node, label))
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn token_folders_reveal_and_hide_their_tokens_when_selected() {
    let mut renderer = renderer(tokens_fixture);
    let width = "Drag handle width: 32dp";
    assert!(!labelled(&renderer, "Drag handle tokens"));
    click(&mut renderer, "Enabled tokens", Role::ListItem);
    for group in [
        "Container tokens",
        "Drag handle tokens",
        "Focus indicator tokens",
    ] {
        assert!(labelled(&renderer, group), "Enabled must reveal {group}");
    }
    assert!(!labelled(&renderer, width));
    click(&mut renderer, "Drag handle tokens", Role::ListItem);
    assert!(labelled(&renderer, width));
    assert!(labelled(&renderer, "Drag handle opacity: 0.4"));
    assert!(!labelled(&renderer, "Focus indicator thickness: 3dp"));
    click(&mut renderer, "Enabled tokens", Role::ListItem);
    assert!(!labelled(&renderer, "Drag handle tokens"));
    assert!(!labelled(&renderer, width));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn music_context_places_native_actions_and_updates_playback_and_cover_art() {
    let mut renderer = renderer(music_fixture);
    for (label, x) in [("Pause", 471.0), ("Next track", 509.0)] {
        let bounds = node(&renderer, label, Role::Button).bounds().unwrap();
        assert!(((bounds.x0 + bounds.x1) * 0.5 - x).abs() <= 1.0);
        assert!(((bounds.y0 + bounds.y1) * 0.5 - 174.0).abs() <= 1.0);
    }
    click(&mut renderer, "Pause", Role::Button);
    node(&renderer, "Play", Role::Button);
    click(&mut renderer, "Play", Role::Button);
    node(&renderer, "Pause", Role::Button);
    let before = renderer.read_pixels().unwrap();
    click(&mut renderer, "Next track", Role::Button);
    let after = renderer.read_pixels().unwrap();
    let changed = (140..208)
        .flat_map(|y| (132..196).map(move |x| (y * 676 + x) * 4))
        .filter(|&offset| before[offset..offset + 4] != after[offset..offset + 4])
        .count();
    assert!(
        changed > 100,
        "Next track must change the native cover photo"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn filters_context_keeps_checkbox_selections_when_closed_and_reopened() {
    let mut renderer = renderer(filters_fixture);
    for (index, (label, selected)) in [
        ("Events", false),
        ("Personal", false),
        ("Projects", true),
        ("Reminders", true),
        ("Family", false),
    ]
    .into_iter()
    .enumerate()
    {
        let checkbox = node(&renderer, label, Role::CheckBox);
        assert_eq!(
            checkbox.toggled(),
            Some(if selected {
                Toggled::True
            } else {
                Toggled::False
            })
        );
        let bounds = checkbox.bounds().unwrap();
        assert!(((bounds.x0 + bounds.x1) * 0.5 - 311.0).abs() <= 1.0);
        assert!(((bounds.y0 + bounds.y1) * 0.5 - (222.0 + index as f64 * 56.0)).abs() <= 1.0);
    }
    click(&mut renderer, "Events", Role::CheckBox);
    click(&mut renderer, "Projects", Role::CheckBox);
    click(&mut renderer, "Close filters", Role::Button);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| {
                node.role() != Role::CheckBox
                    && !(node.role() == Role::Button && crate::has_label(&node, "Close filters"))
            })
    );
    click(&mut renderer, "Open filters", Role::Button);
    for (label, selected) in [("Events", true), ("Projects", false), ("Reminders", true)] {
        assert_eq!(
            node(&renderer, label, Role::CheckBox).toggled(),
            Some(if selected {
                Toggled::True
            } else {
                Toggled::False
            })
        );
    }
}
