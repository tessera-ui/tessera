//! Actual article route lazy navigation regressions.
//!
//! ## Usage
//!
//! Keep cold section navigation, wheel ordering and native example state.

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, Role, Toggled, TreeId},
};

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

fn find(
    renderer: &HeadlessRenderer,
    label: &str,
    role: Option<Role>,
) -> Option<(tessera_ui::accesskit::NodeId, Node)> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.label() == Some(label)
                && role.is_none_or(|role| node.role() == role)
                && !node.is_hidden()
                && node.bounds().is_some()
        })
}

fn reveal(
    renderer: &mut HeadlessRenderer,
    label: &str,
    role: Option<Role>,
    direction: f32,
) -> (tessera_ui::accesskit::NodeId, Node) {
    let scroll_x = (renderer.physical_size().width.min(907) as f32 - 37.0).max(1.0);
    reveal_with_scroll_x(renderer, label, role, direction, scroll_x)
}

fn reveal_with_scroll_x(
    renderer: &mut HeadlessRenderer,
    label: &str,
    role: Option<Role>,
    direction: f32,
    scroll_x: f32,
) -> (tessera_ui::accesskit::NodeId, Node) {
    for _ in 0..48 {
        if let Some(target) = find(renderer, label, role) {
            let bounds = target.1.bounds().unwrap();
            let center = (bounds.y0 + bounds.y1) * 0.5;
            if center
                > (if role == Some(Role::Link) {
                    150.0
                } else {
                    80.0
                })
                && center < f64::from(renderer.physical_size().height) - 36.0
            {
                return target;
            }
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(scroll_x, 430.0),
                0.0,
                (430.0 - center).clamp(-1000.0, 1000.0) as f32,
            );
        } else {
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(scroll_x, 430.0),
                0.0,
                direction,
            );
        }
        renderer.render_frames(20);
    }
    let visible = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .filter_map(|(_, node)| {
            node.label()
                .map(|label| (label.to_owned(), node.role(), node.bounds()))
        })
        .collect::<Vec<_>>();
    panic!("real article scroll did not reveal {label} {role:?}: {visible:?}");
}

fn click(
    renderer: &mut HeadlessRenderer,
    label: &str,
    role: Option<Role>,
    direction: f32,
    wheel: Option<f32>,
) {
    let target = reveal(renderer, label, role, direction);
    let bounds = target.1.bounds().unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    if let Some(wheel) = wheel {
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(870.0, 430.0),
            0.0,
            wheel,
        );
    }
    renderer.render_frames(12);
}

#[test]
#[ignore = "requires a native GPU adapter for actual directory and article navigation"]
fn text_field_route_compacts_navigation_and_reveals_retained_outlined_callouts_and_border() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(crate::app::app),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material Components", None, -600.0, None);
    renderer.render_frames(60);
    click(
        &mut renderer,
        "Open Text fields specifications",
        None,
        -600.0,
        None,
    );
    let specs = reveal(&mut renderer, "Specs", Some(Role::Tab), 800.0);
    assert_eq!(specs.1.bounds().unwrap().height(), 79.0);
    assert_eq!(
        find(&renderer, "Overview", Some(Role::Tab))
            .unwrap()
            .1
            .is_selected(),
        Some(true)
    );
    click(&mut renderer, "Specs", Some(Role::Tab), 800.0, None);
    assert_eq!(
        find(&renderer, "Specs", Some(Role::Tab))
            .unwrap()
            .1
            .is_selected(),
        Some(true)
    );
    click(
        &mut renderer,
        "Filled text field",
        Some(Role::Link),
        -600.0,
        None,
    );
    let compact = find(&renderer, "Specs", Some(Role::Tab)).unwrap();
    assert_eq!(compact.0, specs.0);
    assert_eq!(compact.1.bounds().unwrap().height(), 63.0);
    assert_eq!(compact.1.bounds().unwrap().y0, 73.0);
    let rgba = renderer.read_pixels().unwrap();
    let under_bar = &rgba[(138 * 907 + 450) * 4..(138 * 907 + 450) * 4 + 3];
    let outside_shadow = &rgba[(145 * 907 + 450) * 4..(145 * 907 + 450) * 4 + 3];
    assert!(
        under_bar
            .iter()
            .zip(outside_shadow)
            .any(|(shadow, background)| { u16::from(*shadow) + 2 < u16::from(*background) }),
        "pinned navigation must draw its shadow outside the clipped scrolling tabs"
    );
    click(
        &mut renderer,
        "Outlined text field",
        Some(Role::Link),
        800.0,
        None,
    );
    let panel = find(&renderer, "Light outlined anatomy panel", Some(Role::Group))
        .expect("revealed outlined anatomy panel")
        .1
        .bounds()
        .unwrap();
    let snapshot = renderer.accessibility_snapshot().unwrap();
    for number in 1..=9 {
        let label = format!("Anatomy marker {number}");
        assert!(
            snapshot.nodes.iter().any(|(_, node)| {
                node.role() == Role::Note
                    && node.label() == Some(label.as_str())
                    && node.bounds().is_some_and(|bounds| {
                        bounds.x0 >= panel.x0
                            && bounds.x1 <= panel.x1
                            && bounds.y0 >= panel.y0
                            && bounds.y1 <= panel.y1
                    })
            }),
            "revealing retained anatomy must place {label} inside {panel:?}"
        );
    }
    let field = find(
        &renderer,
        "Outlined empty anatomy container",
        Some(Role::Group),
    )
    .unwrap()
    .1
    .bounds()
    .unwrap();
    let pixels = renderer.read_pixels().unwrap();
    let x = ((field.x0 + field.x1) * 0.5) as usize;
    let pixel = |y: usize| {
        let offset = (y * 907 + x) * 4;
        &pixels[offset..offset + 3]
    };
    let interior = pixel(field.y1 as usize - 8);
    assert!(
        (field.y1 as usize - 2..=field.y1 as usize).any(|y| {
            pixel(y)
                .iter()
                .zip(interior)
                .any(|(edge, inside)| edge.abs_diff(*inside) >= 16)
        }),
        "revealing a retained outlined field must restore its visible bottom border"
    );

    click(&mut renderer, "Specs", Some(Role::Tab), 800.0, None);
    renderer.render_frames(24);
    let expanded = find(&renderer, "Specs", Some(Role::Tab)).unwrap();
    assert_eq!(expanded.0, specs.0);
    assert_eq!(expanded.1.bounds().unwrap().height(), 79.0);
    click(
        &mut renderer,
        "Filled text field",
        Some(Role::Link),
        -600.0,
        None,
    );
    renderer.resize(1400, 871);
    renderer.render_frames(24);
    let first = find(&renderer, "Overview", Some(Role::Tab))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let last = find(&renderer, "Accessibility", Some(Role::Tab))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert!(
        (last.x1 - first.x0 - 1024.0).abs() < 0.1,
        "wide pinned navigation must span 1024px: {first:?} .. {last:?}"
    );
    assert!(
        (first.x0 - (1400.0 - 15.0 - 1024.0) * 0.5).abs() < 0.1,
        "wide pinned navigation must be centered: {first:?} .. {last:?}"
    );
    assert_eq!(
        find(&renderer, "Specs", Some(Role::Tab)).unwrap().0,
        specs.0
    );
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(1300.0, 600.0),
        0.0,
        -520.0,
    );
    renderer.render_frames(30);
    let pixels = renderer.read_pixels().unwrap();
    let y = ((first.y0 + first.y1) / 2.0) as usize;
    let pixel = |x| &pixels[(y * 1400 + x) * 4..][..4];
    assert_eq!(
        pixel(64),
        pixel(12),
        "the pinned header must cover chapter figures beside its centered tabs"
    );
    assert_eq!(
        pixel(1320),
        pixel(1372),
        "the pinned header must cover chapter figures on both sides"
    );
}

#[test]
#[ignore = "requires a native GPU adapter for actual directory and article navigation"]
fn real_buttons_route_retains_visited_native_state_reveals_cold_sections_and_preserves_queued_wheel()
 {
    let mut renderer = HeadlessRenderer::new(
        Box::new(crate::app::app),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material Components", None, -600.0, None);
    for _ in 0..90 {
        let tree = renderer.accessibility_snapshot().unwrap();
        if find(&renderer, "Open Buttons specifications", None).is_some()
            && !tree
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Dialog)
        {
            break;
        }
        renderer.render_frames(1);
    }
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog)
    );
    click(
        &mut renderer,
        "Open Buttons specifications",
        None,
        -600.0,
        None,
    );
    let specs_id = reveal(&mut renderer, "Specs", Some(Role::Tab), 800.0).0;
    click(&mut renderer, "Guidelines", Some(Role::Tab), 800.0, None);
    assert_eq!(
        find(&renderer, "Guidelines", Some(Role::Tab))
            .unwrap()
            .1
            .is_selected(),
        Some(true)
    );
    click(&mut renderer, "Specs", Some(Role::Tab), 800.0, Some(-120.0));
    assert_eq!(
        find(&renderer, "Specs", Some(Role::Tab))
            .unwrap()
            .1
            .is_selected(),
        Some(true)
    );
    assert_eq!(
        find(&renderer, "Specs", Some(Role::Tab)).unwrap().0,
        specs_id,
        "Switching chapters must retain the actual tab identity"
    );
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        specs_id,
        "The clicked tab must keep keyboard focus after chapter replacement"
    );
    let moved = find(&renderer, "Buttons", Some(Role::Heading))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert!(
        moved.y0 < 120.0,
        "queued wheel must remain applied after chapter construction: {moved:?}"
    );
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let first_section = find(&renderer, "Variants", Some(Role::Link)).unwrap().0;
    let mut remaining = vec![snapshot.tree.as_ref().unwrap().root];
    let mut reading_order = Vec::new();
    while let Some(id) = remaining.pop() {
        reading_order.push(id);
        let node = &snapshot.nodes.iter().find(|(key, _)| *key == id).unwrap().1;
        remaining.extend(node.children().iter().rev().copied());
    }
    assert!(
        reading_order
            .iter()
            .position(|id| *id == specs_id)
            .expect("published article tab")
            < reading_order
                .iter()
                .position(|id| *id == first_section)
                .expect("published chapter link"),
        "Article navigation must precede chapter links in assistive reading order"
    );
    tessera_ui::FocusManager::current().request_move_focus(tessera_ui::FocusDirection::Next);
    renderer.render_frames(8);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        first_section,
        "Forward traversal must leave the pinned tabs for the first chapter link"
    );
    assert!(
        find(&renderer, "Shape morph", Some(Role::Heading)).is_none(),
        "Unseen article sections must not be placed before navigation"
    );
    assert!(
        find(&renderer, "Select", Some(Role::Button)).is_none(),
        "Unvisited native examples must not be constructed eagerly"
    );
    click(&mut renderer, "Shape morph", Some(Role::Link), 800.0, None);
    let select = reveal(&mut renderer, "Select", Some(Role::Button), -600.0);
    assert_eq!(select.1.toggled(), Some(Toggled::False));
    click(&mut renderer, "Select", Some(Role::Button), -600.0, None);
    let selected = find(&renderer, "Select", Some(Role::Button)).unwrap();
    assert_eq!(selected.1.toggled(), Some(Toggled::True));
    assert_eq!(selected.1.description(), Some("Activations: 1"));
    let selected_id = selected.0;
    click(&mut renderer, "Measurements", Some(Role::Link), 800.0, None);
    let heading = find(&renderer, "Measurements", Some(Role::Heading))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let pinned_tab = find(&renderer, "Specs", Some(Role::Tab)).unwrap();
    assert_eq!(pinned_tab.0, specs_id);
    let pinned_bounds = pinned_tab.1.bounds().unwrap();
    assert!(
        pinned_bounds.y0 >= 64.0 && pinned_bounds.y1 < 180.0,
        "Article tabs must remain visible above a scrolled chapter: {pinned_bounds:?}"
    );
    assert!(
        heading.y0 >= pinned_bounds.y1 && heading.y0 < pinned_bounds.y1 + 120.0,
        "cold last section must align below the pinned article tabs: {heading:?}"
    );
    let tab_pixels = |renderer: &HeadlessRenderer| {
        let pixels = renderer.read_pixels().unwrap();
        let mut interior = Vec::new();
        for y in (pinned_bounds.y0 as usize + 16)..(pinned_bounds.y1 as usize - 16) {
            for x in (pinned_bounds.x0 as usize + 16)..(pinned_bounds.x1 as usize - 16) {
                let offset = (y * 907 + x) * 4;
                interior.extend_from_slice(&pixels[offset..offset + 4]);
            }
        }
        interior
    };
    let before = tab_pixels(&renderer);
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(870.0, 430.0),
        0.0,
        -120.0,
    );
    renderer.render_frames(20);
    let occluded = tab_pixels(&renderer);
    assert!(
        before == occluded,
        "Chapter content must draw behind the pinned article tabs"
    );
    click(&mut renderer, "Specs", Some(Role::Tab), 800.0, None);
    let header = find(&renderer, "Buttons", Some(Role::Heading))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert!(
        header.y0 >= 64.0,
        "The pinned tab must receive clicks above the overlapping chapter content: {header:?}"
    );
    click(&mut renderer, "Measurements", Some(Role::Link), 800.0, None);
    let label = reveal(&mut renderer, "Label", Some(Role::Button), -600.0)
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        label.height(),
        48.0,
        "real extra-small button keeps its 48dp interaction target"
    );
    click(&mut renderer, "Shape morph", Some(Role::Link), 800.0, None);
    let restored = reveal(&mut renderer, "Select", Some(Role::Button), -600.0);
    assert_eq!(
        restored.0, selected_id,
        "visited composition must retain the original actual control identity"
    );
    assert_eq!(restored.1.toggled(), Some(Toggled::True));
    assert_eq!(restored.1.description(), Some("Activations: 1"));
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: selected_id,
        data: None,
    }));
    renderer.render_frames(12);
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(870.0, 430.0),
        0.0,
        -1200.0,
    );
    renderer.render_frames(60);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        selected_id
    );
    let offscreen = find(&renderer, "Select", Some(Role::Button)).unwrap();
    let offscreen_bounds = offscreen.1.bounds().unwrap();
    assert!(
        offscreen_bounds.y1 < 64.0 || offscreen_bounds.y0 > 871.0,
        "Manual wheel input must move the focused control outside the viewport: {offscreen_bounds:?}"
    );
    renderer.render_frames(12);
    assert_eq!(
        find(&renderer, "Select", Some(Role::Button))
            .unwrap()
            .1
            .bounds(),
        Some(offscreen_bounds),
        "Idle frames must not undo manual scrolling to reveal an already focused item"
    );
    reveal(&mut renderer, "Select", Some(Role::Button), 800.0);
    let selected = find(&renderer, "Select", Some(Role::Button)).unwrap();
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        selected_id
    );
    assert_eq!(selected.1.toggled(), Some(Toggled::True));
    assert_eq!(selected.1.description(), Some("Activations: 1"));
    let bounds = selected.1.bounds().unwrap();
    assert!(
        bounds.y0 >= 64.0 && bounds.y1 <= 871.0,
        "reverse scrolling must reveal the original retained control: {bounds:?}"
    );
}

#[test]
#[ignore = "requires a native GPU adapter for actual date configuration navigation"]
fn saving_date_range_configuration_preserves_article_position() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(crate::app::app),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 1280,
            height: 800,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(32);
    click(&mut renderer, "Material Components", None, -600.0, None);
    renderer.render_frames(60);
    click(
        &mut renderer,
        "Open Date pickers specifications",
        None,
        -600.0,
        None,
    );
    click(&mut renderer, "Specs", Some(Role::Tab), -800.0, None);
    assert_eq!(
        find(&renderer, "Specs", Some(Role::Tab))
            .unwrap()
            .1
            .is_selected(),
        Some(true)
    );
    click(
        &mut renderer,
        "Modal date picker",
        Some(Role::Link),
        -600.0,
        None,
    );
    let scroll_x = renderer.physical_size().width as f32 - 22.0;
    let target = reveal_with_scroll_x(
        &mut renderer,
        "Save configuration date range",
        Some(Role::Button),
        -500.0,
        scroll_x,
    );
    let heading = find(
        &renderer,
        "Modal date picker configurations",
        Some(Role::Heading),
    )
    .unwrap()
    .1
    .bounds()
    .unwrap();
    let bounds = target.1.bounds().unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(1);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    for frame in 0..12 {
        renderer.render_frames(1);
        let current = find(
            &renderer,
            "Modal date picker configurations",
            Some(Role::Heading),
        )
        .unwrap()
        .1
        .bounds()
        .unwrap();
        let tree = renderer.accessibility_snapshot().unwrap();
        let focus = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .map(|(_, node)| (node.label(), node.bounds()));
        assert!(
            (current.y0 - heading.y0).abs() <= 1.0,
            "configuration action moved article at frame {frame}: {heading:?} -> {current:?}, focus {focus:?}"
        );
    }
    assert!(find(&renderer, "Date range saved", None).is_some());
}

#[test]
#[ignore = "requires a native GPU adapter for actual carousel article navigation"]
fn carousel_collection_page_keeps_return_visible_and_restores_article_position() {
    for width in [1280, 390] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(crate::app::app),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width,
                height: 800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(32);
        click(&mut renderer, "Material Components", None, -600.0, None);
        renderer.render_frames(60);
        click(
            &mut renderer,
            "Open Carousel specifications",
            None,
            -600.0,
            None,
        );
        if width == 390 {
            for page in ["Specs", "Guidelines", "Accessibility"] {
                click(
                    &mut renderer,
                    "Go to next tab",
                    Some(Role::Button),
                    -700.0,
                    None,
                );
                assert_eq!(
                    find(&renderer, page, Some(Role::Tab))
                        .unwrap()
                        .1
                        .is_selected(),
                    Some(true),
                    "the next-tab arrow must select {page}"
                );
            }
        } else {
            click(
                &mut renderer,
                "Accessibility",
                Some(Role::Tab),
                -700.0,
                None,
            );
        }
        assert_eq!(
            find(&renderer, "Accessibility", Some(Role::Tab))
                .unwrap()
                .1
                .is_selected(),
            Some(true)
        );
        click(
            &mut renderer,
            "Keyboard navigation",
            Some(Role::Link),
            -500.0,
            None,
        );
        let target = reveal(
            &mut renderer,
            "Show all carousel items",
            Some(Role::Button),
            -500.0,
        );
        let before = target.1.bounds().unwrap();
        let point = PxPosition::from_f32_xy(
            ((before.x0 + before.x1) * 0.5) as f32,
            ((before.y0 + before.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(8);
        for frame in 0..16 {
            renderer.step_frame();
            let after = find(&renderer, "Back to carousel", Some(Role::Button))
                .expect("collection page return control");
            let bounds = after.1.bounds().unwrap();
            assert!(
                bounds.y0 >= 64.0 && bounds.y1 <= 145.0,
                "return action must stay in the collection page header at frame {frame}: {bounds:?}"
            );
            let page = find(&renderer, "All carousel collections", Some(Role::Dialog))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                (page.y0 - 64.0).abs() <= 1.0 && (page.y1 - 800.0).abs() <= 1.0,
                "collection page must use the full content viewport height: {page:?}"
            );
            if width == 390 {
                assert_eq!(
                    page.width(),
                    390.0,
                    "compact collection page must not have modal outer margins"
                );
            }
        }
        click(
            &mut renderer,
            "Back to carousel",
            Some(Role::Button),
            500.0,
            None,
        );
        let restored = find(&renderer, "Show all carousel items", Some(Role::Button)).unwrap();
        assert_eq!(restored.0, target.0);
        assert!((restored.1.bounds().unwrap().y0 - before.y0).abs() <= 1.0);
    }
}

#[tessera_ui::tessera]
fn responsive_article_fixture() {
    use tessera_components::{
        boxed::boxed,
        modifier::{ModifierExt as _, SemanticsArgs},
        theme::material_theme,
    };
    use tessera_ui::{Dp, Modifier, RenderSlot};

    material_theme().child(|| {
        super::super::article(
            "Layout",
            "Responsive article",
            vec![super::super::SpecSection {
                title: "Container sizing",
                content: RenderSlot::new(|| {
                    boxed().modifier(Modifier::new().fill_max_width().height(Dp(64.0)).semantics(
                        SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some("Measured article body".into()),
                            ..Default::default()
                        },
                    ));
                }),
            }],
        );
    });
}

#[test]
#[ignore = "requires a native GPU adapter for actual responsive article layout"]
fn article_body_and_contents_share_responsive_gutters_and_bounded_desktop_width() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(responsive_article_fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 390,
            height: 3000,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    for (width, expected_width, expected_left) in [
        (390, 327.0, 24.0),
        (800, 705.0, 40.0),
        (1024, 833.0, 88.0),
        (1280, 1024.0, 120.5),
        (1600, 1024.0, 280.5),
        (390, 327.0, 24.0),
    ] {
        renderer.resize(width, 3000);
        renderer.render_frames(12);
        let body = find(&renderer, "Measured article body", Some(Role::Group))
            .expect("composed section")
            .1
            .bounds()
            .unwrap();
        let contents = find(&renderer, "Container sizing", Some(Role::Link))
            .expect("table of contents")
            .1
            .bounds()
            .unwrap();
        assert!(
            (body.width() - expected_width).abs() <= 1.0,
            "body width at {width}: {body:?}"
        );
        assert!(
            (body.x0 - expected_left).abs() <= 1.0,
            "body gutter at {width}: {body:?}"
        );
        assert!(
            (contents.x0 - body.x0).abs() <= 1.0,
            "contents and body must share a gutter at {width}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter for actual directory and article navigation"]
fn required_field_remains_pointer_editable_after_article_tab_and_scrolled_reveal() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(crate::app::app),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 1650,
            height: 1200,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material Components", None, -600.0, None);
    renderer.render_frames(60);
    click(
        &mut renderer,
        "Open Text fields specifications",
        None,
        -600.0,
        None,
    );
    click(&mut renderer, "Accessibility", Some(Role::Tab), 800.0, None);
    assert_eq!(
        find(&renderer, "Accessibility", Some(Role::Tab))
            .unwrap()
            .1
            .is_selected(),
        Some(true)
    );
    click(
        &mut renderer,
        "Labeling elements",
        Some(Role::Link),
        -600.0,
        None,
    );
    reveal(&mut renderer, "*required", Some(Role::Label), -600.0);
    let support = find(&renderer, "*required", Some(Role::Label))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let scale = support.height() / 16.0;
    let (id, field) = find(&renderer, "Username*", Some(Role::TextInput)).unwrap();
    let bounds = field.bounds().unwrap();
    for (actual, expected) in [
        (bounds.x0, support.x0),
        (bounds.y0, support.y0 - 44.0 * scale),
        (bounds.width(), 295.0 * scale),
        (bounds.height(), 24.0 * scale),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "required editor {bounds:?} must match its independently positioned supporting label {support:?}"
        );
    }
    let before = renderer.read_pixels().unwrap();
    let point = PxPosition::from_f32_xy(
        (support.x0 + 122.0 * scale) as f32,
        (support.y0 - 32.0 * scale) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(3);
    assert_eq!(field.description(), Some("Required"));
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        id,
        "the visible required editor must receive pointer focus after article scrolling"
    );
    renderer.inject_text("Q");
    renderer.render_frames(2);
    assert_eq!(
        find(&renderer, "Username*", Some(Role::TextInput))
            .unwrap()
            .1
            .value(),
        Some("Q")
    );
    let after = renderer.read_pixels().unwrap();
    let mut changed = false;
    for y in bounds.y0.floor().max(0.0) as usize..bounds.y1.ceil().min(1200.0) as usize {
        for x in bounds.x0.floor().max(0.0) as usize..bounds.x1.ceil().min(1650.0) as usize {
            let offset = (y * 1650 + x) * 4;
            changed |= before[offset..offset + 3] != after[offset..offset + 3];
        }
    }
    assert!(
        changed,
        "real article input must repaint the visible editor after scrolling"
    );
}
