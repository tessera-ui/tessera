use std::cell::Cell;

use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Rect, Role},
    remember, tessera,
};

use crate::{
    boxed::boxed, icon::icon, material_icons::filled, modifier::ModifierExt as _, spacer::spacer,
    text::text, theme::material_theme,
};

use super::{
    SplitButtonDefaults, SplitButtonSize, split_button_layout, split_leading_button,
    split_trailing_button,
};

#[derive(Clone, Copy)]
struct Settings {
    leading: SplitButtonSize,
    trailing: SplitButtonSize,
    width: f64,
    enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            leading: SplitButtonSize::ExtraSmall,
            trailing: SplitButtonSize::ExtraSmall,
            width: 410.0,
            enabled: true,
        }
    }
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    SETTINGS.set(Some(settings));
    material_theme().child(move || {
        boxed().children(move || {
            let value = settings.get();
            split_button_layout()
                .modifier(Modifier::new().width(Dp(value.width)))
                .leading_button(move || {
                    let value = settings.get();
                    split_leading_button()
                        .size(value.leading)
                        .enabled(value.enabled)
                        .accessibility_label("Primary")
                        .on_click(|| {})
                        .content(move || {
                            let value = settings.get();
                            icon().vector(filled::SEND_SVG);
                            spacer().modifier(
                                Modifier::new()
                                    .width(SplitButtonDefaults::icon_spacing(value.leading)),
                            );
                            text().content("Send");
                        });
                })
                .trailing_button(move || {
                    let value = settings.get();
                    split_trailing_button()
                        .size(value.trailing)
                        .enabled(value.enabled)
                        .accessibility_label("Options")
                        .on_click(|| {})
                        .content(|| {
                            icon().vector(filled::ARROW_DROP_DOWN_SVG);
                        });
                });
        });
    });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .expect("button")
        .1
        .bounds()
        .expect("bounds")
}

fn filled_rows(pixels: &[u8], width: usize, bounds: Rect) -> (usize, usize) {
    let background = &pixels[pixels.len() - 4..];
    let rows: Vec<_> = (bounds.y0.ceil() as usize..bounds.y1.floor() as usize)
        .filter(|&y| {
            (bounds.x0.ceil() as usize..bounds.x1.floor() as usize).any(|x| {
                let pixel = &pixels[(y * width + x) * 4..][..4];
                (0..3).any(|channel| pixel[channel].abs_diff(background[channel]) > 20)
            })
        })
        .collect();
    (
        *rows.first().expect("filled top"),
        *rows.last().expect("filled bottom") + 1,
    )
}

fn check_height_recovery(samples: u32, scale: f64) {
    let width = (650.0 * scale) as u32;
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width,
            height: (500.0 * scale) as u32,
            scale_factor: scale,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let initial = renderer.read_pixels().expect("pixels");
    let settings = SETTINGS.get().expect("settings");
    use SplitButtonSize::{ExtraLarge, ExtraSmall, Small};
    for (leading, trailing, available, expected) in [
        (ExtraSmall, ExtraSmall, 410.0, Some(32.0)),
        (ExtraSmall, ExtraSmall, 120.0, Some(40.0)),
        (Small, Small, 410.0, Some(40.0)),
        (ExtraLarge, ExtraLarge, 410.0, Some(136.0)),
        (ExtraLarge, ExtraLarge, 340.0, None),
        (ExtraSmall, ExtraLarge, 410.0, Some(136.0)),
        (ExtraSmall, Small, 410.0, Some(40.0)),
        (ExtraSmall, ExtraSmall, 410.0, Some(32.0)),
    ] {
        settings.set(Settings {
            leading,
            trailing,
            width: available,
            enabled: true,
        });
        renderer.step_frame();
        let primary = bounds(&renderer, "Primary");
        let options = bounds(&renderer, "Options");
        assert_eq!(primary.height(), options.height());
        assert_eq!(primary.y0, options.y0);
        assert!(primary.height() >= 48.0 * scale);
        let pixels = renderer.read_pixels().expect("pixels");
        let primary_rows = filled_rows(&pixels, width as usize, primary);
        let options_rows = filled_rows(&pixels, width as usize, options);
        assert!(
            primary_rows.0.abs_diff(options_rows.0) <= 1,
            "top: {primary_rows:?} != {options_rows:?}"
        );
        assert!(
            primary_rows.1.abs_diff(options_rows.1) <= 1,
            "bottom: {primary_rows:?} != {options_rows:?}"
        );
        let visual_height = (primary_rows.1 - primary_rows.0) as f64;
        if let Some(expected) = expected {
            assert!(
                (visual_height - expected * scale).abs() <= 1.0,
                "{leading:?}/{trailing:?} at {available}dp: {visual_height}px"
            );
        } else {
            assert!(
                visual_height > 136.0 * scale,
                "the narrow XL label must wrap"
            );
        }
    }
    renderer.render_frames(90);
    assert_eq!(renderer.read_pixels().expect("restored pixels"), initial);
    let before = (bounds(&renderer, "Primary"), bounds(&renderer, "Options"));
    settings.with_mut(|settings| settings.enabled = false);
    renderer.render_frames(4);
    assert_eq!(
        (bounds(&renderer, "Primary"), bounds(&renderer, "Options")),
        before
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn synchronized_visual_heights_and_touch_targets_msaa1() {
    check_height_recovery(1, 1.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn synchronized_visual_heights_and_touch_targets_msaa4_fractional_density() {
    check_height_recovery(4, 1.5);
}
