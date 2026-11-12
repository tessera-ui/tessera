//! Interrupted transient-surface animation regressions.
//!
//! ## Usage
//!
//! Keep transient surfaces continuous during rapid visibility changes.

use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, Modifier,
    accesskit::{Node, Role},
    remember, tessera,
    testing::UiTest,
};

use crate::{
    bottom_sheet::{BottomSheetController, bottom_sheet_provider},
    dialog::{DialogController, dialog_provider},
    side_sheet::{
        SideSheetController, SideSheetPosition, modal_side_sheet_provider,
        standard_side_sheet_provider,
    },
    surface::surface,
    test_utils::OverlayController,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Dialog,
    Bottom,
    Start,
    End,
    StandardStart,
    StandardEnd,
}

#[tessera]
fn main_content() {
    surface()
        .modifier(Modifier::new().fill_max_size())
        .style(Color::WHITE.into())
        .child(|| {});
}

#[tessera]
fn marker_content() {
    surface()
        .modifier(Modifier::new().size(Dp(200.0), Dp(120.0)))
        .style(Color::RED.into())
        .accessibility_role(Role::Group)
        .accessibility_label("Motion marker")
        .child(|| {});
}

#[tessera]
fn fixture(kind: Kind, output: tessera_ui::testing::TestProbe<Option<OverlayController>>) {
    let open = match kind {
        Kind::Dialog => OverlayController::Dialog(remember(|| DialogController::new(false))),
        Kind::Bottom => OverlayController::Bottom(remember(|| BottomSheetController::new(false))),
        _ => OverlayController::Side(remember(|| SideSheetController::new(false))),
    };
    output.set(Some(open));
    material_theme().child(move || match open {
        OverlayController::Dialog(controller) => {
            dialog_provider()
                .controller(controller)
                .main_content(|| {
                    main_content();
                })
                .dialog_content(|_| {
                    marker_content();
                });
        }
        OverlayController::Bottom(controller) => {
            bottom_sheet_provider()
                .controller(controller)
                .main_content(|| {
                    main_content();
                })
                .bottom_sheet_content(|| {
                    marker_content();
                });
        }
        OverlayController::Side(controller) => {
            let position = if matches!(kind, Kind::Start | Kind::StandardStart) {
                SideSheetPosition::Start
            } else {
                SideSheetPosition::End
            };
            if matches!(kind, Kind::StandardStart | Kind::StandardEnd) {
                standard_side_sheet_provider()
                    .controller(controller)
                    .position(position)
                    .main_content(|| {
                        main_content();
                    })
                    .side_sheet_content(|| {
                        marker_content();
                    });
                return;
            }
            modal_side_sheet_provider()
                .controller(controller)
                .position(position)
                .main_content(|| {
                    main_content();
                })
                .side_sheet_content(|| {
                    marker_content();
                });
        }
    });
}

fn marker(renderer: &UiTest) -> Option<Node> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some("Motion marker"))
        .map(|(_, n)| n)
}

fn green(renderer: &UiTest, node: &Node) -> u8 {
    let rect = node.bounds().unwrap();
    let width = renderer.physical_size().width as usize;
    let height = renderer.physical_size().height as usize;
    let x = ((rect.x0 + rect.x1) * 0.5) as usize;
    let y = ((rect.y0 + rect.y1) * 0.5) as usize;
    assert!(x < width && y < height);
    renderer.read_pixels()[(y * width + x) * 4 + 1]
}

fn check(samples: u32) {
    for kind in [
        Kind::Dialog,
        Kind::Bottom,
        Kind::Start,
        Kind::End,
        Kind::StandardStart,
        Kind::StandardEnd,
    ] {
        for scale in [1.0, 1.5] {
            let output = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(800, 600)
                .scale_factor(scale)
                .samples(samples)
                .frame_step(Duration::from_millis(10))
                .rendered(crate::TesseraComponents)
                .content(|| fixture(kind, output.clone()));
            renderer.frames(4);
            let baseline = renderer.read_pixels();
            let open = output.get().unwrap();
            open.set(true);
            renderer.frames(8);
            let entering = marker(&renderer).unwrap();
            let entering_bounds = entering.bounds().unwrap();
            let entering_opacity =
                matches!(kind, Kind::Dialog).then(|| green(&renderer, &entering));
            for target in [false, true, false, true] {
                let before = marker(&renderer).unwrap();
                let rect = before.bounds().unwrap();
                let opacity = matches!(kind, Kind::Dialog).then(|| green(&renderer, &before));
                open.set(target);
                renderer.frames(1);
                let after = marker(&renderer).unwrap();
                let next = after.bounds().unwrap();
                assert!(
                    (rect.x0 - next.x0).abs() < 48.0 * scale,
                    "{kind:?}: horizontal jump"
                );
                assert!(
                    (rect.y0 - next.y0).abs() < 48.0 * scale,
                    "{kind:?}: vertical jump"
                );
                if let Some(opacity) = opacity {
                    assert!(
                        opacity.abs_diff(green(&renderer, &after)) < 60,
                        "dialog opacity jumped"
                    );
                }
                renderer.frames(2);
            }
            renderer.frames(80);
            let full = renderer.read_pixels();
            assert!(full != baseline);
            let settled = marker(&renderer).unwrap();
            let is_sheet = !matches!(kind, Kind::Dialog | Kind::Bottom);
            if is_sheet {
                let bounds = settled.bounds().unwrap();
                let expected_x = if matches!(kind, Kind::Start | Kind::StandardStart) {
                    24.0 * scale
                } else {
                    800.0 - 400.0 * scale + 24.0 * scale
                };
                assert!(
                    (bounds.x0 - expected_x).abs() < 0.01,
                    "{kind:?}: width cap or horizontal padding"
                );
                assert!((bounds.y0 - 24.0 * scale).abs() < 0.01);
            }
            if let Some(opacity) = entering_opacity {
                assert!(opacity.abs_diff(green(&renderer, &settled)) > 10);
            } else {
                let settled = settled.bounds().unwrap();
                assert!(
                    (entering_bounds.x0 - settled.x0).abs()
                        + (entering_bounds.y0 - settled.y0).abs()
                        > 8.0 * scale
                );
            }
            open.set(false);
            renderer.frames(5);
            open.set(true);
            renderer.frames(80);
            assert!(
                renderer.read_pixels() == full,
                "{kind:?}: reopened pixels differ"
            );
            open.set(false);
            renderer.frames(80);
            assert!(marker(&renderer).is_none());
            assert!(
                renderer.read_pixels() == baseline,
                "{kind:?}: exit did not restore the scene"
            );
            if is_sheet {
                renderer.resize(320, 600);
                renderer.frames(4);
                open.set(true);
                renderer.frames(80);
                let bounds = marker(&renderer).unwrap().bounds().unwrap();
                assert!(
                    (bounds.x0 - 24.0 * scale).abs() < 0.01,
                    "{kind:?}: narrow viewport placement"
                );
                assert!(bounds.x1 <= 320.0 - 24.0 * scale + 0.01);
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn overlay_reversal_preserves_pixels_geometry_and_restores_the_scene_msaa1() {
    crate::test_utils::run_isolated_test(
        "overlay_motion_tests::overlay_reversal_preserves_pixels_geometry_and_restores_the_scene_msaa1",
        || check(1),
    );
}
#[test]
#[ignore = "requires a native GPU adapter"]
fn overlay_reversal_preserves_pixels_geometry_and_restores_the_scene_msaa4() {
    crate::test_utils::run_isolated_test(
        "overlay_motion_tests::overlay_reversal_preserves_pixels_geometry_and_restores_the_scene_msaa4",
        || check(4),
    );
}
