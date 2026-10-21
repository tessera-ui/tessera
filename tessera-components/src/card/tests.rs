//! Card motion and interaction regressions.
//!
//! ## Usage
//!
//! Verify card variants remain usable during interaction and slow frames.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Callback, Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer,
    MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    remember, tessera, use_context,
};

use crate::{
    boxed::boxed,
    button::button,
    column::column,
    modifier::{InteractionState, ModifierExt as _},
    surface::surface,
    text::text,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};

use super::{CardElevationSpring, CardVariant, card};

#[test]
fn elevation_converges_after_slow_frames_idle_and_retargeting() {
    for interval in [16_666_667, 50_000_000, 200_000_000] {
        let mut spring = CardElevationSpring::new(Dp(1.0));
        spring.set_target(Dp(4.0));
        let mut now = 0;
        for _ in 0..120 {
            now += interval;
            spring.tick(now);
            assert!(spring.value_dp().0.is_finite());
            assert!((0.0..=6.0).contains(&spring.value_dp().0));
        }
        assert_eq!(spring.value_dp(), Dp(4.0));
        assert!(!spring.is_animating());
        now += 5_000_000_000;
        spring.set_target(Dp(1.0));
        spring.tick(now);
        assert!(
            (3.0..4.0).contains(&spring.value_dp().0),
            "retargeting after idle starts with a fresh frame delta"
        );
        for _ in 0..120 {
            now += interval;
            spring.tick(now);
        }
        assert_eq!(spring.value_dp(), Dp(1.0));
        assert!(!spring.is_animating());
        spring.set_target(Dp(4.0));
        spring.tick(now + interval);
        let velocity = spring.spring.velocity();
        spring.set_target(Dp(0.0));
        assert_eq!(spring.spring.velocity(), velocity);
        for frame in 2..122 {
            spring.tick(now + frame * interval);
        }
        assert_eq!(spring.value_dp(), Dp(0.0));
        assert!(!spring.is_animating());
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Config {
    variant: CardVariant,
    enabled: bool,
    clickable: bool,
    alternate: bool,
    covered: bool,
    dark: bool,
    nested_action: bool,
}

thread_local! {
    static CONFIG: Cell<Option<State<Config>>> = const { Cell::new(None) };
    static CALLS: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static NESTED_CALLS: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static INTERACTION: Cell<Option<State<InteractionState>>> = const { Cell::new(None) };
    static CONTENT_BUILDS: Cell<u32> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let config = remember(|| Config {
        variant: CardVariant::Elevated,
        enabled: true,
        clickable: true,
        alternate: false,
        covered: false,
        dark: false,
        nested_action: false,
    });
    let calls = remember(|| 0);
    let nested_calls = remember(|| 0);
    let interaction = remember(InteractionState::new);
    CONFIG.set(Some(config));
    CALLS.set(Some(calls));
    NESTED_CALLS.set(Some(nested_calls));
    INTERACTION.set(Some(interaction));
    material_theme()
        .theme(move || {
            let mut theme = MaterialTheme::default();
            if config.get().dark {
                theme.color_scheme = MaterialColorScheme::dark_from_seed(Color::from_rgb(
                    0.135633335,
                    0.080219820,
                    0.371237695,
                ));
                return theme;
            }
            theme.color_scheme.surface = Color::WHITE;
            theme.color_scheme.surface_container_low = if config.get().alternate {
                Color::new(0.0, 1.0, 1.0, 1.0)
            } else {
                Color::GREEN
            };
            theme.color_scheme.surface_container_highest = Color::RED;
            theme.color_scheme.surface_tint = Color::RED;
            theme.color_scheme.outline_variant = Color::BLACK;
            theme.color_scheme.on_surface = Color::BLUE;
            theme
        })
        .child(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(Color::WHITE)
                        .padding_all(Dp(32.0)),
                )
                .children(move || {
                    let on_click = if config.get().clickable {
                        Some(Callback::new(move || calls.with_mut(|calls| *calls += 1)))
                    } else {
                        None
                    };
                    card()
                        .modifier(Modifier::new().width(Dp(240.0)).height(Dp(96.0)))
                        .variant(config.get().variant)
                        .enabled(config.get().enabled)
                        .on_click_optional(on_click)
                        .interaction_state(interaction)
                        .accessibility_label("Card action")
                        .content(move || {
                            CONTENT_BUILDS.set(CONTENT_BUILDS.get() + 1);
                            boxed()
                                .modifier(Modifier::new().fill_max_size().background(
                                    if config.get().covered {
                                        Color::RED
                                    } else {
                                        Color::TRANSPARENT
                                    },
                                ))
                                .children(move || {
                                    text()
                                        .content("Card content")
                                        .modifier(Modifier::new().padding_all(Dp(16.0)));
                                    if config.get().nested_action {
                                        button()
                                            .accessibility_label("Nested action")
                                            .on_click(move || {
                                                nested_calls.with_mut(|calls| *calls += 1)
                                            })
                                            .child(|| {
                                                text().content("Nested action");
                                            });
                                    }
                                });
                        });
                    button()
                        .accessibility_label("After card")
                        .on_click(|| {})
                        .child(|| {
                            text().content("After card");
                        });
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    let color = match config.get().variant {
                        CardVariant::Filled => scheme.surface_container_highest,
                        CardVariant::Elevated => scheme.surface_container_low,
                        CardVariant::Outlined => scheme.surface,
                    };
                    surface()
                        .style(color.into())
                        .content_color(match config.get().variant {
                            CardVariant::Filled => scheme.on_surface_variant,
                            CardVariant::Elevated | CardVariant::Outlined => scheme.on_surface,
                        })
                        .modifier(Modifier::new().width(Dp(240.0)).height(Dp(96.0)))
                        .accessibility_role(Role::Group)
                        .accessibility_label("Reference color")
                        .child(|| {
                            text()
                                .content("Card content")
                                .accessibility_label("Reference label")
                                .modifier(Modifier::new().padding_all(Dp(16.0)));
                        });
                });
        });
}

fn control(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.label() == Some(label))
        .min_by_key(|(_, node)| node.role() != Role::Button)
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn action(renderer: &mut HeadlessRenderer, id: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }));
    renderer.render_frames(12);
}

fn pixel(renderer: &HeadlessRenderer, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
    renderer.read_pixels().expect("pixels")[offset..offset + 4]
        .try_into()
        .unwrap()
}

fn save(renderer: &HeadlessRenderer, samples: u32) {
    if let Some(directory) = std::env::var_os("TESSERA_CARD_QA_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("QA directory");
        let size = renderer.physical_size();
        tessera_ui::headless::write_png_file(
            directory.join(format!("card-msaa{samples}.png")),
            size.width,
            size.height,
            &renderer.read_pixels().expect("pixels"),
        )
        .expect("QA image");
    }
}

fn check_cards(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 540,
            height: 540,
            scale_factor: 1.5,
            sample_count: samples,
            frame_time: Duration::from_millis(if samples == 4 { 50 } else { 16 }),
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(12);
    let config = CONFIG.get().unwrap();
    let interaction = INTERACTION.get().unwrap();
    let calls = CALLS.get().unwrap();
    let (id, node) = control(&renderer, "Card action");
    let bounds = node.bounds().expect("card bounds");
    assert_eq!(node.role(), Role::Button);
    assert!(node.supports_action(Action::Click));
    assert!(node.supports_action(Action::Focus));
    assert_eq!(bounds.x1 - bounds.x0, 360.0);
    assert_eq!(bounds.y1 - bounds.y0, 144.0);
    assert_eq!(pixel(&renderer, 380, 160), [0, 255, 0, 255]);
    let baseline = renderer.read_pixels().expect("baseline pixels");
    let content_builds = CONTENT_BUILDS.get();
    let position = PxPosition::from_f32_xy(100.0, 100.0);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.render_frames(90);
    assert!(interaction.with(|state| state.is_hovered()));
    assert_ne!(renderer.read_pixels().expect("hover pixels"), baseline);
    assert_eq!(
        CONTENT_BUILDS.get(),
        content_builds,
        "elevation and state layers must not rebuild the content slot"
    );
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 330.0));
    renderer.render_frames(90);
    assert_eq!(renderer.read_pixels().expect("restored pixels"), baseline);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(2);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(12);
    assert_eq!(calls.get(), 1);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 330.0));
    let after = control(&renderer, "After card").0;
    action(&mut renderer, after, Action::Focus);
    renderer.render_frames(90);
    assert_eq!(control(&renderer, "Card action").0, id);
    config.with_mut(|config| config.alternate = true);
    renderer.render_frames(12);
    assert_eq!(pixel(&renderer, 380, 160), [0, 255, 255, 255]);
    config.with_mut(|config| {
        config.alternate = false;
        config.variant = CardVariant::Filled;
    });
    renderer.render_frames(90);
    assert_eq!(pixel(&renderer, 380, 160), [255, 0, 0, 255]);
    config.with_mut(|config| config.variant = CardVariant::Outlined);
    renderer.render_frames(90);
    assert_eq!(pixel(&renderer, 380, 160), [255, 255, 255, 255]);
    let outline = pixel(&renderer, 48, 110);
    assert!(
        outline[0] < 150 && outline[0] == outline[1] && outline[1] == outline[2],
        "outline={outline:?}"
    );
    assert_eq!(pixel(&renderer, 51, 110), [255, 255, 255, 255]);
    assert_eq!(pixel(&renderer, 49, 49), [255, 255, 255, 255]);
    action(&mut renderer, id, Action::Focus);
    let focused_outline = pixel(&renderer, 48, 110);
    assert!(
        focused_outline[2] > 240 && focused_outline[0] < 170 && focused_outline[1] < 170,
        "focused outline must use on-surface: {focused_outline:?}"
    );
    assert_eq!(calls.get(), 1);
    action(&mut renderer, after, Action::Focus);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.render_frames(90);
    let hover_shadow = pixel(&renderer, 380, 193);
    assert!(
        hover_shadow[0] < 255,
        "outlined hover must raise the card: {hover_shadow:?}"
    );
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 330.0));
    renderer.render_frames(90);
    interaction.with_mut(|state| state.set_dragged(true));
    renderer.render_frames(90);
    let dragged = pixel(&renderer, 380, 160);
    assert!(
        dragged[0].abs_diff(236) <= 1 && dragged[1].abs_diff(236) <= 1 && dragged[2] == 255,
        "dragged={dragged:?}"
    );
    config.with_mut(|config| config.enabled = false);
    renderer.render_frames(12);
    assert_eq!(interaction.get(), InteractionState::default());
    assert_eq!(pixel(&renderer, 380, 160), [255, 255, 255, 255]);
    let disabled = control(&renderer, "Card action").1;
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    assert!(!disabled.supports_action(Action::Focus));
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(12);
    assert_eq!(calls.get(), 1);
    FocusManager::current().request_move_focus(FocusDirection::Previous);
    renderer.render_frames(12);
    let tree = renderer.accessibility_snapshot().unwrap();
    assert_ne!(tree.focus, id);
    config.with_mut(|config| {
        config.enabled = true;
        config.clickable = false;
    });
    renderer.render_frames(12);
    let static_card = control(&renderer, "Card action").1;
    assert_ne!(static_card.role(), Role::Button);
    assert!(!static_card.supports_action(Action::Click));
    assert!(!static_card.supports_action(Action::Focus));
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Card content"))
    );
    config.with_mut(|config| config.clickable = true);
    renderer.render_frames(12);
    action(&mut renderer, id, Action::Focus);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
    renderer.resize(420, 360);
    renderer.render_frames(12);
    let resized = control(&renderer, "Card action")
        .1
        .bounds()
        .expect("resized bounds");
    assert!(resized.x1 <= 420.0 && resized.x1 - resized.x0 >= 72.0);
    renderer.resize(540, 540);
    renderer.render_frames(12);
    assert_eq!(control(&renderer, "Card action").1.bounds(), Some(bounds));
    action(&mut renderer, after, Action::Focus);
    config.with_mut(|config| config.clickable = false);
    renderer.render_frames(12);
    config.with_mut(|config| config.covered = true);
    renderer.render_frames(12);
    assert_eq!(pixel(&renderer, 49, 49), [255, 255, 255, 255]);
    assert_eq!(pixel(&renderer, 380, 160), [255, 0, 0, 255]);
    let covered_outline = pixel(&renderer, 48, 110);
    assert!(
        covered_outline[0] < 150 && covered_outline[1] < 150 && covered_outline[2] < 150,
        "opaque content must not cover the outline: {covered_outline:?}"
    );
    config.with_mut(|config| config.clickable = true);
    renderer.render_frames(12);
    interaction.with_mut(|state| state.set_dragged(true));
    renderer.render_frames(90);
    let covered_feedback = pixel(&renderer, 380, 160);
    assert!(
        covered_feedback[0].abs_diff(236) <= 1
            && covered_feedback[1] == 0
            && covered_feedback[2].abs_diff(111) <= 1,
        "drag feedback must cover opaque content: {covered_feedback:?}"
    );
    interaction.with_mut(|state| state.set_dragged(false));
    config.with_mut(|config| config.clickable = false);
    config.with_mut(|config| config.variant = CardVariant::Elevated);
    renderer.render_frames(12);
    let shadow = pixel(&renderer, 380, 193);
    assert!(
        shadow[0] < 255 && shadow[0] == shadow[1] && shadow[1] == shadow[2],
        "shadow={shadow:?}"
    );
    save(&renderer, samples);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 500.0));
    config.with_mut(|config| {
        config.dark = true;
        config.covered = false;
        config.clickable = true;
    });
    for variant in [
        CardVariant::Filled,
        CardVariant::Elevated,
        CardVariant::Outlined,
    ] {
        config.with_mut(|config| config.variant = variant);
        renderer.render_frames(90);
        assert_eq!(
            interaction.get(),
            InteractionState::default(),
            "theme reference comparisons require an idle card"
        );
        let reference = control(&renderer, "Reference color")
            .1
            .bounds()
            .expect("reference bounds");
        assert_eq!(
            pixel(&renderer, 380, 160),
            pixel(
                &renderer,
                (reference.x1 - 28.0) as u32,
                (reference.y1 - 32.0) as u32
            ),
            "dark {variant:?} must use its specified container role"
        );
        assert_eq!(control(&renderer, "Card action").0, id);
        let pixels = renderer.read_pixels().expect("dark text pixels");
        let width = renderer.physical_size().width as usize;
        for y in 24..64 {
            let card_start = ((48 + y) * width + 72) * 4;
            let reference_start =
                ((reference.y0 as usize + y) * width + reference.x0 as usize + 24) * 4;
            assert_eq!(
                &pixels[card_start..card_start + 200 * 4],
                &pixels[reference_start..reference_start + 200 * 4],
                "dark {variant:?} text must match its explicit content role at row {y}"
            );
        }
    }
    let dark_pixels = renderer.read_pixels().expect("dark pixels");
    let width = renderer.physical_size().width;
    assert!(
        (72..112).any(|y| (72..260).any(|x| {
            let offset = ((y * width + x) * 4) as usize;
            dark_pixels[offset..offset + 3]
                .iter()
                .all(|value| *value > 150)
        })),
        "dark card text must adapt to the light on-surface color"
    );
    config.with_mut(|config| {
        config.dark = false;
        config.variant = CardVariant::Elevated;
    });
    renderer.render_frames(90);
    let restored = renderer.read_pixels().expect("restored theme pixels");
    for y in 40..194 {
        let start = ((y * width + 40) * 4) as usize;
        let end = ((y * width + 420) * 4) as usize;
        assert!(
            baseline[start..end] == restored[start..end],
            "card pixels must restore after the theme cycle at y={y}"
        );
    }
    let parent_calls = calls.get();
    config.with_mut(|config| config.nested_action = true);
    renderer.render_frames(12);
    let (nested_id, nested) = control(&renderer, "Nested action");
    let bounds = nested.bounds().expect("nested action bounds");
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(100);
    assert_eq!(NESTED_CALLS.get().unwrap().get(), 1);
    assert_eq!(
        calls.get(),
        parent_calls,
        "the nested action consumes the click"
    );
    assert_eq!(
        renderer
            .accessibility_snapshot()
            .expect("nested focus")
            .focus,
        nested_id
    );
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(100);
    assert_eq!(
        NESTED_CALLS.get().unwrap().get(),
        2,
        "a press and release queued before one frame activate the nested action once"
    );
    assert_eq!(calls.get(), parent_calls);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_card_variants_interactions_and_disabled_semantics() {
    crate::test_utils::run_isolated_test(
        "card::tests::gpu_card_variants_interactions_and_disabled_semantics",
        || check_cards(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_card_variants_interactions_and_disabled_semantics_msaa() {
    crate::test_utils::run_isolated_test(
        "card::tests::gpu_card_variants_interactions_and_disabled_semantics_msaa",
        || check_cards(4),
    );
}
