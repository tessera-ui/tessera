use std::cell::{Cell, RefCell};

use crate::{
    ComputedData, Constraint, CursorEventContent, FocusModifierExt, GestureState,
    LayoutCoordinates, LayoutModifierNode, LayoutResult, Modifier, PointerChange, PointerEventPass,
    PointerInput, PointerInputModifierNode, Px, PxPosition, ScrollDeltaUnit, ScrollEventContent,
    ScrollEventSource, State,
    layout::{MeasureScope, layout},
    testing::{
        layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
        start_layout_test_session,
    },
    time::Instant,
};

use super::{FloatRect, scaled_box};

thread_local! {
    static COORDINATES: RefCell<Vec<LayoutCoordinates>> = const { RefCell::new(Vec::new()) };
    static INPUT: RefCell<Vec<(bool, Option<PxPosition>, Option<PxPosition>, f32)>> = const { RefCell::new(Vec::new()) };
    static PARENT_OFFSET: Cell<Option<State<PxPosition>>> = const { Cell::new(None) };
    static CHILD_OFFSET: Cell<Option<State<PxPosition>>> = const { Cell::new(None) };
    static CHILD_TRANSFORMS: Cell<usize> = const { Cell::new(0) };
    static OBSERVER_VISIBLE: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static SPACE_RECTS: RefCell<Vec<FloatRect>> = const { RefCell::new(Vec::new()) };
    static OUTER_SCALE: Cell<Option<State<f32>>> = const { Cell::new(None) };
    static INNER_SCALE: Cell<Option<State<f32>>> = const { Cell::new(None) };
    static MARK_SPACE: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static NESTED_SPACE: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CONTENT_SIZE: Cell<Option<State<ComputedData>>> = const { Cell::new(None) };
    static SPACE_OFFSET: Cell<Option<State<PxPosition>>> = const { Cell::new(None) };
    static COUNTER_OFFSET: Cell<Option<State<PxPosition>>> = const { Cell::new(None) };
    static SIZES: RefCell<Vec<crate::PxSize>> = const { RefCell::new(Vec::new()) };
    static LATE_SIZES: RefCell<Vec<crate::PxSize>> = const { RefCell::new(Vec::new()) };
    static LATE_OBSERVER: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

#[derive(Clone, PartialEq)]
struct SizeOnlyRoot(State<bool>);

impl LayoutModifierNode for SizeOnlyRoot {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let mut result = LayoutResult::new(ComputedData {
            width: Px(300),
            height: Px(200),
        });
        if self.0.get() {
            let child = scope.children()[0].measure(scope, &Constraint::NONE)?;
            result.place_child(child, PxPosition::from_f32_xy(10.25, 20.5));
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct SizeOnlyContent(State<ComputedData>);

impl LayoutModifierNode for SizeOnlyContent {
    fn measure(
        &self,
        _scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        Ok(LayoutResult::new(self.0.get()))
    }
}

#[crate::tessera(crate)]
fn size_only_observer_fixture() {
    let parent = crate::remember(|| PxPosition::ZERO);
    let child = crate::remember(|| PxPosition::ZERO);
    let visible = crate::remember(|| true);
    let scale = crate::remember(|| 1.0_f32);
    let late = crate::remember(|| false);
    let size = crate::remember(|| ComputedData {
        width: Px(150),
        height: Px(35),
    });
    PARENT_OFFSET.set(Some(parent));
    CHILD_OFFSET.set(Some(child));
    OBSERVER_VISIBLE.set(Some(visible));
    OUTER_SCALE.set(Some(scale));
    CONTENT_SIZE.set(Some(size));
    LATE_OBSERVER.set(Some(late));
    layout()
        .policy(SizeOnlyRoot(visible))
        .modifier(Modifier::new().push_placement(StateTranslation(parent, false)))
        .child(move || {
            scaled_box(scale.get()).child(move || {
                let modifier = Modifier::new()
                    .push_placement(StateTranslation(child, false))
                    .on_layout_size_changed(|size| SIZES.with_borrow_mut(|sizes| sizes.push(size)));
                let modifier = if late.get() {
                    modifier.on_layout_size_changed(|size| {
                        LATE_SIZES.with_borrow_mut(|sizes| sizes.push(size))
                    })
                } else {
                    modifier
                };
                layout().policy(SizeOnlyContent(size)).modifier(modifier);
            });
        });
}

#[test]
fn size_observers_ignore_movement_and_scaling_and_publish_resized_revealed_and_new_subscribers() {
    SIZES.with_borrow_mut(Vec::clear);
    LATE_SIZES.with_borrow_mut(Vec::clear);
    let mut session =
        start_layout_test_session(layout_test(size_only_observer_fixture).viewport_px(300, 200));
    pump_layout_test_session(&mut session, 0);
    assert_eq!(
        SIZES.with_borrow(|sizes| sizes.clone()),
        vec![crate::PxSize::new(Px(150), Px(35))]
    );
    PARENT_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(128.5, -0.25));
    pump_layout_test_session(&mut session, 16_666_667);
    assert!(session.diagnostics().unwrap().geometry_nodes_updated > 0);
    CHILD_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(3.75, 4.0));
    pump_layout_test_session(&mut session, 33_333_334);
    OUTER_SCALE.get().unwrap().set(0.625);
    pump_layout_test_session(&mut session, 50_000_001);
    assert_eq!(SIZES.with_borrow(Vec::len), 1);
    CONTENT_SIZE.get().unwrap().set(ComputedData {
        width: Px(220),
        height: Px(55),
    });
    pump_layout_test_session(&mut session, 66_666_668);
    assert_eq!(
        SIZES.with_borrow(|sizes| *sizes.last().unwrap()),
        crate::PxSize::new(Px(220), Px(55))
    );
    assert_eq!(SIZES.with_borrow(Vec::len), 2);
    OBSERVER_VISIBLE.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 83_333_335);
    CONTENT_SIZE.get().unwrap().set(ComputedData {
        width: Px(250),
        height: Px(60),
    });
    pump_layout_test_session(&mut session, 100_000_002);
    assert_eq!(SIZES.with_borrow(Vec::len), 2);
    OBSERVER_VISIBLE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 116_666_669);
    assert_eq!(SIZES.with_borrow(Vec::len), 3);
    assert_eq!(
        SIZES.with_borrow(|sizes| *sizes.last().unwrap()),
        crate::PxSize::new(Px(250), Px(60))
    );
    LATE_OBSERVER.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 133_333_336);
    assert_eq!(SIZES.with_borrow(Vec::len), 3);
    assert_eq!(
        LATE_SIZES.with_borrow(|sizes| sizes.clone()),
        vec![crate::PxSize::new(Px(250), Px(60))]
    );
    OBSERVER_VISIBLE.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 150_000_003);
    CONTENT_SIZE.get().unwrap().set(ComputedData {
        width: Px(310),
        height: Px(70),
    });
    pump_layout_test_session(&mut session, 166_666_670);
    assert_eq!(SIZES.with_borrow(Vec::len), 3);
    assert_eq!(LATE_SIZES.with_borrow(Vec::len), 1);
    OBSERVER_VISIBLE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 183_333_337);
    assert_eq!(SIZES.with_borrow(Vec::len), 4);
    assert_eq!(
        LATE_SIZES.with_borrow(|sizes| *sizes.last().unwrap()),
        crate::PxSize::new(Px(310), Px(70))
    );
    LATE_OBSERVER.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 200_000_004);
    PARENT_OFFSET.get().unwrap().set(PxPosition::ZERO);
    pump_layout_test_session(&mut session, 216_666_671);
    assert_eq!(SIZES.with_borrow(Vec::len), 4);
    assert_eq!(LATE_SIZES.with_borrow(Vec::len), 2);
    OBSERVER_VISIBLE.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 233_333_338);
    OBSERVER_VISIBLE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 250_000_005);
    assert_eq!(SIZES.with_borrow(Vec::len), 5);
    assert_eq!(
        SIZES.with_borrow(|sizes| *sizes.last().unwrap()),
        crate::PxSize::new(Px(310), Px(70))
    );
    assert_eq!(LATE_SIZES.with_borrow(Vec::len), 2);
}

#[derive(Clone, PartialEq)]
struct SpaceContent(State<bool>, State<ComputedData>);

impl LayoutModifierNode for SpaceContent {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let size = self.1.get();
        let child =
            scope.children()[0].measure(scope, &Constraint::exact(size.width, size.height))?;
        let mut result = LayoutResult::new(ComputedData {
            width: Px(300),
            height: Px(200),
        });
        if self.0.get() {
            result.place_child(child, PxPosition::from_f32_xy(10.25, 20.5));
        }
        Ok(result)
    }
}

#[crate::tessera(crate)]
fn relative_observer_fixture() {
    let parent = crate::remember(|| PxPosition::ZERO);
    let child = crate::remember(|| PxPosition::from_f32_xy(2.25, 3.5));
    let visible = crate::remember(|| true);
    let outer = crate::remember(|| 1.0_f32);
    let inner = crate::remember(|| 0.625_f32);
    let marked = crate::remember(|| true);
    let nested = crate::remember(|| false);
    let space_offset = crate::remember(|| PxPosition::ZERO);
    let counter_offset = crate::remember(|| PxPosition::ZERO);
    let size = crate::remember(|| ComputedData {
        width: Px(150),
        height: Px(35),
    });
    PARENT_OFFSET.set(Some(parent));
    CHILD_OFFSET.set(Some(child));
    OBSERVER_VISIBLE.set(Some(visible));
    OUTER_SCALE.set(Some(outer));
    INNER_SCALE.set(Some(inner));
    MARK_SPACE.set(Some(marked));
    NESTED_SPACE.set(Some(nested));
    SPACE_OFFSET.set(Some(space_offset));
    COUNTER_OFFSET.set(Some(counter_offset));
    CONTENT_SIZE.set(Some(size));
    layout()
        .policy(Anchor)
        .modifier(Modifier::new().push_placement(StateTranslation(parent, false)))
        .child(move || {
            scaled_box(outer.get()).child(move || {
                let modifier = if marked.get() {
                    Modifier::new().layout_coordinate_space()
                } else {
                    Modifier::new()
                };
                layout()
                    .policy(Anchor)
                    .modifier(modifier.push_placement(StateTranslation(space_offset, false)))
                    .child(move || {
                        scaled_box(inner.get()).child(move || {
                            let modifier = if nested.get() {
                                Modifier::new().layout_coordinate_space()
                            } else {
                                Modifier::new()
                            };
                            layout()
                                .policy(SpaceContent(visible, size))
                                .modifier(
                                    modifier
                                        .push_placement(StateTranslation(counter_offset, false)),
                                )
                                .child(move || {
                                    layout().modifier(
                                        Modifier::new()
                                            .push_placement(StateTranslation(child, false))
                                            .on_layout_rect_in_space(|rect| {
                                                SPACE_RECTS
                                                    .with_borrow_mut(|rects| rects.push(rect));
                                            })
                                            .on_layout_coordinates(|coordinates| {
                                                COORDINATES.with_borrow_mut(|items| {
                                                    items.push(coordinates)
                                                });
                                            }),
                                    );
                                });
                        });
                    });
            });
        });
}

#[test]
fn relative_observers_ignore_space_motion_and_track_local_changes_and_reparenting() {
    SPACE_RECTS.with_borrow_mut(Vec::clear);
    COORDINATES.with_borrow_mut(Vec::clear);
    let mut session =
        start_layout_test_session(layout_test(relative_observer_fixture).viewport_px(1000, 800));
    pump_layout_test_session(&mut session, 0);
    let expected = FloatRect {
        x: 18.0625,
        y: 35.5,
        width: 93.75,
        height: 21.875,
    };
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| rects.clone()),
        vec![expected]
    );
    for frame in 1..=5 {
        PARENT_OFFSET
            .get()
            .unwrap()
            .set(PxPosition::from_f32_xy(65536.25 * frame as f32, -0.5));
        pump_layout_test_session(&mut session, frame * 16_666_667);
    }
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), 1);
    assert_eq!(COORDINATES.with_borrow(Vec::len), 6);
    OUTER_SCALE.get().unwrap().set(0.75);
    pump_layout_test_session(&mut session, 100_000_000);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), 1);
    INNER_SCALE.get().unwrap().set(0.5);
    pump_layout_test_session(&mut session, 116_666_667);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        FloatRect {
            x: 16.5,
            y: 32.5,
            width: 75.0,
            height: 17.5
        }
    );
    CHILD_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(3.75, 4.0));
    pump_layout_test_session(&mut session, 133_333_334);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        FloatRect {
            x: 17.25,
            y: 32.75,
            width: 75.0,
            height: 17.5
        }
    );
    CONTENT_SIZE.get().unwrap().set(ComputedData {
        width: Px(120),
        height: Px(40),
    });
    pump_layout_test_session(&mut session, 150_000_001);
    let resized = FloatRect {
        x: 17.25,
        y: 32.75,
        width: 60.0,
        height: 20.0,
    };
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        resized
    );
    let count = SPACE_RECTS.with_borrow(Vec::len);
    OBSERVER_VISIBLE.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 166_666_668);
    PARENT_OFFSET.get().unwrap().set(PxPosition::ZERO);
    pump_layout_test_session(&mut session, 183_333_335);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count);
    OBSERVER_VISIBLE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 200_000_002);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count + 1);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        resized
    );
    MARK_SPACE.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 216_666_669);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count + 2);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        COORDINATES.with_borrow(|items| items.last().unwrap().window_bounds())
    );
    MARK_SPACE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 233_333_336);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count + 3);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        resized
    );
    NESTED_SPACE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 250_000_003);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count + 4);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        FloatRect {
            x: 14.0,
            y: 24.5,
            width: 120.0,
            height: 40.0,
        }
    );
    NESTED_SPACE.get().unwrap().set(false);
    pump_layout_test_session(&mut session, 266_666_670);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count + 5);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        resized
    );
    let window_count = COORDINATES.with_borrow(Vec::len);
    SPACE_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(64.0, 0.0));
    COUNTER_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(-128.0, 0.0));
    pump_layout_test_session(&mut session, 283_333_337);
    assert_eq!(COORDINATES.with_borrow(Vec::len), window_count);
    assert_eq!(SPACE_RECTS.with_borrow(Vec::len), count + 6);
    assert_eq!(
        SPACE_RECTS.with_borrow(|rects| *rects.last().unwrap()),
        FloatRect {
            x: -46.75,
            y: 32.75,
            width: 60.0,
            height: 20.0,
        }
    );
}

#[derive(Clone, PartialEq)]
struct StateTranslation(State<PxPosition>, bool);

impl crate::PlacementModifierNode for StateTranslation {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        if self.1 {
            CHILD_TRANSFORMS.set(CHILD_TRANSFORMS.get() + 1);
        }
        position + self.0.get()
    }
}

#[crate::tessera(crate)]
fn inherited_translation_fixture() {
    let parent = crate::remember(|| PxPosition::ZERO);
    let child = crate::remember(|| PxPosition::from_f32_xy(2.25, 3.5));
    PARENT_OFFSET.set(Some(parent));
    CHILD_OFFSET.set(Some(child));
    layout()
        .policy(Anchor)
        .modifier(Modifier::new().push_placement(StateTranslation(parent, false)))
        .child(move || {
            layout().modifier(
                Modifier::new()
                    .push_placement(StateTranslation(child, true))
                    .on_layout_coordinates(|coordinates| {
                        COORDINATES.with_borrow_mut(|items| items.push(coordinates));
                    }),
            );
        });
}

#[test]
fn inherited_translation_reuses_local_placement_and_preserves_fractional_observers() {
    COORDINATES.with_borrow_mut(Vec::clear);
    CHILD_TRANSFORMS.set(0);
    let mut session =
        start_layout_test_session(layout_test(inherited_translation_fixture).viewport_px(300, 200));
    pump_layout_test_session(&mut session, 0);
    let initial = CHILD_TRANSFORMS.get();
    assert!(initial > 0);
    for frame in 1..=40 {
        let offset = if frame % 2 == 0 { 0.0 } else { 41.25 };
        PARENT_OFFSET
            .get()
            .unwrap()
            .set(PxPosition::from_f32_xy(-0.5, offset));
        pump_layout_test_session(&mut session, frame * 16_666_667);
        assert_eq!(
            session.diagnostics().unwrap().pointer_nodes_updated,
            0,
            "moving decorative layouts must not update the pointer index"
        );
        let bounds = COORDINATES.with_borrow(|items| items.last().unwrap().window_bounds());
        assert_eq!(bounds.x, 12.0);
        assert_eq!(bounds.y, 24.0 + offset);
    }
    assert_eq!(
        CHILD_TRANSFORMS.get(),
        initial,
        "An ancestor translation must not reevaluate unchanged local placement modifiers"
    );
    CHILD_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(3.75, 4.0));
    pump_layout_test_session(&mut session, 700_000_000);
    assert!(CHILD_TRANSFORMS.get() > initial);
    let bounds = COORDINATES.with_borrow(|items| items.last().unwrap().window_bounds());
    assert_eq!((bounds.x, bounds.y), (13.5, 24.5));
}

#[derive(Clone, PartialEq)]
struct PlaceWhenVisible(State<bool>);

impl LayoutModifierNode for PlaceWhenVisible {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let child = scope.children()[0].measure(scope, &Constraint::exact(Px(150), Px(35)))?;
        let mut result = LayoutResult::new(ComputedData {
            width: Px(300),
            height: Px(200),
        });
        if self.0.get() {
            result.place_child(child, PxPosition::from_f32_xy(10.25, 20.5));
        }
        Ok(result)
    }
}

#[crate::tessera(crate)]
fn hidden_position_observer_fixture() {
    let visible = crate::remember(|| true);
    let parent = crate::remember(|| PxPosition::ZERO);
    OBSERVER_VISIBLE.set(Some(visible));
    PARENT_OFFSET.set(Some(parent));
    layout()
        .policy(PlaceWhenVisible(visible))
        .modifier(Modifier::new().push_placement(StateTranslation(parent, false)))
        .child(|| {
            layout().modifier(Modifier::new().on_layout_coordinates(|coordinates| {
                COORDINATES.with_borrow_mut(|items| items.push(coordinates));
            }));
        });
}

#[test]
fn position_observer_resumes_after_hidden_ancestor_moves() {
    COORDINATES.with_borrow_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(hidden_position_observer_fixture).viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(COORDINATES.with_borrow(Vec::len), 1);
    OBSERVER_VISIBLE.get().unwrap().set(false);
    PARENT_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(0.0, 41.25));
    pump_layout_test_session(&mut session, 16_666_667);
    assert_eq!(COORDINATES.with_borrow(Vec::len), 1);
    PARENT_OFFSET
        .get()
        .unwrap()
        .set(PxPosition::from_f32_xy(0.0, 80.5));
    pump_layout_test_session(&mut session, 33_333_334);
    assert_eq!(COORDINATES.with_borrow(Vec::len), 1);
    OBSERVER_VISIBLE.get().unwrap().set(true);
    pump_layout_test_session(&mut session, 50_000_001);
    let coordinates = COORDINATES.with_borrow(|items| {
        assert_eq!(items.len(), 2);
        items[1]
    });
    let bounds = coordinates.window_bounds();
    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (10.25, 101.0, 150.0, 35.0)
    );
}

#[derive(Clone, PartialEq)]
struct Anchor;

impl LayoutModifierNode for Anchor {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let child = scope.children()[0].measure(scope, &Constraint::exact(Px(150), Px(35)))?;
        let mut result = LayoutResult::new(ComputedData {
            width: Px(300),
            height: Px(200),
        });
        result.place_child(child, PxPosition::from_f32_xy(10.25, 20.5));
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct Content;

impl LayoutModifierNode for Content {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        assert_eq!(
            *scope.parent_constraint().as_ref(),
            Constraint::exact(Px(240), Px(56))
        );
        Ok(LayoutResult::new(ComputedData {
            width: Px(240),
            height: Px(56),
        }))
    }
}

struct ObserveInput(bool);

impl PointerInputModifierNode for ObserveInput {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let position = input.cursor_position_rel;
        let absolute = input.cursor_position_abs();
        for change in input.pointer_changes.iter_mut() {
            if let CursorEventContent::Scroll(scroll) = &mut change.content {
                INPUT.with_borrow_mut(|input| {
                    input.push((self.0, position, absolute, scroll.delta_y))
                });
                if self.0 {
                    scroll.delta_y -= 8.0;
                }
            }
        }
    }
}

#[crate::tessera(crate)]
fn scale_fixture() {
    layout()
        .policy(Anchor)
        .modifier(Modifier::new().push_pointer_input(ObserveInput(false)))
        .child(|| {
            scaled_box(0.625).child(|| {
                layout().policy(Content).modifier(
                    Modifier::new()
                        .push_pointer_input(ObserveInput(true))
                        .on_layout_coordinates(|coordinates| {
                            COORDINATES.with_borrow_mut(|items| items.push(coordinates));
                        }),
                );
            });
        });
}

#[test]
fn scaled_layout_keeps_local_dimensions_maps_actual_input_and_returns_window_scroll_remainder() {
    COORDINATES.with_borrow_mut(Vec::clear);
    INPUT.with_borrow_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| {
            scale_fixture();
        })
        .viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    let coordinates =
        COORDINATES.with_borrow(|items| *items.last().expect("Layout observer must run"));
    assert_eq!(coordinates.local_size.width, Px(240));
    assert_eq!(coordinates.local_size.height, Px(56));
    assert_eq!(
        coordinates.window_bounds(),
        FloatRect {
            x: 10.25,
            y: 20.5,
            width: 150.0,
            height: 35.0
        }
    );
    let observed = COORDINATES.with_borrow(Vec::len);
    pump_layout_test_session(&mut session, 1);
    assert_eq!(
        COORDINATES.with_borrow(Vec::len),
        observed,
        "An idle frame must not republish unchanged geometry"
    );
    let position = PxPosition::from_f32_xy(40.25, 35.5);
    let scroll = PointerChange::new(
        Instant::now(),
        crate::MOUSE_POINTER_ID,
        CursorEventContent::Scroll(ScrollEventContent {
            delta_x: 0.0,
            delta_y: 20.0,
            unit: ScrollDeltaUnit::Pixel,
            source: ScrollEventSource::Wheel,
        }),
        GestureState::default(),
    )
    .with_position(position);
    pump_layout_test_session_with_input(&mut session, 16_000_000, vec![scroll], Some(position));
    let samples = INPUT.with_borrow(Clone::clone);
    let child = samples
        .iter()
        .find(|sample| sample.0)
        .expect("Scaled child must receive the sample");
    assert_eq!(child.1, Some(PxPosition::from_f32_xy(48.0, 24.0)));
    assert_eq!(
        child.2,
        Some(position),
        "Canonical window pointer must remain unchanged"
    );
    assert_eq!(child.3, 32.0);
    let parent = samples
        .iter()
        .find(|sample| !sample.0)
        .expect("Ancestor must receive remaining wheel input");
    assert_eq!(
        parent.3, 15.0,
        "Eight local units consume exactly five window units"
    );
}
