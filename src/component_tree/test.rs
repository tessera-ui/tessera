use crate::{
    component_tree::{
        ComponentTree,
        basic_drawable::BasicDrawable,
        node::{
            ComponentNode, Constraint, DEFAULT_LAYOUT_DESC, LayoutDescription, PositionRelation,
        },
    },
    renderer::{DrawCommand, ShapeVertex},
};

#[test]
fn test_component_tree() {
    // Create a new ComponentTree
    let mut tree = ComponentTree::new();

    // Add a root node
    // Here we draw a rectangle
    // with a size of 100x100 and a color of red
    // and place child node in center of the rectangle
    tree.add_node(ComponentNode {
        layout_desc: Box::new(|inputs| {
            let input = inputs[0];
            let x = 100 / 2 - input.width / 2;
            let y = 100 / 2 - input.height / 2;
            vec![LayoutDescription {
                relative_position: PositionRelation {
                    offset_x: x,
                    offset_y: y,
                },
            }]
        }),
        constraint: Constraint {
            min_width: 100,
            min_height: 100,
            max_width: 100,
            max_height: 100,
        },
        drawable: Some(BasicDrawable::Rect {
            color: [1.0, 0.0, 0.0], // Red
        }),
    });

    // Add a child node
    // Here we draw a rectangle with a size of 50x50 and a color of blue
    tree.add_node(ComponentNode {
        layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
        constraint: Constraint {
            min_width: 50,
            min_height: 50,
            max_width: 50,
            max_height: 50,
        },
        drawable: Some(BasicDrawable::Rect {
            color: [0.0, 0.0, 1.0], // Blue
        }),
    });

    let commands = tree.compute();

    // Check if we have the expected number of commands
    assert_eq!(commands.len(), 2);

    // Check if commands are expected
    assert_eq!(
        commands,
        vec![
            DrawCommand::Shape {
                vertices: vec![
                    ShapeVertex {
                        position: [0, 0],
                        color: [1.0, 0.0, 0.0],
                    },
                    ShapeVertex {
                        position: [100, 0],
                        color: [1.0, 0.0, 0.0],
                    },
                    ShapeVertex {
                        position: [100, 100],
                        color: [1.0, 0.0, 0.0],
                    },
                    ShapeVertex {
                        position: [0, 100],
                        color: [1.0, 0.0, 0.0],
                    },
                ],
            },
            DrawCommand::Shape {
                vertices: vec![
                    ShapeVertex {
                        position: [25, 25],
                        color: [0.0, 0.0, 1.0],
                    },
                    ShapeVertex {
                        position: [75, 25],
                        color: [0.0, 0.0, 1.0],
                    },
                    ShapeVertex {
                        position: [75, 75],
                        color: [0.0, 0.0, 1.0],
                    },
                    ShapeVertex {
                        position: [25, 75],
                        color: [0.0, 0.0, 1.0],
                    },
                ],
            },
        ]
    );
}
