use std::cell::Cell;

use tessera_ui::{
    AxisConstraint, ComputedData, LayoutModifierNode, LayoutResult, MeasurementError, Modifier, Px,
    RenderSlot, State,
    layout::{MeasureScope, layout},
    remember, tessera,
    testing::{layout_test, pump_layout_test_session, start_layout_test_session},
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
};

#[derive(Clone)]
struct Inputs {
    ink: bool,
    end: bool,
    stretch: bool,
    weight: f32,
    reverse: bool,
    count: u32,
}

thread_local! {
    static INPUTS: Cell<Option<State<Inputs>>> = const { Cell::new(None) };
    static MEASURES: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, PartialEq)]
struct Leaf;

impl LayoutModifierNode for Leaf {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        MEASURES.set(MEASURES.get() + 1);
        Ok(LayoutResult::new(ComputedData {
            width: scope.parent_constraint().width().clamp(Px(20)),
            height: scope.parent_constraint().height().clamp(Px(10)),
        }))
    }
}

#[tessera]
fn leaf(id: Option<u32>, weight: Option<f32>) {
    layout()
        .policy(Leaf)
        .modifier(
            Modifier::new()
                .weight(weight.unwrap_or(0.0))
                .semantics(SemanticsArgs {
                    test_tag: Some(format!("flex-{}", id.unwrap_or(0))),
                    ..Default::default()
                }),
        );
}

#[tessera]
fn fixture(vertical: bool) {
    let inputs = remember(|| Inputs {
        ink: false,
        end: false,
        stretch: false,
        weight: 0.0,
        reverse: false,
        count: 2,
    });
    INPUTS.set(Some(inputs));
    let current = inputs.get();
    let children = RenderSlot::new(move || {
        let current = inputs.get();
        for index in 0..current.count {
            let id = if current.reverse {
                current.count - index - 1
            } else {
                index
            };
            leaf().id(id).weight(if id == 0 {
                current.weight
            } else if current.weight > 0.0 {
                1.0
            } else {
                0.0
            });
        }
    });
    let cross = if current.stretch {
        CrossAxisAlignment::Stretch
    } else if current.end {
        CrossAxisAlignment::End
    } else {
        CrossAxisAlignment::Start
    };
    let main = if current.end {
        MainAxisAlignment::End
    } else {
        MainAxisAlignment::Start
    };
    let modifier = Modifier::new()
        .background(if current.ink {
            tessera_ui::Color::BLACK
        } else {
            tessera_ui::Color::WHITE
        })
        .constrain(
            Some(AxisConstraint::exact(Px(if vertical { 60 } else { 100 }))),
            Some(AxisConstraint::exact(Px(if vertical { 100 } else { 60 }))),
        );
    if vertical {
        column()
            .modifier(modifier)
            .main_axis_alignment(main)
            .cross_axis_alignment(cross)
            .children_shared(children);
    } else {
        row()
            .modifier(modifier)
            .main_axis_alignment(main)
            .cross_axis_alignment(cross)
            .children_shared(children);
    }
}

#[test]
fn flex_alignment_reuses_measurements_and_weights_and_structure_refresh_constraints() {
    for vertical in [false, true] {
        MEASURES.set(0);
        let mut session = start_layout_test_session(
            layout_test(move || {
                fixture(vertical);
            })
            .viewport_px(240, 180),
        );
        pump_layout_test_session(&mut session, 0)
            .node("flex-0")
            .size(20, 10)
            .position(0, 0);
        let inputs = INPUTS.get().unwrap();
        let initial = MEASURES.get();
        inputs.with_mut(|inputs| inputs.ink = true);
        pump_layout_test_session(&mut session, 8_000_000)
            .node("flex-0")
            .position(0, 0);
        assert_eq!(
            session.diagnostics().unwrap().measure_node_calls,
            1,
            "paint changes must resolve from the root cache"
        );
        assert_eq!(session.diagnostics().unwrap().geometry_nodes_updated, 0);
        inputs.with_mut(|inputs| inputs.end = true);
        let snapshot = pump_layout_test_session(&mut session, 16_666_667);
        snapshot.node("flex-0").position(
            if vertical { 40 } else { 60 },
            if vertical { 80 } else { 50 },
        );
        assert_eq!(
            MEASURES.get(),
            initial,
            "alignment must use cached child measurements"
        );
        inputs.with_mut(|inputs| inputs.stretch = true);
        let snapshot = pump_layout_test_session(&mut session, 33_333_334);
        snapshot.node("flex-0").size(
            if vertical { 60 } else { 20 },
            if vertical { 10 } else { 60 },
        );
        assert!(
            MEASURES.get() > initial,
            "stretch changes the child constraints"
        );
        inputs.with_mut(|inputs| {
            inputs.stretch = false;
            inputs.end = false;
            inputs.weight = 2.0;
        });
        let snapshot = pump_layout_test_session(&mut session, 50_000_001);
        snapshot.node("flex-0").size(
            if vertical { 20 } else { 66 },
            if vertical { 66 } else { 10 },
        );
        snapshot.node("flex-1").size(
            if vertical { 20 } else { 33 },
            if vertical { 33 } else { 10 },
        );
        inputs.with_mut(|inputs| inputs.reverse = true);
        let snapshot = pump_layout_test_session(&mut session, 66_666_668);
        snapshot.node("flex-1").position(0, 0);
        snapshot
            .node("flex-0")
            .position(if vertical { 0 } else { 33 }, if vertical { 33 } else { 0 });
        inputs.with_mut(|inputs| inputs.count = 3);
        let snapshot = pump_layout_test_session(&mut session, 83_333_335);
        snapshot.node("flex-2").position(0, 0);
        snapshot
            .node("flex-0")
            .size(
                if vertical { 20 } else { 50 },
                if vertical { 50 } else { 10 },
            )
            .position(if vertical { 0 } else { 50 }, if vertical { 50 } else { 0 });
        inputs.with_mut(|inputs| inputs.count = 1);
        pump_layout_test_session(&mut session, 100_000_002)
            .node("flex-0")
            .size(
                if vertical { 20 } else { 100 },
                if vertical { 100 } else { 10 },
            )
            .position(0, 0);
    }
}
