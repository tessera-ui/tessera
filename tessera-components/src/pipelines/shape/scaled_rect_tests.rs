//! Nonexpanding anisotropic shape rendering.
//!
//! ## Usage
//!
//! Verify native shape coverage, clipping, and retained input targets.

use std::time::Duration;

use tessera_ui::{
    Callback, Color, Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PlacementModifierNode, PressKeyEventType, PxPosition, State,
    modifier::ModifierCapabilityExt as _, remember, scaled_box, tessera, testing::TestProbe,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::{ClickableArgs, ModifierExt as _},
    shape_def::Shape,
    theme::material_theme,
};

use super::{ShapeCommand, rect_to_uniforms};

#[derive(Clone, Copy, Default, PartialEq)]
struct FixtureHandles {
    scaled: Option<State<bool>>,
    clicks: Option<State<usize>>,
}

#[derive(Clone, Copy, PartialEq)]
struct Origin;

impl PlacementModifierNode for Origin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

#[tessera]
fn fixture(scale: [f32; 2], outer: f32, clipped: bool, probe: TestProbe<FixtureHandles>) {
    let scaled = remember(|| false);
    let clicks = remember(|| 0);
    probe.set(FixtureHandles {
        scaled: Some(scaled),
        clicks: Some(clicks),
    });
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
            .children(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(Modifier::new().push_placement(Origin))
                    .children(move || {
                        scaled_box(outer).child(move || {
                            let mut modifier = Modifier::new().size(Dp(200.0), Dp(80.0));
                            if clipped {
                                modifier = Modifier::new()
                                    .size(Dp(100.0), Dp(40.0))
                                    .clip_shape(Shape::rounded_rectangle(Dp(8.0)));
                            }
                            boxed()
                                .alignment(Alignment::TopStart)
                                .modifier(modifier)
                                .children(move || {
                                    let background = if scaled.get() {
                                        Modifier::new().background_with_scaled_shape(
                                            Color::RED,
                                            Shape::CAPSULE,
                                            scale,
                                        )
                                    } else {
                                        Modifier::new()
                                            .background_with_shape(Color::RED, Shape::CAPSULE)
                                    };
                                    boxed()
                                        .modifier(
                                            background
                                                .required_size(Dp(200.0), Dp(80.0))
                                                .clickable_with(ClickableArgs {
                                                    role: Some(tessera_ui::accesskit::Role::Button),
                                                    label: Some("Scaled background target".into()),
                                                    on_click: Callback::new(move || {
                                                        clicks.with_mut(|value| *value += 1)
                                                    }),
                                                    ..Default::default()
                                                }),
                                        )
                                        .children(|| {});
                                });
                        });
                    });
            });
    });
}

fn create_renderer(
    samples: u32,
    scale: [f32; 2],
    outer: f32,
    clipped: bool,
) -> (HeadlessRenderer, FixtureHandles) {
    let probe = TestProbe::default();
    let observed = probe.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || {
            fixture(scale, outer, clipped, probe.clone());
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 240,
            height: 140,
            sample_count: samples,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    (renderer, observed.get())
}

fn pixel(renderer: &HeadlessRenderer, x: usize, y: usize) -> [u8; 3] {
    let pixels = renderer.read_pixels().unwrap();
    let offset = (y * 240 + x) * 4;
    pixels[offset..offset + 3].try_into().unwrap()
}

#[test]
#[ignore = "requires a native GPU adapter for actual anisotropic SDF pixels and pointer clipping"]
fn scaled_rect_identity_elliptical_caps_zero_and_fractional_clip() {
    for samples in [1, 4] {
        let mut scale = [1.0, 1.0];
        let mut outer = 1.0;
        let (mut renderer, handles) = create_renderer(samples, scale, outer, false);
        let idle = renderer.diagnostics();
        assert_eq!(
            renderer
                .work()
                .phase(tessera_ui::WorkPhase::Measure)
                .count(),
            0,
            "idle frame must not measure"
        );

        assert_eq!(idle.dirty_nodes_param, 0);
        assert_eq!(idle.geometry_nodes_updated, 0);
        let original = renderer.read_pixels().unwrap();
        let bounds = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some("Scaled background target"))
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
            (10.25, 20.5, 200.0, 80.0)
        );
        handles.scaled.unwrap().set(true);
        renderer.render_frames(1);
        let diagnostics = renderer.diagnostics();
        assert_eq!(
            renderer
                .work()
                .phase(tessera_ui::WorkPhase::Measure)
                .count(),
            0,
            "paint-only scaling must not measure"
        );

        assert_eq!(
            diagnostics.dirty_nodes_param, 0,
            "paint-only scaling must not invalidate layout inputs"
        );

        assert_eq!(
            diagnostics.geometry_nodes_updated, 0,
            "paint-only scaling must not place nodes"
        );
        renderer.render_frames(1);
        assert_eq!(
            renderer.read_pixels().unwrap(),
            original,
            "identity must preserve existing coverage exactly"
        );
        drop(renderer);
        scale = [0.32, 1.0];
        let (mut renderer, handles) = create_renderer(samples, scale, outer, false);
        handles.scaled.unwrap().set(true);
        renderer.render_frames(2);
        assert_eq!(
            pixel(&renderer, 135, 90),
            [255, 0, 0],
            "elliptical cap interior outside an ordinary narrowed capsule"
        );
        assert_eq!(
            pixel(&renderer, 77, 60),
            [255, 255, 255],
            "horizontal contraction stays centered"
        );
        assert_eq!(
            pixel(&renderer, 143, 60),
            [255, 255, 255],
            "contraction never expands draw bounds"
        );
        assert_eq!(pixel(&renderer, 30, 60), [255, 255, 255]);
        let unpainted = PxPosition::from_f32_xy(30.5, 60.5);
        renderer.pointer_press(MOUSE_POINTER_ID, unpainted, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, unpainted, PressKeyEventType::Left);
        renderer.render_frames(2);
        assert_eq!(
            handles.clicks.unwrap().get(),
            1,
            "background paint contraction must preserve the full native interaction target"
        );
        drop(renderer);
        scale = [1.0, 0.5];
        let (mut renderer, handles) = create_renderer(samples, scale, outer, false);
        handles.scaled.unwrap().set(true);
        renderer.render_frames(2);
        assert_eq!(
            pixel(&renderer, 20, 45),
            [255, 255, 255],
            "vertical contraction preserves elliptical rather than circular ends"
        );
        assert_eq!(pixel(&renderer, 35, 45), [255, 0, 0]);
        assert_eq!(pixel(&renderer, 110, 39), [255, 255, 255]);
        drop(renderer);
        scale = [0.0, 1.0];
        let (mut renderer, handles) = create_renderer(samples, scale, outer, false);
        handles.scaled.unwrap().set(true);
        renderer.render_frames(2);
        assert!(
            renderer
                .read_pixels()
                .unwrap()
                .chunks_exact(4)
                .all(|pixel| pixel == [255, 255, 255, 255]),
            "zero scale produces no pixels"
        );
        drop(renderer);
        scale = [0.32, 1.0];
        outer = 0.625;
        for clipped in [false, true] {
            let (mut renderer, handles) = create_renderer(samples, scale, outer, clipped);
            handles.scaled.unwrap().set(true);
            renderer.render_frames(2);
            assert_eq!(pixel(&renderer, 65, 40), [255, 0, 0]);
            assert_eq!(
                pixel(&renderer, 80, 40),
                if clipped {
                    [255, 255, 255]
                } else {
                    [255, 0, 0]
                }
            );
            let position = PxPosition::from_f32_xy(80.5, 40.5);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.render_frames(2);
            assert_eq!(handles.clicks.unwrap().get(), usize::from(!clipped));
            drop(renderer);
        }
    }
}

#[test]
fn scaled_rect_rejects_nonfinite_negative_and_expanding_scales() {
    for scale in [
        [f32::NAN, 1.0],
        [f32::INFINITY, 1.0],
        [-0.1, 1.0],
        [1.01, 1.0],
    ] {
        let result = std::panic::catch_unwind(|| {
            rect_to_uniforms(
                &ShapeCommand::ScaledRect {
                    color: Color::RED,
                    corner_radii: [4.0; 4],
                    corner_g2: [2.0; 4],
                    scale,
                },
                tessera_ui::PxSize::new(tessera_ui::Px(20), tessera_ui::Px(20)),
                PxPosition::ZERO,
            )
        });
        assert!(
            result.is_err(),
            "invalid scale {scale:?} must fail its own guard"
        );
    }
}
