//! Native contextual icon button diagrams.
//!
//! ## Usage
//!
//! Verify action targets, live labeling and anchored annotations after resize
//! and scroll.

use std::cell::Cell;

use tessera_components::{scrollable::ScrollableController, theme::material_theme};
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Role, Toggled},
};

use super::*;

thread_local! {
    static SCENE: Cell<usize> = const { Cell::new(0) };
    static SCROLL: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
}

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    let scroll = remember(ScrollableController::new);
    SCROLL.set(Some(scroll));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
            .children(|| {
                spacer().modifier(Modifier::new().height(Dp(32.0)));
                match SCENE.get() {
                    0 => app_bar_figure(),
                    1 => toolbar_figure(),
                    2 => labeling_figure(),
                    _ => density_figure(),
                }
                spacer().modifier(Modifier::new().height(Dp(800.0)));
            });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| {
            panic!(
                "missing native {role:?}: {label}; nodes: {:?}",
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .map(|(_, node)| (node.role(), node.label(), node.bounds()))
                    .collect::<Vec<_>>()
            )
        })
        .1
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = node(renderer, label, Role::Button).bounds().unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(2);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(20);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn contextual_actions_and_annotations_survive_scaled_resize_and_scroll() {
    for scene in 0..4 {
        SCENE.set(scene);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 676,
                height: 520,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(16);
        for width in [676, 390, 1024] {
            renderer.resize(width, 720);
            renderer.render_frames(16);
            SCROLL
                .get()
                .unwrap()
                .with_mut(|scroll| scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -24.0)));
            renderer.render_frames(16);
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
                    if let Ok(directory) = std::env::var("TESSERA_ICON_CONTEXT_OUTPUT") {
                        let path = std::path::Path::new(&directory)
                            .join(format!("wave64-label-selected-{width}.png"));
                        tessera_ui::headless::write_png_file(
                            &path,
                            width,
                            720,
                            &renderer.read_pixels().unwrap(),
                        )
                        .unwrap();
                    }
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
                    let count = before
                        .description()
                        .unwrap()
                        .strip_prefix("Activations: ")
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();
                    click(&mut renderer, label);
                    assert_eq!(
                        node(&renderer, label, Role::Button).description(),
                        Some(format!("Activations: {}", count + 1).as_str())
                    );
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
                    let count = before
                        .description()
                        .unwrap()
                        .strip_prefix("Activations: ")
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();
                    click(&mut renderer, label);
                    assert_eq!(
                        node(&renderer, label, Role::Button).description(),
                        Some(format!("Activations: {}", count + 1).as_str()),
                        "target highlight must not intercept the actual button"
                    );
                }
            }
            if let Ok(directory) = std::env::var("TESSERA_ICON_CONTEXT_OUTPUT") {
                let path = std::path::Path::new(&directory)
                    .join(format!("wave63-context-{scene}-{width}.png"));
                tessera_ui::headless::write_png_file(
                    &path,
                    width,
                    720,
                    &renderer.read_pixels().unwrap(),
                )
                .unwrap();
            }
        }
    }
}
