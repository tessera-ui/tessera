use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, Dp, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, layout::MeasureScope, tessera, testing::UiTest,
};

use super::{horizontal_divider, vertical_divider};

#[derive(Clone, PartialEq)]
struct Constrained(Constraint);

impl LayoutPolicy for Constrained {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child = scope.children()[0];
        let size = child.measure(scope, &self.0)?.size();
        let mut result = LayoutResult::new(ComputedData {
            width: scope.parent_constraint().width().clamp(size.width),
            height: scope.parent_constraint().height().clamp(size.height),
        });
        result.place_child(child, PxPosition::ZERO);
        Ok(result)
    }
}

fn tagged() -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some("divider".into()),
        ..Default::default()
    })
}

#[tessera]
fn fixture(constraint: Constraint, vertical: bool, thickness: Dp, inset: bool) {
    tessera_ui::layout::layout(Constrained(constraint)).child(move || {
        let modifier = if inset {
            tagged().padding_symmetric(Dp(16.0), Dp::ZERO)
        } else {
            tagged()
        };
        if vertical {
            vertical_divider()
                .modifier(modifier)
                .thickness(thickness)
                .color(Color::BLACK);
        } else {
            horizontal_divider()
                .modifier(modifier)
                .thickness(thickness)
                .color(Color::BLACK);
        }
    });
}

#[test]
fn divider_clamps_thickness_and_resolves_bounded_and_unbounded_lengths() {
    crate::test_utils::run_isolated_test(
        "divider::tests::divider_clamps_thickness_and_resolves_bounded_and_unbounded_lengths",
        || {
            for vertical in [false, true] {
                for (axis, length, thickness_axis, thickness, expected) in [
                    (
                        AxisConstraint::at_most(Px(80)),
                        80,
                        AxisConstraint::at_most(Px(2)),
                        Dp(10.0),
                        2,
                    ),
                    (AxisConstraint::NONE, 0, AxisConstraint::NONE, Dp(2.0), 2),
                    (
                        AxisConstraint::at_least(Px(30)),
                        30,
                        AxisConstraint::NONE,
                        Dp(2.0),
                        2,
                    ),
                    (
                        AxisConstraint::at_most(Px(80)),
                        80,
                        AxisConstraint::at_least(Px(4)),
                        Dp(1.0),
                        4,
                    ),
                    (
                        AxisConstraint::at_most(Px(80)),
                        80,
                        AxisConstraint::at_most(Px::ZERO),
                        Dp(10.0),
                        0,
                    ),
                    (
                        AxisConstraint::at_most(Px(80)),
                        80,
                        AxisConstraint::NONE,
                        Dp::ZERO,
                        1,
                    ),
                    (
                        AxisConstraint::at_most(Px(80)),
                        80,
                        AxisConstraint::NONE,
                        Dp(-2.0),
                        0,
                    ),
                ] {
                    let constraint = if vertical {
                        Constraint::new(thickness_axis, axis)
                    } else {
                        Constraint::new(axis, thickness_axis)
                    };
                    let snapshot = UiTest::new()
                        .content(move || fixture(constraint, vertical, thickness, false))
                        .viewport(120, 120)
                        .frame();
                    let (width, height) = if vertical {
                        (expected, length)
                    } else {
                        (length, expected)
                    };
                    snapshot.node("divider").position(0, 0).size(width, height);
                }
            }
            let snapshot = UiTest::new()
                .content(|| {
                    fixture(
                        Constraint::new(
                            AxisConstraint::at_most(Px(80)),
                            AxisConstraint::at_most(Px(20)),
                        ),
                        false,
                        Dp(2.0),
                        true,
                    )
                })
                .viewport(120, 120)
                .frame();
            snapshot.node("divider").rect(0, 0, 80, 2);
        },
    );
}
