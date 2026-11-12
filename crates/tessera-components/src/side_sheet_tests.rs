//! Intrinsic side-sheet host geometry regressions.
//!
//! ## Usage
//!
//! Keep modal and standard supporting panels usable inside scrolling content.

use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, Modifier, PxPosition, RenderSlot, State, remember,
    system_back::{BackSwipeEdge, SystemBackEvent},
    tessera,
    testing::UiTest,
};

use crate::{
    boxed::boxed,
    column::column,
    navigation_drawer::navigation_drawer_item,
    navigation_rail::{navigation_rail, navigation_rail_item},
    theme::material_theme,
};

use super::{
    SideSheetController, SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider,
};

#[derive(Clone, Default)]
struct FixtureOutput {
    open: Option<State<SideSheetController>>,
    dimensions: Option<State<(Dp, Dp)>>,
}

fn tag(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.to_owned()),
        merge_descendants: true,
        ..Default::default()
    })
}

#[tessera]
fn fixture(
    case: (bool, SideSheetPosition, i32),
    output: tessera_ui::testing::TestProbe<FixtureOutput>,
) {
    let (modal, position, width) = case;
    let open = remember(|| SideSheetController::new(true));
    output.with_mut(|output| output.open = Some(open));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().width(Dp(700.0)))
            .children(move || {
                let main = RenderSlot::new(move || {
                    boxed()
                        .modifier(tag("host").size(Dp(f64::from(width)), Dp(480.0)))
                        .children(|| {});
                });
                let sheet = RenderSlot::new(|| {
                    boxed().modifier(tag("sheet").fill_max_size()).children(|| {
                        navigation_rail().expanded(true).content(move |rail_scope| {
                            for index in 0..3 {
                                boxed()
                                    .modifier(tag(&format!("destination-{index}")))
                                    .children(move || {
                                        navigation_rail_item(rail_scope)
                                            .label(format!("Destination {index}"));
                                    });
                            }
                        });
                    });
                });
                if modal {
                    modal_side_sheet_provider()
                        .controller(open)
                        .position(position)
                        .on_close_request(move || open.with_mut(SideSheetController::close))
                        .main_content_shared(main)
                        .side_sheet_content_shared(sheet);
                } else {
                    standard_side_sheet_provider()
                        .controller(open)
                        .position(position)
                        .on_close_request(move || open.with_mut(SideSheetController::close))
                        .main_content_shared(main)
                        .side_sheet_content_shared(sheet);
                }
            });
    });
}

#[test]
fn intrinsic_hosts_bound_side_sheets_and_keep_destinations_separate_and_scrims_clickable() {
    crate::test_utils::run_isolated_test(
        "side_sheet::host_tests::intrinsic_hosts_bound_side_sheets_and_keep_destinations_separate_and_scrims_clickable",
        || {
            for modal in [true, false] {
                for position in [SideSheetPosition::Start, SideSheetPosition::End] {
                    for width in [320, 600] {
                        let case = (modal, position, width);
                        let output = tessera_ui::testing::TestProbe::default();
                        let mut session = UiTest::new()
                            .content(|| fixture(case, output.clone()))
                            .viewport(800, 600);
                        session.frame();
                        let snapshot = session.advance(Duration::from_millis(1_000));
                        snapshot.node("host").size(width, 480);
                        let sheet = snapshot.node("sheet").bounds();
                        assert_eq!(sheet.width.0, width.min(400) - 48);
                        assert_eq!(
                            sheet.height.0, 432,
                            "sheet height must come from its measured host"
                        );
                        let expected_x = if position == SideSheetPosition::End {
                            (width - 400).max(0) + 24
                        } else {
                            24
                        };
                        assert_eq!(sheet.x.0, expected_x);
                        let mut previous_bottom = sheet.y.0;
                        for index in 0..3 {
                            let item = snapshot.node(&format!("destination-{index}")).bounds();
                            assert!(
                                item.height.0 >= 48,
                                "navigation targets must retain usable height"
                            );
                            assert!(item.y.0 >= previous_bottom, "destinations must not overlap");
                            previous_bottom = item.y.0 + item.height.0;
                        }
                        if modal && width == 600 {
                            let x = if position == SideSheetPosition::Start {
                                550.0
                            } else {
                                50.0
                            };
                            let point = PxPosition::from_f32_xy(x, 300.0);
                            session.click_at(point);
                            assert!(
                                !output
                                    .get()
                                    .open
                                    .expect("mounted sheet visibility")
                                    .with(SideSheetController::is_open),
                                "the full measured host must have a dismissible scrim"
                            );
                        }
                    }
                }
            }
        },
    );
}

#[test]
fn modal_sheet_previews_back_without_closing_until_commit() {
    let output = tessera_ui::testing::TestProbe::default();
    let mut session =
        UiTest::new().content(|| fixture((true, SideSheetPosition::Start, 600), output.clone()));
    let initial = session.frame().node("sheet").bounds();
    let open = output.get().open.expect("mounted sheet");
    session.dispatch_system_back(SystemBackEvent::Started(BackSwipeEdge::Left));
    session.dispatch_system_back(SystemBackEvent::Progress {
        progress: 0.5,
        edge: BackSwipeEdge::Left,
    });
    let preview = session
        .advance(Duration::from_millis(16))
        .node("sheet")
        .bounds();
    assert!(preview.x < initial.x);
    assert_eq!(preview.width, initial.width);
    assert!(session.run(|| open.with(SideSheetController::is_open)));

    session.dispatch_system_back(SystemBackEvent::Cancelled);
    session.advance(Duration::from_millis(16));
    session.advance(Duration::from_millis(16));
    let restored = session
        .advance(Duration::from_millis(952))
        .node("sheet")
        .bounds();
    assert_eq!(restored, initial);
    session.dispatch_system_back(SystemBackEvent::Committed);
    assert!(!session.run(|| open.with(SideSheetController::is_open)));
}

#[tessera]
fn configured_fixture(
    case: (bool, SideSheetPosition, i32),
    output: tessera_ui::testing::TestProbe<FixtureOutput>,
) {
    let (_, _, width) = case;
    let open = remember(|| SideSheetController::new(true));
    let dimensions = remember(|| (Dp(360.0), Dp::ZERO));
    output.with_mut(|output| output.open = Some(open));
    output.with_mut(|output| output.dimensions = Some(dimensions));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                boxed()
                    .modifier(tag("configured-host").size(Dp(f64::from(width)), Dp(480.0)))
                    .children(move || {
                        modal_side_sheet_provider()
                            .controller(open)
                            .position(SideSheetPosition::Start)
                            .max_width(dimensions.with(|value| value.0))
                            .content_padding(dimensions.with(|value| value.1))
                            .on_close_request(move || open.with_mut(SideSheetController::close))
                            .main_content(|| {
                                boxed().modifier(Modifier::new().fill_max_size());
                            })
                            .side_sheet_content(|| {
                                column()
                                    .modifier(
                                        tag("configured-sheet")
                                            .fill_max_size()
                                            .padding_all(Dp(12.0)),
                                    )
                                    .children(|| {
                                        navigation_drawer_item()
                                            .label("Inbox")
                                            .modifier(tag("configured-destination"));
                                    });
                            });
                    });
            });
    });
}

#[test]
fn configured_modal_width_and_padding_remeasure_the_actual_sheet_and_preserve_scrim_boundary() {
    crate::test_utils::run_isolated_test(
        "side_sheet::host_tests::configured_modal_width_and_padding_remeasure_the_actual_sheet_and_preserve_scrim_boundary",
        || {
            for width in [600, 320] {
                let case = (true, SideSheetPosition::Start, width);
                let output = tessera_ui::testing::TestProbe::default();
                let mut session = UiTest::new()
                    .content(|| configured_fixture(case, output.clone()))
                    .viewport(800, 600);
                session.frame();
                let initial = session.advance(Duration::from_millis(1_000));
                initial.node("configured-host").size(width, 480);
                initial
                    .node("configured-sheet")
                    .size(width.min(360), 480)
                    .position(0, 0);
                initial
                    .node("configured-destination")
                    .size(width.min(360) - 24, 56)
                    .position(12, 12);
                let dimensions = output.get().dimensions.expect("mounted sheet dimensions");
                dimensions.with_mut(|value| value.0 = Dp(300.0));
                let resized = session.frames(1);
                resized.node("configured-sheet").size(300, 480);
                resized.node("configured-destination").size(276, 56);
                dimensions.with_mut(|value| value.1 = Dp(16.0));
                let padded = session.frames(1);
                padded.node("configured-sheet").rect(16, 16, 268, 448);
                padded.node("configured-destination").rect(28, 28, 244, 56);
                if width == 600 {
                    dimensions.set((Dp(360.0), Dp::ZERO));
                    session.frames(1);
                    session.click_at(PxPosition::from_f32_xy(355.0, 240.0));
                    assert!(
                        output
                            .get()
                            .open
                            .expect("mounted sheet visibility")
                            .with(SideSheetController::is_open),
                        "the sheet's blank ending inset must still block scrim activation"
                    );
                    session.click_at(PxPosition::from_f32_xy(380.0, 240.0));
                    assert!(
                        !output
                            .get()
                            .open
                            .expect("mounted sheet visibility")
                            .with(SideSheetController::is_open),
                        "the old 360–400dp sheet band must now belong to the scrim"
                    );
                }
            }
        },
    );
}

#[test]
fn configured_modal_dimensions_keep_nonfinite_and_negative_inputs_out_of_constraints() {
    crate::test_utils::run_isolated_test(
        "side_sheet::host_tests::configured_modal_dimensions_keep_nonfinite_and_negative_inputs_out_of_constraints",
        || {
            let case = (true, SideSheetPosition::Start, 600);
            let output = tessera_ui::testing::TestProbe::default();
            let mut session = UiTest::new()
                .content(|| configured_fixture(case, output.clone()))
                .viewport(800, 600);
            session.frame();
            session.advance(Duration::from_millis(1_000));
            let dimensions = output.get().dimensions.expect("mounted sheet dimensions");
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                dimensions.set((Dp(value), Dp(value)));
                let snapshot = session.frames(1);
                snapshot.node("configured-sheet").rect(24, 24, 352, 432);
            }
            dimensions.set((Dp(360.0), Dp(-16.0)));
            let snapshot = session.frames(1);
            snapshot.node("configured-sheet").rect(0, 0, 360, 480);
        },
    );
}
