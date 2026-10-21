//! Button group sizing, selection geometry, and content context regressions.
//!
//! ## Usage
//!
//! Keep connected choices consistent across size presets and viewport changes.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Orientation, Role, Toggled, TreeId},
    layout::layout,
    remember, tessera,
};

use crate::{
    alignment::CrossAxisAlignment,
    button::ButtonShape,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};

use super::{
    ButtonGroupsSelectionMode, ButtonGroupsSize, ButtonGroupsState, ButtonGroupsStyle,
    button_groups,
};

const SIZES: [ButtonGroupsSize; 5] = [
    ButtonGroupsSize::ExtraSmall,
    ButtonGroupsSize::Small,
    ButtonGroupsSize::Medium,
    ButtonGroupsSize::Large,
    ButtonGroupsSize::ExtraLarge,
];

// Half coverage between red and white is 188 after sRGB encoding.
const HALF_COVERAGE: u8 = 188;

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    width: Dp,
    calls: usize,
    reverse_sizes: bool,
    multiple: bool,
    items: usize,
    disabled_middle: bool,
    disabled_all: bool,
    enabled: bool,
    required: bool,
    square: bool,
    connected: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static GROUPS: Cell<[Option<State<ButtonGroupsState>>; 5]> = const { Cell::new([None; 5]) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        width: Dp(800.0),
        calls: 0,
        reverse_sizes: false,
        multiple: false,
        items: 3,
        disabled_middle: false,
        disabled_all: false,
        enabled: true,
        required: false,
        square: false,
        connected: true,
    });
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface_container = Color::RED;
            theme.color_scheme.on_surface_variant = Color::RED;
            theme.color_scheme.primary = Color::BLUE;
            theme.color_scheme.on_primary = Color::BLUE;
            theme
        })
        .child(move || {
            column()
                .modifier(Modifier::new().background(Color::WHITE))
                .children(move || {
                    for (index, size) in SIZES.into_iter().enumerate() {
                        let controller = remember(ButtonGroupsState::new);
                        let mut controllers = GROUPS.get();
                        controllers[index] = Some(controller);
                        GROUPS.set(controllers);
                        let size = if settings.get().reverse_sizes {
                            SIZES[4 - index]
                        } else {
                            size
                        };
                        let mut group = button_groups()
                            .state(controller)
                            .style(if settings.get().connected {
                                ButtonGroupsStyle::Connected
                            } else {
                                ButtonGroupsStyle::Standard
                            })
                            .shape_style(if settings.get().square {
                                ButtonShape::Square
                            } else {
                                ButtonShape::Round
                            })
                            .size(size)
                            .enabled(settings.get().enabled)
                            .selection_required(settings.get().required)
                            .disabled_indices(if settings.get().disabled_all {
                                vec![0, 1, 2]
                            } else if settings.get().disabled_middle {
                                vec![1]
                            } else {
                                vec![]
                            })
                            .selection_mode(if settings.get().multiple {
                                ButtonGroupsSelectionMode::Multiple
                            } else {
                                ButtonGroupsSelectionMode::Single
                            })
                            .modifier(Modifier::new().width(settings.get().width).semantics(
                                SemanticsArgs {
                                    label: Some(format!("Group {index}")),
                                    description: Some("Related choices".to_string()),
                                    ..Default::default()
                                },
                            ));
                        for item in 0..settings.get().items {
                            group = group.child(
                                move || {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            layout()
                                                .modifier(Modifier::new().semantics(
                                                    SemanticsArgs {
                                                        test_tag: Some(format!(
                                                            "icon-{index}-{item}"
                                                        )),
                                                        ..Default::default()
                                                    },
                                                ))
                                                .child(|| {
                                                    icon().painter(filled::HOME_SVG);
                                                });
                                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                                            text().content(format!("G{index} {item}"));
                                        });
                                },
                                move |selected| {
                                    assert_eq!(
                                        controller.with(|value| value.is_selected(item)),
                                        selected,
                                        "callbacks must observe the updated selection"
                                    );
                                    settings.with_mut(|value| value.calls += 1);
                                },
                            );
                        }
                        drop(group);
                        spacer().modifier(Modifier::new().height(Dp(12.0)));
                    }
                });
        });
}

fn rect(renderer: &HeadlessRenderer, key: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"].as_str() == Some(key)
            || node["accessibility"]["label"].as_str() == Some(key)
        {
            return ["x", "y", "width", "height"]
                .map(|field| node["bounds"][field].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {key}");
}

fn button(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let mut id = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap()
        .0;
    loop {
        let item = snapshot
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .unwrap();
        if item.1.role() == Role::CheckBox {
            return item.clone();
        }
        id = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.children().contains(&id))
            .unwrap()
            .0;
    }
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    assert!(action(renderer, label, Action::Click));
    renderer.render_frames(45);
}

fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) -> bool {
    let target_node = button(renderer, label).0;
    renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    })
}

fn pixel(renderer: &HeadlessRenderer, x: i64, y: i64) -> [u8; 4] {
    let offset = ((y as u32 * renderer.physical_size().width + x as u32) * 4) as usize;
    renderer.read_pixels().unwrap()[offset..offset + 4]
        .try_into()
        .unwrap()
}

fn check_group_metrics(samples: u32) {
    for scale in [1.0, 1.5] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 1800,
                height: 1200,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(10);
        let settings = SETTINGS.get().unwrap();
        for reverse in [false, true, false] {
            settings.with_mut(|value| value.reverse_sizes = reverse);
            renderer.render_frames(45);
            for index in 0..5 {
                let preset = if reverse { 4 - index } else { index };
                let height = [32.0_f64, 40.0, 56.0, 96.0, 136.0][preset];
                let icon = [20.0, 20.0, 24.0, 32.0, 40.0][preset];
                let line = [20.0, 20.0, 24.0, 32.0, 40.0][preset];
                let mut bounds = Vec::new();
                for item in 0..settings.get().items {
                    let node = button(&renderer, &format!("G{index} {item}")).1;
                    let b = node.bounds().unwrap();
                    assert_eq!(
                        b.y1 - b.y0,
                        height.max(48.0) * scale,
                        "touch height {preset}"
                    );
                    assert_eq!(
                        rect(&renderer, &format!("icon-{index}-{item}"))[2],
                        (icon * scale) as i64
                    );
                    assert_eq!(
                        rect(&renderer, &format!("G{index} {item}"))[3],
                        (line * scale) as i64
                    );
                    assert_eq!(node.toggled(), Some(Toggled::False));
                    bounds.push(b);
                }
                assert_eq!(bounds[2].x1, 800.0 * scale);
                assert_eq!(bounds[1].x0 - bounds[0].x1, 2.0 * scale);
                assert_eq!(bounds[2].x0 - bounds[1].x1, 2.0 * scale);
                assert!(
                    ((bounds[0].x1 - bounds[0].x0) - (bounds[2].x1 - bounds[2].x0)).abs() <= 1.0
                );
                let first = bounds[0];
                let y = first.y0 as i64 + ((height.max(48.0) - height) * scale / 2.0) as i64;
                let pixels = renderer.read_pixels().unwrap();
                let x = ((first.x0 + first.x1) / 2.0) as usize;
                let rows: Vec<_> = (first.y0 as usize..first.y1 as usize)
                    .filter(|row| {
                        let offset = (row * renderer.physical_size().width as usize + x) * 4;
                        pixels[offset + 1] < HALF_COVERAGE
                    })
                    .collect();
                assert_eq!(
                    rows.len(),
                    (height * scale) as usize,
                    "visual height {preset}"
                );
                assert_eq!(rows[0], y as usize, "visual vertical alignment {preset}");
                assert_eq!(
                    pixel(&renderer, (first.x0 + first.x1) as i64 / 2, y + 1),
                    [255, 0, 0, 255],
                    "visual top {preset}"
                );
                assert_eq!(
                    pixel(
                        &renderer,
                        (first.x0 + first.x1) as i64 / 2,
                        y + (height * scale) as i64 - 2
                    ),
                    [255, 0, 0, 255],
                    "visual bottom {preset}"
                );
                let middle = bounds[1];
                let radius = [8.0, 8.0, 8.0, 16.0, 20.0][preset];
                let corner_y =
                    middle.y0 as i64 + ((height.max(48.0) - height) * scale / 2.0) as i64;
                let inside = pixel(
                    &renderer,
                    middle.x0 as i64 + (0.375 * radius * scale) as i64,
                    corner_y + (0.25 * radius * scale) as i64,
                );
                let outside = pixel(
                    &renderer,
                    middle.x0 as i64 + (0.125 * radius * scale) as i64,
                    corner_y + (0.25 * radius * scale) as i64,
                );
                assert!(
                    inside[1] < HALF_COVERAGE,
                    "{radius}dp circular inner corner must include the interior probe: {inside:?}"
                );
                assert!(
                    outside[1] > HALF_COVERAGE,
                    "inner corner curvature must exclude the exterior probe: {outside:?}"
                );
            }
        }
        let before = button(&renderer, "G1 0");
        click(&mut renderer, "G1 0");
        let after = button(&renderer, "G1 0");
        assert_eq!(after.0, before.0, "selection retains the button identity");
        assert_eq!(
            after.1.bounds(),
            before.1.bounds(),
            "connected selection must not expand a button"
        );
        assert_eq!(after.1.toggled(), Some(Toggled::True));
        let b = after.1.bounds().unwrap();
        assert_eq!(
            pixel(
                &renderer,
                ((b.x0 + b.x1) / 2.0) as i64,
                ((b.y0 + b.y1) / 2.0) as i64
            ),
            [0, 0, 255, 255]
        );
        click(&mut renderer, "G1 0");
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::False));
        click(&mut renderer, "G1 0");
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::True));
        click(&mut renderer, "G1 1");
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::False));
        assert_eq!(button(&renderer, "G1 1").1.toggled(), Some(Toggled::True));
        settings.with_mut(|value| value.multiple = true);
        renderer.render_frames(5);
        click(&mut renderer, "G1 0");
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::True));
        assert_eq!(button(&renderer, "G1 1").1.toggled(), Some(Toggled::True));
        settings.with_mut(|value| value.multiple = false);
        renderer.render_frames(5);
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::True));
        assert_eq!(button(&renderer, "G1 1").1.toggled(), Some(Toggled::False));
        click(&mut renderer, "G1 0");
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::False));
        for width in [181.0, 0.0, 800.0] {
            settings.with_mut(|value| value.width = Dp(width));
            renderer.render_frames(8);
            for index in 0..5 {
                let first = button(&renderer, &format!("G{index} 0"))
                    .1
                    .bounds()
                    .unwrap();
                let last = button(&renderer, &format!("G{index} 2"))
                    .1
                    .bounds()
                    .unwrap();
                assert_eq!(last.x1, (width * scale).floor());
                assert_eq!(first.y0, last.y0);
                assert_eq!(first.y1, last.y1);
            }
        }
        click(&mut renderer, "G1 2");
        settings.with_mut(|value| value.items = 2);
        renderer.render_frames(5);
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("G1 2"))
        );
        settings.with_mut(|value| value.items = 3);
        renderer.render_frames(5);
        assert_eq!(
            button(&renderer, "G1 2").1.toggled(),
            Some(Toggled::False),
            "removed choices must not restore stale selection"
        );
        assert_eq!(settings.get().calls, 7);
        let controller = GROUPS.get()[1].unwrap();
        controller.with_mut(|value| value.set_selected(1, true));
        renderer.render_frames(45);
        assert_eq!(button(&renderer, "G1 1").1.toggled(), Some(Toggled::True));
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::False));
        controller.with_mut(ButtonGroupsState::clear);
        renderer.render_frames(45);
        assert_eq!(button(&renderer, "G1 1").1.toggled(), Some(Toggled::False));
        assert_eq!(
            settings.get().calls,
            7,
            "programmatic selection is not a click"
        );
        for index in 0..5 {
            let label = format!("G{index} 1");
            let target = button(&renderer, &label);
            let bounds = target.1.bounds().unwrap();
            let point = PxPosition::from_f32_xy(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            );
            let radius = [8.0, 8.0, 8.0, 16.0, 20.0][index];
            let padding = [8.0, 4.0, 0.0, 0.0, 0.0][index];
            let corner_x = bounds.x0 as i64 + (0.2 * radius * scale) as i64;
            let corner_y = bounds.y0 as i64 + ((padding + 0.3 * radius) * scale) as i64;
            assert!(pixel(&renderer, corner_x, corner_y)[1] > HALF_COVERAGE);
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(30);
            assert!(
                pixel(&renderer, corner_x, corner_y)[1] < HALF_COVERAGE,
                "press must reduce the inner corner for preset {index}"
            );
            assert_eq!(button(&renderer, &label).1.toggled(), Some(Toggled::False));
            assert_eq!(
                settings.get().calls,
                7 + index,
                "press does not activate selection"
            );
            renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(45);
            let selected = button(&renderer, &label);
            assert_eq!(selected.0, target.0);
            assert_eq!(selected.1.bounds(), Some(bounds));
            assert_eq!(selected.1.toggled(), Some(Toggled::True));
            assert_eq!(settings.get().calls, 8 + index);
        }
        let group = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some("Group 1"))
            .unwrap()
            .1;
        assert_eq!(group.role(), Role::Group);
        assert_eq!(group.description(), Some("Related choices"));
        assert_eq!(group.orientation(), Some(Orientation::Horizontal));
        assert_eq!(
            group.children(),
            &(0..3)
                .map(|item| button(&renderer, &format!("G1 {item}")).0)
                .collect::<Vec<_>>()
        );
        settings.with_mut(|value| value.disabled_middle = true);
        renderer.render_frames(8);
        let disabled = button(&renderer, "G1 1").1;
        assert!(disabled.is_disabled());
        assert_eq!(disabled.toggled(), Some(Toggled::True));
        assert!(!disabled.supports_action(Action::Click));
        assert!(!disabled.supports_action(Action::Focus));
        assert!(!action(&mut renderer, "G1 1", Action::Click));
        let bounds = disabled.bounds().unwrap();
        let point = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(45);
        assert_eq!(settings.get().calls, 12);
        assert!(controller.with(|state| state.is_selected(1)));
        assert!(action(&mut renderer, "G1 0", Action::Focus));
        renderer.render_frames(5);
        FocusManager::current().request_move_focus(FocusDirection::Right);
        renderer.render_frames(5);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            button(&renderer, "G1 2").0
        );
        assert_eq!(
            settings.get().calls,
            12,
            "focus movement must not select a choice"
        );
        assert!(action(&mut renderer, "G1 0", Action::Focus));
        renderer.render_frames(5);
        FocusManager::current().request_move_focus(FocusDirection::Next);
        renderer.render_frames(5);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            button(&renderer, "G1 2").0
        );
        settings.with_mut(|value| value.required = true);
        controller.with_mut(ButtonGroupsState::clear);
        renderer.render_frames(45);
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::True));
        click(&mut renderer, "G1 0");
        assert_eq!(
            settings.get().calls,
            12,
            "the last required selection is retained"
        );
        settings.with_mut(|value| value.multiple = true);
        renderer.render_frames(5);
        click(&mut renderer, "G1 2");
        click(&mut renderer, "G1 0");
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::False));
        assert_eq!(button(&renderer, "G1 2").1.toggled(), Some(Toggled::True));
        click(&mut renderer, "G1 2");
        assert_eq!(settings.get().calls, 14);
        settings.with_mut(|value| value.required = false);
        renderer.render_frames(5);
        click(&mut renderer, "G1 2");
        assert_eq!(button(&renderer, "G1 2").1.toggled(), Some(Toggled::False));
        assert_eq!(settings.get().calls, 15);
        settings.with_mut(|value| {
            value.required = true;
            value.disabled_all = true;
        });
        renderer.render_frames(8);
        for item in 0..3 {
            assert!(button(&renderer, &format!("G1 {item}")).1.is_disabled());
            assert!(!controller.with(|state| state.is_selected(item)));
        }
        settings.with_mut(|value| {
            value.disabled_all = false;
            value.disabled_middle = false;
            value.enabled = false;
        });
        renderer.render_frames(8);
        assert!(button(&renderer, "G1 0").1.is_disabled());
        assert!(!action(&mut renderer, "G1 0", Action::Click));
        settings.with_mut(|value| value.enabled = true);
        renderer.render_frames(45);
        assert!(!button(&renderer, "G1 0").1.is_disabled());
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::True));
        assert_eq!(settings.get().calls, 15);
        let before: Vec<_> = (0..5)
            .map(|index| button(&renderer, &format!("G{index} 0")))
            .collect();
        settings.with_mut(|value| {
            value.required = false;
            value.square = true;
        });
        for controller in GROUPS.get().into_iter().flatten() {
            controller.with_mut(ButtonGroupsState::clear);
        }
        renderer.render_frames(45);
        for (index, before) in before.iter().enumerate() {
            let label = format!("G{index} 0");
            let square = button(&renderer, &label);
            let bounds = square.1.bounds().unwrap();
            assert_eq!(square.0, before.0);
            assert_eq!(square.1.bounds(), before.1.bounds());
            assert_eq!(square.1.toggled(), Some(Toggled::False));
            let radius = [8.0, 8.0, 8.0, 16.0, 20.0][index];
            let padding = [8.0, 4.0, 0.0, 0.0, 0.0][index];
            let x = bounds.x0 as i64 + (0.375 * radius * scale) as i64;
            let y = bounds.y0 as i64 + ((padding + 0.25 * radius) * scale) as i64;
            assert!(
                pixel(&renderer, x, y)[1] < HALF_COVERAGE,
                "square outer corner {index}"
            );
            click(&mut renderer, &label);
            let selected = button(&renderer, &label);
            assert_eq!(selected.0, before.0);
            assert_eq!(selected.1.bounds(), before.1.bounds());
            assert_eq!(selected.1.toggled(), Some(Toggled::True));
            assert!(
                pixel(&renderer, x, y)[1] > HALF_COVERAGE,
                "selected full outer corner {index}"
            );
        }
        assert_eq!(settings.get().calls, 20);
        settings.with_mut(|value| value.square = false);
        renderer.render_frames(45);
        for (index, before) in before.iter().enumerate() {
            let selected = button(&renderer, &format!("G{index} 0"));
            assert_eq!(selected.0, before.0);
            assert_eq!(selected.1.bounds(), before.1.bounds());
            assert_eq!(selected.1.toggled(), Some(Toggled::True));
        }
        for controller in GROUPS.get().into_iter().flatten() {
            controller.with_mut(ButtonGroupsState::clear);
        }
        settings.with_mut(|value| value.square = true);
        renderer.render_frames(45);
        let first = button(&renderer, "G1 0");
        let bounds = first.1.bounds().unwrap();
        let center = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        let outer_x = bounds.x0 as i64 + (3.0 * scale) as i64;
        let outer_y = bounds.y0 as i64 + (6.0 * scale) as i64;
        let inner_x = bounds.x1 as i64 - 1 - (1.6 * scale) as i64;
        let inner_y = bounds.y0 as i64 + (6.4 * scale) as i64;
        assert!(pixel(&renderer, outer_x, outer_y)[1] < HALF_COVERAGE);
        assert!(pixel(&renderer, inner_x, inner_y)[1] > HALF_COVERAGE);
        renderer.pointer_move(MOUSE_POINTER_ID, center);
        renderer.pointer_press(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
        renderer.render_frames(30);
        assert!(
            pixel(&renderer, outer_x, outer_y)[1] < HALF_COVERAGE,
            "square outer corner remains square while pressed"
        );
        assert!(
            pixel(&renderer, inner_x, inner_y)[1] < HALF_COVERAGE,
            "square inner corner contracts while pressed"
        );
        assert_eq!(settings.get().calls, 20);
        renderer.pointer_release(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
        renderer.render_frames(45);
        assert_eq!(button(&renderer, "G1 0").0, first.0);
        assert_eq!(button(&renderer, "G1 0").1.bounds(), first.1.bounds());
        assert_eq!(button(&renderer, "G1 0").1.toggled(), Some(Toggled::True));
        assert!(pixel(&renderer, outer_x, outer_y)[1] > HALF_COVERAGE);
        assert_eq!(settings.get().calls, 21);
        let style_before: Vec<_> = (0..5)
            .map(|index| button(&renderer, &format!("G{index} 0")).0)
            .collect();
        let focus_before = renderer.accessibility_snapshot().unwrap().focus;
        for controller in GROUPS.get().into_iter().flatten() {
            controller.with_mut(ButtonGroupsState::clear);
        }
        settings.with_mut(|value| value.connected = false);
        renderer.render_frames(45);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            focus_before,
            "style changes must preserve focus"
        );
        for index in 0..5 {
            let label = format!("G{index} 0");
            let before = button(&renderer, &label);
            assert_eq!(
                before.0, style_before[index],
                "style changes must retain the item identity"
            );
            let bounds = before.1.bounds().unwrap();
            let radius = [12.0, 12.0, 16.0, 28.0, 28.0][index];
            let padding = [8.0, 4.0, 0.0, 0.0, 0.0][index];
            let dx = (0.375 * radius * scale) as i64;
            let dy = ((padding + 0.25 * radius) * scale) as i64;
            assert!(
                pixel(&renderer, bounds.x0 as i64 + dx, bounds.y0 as i64 + dy)[1] < HALF_COVERAGE,
                "standard square resting corner {index}"
            );
            click(&mut renderer, &label);
            let selected = button(&renderer, &label);
            let bounds = selected.1.bounds().unwrap();
            assert_eq!(selected.0, before.0);
            assert_eq!(selected.1.toggled(), Some(Toggled::True));
            assert!(
                pixel(&renderer, bounds.x0 as i64 + dx, bounds.y0 as i64 + dy)[1] > HALF_COVERAGE,
                "standard square selection becomes round {index}"
            );
        }
        assert_eq!(settings.get().calls, 26);
        let before: Vec<_> = (0..5)
            .map(|index| button(&renderer, &format!("G{index} 0")))
            .collect();
        settings.with_mut(|value| value.square = false);
        renderer.render_frames(45);
        for (index, before) in before.iter().enumerate() {
            let selected = button(&renderer, &format!("G{index} 0"));
            assert_eq!(selected.0, before.0);
            assert_eq!(selected.1.toggled(), Some(Toggled::True));
            let bounds = selected.1.bounds().unwrap();
            let radius = [12.0, 12.0, 16.0, 28.0, 28.0][index];
            let padding = [8.0, 4.0, 0.0, 0.0, 0.0][index];
            assert!(
                pixel(
                    &renderer,
                    bounds.x0 as i64 + (0.375 * radius * scale) as i64,
                    bounds.y0 as i64 + ((padding + 0.25 * radius) * scale) as i64
                )[1] < HALF_COVERAGE,
                "standard round selection becomes square {index}"
            );
        }
        assert_eq!(
            settings.get().calls,
            26,
            "shape changes do not activate items"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_connected_group_sizes_context_selection_and_resize_msaa1() {
    crate::test_utils::run_isolated_test(
        "button_groups::native_tests::native_connected_group_sizes_context_selection_and_resize_msaa1",
        || check_group_metrics(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn native_connected_group_sizes_context_selection_and_resize_msaa4() {
    crate::test_utils::run_isolated_test(
        "button_groups::native_tests::native_connected_group_sizes_context_selection_and_resize_msaa4",
        || check_group_metrics(4),
    );
}
