//! Native render-target precision and linear scrim regressions.
//!
//! ## Usage
//!
//! Verify sRGB readback bytes across output formats and multisampling modes.

use std::time::Duration;

use tessera_components::{
    boxed::boxed,
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_rail::{navigation_rail, navigation_rail_item},
    shape_def::Shape,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    surface::surface,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier,
    accesskit::{Node, NodeId, Rect, Role},
    tessera, use_context,
    wgpu::TextureFormat,
};

const ACCENT: Color = Color::from_rgb(0.1356333, 0.08021982, 0.3712377);
const DARK_SURFACE: [u8; 4] = [15, 13, 18, 255];
const WHITE_UNDER_LINEAR_SCRIM: [u8; 4] = [215, 215, 215, 255];

fn observed_region(label: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        role: Some(Role::Group),
        label: Some(label.to_owned()),
        merge_descendants: true,
        ..Default::default()
    })
}

#[tessera]
fn dark_surface_fixture() {
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            let surface_color = use_context::<MaterialTheme>()
                .unwrap()
                .get()
                .color_scheme
                .surface;
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(Color::WHITE)
                        .then(observed_region("Precision dark viewport")),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(16.0), Dp(16.0))
                                .size(Dp(64.0), Dp(64.0))
                                .background_with_shape(
                                    surface_color,
                                    Shape::rounded_rectangle(Dp::ZERO),
                                )
                                .then(observed_region("Precision shape patch")),
                        )
                        .children(|| {});
                    surface()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(16.0), Dp(112.0))
                                .size(Dp(64.0), Dp(64.0))
                                .then(observed_region("Precision surface patch")),
                        )
                        .shape(Shape::rounded_rectangle(Dp::ZERO))
                        .elevation(Dp::ZERO)
                        .style(surface_color.into())
                        .child(|| {});
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(112.0), Dp(16.0))
                                .size(Dp(96.0), Dp(560.0))
                                .then(observed_region("Precision rail wrapper")),
                        )
                        .children(|| {
                            navigation_rail()
                                .expanded(false)
                                .content(move |rail_scope| {
                                    navigation_rail_item(rail_scope)
                                        .label("Home")
                                        .selected(true)
                                        .icon(|| {
                                            spacer()
                                                .modifier(Modifier::new().size(Dp(24.0), Dp(24.0)));
                                        });
                                });
                        });
                });
        });
}

#[tessera]
fn white_scrim_fixture() {
    material_theme().child(|| {
        modal_side_sheet_provider()
            .accessibility_label("Precision scrim sheet")
            .position(SideSheetPosition::Start)
            .max_width(Dp(240.0))
            .content_padding(Dp::ZERO)
            .is_open(true)
            .main_content(|| {
                boxed()
                    .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                    .children(|| {});
            })
            .side_sheet_content(|| {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(|| {});
            });
    });
}

fn render<Output: 'static>(
    root: fn() -> Output,
    width: u32,
    height: u32,
    format: TextureFormat,
    sample_count: u32,
) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(root),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width,
            height,
            format,
            sample_count,
            scale_factor: 1.0,
            frame_time: Duration::from_millis(100),
        },
    )
    .unwrap_or_else(|error| panic!("GPU session for {format:?}, MSAA {sample_count}: {error}"));
    // Two virtual seconds exceed the native visibility and rail motion bounds.
    renderer.render_frames(21);
    renderer
}

fn bounds(nodes: &[(NodeId, Node)], label: &str, role: Role) -> Rect {
    nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label) && !node.is_hidden())
        .unwrap_or_else(|| panic!("missing actual geometry for {label}"))
        .1
        .bounds()
        .unwrap()
}

fn assert_opaque_roi(
    pixels: &[u8],
    width: usize,
    center: (usize, usize),
    expected: [u8; 4],
    label: &str,
    format: TextureFormat,
    sample_count: u32,
) {
    for y in center.1 - 1..=center.1 + 1 {
        for x in center.0 - 1..=center.0 + 1 {
            let offset = (y * width + x) * 4;
            let actual: [u8; 4] = pixels[offset..offset + 4].try_into().unwrap();
            assert_eq!(
                actual, expected,
                "{label} at ({x}, {y}), output {format:?}, MSAA {sample_count}"
            );
        }
    }
}

fn check_dark_surface(format: TextureFormat, sample_count: u32) {
    let mut renderer = render(dark_surface_fixture, 256, 600, format, sample_count);
    for (stage, (phase, width, height)) in [
        ("cold", 256, 600),
        ("warm", 256, 600),
        ("resized", 320, 680),
        ("shrunk", 256, 600),
    ]
    .into_iter()
    .enumerate()
    {
        if stage >= 2 {
            renderer.resize(width, height);
        }
        if stage > 0 {
            renderer.render_frames(21);
        }
        let size = renderer.physical_size();
        assert_eq!((size.width, size.height), (width, height), "{phase}");
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let viewport = bounds(&nodes, "Precision dark viewport", Role::Group);
        assert_eq!(
            (
                viewport.x0,
                viewport.y0,
                viewport.width(),
                viewport.height()
            ),
            (0.0, 0.0, f64::from(width), f64::from(height)),
            "{phase} viewport geometry"
        );
        let shape = bounds(&nodes, "Precision shape patch", Role::Group);
        let surface = bounds(&nodes, "Precision surface patch", Role::Group);
        let rail_wrapper = bounds(&nodes, "Precision rail wrapper", Role::Group);
        assert_eq!((shape.width(), shape.height()), (64.0, 64.0));
        assert_eq!((surface.width(), surface.height()), (64.0, 64.0));
        assert_eq!((rail_wrapper.width(), rail_wrapper.height()), (96.0, 560.0));
        for rect in [shape, surface, rail_wrapper] {
            assert!(
                rect.x0 >= viewport.x0
                    && rect.y0 >= viewport.y0
                    && rect.x1 <= viewport.x1
                    && rect.y1 <= viewport.y1,
                "{phase} observed region must fit the actual viewport: {rect:?}, {viewport:?}"
            );
        }
        let pixels = renderer.read_pixels().unwrap();
        let stride = size.width as usize;
        assert_eq!(pixels.len(), stride * size.height as usize * 4, "{phase}");
        for (label, rect, y) in [
            ("background Shape patch", shape, (shape.y0 + shape.y1) * 0.5),
            (
                "zero-corner Surface patch",
                surface,
                (surface.y0 + surface.y1) * 0.5,
            ),
            // Sample the empty lower rail inside the observed wrapper.
            (
                "native collapsed rail container",
                rail_wrapper,
                rail_wrapper.y1 - 32.0,
            ),
        ] {
            assert_opaque_roi(
                &pixels,
                stride,
                (((rect.x0 + rect.x1) * 0.5) as usize, y as usize),
                DARK_SURFACE,
                &format!("{phase}: {label}"),
                format,
                sample_count,
            );
        }
        assert_opaque_roi(
            &pixels,
            stride,
            (stride - 16, size.height as usize - 16),
            [255, 255, 255, 255],
            &format!("{phase}: viewport background corner"),
            format,
            sample_count,
        );
    }
}

fn check_linear_scrim(format: TextureFormat, sample_count: u32) {
    let mut renderer = render(white_scrim_fixture, 400, 200, format, sample_count);
    for (stage, (phase, width, height)) in [
        ("cold", 400, 200),
        ("warm", 400, 200),
        ("resized", 448, 240),
        ("shrunk", 400, 200),
    ]
    .into_iter()
    .enumerate()
    {
        if stage >= 2 {
            renderer.resize(width, height);
        }
        if stage > 0 {
            renderer.render_frames(21);
        }
        let size = renderer.physical_size();
        assert_eq!((size.width, size.height), (width, height), "{phase}");
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let sheet = bounds(&nodes, "Precision scrim sheet", Role::Dialog);
        assert_eq!(
            (sheet.x0, sheet.y0, sheet.width(), sheet.height()),
            (0.0, 0.0, 240.0, f64::from(height)),
            "{phase} sheet geometry must follow the actual viewport height"
        );
        assert!(sheet.x1 < 349.0);
        assert!(sheet.x1 < f64::from(size.width) - 33.0);
        let pixels = renderer.read_pixels().unwrap();
        let stride = size.width as usize;
        assert_eq!(pixels.len(), stride * size.height as usize * 4, "{phase}");
        for center in [(350, 100), (stride - 32, size.height as usize - 32)] {
            assert_opaque_roi(
                &pixels,
                stride,
                center,
                WHITE_UNDER_LINEAR_SCRIM,
                &format!("{phase}: native 0.32 black scrim over opaque white"),
                format,
                sample_count,
            );
        }
    }
}

macro_rules! precision_case {
    ($name:ident, $check:ident, $format:ident, $samples:literal) => {
        #[test]
        #[ignore = "requires a native GPU adapter; validates final sRGB readback bytes"]
        fn $name() {
            $check(TextureFormat::$format, $samples);
        }
    };
}

precision_case!(
    dark_surface_srgb_msaa1,
    check_dark_surface,
    Rgba8UnormSrgb,
    1
);
precision_case!(
    dark_surface_srgb_msaa4,
    check_dark_surface,
    Rgba8UnormSrgb,
    4
);
precision_case!(dark_surface_unorm_msaa1, check_dark_surface, Rgba8Unorm, 1);
precision_case!(dark_surface_unorm_msaa4, check_dark_surface, Rgba8Unorm, 4);
precision_case!(
    linear_scrim_srgb_msaa1,
    check_linear_scrim,
    Rgba8UnormSrgb,
    1
);
precision_case!(
    linear_scrim_srgb_msaa4,
    check_linear_scrim,
    Rgba8UnormSrgb,
    4
);
precision_case!(linear_scrim_unorm_msaa1, check_linear_scrim, Rgba8Unorm, 1);
precision_case!(linear_scrim_unorm_msaa4, check_linear_scrim, Rgba8Unorm, 4);
