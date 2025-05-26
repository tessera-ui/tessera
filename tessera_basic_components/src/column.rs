use tessera::{ComponentNode, Constraint, LayoutDescription, PositionRelation, TesseraRuntime};

/// A simple column component that arranges its children vertically.s
pub fn column<const N: usize>(children: [&dyn Fn(); N]) {
    {
        // Add a column node
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(|_, children| {
                    let mut y_offset = 0;
                    children
                        .iter()
                        .map(|size| {
                            let result = LayoutDescription {
                                relative_position: PositionRelation {
                                    offset_x: 0,
                                    offset_y: y_offset,
                                },
                            };
                            y_offset += size.height;
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
        // Pop the column node from the component tree
        TesseraRuntime::write().component_tree.pop_node();
    }
}
