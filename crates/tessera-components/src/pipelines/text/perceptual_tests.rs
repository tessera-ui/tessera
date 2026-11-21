use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, Modifier, State, key, remember, tessera,
    testing::{TestProbe, UiTest, WorkPhase},
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    experiment::hig::{
        text::{text, write_font_system},
        theme::TextStyle,
    },
    modifier::ModifierExt as _,
};

#[derive(Clone, Copy, PartialEq)]
struct Presentation {
    dark: bool,
    opacity: f32,
    shift: f64,
    width: f64,
    generation: u32,
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            dark: false,
            opacity: 1.0,
            shift: 0.0,
            width: 160.0,
            generation: 0,
        }
    }
}

#[tessera]
fn fixture(probe: TestProbe<Option<State<Presentation>>>) {
    remember(|| {
        write_font_system().db_mut().load_font_data(
            include_bytes!("../../../../tessera-cosmic-text/fonts/InterVariable.ttf").to_vec(),
        );
    });
    let state = remember(Presentation::default);
    probe.set(Some(state));
    let presentation = state.get();
    let background = if presentation.dark {
        Color::BLACK
    } else {
        Color::WHITE
    };
    let foreground = if presentation.dark {
        Color::WHITE
    } else {
        Color::BLACK
    };
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().fill_max_size().background(background))
        .children(move || {
            key(presentation.generation, move || {
                text("Retained text wraps and clips to the allocated width.")
                    .style(TextStyle {
                        font_family: Some("Inter Variable"),
                        ..Default::default()
                    })
                    .color(foreground.with_alpha(presentation.opacity))
                    .modifier(
                        Modifier::new()
                            .offset(Dp(8.0 + presentation.shift), Dp(8.0))
                            .width(Dp(presentation.width))
                            .height(Dp(40.0))
                            .clip_to_bounds()
                            .semantics(SemanticsArgs {
                                test_tag: Some("label".into()),
                                ..Default::default()
                            }),
                    );
            });
        });
}

fn check_retained_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(360, 160)
        .scale_factor(1.5)
        .samples(4)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let initial = ui.read_pixels();
    assert!(initial.chunks_exact(4).any(|pixel| pixel[0] < 100));
    let state = probe.get().unwrap();
    state.with_mut(|value| value.opacity = 0.5);
    ui.frames(1);
    assert_eq!(ui.work().phase(WorkPhase::Measure).count(), 0);
    let dimmed = ui.read_pixels();
    let mut changed = 0;
    for (before, after) in initial.chunks_exact(4).zip(dimmed.chunks_exact(4)) {
        if before[0] < 250 {
            changed += 1;
            let expected = (u16::from(before[0]) + 255) / 2;
            assert!((i32::from(after[0]) - i32::from(expected)).abs() <= 2);
        }
    }
    assert!(changed > 100);
    for (dark, shift, width) in [
        (false, 0.25, 160.0),
        (true, 0.25, 90.0),
        (true, -10.5, 120.0),
        (false, 0.0, 160.0),
    ] {
        state.with_mut(|value| {
            value.dark = dark;
            value.shift = shift;
            value.width = width;
            value.opacity = 1.0;
        });
        ui.frames(2);
        let retained = ui.read_pixels();
        let bounds = ui.frames(1).node("label").bounds();
        let background = if dark { 0 } else { 255 };
        for y in 0..160 {
            for x in 0..360 {
                if x < bounds.x.0.max(0) as usize
                    || x >= (bounds.x.0 + bounds.width.0).max(0) as usize
                    || y < bounds.y.0.max(0) as usize
                    || y >= (bounds.y.0 + bounds.height.0).max(0) as usize
                {
                    assert_eq!(retained[(y * 360 + x) * 4], background);
                }
            }
        }
        state.with_mut(|value| value.generation += 1);
        ui.frames(2);
        assert_eq!(retained, ui.read_pixels());
    }
    assert_eq!(initial, ui.read_pixels());
    let label = ui
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .map(|(_, node)| node)
        .find(|node| node.label() == Some("Retained text wraps and clips to the allocated width."))
        .unwrap();
    assert_eq!(label.role(), tessera_ui::accesskit::Role::Label);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn retained_text_blends_in_srgb_and_recovers_after_reflow_and_fractional_clipping() {
    crate::test_utils::run_isolated_test(
        "pipelines::text::pipeline::perceptual::drawing_tests::retained_text_blends_in_srgb_and_recovers_after_reflow_and_fractional_clipping",
        check_retained_pixels,
    );
}
