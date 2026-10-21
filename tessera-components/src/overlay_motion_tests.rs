//! Interrupted transient-surface animation regressions.
//!
//! ## Usage
//!
//! Keep transient surfaces continuous during rapid visibility changes.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State,
    accesskit::{Node, Role},
    remember, tessera,
};

use crate::{
    bottom_sheet::bottom_sheet_provider,
    dialog::dialog_provider,
    modifier::ModifierExt as _,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider},
    surface::surface,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Dialog,
    Bottom,
    Start,
    End,
    StandardStart,
    StandardEnd,
}

thread_local! {
    static KIND: Cell<Kind> = const { Cell::new(Kind::Dialog) };
    static OPEN: Cell<Option<State<bool>>> = const { Cell::new(None) };
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
fn fixture() {
    let open = remember(|| false);
    OPEN.set(Some(open));
    material_theme().child(move || match KIND.get() {
        Kind::Dialog => {
            dialog_provider()
                .is_open(open.get())
                .main_content(|| {
                    main_content();
                })
                .dialog_content(|| {
                    marker_content();
                });
        }
        Kind::Bottom => {
            bottom_sheet_provider()
                .is_open(open.get())
                .main_content(|| {
                    main_content();
                })
                .bottom_sheet_content(|| {
                    marker_content();
                });
        }
        kind => {
            let position = if matches!(kind, Kind::Start | Kind::StandardStart) {
                SideSheetPosition::Start
            } else {
                SideSheetPosition::End
            };
            if matches!(kind, Kind::StandardStart | Kind::StandardEnd) {
                standard_side_sheet_provider()
                    .is_open(open.get())
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
                .is_open(open.get())
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

fn marker(renderer: &HeadlessRenderer) -> Option<Node> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some("Motion marker"))
        .map(|(_, n)| n)
}

fn green(renderer: &HeadlessRenderer, node: &Node) -> u8 {
    let rect = node.bounds().unwrap();
    let width = renderer.physical_size().width as usize;
    let height = renderer.physical_size().height as usize;
    let x = ((rect.x0 + rect.x1) * 0.5) as usize;
    let y = ((rect.y0 + rect.y1) * 0.5) as usize;
    assert!(x < width && y < height);
    renderer.read_pixels().unwrap()[(y * width + x) * 4 + 1]
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
        KIND.set(kind);
        for scale in [1.0, 1.5] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 800,
                    height: 600,
                    scale_factor: scale,
                    sample_count: samples,
                    frame_time: Duration::from_millis(10),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            let baseline = renderer.read_pixels().unwrap();
            let open = OPEN.get().unwrap();
            open.set(true);
            renderer.render_frames(8);
            let entering = marker(&renderer).unwrap();
            let entering_bounds = entering.bounds().unwrap();
            let entering_opacity =
                matches!(kind, Kind::Dialog).then(|| green(&renderer, &entering));
            for target in [false, true, false, true] {
                let before = marker(&renderer).unwrap();
                let rect = before.bounds().unwrap();
                let opacity = matches!(kind, Kind::Dialog).then(|| green(&renderer, &before));
                open.set(target);
                renderer.step_frame();
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
                renderer.render_frames(2);
            }
            renderer.render_frames(80);
            let full = renderer.read_pixels().unwrap();
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
            renderer.render_frames(5);
            open.set(true);
            renderer.render_frames(80);
            assert!(
                renderer.read_pixels().unwrap() == full,
                "{kind:?}: reopened pixels differ"
            );
            open.set(false);
            renderer.render_frames(80);
            assert!(marker(&renderer).is_none());
            assert!(
                renderer.read_pixels().unwrap() == baseline,
                "{kind:?}: exit did not restore the scene"
            );
            if is_sheet {
                renderer.resize(320, 600);
                renderer.render_frames(4);
                open.set(true);
                renderer.render_frames(80);
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
