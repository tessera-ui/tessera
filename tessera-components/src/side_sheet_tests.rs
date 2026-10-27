//! Intrinsic side-sheet host geometry regressions.
//!
//! ## Usage
//!
//! Keep modal and standard supporting panels usable inside scrolling content.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    CursorEventContent, Dp, GestureState, MOUSE_POINTER_ID, Modifier, PointerChange,
    PressKeyEventType, PxPosition, RenderSlot, State, remember, tessera,
    testing::{
        layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
        start_layout_test_session,
    },
    time::Instant,
};

use crate::{
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_rail::{navigation_rail, navigation_rail_item},
    theme::material_theme,
};

use super::{SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider};

thread_local! {
    static CASE: Cell<(bool, SideSheetPosition, i32)> = const { Cell::new((true, SideSheetPosition::Start, 600)) };
    static OPEN: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

fn tag(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.to_owned()),
        ..Default::default()
    })
}

#[tessera]
fn fixture() {
    let (modal, position, width) = CASE.get();
    let open = remember(|| true);
    OPEN.set(Some(open));
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
                        navigation_rail().expanded(true).content(|| {
                            for index in 0..3 {
                                boxed()
                                    .modifier(tag(&format!("destination-{index}")))
                                    .children(move || {
                                        navigation_rail_item()
                                            .label(format!("Destination {index}"));
                                    });
                            }
                        });
                    });
                });
                if modal {
                    modal_side_sheet_provider()
                        .is_open(open.get())
                        .position(position)
                        .on_close_request(move || open.set(false))
                        .main_content_shared(main)
                        .side_sheet_content_shared(sheet);
                } else {
                    standard_side_sheet_provider()
                        .is_open(open.get())
                        .position(position)
                        .on_close_request(move || open.set(false))
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
                        CASE.set((modal, position, width));
                        let mut session =
                            start_layout_test_session(layout_test(fixture).viewport_px(800, 600));
                        pump_layout_test_session(&mut session, 0);
                        let snapshot = pump_layout_test_session(&mut session, 1_000_000_000);
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
                            let origin = Instant::now();
                            for (index, content) in [
                                CursorEventContent::Pressed(PressKeyEventType::Left),
                                CursorEventContent::Released(PressKeyEventType::Left),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let nanos = 1_100_000_000 + index as u64 * 16_666_667;
                                let change = PointerChange::new(
                                    origin + Duration::from_nanos(nanos),
                                    MOUSE_POINTER_ID,
                                    content,
                                    GestureState::TapCandidate,
                                )
                                .with_position(point);
                                pump_layout_test_session_with_input(
                                    &mut session,
                                    nanos,
                                    vec![change],
                                    Some(point),
                                );
                            }
                            assert!(
                                !OPEN.get().unwrap().get(),
                                "the full measured host must have a dismissible scrim"
                            );
                        }
                    }
                }
            }
        },
    );
}
