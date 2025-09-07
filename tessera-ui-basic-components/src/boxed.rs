//! Provides the `Boxed` component for overlaying multiple child components in a single container.
//!
//! The `Boxed` module enables stacking and aligning several UI elements on top of each other,
//! making it ideal for building layered interfaces, overlays, decorations, or custom backgrounds.
//! Children are positioned according to the specified [`Alignment`],
//! and the container size adapts to the largest child or can be customized via [`DimensionValue`].
//!
//! Typical use cases include tooltips, badges, composite controls, or any scenario where
//! multiple widgets need to share the same space with flexible alignment.
use derive_builder::Builder;
use tessera_ui::{ComputedData, Constraint, DimensionValue, Px, PxPosition, place_node, tessera};

use crate::alignment::Alignment;

/// Arguments for the `Boxed` component.
#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned")]
pub struct BoxedArgs {
    /// The alignment of children within the `Boxed` container.
    #[builder(default)]
    pub alignment: Alignment,
    /// Width behavior for the boxed container.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub width: DimensionValue,
    /// Height behavior for the boxed container.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
}

impl Default for BoxedArgs {
    fn default() -> Self {
        BoxedArgsBuilder::default().build().unwrap()
    }
}

/// A scope for declaratively adding children to a `boxed` component.
pub struct BoxedScope<'a> {
    child_closures: &'a mut Vec<Box<dyn FnOnce() + Send + Sync>>,
}

impl<'a> BoxedScope<'a> {
    /// Adds a child component to the box.
    pub fn child<F>(&mut self, child_closure: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.child_closures.push(Box::new(child_closure));
    }
}

/// Helper: resolve an effective final dimension from a DimensionValue and the largest child size.
/// Keeps logic concise and documented in one place.
fn resolve_final_dimension(dv: DimensionValue, largest_child: Px) -> Px {
    match dv {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Fill { min, max } => {
            let mut v = max.unwrap_or(largest_child);
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

/// Helper: compute centered offset along one axis.
fn center_axis(container: Px, child: Px) -> Px {
    (container - child) / 2
}

/// Helper: compute child placement (x, y) inside the container according to alignment.
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

/// A component that overlays its children on top of each other.
///
/// The `boxed` component acts as a container that stacks all its child components.
/// The size of the container is determined by the dimensions of the largest child,
/// and the alignment of the children within the container can be customized.
///
/// Children are added via the `scope` closure, which provides a `BoxedScope`
/// to add children declaratively.
#[tessera]
pub fn boxed<F>(args: BoxedArgs, scope_config: F)
where
    F: FnOnce(&mut BoxedScope),
{
    let mut child_closures: Vec<Box<dyn FnOnce() + Send + Sync>> = Vec::new();

    {
        let mut scope = BoxedScope {
            child_closures: &mut child_closures,
        };
        scope_config(&mut scope);
    }

    let n = child_closures.len();

    // Measurement closure: measure all present children and compute container size.
    measure(Box::new(move |input| {
        assert_eq!(
            input.children_ids.len(),
            n,
            "Mismatch between children defined in scope and runtime children count"
        );

        let boxed_intrinsic_constraint = Constraint::new(args.width, args.height);
        let effective_constraint = boxed_intrinsic_constraint.merge(input.parent_constraint);

        // Track largest child sizes
        let mut max_child_width = Px(0);
        let mut max_child_height = Px(0);
        let mut children_sizes = vec![None; n];

        let children_to_measure: Vec<_> = input
            .children_ids
            .iter()
            .map(|&child_id| (child_id, effective_constraint))
            .collect();

        let children_results = input.measure_children(children_to_measure)?;

        for (i, &child_id) in input.children_ids.iter().enumerate().take(n) {
            if let Some(child_result) = children_results.get(&child_id) {
                max_child_width = max_child_width.max(child_result.width);
                max_child_height = max_child_height.max(child_result.height);
                children_sizes[i] = Some(*child_result);
            }
        }

        // Resolve final container dimensions using helpers.
        let final_width = resolve_final_dimension(effective_constraint.width, max_child_width);
        let final_height = resolve_final_dimension(effective_constraint.height, max_child_height);

        // Place each measured child according to alignment.
        for (i, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child_id = input.children_ids[i];
                let (x, y) = compute_child_offset(
                    args.alignment,
                    final_width,
                    final_height,
                    child_size.width,
                    child_size.height,
                );
                place_node(child_id, PxPosition::new(x, y), input.metadatas);
            }
        }

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    // Render child closures after measurement.
    for child_closure in child_closures {
        child_closure();
    }
}
