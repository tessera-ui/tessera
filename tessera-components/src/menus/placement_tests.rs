use std::cell::Cell;

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Rect, Role},
    remember, tessera,
};

use crate::{
    boxed::boxed,
    modifier::{ModifierExt as _, Padding},
    surface::{SurfaceStyle, surface},
    theme::material_theme,
};

use super::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider};

#[derive(Clone, Copy)]
struct Settings {
    anchor: Option<MenuAnchor>,
    placement: MenuPlacement,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            anchor: Some(MenuAnchor::from_dp(
                (Dp(80.0), Dp(80.0)),
                (Dp(80.0), Dp(40.0)),
            )),
            placement: MenuPlacement::BelowStart,
        }
    }
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CONTROLLER: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let controller = remember(MenuController::new);
    SETTINGS.set(Some(settings));
    CONTROLLER.set(Some(controller));
    material_theme().child(move || {
        boxed()
            .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                Dp(32.0),
                Dp(24.0),
                Dp::ZERO,
                Dp::ZERO,
            )))
            .children(move || {
                menu_provider()
                    .anchor_optional(settings.get().anchor)
                    .placement(settings.get().placement)
                    .offset([Dp(12.0), Dp(8.0)])
                    .elevation(Dp::ZERO)
                    .controller(controller)
                    .modifier(Modifier::new().width(Dp(180.0)))
                    .main_content(|| {
                        surface()
                            .style(SurfaceStyle::Filled {
                                color: Color::WHITE,
                            })
                            .modifier(Modifier::new().fill_max_size())
                            .child(|| {});
                    })
                    .menu_content(|| {
                        menu_item().label("First action").on_click(|| {});
                        menu_item().label("Second action").on_click(|| {});
                    });
            });
    });
}

fn panel(renderer: &HeadlessRenderer) -> Rect {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Menu)
        .expect("menu")
        .1
        .bounds()
        .expect("bounds")
}

fn check_placement(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 900,
            height: 900,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(3);
    let settings = SETTINGS.get().expect("settings");
    let controller = CONTROLLER.get().expect("controller");
    controller.with_mut(|menu| {
        menu.open_at(MenuAnchor::from_dp(
            (Dp(200.0), Dp(200.0)),
            (Dp(80.0), Dp(40.0)),
        ))
    });
    renderer.render_frames(4);
    let first = panel(&renderer);
    assert_eq!(
        first.x0,
        (32.0 + 80.0 + 12.0) * 1.5,
        "current anchor overrides the controller snapshot"
    );
    assert_eq!(first.y0, (24.0 + 80.0 + 40.0 + 8.0) * 1.5);
    let tree = renderer.accessibility_snapshot().expect("tree");
    let focus = tree.focus;
    assert_eq!(
        tree.nodes
            .iter()
            .find(|(id, _)| *id == focus)
            .expect("focused menu action")
            .1
            .label(),
        Some("First action"),
        "opening the menu focuses its first action"
    );
    settings.with_mut(|settings| {
        settings.anchor = Some(MenuAnchor::from_dp(
            (Dp(250.0), Dp(240.0)),
            (Dp(80.0), Dp(40.0)),
        ))
    });
    renderer.render_frames(4);
    let moved = panel(&renderer);
    assert_eq!(
        moved.x0,
        (32.0 + 250.0 + 12.0) * 1.5,
        "open menu follows reflow without reopening"
    );
    assert_eq!(moved.y0, (24.0 + 240.0 + 40.0 + 8.0) * 1.5);
    assert_eq!(
        renderer.accessibility_snapshot().expect("tree").focus,
        focus,
        "repositioning preserves menu focus"
    );
    let pixels = renderer.read_pixels().expect("moved pixels");
    let previous_center = ((first.x0 + first.x1) / 2.0, (first.y0 + first.y1) / 2.0);
    let index = (previous_center.1 as usize * 900 + previous_center.0 as usize) * 4;
    assert_eq!(
        &pixels[index..index + 4],
        &[255; 4],
        "old panel pixels are removed"
    );

    for (placement, x, y, width, height) in [
        (MenuPlacement::BelowStart, 80.0, 480.0, 80.0, 40.0),
        (MenuPlacement::AboveEnd, 80.0, 8.0, 80.0, 40.0),
        (MenuPlacement::RightStart, 340.0, 120.0, 40.0, 40.0),
        (MenuPlacement::LeftStart, 8.0, 120.0, 40.0, 40.0),
    ] {
        settings.set(Settings {
            anchor: Some(MenuAnchor::from_dp((Dp(x), Dp(y)), (Dp(width), Dp(height)))),
            placement,
        });
        renderer.render_frames(4);
        let bounds = panel(&renderer);
        match placement {
            MenuPlacement::BelowStart => assert_eq!(
                bounds.y1,
                (24.0 + y - 8.0) * 1.5,
                "bottom-edge menu opens above its anchor"
            ),
            MenuPlacement::AboveEnd => assert_eq!(
                bounds.y0,
                (24.0 + y + height + 8.0) * 1.5,
                "top-edge menu opens below its anchor"
            ),
            MenuPlacement::RightStart => assert_eq!(
                bounds.x1,
                (32.0 + x - 12.0) * 1.5,
                "side flip accounts for the requested gap"
            ),
            MenuPlacement::LeftStart => assert_eq!(
                bounds.x0,
                (32.0 + x + width + 12.0) * 1.5,
                "left-edge menu opens to the right"
            ),
            _ => unreachable!(),
        }
        assert!(bounds.x0 >= 48.0 && bounds.x1 <= 900.0 && bounds.y0 >= 36.0 && bounds.y1 <= 900.0);
    }
    renderer.resize(318, 900);
    renderer.render_frames(4);
    let fitted = panel(&renderer);
    assert_eq!(fitted.x0, 48.0, "exact-fit viewport clamps to its origin");
    assert_eq!(fitted.x1, 318.0);
    renderer.resize(900, 900);
    settings.set(Settings {
        anchor: None,
        placement: MenuPlacement::BelowStart,
    });
    renderer.render_frames(4);
    let saved = panel(&renderer);
    assert_eq!(
        saved.x0,
        (32.0 + 200.0 + 12.0) * 1.5,
        "clearing the current anchor restores the controller snapshot"
    );
    assert_eq!(saved.y0, (24.0 + 200.0 + 40.0 + 8.0) * 1.5);
    controller.with_mut(MenuController::close);
    renderer.render_frames(4);
    assert!(
        !renderer
            .accessibility_snapshot()
            .expect("tree")
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Menu)
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_tracks_current_anchors_and_flips_before_overlapping_them() {
    check_placement(1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_tracks_current_anchors_and_flips_before_overlapping_them_msaa() {
    check_placement(4);
}
