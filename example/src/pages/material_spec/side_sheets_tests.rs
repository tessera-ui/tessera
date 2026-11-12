use tessera_components::theme::material_theme;
use tessera_ui::{
    accesskit::Role,
    headless::{HeadlessConfig, HeadlessRenderer},
    testing::{layout_test, pump_layout_test_session, start_layout_test_session},
};

use super::{
    figures::{self, measurement_tag},
    *,
};

const FRAME: u64 = 16_666_667;

#[tessera]
fn measurement_fixture(modal: bool) {
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                figures::measurement_figure(modal);
            });
    });
}

#[tessera]
fn anatomy_fixture(modal: bool) {
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                figures::anatomy(modal, "fixture");
            });
    });
}

fn glyph_center(rect: tessera_ui::PxRect) -> (i32, i32) {
    (rect.x.0 + rect.width.0 / 2, rect.y.0 + rect.height.0 / 2)
}

#[test]
fn measurement_figures_place_real_header_glyphs_at_the_official_insets() {
    for modal in [false, true] {
        for (width, sheet_width) in [(720, 400), (320, 256)] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    measurement_fixture(modal);
                })
                .viewport_px(width, 800),
            );
            pump_layout_test_session(&mut session, 0);
            pump_layout_test_session(&mut session, FRAME);
            let snapshot = pump_layout_test_session(&mut session, FRAME * 2);
            let tag = |name| measurement_tag(modal, name);
            snapshot.node(&tag("figure")).size(width as i32, 480);
            snapshot.node(&tag("container")).size(sheet_width, 384);
            let container = snapshot.node(&tag("container")).bounds();
            let headline = snapshot.node(&tag("headline")).bounds();
            let (close_x, close_y) = glyph_center(snapshot.node(&tag("close")).bounds());
            assert_eq!(
                container.x.0 + container.width.0 - (close_x + 12),
                24,
                "close glyph end padding"
            );
            assert_eq!(close_y - 12 - container.y.0, 24, "close glyph top inset");
            assert_eq!(close_x - 12 - (headline.x.0 + headline.width.0), 12);
            if modal {
                let (back_x, back_y) = glyph_center(snapshot.node(&tag("back")).bounds());
                assert_eq!(back_x - 12 - container.x.0, 16, "start padding with icon");
                assert_eq!(headline.x.0 - (back_x + 12), 12);
                assert_eq!(back_y, close_y);
                let actions = snapshot.node(&tag("actions")).bounds();
                assert_eq!(actions.height.0, 72);
                assert_eq!(
                    actions.y.0 + actions.height.0,
                    container.y.0 + container.height.0
                );
            } else {
                assert_eq!(headline.x.0 - container.x.0, 24, "start padding");
            }
        }
    }
}

#[test]
fn anatomy_figures_number_every_legend_entry() {
    for modal in [false, true] {
        let snapshot = layout_test(move || {
            anatomy_fixture(modal);
        })
        .viewport_px(1000, 900)
        .run();
        let count = figures::anatomy_legend(modal).len();
        let mut previous = None;
        for number in 1..=count {
            let bounds = snapshot
                .node(&format!("side-marker-fixture-{number}"))
                .size(24, 32)
                .bounds();
            assert!(bounds.x.0 >= 0 && bounds.x.0 + 24 <= 1000);
            assert_ne!(Some((bounds.x.0, bounds.y.0)), previous);
            previous = Some((bounds.x.0, bounds.y.0));
        }
    }
}

#[test]
fn live_modal_side_sheet_covers_the_app_bar_and_closes_when_its_route_exits() {
    let (width, height) = (907, 871);
    let navigation = tessera_ui::testing::TestProbe::default();
    let observed = navigation.clone();
    let mut session = start_layout_test_session(
        layout_test(move || {
            crate::app::app_fixture(observed.clone());
        })
        .viewport_px(width, height),
    );
    let mut frame = 0;
    let mut pump = |session: &mut _, frames: u64| {
        let mut snapshot = None;
        for _ in 0..frames {
            snapshot = Some(pump_layout_test_session(session, frame));
            frame += FRAME;
        }
        snapshot.unwrap()
    };
    pump(&mut session, 2);
    let navigation = navigation.get().expect("mounted app navigation");
    navigation.router.with_mut(|router| {
        router.reset(
            crate::pages::material_components::MaterialComponentsPageDestination {
                sessions: navigation.sessions,
            },
        );
        router.push(
            crate::pages::material_components::MaterialArticlePageDestination {
                index: super::super::catalog::component_index(34).unwrap(),
                sessions: navigation.sessions,
            },
        );
    });
    pump(&mut session, 2);
    let live = navigation.sessions.expect("root app sessions").side_sheet;
    assert!(live.route_active());
    assert!(
        !live.can_open(),
        "the article must not open its modal over the open directory sheet"
    );
    navigation.directory.with_mut(|directory| directory.close());
    pump(&mut session, 40);
    assert!(live.can_open());
    live.open.with_mut(SideSheetController::open);
    let snapshot = pump(&mut session, 40);
    let sheet = snapshot.node("side-sheet-live-content").bounds();
    assert_eq!(sheet.y.0, 0, "the live modal must start above the app bar");
    assert_eq!(sheet.height.0, height as i32);
    assert_eq!(sheet.x.0 + sheet.width.0, width as i32);
    assert_eq!(sheet.width.0, 400);
    navigation
        .router
        .with_mut(tessera_shard::RouterController::pop);
    pump(&mut session, 2);
    assert!(
        !live.open.with(SideSheetController::is_open),
        "leaving the article must close the root modal"
    );
    assert!(!live.can_open());
}

#[test]
#[ignore = "Requires a native GPU adapter for complete side-sheet measurement accessibility snapshots"]
fn measurement_figures_publish_real_headlines_content_and_controls_in_both_variants() {
    for modal in [false, true] {
        for width in [720, 320] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || {
                    measurement_fixture(modal);
                }),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width,
                    height: 800,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(8);
            let snapshot = renderer.accessibility_snapshot().unwrap();
            for label in ["Headline", "Labels", "Events", "Personal", "Projects"] {
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| crate::has_label(&node, label) && !node.is_hidden()),
                    "missing measured sheet content: {label}"
                );
            }
            let labels: &[&str] = if modal {
                &[
                    "Side sheet figure back",
                    "Side sheet figure close",
                    "Save filters",
                    "Cancel filters",
                ]
            } else {
                &["Side sheet figure close"]
            };
            for label in labels {
                let node = snapshot
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == Role::Button
                            && crate::has_label(&node, *label)
                            && !node.is_hidden()
                    })
                    .unwrap_or_else(|| panic!("missing measured sheet button: {label}"));
                let bounds = node.1.bounds().unwrap();
                assert!(bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width));
                assert!(bounds.y0 >= 0.0 && bounds.y1 <= 480.0);
            }
            let mut expected = vec!["Dimension 24", "Dimension 12"];
            if modal {
                expected.push("Dimension 16");
            }
            for label in expected {
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| crate::has_label(&node, label)),
                    "missing measurement label: {label}"
                );
            }
        }
    }
}
