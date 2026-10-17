use std::cell::Cell;

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Rect, Role},
    remember, tessera,
};

use crate::{
    modifier::ModifierExt as _,
    surface::{SurfaceStyle, surface},
    theme::material_theme,
};

use super::{MenuAnchor, MenuController, MenuPosition, menu_group, menu_item, menu_provider};

thread_local! {
    static CONTROLLER: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static GROUPED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static LONG: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static ACTIONS: Cell<Option<State<u32>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let controller = remember(MenuController::new);
    let grouped = remember(|| true);
    let long = remember(|| false);
    let actions = remember(|| 0);
    CONTROLLER.set(Some(controller));
    GROUPED.set(Some(grouped));
    LONG.set(Some(long));
    ACTIONS.set(Some(actions));
    material_theme().child(move || {
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
            .menu_content(move || {
                if grouped.get() {
                    menu_group()
                        .position(MenuPosition::First)
                        .accessibility_label("First group")
                        .children(move || {
                            menu_item()
                                .label("Share")
                                .on_click(move || actions.set(actions.get() + 1));
                            if long.get() {
                                for index in 0..12 {
                                    menu_item()
                                        .label(format!("Extra action {index}"))
                                        .on_click(move || actions.set(actions.get() + 1));
                                }
                            }
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .accessibility_label("Last group")
                        .children(move || {
                            menu_item()
                                .label("Delete")
                                .on_click(move || actions.set(actions.get() + 1));
                        });
                } else {
                    menu_item().label("Copy link").on_click(|| {});
                }
            });
    });
}

fn bounds(renderer: &HeadlessRenderer, role: Role, label: Option<&str>) -> Rect {
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
    renderer.render_frames(4);
    let background = renderer.read_pixels().expect("background");
    let controller = CONTROLLER.get().expect("controller");
    let grouped = GROUPED.get().expect("grouped");
    for is_grouped in [true, false] {
        grouped.set(is_grouped);
        controller.with_mut(|menu| {
            menu.open_at(MenuAnchor::from_dp(
                (Dp(80.0), Dp(60.0)),
                (Dp(80.0), Dp(40.0)),
            ));
        });
        renderer.render_frames(6);
        let pixels = renderer.read_pixels().expect("shadows");
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
        renderer.render_frames(8);
        assert_eq!(renderer.read_pixels().expect("cached shadows"), pixels);
        controller.with_mut(MenuController::close);
        renderer.render_frames(4);
        assert_eq!(renderer.read_pixels().expect("dismissed"), background);
    }

    grouped.set(true);
    LONG.get().expect("long").set(true);
    controller.with_mut(|menu| {
        menu.open_at(MenuAnchor::from_dp(
            (Dp(80.0), Dp(60.0)),
            (Dp(80.0), Dp(40.0)),
        ));
    });
    renderer.render_frames(6);
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
    renderer.render_frames(90);
    let last = bounds(&renderer, Role::Group, Some("Last group"));
    assert!(
        last.y0 >= panel.y0 && last.y1 <= panel.y1 + 1.0,
        "last group must scroll into view: {last:?}, {panel:?}"
    );
    let pixels = renderer.read_pixels().expect("scrolled");
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
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.render_frames(4);
    assert!(
        !controller.with(MenuController::is_open),
        "shadow pixels do not enlarge the menu's click target"
    );
    assert_eq!(ACTIONS.get().expect("actions").get(), 0);
    assert_eq!(
        renderer.read_pixels().expect("dismissed after scrolling"),
        background
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_shadows_extend_beyond_group_and_panel_edges() {
    check_shadows(1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_shadows_extend_beyond_group_and_panel_edges_msaa() {
    check_shadows(4);
}
