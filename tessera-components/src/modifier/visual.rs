//! Visual modifiers for opacity, clipping, backgrounds, and borders.
//!
//! ## Usage
//!
//! Apply basic visual effects like alpha, clipping, and shape borders.

use tessera_ui::{Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, PxSize};

use crate::{
    pipelines::shape::command::ShapeCommand,
    shape_def::{ResolvedShape, Shape},
};

fn shape_background_command(color: Color, shape: Shape, size: PxSize) -> ShapeCommand {
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::Rect {
            color,
            corner_radii,
            corner_g2,
        },
        ResolvedShape::Ellipse => ShapeCommand::Ellipse { color },
    }
}

fn shape_border_command(color: Color, width: Dp, shape: Shape, size: PxSize) -> ShapeCommand {
    let border_width = width.to_pixels_f32();
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::OutlinedRect {
            color,
            corner_radii,
            corner_g2,
            border_width,
        },
        ResolvedShape::Ellipse => ShapeCommand::OutlinedEllipse {
            color,
            border_width,
        },
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AlphaModifierNode {
    pub alpha: f32,
}

impl DrawModifierNode for AlphaModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        {
            let mut metadata = ctx.render_input.metadata_mut();
            metadata.multiply_opacity(self.alpha);
        }
        content.draw(ctx.render_input);
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ClipModifierNode;

#[derive(Clone, Copy)]
pub(crate) struct ShapeClipModifierNode {
    pub shape: Shape,
}

impl DrawModifierNode for ShapeClipModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("shape clip must have a measured size");
        let shape = match self.shape.resolve_for_size(size.into()) {
            ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } => tessera_ui::ClipShape::RoundedRectangle {
                radii: corner_radii,
                exponents: corner_g2,
            },
            ResolvedShape::Ellipse => tessera_ui::ClipShape::Ellipse,
        };
        metadata.set_clip_shape(shape);
        content.draw(ctx.render_input);
    }
}

impl DrawModifierNode for ClipModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        ctx.render_input.metadata_mut().set_clips_children(true);
        content.draw(ctx.render_input);
    }
}

#[derive(Clone)]
pub(crate) struct BackgroundModifierNode {
    pub color: Color,
    pub shape: Shape,
}

impl DrawModifierNode for BackgroundModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        {
            let mut metadata = ctx.render_input.metadata_mut();
            let size = metadata
                .computed_data()
                .expect("background modifier must have computed size before record");
            metadata
                .fragment_mut()
                .push_draw_command(shape_background_command(
                    self.color,
                    self.shape,
                    size.into(),
                ));
        }
        content.draw(ctx.render_input);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ScaledBackgroundModifierNode {
    pub color: Color,
    pub shape: Shape,
    pub scale: [f32; 2],
}

impl DrawModifierNode for ScaledBackgroundModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        {
            let mut metadata = ctx.render_input.metadata_mut();
            let size = metadata
                .computed_data()
                .expect("measured scaled background");
            let (corner_radii, corner_g2) = match self.shape.resolve_for_size(size.into()) {
                ResolvedShape::Rounded {
                    corner_radii,
                    corner_g2,
                } => (corner_radii, corner_g2),
                ResolvedShape::Ellipse => ([-1.0; 4], [0.0; 4]),
            };
            metadata
                .fragment_mut()
                .push_draw_command(ShapeCommand::ScaledRect {
                    color: self.color,
                    corner_radii,
                    corner_g2,
                    scale: self.scale,
                });
        }
        content.draw(ctx.render_input);
    }
}

#[derive(Clone)]
pub(crate) struct BorderModifierNode {
    pub width: Dp,
    pub color: Color,
    pub shape: Shape,
}

impl DrawModifierNode for BorderModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        content.draw(ctx.render_input);
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("border modifier must have computed size before record");
        metadata
            .fragment_mut()
            .push_draw_command(shape_border_command(
                self.color,
                self.width,
                self.shape,
                size.into(),
            ));
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        Modifier, State,
        headless::{HeadlessConfig, HeadlessRenderer},
        remember, tessera,
    };

    use crate::{
        boxed::boxed, modifier::ModifierExt as _, surface::surface, theme::material_theme,
    };

    use super::*;

    thread_local! {
        static OPACITY: Cell<Option<State<f32>>> = const { Cell::new(None) };
        static CLIPPED: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static SHAPE_STAGE: Cell<Option<State<u8>>> = const { Cell::new(None) };
        static SHAPE_CLICKS: Cell<Option<State<usize>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn visual_fixture() {
        let opacity = remember(|| 0.5f32);
        let clipped = remember(|| false);
        OPACITY.set(Some(opacity));
        CLIPPED.set(Some(clipped));
        material_theme().child(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .children(move || {
                    let mut modifier = Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .alpha(opacity.get());
                    if clipped.get() {
                        modifier = modifier.clip_to_bounds();
                    }
                    boxed().modifier(modifier).children(|| {
                        surface()
                            .style(Color::new(1.0, 0.0, 0.0, 1.0).into())
                            .modifier(
                                Modifier::new()
                                    .size(Dp(40.0), Dp(40.0))
                                    .offset(Dp(40.0), Dp(0.0)),
                            )
                            .child(|| {});
                    });
                });
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn cached_layout_rebuilds_opacity_and_clipping_for_each_record_pass() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                visual_fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 160,
                height: 100,
                ..Default::default()
            },
        )
        .expect("headless GPU renderer");
        renderer.step_frame();
        let half_opacity = renderer.read_pixels().expect("half-opacity pixels");
        renderer.render_frames(3);
        let repeated = renderer.read_pixels().expect("repeated pixels");
        assert!(
            half_opacity == repeated,
            "constant alpha must not accumulate across cached frames"
        );

        OPACITY.get().expect("fixture state").set(0.0);
        renderer.render_frames(3);
        let invisible = renderer.read_pixels().expect("invisible pixels");
        assert!(invisible != half_opacity);
        OPACITY.get().expect("fixture state").set(1.0);
        renderer.render_frames(3);
        let visible = renderer.read_pixels().expect("visible pixels");
        assert!(
            visible != invisible,
            "an alpha-zero subtree must become visible again"
        );
        assert!(visible != half_opacity);

        CLIPPED.get().expect("fixture state").set(true);
        renderer.render_frames(3);
        let clipped = renderer.read_pixels().expect("clipped pixels");
        assert!(
            clipped == invisible,
            "the offset child must be clipped outside the container"
        );
        CLIPPED.get().expect("fixture state").set(false);
        renderer.render_frames(3);
        let unclipped = renderer.read_pixels().expect("unclipped pixels");
        assert!(
            unclipped == visible,
            "removed clipping must not persist on a layout cache hit"
        );
    }

    #[tessera]
    fn shape_fixture() {
        let stage = remember(|| 0u8);
        let clicks = remember(|| 0usize);
        let raster = remember(|| crate::image::ImageData::from_rgba(vec![0, 0, 255, 255], 1, 1));
        SHAPE_STAGE.set(Some(stage));
        SHAPE_CLICKS.set(Some(clicks));
        material_theme().child(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .children(move || {
                    let mut modifier = Modifier::new()
                        .size(Dp(100.0), Dp(80.0))
                        .offset(Dp(10.25), Dp(10.75));
                    if stage.get() != 2 {
                        modifier = modifier.clip_shape(Shape::Ellipse);
                    }
                    boxed()
                        .modifier(modifier.background(Color::RED))
                        .children(move || {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .size(Dp(80.0), Dp(80.0))
                                        .offset(Dp(60.0), Dp(0.0))
                                        .clip_shape(Shape::rounded_rectangle(Dp(20.0))),
                                )
                                .children(move || {
                                    if stage.get() == 3 {
                                        crate::fluid_glass::fluid_glass()
                                            .blur_radius(Dp(4.0))
                                            .contrast(1.1)
                                            .tint_color(Color::BLUE)
                                            .shape(Shape::RECTANGLE)
                                            .modifier(Modifier::new().fill_max_size())
                                            .child(|| {});
                                    } else if stage.get() == 1 {
                                        crate::image::image()
                                            .raster(raster.get())
                                            .modifier(Modifier::new().fill_max_size());
                                    } else {
                                        surface()
                                            .style(Color::BLUE.into())
                                            .elevation(Dp(3.0))
                                            .modifier(Modifier::new().fill_max_size())
                                            .child(|| {});
                                    }
                                    crate::text::text()
                                        .content("Mask")
                                        .color(Color::BLACK)
                                        .modifier(Modifier::new().offset(Dp(0.0), Dp(40.0)));
                                });
                        });
                    boxed().modifier(
                        Modifier::new()
                            .size(Dp(20.0), Dp(20.0))
                            .offset(Dp(160.0), Dp(10.0))
                            .background(Color::GREEN),
                    );
                    boxed().modifier(
                        Modifier::new()
                            .size(Dp(20.0), Dp(20.0))
                            .offset(Dp(-19.75), Dp(200.25))
                            .background(Color::RED),
                    );
                    boxed()
                        .modifier(
                            Modifier::new()
                                .size(Dp(100.0), Dp(80.0))
                                .offset(Dp(10.0), Dp(110.0))
                                .clip_shape(Shape::Ellipse),
                        )
                        .children(move || {
                            crate::button::button()
                                .modifier(Modifier::new().fill_max_size())
                                .shape(Shape::RECTANGLE)
                                .on_click(move || clicks.with_mut(|value| *value += 1))
                                .child(|| {});
                        });
                });
        });
    }

    fn check_shape_clipping(samples: u32) {
        use tessera_ui::{MOUSE_POINTER_ID, PressKeyEventType, PxPosition};

        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                shape_fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 200,
                height: 220,
                sample_count: samples,
                ..Default::default()
            },
        )
        .expect("shape clipping renderer");
        renderer.render_frames(8);
        let first = renderer.read_pixels().expect("masked pixels");
        let pixel = |pixels: &[u8], x: usize, y: usize| {
            pixels[(y * 200 + x) * 4..(y * 200 + x) * 4 + 4].to_vec()
        };
        assert_eq!(pixel(&first, 11, 11), [255, 255, 255, 255]);
        assert_eq!(pixel(&first, 60, 25), [255, 0, 0, 255]);
        assert_eq!(pixel(&first, 95, 30), [0, 0, 255, 255]);
        assert_eq!(pixel(&first, 130, 30), [255, 255, 255, 255]);
        assert_eq!(pixel(&first, 165, 15), [0, 255, 0, 255]);
        let coverage_pixels = (11..88)
            .flat_map(|y| (11..55).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let value = pixel(&first, x, y);
                value[0] == 255 && value[1] > 0 && value[1] < 255 && value[1] == value[2]
            })
            .count();
        if samples == 4 {
            assert!(
                pixel(&first, 0, 205)[1] < 255,
                "culling must preserve subpixel coverage at the viewport edge"
            );
            assert!(
                pixel(&first, 60, 10)[1] < 255,
                "fractional draw scissors must preserve the mask's top coverage"
            );
            assert!(
                coverage_pixels > 0,
                "multisample shape edges must retain partial coverage"
            );
        }
        SHAPE_STAGE.get().expect("stage").set(1);
        renderer.render_frames(6);
        let raster = renderer.read_pixels().expect("masked raster pixels");
        assert_eq!(pixel(&raster, 95, 30), [0, 0, 255, 255]);
        assert_eq!(pixel(&raster, 130, 30), [255, 255, 255, 255]);
        SHAPE_STAGE.get().expect("stage").set(3);
        renderer.render_frames(6);
        let sampled = renderer.read_pixels().expect("masked scene sampling");
        assert_eq!(pixel(&sampled, 11, 11), [255, 255, 255, 255]);
        assert_eq!(pixel(&sampled, 130, 30), [255, 255, 255, 255]);
        assert_ne!(pixel(&sampled, 95, 30), [255, 255, 255, 255]);
        if let Some(directory) = std::env::var_os("TESSERA_SHAPE_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).expect("shape QA directory");
            for (name, pixels) in [("nested", &first), ("sampled", &sampled)] {
                tessera_ui::headless::write_png_file(
                    directory.join(format!("shape-clip-{name}-msaa{samples}.png")),
                    200,
                    220,
                    pixels,
                )
                .expect("shape QA image");
            }
        }
        SHAPE_STAGE.get().expect("stage").set(2);
        renderer.render_frames(6);
        assert_eq!(
            pixel(&renderer.read_pixels().expect("unmasked pixels"), 11, 11),
            [255, 0, 0, 255]
        );
        SHAPE_STAGE.get().expect("stage").set(0);
        renderer.render_frames(6);
        assert!(
            renderer.read_pixels().expect("restored mask") == first,
            "cached masks must restore exactly"
        );

        let clicks = SHAPE_CLICKS.get().expect("click state");
        for (x, y, expected) in [(11.0, 111.0, 0), (60.0, 150.0, 1)] {
            let point = PxPosition::from_f32_xy(x, y);
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.step_frame();
            renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(3);
            assert_eq!(clicks.get(), expected, "clipped pointer target at {x},{y}");
        }
        renderer.pointer_press(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(60.0, 150.0),
            PressKeyEventType::Left,
        );
        renderer.step_frame();
        let corner = PxPosition::from_f32_xy(11.0, 111.0);
        renderer.pointer_move(MOUSE_POINTER_ID, corner);
        renderer.pointer_release(MOUSE_POINTER_ID, corner, PressKeyEventType::Left);
        renderer.render_frames(3);
        assert_eq!(
            clicks.get(),
            1,
            "releasing outside the mask cancels activation"
        );
    }

    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn shape_clips_survive_nested_masks_barriers_and_input_at_one_sample() {
        check_shape_clipping(1);
    }

    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn shape_clips_survive_nested_masks_barriers_and_input_at_four_samples() {
        check_shape_clipping(4);
    }
}
