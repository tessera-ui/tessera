use crate::renderer::{DrawCommand, ShapeUniforms, ShapeVertex, TextData};

/// These are very basic drawables that trans into DrawerCommand
/// , basically just copys of DrawerCommand without position
pub enum BasicDrawable {
    /// A rectangle
    Rect {
        /// Color of the rectangle(RGB)
        color: [f32; 3],
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle
        shadow: Option<ShadowProps>,
    },
    /// A Text
    Text {
        /// Text data to draw(without position setted)
        data: TextData,
    },
}

/// Properties for shadow, used in BasicDrawable::Rect
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowProps {
    /// Color of the shadow (RGBA)
    pub color: [f32; 4],
    /// Offset of the shadow in the format [x, y]
    pub offset: [f32; 2],
    /// Smoothness of the shadow, typically a value between 0.0 and 1.0
    pub smoothness: f32,
}

impl BasicDrawable {
    /// Convert BasicDrawable to a DrawCommand
    pub fn into_draw_command(self, size: [u32; 2], position: [u32; 2]) -> DrawCommand {
        match self {
            BasicDrawable::Rect {
                color,
                corner_radius,
                shadow,
            } => {
                let width = size[0];
                let height = size[1];

                // Define local_pos for the 4 corners of a rectangle, normalized to [-0.5, 0.5]
                // Order: Top-Left, Top-Right, Bottom-Right, Bottom-Left (matching original vertex generation)
                let rect_local_pos = [
                    [-0.5, -0.5], // Top-Left
                    [0.5, -0.5],  // Top-Right
                    [0.5, 0.5],   // Bottom-Right
                    [-0.5, 0.5],  // Bottom-Left
                ];

                let vertices = vec![
                    ShapeVertex {
                        position: [position[0] as f32, position[1] as f32, 0.0], // Top-Left
                        color,
                        local_pos: rect_local_pos[0],
                    },
                    ShapeVertex {
                        position: [(position[0] + width) as f32, position[1] as f32, 0.0], // Top-Right
                        color,
                        local_pos: rect_local_pos[1],
                    },
                    ShapeVertex {
                        position: [
                            (position[0] + width) as f32,
                            (position[1] + height) as f32,
                            0.0,
                        ], // Bottom-Right
                        color,
                        local_pos: rect_local_pos[2],
                    },
                    ShapeVertex {
                        position: [position[0] as f32, (position[1] + height) as f32, 0.0], // Bottom-Left
                        color,
                        local_pos: rect_local_pos[3],
                    },
                ];

                let object_rgba_color = [color[0], color[1], color[2], 1.0f32]; // Assume opaque object color

                let (shadow_rgba_color, shadow_offset_vec, shadow_smooth_val) =
                    if let Some(s_props) = shadow {
                        (s_props.color, s_props.offset, s_props.smoothness)
                    } else {
                        // Default values for uniforms if no shadow, shadow pass might be skipped by Drawer
                        ([0.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.0)
                    };

                let uniforms = ShapeUniforms {
                    size_cr_is_shadow: [width as f32, height as f32, corner_radius, 0.0], // is_shadow (last element) will be set by Drawer for each pass
                    object_color: object_rgba_color,
                    shadow_color: shadow_rgba_color,
                    shadow_params: [
                        shadow_offset_vec[0],
                        shadow_offset_vec[1],
                        shadow_smooth_val,
                        0.0,
                    ], // last element is padding
                };

                DrawCommand::Shape { vertices, uniforms }
            }
            BasicDrawable::Text { mut data } => {
                data.position = Some(position);
                DrawCommand::Text { data }
            }
        }
    }
}
