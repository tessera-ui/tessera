use std::cell::Cell;

use tessera_ui::{
    Dp, Modifier, State, remember, tessera,
    testing::{layout_test, pump_layout_test_session, start_layout_test_session},
};

use crate::{
    badge::badge_with_content,
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_bar::{NavigationBarIconPosition, navigation_bar, navigation_bar_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};

#[derive(Clone, PartialEq)]
struct Options {
    expanded: bool,
    scale: f64,
    label: String,
    activations: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            expanded: false,
            scale: 1.0,
            label: "Inbox".into(),
            activations: 0,
        }
    }
}

thread_local! { static OPTIONS: Cell<Option<State<Options>>> = const { Cell::new(None) }; }

fn tagged(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.into()),
        merge_descendants: true,
        ..Default::default()
    })
}

#[tessera]
fn rail_fixture() {
    let options = remember(Options::default);
    OPTIONS.set(Some(options));
    material_theme()
        .theme(move || {
            let scale = options.with(|options| options.scale);
            let mut theme = MaterialTheme::default();
            theme.typography.label_medium.font_size =
                theme.typography.label_medium.font_size * scale;
            theme.typography.label_medium.line_height = theme
                .typography
                .label_medium
                .line_height
                .map(|height| height * scale);
            theme.typography.label_large.font_size = theme.typography.label_large.font_size * scale;
            theme.typography.label_large.line_height = theme
                .typography
                .label_large
                .line_height
                .map(|height| height * scale);
            theme
        })
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    boxed()
                        .modifier(tagged("rail").height(Dp(300.0)))
                        .children(move || {
                            navigation_rail()
                                .expanded(options.with(|options| options.expanded))
                                .content(move || {
                                    navigation_rail_item()
                                        .label(options.with(|options| options.label.clone()))
                                        .selected(true)
                                        .accessibility_description("3 unread messages")
                                        .on_click(move || {
                                            options.with_mut(|options| options.activations += 1)
                                        })
                                        .icon(|| {
                                            spacer().modifier(
                                                tagged("rail-icon").size(Dp(24.0), Dp(24.0)),
                                            );
                                        })
                                        .badge(|| {
                                            badge_with_content()
                                                .modifier(tagged("rail-badge"))
                                                .content(|| {
                                                    text().content("3");
                                                });
                                        });
                                });
                        });
                });
        });
}

#[tessera]
fn bar_fixture() {
    let options = remember(Options::default);
    OPTIONS.set(Some(options));
    material_theme()
        .theme(move || {
            let scale = options.with(|options| options.scale);
            let mut theme = MaterialTheme::default();
            theme.typography.label_medium.font_size =
                theme.typography.label_medium.font_size * scale;
            theme.typography.label_medium.line_height = theme
                .typography
                .label_medium
                .line_height
                .map(|height| height * scale);
            theme
        })
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    boxed()
                        .modifier(tagged("bar").fill_max_width())
                        .children(move || {
                            navigation_bar()
                                .icon_position(if options.with(|options| options.expanded) {
                                    NavigationBarIconPosition::Start
                                } else {
                                    NavigationBarIconPosition::Top
                                })
                                .content(|| {
                                    for (index, label) in
                                        ["Home", "Inbox", "Saved"].into_iter().enumerate()
                                    {
                                        navigation_bar_item()
                                            .label(label)
                                            .selected(index == 0)
                                            .icon(move || {
                                                spacer().modifier(
                                                    tagged(
                                                        ["bar-icon0", "bar-icon1", "bar-icon2"]
                                                            [index],
                                                    )
                                                    .size(Dp(24.0), Dp(24.0)),
                                                );
                                            });
                                    }
                                });
                        });
                });
        });
}

#[test]
fn rail_badge_moves_from_icon_overlap_to_trailing_label_without_changing_target_width() {
    let mut session = start_layout_test_session(layout_test(rail_fixture).viewport_px(320, 400));
    let collapsed = pump_layout_test_session(&mut session, 0);
    collapsed.node("rail").size(96, 300);
    let icon = collapsed.node("rail-icon").bounds();
    let badge = collapsed.node("rail-badge").bounds();
    assert_eq!(badge.x.0, icon.x.0 + 12);
    assert_eq!(badge.y.0, icon.y.0 - 2);
    let options = OPTIONS.get().unwrap();
    options.with_mut(|options| options.expanded = true);
    pump_layout_test_session(&mut session, 16_666_667);
    let expanded = pump_layout_test_session(&mut session, 400_000_000);
    expanded.node("rail").size(220, 300);
    let icon = expanded.node("rail-icon").bounds();
    let badge = expanded.node("rail-badge").bounds();
    assert!(
        badge.x.0 > icon.x.0 + icon.width.0 + 8,
        "expanded badge must follow the label"
    );
    assert_eq!(icon.y.0 + icon.height.0 / 2, badge.y.0 + badge.height.0 / 2);
    assert!(badge.x.0 + badge.width.0 <= 220 - 36);
}

#[test]
fn navigation_bar_grows_for_twice_sized_labels_and_keeps_icons_inside_one_shared_height() {
    let mut session = start_layout_test_session(layout_test(bar_fixture).viewport_px(600, 400));
    let normal = pump_layout_test_session(&mut session, 0);
    normal.node("bar").size(600, 64);
    let options = OPTIONS.get().unwrap();
    options.with_mut(|options| options.scale = 2.0);
    let large = pump_layout_test_session(&mut session, 16_666_667);
    let bar = large.node("bar").bounds();
    assert!(
        bar.height.0 >= 80,
        "64dp fixed height clips twice-sized vertical labels: {bar:?}"
    );
    for key in ["bar-icon0", "bar-icon1", "bar-icon2"] {
        let icon = large.node(key).bounds();
        assert!(icon.y.0 >= bar.y.0 + 4);
        assert!(icon.y.0 + icon.height.0 < bar.y.0 + bar.height.0);
    }
    assert_eq!(
        large.node("bar-icon0").bounds().y,
        large.node("bar-icon2").bounds().y
    );
    options.with_mut(|options| options.expanded = true);
    let horizontal = pump_layout_test_session(&mut session, 33_333_334);
    assert!(horizontal.node("bar").bounds().height.0 >= 72);
}

#[test]
fn expanded_rail_wraps_large_labels_before_the_badge_and_grows_the_item() {
    let mut session = start_layout_test_session(layout_test(rail_fixture).viewport_px(320, 600));
    pump_layout_test_session(&mut session, 0);
    let options = OPTIONS.get().unwrap();
    options.with_mut(|options| {
        options.expanded = true;
        options.scale = 2.0;
        options.label = "Saved messages".into();
    });
    pump_layout_test_session(&mut session, 16_666_667);
    let large = pump_layout_test_session(&mut session, 400_000_000);
    let icon = large.node("rail-icon").bounds();
    let badge = large.node("rail-badge").bounds();
    assert!(badge.x.0 > icon.x.0 + icon.width.0 + 8);
    assert!(badge.x.0 + badge.width.0 <= 220 - 36);
    assert!(
        icon.y.0 > 44 + 16,
        "large label must grow and vertically center the item: {icon:?}"
    );
}

#[cfg(not(target_family = "wasm"))]
struct Pipelines;

#[cfg(not(target_family = "wasm"))]
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
}

#[cfg(not(target_family = "wasm"))]
#[test]
#[ignore = "requires a native GPU adapter"]
fn expanded_indicator_hugs_badged_content_while_the_full_target_activates_and_announces_count() {
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::Role,
    };

    let mut renderer = HeadlessRenderer::new(
        Box::new(rail_fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 320,
            height: 400,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(2);
    let options = OPTIONS.get().unwrap();
    options.with_mut(|options| options.expanded = true);
    renderer.render_frames(30);
    let snapshot = renderer
        .accessibility_snapshot()
        .expect("rail accessibility tree");
    let target = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Tab && node.label() == Some("Inbox"))
        .unwrap_or_else(|| {
            panic!("Inbox destination missing from accessibility tree: {snapshot:#?}")
        })
        .1
        .clone();
    assert_eq!(target.description(), Some("3 unread messages"));
    let bounds = target.bounds().unwrap();
    assert_eq!(bounds.width(), 220.0);
    let pixels = renderer.read_pixels().unwrap();
    let pixel = |x: usize, y: usize| &pixels[(y * 320 + x) * 4..(y * 320 + x) * 4 + 4];
    let y = ((bounds.y0 + bounds.y1) * 0.5) as usize;
    assert_ne!(
        pixel(25, y),
        pixel(199, y),
        "expanded indicator should hug contents instead of filling the target"
    );
    assert_eq!(pixel(199, y), pixel(219, y));
    let position = PxPosition::from_f32_xy((bounds.x1 - 2.0) as f32, y as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(2);
    assert_eq!(
        options.with(|options| options.activations),
        1,
        "space outside the visual indicator remains part of the destination target"
    );
}
