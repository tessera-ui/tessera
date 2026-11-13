//! Card motion and interaction regressions.
//!
//! ## Usage
//!
//! Verify card variants remain usable during interaction and slow frames.

use std::time::Duration;

use tessera_foundation::modifier::{InteractionState, ModifierExt as _};
use tessera_ui::{
    Callback, Color, Dp, FocusDirection, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition,
    State,
    accesskit::{Action, Node, NodeId, Role},
    focus, remember, tessera,
    testing::UiTest,
    use_context,
};

use crate::{
    animation::test_instant as at,
    boxed::boxed,
    button::button,
    column::column,
    modifier::ModifierExt as _,
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
            spring.tick(at(now));
            assert!(spring.value_dp().0.is_finite());
            assert!((0.0..=6.0).contains(&spring.value_dp().0));
        }
        assert_eq!(spring.value_dp(), Dp(4.0));
        assert!(!spring.is_animating());
        now += 5_000_000_000;
        spring.set_target(Dp(1.0));
        spring.tick(at(now));
        assert!(
            (3.0..4.0).contains(&spring.value_dp().0),
            "retargeting after idle starts with a fresh frame delta"
        );
        for _ in 0..120 {
            now += interval;
            spring.tick(at(now));
        }
        assert_eq!(spring.value_dp(), Dp(1.0));
        assert!(!spring.is_animating());
        spring.set_target(Dp(4.0));
        spring.tick(at(now + interval));
        let velocity = spring.spring.velocity();
        spring.set_target(Dp(0.0));
        assert_eq!(spring.spring.velocity(), velocity);
        for frame in 2..122 {
            spring.tick(at(now + frame * interval));
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

#[derive(Clone, Default)]
struct FixtureHandles {
    config: Option<State<Config>>,
    calls: Option<State<u32>>,
    nested_calls: Option<State<u32>>,
    interaction: Option<State<InteractionState>>,
}

#[tessera]
fn fixture(probe: tessera_ui::testing::TestProbe<FixtureHandles>) {
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
    probe.set(FixtureHandles {
        config: Some(config),
        calls: Some(calls),
        nested_calls: Some(nested_calls),
        interaction: Some(interaction),
    });
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
                            card_content(config, nested_calls);
                        });
                    button()
                        .accessibility_label("After card")
                        .on_click(|| {})
                        .child(|| {
                            text("After card");
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
                            text("Card content")
                                .accessibility_label("Reference label")
                                .modifier(Modifier::new().padding_all(Dp(16.0)));
                        });
                });
        });
}

#[tessera]
fn card_content(config: State<Config>, nested_calls: State<u32>) {
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(if config.get().covered {
                    Color::RED
                } else {
                    Color::TRANSPARENT
                }),
        )
        .children(move || {
            text("Card content").modifier(Modifier::new().padding_all(Dp(16.0)));
            if config.get().nested_action {
                button()
                    .accessibility_label("Nested action")
                    .on_click(move || nested_calls.with_mut(|calls| *calls += 1))
                    .child(|| {
                        text("Nested action");
                    });
            }
        });
}

fn control(renderer: &UiTest, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.label() == Some(label))
        .min_by_key(|(_, node)| node.role() != Role::Button)
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn action(renderer: &mut UiTest, id: NodeId, action: Action) {
    assert!(renderer.accessibility_action(id, action));
    renderer.frames(12);
}

fn check_cards() {
    let probe = tessera_ui::testing::TestProbe::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(540, 540)
        .scale_factor(1.5)
        .frame_step(Duration::from_millis(16))
        .rendered(crate::TesseraComponents)
        .content(move || fixture(observed.clone()));
    renderer.frames(1);
    assert!(
        renderer
            .work()
            .count("card_content", tessera_ui::testing::WorkPhase::Build)
            > 0
    );
    renderer.frames(11);
    let config = probe.get().config.unwrap();
    let interaction = probe.get().interaction.unwrap();
    let calls = probe.get().calls.unwrap();
    let (id, node) = control(&renderer, "Card action");
    let bounds = node.bounds().expect("card bounds");
    assert_eq!(node.role(), Role::Button);
    assert!(node.supports_action(Action::Click));
    assert!(node.supports_action(Action::Focus));
    assert_eq!(bounds.x1 - bounds.x0, 360.0);
    assert_eq!(bounds.y1 - bounds.y0, 144.0);
    assert_eq!(renderer.pixel(380, 160), [0, 255, 0, 255]);
    let baseline = renderer.read_pixels();
    let position = PxPosition::from_f32_xy(100.0, 100.0);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    for _ in 0..90 {
        renderer.frames(1);
        assert_eq!(
            renderer
                .work()
                .count("card_content", tessera_ui::testing::WorkPhase::Build),
            0,
            "elevation and state layers must not rebuild the content component"
        );
    }
    assert!(interaction.with(|state| state.is_hovered()));
    assert_ne!(renderer.read_pixels(), baseline);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 330.0));
    renderer.frames(90);
    assert_eq!(renderer.read_pixels(), baseline);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(2);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(12);
    assert_eq!(calls.get(), 1);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 330.0));
    let after = control(&renderer, "After card").0;
    action(&mut renderer, after, Action::Focus);
    renderer.frames(90);
    assert_eq!(control(&renderer, "Card action").0, id);
    config.with_mut(|config| config.alternate = true);
    renderer.frames(12);
    assert_eq!(renderer.pixel(380, 160), [0, 255, 255, 255]);
    config.with_mut(|config| {
        config.alternate = false;
        config.variant = CardVariant::Filled;
    });
    renderer.frames(90);
    assert_eq!(renderer.pixel(380, 160), [255, 0, 0, 255]);
    config.with_mut(|config| config.variant = CardVariant::Outlined);
    renderer.frames(90);
    assert_eq!(renderer.pixel(380, 160), [255, 255, 255, 255]);
    let outline = renderer.pixel(48, 110);
    assert!(
        outline[0] < 150 && outline[0] == outline[1] && outline[1] == outline[2],
        "outline={outline:?}"
    );
    assert_eq!(renderer.pixel(51, 110), [255, 255, 255, 255]);
    assert_eq!(renderer.pixel(49, 49), [255, 255, 255, 255]);
    action(&mut renderer, id, Action::Focus);
    let focused_outline = renderer.pixel(48, 110);
    assert!(
        focused_outline[2] > 240 && focused_outline[0] < 170 && focused_outline[1] < 170,
        "focused outline must use on-surface: {focused_outline:?}"
    );
    assert_eq!(calls.get(), 1);
    action(&mut renderer, after, Action::Focus);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.frames(90);
    let hover_shadow = renderer.pixel(380, 193);
    assert!(
        hover_shadow[0] < 255,
        "outlined hover must raise the card: {hover_shadow:?}"
    );
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(500.0, 330.0));
    renderer.frames(90);
    interaction.with_mut(|state| state.set_dragged(true));
    renderer.frames(90);
    let dragged = renderer.pixel(380, 160);
    assert!(
        dragged[0].abs_diff(236) <= 1 && dragged[1].abs_diff(236) <= 1 && dragged[2] == 255,
        "dragged={dragged:?}"
    );
    config.with_mut(|config| config.enabled = false);
    renderer.frames(12);
    assert_eq!(interaction.get(), InteractionState::default());
    assert_eq!(renderer.pixel(380, 160), [255, 255, 255, 255]);
    let disabled = control(&renderer, "Card action").1;
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    assert!(!disabled.supports_action(Action::Focus));
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(12);
    assert_eq!(calls.get(), 1);
    focus::request_move_focus(FocusDirection::Previous);
    renderer.frames(12);
    let tree = renderer.accessibility_snapshot().unwrap();
    assert_ne!(tree.focus, id);
    config.with_mut(|config| {
        config.enabled = true;
        config.clickable = false;
    });
    renderer.frames(12);
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
    renderer.frames(12);
    action(&mut renderer, id, Action::Focus);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
    renderer.resize(420, 360);
    renderer.frames(12);
    let resized = control(&renderer, "Card action")
        .1
        .bounds()
        .expect("resized bounds");
    assert!(resized.x1 <= 420.0 && resized.x1 - resized.x0 >= 72.0);
    renderer.resize(540, 540);
    renderer.frames(12);
    assert_eq!(control(&renderer, "Card action").1.bounds(), Some(bounds));
    action(&mut renderer, after, Action::Focus);
    config.with_mut(|config| config.clickable = false);
    renderer.frames(12);
    config.with_mut(|config| config.covered = true);
    renderer.frames(12);
    assert_eq!(renderer.pixel(49, 49), [255, 255, 255, 255]);
    assert_eq!(renderer.pixel(380, 160), [255, 0, 0, 255]);
    let covered_outline = renderer.pixel(48, 110);
    assert!(
        covered_outline[0] < 150 && covered_outline[1] < 150 && covered_outline[2] < 150,
        "opaque content must not cover the outline: {covered_outline:?}"
    );
    config.with_mut(|config| config.clickable = true);
    renderer.frames(12);
    interaction.with_mut(|state| state.set_dragged(true));
    renderer.frames(90);
    let covered_feedback = renderer.pixel(380, 160);
    assert!(
        covered_feedback[0].abs_diff(236) <= 1
            && covered_feedback[1] == 0
            && covered_feedback[2].abs_diff(111) <= 1,
        "drag feedback must cover opaque content: {covered_feedback:?}"
    );
    interaction.with_mut(|state| state.set_dragged(false));
    config.with_mut(|config| config.clickable = false);
    config.with_mut(|config| config.variant = CardVariant::Elevated);
    renderer.frames(12);
    let shadow = renderer.pixel(380, 193);
    assert!(
        shadow[0] < 255 && shadow[0] == shadow[1] && shadow[1] == shadow[2],
        "shadow={shadow:?}"
    );
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
        renderer.frames(90);
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
            renderer.pixel(380, 160),
            renderer.pixel((reference.x1 - 28.0) as u32, (reference.y1 - 32.0) as u32),
            "dark {variant:?} must use its specified container role"
        );
        assert_eq!(control(&renderer, "Card action").0, id);
        let pixels = renderer.read_pixels();
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
    let dark_pixels = renderer.read_pixels();
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
    focus::clear_focus(true);
    renderer.frames(90);
    let restored = renderer.read_pixels();
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
    renderer.frames(12);
    let (nested_id, nested) = control(&renderer, "Nested action");
    let bounds = nested.bounds().expect("nested action bounds");
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(1);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(100);
    assert_eq!(probe.get().nested_calls.unwrap().get(), 1);
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
    renderer.frames(100);
    assert_eq!(
        probe.get().nested_calls.unwrap().get(),
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
        check_cards,
    );
}
