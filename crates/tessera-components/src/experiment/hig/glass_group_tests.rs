use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, Modifier, State, remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{alignment::Alignment, boxed::boxed, image::ImageData, spacer::spacer};

use super::glass_group;

fn stripes() -> ImageData {
    let (width, height) = (240, 120);
    let mut pixels = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let stripe = if (x + y) / 12 % 2 == 0 { 36 } else { 0 };
            pixels.extend_from_slice(&[
                (60 + x * 150 / width + stripe) as u8,
                (90 + y * 100 / height) as u8,
                (220 - x * 80 / width) as u8,
                255,
            ]);
        }
    }
    ImageData::from_rgba(pixels, width as u32, height as u32)
}

#[tessera]
fn fixture(probe: TestProbe<Option<State<bool>>>) {
    let visible = remember(|| false);
    let image = remember(stripes);
    probe.set(Some(visible));
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().size(Dp(240.0), Dp(120.0)))
        .children(move || {
            crate::image::image()
                .painter(image.get())
                .modifier(Modifier::new().fill_max_size());
            if visible.get() {
                glass_group().spacing(Dp(16.0)).gap(Dp(6.0)).child(|| {
                    spacer().modifier(Modifier::new().size(Dp(44.0), Dp(36.0)));
                    spacer().modifier(Modifier::new().size(Dp(44.0), Dp(36.0)));
                });
            }
        });
}

#[test]
#[ignore = "requires a native GPU adapter for glass pixels"]
fn fused_glass_leaves_the_backdrop_untouched_beyond_its_shadow_at_one_pixel_per_point() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(240, 120)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let backdrop = ui.read_pixels();
    ui.run(|| probe.get().expect("fixture state").set(true));
    ui.frames(3);
    let glass = ui.read_pixels();
    let group = ui.node("glass_group").bounds();
    let reach = 12;
    let mut changed = 0;
    for y in 0..120 {
        for x in 0..240 {
            let near = x >= group.x.0 - reach
                && x < group.x.0 + group.width.0 + reach
                && y >= group.y.0 - reach
                && y < group.y.0 + group.height.0 + reach;
            let index = (y * 240 + x) as usize * 4;
            let delta = (0..3)
                .map(|channel| backdrop[index + channel].abs_diff(glass[index + channel]))
                .max()
                .expect("three channels");
            if near {
                changed += usize::from(delta > 8);
            } else {
                assert!(delta <= 2, "pixel ({x}, {y}) changed by {delta}");
            }
        }
    }
    assert!(
        changed > 1000,
        "the glass must render: {changed} pixels changed"
    );
}
