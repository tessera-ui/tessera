use std::time::Duration;

use tessera_ui::{
    CursorEventContent, FocusDirection, GestureState, MOUSE_POINTER_ID, PointerChange,
    PressKeyEventType, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource, accesskit::Role,
    focus, remember, testing::UiTest,
};

use crate::{
    button::button,
    modifier::ModifierExt as _,
    scaffold::scaffold,
    scrollable::ScrollableController,
    theme::{MaterialTheme, material_theme},
};

use super::*;

#[derive(Clone, Copy)]
struct Settings {
    behavior: AppBarScrollBehavior,
    enabled: bool,
    snap: bool,
    variant: TopAppBarVariant,
}

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
    bar: Option<State<AppBarScrollController>>,
    content: Option<State<ScrollableController>>,
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let settings = remember(|| Settings {
        behavior: AppBarScrollBehavior::ExitUntilCollapsed,
        enabled: true,
        snap: true,
        variant: TopAppBarVariant::LargeFlexible,
    });
    let state = remember(AppBarScrollController::default);
    let content = remember(ScrollableController::new);
    observed.with_mut(|handles| handles.settings = Some(settings));
    observed.with_mut(|handles| handles.bar = Some(state));
    observed.with_mut(|handles| handles.content = Some(content));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface = Color::WHITE;
            theme.color_scheme.surface_container = Color::RED;
            theme
        })
        .child(move || {
            app_bar_scroll_provider()
                .controller(state)
                .behavior(settings.get().behavior)
                .enabled(settings.get().enabled)
                .snap(settings.get().snap)
                .scroll_controller(content)
                .child(move || {
                    scaffold()
                        .modifier(Modifier::new().fill_max_size())
                        .top_bar(move || {
                            top_app_bar()
                                .scroll_controller(state)
                                .title("Library")
                                .subtitle("Saved items")
                                .variant(settings.get().variant)
                                .navigation_icon(|| {
                                    icon_button()
                                        .standard()
                                        .icon(filled::ARROW_BACK_SVG)
                                        .accessibility_label("Back")
                                        .on_click(|| {});
                                });
                        })
                        .content(move || {
                            column()
                                .modifier(Modifier::new().fill_max_size().vertical_scroll(content))
                                .children(|| {
                                    for index in 0..40 {
                                        surface()
                                            .modifier(
                                                Modifier::new().fill_max_width().height(Dp(48.0)),
                                            )
                                            .style(Color::WHITE.into())
                                            .child(move || {
                                                if index == 5 {
                                                    button()
                                                        .accessibility_label("Content action")
                                                        .on_click(|| {})
                                                        .child(|| {
                                                            text("Content action");
                                                        });
                                                } else {
                                                    text(format!("Item {index}"));
                                                }
                                            });
                                    }
                                });
                        });
                });
        });
}

fn height(renderer: &UiTest) -> f64 {
    let bounds = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Toolbar)
        .expect("toolbar")
        .1
        .bounds()
        .expect("bounds");
    bounds.y1 - bounds.y0
}

fn labels(renderer: &UiTest) -> usize {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.role() == Role::Label && node.label() == Some("Library"))
        .count()
}

fn focused_label(renderer: &UiTest) -> Option<String> {
    let tree = renderer.accessibility_snapshot().expect("tree");
    tree.nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .and_then(|(_, node)| node.label().map(str::to_owned))
}

fn background(renderer: &UiTest, color: [u8; 3]) {
    let pixels = renderer.read_pixels();
    let index = (5 * renderer.physical_size().width as usize + 500) * 4;
    assert_eq!(&pixels[index..index + 3], &color);
}

fn wheel(renderer: &mut UiTest, delta: f32) {
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(300.0, 500.0),
        0.0,
        delta,
    );
    renderer.frames(3);
}

fn reset(
    renderer: &mut UiTest,
    behavior: AppBarScrollBehavior,
    snap: bool,
    observed: &tessera_ui::testing::TestProbe<FixtureHandles>,
) {
    observed
        .get()
        .settings
        .expect("settings")
        .with_mut(|settings| {
            settings.behavior = behavior;
            settings.snap = snap;
            settings.variant = TopAppBarVariant::LargeFlexible;
            settings.enabled = true;
        });
    observed
        .get()
        .bar
        .expect("bar")
        .with_mut(AppBarScrollController::reset);
    observed
        .get()
        .content
        .expect("content")
        .with_mut(|controller| controller.set_scroll_position(PxPosition::ZERO));
    renderer.frames(8);
}

fn touch(renderer: &mut UiTest, content: CursorEventContent, state: GestureState) {
    let timestamp =
        renderer.frame_origin() + renderer.frame_time() + Duration::from_nanos(16_666_667);
    renderer.inject_pointer(
        PointerChange::new(timestamp, MOUSE_POINTER_ID, content, state)
            .with_position(PxPosition::from_f32_xy(300.0, 500.0)),
    );
    renderer.frames(1);
}

fn check_scrolling() {
    let observed = tessera_ui::testing::TestProbe::default();
    let fixture_observed = observed.clone();
    let mut renderer = UiTest::new()
        .viewport(600, 900)
        .scale_factor(1.5)
        .rendered(crate::TesseraComponents)
        .content(move || fixture(fixture_observed.clone()));
    renderer.frames(8);
    let bar = observed.get().bar.expect("bar");
    let content = observed.get().content.expect("content");
    assert_eq!(height(&renderer), 228.0);
    assert_eq!(labels(&renderer), 1);
    let baseline = renderer.read_pixels();
    wheel(&mut renderer, -30.0);
    assert!((height(&renderer) - 198.0).abs() <= 1.0);
    assert_eq!(content.with(ScrollableController::target_position).y, 0.0);
    assert_eq!(labels(&renderer), 1);
    renderer.frames(90);
    assert_eq!(height(&renderer), 228.0);
    assert!(!bar.with(AppBarScrollController::is_animating));
    assert!(
        renderer.read_pixels() == baseline,
        "a short wheel gesture snaps back to the original pixels"
    );

    wheel(&mut renderer, -180.0);
    renderer.frames(90);
    assert_eq!(height(&renderer), 96.0);
    assert_eq!(labels(&renderer), 1);
    assert!(content.with(ScrollableController::target_position).y <= -47.0);
    background(&renderer, [255, 0, 0]);
    wheel(&mut renderer, 24.0);
    assert_eq!(
        height(&renderer),
        96.0,
        "reverse scrolling inside the content retains the compact header"
    );
    wheel(&mut renderer, 1000.0);
    renderer.frames(90);
    assert_eq!(height(&renderer), 228.0);
    background(&renderer, [255, 255, 255]);

    reset(
        &mut renderer,
        AppBarScrollBehavior::EnterAlways,
        false,
        &observed,
    );
    wheel(&mut renderer, -300.0);
    renderer.frames(40);
    let child = content.with(ScrollableController::target_position);
    assert!(child.y < 0.0);
    wheel(&mut renderer, 24.0);
    assert!((height(&renderer) - 120.0).abs() <= 1.0);
    assert_eq!(
        content.with(ScrollableController::target_position),
        child,
        "the header expands before reverse scrolling reaches the content"
    );
    assert_eq!(labels(&renderer), 1);

    reset(&mut renderer, AppBarScrollBehavior::Pinned, true, &observed);
    wheel(&mut renderer, -60.0);
    renderer.frames(90);
    assert_eq!(height(&renderer), 228.0);
    background(&renderer, [255, 0, 0]);
    content.with_mut(|controller| controller.set_scroll_position(PxPosition::ZERO));
    renderer.frames(90);
    background(&renderer, [255, 255, 255]);
    assert!(!bar.with(AppBarScrollController::is_animating));

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        false,
        &observed,
    );
    observed
        .get()
        .settings
        .expect("settings")
        .with_mut(|settings| settings.enabled = false);
    renderer.frames(3);
    wheel(&mut renderer, -50.0);
    assert_eq!(height(&renderer), 228.0);
    assert!(content.with(ScrollableController::target_position).y < 0.0);

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        true,
        &observed,
    );
    touch(
        &mut renderer,
        CursorEventContent::Pressed(PressKeyEventType::Left),
        GestureState::TapCandidate,
    );
    touch(
        &mut renderer,
        CursorEventContent::Scroll(ScrollEventContent {
            delta_x: 0.0,
            delta_y: -30.0,
            unit: ScrollDeltaUnit::Pixel,
            source: ScrollEventSource::Touch,
        }),
        GestureState::Dragged,
    );
    renderer.frames(30);
    assert_eq!(
        height(&renderer),
        198.0,
        "a held touch does not trigger the wheel idle snap"
    );
    touch(
        &mut renderer,
        CursorEventContent::Released(PressKeyEventType::Left),
        GestureState::Dragged,
    );
    renderer.frames(90);
    assert_eq!(
        height(&renderer),
        228.0,
        "releasing without velocity still settles the nested header"
    );

    renderer.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(300.0, 200.0),
        PressKeyEventType::Left,
    );
    renderer.frames(1);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(300.0, 20.0));
    renderer.frames(1);
    renderer.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(300.0, 20.0),
        PressKeyEventType::Left,
    );
    renderer.frames(90);
    assert_eq!(
        height(&renderer),
        96.0,
        "dragging the bar itself updates its shared collapse state"
    );

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        false,
        &observed,
    );
    touch(
        &mut renderer,
        CursorEventContent::Pressed(PressKeyEventType::Left),
        GestureState::TapCandidate,
    );
    for _ in 0..3 {
        touch(
            &mut renderer,
            CursorEventContent::Scroll(ScrollEventContent {
                delta_x: 0.0,
                delta_y: -20.0,
                unit: ScrollDeltaUnit::Pixel,
                source: ScrollEventSource::Touch,
            }),
            GestureState::Dragged,
        );
    }
    let before_release = bar.with(AppBarScrollController::collapsed_fraction);
    assert!(before_release > 0.0 && before_release < 1.0);
    assert_eq!(content.with(ScrollableController::target_position).y, 0.0);
    touch(
        &mut renderer,
        CursorEventContent::Released(PressKeyEventType::Left),
        GestureState::Dragged,
    );
    renderer.frames(8);
    assert_eq!(
        bar.with(AppBarScrollController::collapsed_fraction),
        1.0,
        "touch velocity consumed by the header continues through nested inertia"
    );
    let after_fling = content.with(ScrollableController::target_position).y;
    assert!(
        after_fling < 0.0,
        "remaining fling movement reaches the content"
    );
    renderer.frames(8);
    assert!(
        content.with(ScrollableController::target_position).y < after_fling,
        "fractional inertia steps continue after header collapse"
    );
    renderer.frames(180);
    assert!(!bar.with(AppBarScrollController::is_animating));

    reset(
        &mut renderer,
        AppBarScrollBehavior::EnterAlways,
        false,
        &observed,
    );
    observed
        .get()
        .settings
        .expect("settings")
        .with_mut(|settings| settings.variant = TopAppBarVariant::Small);
    renderer.frames(5);
    assert_eq!(height(&renderer), 96.0);
    wheel(&mut renderer, -150.0);
    assert_eq!(height(&renderer), 0.0);
    assert!(
        !renderer
            .accessibility_snapshot()
            .expect("tree")
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Button && node.label() == Some("Back")),
        "a hidden small header does not publish invisible navigation controls"
    );
    focus::clear_focus(true);
    focus::request_move_focus(FocusDirection::Next);
    renderer.frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Content action"));
    assert_eq!(height(&renderer), 0.0);
    focus::request_move_focus(FocusDirection::Previous);
    renderer.frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Content action"));
    assert_eq!(height(&renderer), 0.0);
    wheel(&mut renderer, 150.0);
    assert_eq!(height(&renderer), 96.0);
    assert_eq!(labels(&renderer), 1);
    focus::clear_focus(true);
    focus::request_move_focus(FocusDirection::Next);
    renderer.frames(8);
    let first = focused_label(&renderer);
    focus::request_move_focus(FocusDirection::Next);
    renderer.frames(8);
    let second = focused_label(&renderer);
    let visible_targets = [first.as_deref(), second.as_deref()];
    assert!(visible_targets.contains(&Some("Back")));
    assert!(visible_targets.contains(&Some("Content action")));
    focus::request_move_focus(FocusDirection::Previous);
    renderer.frames(8);
    assert_eq!(focused_label(&renderer), first);
    focus::clear_focus(true);

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        true,
        &observed,
    );
    renderer.resize(1, 1);
    renderer.frames(3);
    renderer.resize(600, 900);
    renderer.frames(8);
    assert!(
        renderer.read_pixels() == baseline,
        "nested-scroll geometry and pixels recover after collapse-sized viewports"
    );
    for _ in 0..12 {
        renderer.frames(1);
        let clock = renderer.frame_clock_diagnostics();
        assert_eq!(clock.resumed_tasks, 0, "the provider stops ticking at rest");
        assert_eq!(
            clock.waiting_tasks, 0,
            "the provider unregisters its frame callback at rest"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_app_bar_nested_scrolling_snap_color_and_inertia() {
    crate::test_utils::run_isolated_test(
        "app_bar::scroll_tests::gpu_app_bar_nested_scrolling_snap_color_and_inertia",
        check_scrolling,
    );
}
