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

#[tessera]
fn checkout_fixture() {
    material_theme().child(|| {
        page_figures::checkout(true).initial(0);
    });
}

#[tessera]
fn language_fixture() {
    material_theme().child(page_figures::language);
}

#[tessera]
fn matrix_fixture() {
    material_theme().child(states);
}

fn click_button(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing radio figure action {label}"))
        .1
        .bounds()
        .unwrap();
    click(
        renderer,
        (bounds.x0 + bounds.x1) * 0.5,
        (bounds.y0 + bounds.y1) * 0.5,
    );
}

fn click_label(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = radio(renderer, label).1.bounds().unwrap();
    click(renderer, bounds.x1 + 12.0, (bounds.y0 + bounds.y1) * 0.5);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkout_label_selection_remains_exclusive_and_explicit_clear_works_after_resize() {
    let mut renderer = renderer(checkout_fixture);
    for width in [1024, 390, 1024] {
        renderer.resize(width, 800);
        renderer.render_frames(12);
        click_label(&mut renderer, "Office");
        assert_eq!(radio(&renderer, "Office").1.toggled(), Some(Toggled::True));
        assert_eq!(radio(&renderer, "Home").1.toggled(), Some(Toggled::False));
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            radio(&renderer, "Office").0
        );
        click_label(&mut renderer, "Office");
        assert_eq!(
            radio(&renderer, "Office").1.toggled(),
            Some(Toggled::True),
            "Selecting a selected radio must not clear the group"
        );
        click_button(&mut renderer, "Clear shipping selection");
        for label in ["Home", "Office"] {
            assert_eq!(radio(&renderer, label).1.toggled(), Some(Toggled::False));
            let bounds = radio(&renderer, label).1.bounds().unwrap();
            assert_eq!((bounds.width(), bounds.height()), (48.0, 48.0));
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn language_dialog_cancel_restores_saved_choice_and_continue_commits_the_draft() {
    let mut renderer = renderer(language_fixture);
    renderer.resize(676, 900);
    renderer.render_frames(12);
    assert_eq!(radio(&renderer, "Spanish").1.toggled(), Some(Toggled::True));
    click_label(&mut renderer, "English");
    assert_eq!(radio(&renderer, "English").1.toggled(), Some(Toggled::True));
    click_button(&mut renderer, "Cancel language selection");
    assert_eq!(radio(&renderer, "Spanish").1.toggled(), Some(Toggled::True));
    click_label(&mut renderer, "Chinese (Mandarin)");
    click_button(&mut renderer, "Confirm language selection");
    click_label(&mut renderer, "English");
    click_button(&mut renderer, "Cancel language selection");
    assert_eq!(
        radio(&renderer, "Chinese (Mandarin)").1.toggled(),
        Some(Toggled::True)
    );
    for label in ["Spanish", "English"] {
        assert_eq!(radio(&renderer, label).1.toggled(), Some(Toggled::False));
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn state_matrix_keeps_twenty_native_radios_and_disabled_examples_inert_after_resize() {
    let mut renderer = renderer(matrix_fixture);
    for width in [1024, 390, 1024] {
        renderer.resize(width, 1000);
        renderer.render_frames(12);
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        assert_eq!(
            nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::RadioButton)
                .count(),
            20
        );
        let disabled = radio(&renderer, "Light Unselected disabled radio").1;
        assert!(disabled.is_disabled());
        let bounds = disabled.bounds().unwrap();
        click(
            &mut renderer,
            (bounds.x0 + bounds.x1) * 0.5,
            (bounds.y0 + bounds.y1) * 0.5,
        );
        assert_eq!(
            radio(&renderer, "Light Unselected disabled radio")
                .1
                .toggled(),
            Some(Toggled::False)
        );
    }
}
