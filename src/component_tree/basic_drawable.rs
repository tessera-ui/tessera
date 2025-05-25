use crate::renderer::{DrawCommand, ShapeVertex, TextData};

/// These are very basic drawables that trans into DrawerCommand
/// , basically just copys of DrawerCommand without position
pub enum BasicDrawable {
    /// A rectangle
    Rect {
        /// filled color(RGB)
        color: [f32; 3],
    },
    /// A Text
    Text {
        /// Text data to draw(without position setted)
        data: TextData,
    },
}

impl BasicDrawable {
    /// Convert BasicDrawable to a DrawCommand
    pub fn into_draw_command(
        self,
        size: [u32; 2],
        position: [u32; 2],
    ) -> DrawCommand {
        match self {
            BasicDrawable::Rect { color } => {
                let width = size[0];
                let height = size[1];
                DrawCommand::Shape {
                vertices: vec![
                    ShapeVertex {
                        position: [position[0], position[1]],
                        color,
                    },
                    ShapeVertex {
                        position: [position[0] + width, position[1]],
                        color,
                    },
                    ShapeVertex {
                        position: [position[0] + width, position[1] + height],
                        color,
                    },
                    ShapeVertex {
                        position: [position[0], position[1] + height],
                        color,
                    },
                ],
            }
            },
            BasicDrawable::Text {
                mut data
            } => {
                data.position = Some(position);
                DrawCommand::Text {
                    data,
                }
            }
        }
    }
}
