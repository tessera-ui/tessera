use crate::{
    ComputedData, Constraint, CursorEventContent, GestureState, LayoutCoordinates, LayoutPolicy,
    LayoutResult, Modifier, PointerChange, PointerEventPass, PointerInput,
    PointerInputModifierNode, Px, PxPosition, ScrollDeltaUnit, ScrollEventContent,
    ScrollEventSource, State,
    layout::{MeasureScope, Stack, layout},
    modifier::FocusModifierExt as _,
    testing::{
        layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
        start_layout_test_session,
    },
    time::Instant,
};

use super::{FloatRect, scaled_box};

type ObservedInput = (bool, Option<PxPosition>, Option<PxPosition>, f32, bool);

#[derive(Clone, Default, PartialEq)]
struct GeometryProbe {
    visible_bounds: crate::testing::TestProbe<Vec<Option<FloatRect>>>,
    coordinates: crate::testing::TestProbe<Vec<LayoutCoordinates>>,
    input: crate::testing::TestProbe<Vec<ObservedInput>>,
    parent_offset: crate::testing::TestProbe<Option<State<PxPosition>>>,
    child_offset: crate::testing::TestProbe<Option<State<PxPosition>>>,
    observer_visible: crate::testing::TestProbe<Option<State<bool>>>,
    space_rects: crate::testing::TestProbe<Vec<FloatRect>>,
    outer_scale: crate::testing::TestProbe<Option<State<f32>>>,
    inner_scale: crate::testing::TestProbe<Option<State<f32>>>,
    mark_space: crate::testing::TestProbe<Option<State<bool>>>,
    nested_space: crate::testing::TestProbe<Option<State<bool>>>,
    content_size: crate::testing::TestProbe<Option<State<ComputedData>>>,
    space_offset: crate::testing::TestProbe<Option<State<PxPosition>>>,
    counter_offset: crate::testing::TestProbe<Option<State<PxPosition>>>,
    sizes: crate::testing::TestProbe<Vec<crate::PxSize>>,
    late_sizes: crate::testing::TestProbe<Vec<crate::PxSize>>,
    late_observer: crate::testing::TestProbe<Option<State<bool>>>,
}

#[derive(Clone, PartialEq)]
struct SizeOnlyRoot(State<bool>);

impl LayoutPolicy for SizeOnlyRoot {
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

impl LayoutPolicy for SizeOnlyContent {
    fn measure(
        &self,
        _scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        Ok(LayoutResult::new(self.0.get()))
    }
}

#[crate::tessera]
fn size_only_observer_fixture(probe: GeometryProbe) {
    let parent = crate::remember(|| PxPosition::ZERO);
    let child = crate::remember(|| PxPosition::ZERO);
    let visible = crate::remember(|| true);
    let scale = crate::remember(|| 1.0_f32);
    let late = crate::remember(|| false);
    let size = crate::remember(|| ComputedData {
        width: Px(150),
        height: Px(35),
    });
    probe.parent_offset.set(Some(parent));
    probe.child_offset.set(Some(child));
    probe.observer_visible.set(Some(visible));
    probe.outer_scale.set(Some(scale));
    probe.content_size.set(Some(size));
    probe.late_observer.set(Some(late));
    crate::layout::layout(SizeOnlyRoot(visible))
        .modifier(Modifier::new().push_placement(StateTranslation(parent)))
        .child(move || {
            let probe = probe.clone();
            scaled_box(scale.get()).child(move || {
                let probe = probe.clone();
                let modifier = Modifier::new()
                    .push_placement(StateTranslation(child))
                    .on_layout_size_changed(probe.sizes.recorder())
                    .on_visible_window_bounds(probe.visible_bounds.recorder());
                let modifier = if late.get() {
                    modifier.on_layout_size_changed(probe.late_sizes.recorder())
                } else {
                    modifier
                };
                crate::layout::layout(SizeOnlyContent(size)).modifier(modifier);
            });
        });
}

#[test]
fn visible_bounds_observers_clip_fractional_geometry_to_the_window_and_report_unplacement() {
    let probe = GeometryProbe::default();
    probe.visible_bounds.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| size_only_observer_fixture(probe.clone())).viewport_px(160, 100),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(
        probe.visible_bounds.with(|items| items.clone()),
        vec![Some(FloatRect {
            x: 10.25,
            y: 20.5,
            width: 149.75,
            height: 35.0,
        })]
    );
    pump_layout_test_session(&mut session, 16_666_667);
    assert_eq!(probe.visible_bounds.with(Vec::len), 1);
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(128.5, 0.0));
    pump_layout_test_session(&mut session, 33_333_334);
    assert_eq!(
        probe
            .visible_bounds
            .with(|items| *items.last().expect("test fixture recorded a value")),
        Some(FloatRect {
            x: 138.75,
            y: 20.5,
            width: 21.25,
            height: 35.0,
        })
    );
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 50_000_001);
    assert_eq!(
        probe
            .visible_bounds
            .with(|items| *items.last().expect("test fixture recorded a value")),
        None
    );
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 66_666_668);
    assert_eq!(
        probe
            .visible_bounds
            .with(|items| *items.last().expect("test fixture recorded a value")),
        Some(FloatRect {
            x: 138.75,
            y: 20.5,
            width: 21.25,
            height: 35.0,
        })
    );
}

#[test]
fn size_observers_ignore_movement_and_scaling_and_publish_resized_revealed_and_new_subscribers() {
    let probe = GeometryProbe::default();
    probe.sizes.with_mut(Vec::clear);
    probe.late_sizes.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| size_only_observer_fixture(probe.clone())).viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(
        probe.sizes.with(|sizes| sizes.clone()),
        vec![crate::PxSize::new(Px(150), Px(35))]
    );
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(128.5, -0.25));
    pump_layout_test_session(&mut session, 16_666_667);
    assert!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .geometry_nodes_updated
            > 0
    );
    probe
        .child_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(3.75, 4.0));
    pump_layout_test_session(&mut session, 33_333_334);
    probe
        .outer_scale
        .get()
        .expect("test fixture entry exists")
        .set(0.625);
    pump_layout_test_session(&mut session, 50_000_001);
    assert_eq!(probe.sizes.with(Vec::len), 1);
    probe
        .content_size
        .get()
        .expect("test fixture entry exists")
        .set(ComputedData {
            width: Px(220),
            height: Px(55),
        });
    pump_layout_test_session(&mut session, 66_666_668);
    assert_eq!(
        probe
            .sizes
            .with(|sizes| *sizes.last().expect("test fixture recorded a value")),
        crate::PxSize::new(Px(220), Px(55))
    );
    assert_eq!(probe.sizes.with(Vec::len), 2);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 83_333_335);
    probe
        .content_size
        .get()
        .expect("test fixture entry exists")
        .set(ComputedData {
            width: Px(250),
            height: Px(60),
        });
    pump_layout_test_session(&mut session, 100_000_002);
    assert_eq!(probe.sizes.with(Vec::len), 2);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 116_666_669);
    assert_eq!(probe.sizes.with(Vec::len), 3);
    assert_eq!(
        probe
            .sizes
            .with(|sizes| *sizes.last().expect("test fixture recorded a value")),
        crate::PxSize::new(Px(250), Px(60))
    );
    probe
        .late_observer
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 133_333_336);
    assert_eq!(probe.sizes.with(Vec::len), 3);
    assert_eq!(
        probe.late_sizes.with(|sizes| sizes.clone()),
        vec![crate::PxSize::new(Px(250), Px(60))]
    );
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 150_000_003);
    probe
        .content_size
        .get()
        .expect("test fixture entry exists")
        .set(ComputedData {
            width: Px(310),
            height: Px(70),
        });
    pump_layout_test_session(&mut session, 166_666_670);
    assert_eq!(probe.sizes.with(Vec::len), 3);
    assert_eq!(probe.late_sizes.with(Vec::len), 1);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 183_333_337);
    assert_eq!(probe.sizes.with(Vec::len), 4);
    assert_eq!(
        probe
            .late_sizes
            .with(|sizes| *sizes.last().expect("test fixture recorded a value")),
        crate::PxSize::new(Px(310), Px(70))
    );
    probe
        .late_observer
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 200_000_004);
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::ZERO);
    pump_layout_test_session(&mut session, 216_666_671);
    assert_eq!(probe.sizes.with(Vec::len), 4);
    assert_eq!(probe.late_sizes.with(Vec::len), 2);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 233_333_338);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 250_000_005);
    assert_eq!(probe.sizes.with(Vec::len), 5);
    assert_eq!(
        probe
            .sizes
            .with(|sizes| *sizes.last().expect("test fixture recorded a value")),
        crate::PxSize::new(Px(310), Px(70))
    );
    assert_eq!(probe.late_sizes.with(Vec::len), 2);
}

#[derive(Clone, PartialEq)]
struct SpaceContent(State<bool>, State<ComputedData>);

impl LayoutPolicy for SpaceContent {
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

#[crate::tessera]
fn relative_observer_fixture(probe: GeometryProbe) {
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
    probe.parent_offset.set(Some(parent));
    probe.child_offset.set(Some(child));
    probe.observer_visible.set(Some(visible));
    probe.outer_scale.set(Some(outer));
    probe.inner_scale.set(Some(inner));
    probe.mark_space.set(Some(marked));
    probe.nested_space.set(Some(nested));
    probe.space_offset.set(Some(space_offset));
    probe.counter_offset.set(Some(counter_offset));
    probe.content_size.set(Some(size));
    crate::layout::layout(Anchor)
        .modifier(Modifier::new().push_placement(StateTranslation(parent)))
        .child(move || {
            let probe = probe.clone();
            scaled_box(outer.get()).child(move || {
                let probe = probe.clone();
                let modifier = if marked.get() {
                    Modifier::new().layout_coordinate_space()
                } else {
                    Modifier::new()
                };
                crate::layout::layout(Anchor)
                    .modifier(modifier.push_placement(StateTranslation(space_offset)))
                    .child(move || {
                        let probe = probe.clone();
                        scaled_box(inner.get()).child(move || {
                            let probe = probe.clone();
                            let modifier = if nested.get() {
                                Modifier::new().layout_coordinate_space()
                            } else {
                                Modifier::new()
                            };
                            crate::layout::layout(SpaceContent(visible, size))
                                .modifier(modifier.push_placement(StateTranslation(counter_offset)))
                                .child(move || {
                                    let probe = probe.clone();
                                    layout(Stack).modifier(
                                        Modifier::new()
                                            .push_placement(StateTranslation(child))
                                            .on_layout_rect_in_space(probe.space_rects.recorder())
                                            .on_layout_coordinates(probe.coordinates.recorder()),
                                    );
                                });
                        });
                    });
            });
        });
}

#[test]
fn relative_observers_ignore_space_motion_and_track_local_changes_and_reparenting() {
    let probe = GeometryProbe::default();
    probe.space_rects.with_mut(Vec::clear);
    probe.coordinates.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| relative_observer_fixture(probe.clone())).viewport_px(1000, 800),
    );
    pump_layout_test_session(&mut session, 0);
    let expected = FloatRect {
        x: 18.0625,
        y: 35.5,
        width: 93.75,
        height: 21.875,
    };
    assert_eq!(
        probe.space_rects.with(|rects| rects.clone()),
        vec![expected]
    );
    for frame in 1..=5 {
        probe
            .parent_offset
            .get()
            .expect("test fixture entry exists")
            .set(PxPosition::from_f32_xy(65536.25 * frame as f32, -0.5));
        pump_layout_test_session(&mut session, frame * 16_666_667);
    }
    assert_eq!(probe.space_rects.with(Vec::len), 1);
    assert_eq!(probe.coordinates.with(Vec::len), 6);
    probe
        .outer_scale
        .get()
        .expect("test fixture entry exists")
        .set(0.75);
    pump_layout_test_session(&mut session, 100_000_000);
    assert_eq!(probe.space_rects.with(Vec::len), 1);
    probe
        .inner_scale
        .get()
        .expect("test fixture entry exists")
        .set(0.5);
    pump_layout_test_session(&mut session, 116_666_667);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        FloatRect {
            x: 16.5,
            y: 32.5,
            width: 75.0,
            height: 17.5
        }
    );
    probe
        .child_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(3.75, 4.0));
    pump_layout_test_session(&mut session, 133_333_334);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        FloatRect {
            x: 17.25,
            y: 32.75,
            width: 75.0,
            height: 17.5
        }
    );
    probe
        .content_size
        .get()
        .expect("test fixture entry exists")
        .set(ComputedData {
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
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        resized
    );
    let count = probe.space_rects.with(Vec::len);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 166_666_668);
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::ZERO);
    pump_layout_test_session(&mut session, 183_333_335);
    assert_eq!(probe.space_rects.with(Vec::len), count);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 200_000_002);
    assert_eq!(probe.space_rects.with(Vec::len), count + 1);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        resized
    );
    probe
        .mark_space
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 216_666_669);
    assert_eq!(probe.space_rects.with(Vec::len), count + 2);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        probe.coordinates.with(|items| items
            .last()
            .expect("test fixture recorded a value")
            .window_bounds())
    );
    probe
        .mark_space
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 233_333_336);
    assert_eq!(probe.space_rects.with(Vec::len), count + 3);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        resized
    );
    probe
        .nested_space
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 250_000_003);
    assert_eq!(probe.space_rects.with(Vec::len), count + 4);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        FloatRect {
            x: 14.0,
            y: 24.5,
            width: 120.0,
            height: 40.0,
        }
    );
    probe
        .nested_space
        .get()
        .expect("test fixture entry exists")
        .set(false);
    pump_layout_test_session(&mut session, 266_666_670);
    assert_eq!(probe.space_rects.with(Vec::len), count + 5);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        resized
    );
    let window_count = probe.coordinates.with(Vec::len);
    probe
        .space_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(64.0, 0.0));
    probe
        .counter_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(-128.0, 0.0));
    pump_layout_test_session(&mut session, 283_333_337);
    assert_eq!(probe.coordinates.with(Vec::len), window_count);
    assert_eq!(probe.space_rects.with(Vec::len), count + 6);
    assert_eq!(
        probe
            .space_rects
            .with(|rects| *rects.last().expect("test fixture recorded a value")),
        FloatRect {
            x: -46.75,
            y: 32.75,
            width: 60.0,
            height: 20.0,
        }
    );
}

struct GeometryChildTag;
impl crate::SemanticsModifierNode for GeometryChildTag {
    fn apply(
        &self,
        node: &mut crate::AccessibilityNode,
        _: &mut Option<crate::AccessibilityActionHandler>,
    ) {
        node.key = Some("geometry-child".to_owned());
    }
}

#[derive(Clone, PartialEq)]
struct StateTranslation(State<PxPosition>);

impl crate::PlacementModifierNode for StateTranslation {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + self.0.get()
    }
}

#[crate::tessera]
fn observer_follow_fixture(probe: GeometryProbe) {
    let source = crate::remember(|| PxPosition::ZERO);
    let observed = crate::remember(|| PxPosition::from_f32_xy(100.0, 0.0));
    let size = crate::remember(|| ComputedData {
        width: Px(20),
        height: Px(10),
    });
    probe.parent_offset.set(Some(source));
    layout(Stack).child(move || {
        let input = probe.input.clone();
        scaled_box(0.625).child(move || {
            crate::layout::layout(SizeOnlyContent(size)).modifier(
                Modifier::new()
                    .push_pointer_input(ObserveInput(false, input.clone()))
                    .push_placement(StateTranslation(source))
                    .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                        let bounds = coordinates.window_bounds();
                        observed.set_if_changed(PxPosition::from_f32_xy(bounds.x, bounds.y));
                    }),
            );
        });
        crate::layout::layout(SizeOnlyContent(size)).modifier(
            Modifier::new()
                .push_pointer_input(ObserveInput(true, probe.input.clone()))
                .push_placement(StateTranslation(observed))
                .push_semantics(GeometryChildTag),
        );
    });
}

#[test]
fn geometry_observer_updates_settle_placement_before_the_frame_snapshot() {
    let probe = GeometryProbe::default();
    let mut session = start_layout_test_session(
        layout_test(|| observer_follow_fixture(probe.clone())).viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    assert!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .cache_miss_no_entry
            > 0
    );
    assert_eq!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .pointer_nodes_dispatched,
        2
    );
    let before = session.inspect_components();
    assert_eq!(
        before
            .tagged("geometry-child")
            .bounds()
            .expect("test fixture provides bounds")
            .x,
        0.0
    );
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(12.0, 2.0));
    pump_layout_test_session(&mut session, 1);
    assert_eq!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .pointer_nodes_dispatched,
        2
    );
    assert_eq!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .cache_miss_no_entry,
        0
    );
    let after = session.inspect_components();
    let bounds = after
        .tagged("geometry-child")
        .bounds()
        .expect("test fixture provides bounds");
    assert_eq!((bounds.x, bounds.y), (7.5, 1.25));
    assert_eq!(
        session
            .work()
            .count("observer_follow_fixture", crate::work::WorkPhase::Build),
        0
    );
    assert_eq!(
        session
            .work()
            .executed::<SizeOnlyContent>(crate::work::WorkPhase::Measure),
        0
    );
    pump_layout_test_session(&mut session, 2);
    assert_eq!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .pointer_nodes_dispatched,
        0
    );
    assert_eq!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .cache_miss_no_entry,
        0
    );
    assert_eq!(
        session
            .work()
            .executed::<StateTranslation>(crate::work::WorkPhase::Transform),
        0
    );
}

#[crate::tessera]
fn inherited_translation_fixture(probe: GeometryProbe) {
    let parent = crate::remember(|| PxPosition::ZERO);
    let child = crate::remember(|| PxPosition::from_f32_xy(2.25, 3.5));
    probe.parent_offset.set(Some(parent));
    probe.child_offset.set(Some(child));
    crate::layout::layout(Anchor)
        .modifier(Modifier::new().push_placement(StateTranslation(parent)))
        .child(move || {
            let probe = probe.clone();
            layout(Stack).modifier(
                Modifier::new()
                    .push_placement(StateTranslation(child))
                    .push_semantics(GeometryChildTag)
                    .on_layout_coordinates(probe.coordinates.recorder()),
            );
        });
}

#[test]
fn inherited_translation_reuses_local_placement_and_preserves_fractional_observers() {
    let probe = GeometryProbe::default();
    probe.coordinates.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| inherited_translation_fixture(probe.clone())).viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(
        session
            .diagnostics()
            .expect("test fixture provides diagnostics")
            .pointer_nodes_updated,
        0
    );
    let snapshot = session.run(crate::testing::ComponentSnapshot::capture);
    let child = snapshot.tagged("geometry-child").instance_id();
    assert!(
        session
            .work()
            .for_instance(child, crate::work::WorkPhase::Transform)
            > 0
    );
    for frame in 1..=40 {
        let offset = if frame % 2 == 0 { 0.0 } else { 41.25 };
        probe
            .parent_offset
            .get()
            .expect("test fixture entry exists")
            .set(PxPosition::from_f32_xy(-0.5, offset));
        pump_layout_test_session(&mut session, frame * 16_666_667);
        assert_eq!(
            session
                .diagnostics()
                .expect("test fixture provides diagnostics")
                .pointer_nodes_updated,
            0,
            "moving decorative layouts must not update the pointer index"
        );
        assert_eq!(
            session
                .work()
                .for_instance(child, crate::work::WorkPhase::Transform),
            0
        );
        let bounds = probe.coordinates.with(|items| {
            items
                .last()
                .expect("test fixture recorded a value")
                .window_bounds()
        });
        assert_eq!(bounds.x, 12.0);
        assert_eq!(bounds.y, 24.0 + offset);
    }
    probe
        .child_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(3.75, 4.0));
    pump_layout_test_session(&mut session, 700_000_000);
    assert!(
        session
            .work()
            .for_instance(child, crate::work::WorkPhase::Transform)
            > 0
    );
    let bounds = probe.coordinates.with(|items| {
        items
            .last()
            .expect("test fixture recorded a value")
            .window_bounds()
    });
    assert_eq!((bounds.x, bounds.y), (13.5, 24.5));
}

#[derive(Clone, PartialEq)]
struct PlaceWhenVisible(State<bool>);

impl LayoutPolicy for PlaceWhenVisible {
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

#[crate::tessera]
fn hidden_position_observer_fixture(probe: GeometryProbe) {
    let visible = crate::remember(|| true);
    let parent = crate::remember(|| PxPosition::ZERO);
    probe.observer_visible.set(Some(visible));
    probe.parent_offset.set(Some(parent));
    crate::layout::layout(PlaceWhenVisible(visible))
        .modifier(Modifier::new().push_placement(StateTranslation(parent)))
        .child(move || {
            let probe = probe.clone();
            layout(Stack)
                .modifier(Modifier::new().on_layout_coordinates(probe.coordinates.recorder()));
        });
}

#[test]
fn position_observer_resumes_after_hidden_ancestor_moves() {
    let probe = GeometryProbe::default();
    probe.coordinates.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| hidden_position_observer_fixture(probe.clone())).viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    assert_eq!(probe.coordinates.with(Vec::len), 1);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(false);
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(0.0, 41.25));
    pump_layout_test_session(&mut session, 16_666_667);
    assert_eq!(probe.coordinates.with(Vec::len), 1);
    probe
        .parent_offset
        .get()
        .expect("test fixture entry exists")
        .set(PxPosition::from_f32_xy(0.0, 80.5));
    pump_layout_test_session(&mut session, 33_333_334);
    assert_eq!(probe.coordinates.with(Vec::len), 1);
    probe
        .observer_visible
        .get()
        .expect("test fixture entry exists")
        .set(true);
    pump_layout_test_session(&mut session, 50_000_001);
    let coordinates = probe.coordinates.with(|items| {
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

impl LayoutPolicy for Anchor {
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

impl LayoutPolicy for Content {
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

struct ObserveInput(bool, crate::testing::TestProbe<Vec<ObservedInput>>);

impl PointerInputModifierNode for ObserveInput {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let position = input.cursor_position_rel;
        let absolute = input.cursor_position_abs();
        for change in input.pointer_changes.iter_mut() {
            let discrete = change.is_discrete_scroll();
            if let CursorEventContent::Scroll(scroll) = &mut change.content {
                self.1.with_mut(|input| {
                    input.push((self.0, position, absolute, scroll.delta_y, discrete));
                });
                if self.0 {
                    scroll.delta_y -= 8.0;
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ResponsiveInputStage(State<ComputedData>);

impl LayoutPolicy for ResponsiveInputStage {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let child = scope.children()[0].measure(scope, &Constraint::NONE)?;
        let mut result = LayoutResult::new(self.0.get());
        result.place_child(child, PxPosition::ZERO);
        Ok(result)
    }
}

#[crate::tessera]
fn dynamic_scale_input_fixture(probe: GeometryProbe) {
    let scale = crate::remember(|| 1.0_f32);
    let visible = crate::remember(|| true);
    let size = crate::remember(|| ComputedData {
        width: Px(48),
        height: Px(48),
    });
    let stage_size = crate::remember(|| ComputedData {
        width: Px(200),
        height: Px(80),
    });
    probe.content_size.set(Some(stage_size));
    let content = crate::RenderSlot::new(move || {
        crate::layout::layout(SizeOnlyContent(size))
            .modifier(Modifier::new().push_pointer_input(ObserveInput(true, probe.input.clone())));
    });
    crate::layout::layout(SizeOnlyRoot(visible)).child(move || {
        crate::layout::layout(ResponsiveInputStage(stage_size))
            .modifier(Modifier::new().on_layout_coordinates(
                move |coordinates: crate::LayoutCoordinates| {
                    let next = coordinates.local_size.width.0 as f32 / 200.0;
                    if scale.get() != next {
                        scale.set(next);
                    }
                },
            ))
            .child(move || {
                scaled_box(scale.get()).child_shared(content);
            });
    });
}

#[test]
fn dynamic_ancestor_scale_updates_unobserved_pointer_coordinates_and_scroll_units() {
    let probe = GeometryProbe::default();
    {
        let mut previous = start_layout_test_session(
            layout_test(|| dynamic_scale_input_fixture(probe.clone())).viewport_px(300, 200),
        );
        pump_layout_test_session(&mut previous, 0);
    }
    probe.input.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| dynamic_scale_input_fixture(probe.clone())).viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    for (index, (width, scale, x, y, local_delta)) in [
        (200, 1.0, 50.25, 44.5, 24.0),
        (300, 1.5, 70.25, 56.5, 16.0),
        (100, 0.5, 30.25, 32.5, 48.0),
        (200, 1.0, 50.25, 44.5, 24.0),
    ]
    .into_iter()
    .enumerate()
    {
        probe
            .content_size
            .get()
            .expect("test fixture entry exists")
            .set(ComputedData {
                width: Px(width),
                height: Px(80),
            });
        let frame = (index as u64 * 4 + 1) * 16_666_667;
        pump_layout_test_session(&mut session, frame);
        pump_layout_test_session(&mut session, frame + 16_666_667);
        pump_layout_test_session(&mut session, frame + 33_333_334);
        probe.input.with_mut(Vec::clear);
        let position = PxPosition::from_f32_xy(x, y);
        let scroll = PointerChange::new(
            Instant::now(),
            crate::MOUSE_POINTER_ID,
            CursorEventContent::Scroll(ScrollEventContent {
                delta_x: 0.0,
                delta_y: 24.0,
                unit: ScrollDeltaUnit::Pixel,
                source: ScrollEventSource::Wheel,
            }),
            GestureState::default(),
        )
        .with_position(position);
        pump_layout_test_session_with_input(
            &mut session,
            frame + 50_000_001,
            vec![scroll],
            Some(position),
        );
        let samples = probe.input.with(Clone::clone);
        assert_eq!(samples.len(), 1, "Input delivery at scale {scale}");
        assert!(!samples[0].4, "Scaling cannot make precise input discrete");
        assert_eq!(
            samples[0].1,
            Some(PxPosition::from_f32_xy(40.0, 24.0)),
            "Local pointer coordinates at scale {scale}"
        );
        assert_eq!(
            samples[0].2,
            Some(position),
            "Canonical window pointer at scale {scale}"
        );
        assert_eq!(
            samples[0].3, local_delta,
            "Local scroll units at scale {scale}"
        );
    }
}

#[crate::tessera]
fn scale_fixture(probe: GeometryProbe) {
    crate::layout::layout(Anchor)
        .modifier(Modifier::new().push_pointer_input(ObserveInput(false, probe.input.clone())))
        .child(move || {
            let probe = probe.clone();
            scaled_box(0.625).child(move || {
                let probe = probe.clone();
                crate::layout::layout(Content).modifier(
                    Modifier::new()
                        .push_pointer_input(ObserveInput(true, probe.input.clone()))
                        .on_layout_coordinates(probe.coordinates.recorder()),
                );
            });
        });
}

#[test]
fn scaled_layout_keeps_local_dimensions_maps_actual_input_and_returns_window_scroll_remainder() {
    let probe = GeometryProbe::default();
    probe.coordinates.with_mut(Vec::clear);
    probe.input.with_mut(Vec::clear);
    let mut session = start_layout_test_session(
        layout_test(|| {
            scale_fixture(probe.clone());
        })
        .viewport_px(300, 200),
    );
    pump_layout_test_session(&mut session, 0);
    let coordinates = probe
        .coordinates
        .with(|items| *items.last().expect("Layout observer must run"));
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
    let observed = probe.coordinates.with(Vec::len);
    pump_layout_test_session(&mut session, 1);
    assert_eq!(
        probe.coordinates.with(Vec::len),
        observed,
        "An idle frame must not republish unchanged geometry"
    );
    let position = PxPosition::from_f32_xy(40.25, 35.5);
    let scroll = PointerChange::new(
        Instant::now(),
        crate::MOUSE_POINTER_ID,
        CursorEventContent::Scroll(ScrollEventContent {
            delta_x: 0.0,
            delta_y: 40.0,
            unit: ScrollDeltaUnit::Pixel,
            source: ScrollEventSource::Wheel,
        }),
        GestureState::default(),
    )
    .with_position(position);
    pump_layout_test_session_with_input(&mut session, 16_000_000, vec![scroll], Some(position));
    let samples = probe.input.with(Clone::clone);
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
    assert_eq!(child.3, 64.0);
    assert!(child.4);
    let parent = samples
        .iter()
        .find(|sample| !sample.0)
        .expect("Ancestor must receive remaining wheel input");
    assert_eq!(
        parent.3, 35.0,
        "Eight local units consume exactly five window units"
    );
    assert!(parent.4, "A partial wheel step must keep its animation");
}
