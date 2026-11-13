//! A container for stacking and aligning multiple children.
//!
//! ## Usage
//!
//! Use to create layered UIs, overlays, or composite controls.
use tessera_ui::{
    ComputedData, LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot,
    layout::{MeasureScope, PlacementResult, PlacementScope},
    tessera,
};

use crate::alignment::Alignment;

fn center_axis(container: Px, child: Px) -> f32 {
    (container.0 - child.0) as f32 / 2.0
}

fn compute_child_offset(
    alignment: Alignment,
    container_w: Px,
    container_h: Px,
    child_w: Px,
    child_h: Px,
) -> (f32, f32) {
    match alignment {
        Alignment::TopStart => (0.0, 0.0),
        Alignment::TopCenter => (center_axis(container_w, child_w), 0.0),
        Alignment::TopEnd => ((container_w.0 - child_w.0) as f32, 0.0),
        Alignment::CenterStart => (0.0, center_axis(container_h, child_h)),
        Alignment::Center => (
            center_axis(container_w, child_w),
            center_axis(container_h, child_h),
        ),
        Alignment::CenterEnd => (
            (container_w.0 - child_w.0) as f32,
            center_axis(container_h, child_h),
        ),
        Alignment::BottomStart => (0.0, (container_h.0 - child_h.0) as f32),
        Alignment::BottomCenter => (
            center_axis(container_w, child_w),
            (container_h.0 - child_h.0) as f32,
        ),
        Alignment::BottomEnd => (
            (container_w.0 - child_w.0) as f32,
            (container_h.0 - child_h.0) as f32,
        ),
    }
}

/// # boxed
///
/// A container that overlays its children, aligning them relative to each
/// other.
///
/// ## Usage
///
/// Stack children on top of each other to create layered interfaces, such as a
/// badge on an icon or text over an image.
///
/// ## Parameters
///
/// - `alignment` — default alignment applied to children without parent data.
/// - `modifier` — modifier chain applied to the boxed container.
/// - `children` — child slot rendered inside the boxed container.
///
/// ## Examples
///
/// ```
/// use tessera_components::{alignment::Alignment, boxed::boxed, text::text};
/// use tessera_foundation::modifier::ModifierExt as _;
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// boxed().children(|| {
///     text("Background");
///     text("Foreground")
///         .modifier(Modifier::new().align(Alignment::Center));
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn boxed(
    alignment: Option<Alignment>,
    modifier: Option<Modifier>,
    children: Option<RenderSlot>,
) {
    let alignment = alignment.unwrap_or_default();
    let modifier = modifier.unwrap_or_default();
    tessera_ui::layout::layout(BoxedLayout { alignment })
        .modifier(modifier)
        .child(children.unwrap_or_else(RenderSlot::empty));
}

#[derive(Clone, PartialEq)]
struct BoxedLayout {
    alignment: Alignment,
}

impl tessera_ui::LayoutPolicy for BoxedLayout {
    fn place(
        &self,
        scope: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let size = scope.size();
        let mut result = LayoutResult::new(size);
        for child in scope.children() {
            let alignment = child
                .parent_data::<tessera_foundation::modifier::AlignmentParentData>()
                .map_or(self.alignment, |data| data.alignment);
            let child_size = child.size();
            let (x, y) = compute_child_offset(
                alignment,
                size.width,
                size.height,
                child_size.width,
                child_size.height,
            );
            result.place_child(child, PxPosition::from_f32_xy(x, y));
        }
        Some(result.into_placement_result())
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_alignments = collect_child_alignments(input, &children);
        let n = child_alignments.len();
        debug_assert_eq!(
            children.len(),
            n,
            "Mismatch between children defined in scope and runtime children count"
        );

        let parent_constraint = *input.parent_constraint().as_ref();
        let child_constraint = input.parent_constraint().without_min();

        let mut max_child_width = Px(0);
        let mut max_child_height = Px(0);
        let mut children_sizes = vec![None; n];

        for (i, child) in children.iter().enumerate().take(n) {
            let child_result = child.measure(input, &child_constraint)?;
            max_child_width = max_child_width.max(child_result.width);
            max_child_height = max_child_height.max(child_result.height);
            children_sizes[i] = Some(child_result);
        }

        let final_width = parent_constraint.width.clamp(max_child_width);
        let final_height = parent_constraint.height.clamp(max_child_height);

        for (i, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child_id = children[i];
                let child_alignment = child_alignments[i].unwrap_or(self.alignment);
                let (x, y) = compute_child_offset(
                    child_alignment,
                    final_width,
                    final_height,
                    child_size.width,
                    child_size.height,
                );
                result.place_child(child_id, PxPosition::from_f32_xy(x, y));
            }
        }

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
    }
}

fn collect_child_alignments(
    scope: &MeasureScope<'_>,
    children: &[tessera_ui::layout::LayoutChild],
) -> Vec<Option<Alignment>> {
    children
        .iter()
        .map(|child| {
            child
                .parent_data::<tessera_foundation::modifier::AlignmentParentData>(scope)
                .map(|data| data.alignment)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::{
        AxisConstraint, ComputedData, LayoutResult, MeasurementError, Modifier, NoopDrawModifier,
        Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::alignment::Alignment;

    use super::boxed;

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
    fn fixed_test_box(
        modifier: Option<Modifier>,
        tag: Option<String>,
        width: Option<i32>,
        height: Option<i32>,
    ) {
        let modifier = modifier.unwrap_or_default();
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();

        tessera_ui::layout::layout(FixedTestLayout { width, height })
            .draw(NoopDrawModifier)
            .modifier(modifier.then(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            })));
    }

    #[tessera]
    fn boxed_layout_case() {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(100))),
                Some(AxisConstraint::exact(Px::new(80))),
            ))
            .children(|| {
                boxed_start_box();
                layout(tessera_ui::layout::Stack)
                    .modifier(Modifier::new().align(Alignment::BottomEnd))
                    .child(|| {
                        boxed_end_box();
                    });
            });
    }

    #[tessera]
    fn boxed_start_box() {
        fixed_test_box()
            .tag("boxed_start".to_string())
            .width(20)
            .height(10);
    }

    #[tessera]
    fn boxed_end_box() {
        fixed_test_box()
            .tag("boxed_end".to_string())
            .width(30)
            .height(15);
    }

    #[tessera]
    fn boxed_center_case() {
        boxed()
            .alignment(Alignment::Center)
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(100))),
                Some(AxisConstraint::exact(Px::new(80))),
            ))
            .children(|| {
                fixed_test_box()
                    .tag("boxed_center".to_string())
                    .width(20)
                    .height(10);
            });
    }

    #[tessera]
    fn boxed_forwarded_parent_data_case() {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(100))),
                Some(AxisConstraint::exact(Px::new(80))),
            ))
            .children(|| {
                fixed_test_box()
                    .modifier(Modifier::new().align(Alignment::Center))
                    .tag("boxed_forwarded".to_string())
                    .width(20)
                    .height(10);
            });
    }

    #[test]
    fn boxed_honors_child_alignment_override() {
        let layout = tessera_ui::testing::UiTest::new()
            .viewport(120, 100)
            .content(boxed_layout_case)
            .frame();
        layout.node("boxed_start").rect(0, 0, 20, 10);
        layout.node("boxed_end").rect(70, 65, 30, 15);
    }

    #[test]
    fn boxed_uses_default_alignment_for_children_without_parent_data() {
        tessera_ui::testing::UiTest::new()
            .viewport(120, 100)
            .content(boxed_center_case)
            .frame()
            .node("boxed_center")
            .rect(40, 35, 20, 10);
    }

    #[test]
    fn boxed_honors_alignment_on_wrapped_children() {
        tessera_ui::testing::UiTest::new()
            .viewport(120, 100)
            .content(boxed_forwarded_parent_data_case)
            .frame()
            .node("boxed_forwarded")
            .rect(40, 35, 20, 10);
    }
}
