//! List color feedback through the headless renderer.
//!
//! ## Usage
//!
//! Preserve transitions and state precedence without pointer or shape updates.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, Modifier, State, remember, tessera,
    testing::{TestProbe, UiTest},
    use_context,
};

use crate::{
    md3::theme::{ContentColor, MaterialTheme, material_theme},
    modifier::ModifierExt as _,
    shape_def::Shape,
    spacer::spacer,
};

use super::{ListItemDefaults, list_item};

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    selected: bool,
    dragged: bool,
    enabled: bool,
    alternate: bool,
    custom: bool,
}

#[tessera]
fn ink(name: String) {
    let color = use_context::<ContentColor>().unwrap().get().current;
    spacer().modifier(
        Modifier::new()
            .size(Dp(8.0), Dp(8.0))
            .background(color)
            .semantics(SemanticsArgs {
                test_tag: Some(name),
                ..Default::default()
            }),
    );
}

#[tessera]
fn fixture(probe: Option<TestProbe<Option<State<Settings>>>>) {
    let settings = remember(|| Settings {
        selected: false,
        dragged: false,
        enabled: true,
        alternate: false,
        custom: false,
    });
    if let Some(probe) = probe {
        probe.set(Some(settings));
    }
    material_theme()
        .theme(move || {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface = Color::WHITE;
            theme.color_scheme.on_surface = Color::RED;
            theme.color_scheme.on_surface_variant = Color::GREEN;
            theme.color_scheme.secondary_container = if settings.get().alternate {
                Color::GREEN
            } else {
                Color::BLUE
            };
            theme.color_scheme.on_secondary_container = Color::WHITE;
            theme.color_scheme.tertiary_container = Color::RED;
            theme.color_scheme.on_tertiary_container = Color::BLUE;
            theme
        })
        .child(move || {
            let value = settings.get();
            let theme = use_context::<MaterialTheme>().unwrap().get();
            let mut colors = ListItemDefaults::colors(&theme.color_scheme);
            if value.custom {
                colors.dragged_container_color = Color::GREEN;
                colors.dragged_leading_color = Color::RED;
                colors.dragged_trailing_color = Color::WHITE;
            }
            list_item()
                .modifier(Modifier::new().width(Dp(160.0)))
                .shape(Shape::RECTANGLE)
                .enabled(value.enabled)
                .selected(value.selected)
                .dragged(value.dragged)
                .colors(colors)
                .leading(|| {
                    ink("lead".into());
                })
                .trailing(|| {
                    ink("trail".into());
                });
        });
}

fn slot_pixel(renderer: &UiTest, tag: &str) -> [u8; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == tag {
            let bounds = &node["bounds"];
            let x = bounds["x"].as_i64().unwrap() + bounds["width"].as_i64().unwrap() / 2;
            let y = bounds["y"].as_i64().unwrap() + bounds["height"].as_i64().unwrap() / 2;
            return renderer.pixel(x as u32, y as u32);
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {tag}");
}

fn near(actual: [u8; 4], expected: [u8; 4]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(actual.abs_diff(expected) <= 2, "{actual} != {expected}");
    }
}

fn check() {
    for scale in [1.0, 1.5] {
        let probe = TestProbe::<Option<State<Settings>>>::default();
        let observed = probe.clone();
        let mut renderer = UiTest::new()
            .viewport(300, 160)
            .scale_factor(scale)
            .rendered(crate::TesseraComponents)
            .content(move || fixture().probe(probe.clone()));
        renderer.frames(4);
        assert!(
            renderer
                .work_totals()
                .count("list_item", tessera_ui::testing::WorkPhase::Build)
                > 0
        );
        let settings = observed.get().unwrap();
        near(renderer.pixel(4, 4), [255; 4]);
        near(slot_pixel(&renderer, "lead"), [0, 255, 0, 255]);
        settings.with_mut(|s| s.selected = true);
        renderer.frames(3);
        let partial = renderer.pixel(4, 4);
        assert_ne!(
            partial, [255; 4],
            "selection must animate without shape motion"
        );
        assert_ne!(
            partial,
            [0, 0, 255, 255],
            "selection must have intermediate colors"
        );
        renderer.frames(30);
        near(renderer.pixel(4, 4), [0, 0, 255, 255]);
        near(slot_pixel(&renderer, "lead"), [255; 4]);
        settings.with_mut(|s| s.selected = false);
        renderer.frames(3);
        settings.with_mut(|s| s.selected = true);
        renderer.frames(30);
        near(renderer.pixel(4, 4), [0, 0, 255, 255]);
        settings.with_mut(|s| s.alternate = true);
        renderer.frames(30);
        near(renderer.pixel(4, 4), [0, 255, 0, 255]);
        settings.with_mut(|s| s.dragged = true);
        renderer.frames(30);
        near(renderer.pixel(4, 4), [255, 0, 0, 255]);
        near(slot_pixel(&renderer, "lead"), [0, 0, 255, 255]);
        near(slot_pixel(&renderer, "trail"), [0, 0, 255, 255]);
        settings.with_mut(|s| s.custom = true);
        renderer.frames(30);
        near(renderer.pixel(4, 4), [0, 255, 0, 255]);
        near(slot_pixel(&renderer, "lead"), [255, 0, 0, 255]);
        near(slot_pixel(&renderer, "trail"), [255; 4]);
        settings.with_mut(|s| s.enabled = false);
        renderer.frames(30);
        near(renderer.pixel(4, 4), [255; 4]);
        near(slot_pixel(&renderer, "lead"), [255, 206, 206, 255]);
        near(slot_pixel(&renderer, "trail"), [255, 206, 206, 255]);
        settings.with_mut(|s| {
            s.enabled = true;
            s.selected = false;
            s.dragged = false;
            s.custom = false;
            s.alternate = false;
        });
        renderer.frames(30);
        near(renderer.pixel(4, 4), [255; 4]);
        near(slot_pixel(&renderer, "lead"), [0, 255, 0, 255]);
        let builds = renderer
            .work_totals()
            .count("list_item", tessera_ui::testing::WorkPhase::Build);
        let final_pixels = renderer.read_pixels();
        renderer.frames(8);
        assert_eq!(
            renderer
                .work_totals()
                .count("list_item", tessera_ui::testing::WorkPhase::Build),
            builds,
            "settled colors must stop scheduling builds"
        );
        assert_eq!(renderer.read_pixels(), final_pixels);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn list_colors_animate_reverse_retarget_and_settle_without_shape_motion() {
    crate::test_utils::run_isolated_test(
        "md3::list_item::color_tests::list_colors_animate_reverse_retarget_and_settle_without_shape_motion",
        check,
    );
}
