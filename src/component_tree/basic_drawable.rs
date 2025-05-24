use crate::renderer::{DrawCommand, ShapeVertex, TextConstraint};

/// These are very basic drawables that trans into DrawerCommand with start position
pub enum BasicDrawable {
    /// A rectangle
    Rect {
        /// filled color(RGB)
        color: [f32; 3],
    },
    /// A Text
    Text {
        /// text content
        text: String,
        /// color of the text(RGB)
        color: [f32; 3],
        /// font size
        font_size: f32,
        /// line height
        line_height: f32,
    },
}

impl BasicDrawable {
    /// Convert BasicDrawable to a DrawCommand
    pub fn to_draw_command(&self, size: [u32; 2], position: [u32; 2]) -> DrawCommand {
        match self {
            BasicDrawable::Rect { color } => DrawCommand::Shape {
                vertices: vec![
                    ShapeVertex {
                        position: [position[0], position[1]],
                        color: *color,
                    },
                    ShapeVertex {
                        position: [position[0] + size[0], position[1]],
                        color: *color,
                    },
                    ShapeVertex {
                        position: [position[0] + size[0], position[1] + size[1]],
                        color: *color,
                    },
                    ShapeVertex {
                        position: [position[0], position[1] + size[1]],
                        color: *color,
                    },
                ],
            },
            BasicDrawable::Text {
                text,
                color,
                font_size,
                line_height,
            } => DrawCommand::Text {
                text: text.clone(),
                position,
                color: *color,
                size: *font_size,
                line_height: *line_height,
                constraint: TextConstraint {
                    max_width: size[0],
                    max_height: size[1],
                },
            },
        }
    }
}

#[test]
fn test_basic_drawable_to_draw_command() {
    let rect = BasicDrawable::Rect {
        color: [1.0, 0.0, 0.0], // Red
    };
    let text = BasicDrawable::Text {
        text: "Hello".to_string(),
        color: [0.0, 1.0, 0.0], // Green
        font_size: 16.0,
        line_height: 20.0,
    };

    let rect_command = rect.to_draw_command([100, 100], [0, 0]);
    let text_command = text.to_draw_command([100, 50], [10, 10]);

    assert_eq!(
        rect_command,
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
        }
    );

    assert_eq!(
        text_command,
        DrawCommand::Text {
            text: "Hello".to_string(),
            position: [10, 10],
            color: [0.0, 1.0, 0.0],
            size: 16.0,
            line_height: 20.0,
            constraint: TextConstraint {
                max_width: 100,
                max_height: 50,
            },
        }
    );
}
