use tessera::{ComponentNode, Constraint, LayoutDescription, PositionRelation, TesseraRuntime};

/// A simple row component that arranges its children horizontally.
pub fn row<const N: usize>(children: [&dyn Fn(); N]) {
    {
        // Add a row node
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(|_, children| {
                    let mut x_offset = 0;
                    children
                        .iter()
                        .map(|size| {
                            let result = LayoutDescription {
                                relative_position: PositionRelation {
                                    offset_x: x_offset,
                                    offset_y: 0,
                                },
                            };
                            x_offset += size.width;
                            result
                        })
                        .collect()
                }),
                constraint: Constraint::NONE,
                drawable: None,
            });
    }

    children.iter().for_each(|child| {
        // Execute each child component function
        child();
    });

    {
        // Pop the row node from the component tree
        TesseraRuntime::write().component_tree.pop_node();
    }
}
