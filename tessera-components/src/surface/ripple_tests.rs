//! Ripple repainting with stationary input and retained content.
//!
//! ## Usage
//!
//! Let click feedback finish without pointer movement or application updates.

use std::time::Duration;

use tessera_ui::{
    Color, ComputedData, Dp, HeadlessConfig, HeadlessRenderer, LayoutResult, MOUSE_POINTER_ID,
    MeasurementError, Modifier, PressKeyEventType, Px, PxPosition,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::{button::button, modifier::ModifierExt as _, theme::material_theme};

#[derive(Clone, PartialEq)]
struct ContentMeasure;

impl tessera_ui::LayoutModifierNode for ContentMeasure {
    fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: Px(20),
            height: Px(20),
        }))
    }
}

#[tessera]
fn retained_content() {
    layout().policy(ContentMeasure);
}

#[tessera]
fn fixture(clicks: tessera_ui::testing::TestProbe<usize>) {
    material_theme().child(move || {
        let clicks = clicks.clone();
        button()
            .modifier(Modifier::new().size(Dp(160.0), Dp(64.0)))
            .color(Color::WHITE)
            .content_color(Color::BLACK)
            .ripple_color(Color::BLACK)
            .elevation(Dp(0.0))
            .accessibility_label("No-op button")
            .on_click(move || clicks.with_mut(|count| *count += 1))
            .child(|| {
                retained_content();
            });
    });
}

fn assert_retained_frames(renderer: &mut HeadlessRenderer, frames: u32) {
    for _ in 0..frames {
        renderer.step_frame();
        assert_eq!(
            renderer
                .work()
                .executed::<ContentMeasure>(tessera_ui::WorkPhase::Measure),
            0,
            "ripple must not remeasure content"
        );
        assert_eq!(
            renderer
                .work()
                .count("retained_content", tessera_ui::WorkPhase::Build),
            0,
            "ripple must retain the button content"
        );
    }
}

fn check_stationary_ripple(samples: u32) {
    let clicks = tessera_ui::testing::TestProbe::new(0usize);
    let observed = clicks.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || fixture(observed.clone())),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 240,
            height: 120,
            sample_count: samples,
            frame_time: Duration::from_millis(20),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.step_frame();
    assert!(
        renderer
            .work()
            .executed::<ContentMeasure>(tessera_ui::WorkPhase::Measure)
            > 0
    );
    renderer.render_frames(4);
    let (_, node) = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("No-op button"))
        .unwrap();
    let bounds = node.bounds().unwrap();
    let center = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, center);
    renderer.pointer_press(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
    renderer.render_frames(40);
    let baseline = renderer.read_pixels().unwrap();
    let builds = renderer
        .work_totals()
        .count("retained_content", tessera_ui::WorkPhase::Build);
    assert!(builds > 0);
    for click in 2..=3 {
        renderer.pointer_press(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
        assert_retained_frames(&mut renderer, 2);
        let early = renderer.read_pixels().unwrap();
        assert!(early != baseline, "click must paint a visible ripple");
        assert_retained_frames(&mut renderer, 4);
        let later = renderer.read_pixels().unwrap();
        assert!(
            later != early,
            "ripple pixels must advance while the pointer and application state are unchanged"
        );
        assert_retained_frames(&mut renderer, 20);
        assert!(
            renderer.read_pixels().unwrap() == baseline,
            "the final frame must clear the ripple without another input event"
        );
        assert_retained_frames(&mut renderer, 10);
        assert!(renderer.read_pixels().unwrap() == baseline);
        assert_eq!(clicks.get(), click);
        assert_eq!(
            renderer
                .work_totals()
                .count("retained_content", tessera_ui::WorkPhase::Build),
            builds,
            "ripple must retain the button content"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn stationary_noop_button_ripple_advances_finishes_and_restarts_msaa1() {
    crate::test_utils::run_isolated_test(
        "surface::ripple_tests::stationary_noop_button_ripple_advances_finishes_and_restarts_msaa1",
        || check_stationary_ripple(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn stationary_noop_button_ripple_advances_finishes_and_restarts_msaa4() {
    crate::test_utils::run_isolated_test(
        "surface::ripple_tests::stationary_noop_button_ripple_advances_finishes_and_restarts_msaa4",
        || check_stationary_ripple(4),
    );
}
