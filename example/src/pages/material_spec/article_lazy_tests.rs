use tessera_ui::{
    FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Toggled, TreeId},
};

fn find(
    renderer: &HeadlessRenderer,
    label: &str,
    role: Option<Role>,
) -> Option<(tessera_ui::accesskit::NodeId, Node)> {
    renderer
        .inspect_accessibility()
        .expect("published accessibility")
        .reading_order()
        .find(|(_, node)| {
            crate::has_label(&node, label)
                && role.is_none_or(|role| node.role() == role)
                && !node.is_hidden()
                && node.bounds().is_some()
        })
        .map(|(id, node)| (id, node.clone()))
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
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material components", None, -600.0, None);
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
                    && crate::has_label(&node, label.as_str())
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
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material components", None, -600.0, None);
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
        find(&renderer, "Play", Some(Role::Button)).is_none(),
        "Unvisited native examples must not be constructed eagerly"
    );
    click(&mut renderer, "Shape morph", Some(Role::Link), 800.0, None);
    let select = reveal(&mut renderer, "Play", Some(Role::Button), -600.0);
    assert_eq!(select.1.toggled(), Some(Toggled::False));
    click(&mut renderer, "Play", Some(Role::Button), -600.0, None);
    let selected = find(&renderer, "Play", Some(Role::Button)).unwrap();
    assert_eq!(selected.1.toggled(), Some(Toggled::True));
    assert!(crate::text_matches(
        selected.1.description(),
        "Activations: 1"
    ));
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
    let heading = find(&renderer, "Measurements", Some(Role::Heading))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let semantics = renderer
        .inspect_accessibility()
        .expect("published accessibility");
    let label = semantics
        .reading_order()
        .find(|(_, node)| {
            crate::has_label(&node, "Label")
                && node.role() == Role::Button
                && node.bounds().is_some_and(|bounds| {
                    bounds.y0 >= heading.y1
                        && bounds.y1 < f64::from(renderer.physical_size().height)
                })
        })
        .expect("the first measurement row is visible below its chapter heading")
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        label.height(),
        48.0,
        "real extra-small button keeps its 48dp interaction target"
    );
    click(&mut renderer, "Shape morph", Some(Role::Link), 800.0, None);
    let restored = reveal(&mut renderer, "Play", Some(Role::Button), -600.0);
    assert_eq!(
        restored.0, selected_id,
        "visited composition must retain the original actual control identity"
    );
    assert_eq!(restored.1.toggled(), Some(Toggled::True));
    assert!(crate::text_matches(
        restored.1.description(),
        "Activations: 1"
    ));
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
    let offscreen = find(&renderer, "Play", Some(Role::Button)).unwrap();
    let offscreen_bounds = offscreen.1.bounds().unwrap();
    assert!(
        offscreen_bounds.y1 < 64.0 || offscreen_bounds.y0 > 871.0,
        "Manual wheel input must move the focused control outside the viewport: {offscreen_bounds:?}"
    );
    renderer.render_frames(12);
    assert_eq!(
        find(&renderer, "Play", Some(Role::Button))
            .unwrap()
            .1
            .bounds(),
        Some(offscreen_bounds),
        "Idle frames must not undo manual scrolling to reveal an already focused item"
    );
    reveal(&mut renderer, "Play", Some(Role::Button), 800.0);
    let selected = find(&renderer, "Play", Some(Role::Button)).unwrap();
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        selected_id
    );
    assert_eq!(selected.1.toggled(), Some(Toggled::True));
    assert!(crate::text_matches(
        selected.1.description(),
        "Activations: 1"
    ));
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
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 1280,
            height: 800,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(32);
    click(&mut renderer, "Material components", None, -600.0, None);
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
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width,
                height: 800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(32);
        click(&mut renderer, "Material components", None, -600.0, None);
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
        vec![Box::new(crate::TestPipelines)],
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
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 1650,
            height: 1200,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material components", None, -600.0, None);
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
    assert!(crate::text_matches(field.description(), "Required"));
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

fn tap_app_point(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
}

fn tap_app_bounds(renderer: &mut HeadlessRenderer, bounds: Rect) {
    tap_app_point(
        renderer,
        PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        ),
    );
}

fn node_is_exposed(nodes: &[(NodeId, Node)], mut id: NodeId) -> bool {
    for _ in 0..=nodes.len() {
        let Some((_, node)) = nodes.iter().find(|(candidate, _)| *candidate == id) else {
            return false;
        };
        if node.is_hidden() {
            return false;
        }
        if let Some((parent, _)) = nodes.iter().find(|(_, node)| node.children().contains(&id)) {
            id = *parent;
        } else {
            return true;
        }
    }
    panic!("accessibility ancestry must not contain a cycle");
}

fn app_mail_destination(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    let dialog = find(renderer, "Mail navigation", Some(Role::Dialog))
        .expect("actual root mail dialog")
        .0;
    let tree = renderer.inspect_accessibility().unwrap();
    tree.subtree(dialog)
        .find(|(id, node)| {
            node.role() == Role::Tab
                && crate::has_label(&node, label)
                && node_is_exposed(tree.nodes(), *id)
        })
        .map(|(id, node)| (id, node.clone()))
        .expect("actual destination inside the live dialog, not an instructional figure")
}

fn actual_app_drawer_article() -> (HeadlessRenderer, crate::app::AppNavigationHandles) {
    let navigation = tessera_ui::testing::TestProbe::default();
    let observed = navigation.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            crate::app::app_fixture(observed.clone());
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            frame_time: std::time::Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material components", None, -600.0, None);
    renderer.render_frames(60);
    click(
        &mut renderer,
        "Open Navigation drawer specifications",
        None,
        -600.0,
        None,
    );
    assert!(
        navigation
            .get()
            .expect("mounted app navigation")
            .directory
            .with(|directory| !directory.is_open() && !directory.is_animating())
    );
    (renderer, navigation.get().expect("mounted app navigation"))
}

fn launch_app_mail(renderer: &mut HeadlessRenderer) -> NodeId {
    let (opener, node) = reveal(
        renderer,
        "Open modal navigation drawer",
        Some(Role::Button),
        -600.0,
    );
    tap_app_bounds(renderer, node.bounds().unwrap());
    renderer.render_frames(60);
    let inbox = app_mail_destination(renderer, "Inbox");
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        inbox.0,
        "the live modal must initially focus the actual Inbox destination"
    );
    opener
}

fn assert_app_mail_closed(renderer: &HeadlessRenderer, opener: NodeId) {
    assert!(find(renderer, "Mail navigation", Some(Role::Dialog)).is_none());
    let tree = renderer.accessibility_snapshot().unwrap();
    assert!(node_is_exposed(&tree.nodes, opener));
    assert_eq!(
        tree.focus, opener,
        "native exit completion must restore the actual invoking launch button"
    );
}

fn initial_mail_focus_outline_pixels(renderer: &HeadlessRenderer, bounds: Rect) -> usize {
    let secondary = tessera_components::theme::MaterialTheme::default()
        .color_scheme
        .secondary;
    let expected = [secondary.r, secondary.g, secondary.b].map(|linear| {
        let encoded = if linear <= 0.003_130_8 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        (encoded * 255.0).round() as u8
    });
    let size = renderer.physical_size();
    let pixels = renderer.read_pixels().unwrap();
    let x0 = (bounds.x0 + bounds.width() * 0.25).ceil().max(0.0) as u32;
    let x1 = (bounds.x1 - bounds.width() * 0.25)
        .floor()
        .min(f64::from(size.width)) as u32;
    let y0 = (bounds.y0 - 5.0).floor().max(0.0) as u32;
    let y1 = (bounds.y0 - 2.0).ceil().min(f64::from(size.height)) as u32;
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let offset = ((y * size.width + x) * 4) as usize;
            (0..3).all(|channel| pixels[offset + channel].abs_diff(expected[channel]) <= 2)
                && pixels[offset + 3] == 255
        })
        .count()
}

#[test]
#[ignore = "requires a native GPU adapter for actual modal initial focus and painting"]
fn live_drawer_initial_focus_paints_the_first_destination_and_traverses_without_selection() {
    let (mut renderer, _navigation) = actual_app_drawer_article();
    let (opener, _) = reveal(
        &mut renderer,
        "Open modal navigation drawer",
        Some(Role::Button),
        -600.0,
    );
    for action in [Action::Focus, Action::Click] {
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: opener,
            data: None,
        }));
    }
    renderer.step_frame();
    let inbox = app_mail_destination(&renderer, "Inbox");
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        inbox.0,
        "opening must focus Inbox on the first layout without a later focus action"
    );
    renderer.render_frames(32);
    let inbox = app_mail_destination(&renderer, "Inbox");
    assert_eq!(inbox.1.is_selected(), Some(true));
    assert!(
        initial_mail_focus_outline_pixels(&renderer, inbox.1.bounds().unwrap()) >= 100,
        "the actual initially focused Inbox must paint the secondary outline outside its bounds"
    );
    for label in ["Outbox", "Favorites"] {
        assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
        renderer.step_frame();
        let destination = app_mail_destination(&renderer, label);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            destination.0
        );
        assert_eq!(destination.1.is_selected(), Some(false));
        assert_eq!(
            app_mail_destination(&renderer, "Inbox").1.is_selected(),
            Some(true)
        );
        assert!(find(&renderer, "Mail navigation", Some(Role::Dialog)).is_some());
        assert!(
            initial_mail_focus_outline_pixels(&renderer, destination.1.bounds().unwrap()) >= 100,
            "queued traversal must move the actual focus outline to {label} without activation"
        );
    }
    let favorites = app_mail_destination(&renderer, "Favorites").0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: favorites,
        data: None,
    }));
    renderer.render_frames(60);
    assert_app_mail_closed(&renderer, opener);
    assert!(
        find(
            &renderer,
            "Current destination: Favorites",
            Some(Role::Label)
        )
        .is_some()
    );
}

#[test]
#[ignore = "requires a native GPU adapter for the actual app shell and modal focus"]
fn live_drawer_covers_the_real_app_bar_blocks_background_and_restores_its_launch_button() {
    let (mut renderer, navigation) = actual_app_drawer_article();
    reveal(
        &mut renderer,
        "Open modal navigation drawer",
        Some(Role::Button),
        -600.0,
    );
    let selected_article = navigation.material.get();
    let (back_id, back) = find(&renderer, "Back to components", Some(Role::Button))
        .expect("real app bar navigation button");
    let before = renderer.read_pixels().unwrap();
    let opener = launch_app_mail(&mut renderer);
    let dialog = find(&renderer, "Mail navigation", Some(Role::Dialog))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert_eq!(dialog.x0, 0.0);
    assert_eq!(dialog.y0, 0.0);
    assert_eq!(dialog.height(), 871.0);
    assert_eq!(dialog.width(), 360.0);
    let tree = renderer.accessibility_snapshot().unwrap();
    assert!(
        !node_is_exposed(&tree.nodes, back_id),
        "the modal background must hide the real top bar, including through hidden ancestors"
    );
    let after = renderer.read_pixels().unwrap();
    let mut darkened = 0;
    for y in 16..24 {
        for x in 600..624 {
            let offset = (y * 907 + x) * 4;
            if (0..3).all(|channel| {
                u16::from(after[offset + channel]) + 8 <= u16::from(before[offset + channel])
            }) {
                darkened += 1;
            }
        }
    }
    assert!(
        darkened >= 160,
        "the native scrim must darken the blank top-bar area outside the sheet: {darkened}/192"
    );
    tap_app_bounds(&mut renderer, back.bounds().unwrap());
    renderer.render_frames(3);
    assert_eq!(navigation.material.get(), selected_article);
    assert!(!navigation.directory.with(|directory| directory.is_open()));
    assert!(find(&renderer, "Mail navigation", Some(Role::Dialog)).is_some());
    let outbox = app_mail_destination(&renderer, "Outbox");
    tap_app_bounds(&mut renderer, outbox.1.bounds().unwrap());
    renderer.render_frames(60);
    assert_app_mail_closed(&renderer, opener);
    assert!(find(&renderer, "Current destination: Outbox", Some(Role::Label)).is_some());
    let opener = launch_app_mail(&mut renderer);
    tap_app_point(&mut renderer, PxPosition::from_f32_xy(612.0, 20.0));
    renderer.render_frames(60);
    assert_app_mail_closed(&renderer, opener);
    assert_eq!(navigation.material.get(), selected_article);
    assert!(!navigation.directory.with(|directory| directory.is_open()));
    assert!(node_is_exposed(
        &renderer.accessibility_snapshot().unwrap().nodes,
        back_id
    ));
}

#[test]
#[ignore = "requires a native GPU adapter for actual article and router exit animations"]
fn live_drawer_route_exit_closes_without_unmounting_the_host_or_losing_session_state() {
    for leave_material_route in [false, true] {
        let (mut renderer, navigation) = actual_app_drawer_article();
        launch_app_mail(&mut renderer);
        let outbox = app_mail_destination(&renderer, "Outbox");
        tap_app_bounds(&mut renderer, outbox.1.bounds().unwrap());
        renderer.render_frames(60);
        let opener = launch_app_mail(&mut renderer);
        let (dialog_id, _) = find(&renderer, "Mail navigation", Some(Role::Dialog)).unwrap();
        let inbox_id = app_mail_destination(&renderer, "Inbox").0;
        if leave_material_route {
            navigation
                .router
                .with_mut(|router| router.replace(crate::pages::home::HomePageDestination {}));
        } else {
            navigation.material.set(None);
        }
        renderer.step_frame();
        assert_eq!(
            find(&renderer, "Mail navigation", Some(Role::Dialog))
                .expect("the root modal must stay mounted during its route-exit animation")
                .0,
            dialog_id
        );
        assert_eq!(app_mail_destination(&renderer, "Inbox").0, inbox_id);
        renderer.render_frames(60);
        let tree = renderer.accessibility_snapshot().unwrap();
        assert!(find(&renderer, "Mail navigation", Some(Role::Dialog)).is_none());
        assert!(!tree.nodes.iter().any(|(id, _)| *id == dialog_id));
        assert!(!tree.nodes.iter().any(|(id, _)| *id == opener));
        assert!(!tree.nodes.iter().any(|(id, _)| *id == inbox_id));
        assert_ne!(tree.focus, opener);
        assert_ne!(tree.focus, inbox_id);
        assert!(tree.nodes.iter().any(|(id, _)| *id == tree.focus));
        assert!(navigation.router.is_alive());
        assert!(navigation.material.is_alive());
        assert!(navigation.directory.is_alive());
        if leave_material_route {
            let menu = find(&renderer, "Open navigation drawer", Some(Role::Button))
                .expect("home route app-bar menu");
            tap_app_bounds(&mut renderer, menu.1.bounds().unwrap());
            renderer.render_frames(60);
            click(&mut renderer, "Material components", None, -600.0, None);
            renderer.render_frames(60);
        } else {
            click(
                &mut renderer,
                "Open Navigation drawer specifications",
                None,
                -600.0,
                None,
            );
        }
        reveal(
            &mut renderer,
            "Open modal navigation drawer",
            Some(Role::Button),
            -600.0,
        );
        assert!(find(&renderer, "Mail navigation", Some(Role::Dialog)).is_none());
        assert!(
            find(&renderer, "Current destination: Outbox", Some(Role::Label)).is_some(),
            "destination state must remain owned by the root session after article/router unmount"
        );
        let opener = launch_app_mail(&mut renderer);
        tap_app_point(&mut renderer, PxPosition::from_f32_xy(612.0, 20.0));
        renderer.render_frames(60);
        assert_app_mail_closed(&renderer, opener);
    }
}

fn actual_app_rail_article() -> (HeadlessRenderer, crate::app::AppNavigationHandles) {
    let navigation = tessera_ui::testing::TestProbe::default();
    let observed = navigation.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            crate::app::app_fixture(observed.clone());
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 907,
            height: 871,
            sample_count: 1,
            frame_time: std::time::Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    click(&mut renderer, "Material components", None, -600.0, None);
    renderer.render_frames(60);
    click(
        &mut renderer,
        "Open Navigation rail specifications",
        None,
        -600.0,
        None,
    );
    click(&mut renderer, "Guidelines", Some(Role::Tab), 800.0, None);
    click(&mut renderer, "Behavior", Some(Role::Link), -600.0, None);
    (renderer, navigation.get().expect("mounted app navigation"))
}

fn app_rail_destination(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    let dialog = find(renderer, "Mail rail navigation", Some(Role::Dialog))
        .expect("actual root rail dialog")
        .0;
    let tree = renderer.inspect_accessibility().unwrap();
    tree.subtree(dialog)
        .find(|(id, node)| {
            node.role() == Role::Tab
                && crate::has_label(&node, label)
                && node_is_exposed(tree.nodes(), *id)
        })
        .map(|(id, node)| (id, node.clone()))
        .expect("actual destination inside the live rail")
}

fn launch_app_rail(renderer: &mut HeadlessRenderer) -> NodeId {
    let (opener, _) = reveal(
        renderer,
        "Open modal navigation rail",
        Some(Role::Button),
        -600.0,
    );
    for action in [Action::Focus, Action::Click] {
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: opener,
            data: None,
        }));
    }
    renderer.step_frame();
    let close = find(renderer, "Close modal navigation rail", Some(Role::Button))
        .expect("native header is the first interactive control");
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        close.0,
        "initial focus must enter the actual rail header on the first layout"
    );
    renderer.render_frames(32);
    opener
}

fn assert_app_rail_closed(renderer: &HeadlessRenderer, opener: NodeId) {
    assert!(find(renderer, "Mail rail navigation", Some(Role::Dialog)).is_none());
    let tree = renderer.accessibility_snapshot().unwrap();
    assert!(node_is_exposed(&tree.nodes, opener));
    assert_eq!(
        tree.focus, opener,
        "exit must restore the actual invoking rail button"
    );
}

#[test]
#[ignore = "requires a native GPU adapter for actual app rail modality and initial focus"]
fn live_rail_covers_the_real_app_bar_and_initial_focus_traverses_without_selection() {
    let (mut renderer, navigation) = actual_app_rail_article();
    reveal(
        &mut renderer,
        "Open modal navigation rail",
        Some(Role::Button),
        -600.0,
    );
    let article = navigation.material.get();
    let (back_id, back) = find(&renderer, "Back to components", Some(Role::Button)).unwrap();
    let before = renderer.read_pixels().unwrap();
    let opener = launch_app_rail(&mut renderer);
    let dialog = find(&renderer, "Mail rail navigation", Some(Role::Dialog))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert_eq!(
        (dialog.x0, dialog.y0, dialog.width(), dialog.height()),
        (0.0, 0.0, 220.0, 871.0)
    );
    assert!(!node_is_exposed(
        &renderer.accessibility_snapshot().unwrap().nodes,
        back_id
    ));
    let after = renderer.read_pixels().unwrap();
    let mut darkened = 0;
    for y in 16..24 {
        for x in 600..624 {
            let offset = (y * 907 + x) * 4;
            if (0..3).all(|channel| {
                u16::from(after[offset + channel]) + 8 <= u16::from(before[offset + channel])
            }) {
                darkened += 1;
            }
        }
    }
    assert!(
        darkened >= 160,
        "the real rail scrim must cover the blank top bar: {darkened}/192"
    );
    for label in ["Home", "Inbox"] {
        assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
        renderer.step_frame();
        let destination = app_rail_destination(&renderer, label);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            destination.0
        );
        assert_eq!(
            app_rail_destination(&renderer, "Home").1.is_selected(),
            Some(true)
        );
        assert_eq!(
            app_rail_destination(&renderer, "Inbox").1.is_selected(),
            Some(false)
        );
        assert!(
            initial_mail_focus_outline_pixels(&renderer, destination.1.bounds().unwrap()) >= 100,
            "real focus must draw the native indicator for {label}"
        );
    }
    let inbox = app_rail_destination(&renderer, "Inbox").0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: inbox,
        data: None,
    }));
    renderer.render_frames(60);
    assert_app_rail_closed(&renderer, opener);
    let opener = launch_app_rail(&mut renderer);
    assert_eq!(
        app_rail_destination(&renderer, "Inbox").1.is_selected(),
        Some(true)
    );
    tap_app_bounds(&mut renderer, back.bounds().unwrap());
    renderer.render_frames(3);
    assert_eq!(navigation.material.get(), article);
    assert!(find(&renderer, "Mail rail navigation", Some(Role::Dialog)).is_some());
    tap_app_point(&mut renderer, PxPosition::from_f32_xy(612.0, 20.0));
    renderer.render_frames(60);
    assert_app_rail_closed(&renderer, opener);
    assert!(!navigation.directory.with(|directory| directory.is_open()));
}

#[test]
#[ignore = "requires a native GPU adapter for real rail route-exit focus cleanup"]
fn live_rail_route_exit_clears_modal_focus_and_preserves_selection_on_return() {
    for leave_material_route in [false, true] {
        let (mut renderer, navigation) = actual_app_rail_article();
        launch_app_rail(&mut renderer);
        let inbox = app_rail_destination(&renderer, "Inbox").0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: inbox,
            data: None,
        }));
        renderer.render_frames(8);
        let opener = launch_app_rail(&mut renderer);
        let dialog = find(&renderer, "Mail rail navigation", Some(Role::Dialog))
            .unwrap()
            .0;
        let home = app_rail_destination(&renderer, "Home").0;
        if leave_material_route {
            navigation
                .router
                .with_mut(|router| router.replace(crate::pages::home::HomePageDestination {}));
        } else {
            navigation.material.set(None);
        }
        renderer.step_frame();
        assert!(
            find(&renderer, "Mail rail navigation", Some(Role::Dialog)).is_none(),
            "the native modal rail closes immediately rather than running a side-sheet exit animation"
        );
        renderer.render_frames(4);
        let tree = renderer.accessibility_snapshot().unwrap();
        for stale in [dialog, opener, home] {
            assert!(!tree.nodes.iter().any(|(id, _)| *id == stale));
            assert_ne!(tree.focus, stale);
        }
        assert!(tree.nodes.iter().any(|(id, _)| *id == tree.focus));
        assert!(navigation.router.is_alive() && navigation.material.is_alive());
        if leave_material_route {
            let menu = find(&renderer, "Open navigation drawer", Some(Role::Button)).unwrap();
            tap_app_bounds(&mut renderer, menu.1.bounds().unwrap());
            renderer.render_frames(60);
            click(&mut renderer, "Material components", None, -600.0, None);
            renderer.render_frames(60);
        } else {
            click(
                &mut renderer,
                "Open Navigation rail specifications",
                None,
                -600.0,
                None,
            );
        }
        click(&mut renderer, "Guidelines", Some(Role::Tab), 800.0, None);
        click(&mut renderer, "Behavior", Some(Role::Link), -600.0, None);
        let opener = launch_app_rail(&mut renderer);
        assert_eq!(
            app_rail_destination(&renderer, "Inbox").1.is_selected(),
            Some(true),
            "the always-mounted root session must retain selection after article or router unmount"
        );
        let close = find(&renderer, "Close modal navigation rail", Some(Role::Button))
            .unwrap()
            .0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: close,
            data: None,
        }));
        renderer.render_frames(8);
        assert_app_rail_closed(&renderer, opener);
    }
}
