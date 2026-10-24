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
