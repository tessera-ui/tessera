use tessera::{
    ComponentNodeMetaDatas, ComponentNodeTree, ComputedData, Constraint, NodeId, measure_node,
    place_node,
};
use tessera_macros::tessera;

/// Represents a child of a column with a weight and a child component.
/// The weight determines how much space the child should take relative to other children.
/// The rule is, we first check if the constraint has a max height and whether chidren can fit in it
/// with min width.
/// if it doesn't, we just ignore the weight and ask child to measure itself with no height constraint,
/// but if it does, we first measure all children(without weight) to get their min height and then
/// use the remaining space to distribute among children based on their weights.
pub struct ColumnChild {
    pub weight: Option<f32>,
    pub child: &'static dyn Fn(),
}

impl ColumnChild {
    /// Creates a new `ColumnChild` with the given weight and child component.
    pub fn new(weight: Option<f32>, child: &'static dyn Fn()) -> Self {
        ColumnChild { weight, child }
    }
}

pub trait AsColumnChild {
    fn into_column_child(self) -> ColumnChild;
}

impl<F: Fn()> AsColumnChild for &'static F {
    fn into_column_child(self) -> ColumnChild {
        ColumnChild {
            weight: None,
            child: self,
        }
    }
}

/// A simple column component that arranges its children vertically.
#[tessera]
pub fn column<const N: usize>(children: [ColumnChild; N]) {
    let weights = children
        .iter()
        .map(|child| child.weight)
        .collect::<Vec<_>>();
    measure(Box::new(
        move |_, tree, constraint, children_ids, metadatas| {
            if constraint.max_height.is_none() {
                // if no height constraint, use default measure policy
                return no_height_measure_policy(tree, constraint, children_ids, metadatas);
            }
            // first, measure all children without height constraint
            let test_constraint = Constraint {
                min_width: constraint.min_width,
                max_width: constraint.max_width,
                min_height: None,
                max_height: None,
            };
            let test_measurements: Vec<_> = children_ids
                .iter()
                .map(|&child_id| measure_node(child_id, &test_constraint, tree, metadatas))
                .collect();
            // calculate total height of test measurements
            let total_height = test_measurements
                .iter()
                .map(|size| size.height)
                .sum::<u32>();
            // if total height is less than max height, we can use no height measure policy
            if total_height <= constraint.max_height.unwrap() {
                return no_height_measure_policy(tree, constraint, children_ids, metadatas);
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
            // calculate total height of unweighted children
            let unweighted_total_height = actual_measurements
                .iter()
                .filter_map(|&size| size)
                .map(|size| size.height)
                .sum::<u32>();
            // calculate remaining height and total weight for weighted children
            let remaining_height = constraint
                .max_height
                .unwrap()
                .saturating_sub(unweighted_total_height);
            let total_weight: f32 = weights.iter().filter_map(|&weight| weight).sum();
            // and now we can measure weighted children and place all children
            let mut offset_y = 0;
            let mut column_width = 0;
            for (index, size) in actual_measurements.iter().enumerate() {
                if let Some(size) = size {
                    // a unweighted child, place it directly
                    place_node(children_ids[index], [0, offset_y], metadatas);
                    offset_y += size.height;
                    column_width = column_width.max(size.width);
                } else {
                    // a weighted child, we need to measure it with remaining height
                    let weight = weights[index].unwrap();
                    let height = (weight / total_weight * remaining_height as f32) as u32;
                    let constraint = Constraint {
                        min_width: constraint.min_width,
                        max_width: constraint.max_width,
                        min_height: Some(height),
                        max_height: Some(height),
                    };
                    let child_size =
                        measure_node(children_ids[index], &constraint, tree, metadatas);
                    place_node(children_ids[index], [0, offset_y], metadatas);
                    offset_y += height;
                    column_width = column_width.max(child_size.width);
                }
            }
            // use offset_y as the total height of the column
            let column_height = offset_y;
            ComputedData {
                width: column_width,
                height: column_height,
            }
        },
    ));

    for child in children {
        (child.child)();
    }
}

// default measure policy if no height constraint is provided
fn no_height_measure_policy(
    tree: &ComponentNodeTree,
    constraint: &Constraint,
    children_ids: &[NodeId],
    metadatas: &mut ComponentNodeMetaDatas,
) -> ComputedData {
    // we take total height and maximum width of children
    let mut width = 0;
    let mut height = 0;
    for &child_id in children_ids {
        let child_size = measure_node(child_id, constraint, tree, metadatas);
        height += child_size.height;
        width = width.max(child_size.width);
    }
    // place children, starting from the top corner of self
    let mut y_offset = 0;
    for &child_id in children_ids {
        place_node(child_id, [0, y_offset], metadatas);
        y_offset += metadatas
            .get(&child_id)
            .unwrap()
            .computed_data
            .unwrap()
            .height;
    }
    // return the size of the column
    ComputedData { width, height }
}
