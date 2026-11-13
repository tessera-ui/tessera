use tessera_foundation::modifier::{ModifierExt as _, Padding};
use tessera_ui::{
    Color, ComputedData, Constraint, Dp, DrawModifierContent, DrawModifierContext,
    DrawModifierNode, LayoutModifier, LayoutModifierChild, LayoutModifierInput, LayoutPolicy,
    LayoutResult, MeasurementError, Modifier, PxPosition, State,
    layout::{MeasureScope, PlacementResult, PlacementScope},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
    testing::{TestProbe, UiTest, WorkPhase},
};

use crate::{
    boxed::boxed, modifier::ModifierExt as _, pipelines::shape::command::ShapeCommand, text::text,
    theme::material_theme,
};

#[derive(Clone, Copy, Default, PartialEq)]
struct FixtureHandles {
    direct: Option<State<bool>>,
    inset: Option<State<Dp>>,
    ink: Option<State<Color>>,
    shift: Option<State<Dp>>,
    placed: Option<State<bool>>,
}

#[tessera]
fn padded_text_fixture(probe: TestProbe<FixtureHandles>) {
    let direct = remember(|| false);
    let inset = remember(|| Dp(12.0));
    let ink = remember(|| Color::BLACK);
    probe.set(FixtureHandles {
        direct: Some(direct),
        inset: Some(inset),
        ink: Some(ink),
        ..Default::default()
    });
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
                    text("Padded label")
                        .modifier(outer.then(inner))
                        .color(ink.get());
                } else {
                    boxed().modifier(outer).children(move || {
                        boxed().modifier(inner.clone()).children(move || {
                            text("Padded label").color(ink.get());
                        });
                    });
                }
            });
    });
}

fn check_padded_drawing() {
    let probe = TestProbe::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(480, 240)
        .scale_factor(1.5)
        .rendered(crate::TesseraComponents)
        .content(move || padded_text_fixture(probe.clone()));
    renderer.frames(4);
    let reference = renderer.read_pixels();
    let direct = observed.get().direct.expect("direct state");
    direct.set(true);
    renderer.frames(4);
    let actual = renderer.read_pixels();
    assert!(
        reference == actual,
        "layout and draw modifier order must match equivalent nested containers"
    );
    renderer.frames(5);
    assert!(actual == renderer.read_pixels());

    observed.get().inset.expect("padding state").set(Dp(25.0));
    observed.get().ink.expect("ink state").set(Color::WHITE);
    renderer.frames(4);
    let changed = renderer.read_pixels();
    direct.set(false);
    renderer.frames(4);
    assert!(changed == renderer.read_pixels());
    observed.get().inset.expect("padding state").set(Dp(12.0));
    observed.get().ink.expect("ink state").set(Color::BLACK);
    renderer.frames(4);
    assert!(reference == renderer.read_pixels());
    direct.set(true);
    renderer.frames(4);
    assert!(reference == renderer.read_pixels());
    for width in [120, 1, 480] {
        renderer.resize(width, 240);
        renderer.frames(4);
        let direct_pixels = renderer.read_pixels();
        direct.set(false);
        renderer.frames(4);
        assert!(
            direct_pixels == renderer.read_pixels(),
            "drawing layers must recover at viewport width {width}"
        );
        direct.set(true);
        renderer.frames(4);
    }
    assert!(reference == renderer.read_pixels());
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn padded_text_and_ordered_backgrounds_match_nested_containers() {
    crate::test_utils::run_isolated_test(
        "text::drawing_tests::padded_text_and_ordered_backgrounds_match_nested_containers",
        check_padded_drawing,
    );
}

#[derive(Clone, Copy, PartialEq)]
struct ShiftContents {
    offset: Dp,
    placed: bool,
}

impl LayoutModifier for ShiftContents {
    fn measure(
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

impl LayoutPolicy for SolidLeaf {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData::min_from_constraint(
            scope.parent_constraint().as_ref(),
        )))
    }

    fn place(
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
fn placement_fixture(probe: TestProbe<FixtureHandles>) {
    let shift = remember(|| Dp::ZERO);
    let placed = remember(|| true);
    probe.set(FixtureHandles {
        shift: Some(shift),
        placed: Some(placed),
        ..Default::default()
    });
    boxed()
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(move || {
            tessera_ui::layout::layout(SolidLeaf)
                .modifier(Modifier::new().push_layout(ShiftContents {
                    offset: shift.get(),
                    placed: placed.get(),
                }))
                .draw(SolidLeaf);
        });
}

fn retained_layout_frames(renderer: &mut UiTest, count: usize) {
    for _ in 0..count {
        renderer.frames(1);
        assert_eq!(renderer.work().executed::<SolidLeaf>(WorkPhase::Measure), 0);
    }
}

fn check_placement_only_drawing() {
    let probe = TestProbe::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(240, 180)
        .scale_factor(1.5)
        .rendered(crate::TesseraComponents)
        .content(move || placement_fixture(probe.clone()));
    renderer.frames(1);
    assert!(renderer.work().executed::<SolidLeaf>(WorkPhase::Measure) > 0);
    retained_layout_frames(&mut renderer, 3);
    let original = renderer.read_pixels();
    let pixel = |pixels: &[u8], x: usize, y: usize| pixels[(y * 240 + x) * 4..][..3].to_vec();
    assert_eq!(pixel(&original, 2, 14), [255, 0, 0]);
    assert_eq!(pixel(&original, 62, 14), [255, 255, 255]);
    observed.get().shift.expect("offset state").set(Dp(12.0));
    retained_layout_frames(&mut renderer, 4);
    let shifted = renderer.read_pixels();
    assert_eq!(pixel(&shifted, 2, 14), [255, 255, 255]);
    assert_eq!(pixel(&shifted, 20, 14), [255, 0, 0]);
    assert_eq!(pixel(&shifted, 80, 14), [255, 255, 255]);
    observed.get().placed.expect("placement state").set(false);
    retained_layout_frames(&mut renderer, 4);
    let hidden = renderer.read_pixels();
    assert_eq!(pixel(&hidden, 20, 14), [255, 255, 255]);
    observed.get().placed.expect("placement state").set(true);
    observed.get().shift.expect("offset state").set(Dp::ZERO);
    retained_layout_frames(&mut renderer, 4);
    assert!(original == renderer.read_pixels());
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn placement_only_content_moves_and_hides_without_remeasuring_the_policy() {
    crate::test_utils::run_isolated_test(
        "text::drawing_tests::placement_only_content_moves_and_hides_without_remeasuring_the_policy",
        check_placement_only_drawing,
    );
}

#[derive(Clone, PartialEq)]
struct StateInkDraw {
    ink: State<Color>,
}

impl DrawModifierNode for StateInkDraw {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, _content: &mut dyn DrawModifierContent) {
        ctx.render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(ShapeCommand::Rect {
                color: self.ink.get(),
                corner_radii: [0.0; 4],
                corner_g2: [2.0; 4],
            });
    }
}

#[tessera]
fn paint_state_fixture(probe: TestProbe<FixtureHandles>) {
    let ink = remember(|| Color::new(1.0, 0.0, 0.0, 1.0));
    probe.set(FixtureHandles {
        ink: Some(ink),
        ..Default::default()
    });
    tessera_ui::layout::layout(SolidLeaf).draw(StateInkDraw { ink });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn paint_state_updates_pixels_without_rebuilding_or_measuring_and_reuses_stable_fragments() {
    crate::test_utils::run_isolated_test(
        "text::drawing_tests::paint_state_updates_pixels_without_rebuilding_or_measuring_and_reuses_stable_fragments",
        || {
            let probe = TestProbe::default();
            let observed = probe.clone();
            let mut renderer = UiTest::new()
                .viewport(64, 48)
                .rendered(crate::TesseraComponents)
                .content(move || paint_state_fixture(probe.clone()));
            renderer.frames(1);
            let reference = renderer.read_pixels();
            let pixel = (12 * 64 + 16) * 4;
            assert_eq!(&reference[pixel..pixel + 4], &[255, 0, 0, 255]);
            assert!(renderer.work().executed::<SolidLeaf>(WorkPhase::Measure) > 0);
            assert_eq!(
                renderer
                    .work()
                    .executed::<StateInkDraw>(tessera_ui::testing::WorkPhase::Paint),
                1
            );
            let builds = renderer
                .work_totals()
                .count("paint_state_fixture", tessera_ui::testing::WorkPhase::Build);
            assert!(builds > 0);
            retained_paint_frames(&mut renderer, 3);
            assert!(reference == renderer.read_pixels());
            observed
                .get()
                .ink
                .expect("ink state")
                .set(Color::new(0.0, 0.0, 1.0, 1.0));
            renderer.frames(1);
            let changed = renderer.read_pixels();
            assert_eq!(&changed[pixel..pixel + 4], &[0, 0, 255, 255]);
            assert_eq!(renderer.work().executed::<SolidLeaf>(WorkPhase::Measure), 0);
            assert_eq!(
                renderer
                    .work_totals()
                    .count("paint_state_fixture", tessera_ui::testing::WorkPhase::Build),
                builds
            );
            assert_eq!(
                renderer
                    .work()
                    .executed::<StateInkDraw>(tessera_ui::testing::WorkPhase::Paint),
                1
            );
            retained_paint_frames(&mut renderer, 3);
            assert!(changed == renderer.read_pixels());
            assert_eq!(
                renderer
                    .work()
                    .executed::<StateInkDraw>(tessera_ui::testing::WorkPhase::Paint),
                0
            );
        },
    );
}

fn retained_paint_frames(renderer: &mut UiTest, count: usize) {
    for _ in 0..count {
        renderer.frames(1);
        assert_eq!(renderer.work().executed::<SolidLeaf>(WorkPhase::Measure), 0);
        assert_eq!(
            renderer
                .work()
                .executed::<StateInkDraw>(tessera_ui::testing::WorkPhase::Paint),
            0
        );
    }
}
