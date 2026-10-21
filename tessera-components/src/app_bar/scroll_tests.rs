use std::{cell::Cell, time::Duration};

use tessera_ui::{
    CursorEventContent, FocusDirection, FocusManager, GestureState, HeadlessConfig,
    HeadlessRenderer, MOUSE_POINTER_ID, PointerChange, PressKeyEventType, ScrollDeltaUnit,
    ScrollEventContent, ScrollEventSource, accesskit::Role, remember,
};

use crate::{
    button::button,
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

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static BAR: Cell<Option<State<AppBarScrollState>>> = const { Cell::new(None) };
    static CONTENT: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        behavior: AppBarScrollBehavior::ExitUntilCollapsed,
        enabled: true,
        snap: true,
        variant: TopAppBarVariant::LargeFlexible,
    });
    let state = remember(AppBarScrollState::default);
    let content = remember(ScrollableController::new);
    SETTINGS.set(Some(settings));
    BAR.set(Some(state));
    CONTENT.set(Some(content));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface = Color::WHITE;
            theme.color_scheme.surface_container = Color::RED;
            theme
        })
        .child(move || {
            app_bar_scroll_provider()
                .state(state)
                .behavior(settings.get().behavior)
                .enabled(settings.get().enabled)
                .snap(settings.get().snap)
                .scroll_controller(content)
                .child(move || {
                    scaffold()
                        .modifier(Modifier::new().fill_max_size())
                        .top_bar(move || {
                            top_app_bar()
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
                                                            text().content("Content action");
                                                        });
                                                } else {
                                                    text().content(format!("Item {index}"));
                                                }
                                            });
                                    }
                                });
                        });
                });
        });
}

fn height(renderer: &HeadlessRenderer) -> f64 {
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

fn labels(renderer: &HeadlessRenderer) -> usize {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.role() == Role::Label && node.label() == Some("Library"))
        .count()
}

fn focused_label(renderer: &HeadlessRenderer) -> Option<String> {
    let tree = renderer.accessibility_snapshot().expect("tree");
    tree.nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .and_then(|(_, node)| node.label().map(str::to_owned))
}

fn background(renderer: &HeadlessRenderer, color: [u8; 3]) {
    let pixels = renderer.read_pixels().expect("pixels");
    let index = (5 * renderer.physical_size().width as usize + 500) * 4;
    assert_eq!(&pixels[index..index + 3], &color);
}

fn wheel(renderer: &mut HeadlessRenderer, delta: f32) {
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(300.0, 500.0),
        0.0,
        delta,
    );
    renderer.render_frames(3);
}

fn reset(renderer: &mut HeadlessRenderer, behavior: AppBarScrollBehavior, snap: bool) {
    SETTINGS.get().expect("settings").with_mut(|settings| {
        settings.behavior = behavior;
        settings.snap = snap;
        settings.variant = TopAppBarVariant::LargeFlexible;
        settings.enabled = true;
    });
    BAR.get().expect("bar").with_mut(AppBarScrollState::reset);
    CONTENT
        .get()
        .expect("content")
        .with_mut(|controller| controller.set_scroll_position(PxPosition::ZERO));
    renderer.render_frames(8);
}

fn touch(renderer: &mut HeadlessRenderer, content: CursorEventContent, state: GestureState) {
    let timestamp = tessera_ui::current_frame_time().expect("frame timestamp")
        + Duration::from_nanos(16_666_667);
    renderer.inject_pointer(
        PointerChange::new(timestamp, MOUSE_POINTER_ID, content, state)
            .with_position(PxPosition::from_f32_xy(300.0, 500.0)),
    );
    renderer.step_frame();
}

fn check_scrolling(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 900,
            scale_factor: 1.5,
            sample_count: samples,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(8);
    let bar = BAR.get().expect("bar");
    let content = CONTENT.get().expect("content");
    assert_eq!(height(&renderer), 228.0);
    assert_eq!(labels(&renderer), 1);
    let baseline = renderer.read_pixels().expect("baseline");
    wheel(&mut renderer, -30.0);
    assert!((height(&renderer) - 198.0).abs() <= 1.0);
    assert_eq!(content.with(ScrollableController::target_position).y, 0.0);
    assert_eq!(labels(&renderer), 1);
    renderer.render_frames(90);
    assert_eq!(height(&renderer), 228.0);
    assert!(!bar.with(AppBarScrollState::is_animating));
    assert!(
        renderer.read_pixels().expect("restored") == baseline,
        "a short wheel gesture snaps back to the original pixels"
    );

    wheel(&mut renderer, -180.0);
    renderer.render_frames(90);
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
    renderer.render_frames(90);
    assert_eq!(height(&renderer), 228.0);
    background(&renderer, [255, 255, 255]);

    reset(&mut renderer, AppBarScrollBehavior::EnterAlways, false);
    wheel(&mut renderer, -300.0);
    renderer.render_frames(40);
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

    reset(&mut renderer, AppBarScrollBehavior::Pinned, true);
    wheel(&mut renderer, -60.0);
    renderer.render_frames(90);
    assert_eq!(height(&renderer), 228.0);
    background(&renderer, [255, 0, 0]);
    content.with_mut(|controller| controller.set_scroll_position(PxPosition::ZERO));
    renderer.render_frames(90);
    background(&renderer, [255, 255, 255]);
    assert!(!bar.with(AppBarScrollState::is_animating));

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        false,
    );
    SETTINGS
        .get()
        .expect("settings")
        .with_mut(|settings| settings.enabled = false);
    renderer.render_frames(3);
    wheel(&mut renderer, -50.0);
    assert_eq!(height(&renderer), 228.0);
    assert!(content.with(ScrollableController::target_position).y < 0.0);

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        true,
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
    renderer.render_frames(30);
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
    renderer.render_frames(90);
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
    renderer.step_frame();
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(300.0, 20.0));
    renderer.step_frame();
    renderer.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(300.0, 20.0),
        PressKeyEventType::Left,
    );
    renderer.render_frames(90);
    assert_eq!(
        height(&renderer),
        96.0,
        "dragging the bar itself updates its shared collapse state"
    );

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        false,
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
    let before_release = bar.with(AppBarScrollState::collapsed_fraction);
    assert!(before_release > 0.0 && before_release < 1.0);
    assert_eq!(content.with(ScrollableController::target_position).y, 0.0);
    touch(
        &mut renderer,
        CursorEventContent::Released(PressKeyEventType::Left),
        GestureState::Dragged,
    );
    renderer.render_frames(8);
    assert_eq!(
        bar.with(AppBarScrollState::collapsed_fraction),
        1.0,
        "touch velocity consumed by the header continues through nested inertia"
    );
    let after_fling = content.with(ScrollableController::target_position).y;
    assert!(
        after_fling < 0.0,
        "remaining fling movement reaches the content"
    );
    renderer.render_frames(8);
    assert!(
        content.with(ScrollableController::target_position).y < after_fling,
        "fractional inertia steps continue after header collapse"
    );
    renderer.render_frames(180);
    assert!(!bar.with(AppBarScrollState::is_animating));

    reset(&mut renderer, AppBarScrollBehavior::EnterAlways, false);
    SETTINGS
        .get()
        .expect("settings")
        .with_mut(|settings| settings.variant = TopAppBarVariant::Small);
    renderer.render_frames(5);
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
    FocusManager::current().clear_focus(true);
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Content action"));
    assert_eq!(height(&renderer), 0.0);
    FocusManager::current().request_move_focus(FocusDirection::Previous);
    renderer.render_frames(8);
    assert_eq!(focused_label(&renderer).as_deref(), Some("Content action"));
    assert_eq!(height(&renderer), 0.0);
    wheel(&mut renderer, 150.0);
    assert_eq!(height(&renderer), 96.0);
    assert_eq!(labels(&renderer), 1);
    FocusManager::current().clear_focus(true);
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(8);
    let first = focused_label(&renderer);
    FocusManager::current().request_move_focus(FocusDirection::Next);
    renderer.render_frames(8);
    let second = focused_label(&renderer);
    let visible_targets = [first.as_deref(), second.as_deref()];
    assert!(visible_targets.contains(&Some("Back")));
    assert!(visible_targets.contains(&Some("Content action")));
    FocusManager::current().request_move_focus(FocusDirection::Previous);
    renderer.render_frames(8);
    assert_eq!(focused_label(&renderer), first);
    FocusManager::current().clear_focus(true);

    reset(
        &mut renderer,
        AppBarScrollBehavior::ExitUntilCollapsed,
        true,
    );
    renderer.resize(1, 1);
    renderer.render_frames(3);
    renderer.resize(600, 900);
    renderer.render_frames(8);
    assert!(
        renderer.read_pixels().expect("resized") == baseline,
        "nested-scroll geometry and pixels recover after collapse-sized viewports"
    );
    let ticks = bar.with(|state| state.frame_ticks);
    renderer.render_frames(12);
    assert_eq!(
        bar.with(|state| state.frame_ticks),
        ticks,
        "the provider unregisters its frame callback at rest"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_app_bar_nested_scrolling_snap_color_and_inertia() {
    crate::test_utils::run_isolated_test(
        "app_bar::scroll_tests::gpu_app_bar_nested_scrolling_snap_color_and_inertia",
        || check_scrolling(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_app_bar_nested_scrolling_snap_color_and_inertia_msaa() {
    crate::test_utils::run_isolated_test(
        "app_bar::scroll_tests::gpu_app_bar_nested_scrolling_snap_color_and_inertia_msaa",
        || check_scrolling(4),
    );
}
