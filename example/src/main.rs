use tessera::{
    BasicDrawable, ComponentNode, Constraint, DEFAULT_LAYOUT_DESC, LayoutDescription,
    PositionRelation, Renderer, TesseraRuntime, TextConstraint, TextData,
};

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    Renderer::run(background)
}

fn background() {
    {
        // Add a root node
        // Here we draw a rectangle with screen size, colored in white
        let window_size = TesseraRuntime::read().window_size;
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(|self_size, children_size| {
                    let input = children_size[0];
                    let x = self_size.width / 2 - input.width / 2;
                    let y = self_size.height / 2 - input.height / 2;
                    vec![LayoutDescription {
                        relative_position: PositionRelation {
                            offset_x: x,
                            offset_y: y,
                        },
                    }]
                }),
                constraint: Constraint {
                    min_width: Some(window_size[0]),
                    min_height: Some(window_size[1]),
                    max_width: Some(window_size[0]),
                    max_height: Some(window_size[1]),
                },
                drawable: Some(BasicDrawable::Rect {
                    color: [1.0, 1.0, 1.0], // Red
                }),
            });
    }

    rect();

    {
        TesseraRuntime::write().component_tree.pop_node();
    }
}

fn rect() {
    {
        // Add a rectangle node
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                constraint: Constraint {
                    min_width: Some(100),
                    min_height: Some(100),
                    max_width: None,
                    max_height: None,
                },
                drawable: Some(BasicDrawable::Rect {
                    color: [255.0, 0.0, 0.0], // Red
                }),
            });
    }

    text();

    {
        TesseraRuntime::write().component_tree.pop_node();
    }
}

fn text() {
    {
        // Add a text node
        TesseraRuntime::write().component_tree.add_node(ComponentNode {
                    layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                    constraint: Constraint::NONE,
                    drawable: Some(BasicDrawable::Text {
                        data: TextData::new(
                            "Hello, this is Tessera~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~".to_string(),
                            [0, 255, 0], // Green
                            50.0,
                            50.0,
                            TextConstraint { max_width: None, max_height: None }
                        ),
                    }),
                });
    }

    {
        TesseraRuntime::write().component_tree.pop_node();
    }
}
