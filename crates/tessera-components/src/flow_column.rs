//! A flowing vertical layout component.
//!
//! ## Usage
//!
//! Wrap tall lists or cards into multiple columns.
use tessera_ui::{Dp, Modifier, Px, RenderSlot, layout::layout, tessera};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    flow_row::flow_layout::{FlowAxis, FlowLayout},
};

/// # flow_column
///
/// A layout component that wraps children into multiple vertical columns.
///
/// ## Usage
///
/// Wrap tall item stacks into columns for responsive dashboards or menus.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the flowing column container.
/// - `main_axis_alignment` — alignment within each wrapped column.
/// - `cross_axis_alignment` — cross-axis alignment for items inside a column.
/// - `line_alignment` — alignment between wrapped columns.
/// - `item_spacing` — spacing between items in the same column.
/// - `line_spacing` — spacing between wrapped columns.
/// - `max_items_per_line` — optional cap for items per column.
/// - `max_lines` — optional cap for total wrapped columns.
/// - `children` — child slot rendered inside the layout.
///
/// ## Examples
///
/// ```
/// use tessera_components::{boxed::boxed, flow_column::flow_column, modifier::{ModifierExt as _, SemanticsArgs}};
/// use tessera_ui::{Dp, Modifier, tessera, testing::layout_test};
///
/// #[tessera]
/// fn demo() {
///     flow_column().modifier(Modifier::new().height(Dp(50.0))).children(|| {
///         for tag in ["first", "second"] {
///             boxed().modifier(
///                 Modifier::new().size(Dp(10.0), Dp(30.0))
///                     .semantics(SemanticsArgs { test_tag: Some(tag.into()), ..Default::default() })
///             ).children(|| {});
///         }
///     });
/// }
///
/// let snapshot = layout_test(demo).viewport_px(100, 50).run();
/// snapshot.node("first").position(0, 0).size(10, 30);
/// snapshot.node("second").position(10, 0).size(10, 30);
/// ```
#[tessera]
pub fn flow_column(
    modifier: Option<Modifier>,
    main_axis_alignment: Option<MainAxisAlignment>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    line_alignment: Option<MainAxisAlignment>,
    item_spacing: Option<Dp>,
    line_spacing: Option<Dp>,
    max_items_per_line: Option<usize>,
    max_lines: Option<usize>,
    children: Option<RenderSlot>,
) {
    let modifier = modifier.unwrap_or_default();
    let main_axis_alignment = main_axis_alignment.unwrap_or_default();
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let line_alignment = line_alignment.unwrap_or_default();
    let item_spacing = Px::from(item_spacing.unwrap_or(Dp(0.0))).max(Px::ZERO);
    let line_spacing = Px::from(line_spacing.unwrap_or(Dp(0.0))).max(Px::ZERO);
    let max_items_per_line = max_items_per_line.unwrap_or(usize::MAX);
    let max_lines = max_lines.unwrap_or(usize::MAX);
    layout()
        .policy(FlowLayout {
            axis: FlowAxis::Vertical,
            main_axis_alignment,
            cross_axis_alignment,
            line_alignment,
            item_spacing,
            line_spacing,
            max_items_per_line,
            max_lines,
        })
        .modifier(modifier)
        .child_shared(children.unwrap_or_else(RenderSlot::empty));
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        AxisConstraint, ComputedData, Dp, LayoutResult, MeasurementError, Modifier,
        NoopDrawModifier, Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        modifier::{ModifierExt as _, SemanticsArgs},
    };

    use super::flow_column;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl tessera_ui::LayoutModifierNode for FixedTestLayout {
        fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    #[tessera]
    fn fixed_test_box(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();

        layout()
            .policy(FixedTestLayout { width, height })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn flow_column_layout_case() {
        flow_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::new(Px::ZERO, Some(Px::new(100)))),
                Some(AxisConstraint::exact(Px::new(40))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .line_alignment(MainAxisAlignment::Start)
            .item_spacing(Px::new(3).into())
            .line_spacing(Px::new(4).into())
            .children(|| {
                fixed_test_box()
                    .tag("flow_column_first".to_string())
                    .width(20)
                    .height(12);
                fixed_test_box()
                    .tag("flow_column_second".to_string())
                    .width(25)
                    .height(15);
                fixed_test_box()
                    .tag("flow_column_third".to_string())
                    .width(18)
                    .height(10);
            });
    }

    #[test]
    fn flow_column_wraps_children_across_columns() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                flow_column_layout_case();
            },
            expect: {
                node("flow_column_first").position(0, 0).size(20, 12);
                node("flow_column_second").position(0, 15).size(25, 15);
                node("flow_column_third").position(29, 0).size(18, 10);
            }
        }
    }

    #[tessera]
    fn sized_test_box(tag: Option<String>, width: Option<Dp>, height: Option<Dp>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or(Dp(0.0));
        let height = height.unwrap_or(Dp(0.0));

        layout()
            .draw(NoopDrawModifier)
            .modifier(
                Modifier::new()
                    .size(width, height)
                    .semantics(SemanticsArgs {
                        test_tag: Some(tag),
                        ..Default::default()
                    }),
            );
    }

    #[tessera]
    fn flow_column_center_sized_case() {
        flow_column()
            .modifier(Modifier::new().fill_max_width())
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .line_alignment(MainAxisAlignment::Start)
            .children(|| {
                sized_test_box()
                    .tag("flow_column_center_sized".to_string())
                    .width(Dp(32.0))
                    .height(Dp(4.0));
            });
    }

    #[test]
    fn flow_column_center_measures_sized_child_intrinsically() {
        tessera_ui::assert_layout! {
            viewport: (100, 60),
            content: {
                flow_column_center_sized_case();
            },
            expect: {
                node("flow_column_center_sized").position(0, 0).size(32, 4);
            }
        }
    }
    #[tessera]
    fn stretch_wrapped_case() {
        flow_column()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::new(Px::ZERO, Some(Px(100)))),
                Some(AxisConstraint::exact(Px(60))),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .item_spacing(Dp(5.0))
            .line_spacing(Dp(4.0))
            .children(|| {
                sized_test_box()
                    .tag("stretch_0".to_owned())
                    .width(Dp(10.0))
                    .height(Dp(30.0));
                sized_test_box()
                    .tag("stretch_1".to_owned())
                    .width(Dp(20.0))
                    .height(Dp(25.0));
                sized_test_box()
                    .tag("stretch_2".to_owned())
                    .width(Dp(12.0))
                    .height(Dp(20.0));
            });
    }

    #[test]
    fn stretch_matches_the_cross_size_of_each_wrapped_column() {
        tessera_ui::assert_layout! {
            viewport: (100, 100),
            content: { stretch_wrapped_case(); },
            expect: {
                node("stretch_0").position(0, 0).size(20, 30);
                node("stretch_1").position(0, 35).size(20, 25);
                node("stretch_2").position(24, 0).size(12, 20);
            }
        }
    }
}
