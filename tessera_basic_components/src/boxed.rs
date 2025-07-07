use crate::alignment::Alignment;
use derive_builder::Builder;
use tessera::{ComputedData, Constraint, DimensionValue, Px, PxPosition, place_node};
use tessera_macros::tessera;

/// Arguments for the `Boxed` component.
#[derive(Clone, Debug, Default, Builder)]
pub struct BoxedArgs {
    /// The alignment of children within the `Boxed` container.
    pub alignment: Alignment,
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
        let boxed_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
            DimensionValue::Wrap {
                min: None,
                max: None,
            },
        );
        let effective_constraint = boxed_constraint.merge(input.parent_constraint);

        let mut max_width = Px(0);
        let mut max_height = Px(0);
        let mut children_sizes = vec![None; N];

        for i in 0..N {
            let child_id = input.children_ids[i];
            let child_result = tessera::measure_node(
                child_id,
                &effective_constraint,
                input.tree,
                input.metadatas,
            )?;
            max_width = max_width.max(child_result.width);
            max_height = max_height.max(child_result.height);
            children_sizes[i] = Some(child_result);
        }

        for (i, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child_id = input.children_ids[i];

                let (x, y) = match args.alignment {
                    Alignment::TopStart => (Px(0), Px(0)),
                    Alignment::TopCenter => ((max_width - child_size.width) / 2, Px(0)),
                    Alignment::TopEnd => (max_width - child_size.width, Px(0)),
                    Alignment::CenterStart => (Px(0), (max_height - child_size.height) / 2),
                    Alignment::Center => (
                        (max_width - child_size.width) / 2,
                        (max_height - child_size.height) / 2,
                    ),
                    Alignment::CenterEnd => (
                        max_width - child_size.width,
                        (max_height - child_size.height) / 2,
                    ),
                    Alignment::BottomStart => (Px(0), max_height - child_size.height),
                    Alignment::BottomCenter => (
                        (max_width - child_size.width) / 2,
                        max_height - child_size.height,
                    ),
                    Alignment::BottomEnd => {
                        (max_width - child_size.width, max_height - child_size.height)
                    }
                };
                place_node(child_id, PxPosition::new(x, y), input.metadatas);
            }
        }

        Ok(ComputedData {
            width: max_width,
            height: max_height,
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

pub use crate::boxed_ui;
