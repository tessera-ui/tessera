//! Provides the `Boxed` component for overlaying multiple child components in a single container.
//!
//! The `Boxed` module enables stacking and aligning several UI elements on top of each other,
//! making it ideal for building layered interfaces, overlays, decorations, or custom backgrounds.
//! Children are positioned according to the specified [`Alignment`](crate::alignment::Alignment),
//! and the container size adapts to the largest child or can be customized via [`DimensionValue`].
//!
//! Typical use cases include tooltips, badges, composite controls, or any scenario where
//! multiple widgets need to share the same space with flexible alignment.
//!
//! This module also provides supporting types and a macro for ergonomic usage.
use derive_builder::Builder;
use tessera_ui::{ComputedData, Constraint, DimensionValue, Px, PxPosition, tessera};

use crate::alignment::Alignment;

pub use crate::boxed_ui;

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

/// `BoxedItem` represents a stackable child component.
pub struct BoxedItem {
    pub child: Box<dyn FnOnce() + Send + Sync>,
}

impl BoxedItem {
    pub fn new(child: Box<dyn FnOnce() + Send + Sync>) -> Self {
        BoxedItem { child }
    }
}

/// A trait for converting various types into a `BoxedItem`.
pub trait AsBoxedItem {
    fn into_boxed_item(self) -> BoxedItem;
}

impl AsBoxedItem for BoxedItem {
    fn into_boxed_item(self) -> BoxedItem {
        self
    }
}

impl<F: FnOnce() + Send + Sync + 'static> AsBoxedItem for F {
    fn into_boxed_item(self) -> BoxedItem {
        BoxedItem {
            child: Box::new(self),
        }
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
/// It's useful for creating layered UIs where components need to be placed
/// relative to a common parent.
///
/// # Arguments
///
/// * `args`: A `BoxedArgs` struct that specifies the configuration for the container.
///   - `alignment`: Controls how children are positioned within the box.
///     See [`Alignment`](crate::alignment::Alignment) for available options.
///   - `width`: The width of the container. Can be fixed, fill the parent, or wrap the content.
///     See [`DimensionValue`](tessera_ui::DimensionValue) for details.
///   - `height`: The height of the container. Can be fixed, fill the parent, or wrap the content.
///     See [`DimensionValue`](tessera_ui::DimensionValue) for details.
///
/// * `children_items_input`: An array of child components to be rendered inside the box.
///   Any component that implements the `AsBoxedItem` trait can be a child.
#[tessera]
pub fn boxed<const N: usize>(args: BoxedArgs, children_items_input: [impl AsBoxedItem; N]) {
    // Convert inputs to boxed items and collect their closures.
    let children_items: [BoxedItem; N] =
        children_items_input.map(|item_input| item_input.into_boxed_item());

    let mut child_closures = Vec::with_capacity(N);
    for child_item in children_items {
        child_closures.push(child_item.child);
    }

    // Measurement closure: measure all present children and compute container size.
    measure(Box::new(move |input| {
        let boxed_intrinsic_constraint = Constraint::new(args.width, args.height);
        let effective_constraint = boxed_intrinsic_constraint.merge(input.parent_constraint);

        // Track largest child sizes
        let mut max_child_width = Px(0);
        let mut max_child_height = Px(0);
        let mut children_sizes = vec![None; N];

        for (i, &child_id) in input.children_ids.iter().enumerate().take(N) {
            let child_result = input.measure_child(child_id, &effective_constraint)?;
            max_child_width = max_child_width.max(child_result.width);
            max_child_height = max_child_height.max(child_result.height);
            children_sizes[i] = Some(child_result);
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
                input.place_child(child_id, PxPosition::new(x, y));
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

/// A macro for simplifying `Boxed` component declarations.
#[macro_export]
macro_rules! boxed_ui {
    ($args:expr $(, $child:expr)* $(,)?) => {
        {
            use $crate::boxed::AsBoxedItem;
            $crate::boxed::boxed($args, [
                $(
                    $child.into_boxed_item()
                ),*
            ])
        }
    };
}
