use derive_builder::Builder;
use tessera_ui::{ComputedData, Constraint, DimensionValue, Px, PxPosition, place_node};
use tessera_ui_macros::tessera;

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

/// The `Boxed` component: stacks all its children, with the size being
/// that of the largest child.
#[tessera]
pub fn boxed<const N: usize>(args: BoxedArgs, children_items_input: [impl AsBoxedItem; N]) {
    let children_items: [BoxedItem; N] =
        children_items_input.map(|item_input| item_input.into_boxed_item());

    let mut child_closures = Vec::with_capacity(N);

    for child_item in children_items {
        child_closures.push(child_item.child);
    }

    measure(Box::new(move |input| {
        let boxed_intrinsic_constraint = Constraint::new(args.width, args.height);
        let effective_constraint = boxed_intrinsic_constraint.merge(input.parent_constraint);

        let mut max_child_width = Px(0);
        let mut max_child_height = Px(0);
        let mut children_sizes = vec![None; N];

        for i in 0..N {
            let child_id = input.children_ids[i];
            let child_result = tessera_ui::measure_node(
                child_id,
                &effective_constraint,
                input.tree,
                input.metadatas,
                input.compute_resource_manager.clone(),
                input.gpu,
            )?;
            max_child_width = max_child_width.max(child_result.width);
            max_child_height = max_child_height.max(child_result.height);
            children_sizes[i] = Some(child_result);
        }

        let final_width = match effective_constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Fill { min, max } => {
                let mut w = max.unwrap_or(max_child_width);
                if let Some(min_w) = min {
                    w = w.max(min_w);
                }
                w
            }
            DimensionValue::Wrap { min, max } => {
                let mut w = max_child_width;
                if let Some(min_w) = min {
                    w = w.max(min_w);
                }
                if let Some(max_w) = max {
                    w = w.min(max_w);
                }
                w
            }
        };

        let final_height = match effective_constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Fill { min, max } => {
                let mut h = max.unwrap_or(max_child_height);
                if let Some(min_h) = min {
                    h = h.max(min_h);
                }
                h
            }
            DimensionValue::Wrap { min, max } => {
                let mut h = max_child_height;
                if let Some(min_h) = min {
                    h = h.max(min_h);
                }
                if let Some(max_h) = max {
                    h = h.min(max_h);
                }
                h
            }
        };

        for (i, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child_id = input.children_ids[i];

                let (x, y) = match args.alignment {
                    Alignment::TopStart => (Px(0), Px(0)),
                    Alignment::TopCenter => ((final_width - child_size.width) / 2, Px(0)),
                    Alignment::TopEnd => (final_width - child_size.width, Px(0)),
                    Alignment::CenterStart => (Px(0), (final_height - child_size.height) / 2),
                    Alignment::Center => (
                        (final_width - child_size.width) / 2,
                        (final_height - child_size.height) / 2,
                    ),
                    Alignment::CenterEnd => (
                        final_width - child_size.width,
                        (final_height - child_size.height) / 2,
                    ),
                    Alignment::BottomStart => (Px(0), final_height - child_size.height),
                    Alignment::BottomCenter => (
                        (final_width - child_size.width) / 2,
                        final_height - child_size.height,
                    ),
                    Alignment::BottomEnd => (
                        final_width - child_size.width,
                        final_height - child_size.height,
                    ),
                };
                place_node(child_id, PxPosition::new(x, y), input.metadatas);
            }
        }

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

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
