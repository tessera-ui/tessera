//! Native time-picker figure surface regressions.
//!
//! ## Usage
//!
//! Keep the dialog background around its content padding.

use std::{cell::Cell, time::Duration};

use tessera_components::{column::column, modifier::ModifierExt as _, theme::material_theme};
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition,
    RenderModule, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, TreeId},
    tessera,
    winit::event::Ime,
};

thread_local! {
    static INPUT: Cell<bool> = const { Cell::new(false) };
    static TWENTY_FOUR: Cell<bool> = const { Cell::new(false) };
    pub(super) static GEOMETRY: Cell<Option<State<super::PreciseGeometry>>> = const { Cell::new(None) };
}

#[tessera]
fn color_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                super::time_color_pair(INPUT.get());
            });
    });
}

#[tessera]
fn configuration_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                super::time_configuration_pair(INPUT.get(), TWENTY_FOUR.get());
            });
    });
}

fn renderer(root: fn()) -> HeadlessRenderer {
    renderer_at_width(root, 1100)
}

fn renderer_at_width(root: fn(), width: u32) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(root),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width,
            height: 1400,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    renderer
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn narrow_configuration_keeps_horizontal_dial_and_maps_pointer_selection() {
    INPUT.set(false);
    TWENTY_FOUR.set(false);
    let mut renderer = renderer_at_width(configuration_fixture, 640);
    let left = node(
        &renderer,
        "12-hour vertical dial time configuration",
        Role::Group,
        None,
    )
    .1
    .bounds()
    .unwrap();
    let right = node(
        &renderer,
        "12-hour horizontal dial time configuration",
        Role::Group,
        None,
    )
    .1
    .bounds()
    .unwrap();
    assert!((left.width() - 205.0).abs() <= 0.05);
    assert!((right.width() - 357.5).abs() <= 0.05);
    assert!(
        left.x1 < right.x0,
        "the paired native artboard must not wrap"
    );
    let target = node(&renderer, "Hour 3 of 12", Role::Button, Some(right))
        .1
        .bounds()
        .unwrap();
    assert!((target.width() - 30.0).abs() <= 0.05);
    assert!((target.height() - 30.0).abs() <= 0.05);
    let point = PxPosition::from_f32_xy(
        ((target.x0 + target.x1) * 0.5) as f32,
        ((target.y0 + target.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
    assert_eq!(
        node(&renderer, "Hour", Role::Button, Some(right)).1.value(),
        Some("03")
    );
    assert_eq!(
        node(&renderer, "Hour", Role::Button, Some(left)).1.value(),
        Some("07")
    );
}

fn node(
    renderer: &HeadlessRenderer,
    label: &str,
    role: Role,
    within: Option<Rect>,
) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.role() == role
                && node.label() == Some(label)
                && within.is_none_or(|parent| {
                    node.bounds().is_some_and(|bounds| {
                        bounds.x0 >= parent.x0
                            && bounds.x1 <= parent.x1
                            && bounds.y0 >= parent.y0
                            && bounds.y1 <= parent.y1
                    })
                })
        })
        .unwrap_or_else(|| panic!("missing {label} ({role:?}) inside {within:?}"))
}

fn action(renderer: &mut HeadlessRenderer, target_node: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(4);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn time_color_pairs_keep_native_roles_aligned_and_selection_independent() {
    for input in [false, true] {
        INPUT.set(input);
        let mut renderer = renderer(color_fixture);
        let kind = if input { "input" } else { "dial" };
        let left = node(
            &renderer,
            &format!("Light {kind} color annotated time picker"),
            Role::Group,
            None,
        )
        .1
        .bounds()
        .unwrap();
        let right = node(
            &renderer,
            &format!("Dark {kind} color annotated time picker"),
            Role::Group,
            None,
        )
        .1
        .bounds()
        .unwrap();
        assert!(left.x1 < right.x0);
        assert!((left.width() - 328.0 * 1100.0 / 1024.0).abs() <= 0.05);
        assert_eq!(left.height(), right.height());
        let role = if input { Role::TextInput } else { Role::Button };
        let hour = node(&renderer, "Hour", role, Some(left))
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            node(&renderer, "Hour", role, Some(right)).1.value(),
            Some("07")
        );
        let marker_label = format!(
            "Time color role {}: Primary container",
            if input { 13 } else { 17 }
        );
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let markers: Vec<_> = snapshot
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.role() == Role::Note && node.label() == Some(marker_label.as_str())
            })
            .collect();
        assert_eq!(markers.len(), 2);
        let marker = markers
            .iter()
            .find(|(_, node)| node.bounds().unwrap().x0 < 550.0)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert!(marker.x1 < hour.x0);
        let badges = snapshot
            .nodes
            .iter()
            .filter_map(|(_, node)| {
                (node.role() == Role::Note
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Time color role ")))
                .then(|| (node.label().unwrap(), node.bounds().unwrap()))
            })
            .collect::<Vec<_>>();
        for (index, (label, bounds)) in badges.iter().enumerate() {
            for (other_label, other) in &badges[index + 1..] {
                assert!(
                    bounds.x1 <= other.x0
                        || other.x1 <= bounds.x0
                        || bounds.y1 <= other.y0
                        || other.y1 <= bounds.y0,
                    "time color badges overlap: {label} {bounds:?}, {other_label} {other:?}"
                );
            }
        }
        let expected = if input { 13 } else { 17 };
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Note
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Time color role ")))
                .count(),
            expected * 2
        );
        let pixels = renderer.read_pixels().unwrap();
        assert_ne!(
            &pixels[(20 * 1100 + 100) * 4..][..3],
            &pixels[(20 * 1100 + 650) * 4..][..3]
        );
        if input {
            let target = node(&renderer, "Hour", role, Some(left)).0;
            action(&mut renderer, target, Action::Focus);
            renderer.inject_ime_event(Ime::Commit("3".into()));
            renderer.render_frames(4);
        } else {
            let target = node(&renderer, "Hour 3 of 12", Role::Button, Some(left)).0;
            action(&mut renderer, target, Action::Click);
        }
        assert_eq!(
            node(&renderer, "Hour", role, Some(left)).1.value(),
            Some(if input { "3" } else { "03" })
        );
        assert_eq!(
            node(&renderer, "Hour", role, Some(right)).1.value(),
            Some("07")
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn time_configuration_pairs_keep_native_orientation_and_independent_lifetime() {
    for (input, twenty_four, left_label, right_label, left_width) in [
        (
            false,
            false,
            "12-hour vertical dial",
            "12-hour horizontal dial",
            328.0,
        ),
        (
            false,
            true,
            "24-hour vertical dial",
            "24-hour horizontal dial",
            304.0,
        ),
        (true, false, "12-hour input", "24-hour input", 328.0),
    ] {
        INPUT.set(input);
        TWENTY_FOUR.set(twenty_four);
        let mut renderer = renderer(configuration_fixture);
        let left = node(
            &renderer,
            &format!("{left_label} time configuration"),
            Role::Group,
            None,
        )
        .1
        .bounds()
        .unwrap();
        let right = node(
            &renderer,
            &format!("{right_label} time configuration"),
            Role::Group,
            None,
        )
        .1
        .bounds()
        .unwrap();
        assert!(left.x1 < right.x0);
        assert!((left.width() - left_width * 1100.0 / 1024.0).abs() <= 0.05);
        assert!(
            (right.width() - (if input { 304.0 } else { 572.0 }) * 1100.0 / 1024.0).abs() <= 0.05
        );
        let role = if input { Role::TextInput } else { Role::Button };
        let hour = node(&renderer, "Hour", role, Some(left))
            .1
            .bounds()
            .unwrap();
        let minute = node(&renderer, "Minute", role, Some(left))
            .1
            .bounds()
            .unwrap();
        assert!(hour.x1 < minute.x0);
        assert_eq!(hour.y0, minute.y0);
        if input {
            let target = node(&renderer, "Hour", role, Some(left)).0;
            action(&mut renderer, target, Action::Focus);
            renderer.inject_ime_event(Ime::Commit("3".into()));
            renderer.render_frames(4);
        } else {
            let target = node(
                &renderer,
                &format!("Hour 3 of {}", if twenty_four { 24 } else { 12 }),
                Role::Button,
                Some(left),
            )
            .0;
            action(&mut renderer, target, Action::Click);
        }
        assert_eq!(
            node(&renderer, "Hour", role, Some(right)).1.value(),
            Some(if input || twenty_four { "20" } else { "07" })
        );
        let confirm = node(
            &renderer,
            &format!("{left_label} confirm time"),
            Role::Button,
            Some(left),
        )
        .0;
        action(&mut renderer, confirm, Action::Click);
        assert!(
            renderer.accessibility_snapshot().unwrap().nodes.iter().all(
                |(_, node)| node.label() != Some(format!("{left_label} time preview").as_str())
            )
        );
        node(&renderer, "Hour", role, Some(right));
        let reopen = node(
            &renderer,
            &format!("Reopen {left_label}"),
            Role::Button,
            None,
        )
        .0;
        action(&mut renderer, reopen, Action::Click);
        let restored = node(
            &renderer,
            &format!("{left_label} time configuration"),
            Role::Group,
            None,
        )
        .1
        .bounds()
        .unwrap();
        assert_eq!(
            node(&renderer, "Hour", role, Some(restored)).1.value(),
            Some("03")
        );
    }
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
        super::time_figure(
            false,
            false,
            false,
            super::FigureKind::Specs,
            "Surface".into(),
        );
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn annotated_time_surface_paints_its_24dp_content_inset() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 800,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let geometry = GEOMETRY.get().unwrap().get();
    let container = geometry["container"];
    let headline = geometry["headline"];
    assert_eq!(container.local_size.width.0, 328);
    let local_headline = container
        .local_to_window
        .inverse_point(headline.local_to_window.translation());
    assert!((local_headline.x - 24.0).abs() <= 0.01);
    let container = container.window_bounds();
    assert!((container.width - 328.0 * 900.0 / 1024.0).abs() <= 0.01);
    let pixels = renderer.read_pixels().unwrap();
    let y = (container.y + container.height / 2.0) as usize;
    let pixel = |x: i32| &pixels[(y * 900 + x as usize) * 4..][..4];
    assert_ne!(
        pixel((container.x + 8.0) as i32),
        pixel((container.x - 8.0) as i32),
        "content padding must be inside the painted dialog surface"
    );
}
