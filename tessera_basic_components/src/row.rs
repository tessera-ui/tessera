use tessera::{
    ComponentNodeMetaDatas, ComponentNodeTree, ComputedData, Constraint, NodeId, measure_node,
    place_node,
};
use tessera_macros::tessera;

/// Represents a child of a row with a weight and a child component.
/// The weight determines how much space the child should take relative to other children.
/// The rule is, we first check if the constraint has a max width and whether chidren can fit in it
/// with min height.
/// if it doesn't, we just ignore the weight and ask child to measure itself with no width constraint,
/// but if it does, we first measure all children(without weight) to get their min width and then
/// use the remaining space to distribute among children based on their weights.
pub struct RowChild {
    pub weight: Option<f32>,
    pub child: &'static dyn Fn(),
}

impl RowChild {
    /// Creates a new `RowChild` with the given weight and child component.
    pub fn new(weight: Option<f32>, child: &'static dyn Fn()) -> Self {
        RowChild { weight, child }
    }
}

pub trait AsRowChild {
    fn as_row_child(self) -> RowChild;
}

impl<F: Fn()> AsRowChild for &'static F {
    fn as_row_child(self) -> RowChild {
        RowChild {
            weight: None,
            child: self,
        }
    }
}

/// A simple row component that arranges its children horizontally.
#[tessera]
pub fn row<const N: usize>(children: [RowChild; N]) {
    let weights = children
        .iter()
        .map(|child| child.weight)
        .collect::<Vec<_>>();
    measure(Box::new(
        move |_, tree, constraint, children_ids, metadatas| {
            if constraint.max_width.is_none() {
                // if no width constraint, use default measure policy
                return no_width_measure_policy(tree, constraint, children_ids, metadatas);
            }
            // first, measure all children without width constraint
            let test_constraint = Constraint {
                min_width: None,
                max_width: None,
                min_height: constraint.min_height,
                max_height: constraint.max_height,
            };
            let test_measurements: Vec<_> = children_ids
                .iter()
                .map(|&child_id| measure_node(child_id, &test_constraint, tree, metadatas))
                .collect();
            // calculate total width of test measurements
            let total_width = test_measurements.iter().map(|size| size.width).sum::<u32>();
            // if total width is less than max width, we can use no width measure policy
            if total_width <= constraint.max_width.unwrap() {
                return no_width_measure_policy(tree, constraint, children_ids, metadatas);
            }
            // else we need to measure unweighted children first
            // for unweighted children, we just simply use test_measurements's results
            let actual_measurements: Vec<_> = weights
                .iter()
                .zip(test_measurements.iter())
                .map(|(weight, size)| {
                    if weight.is_some() {
                        None // this is a weighted child, we will measure it later
                    } else {
                        Some(*size) // this is an unweighted child, use its size directly
                    }
                })
                .collect();
            // calculate total width of unweighted children
            let unweighted_total_width = actual_measurements
                .iter()
                .filter_map(|&size| size)
                .map(|size| size.width)
                .sum::<u32>();
            // calculate remaining width and total weight for weighted children
            let remaining_width = constraint
                .max_width
                .unwrap()
                .saturating_sub(unweighted_total_width);
            let total_weight: f32 = weights.iter().filter_map(|&weight| weight).sum();
            // and now we can measure weighted children and place all children
            let mut offset_x = 0;
            let mut row_height = 0;
            for (index, size) in actual_measurements.iter().enumerate() {
                if let Some(size) = size {
                    // a unweighted child, place it directly
                    place_node(children_ids[index], [offset_x, 0], metadatas);
                    offset_x += size.width;
                    row_height = row_height.max(size.height);
                } else {
                    // a weighted child, we need to measure it with remaining width
                    let weight = weights[index].unwrap();
                    let width = (weight / total_weight * remaining_width as f32) as u32;
                    let constraint = Constraint {
                        min_width: Some(width),
                        max_width: Some(width),
                        min_height: constraint.min_height,
                        max_height: constraint.max_height,
                    };
                    let child_size =
                        measure_node(children_ids[index], &constraint, tree, metadatas);
                    place_node(children_ids[index], [offset_x, 0], metadatas);
                    offset_x += width;
                    row_height = row_height.max(child_size.height);
                }
            }
            // use offset_x as the total width of the row
            let row_width = offset_x;
            ComputedData {
                width: row_width,
                height: row_height,
            }
        },
    ));

    for child in children {
        (child.child)();
    }
}

// default measure policy if no width constraint is provided
fn no_width_measure_policy(
    tree: &ComponentNodeTree,
    constraint: &Constraint,
    children_ids: &[NodeId],
    metadatas: &mut ComponentNodeMetaDatas,
) -> ComputedData {
    // we take total width and maximum height of children
    let mut width = 0;
    let mut height = 0;
    for &child_id in children_ids {
        let child_size = measure_node(child_id, constraint, tree, metadatas);
        width += child_size.width;
        height = height.max(child_size.height);
    }
    // place children, starting from the left corner of self
    let mut x_offset = 0;
    for &child_id in children_ids {
        place_node(child_id, [x_offset, 0], metadatas);
        x_offset += metadatas
            .get(&child_id)
            .unwrap()
            .computed_data
            .unwrap()
            .width;
    }
    // return the size of the row
    ComputedData { width, height }
}
