//! A container for stacking and aligning multiple children.
//!
//! ## Usage
//!
//! Use to create layered UIs, overlays, or composite controls.
use tessera_ui::{
    ComputedData, Constraint, DimensionValue, LayoutInput, LayoutOutput, LayoutPolicy,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, layout::layout_primitive, tessera,
};

use crate::alignment::Alignment;

fn resolve_final_dimension(dv: DimensionValue, largest_child: Px) -> Px {
    match dv {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Fill { min, max } => {
            let Some(max) = max else {
                panic!(
                    "Seems that you are trying to fill an infinite dimension, which is not allowed\nboxed constraint = {dv:?}"
                );
            };
            let mut v = max.max(largest_child);
            if let Some(min_v) = min {
                v = v.max(min_v);
            }
            v
        }
        DimensionValue::Wrap { min, max } => {
            let mut v = largest_child;
            if let Some(min_v) = min {
                v = v.max(min_v);
            }
            if let Some(max_v) = max {
                v = v.min(max_v);
            }
            v
        }
    }
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
pub fn boxed(alignment: Alignment, modifier: Modifier, children: RenderSlot) {
    layout_primitive()
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
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_alignments = collect_child_alignments(input);
        let n = child_alignments.len();
        debug_assert_eq!(
            input.children_ids().len(),
            n,
            "Mismatch between children defined in scope and runtime children count"
        );

        let effective_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

        let mut max_child_width = Px(0);
        let mut max_child_height = Px(0);
        let mut children_sizes = vec![None; n];

        let children_to_measure: Vec<_> = input
            .children_ids()
            .iter()
            .map(|&child_id| (child_id, effective_constraint))
            .collect();

        let children_results = input.measure_children(children_to_measure)?;

        for (i, &child_id) in input.children_ids().iter().enumerate().take(n) {
            if let Some(child_result) = children_results.get(&child_id) {
                max_child_width = max_child_width.max(child_result.width);
                max_child_height = max_child_height.max(child_result.height);
                children_sizes[i] = Some(*child_result);
            }
        }

        let final_width = resolve_final_dimension(effective_constraint.width, max_child_width);
        let final_height = resolve_final_dimension(effective_constraint.height, max_child_height);

        for (i, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child_id = input.children_ids()[i];
                let child_alignment = child_alignments[i].unwrap_or(self.alignment);
                let (x, y) = compute_child_offset(
                    child_alignment,
                    final_width,
                    final_height,
                    child_size.width,
                    child_size.height,
                );
                output.place_child(child_id, PxPosition::new(x, y));
            }
        }

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}

fn collect_child_alignments(input: &LayoutInput<'_>) -> Vec<Option<Alignment>> {
    input
        .children_ids()
        .iter()
        .map(|&child_id| {
            input
                .child_parent_data::<crate::modifier::AlignmentParentData>(child_id)
                .map(|data| data.alignment)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        ComputedData, DimensionValue, LayoutInput, LayoutOutput, LayoutPolicy, MeasurementError,
        Modifier, NoopRenderPolicy, Px, layout::layout_primitive, tessera,
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
        fn measure(
            &self,
            _input: &LayoutInput<'_>,
            _output: &mut LayoutOutput<'_>,
        ) -> Result<ComputedData, MeasurementError> {
            Ok(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            })
        }
    }

    #[tessera]
    fn fixed_test_box(tag: String, width: i32, height: i32) {
        layout_primitive()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn boxed_layout_case() {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(Px::new(100))),
                Some(DimensionValue::Fixed(Px::new(80))),
            ))
            .children(|| {
                boxed_start_box();
                layout_primitive()
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
                Some(DimensionValue::Fixed(Px::new(100))),
                Some(DimensionValue::Fixed(Px::new(80))),
            ))
            .children(|| {
                fixed_test_box()
                    .tag("boxed_center".to_string())
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
}
