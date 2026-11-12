//! A flowing horizontal layout component.
//!
//! ## Usage
//!
//! Wrap chips, tags, or button groups across multiple rows.
use tessera_ui::{Dp, Modifier, Px, RenderSlot, tessera};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

#[path = "flow_layout.rs"]
pub(crate) mod flow_layout;

use self::flow_layout::{FlowAxis, FlowLayout};

/// # flow_row
///
/// A layout component that wraps children into multiple horizontal rows.
///
/// ## Usage
///
/// Wrap chips, tags, or button groups to fit variable-width screens.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the flowing row container.
/// - `main_axis_alignment` — alignment within each wrapped line.
/// - `cross_axis_alignment` — cross-axis alignment for items inside a line.
/// - `line_alignment` — alignment between wrapped lines.
/// - `item_spacing` — spacing between items in the same line.
/// - `line_spacing` — spacing between wrapped lines.
/// - `max_items_per_line` — optional cap for items per line.
/// - `max_lines` — optional cap for total wrapped lines.
/// - `children` — child slot rendered inside the layout.
///
/// ## Examples
///
/// ```
/// use tessera_components::boxed::boxed;
/// use tessera_components::flow_row::flow_row;
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Dp, Modifier, tessera, testing::layout_test};
///
/// #[tessera]
/// fn demo() {
///     flow_row().modifier(Modifier::new().width(Dp(50.0))).children(|| {
///         for tag in ["first", "second"] {
///             boxed().modifier(
///                 Modifier::new().size(Dp(30.0), Dp(10.0))
///                     .semantics(SemanticsArgs { test_tag: Some(tag.into()), ..Default::default() })
///             ).children(|| {});
///         }
///     });
/// }
///
/// let snapshot = layout_test(demo).viewport_px(50, 100).run();
/// snapshot.node("first").position(0, 0).size(30, 10);
/// snapshot.node("second").position(0, 10).size(30, 10);
/// ```
#[tessera]
pub fn flow_row(
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
    tessera_ui::layout::layout(FlowLayout {
        axis: FlowAxis::Horizontal,
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
    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::{
        AxisConstraint, ComputedData, Dp, LayoutResult, MeasurementError, Modifier,
        NoopDrawModifier, Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

    use super::flow_row;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl tessera_ui::LayoutPolicy for FixedTestLayout {
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

        tessera_ui::layout::layout(FixedTestLayout { width, height })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn flow_row_layout_case() {
        flow_row()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::new(Px::ZERO, Some(Px::new(100)))),
            ))
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .line_alignment(MainAxisAlignment::Start)
            .item_spacing(Px::new(5).into())
            .line_spacing(Px::new(4).into())
            .children(|| {
                fixed_test_box()
                    .tag("flow_row_first".to_string())
                    .width(30)
                    .height(10);
                fixed_test_box()
                    .tag("flow_row_second".to_string())
                    .width(25)
                    .height(10);
                fixed_test_box()
                    .tag("flow_row_third".to_string())
                    .width(20)
                    .height(12);
            });
    }

    #[test]
    fn flow_row_wraps_children_across_lines() {
        tessera_ui::assert_layout! {
            viewport: (100, 100),
            content: {
                flow_row_layout_case();
            },
            expect: {
                node("flow_row_first").position(0, 0).size(30, 10);
                node("flow_row_second").position(35, 0).size(25, 10);
                node("flow_row_third").position(0, 14).size(20, 12);
            }
        }
    }

    #[tessera]
    fn sized_test_box(tag: Option<String>, width: Option<Dp>, height: Option<Dp>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or(Dp(0.0));
        let height = height.unwrap_or(Dp(0.0));

        layout(tessera_ui::layout::Stack)
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
    fn flow_row_center_sized_case() {
        flow_row()
            .modifier(Modifier::new().fill_max_height())
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .line_alignment(MainAxisAlignment::Start)
            .children(|| {
                sized_test_box()
                    .tag("flow_row_center_sized".to_string())
                    .width(Dp(4.0))
                    .height(Dp(32.0));
            });
    }

    #[test]
    fn flow_row_center_measures_sized_child_intrinsically() {
        tessera_ui::assert_layout! {
            viewport: (60, 100),
            content: {
                flow_row_center_sized_case();
            },
            expect: {
                node("flow_row_center_sized").position(0, 0).size(4, 32);
            }
        }
    }
    #[tessera]
    fn stretch_wrapped_case() {
        flow_row()
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px(60))),
                Some(AxisConstraint::new(Px::ZERO, Some(Px(100)))),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .item_spacing(Dp(5.0))
            .line_spacing(Dp(4.0))
            .children(|| {
                sized_test_box()
                    .tag("stretch_0".to_owned())
                    .width(Dp(30.0))
                    .height(Dp(10.0));
                sized_test_box()
                    .tag("stretch_1".to_owned())
                    .width(Dp(25.0))
                    .height(Dp(20.0));
                sized_test_box()
                    .tag("stretch_2".to_owned())
                    .width(Dp(20.0))
                    .height(Dp(12.0));
            });
    }

    #[test]
    fn stretch_matches_the_cross_size_of_each_wrapped_row() {
        tessera_ui::assert_layout! {
            viewport: (100, 100),
            content: { stretch_wrapped_case(); },
            expect: {
                node("stretch_0").position(0, 0).size(30, 20);
                node("stretch_1").position(35, 0).size(25, 20);
                node("stretch_2").position(0, 24).size(20, 12);
            }
        }
    }
}
