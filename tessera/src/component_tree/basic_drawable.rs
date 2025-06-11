use crate::renderer::{DrawCommand, ShapeUniforms, ShapeVertex, TextData};

/// These are very basic drawables that trans into DrawerCommand
/// , basically just copys of DrawerCommand without position
pub enum BasicDrawable {
    /// A filled rectangle
    Rect {
        /// Color of the rectangle (RGBA)
        color: [f32; 4],
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle
        shadow: Option<ShadowProps>,
    },
    /// An outlined rectangle
    OutlinedRect {
        /// Color of the border (RGBA)
        color: [f32; 4],
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
    },
    /// A Text
    Text {
        /// Text data to draw(without position setted)
        data: TextData,
    },
}

/// Properties for shadow, used in BasicDrawable variants
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
                Self::rect_to_draw_command(
                    size,
                    position,
                    color, // RGBA
                    corner_radius,
                    shadow,
                    0.0, // border_width for fill is 0
                    0.0, // render_mode for fill is 0.0
                )
            }
            BasicDrawable::OutlinedRect {
                color,
                corner_radius,
                shadow,
                border_width,
            } => {
                Self::rect_to_draw_command(
                    size,
                    position,
                    color, // RGBA, This color is for the border
                    corner_radius,
                    shadow,
                    border_width,
                    1.0, // render_mode for outline is 1.0
                )
            }
            BasicDrawable::Text { mut data } => {
                data.position = Some(position);
                DrawCommand::Text { data }
            }
        }
    }

    /// Helper function to create Shape DrawCommand for both Rect and OutlinedRect
    fn rect_to_draw_command(
        size: [u32; 2],
        position: [u32; 2],
        primary_color_rgba: [f32; 4], // Changed from primary_color_rgb
        corner_radius: f32,
        shadow: Option<ShadowProps>,
        border_width: f32,
        render_mode: f32,
    ) -> DrawCommand {
        let width = size[0];
        let height = size[1];

        let rect_local_pos = [
            [-0.5, -0.5], // Top-Left
            [0.5, -0.5],  // Top-Right
            [0.5, 0.5],   // Bottom-Right
            [-0.5, 0.5],  // Bottom-Left
        ];

        // Vertex color is less important now as shader uses uniform primary_color
        let vertex_color_placeholder_rgb = [0.0, 0.0, 0.0]; // Kept as RGB for vertex data

        let vertices = vec![
            ShapeVertex {
                position: [position[0] as f32, position[1] as f32, 0.0],
                color: vertex_color_placeholder_rgb,
                local_pos: rect_local_pos[0],
            },
            ShapeVertex {
                position: [(position[0] + width) as f32, position[1] as f32, 0.0],
                color: vertex_color_placeholder_rgb,
                local_pos: rect_local_pos[1],
            },
            ShapeVertex {
                position: [
                    (position[0] + width) as f32,
                    (position[1] + height) as f32,
                    0.0,
                ],
                color: vertex_color_placeholder_rgb,
                local_pos: rect_local_pos[2],
            },
            ShapeVertex {
                position: [position[0] as f32, (position[1] + height) as f32, 0.0],
                color: vertex_color_placeholder_rgb,
                local_pos: rect_local_pos[3],
            },
        ];

        // primary_color_rgba is now directly used
        // let primary_rgba_color = [primary_color_rgb[0], primary_color_rgb[1], primary_color_rgb[2], 1.0f32];

        let (shadow_rgba_color, shadow_offset_vec, shadow_smooth_val) =
            if let Some(s_props) = shadow {
                (s_props.color, s_props.offset, s_props.smoothness)
            } else {
                ([0.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.0)
            };

        let uniforms = ShapeUniforms {
            size_cr_border_width: [width as f32, height as f32, corner_radius, border_width],
            primary_color: primary_color_rgba, // Directly use the RGBA color
            shadow_color: shadow_rgba_color,
            render_params: [
                shadow_offset_vec[0],
                shadow_offset_vec[1],
                shadow_smooth_val,
                render_mode, // 0.0 for fill, 1.0 for outline
            ],
        };

        DrawCommand::Shape { vertices, uniforms }
    }
}
