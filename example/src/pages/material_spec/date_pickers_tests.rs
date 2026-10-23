//! Date-field anchoring and calendar interaction regressions.
//!
//! ## Usage
//!
//! Keep the input stationary while its optional calendar opens and closes.

use std::cell::Cell;

use tessera_components::theme::material_theme;
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, RenderModule, State,
    accesskit::{Action, ActionRequest, Node, Role, TreeId},
    tessera,
};

thread_local! {
    pub(super) static GEOMETRY: Cell<Option<State<super::Geometry>>> = const { Cell::new(None) };
}

struct Pipelines;

impl RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        super::date_figure(super::Figure::Docked, true, false);
    });
}

#[tessera]
fn article_fixture() {
    material_theme().child(super::date_pickers_spec);
}

fn field(renderer: &HeadlessRenderer) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::TextInput && node.label() == Some("Event date"))
        .expect("date field")
        .1
}

fn activate(renderer: &mut HeadlessRenderer, label: &str) {
    let target_node = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.supports_action(Action::Click))
        .unwrap_or_else(|| panic!("missing actionable {label}"))
        .0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(3);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn docked_field_keeps_its_anchor_and_calendar_selection_updates_real_input() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 800,
            height: 800,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let before = field(&renderer).bounds().unwrap();
    let visual = GEOMETRY.get().unwrap().with(|geometry| geometry["field"]);
    assert_eq!(visual.height.0, 56);
    let pixels = renderer.read_pixels().unwrap();
    let mut ink = 0;
    for y in before.y0 as usize..before.y1 as usize {
        for x in before.x0 as usize..(before.x0 + 128.0).min(before.x1) as usize {
            let pixel = &pixels[(y * 800 + x) * 4..][..3];
            ink += usize::from(pixel.iter().all(|channel| *channel < 128));
        }
    }
    assert!(
        ink > 20,
        "the selected date must be painted inside the field"
    );
    activate(&mut renderer, "Toggle event date calendar");
    assert_eq!(field(&renderer).bounds(), Some(before));
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Docked calendar"))
    );
    activate(&mut renderer, "Toggle event date calendar");
    assert_eq!(field(&renderer).bounds(), Some(before));
    activate(&mut renderer, "August 19, 2025");
    assert_eq!(field(&renderer).value(), Some("08/19/2025"));
    activate(&mut renderer, "August 26, 2025");
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let selected = snapshot
        .nodes
        .iter()
        .find(|(_, node)| {
            node.label() == Some("August 26, 2025") && node.supports_action(Action::Click)
        })
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let annotation = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Note && node.label() == Some("Anatomy marker 10"))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        (annotation.y0 + annotation.y1) / 2.0,
        (selected.y0 + selected.y1) / 2.0
    );
    activate(&mut renderer, "OK");
    assert_eq!(field(&renderer).bounds(), Some(before));
    assert_eq!(field(&renderer).value(), Some("08/26/2025"));
    activate(&mut renderer, "Toggle event date calendar");
    activate(&mut renderer, "August 20, 2025");
    assert_eq!(field(&renderer).value(), Some("08/20/2025"));
    activate(&mut renderer, "Cancel");
    assert_eq!(field(&renderer).bounds(), Some(before));
    assert_eq!(field(&renderer).value(), Some("08/26/2025"));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn date_article_switches_real_chapters_and_unmounts_inactive_pages() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(article_fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 8000,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    for (page, present, absent) in [
        ("Overview", "Differences from M2", "Tokens & specs"),
        ("Guidelines", "Usage", "Differences from M2"),
        ("Accessibility", "Date entry methods", "Usage"),
        ("Specs", "Tokens & specs", "Date entry methods"),
    ] {
        activate(&mut renderer, page);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(snapshot.nodes.iter().any(|(_, node)| {
            node.role() == Role::Tab
                && node.label() == Some(page)
                && node.is_selected() == Some(true)
        }));
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Heading && node.label() == Some(present)),
            "missing {present} on {page}"
        );
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Heading && node.label() == Some(absent)),
            "inactive {absent} remained on {page}"
        );
    }
}
