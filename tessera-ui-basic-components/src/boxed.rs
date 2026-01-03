//! A container for stacking and aligning multiple children.
//!
//! ## Usage
//!
//! Use to create layered UIs, overlays, or composite controls.
use derive_setters::Setters;
use tessera_ui::{
    ComputedData, Constraint, DimensionValue, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, Modifier, Px, PxPosition, tessera,
};

use crate::alignment::Alignment;

/// Arguments for the `Boxed` component.
#[derive(Clone, Debug, Setters)]
pub struct BoxedArgs {
    /// The alignment of children within the `Boxed` container.
    pub alignment: Alignment,
    /// Modifier chain applied to the boxed subtree.
    pub modifier: Modifier,
}

impl Default for BoxedArgs {
    fn default() -> Self {
        Self {
            alignment: Alignment::default(),
            modifier: Modifier::new(),
        }
    }
}

/// A scope for declaratively adding children to a `boxed` component.
pub struct BoxedScope<'a> {
    child_closures: &'a mut Vec<Box<dyn FnOnce() + Send + Sync>>,
    child_alignments: &'a mut Vec<Option<Alignment>>,
}

impl<'a> BoxedScope<'a> {
    /// Adds a child component to the box.
    pub fn child<F>(&mut self, child_closure: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
        self.child_alignments.push(None);
    }

    /// Adds a child component with a custom alignment overriding the container
    /// default.
    pub fn child_with_alignment<F>(&mut self, alignment: Alignment, child_closure: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
        self.child_alignments.push(Some(alignment));
    }
}

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
/// - `args` — configures default alignment and modifiers; see [`BoxedArgs`].
/// - `scope_config` — a closure that receives a [`BoxedScope`] for adding
///   children.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::alignment::Alignment;
/// use tessera_ui_basic_components::boxed::{BoxedArgs, boxed};
/// use tessera_ui_basic_components::text::{TextArgs, text};
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// boxed(BoxedArgs::default(), |scope| {
///     // Add a child that will be in the background (rendered first).
///     scope.child(|| {
///         text(TextArgs::default().text("Background"));
///     });
///     // Add another child aligned to the center, which will appear on top.
///     scope.child_with_alignment(Alignment::Center, || {
///         text(TextArgs::default().text("Foreground"));
///     });
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn boxed<F>(args: BoxedArgs, scope_config: F)
where
    F: FnOnce(&mut BoxedScope),
{
    let modifier = args.modifier;

    let mut child_closures: Vec<Box<dyn FnOnce() + Send + Sync>> = Vec::new();
    let mut child_alignments: Vec<Option<Alignment>> = Vec::new();

    {
        let mut scope = BoxedScope {
            child_closures: &mut child_closures,
            child_alignments: &mut child_alignments,
        };
        scope_config(&mut scope);
    }

    modifier.run(move || boxed_inner(args, child_closures, child_alignments));
}

#[tessera]
fn boxed_inner(
    args: BoxedArgs,
    child_closures: Vec<Box<dyn FnOnce() + Send + Sync>>,
    child_alignments: Vec<Option<Alignment>>,
) {
    layout(BoxedLayout {
        alignment: args.alignment,
        child_alignments,
    });

    for child_closure in child_closures {
        child_closure();
    }
}

#[derive(Clone, PartialEq)]
struct BoxedLayout {
    alignment: Alignment,
    child_alignments: Vec<Option<Alignment>>,
}

impl LayoutSpec for BoxedLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let n = self.child_alignments.len();
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
                let child_alignment = self.child_alignments[i].unwrap_or(self.alignment);
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
