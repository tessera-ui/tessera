use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Rect, Role},
    remember, tessera,
    testing::UiTest,
};

use crate::{
    boxed::boxed,
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, material_theme},
};

use super::{
    MenuAnchor, MenuController, MenuDefaults, MenuPosition, menu_group, menu_item, menu_provider,
};

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    controller: Option<State<MenuController>>,
    grouped: Option<State<bool>>,
    long: Option<State<bool>>,
    actions: Option<State<u32>>,
    reference: Option<State<Option<Vec<Rect>>>>,
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let controller = remember(MenuController::new);
    let grouped = remember(|| true);
    let long = remember(|| false);
    let actions = remember(|| 0);
    let reference = remember(|| None::<Vec<Rect>>);
    observed.with_mut(|handles| handles.controller = Some(controller));
    observed.with_mut(|handles| handles.grouped = Some(grouped));
    observed.with_mut(|handles| handles.long = Some(long));
    observed.with_mut(|handles| handles.actions = Some(actions));
    observed.with_mut(|handles| handles.reference = Some(reference));
    material_theme().child(move || {
        if let Some(rects) = reference.get() {
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: Color::WHITE,
                        })
                        .modifier(Modifier::new().fill_max_size())
                        .child(|| {});
                    for (index, rect) in rects.iter().enumerate() {
                        surface()
                            .modifier(
                                Modifier::new()
                                    .offset(
                                        Dp::from_pixels_f64(rect.x0),
                                        Dp::from_pixels_f64(rect.y0),
                                    )
                                    .size(
                                        Dp::from_pixels_f64(rect.x1 - rect.x0),
                                        Dp::from_pixels_f64(rect.y1 - rect.y0),
                                    ),
                            )
                            .shape(MenuDefaults::group_shape(if index == 0 {
                                MenuPosition::First
                            } else {
                                MenuPosition::Last
                            }))
                            .style(SurfaceStyle::Filled {
                                color: MaterialTheme::default().color_scheme.surface_container_low,
                            })
                            .elevation(Dp(3.0))
                            .accessibility_role(Role::Group)
                            .accessibility_label(format!("Reference {index}"))
                            .child(|| {});
                    }
                });
        } else {
            menu_provider()
                .controller(controller)
                .grouped(grouped.get())
                .max_height(Dp(160.0).into())
                .modifier(Modifier::new().width(Dp(240.0)))
                .main_content(|| {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: Color::WHITE,
                        })
                        .modifier(Modifier::new().fill_max_size())
                        .child(|| {});
                })
                .menu_content(move |menu| {
                    if grouped.get() {
                        menu_group()
                            .menu(menu)
                            .position(MenuPosition::First)
                            .accessibility_label("First group")
                            .children(move || {
                                menu_item()
                                    .menu(menu)
                                    .label("Share")
                                    .on_click(move || actions.set(actions.get() + 1));
                                if long.get() {
                                    for index in 0..12 {
                                        menu_item()
                                            .menu(menu)
                                            .label(format!("Extra action {index}"))
                                            .on_click(move || actions.set(actions.get() + 1));
                                    }
                                }
                            });
                        menu_group()
                            .menu(menu)
                            .position(MenuPosition::Last)
                            .accessibility_label("Last group")
                            .children(move || {
                                menu_item()
                                    .menu(menu)
                                    .label("Delete")
                                    .on_click(move || actions.set(actions.get() + 1));
                            });
                    } else {
                        menu_item().menu(menu).label("Copy link").on_click(|| {});
                    }
                });
        }
    });
}

fn bounds(renderer: &UiTest, role: Role, label: Option<&str>) -> Rect {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.role() == role && label.is_none_or(|label| node.label() == Some(label))
        })
        .expect("menu surface")
        .1
        .bounds()
        .expect("surface bounds")
}

fn pixel(pixels: &[u8], x: f64, y: f64) -> u8 {
    pixels[(y as usize * 900 + x as usize) * 4]
}

fn check_shadows(samples: u32) {
    let observed = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(900, 900)
        .scale_factor(1.5)
        .samples(samples)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(observed.clone()));
    renderer.frames(4);
    let background = renderer.read_pixels();
    let controller = observed.get().controller.expect("controller");
    let grouped = observed.get().grouped.expect("grouped");
    for is_grouped in [true, false] {
        grouped.set(is_grouped);
        controller.with_mut(|menu| {
            menu.open_at(MenuAnchor::from_dp(
                (Dp(80.0), Dp(60.0)),
                (Dp(80.0), Dp(40.0)),
            ));
        });
        renderer.frames(6);
        let pixels = renderer.read_pixels();
        let panel = bounds(&renderer, Role::Menu, None);
        let first = if is_grouped {
            bounds(&renderer, Role::Group, Some("First group"))
        } else {
            panel
        };
        for (x, y) in [
            (panel.x0 - 2.0, (first.y0 + first.y1) / 2.0),
            (panel.x1 + 1.0, (first.y0 + first.y1) / 2.0),
            ((panel.x0 + panel.x1) / 2.0, panel.y1 + 2.0),
        ] {
            let near = pixel(&pixels, x, y);
            assert!(
                near < 253,
                "shadow clipped at {x}, {y}: {near}; grouped={is_grouped}"
            );
        }
        for (x, y) in [
            (panel.x0 - 30.0, (first.y0 + first.y1) / 2.0),
            (panel.x1 + 30.0, (first.y0 + first.y1) / 2.0),
            ((panel.x0 + panel.x1) / 2.0, panel.y1 + 30.0),
        ] {
            assert_eq!(
                pixel(&pixels, x, y),
                255,
                "shadow must fade back to the page"
            );
        }
        assert!(
            pixel(
                &pixels,
                panel.x0 - Dp(4.0).to_pixels_f32() as f64,
                (first.y0 + first.y1) / 2.0
            ) < 253,
            "the ambient shadow must spread beyond the narrow key shadow"
        );
        renderer.frames(8);
        assert_eq!(renderer.read_pixels(), pixels);
        controller.with_mut(MenuController::close);
        renderer.frames(4);
        assert_eq!(renderer.read_pixels(), background);
    }

    grouped.set(true);
    observed.get().long.expect("long").set(true);
    controller.with_mut(|menu| {
        menu.open_at(MenuAnchor::from_dp(
            (Dp(80.0), Dp(60.0)),
            (Dp(80.0), Dp(40.0)),
        ));
    });
    renderer.frames(6);
    let panel = bounds(&renderer, Role::Menu, None);
    assert_eq!(
        panel.y1 - panel.y0,
        240.0,
        "long menu keeps its viewport limit"
    );
    let center = PxPosition::from_f32_xy(
        ((panel.x0 + panel.x1) / 2.0) as f32,
        ((panel.y0 + panel.y1) / 2.0) as f32,
    );
    renderer.pointer_scroll(MOUSE_POINTER_ID, center, 0.0, -2000.0);
    renderer.frames(90);
    let last = bounds(&renderer, Role::Group, Some("Last group"));
    assert!(
        last.y0 >= panel.y0 && last.y1 <= panel.y1 + 1.0,
        "last group must scroll into view: {last:?}, {panel:?}"
    );
    let pixels = renderer.read_pixels();
    assert_eq!(
        pixel(&pixels, center.x.into(), panel.y1 + 3.0),
        255,
        "scroll content must not paint below the viewport"
    );
    assert!(
        pixel(&pixels, panel.x0 - 2.0, (last.y0 + last.y1) / 2.0) < 253,
        "scrolling preserves lateral group shadows"
    );
    let outside =
        PxPosition::from_f32_xy((panel.x0 - 2.0) as f32, ((last.y0 + last.y1) / 2.0) as f32);
    renderer.pointer_press(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.frames(1);
    renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.frames(4);
    assert!(
        !controller.with(MenuController::is_open),
        "shadow pixels do not enlarge the menu's click target"
    );
    assert_eq!(observed.get().actions.expect("actions").get(), 0);
    assert_eq!(renderer.read_pixels(), background);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_shadows_extend_beyond_group_and_panel_edges() {
    crate::test_utils::run_isolated_test(
        "menus::shadow_tests::gpu_menu_shadows_extend_beyond_group_and_panel_edges",
        || check_shadows(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_shadows_extend_beyond_group_and_panel_edges_msaa() {
    crate::test_utils::run_isolated_test(
        "menus::shadow_tests::gpu_menu_shadows_extend_beyond_group_and_panel_edges_msaa",
        || check_shadows(4),
    );
}

fn inside_group(rect: Rect, index: usize, x: f64, y: f64) -> bool {
    if x < rect.x0 || x >= rect.x1 || y < rect.y0 || y >= rect.y1 {
        return false;
    }
    let outer = Dp(16.0).to_pixels_f32() as f64;
    let inner = Dp(8.0).to_pixels_f32() as f64;
    let radius = if (y < (rect.y0 + rect.y1) / 2.0) == (index == 0) {
        outer
    } else {
        inner
    };
    let dx = x - x.clamp(rect.x0 + radius, rect.x1 - radius);
    let dy = y - y.clamp(rect.y0 + radius, rect.y1 - radius);
    dx * dx + dy * dy <= (radius + 1.0).powi(2)
}

fn check_corner_shadows(samples: u32) {
    let observed = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(900, 900)
        .scale_factor(1.5)
        .samples(samples)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(observed.clone()));
    renderer.frames(4);
    observed
        .get()
        .controller
        .expect("controller")
        .with_mut(|menu| {
            menu.open_at(MenuAnchor::from_dp(
                (Dp(80.0), Dp(60.0)),
                (Dp(80.0), Dp(40.0)),
            ));
        });
    renderer.frames(8);
    let rects = vec![
        bounds(&renderer, Role::Group, Some("First group")),
        bounds(&renderer, Role::Group, Some("Last group")),
    ];
    let actual = renderer.read_pixels();
    observed
        .get()
        .reference
        .expect("reference")
        .set(Some(rects.clone()));
    renderer.frames(8);
    let reference = renderer.read_pixels();
    for (index, rect) in rects.iter().enumerate() {
        assert_eq!(
            bounds(&renderer, Role::Group, Some(&format!("Reference {index}"))),
            *rect
        );
    }
    let mut checked = 0;
    for rect in &rects {
        for (cx, cy) in [
            (rect.x0, rect.y0),
            (rect.x1, rect.y0),
            (rect.x1, rect.y1),
            (rect.x0, rect.y1),
        ] {
            for y in (cy as usize - 14)..(cy as usize + 14) {
                for x in (cx as usize - 14)..(cx as usize + 14) {
                    if rects.iter().enumerate().any(|(index, rect)| {
                        inside_group(*rect, index, x as f64 + 0.5, y as f64 + 0.5)
                    }) {
                        continue;
                    }
                    let offset = (y * 900 + x) * 4;
                    for channel in 0..4 {
                        assert!(
                            actual[offset + channel].abs_diff(reference[offset + channel]) <= 2,
                            "corner shadow at ({x}, {y}), channel {channel}: {} != {}",
                            actual[offset + channel],
                            reference[offset + channel]
                        );
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 3000, "all eight corners must be compared");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_corner_shadows_match_unclipped_surfaces() {
    crate::test_utils::run_isolated_test(
        "menus::shadow_tests::gpu_menu_corner_shadows_match_unclipped_surfaces",
        || check_corner_shadows(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_corner_shadows_match_unclipped_surfaces_msaa() {
    crate::test_utils::run_isolated_test(
        "menus::shadow_tests::gpu_menu_corner_shadows_match_unclipped_surfaces_msaa",
        || check_corner_shadows(4),
    );
}
