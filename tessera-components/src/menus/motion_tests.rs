use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID,
    Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Rect, Role},
    remember, tessera,
};

use crate::{
    modifier::ModifierExt as _,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, material_theme},
};

use super::{MenuAnchor, MenuController, MenuPosition, menu_group, menu_item, menu_provider};

thread_local! {
    static ROOT: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static ACTIONS: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static FIXED: Cell<Option<State<Option<Shape>>>> = const { Cell::new(None) };
    static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let root = remember(MenuController::new);
    let actions = remember(|| 0);
    let fixed = remember(|| None);
    let enabled = remember(|| true);
    ROOT.set(Some(root));
    ACTIONS.set(Some(actions));
    FIXED.set(Some(fixed));
    ENABLED.set(Some(enabled));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface_container_low = Color::new(0.08, 0.16, 0.24, 1.0);
            theme
        })
        .child(move || {
            menu_provider()
                .controller(root)
                .grouped(true)
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
                    menu_group()
                        .position(MenuPosition::First)
                        .shape_optional(fixed.get())
                        .accessibility_label("First group")
                        .children(move || {
                            menu_item()
                                .label("Share")
                                .enabled(enabled.get())
                                .submenu_content(move || {
                                    menu_item()
                                        .label("Copy")
                                        .on_click(move || actions.set(actions.get() + 1));
                                    menu_item().label("More").submenu_content(move || {
                                        menu_item()
                                            .label("Export")
                                            .on_click(move || actions.set(actions.get() + 10));
                                    });
                                });
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .accessibility_label("Last group")
                        .children(move || {
                            menu_item()
                                .label("Other")
                                .on_click(move || actions.set(actions.get() + 100));
                        });
                });
        });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Option<Rect> {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() != Role::Label)
        .and_then(|(_, node)| node.bounds())
}

fn point(renderer: &HeadlessRenderer, label: &str) -> PxPosition {
    let bounds = bounds(renderer, label).unwrap_or_else(|| panic!("missing {label}"));
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn hover(renderer: &mut HeadlessRenderer, label: &str) {
    renderer.pointer_move(MOUSE_POINTER_ID, point(renderer, label));
    renderer.step_frame();
}

fn focus(renderer: &HeadlessRenderer, label: &str) {
    let tree = renderer.accessibility_snapshot().expect("tree");
    assert_eq!(
        tree.nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .expect("focus")
            .1
            .label(),
        Some(label)
    );
}

fn corner(renderer: &mut HeadlessRenderer, bounds: Rect) -> u8 {
    let width = renderer.physical_size().width as usize;
    let x = (bounds.x0 + 6.0) as usize;
    let y = (bounds.y0 + 6.0) as usize;
    renderer.read_pixels().expect("pixels")[(y * width + x) * 4]
}

fn check_motion(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 1500,
            height: 1000,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let background = renderer.read_pixels().expect("background");
    ROOT.get().expect("root").with_mut(|root| {
        root.open_at(MenuAnchor::from_dp(
            (Dp(80.0), Dp(80.0)),
            (Dp(20.0), Dp(20.0)),
        ))
    });
    renderer.render_frames(4);
    let first = bounds(&renderer, "First group").expect("first group");
    assert!(
        corner(&mut renderer, first) > 200,
        "resting 16dp corner leaves the sample outside the fill"
    );
    hover(&mut renderer, "Other");
    focus(&renderer, "Other");
    renderer.render_frames(60);
    assert!(
        corner(&mut renderer, first) < 150,
        "previous group contracts to 8dp corners"
    );
    assert_eq!(
        bounds(&renderer, "First group"),
        Some(first),
        "shape changes preserve group geometry"
    );
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(1450.0, 950.0));
    renderer.step_frame();
    assert!(FocusManager::current().move_focus(FocusDirection::Up));
    renderer.render_frames(60);
    focus(&renderer, "Share");
    assert!(
        corner(&mut renderer, first) > 200,
        "keyboard focus restores the resting shape"
    );

    hover(&mut renderer, "Share");
    renderer.render_frames(3);
    assert!(
        bounds(&renderer, "Copy").is_none(),
        "short pointer crossings do not open a branch"
    );
    assert!(FocusManager::current().move_focus(FocusDirection::Down));
    renderer.render_frames(20);
    focus(&renderer, "Other");
    assert!(
        bounds(&renderer, "Copy").is_none(),
        "keyboard navigation cancels pending pointer intent"
    );
    hover(&mut renderer, "Share");
    renderer.render_frames(3);
    ENABLED.get().expect("enabled").set(false);
    renderer.render_frames(20);
    assert!(bounds(&renderer, "Copy").is_none());
    ENABLED.get().expect("enabled").set(true);
    renderer.render_frames(20);
    assert!(
        bounds(&renderer, "Copy").is_none(),
        "reenabling a stationary source cannot replay an old hover intent"
    );
    hover(&mut renderer, "Share");
    renderer.render_frames(12);
    assert!(
        bounds(&renderer, "Copy").is_some(),
        "dwell opens the submenu"
    );
    hover(&mut renderer, "More");
    renderer.render_frames(3);
    hover(&mut renderer, "Copy");
    renderer.render_frames(20);
    assert!(
        bounds(&renderer, "Export").is_none(),
        "leaving a source cancels its pending open"
    );
    hover(&mut renderer, "More");
    renderer.render_frames(12);
    assert!(bounds(&renderer, "Export").is_some());
    let menus = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.role() == Role::Menu)
        .map(|(_, node)| node.bounds().expect("menu bounds"))
        .collect::<Vec<_>>();
    let child = menus[1];
    hover(&mut renderer, "Export");
    renderer.render_frames(60);
    focus(&renderer, "Export");
    assert!(
        corner(&mut renderer, child) < 150,
        "inactive child panel contracts when moving to its descendant"
    );
    let settled = renderer.read_pixels().expect("settled");
    renderer.render_frames(30);
    assert!(renderer.read_pixels().expect("idle") == settled);

    renderer.resize(585, 1000);
    renderer.render_frames(8);
    hover(&mut renderer, "Export");
    renderer.render_frames(20);
    focus(&renderer, "Export");
    assert!(
        bounds(&renderer, "Copy").is_some(),
        "covered parent items cannot close the active overlay"
    );
    renderer.resize(1500, 1000);
    renderer.render_frames(8);
    hover(&mut renderer, "Other");
    renderer.render_frames(3);
    hover(&mut renderer, "Export");
    renderer.render_frames(20);
    assert!(
        bounds(&renderer, "Export").is_some(),
        "crossing an ancestor item on the way to a child keeps the branch"
    );
    hover(&mut renderer, "Other");
    renderer.render_frames(12);
    assert!(
        bounds(&renderer, "Copy").is_none(),
        "dwelling on a leaf closes obsolete descendants"
    );
    focus(&renderer, "Other");
    renderer.pointer_move(MOUSE_POINTER_ID + 1, point(&renderer, "Share"));
    renderer.render_frames(20);
    focus(&renderer, "Other");
    assert!(
        bounds(&renderer, "Copy").is_none(),
        "touch motion does not create hover intent"
    );

    FIXED
        .get()
        .expect("fixed")
        .set(Some(Shape::rounded_rectangle(Dp(24.0))));
    renderer.render_frames(60);
    hover(&mut renderer, "Share");
    renderer.render_frames(3);
    hover(&mut renderer, "Other");
    renderer.render_frames(60);
    assert!(
        corner(&mut renderer, first) > 200,
        "an explicit shape remains fixed after leaving the group"
    );
    FIXED.get().expect("fixed").set(None);
    renderer.render_frames(60);
    assert!(corner(&mut renderer, first) < 150);
    assert_eq!(
        ACTIONS.get().expect("actions").get(),
        0,
        "hover never invokes an action"
    );
    hover(&mut renderer, "Share");
    renderer.render_frames(12);
    let copy = point(&renderer, "Copy");
    renderer.pointer_move(MOUSE_POINTER_ID, copy);
    renderer.pointer_press(MOUSE_POINTER_ID, copy, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, copy, PressKeyEventType::Left);
    renderer.render_frames(90);
    assert_eq!(ACTIONS.get().expect("actions").get(), 1);
    assert!(renderer.read_pixels().expect("dismissed") == background);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_hover_focus_branch_intent_and_container_motion() {
    check_motion(1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_hover_focus_branch_intent_and_container_motion_msaa() {
    check_motion(4);
}
