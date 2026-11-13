use tessera_components::md3::{
    modifier::ModifierExt as _, scrollable::ScrollableController, theme::material_theme,
};
use tessera_ui::{
    MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Role, Toggled},
    testing::UiTest,
};

use super::*;

#[tessera]
fn fixture(
    scene: usize,
    observed_scroll: tessera_ui::testing::TestProbe<Option<State<ScrollableController>>>,
) {
    let scroll = remember(ScrollableController::new);
    observed_scroll.set(Some(scroll));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(32.0)));
                match scene {
                    0 => {
                        app_bar_figure();
                    }
                    1 => {
                        toolbar_figure();
                    }
                    2 => {
                        labeling_figure();
                    }
                    _ => {
                        density_figure();
                    }
                }
                spacer().modifier(Modifier::new().height(Dp(800.0)));
            });
    });
}

fn node(renderer: &UiTest, label: &str, role: Role) -> Node {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    snapshot
        .find(&crate::res::I18N.text(label), role)
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
        .1
        .clone()
}

fn click(renderer: &mut UiTest, label: &str) {
    let bounds = node(renderer, label, Role::Button).bounds().unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(2);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(20);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn contextual_actions_and_annotations_survive_scaled_resize_and_scroll() {
    for scene in 0..4 {
        let scroll = tessera_ui::testing::TestProbe::default();
        let observed = scroll.clone();
        let mut renderer = UiTest::new()
            .viewport(676, 520)
            .rendered(crate::TestPipelines)
            .content(move || fixture(scene, observed.clone()));
        renderer.frames(16);
        for width in [676, 390, 1024] {
            renderer.resize(width, 720);
            renderer.frames(16);
            scroll
                .get()
                .unwrap()
                .with_mut(|scroll| scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -24.0)));
            renderer.frames(16);
            let scale = f64::from(width) / 676.0;
            if scene == 0 || scene == 2 {
                let favorite = node(&renderer, "Add to favorites", Role::Button);
                assert_eq!(favorite.toggled(), Some(Toggled::False));
                click(&mut renderer, "Add to favorites");
                assert_eq!(
                    node(&renderer, "Remove from favorites", Role::Button).toggled(),
                    Some(Toggled::True)
                );
                if scene == 2 {
                    let target = node(&renderer, "Remove from favorites", Role::Button)
                        .bounds()
                        .unwrap();
                    let annotation = node(&renderer, "Icon button label and role", Role::Note)
                        .bounds()
                        .unwrap();
                    assert!(
                        ((target.y0 + target.y1) - (annotation.y0 + annotation.y1)).abs() < 2.0,
                        "label annotation must follow the actual target after resize/scroll"
                    );
                    for label in ["Label", "Role", "Button"] {
                        let text = node(&renderer, label, Role::Label).bounds().unwrap();
                        assert!(
                            text.y0 >= annotation.y0 && text.y1 <= annotation.y1 + 1.0,
                            "annotation text {label} escaped its container: {text:?} {annotation:?}"
                        );
                    }
                }
                click(&mut renderer, "Remove from favorites");
            }
            if scene == 1 {
                for label in [
                    "Archive message",
                    "Delete message",
                    "Mark unread",
                    "Snooze message",
                    "More message options",
                    "Undo message action",
                ] {
                    let before = node(&renderer, label, Role::Button);
                    let description = crate::visible_text(before.description().unwrap());
                    let count = description
                        .strip_prefix("Activations: ")
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();
                    click(&mut renderer, label);
                    assert!(crate::text_matches(
                        node(&renderer, label, Role::Button).description(),
                        &format!("Activations: {}", count + 1)
                    ));
                }
            }
            if scene == 3 {
                for label in [
                    "Delete dinner message",
                    "Archive dinner message",
                    "Mark dinner message unread",
                    "More dinner message options",
                ] {
                    let before = node(&renderer, label, Role::Button);
                    let bounds = before.bounds().unwrap();
                    assert!((bounds.width() - 48.0 * scale).abs() < 2.0);
                    assert!((bounds.height() - 48.0 * scale).abs() < 2.0);
                    let description = crate::visible_text(before.description().unwrap());
                    let count = description
                        .strip_prefix("Activations: ")
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();
                    click(&mut renderer, label);
                    assert!(
                        crate::text_matches(
                            node(&renderer, label, Role::Button).description(),
                            &format!("Activations: {}", count + 1)
                        ),
                        "target highlight must not intercept the actual button"
                    );
                }
            }
            if scene == 2 || scene == 3 {
                let pixels = renderer.read_pixels();
                let y = 8.0 + (if scene == 2 { 250.0 } else { 280.0 }) * scale;
                let sample = |x: f64| {
                    let offset =
                        ((y.floor() as usize * width as usize) + (x * scale).floor() as usize) * 4;
                    pixels[offset..offset + 4].to_vec()
                };
                assert_eq!(
                    sample(if scene == 2 { 470.0 } else { 300.0 }),
                    sample(650.0),
                    "cropped phone content must not paint outside the device: scene={scene}, width={width}"
                );
            }
        }
    }
}
