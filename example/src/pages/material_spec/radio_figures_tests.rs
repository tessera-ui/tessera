//! Radio figure selection and compact label interaction regressions.
//!
//! ## Usage
//!
//! Verify native selection through radio targets and their adjacent labels.

use std::time::Duration;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, NodeId, Role, Toggled},
};

use super::*;

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn ringtone_fixture() {
    material_theme().child(|| {
        ringtone();
    });
}

#[tessera]
fn overview_fixture() {
    material_theme().child(|| {
        overview();
    });
}

fn renderer(fixture: fn()) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 600,
            height: 300,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    renderer
}

fn radio(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    let radio = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::RadioButton && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing radio {label}"));
    assert_eq!(
        radio.1.is_selected(),
        None,
        "{label} must publish its radio state through toggled semantics"
    );
    radio
}

fn click(renderer: &mut HeadlessRenderer, x: f64, y: f64) {
    let position = PxPosition::from_f32_xy(x as f32, y as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(4);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn compact_ringtone_places_four_native_options_and_adjacent_label_selects_its_radio() {
    let mut renderer = renderer(ringtone_fixture);
    let mut bottom = 0.0;
    for (label, selected) in [
        ("None", false),
        ("Callisto", true),
        ("Ganymede", false),
        ("Luna", false),
    ] {
        let node = radio(&renderer, label).1;
        assert_eq!(
            node.toggled(),
            Some(if selected {
                Toggled::True
            } else {
                Toggled::False
            })
        );
        let bounds = node.bounds().unwrap();
        assert!(bounds.x0 >= 0.0 && bounds.x1 <= 600.0);
        assert!(bounds.y0 >= bottom && bounds.height() > 0.0);
        assert!(
            bounds.y1 <= 413.0 * 600.0 / 1024.0 + 0.01,
            "{label} must remain inside the compact ringtone artboard: {bounds:?}"
        );
        bottom = bounds.y1;
    }
    let luna = radio(&renderer, "Luna").1.bounds().unwrap();
    click(&mut renderer, luna.x1 + 12.0, (luna.y0 + luna.y1) * 0.5);
    assert_eq!(radio(&renderer, "Luna").1.toggled(), Some(Toggled::True));
    for label in ["None", "Callisto", "Ganymede"] {
        assert_eq!(radio(&renderer, label).1.toggled(), Some(Toggled::False));
    }
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        radio(&renderer, "Luna").0,
        "an adjacent label must focus its own radio"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn overview_two_native_radios_keep_exclusive_selection_on_pointer_activation() {
    let mut renderer = renderer(overview_fixture);
    let radios = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::RadioButton)
        .count();
    assert_eq!(radios, 2);
    assert_eq!(
        radio(&renderer, "First option").1.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        radio(&renderer, "Second option").1.toggled(),
        Some(Toggled::False)
    );
    let second = radio(&renderer, "Second option").1.bounds().unwrap();
    click(
        &mut renderer,
        (second.x0 + second.x1) * 0.5,
        (second.y0 + second.y1) * 0.5,
    );
    assert_eq!(
        radio(&renderer, "First option").1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        radio(&renderer, "Second option").1.toggled(),
        Some(Toggled::True)
    );
}
