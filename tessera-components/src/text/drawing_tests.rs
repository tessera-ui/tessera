use std::cell::Cell;

use tessera_ui::{
    Color, ComputedData, Constraint, Dp, DrawModifierContent, DrawModifierContext,
    DrawModifierNode, HeadlessConfig, HeadlessRenderer, LayoutModifierChild, LayoutModifierInput,
    LayoutModifierNode, LayoutResult, MeasurementError, Modifier, PxPosition, State,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
};

use crate::{
    boxed::boxed,
    modifier::{ModifierExt as _, Padding},
    pipelines::shape::command::ShapeCommand,
    text::text,
    theme::material_theme,
};

thread_local! {
    static DIRECT: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static INSET: Cell<Option<State<Dp>>> = const { Cell::new(None) };
    static INK: Cell<Option<State<Color>>> = const { Cell::new(None) };
    static SHIFT: Cell<Option<State<Dp>>> = const { Cell::new(None) };
    static PLACED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static MEASURES: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn padded_text_fixture() {
    let direct = remember(|| false);
    let inset = remember(|| Dp(12.0));
    let ink = remember(|| Color::BLACK);
    DIRECT.set(Some(direct));
    INSET.set(Some(inset));
    INK.set(Some(ink));
    material_theme().child(move || {
        boxed()
            .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
            .children(move || {
                let outer = Modifier::new()
                    .background(Color::new(1.0, 0.0, 0.0, 1.0))
                    .border(Dp(2.0), Color::new(0.0, 0.6, 0.0, 1.0))
                    .padding(Padding {
                        left: inset.get(),
                        top: Dp(8.0),
                        right: Dp(16.0),
                        bottom: Dp(10.0),
                    });
                let inner = Modifier::new()
                    .background(Color::new(0.0, 0.0, 1.0, 1.0))
                    .padding(Padding {
                        left: Dp(5.0),
                        top: Dp(6.0),
                        right: Dp(7.0),
                        bottom: Dp(4.0),
                    });
                if direct.get() {
                    text()
                        .modifier(outer.then(inner))
                        .content("Padded label")
                        .color(ink.get());
                } else {
                    boxed().modifier(outer).children(move || {
                        boxed().modifier(inner.clone()).children(move || {
                            text().content("Padded label").color(ink.get());
                        });
                    });
                }
            });
    });
}

fn check_padded_drawing(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| padded_text_fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 480,
            height: 240,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let reference = renderer.read_pixels().expect("nested reference");
    let direct = DIRECT.get().expect("direct state");
    direct.set(true);
    renderer.render_frames(4);
    let actual = renderer.read_pixels().expect("direct modifiers");
    assert!(
        reference == actual,
        "layout and draw modifier order must match equivalent nested containers"
    );
    renderer.render_frames(5);
    assert!(actual == renderer.read_pixels().expect("cached direct drawing"));

    INSET.get().expect("padding state").set(Dp(25.0));
    INK.get().expect("ink state").set(Color::WHITE);
    renderer.render_frames(4);
    let changed = renderer.read_pixels().expect("changed direct drawing");
    direct.set(false);
    renderer.render_frames(4);
    assert!(changed == renderer.read_pixels().expect("changed reference"));
    INSET.get().expect("padding state").set(Dp(12.0));
    INK.get().expect("ink state").set(Color::BLACK);
    renderer.render_frames(4);
    assert!(reference == renderer.read_pixels().expect("restored reference"));
    direct.set(true);
    renderer.render_frames(4);
    assert!(reference == renderer.read_pixels().expect("restored direct drawing"));
    for width in [120, 1, 480] {
        renderer.resize(width, 240);
        renderer.render_frames(4);
        let direct_pixels = renderer.read_pixels().expect("resized direct drawing");
        direct.set(false);
        renderer.render_frames(4);
        assert!(
            direct_pixels == renderer.read_pixels().expect("resized reference"),
            "drawing layers must recover at viewport width {width}"
        );
        direct.set(true);
        renderer.render_frames(4);
    }
    assert!(reference == renderer.read_pixels().expect("restored viewport"));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn padded_text_and_ordered_backgrounds_match_nested_containers() {
    crate::test_utils::run_isolated_test(
        "text::drawing_tests::padded_text_and_ordered_backgrounds_match_nested_containers",
        || check_padded_drawing(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn padded_text_and_ordered_backgrounds_match_nested_containers_msaa() {
    crate::test_utils::run_isolated_test(
        "text::drawing_tests::padded_text_and_ordered_backgrounds_match_nested_containers_msaa",
        || check_padded_drawing(4),
    );
}

#[derive(Clone, Copy, PartialEq)]
struct ShiftContents {
    offset: Dp,
    placed: bool,
}

impl LayoutModifierNode for ShiftContents {
    fn measure_content(
        &self,
        _input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        child.measure(&Constraint::exact(Dp(40.0).into(), Dp(24.0).into()))?;
        if self.placed {
            child.place(PxPosition::new(self.offset.into(), Dp(8.0).into()));
        }
        Ok(LayoutResult::new(ComputedData {
            width: Dp(100.0).into(),
            height: Dp(80.0).into(),
        }))
    }

    fn measure_eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SolidLeaf;

impl LayoutModifierNode for SolidLeaf {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        MEASURES.set(MEASURES.get() + 1);
        Ok(LayoutResult::new(ComputedData::min_from_constraint(
            scope.parent_constraint().as_ref(),
        )))
    }

    fn place_children(
        &self,
        _scope: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        Some(PlacementResult::new())
    }
}

impl DrawModifierNode for SolidLeaf {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, _content: &mut dyn DrawModifierContent) {
        ctx.render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(ShapeCommand::Rect {
                color: Color::new(1.0, 0.0, 0.0, 1.0),
                corner_radii: [0.0; 4],
                corner_g2: [2.0; 4],
            });
    }
}

#[tessera]
fn placement_fixture() {
    let shift = remember(|| Dp::ZERO);
    let placed = remember(|| true);
    SHIFT.set(Some(shift));
    PLACED.set(Some(placed));
    boxed()
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(move || {
            layout()
                .modifier(Modifier::new().push_layout(ShiftContents {
                    offset: shift.get(),
                    placed: placed.get(),
                }))
                .policy(SolidLeaf)
                .draw(SolidLeaf);
        });
}

fn check_placement_only_drawing() {
    MEASURES.set(0);
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| placement_fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 240,
            height: 180,
            scale_factor: 1.5,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let original = renderer.read_pixels().expect("original content");
    let measures = MEASURES.get();
    let pixel = |pixels: &[u8], x: usize, y: usize| pixels[(y * 240 + x) * 4..][..3].to_vec();
    assert_eq!(pixel(&original, 2, 14), [255, 0, 0]);
    assert_eq!(pixel(&original, 62, 14), [255, 255, 255]);
    SHIFT.get().expect("offset state").set(Dp(12.0));
    renderer.render_frames(4);
    let shifted = renderer.read_pixels().expect("shifted content");
    assert_eq!(pixel(&shifted, 2, 14), [255, 255, 255]);
    assert_eq!(pixel(&shifted, 20, 14), [255, 0, 0]);
    assert_eq!(pixel(&shifted, 80, 14), [255, 255, 255]);
    assert_eq!(MEASURES.get(), measures);
    PLACED.get().expect("placement state").set(false);
    renderer.render_frames(4);
    let hidden = renderer.read_pixels().expect("unplaced content");
    assert_eq!(pixel(&hidden, 20, 14), [255, 255, 255]);
    assert_eq!(MEASURES.get(), measures);
    PLACED.get().expect("placement state").set(true);
    SHIFT.get().expect("offset state").set(Dp::ZERO);
    renderer.render_frames(4);
    assert!(original == renderer.read_pixels().expect("restored content"));
    assert_eq!(MEASURES.get(), measures);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn placement_only_content_moves_and_hides_without_remeasuring_the_policy() {
    crate::test_utils::run_isolated_test(
        "text::drawing_tests::placement_only_content_moves_and_hides_without_remeasuring_the_policy",
        check_placement_only_drawing,
    );
}
