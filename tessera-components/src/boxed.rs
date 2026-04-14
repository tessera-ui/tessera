//! A container for stacking and aligning multiple children.
//!
//! ## Usage
//!
//! Use to create layered UIs, overlays, or composite controls.
use tessera_ui::{
    AxisConstraint, ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlot,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::alignment::Alignment;

fn resolve_final_dimension(axis: AxisConstraint, largest_child: Px) -> Px {
    axis.clamp(largest_child)
}

fn center_axis(container: Px, child: Px) -> Px {
    (container - child) / 2
}

fn compute_child_offset(
    alignment: Alignment,
    container_w: Px,
    container_h: Px,
    child_w: Px,
    child_h: Px,
) -> (Px, Px) {
    match alignment {
        Alignment::TopStart => (Px(0), Px(0)),
        Alignment::TopCenter => (center_axis(container_w, child_w), Px(0)),
        Alignment::TopEnd => (container_w - child_w, Px(0)),
        Alignment::CenterStart => (Px(0), center_axis(container_h, child_h)),
        Alignment::Center => (
            center_axis(container_w, child_w),
            center_axis(container_h, child_h),
        ),
        Alignment::CenterEnd => (container_w - child_w, center_axis(container_h, child_h)),
        Alignment::BottomStart => (Px(0), container_h - child_h),
        Alignment::BottomCenter => (center_axis(container_w, child_w), container_h - child_h),
        Alignment::BottomEnd => (container_w - child_w, container_h - child_h),
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
/// use tessera_components::{
///     alignment::Alignment, boxed::boxed, modifier::ModifierExt as _, text::text,
/// };
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// boxed().children(|| {
///     text().content("Background");
///     text()
///         .content("Foreground")
///         .modifier(Modifier::new().align(Alignment::Center));
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn boxed(
    alignment: Option<Alignment>,
    modifier: Option<Modifier>,
    children: Option<RenderSlot>,
) {
    let alignment = alignment.unwrap_or_default();
    let modifier = modifier.unwrap_or_default();
    let children = children.unwrap_or_else(RenderSlot::empty);
    layout()
        .modifier(modifier)
        .layout_policy(BoxedLayout { alignment })
        .child(move || {
            children.render();
        });
}

#[derive(Clone, PartialEq)]
struct BoxedLayout {
    alignment: Alignment,
}

impl LayoutPolicy for BoxedLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let child_alignments = collect_child_alignments(&children);
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
            let child_result = child.measure(&child_constraint)?;
            max_child_width = max_child_width.max(child_result.width);
            max_child_height = max_child_height.max(child_result.height);
            children_sizes[i] = Some(child_result);
        }

        let final_width = resolve_final_dimension(parent_constraint.width, max_child_width);
        let final_height = resolve_final_dimension(parent_constraint.height, max_child_height);

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
                result.place_child(child_id, PxPosition::new(x, y));
            }
        }

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
    }
}

fn collect_child_alignments(
    children: &[tessera_ui::layout::LayoutChild<'_>],
) -> Vec<Option<Alignment>> {
    children
        .iter()
        .map(|child| {
            child
                .parent_data::<crate::modifier::AlignmentParentData>()
                .map(|data| data.alignment)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        AxisConstraint, ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier,
        NoopRenderPolicy, Px,
        layout::{MeasureScope, layout},
        tessera,
    };

    use crate::{
        alignment::Alignment,
        modifier::{ModifierExt as _, SemanticsArgs},
    };

    use super::boxed;

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
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
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn forwarded_modifier_test_box(
        modifier: Option<Modifier>,
        tag: Option<String>,
        width: Option<i32>,
        height: Option<i32>,
    ) {
        let modifier = modifier.unwrap_or_default();
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();

        layout()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
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
                layout()
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
                forwarded_modifier_test_box()
                    .modifier(Modifier::new().align(Alignment::Center))
                    .tag("boxed_forwarded".to_string())
                    .width(20)
                    .height(10);
            });
    }

    #[test]
    fn boxed_honors_child_alignment_override() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                boxed_layout_case();
            },
            expect: {
                node("boxed_start").position(0, 0).size(20, 10);
                node("boxed_end").position(70, 65).size(30, 15);
            }
        }
    }

    #[test]
    fn boxed_uses_default_alignment_for_children_without_parent_data() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                boxed_center_case();
            },
            expect: {
                node("boxed_center").position(40, 35).size(20, 10);
            }
        }
    }

    #[test]
    fn boxed_honors_alignment_on_wrapped_children() {
        tessera_ui::assert_layout! {
            viewport: (120, 100),
            content: {
                boxed_forwarded_parent_data_case();
            },
            expect: {
                node("boxed_forwarded").position(40, 35).size(20, 10);
            }
        }
    }
}
