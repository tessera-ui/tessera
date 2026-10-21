//! Pointer and pixel geometry after lazy-list content reflow.
//!
//! ## Usage
//!
//! Keep expanded list rows clickable at their visible scrolled positions.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, RenderSlot, State, accesskit::Node, remember, tessera,
};

use crate::{
    column::column,
    lazy_list::{LazyListController, lazy_column},
    modifier::ModifierExt as _,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};

use super::list_item;

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    expanded: bool,
    selected: bool,
    calls: usize,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static SCROLL: Cell<Option<State<LazyListController>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        expanded: false,
        selected: false,
        calls: 0,
    });
    let controller = remember(LazyListController::new);
    SETTINGS.set(Some(settings));
    SCROLL.set(Some(controller));
    material_theme()
        .theme(move || {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface = Color::RED;
            theme.color_scheme.secondary_container = Color::BLUE;
            if settings.get().expanded {
                theme.typography.body_large.font_size = Dp(22.0);
                theme.typography.body_large.line_height = Some(Dp(32.0));
            }
            theme
        })
        .child(move || {
            lazy_column()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .controller(controller)
                .overscan(1)
                .estimated_item_size(Dp(480.0))
                .item(|| {
                    spacer().modifier(Modifier::new().height(Dp(480.0)));
                })
                .item(move || {
                    let content = if settings.get().expanded {
                        Some(RenderSlot::new(|| {
                            column().children(|| {
                                text().content("One-line item");
                                text().content("Custom main content");
                            });
                        }))
                    } else {
                        None
                    };
                    list_item()
                        .modifier(Modifier::new().width(if settings.get().expanded {
                            Dp(200.0)
                        } else {
                            Dp(240.0)
                        }))
                        .headline("One-line item")
                        .content_optional(content)
                        .selected(settings.get().selected)
                        .accessibility_label("Scrolled row")
                        .leading(|| {
                            spacer().modifier(Modifier::new().size(Dp(20.0), Dp(20.0)));
                        })
                        .trailing(|| {
                            text().content("1");
                        })
                        .on_click(move || {
                            settings.with_mut(|s| {
                                s.calls += 1;
                                s.selected = !s.selected;
                            });
                        });
                })
                .item(|| {
                    spacer().modifier(Modifier::new().height(Dp(480.0)));
                });
        });
}

fn entry(renderer: &HeadlessRenderer) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Scrolled row"))
        .unwrap()
        .1
}

fn point(renderer: &HeadlessRenderer) -> PxPosition {
    let bounds = entry(renderer).bounds().unwrap();
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        (bounds.y1 - 6.0) as f32,
    )
}

fn pixel(renderer: &HeadlessRenderer, point: PxPosition) -> [u8; 4] {
    let pixels = renderer.read_pixels().unwrap();
    let offset = ((point.y as u32 * renderer.physical_size().width + point.x as u32) * 4) as usize;
    pixels[offset..offset + 4].try_into().unwrap()
}

fn tap(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(60);
}

fn check(samples: u32) {
    for scale in [1.0, 1.5] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: (400.0 * scale) as u32,
                height: (320.0 * scale) as u32,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let settings = SETTINGS.get().unwrap();
        let controller = SCROLL.get().unwrap();
        controller.with_mut(|c| c.scroll_to(Dp::ZERO, Dp(400.0)));
        renderer.render_frames(12);
        let initial = entry(&renderer).bounds().unwrap();
        assert!((initial.y0 - 80.0 * scale).abs() <= 1.0);
        settings.with_mut(|s| s.expanded = true);
        renderer.render_frames(30);
        let expanded = entry(&renderer).bounds().unwrap();
        assert!(expanded.y1 - expanded.y0 > initial.y1 - initial.y0);
        assert!(expanded.y1 < renderer.physical_size().height as f64);
        let expanded_point = point(&renderer);
        assert!(f64::from(expanded_point.y) > initial.y1);
        assert_eq!(pixel(&renderer, expanded_point), [255, 0, 0, 255]);
        tap(&mut renderer, expanded_point);
        assert_eq!(settings.get().calls, 1);
        assert_eq!(entry(&renderer).is_selected(), Some(true));
        settings.with_mut(|s| s.expanded = false);
        renderer.render_frames(30);
        let current_point = point(&renderer);
        tap(&mut renderer, current_point);
        assert_eq!(settings.get().calls, 2);
        assert_eq!(entry(&renderer).is_selected(), Some(false));
        let previous_point = point(&renderer);
        controller.with_mut(|c| c.scroll_to(Dp::ZERO, Dp::ZERO));
        renderer.render_frames(12);
        tap(&mut renderer, previous_point);
        assert_eq!(
            settings.get().calls,
            2,
            "old scrolled bounds must not receive clicks"
        );
        controller.with_mut(|c| c.scroll_to(Dp::ZERO, Dp(400.0)));
        renderer.render_frames(12);
        let current_point = point(&renderer);
        tap(&mut renderer, current_point);
        assert_eq!(settings.get().calls, 3);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn scrolled_list_reflow_preserves_visible_pixels_and_pointer_bounds_msaa1() {
    crate::test_utils::run_isolated_test(
        "list_item::reflow_tests::scrolled_list_reflow_preserves_visible_pixels_and_pointer_bounds_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn scrolled_list_reflow_preserves_visible_pixels_and_pointer_bounds_msaa4() {
    crate::test_utils::run_isolated_test(
        "list_item::reflow_tests::scrolled_list_reflow_preserves_visible_pixels_and_pointer_bounds_msaa4",
        || check(4),
    );
}
