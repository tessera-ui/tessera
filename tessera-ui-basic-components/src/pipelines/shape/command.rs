use tessera_ui::{Color, DrawCommand, PxPosition, PxSize};

use super::{ShapeUniforms, ShapeVertex};

/// Represents a shape drawable
#[derive(Debug, Clone)]
pub enum ShapeCommand {
    /// A filled rectangle
    Rect {
        /// Color of the rectangle (RGBA)
        color: Color,
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle
        shadow: Option<ShadowProps>,
    },
    /// An outlined rectangle
    OutlinedRect {
        /// Color of the border (RGBA)
        color: Color,
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
    },
    /// A filled rectangle with ripple effect animation
    RippleRect {
        /// Color of the rectangle (RGBA)
        color: Color,
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle
        shadow: Option<ShadowProps>,
        /// Ripple effect properties
        ripple: RippleProps,
    },
    /// An outlined rectangle with ripple effect animation
    RippleOutlinedRect {
        /// Color of the border (RGBA)
        color: Color,
        /// Corner radius of the rectangle
        corner_radius: f32,
        /// Shadow properties of the rectangle (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
        /// Ripple effect properties
        ripple: RippleProps,
    },
    /// A filled ellipse
    Ellipse {
        /// Color of the ellipse (RGBA)
        color: Color,
        /// Shadow properties of the ellipse
        shadow: Option<ShadowProps>,
    },
    /// An outlined ellipse
    OutlinedEllipse {
        /// Color of the border (RGBA)
        color: Color,
        /// Shadow properties of the ellipse (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
    },
}

impl DrawCommand for ShapeCommand {
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        // No specific barrier requirements for shape commands
        None
    }
}

/// Properties for shadow, used in BasicDrawable variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowProps {
    /// Color of the shadow (RGBA)
    pub color: Color,
    /// Offset of the shadow in the format [x, y]
    pub offset: [f32; 2],
    /// Smoothness of the shadow, typically a value between 0.0 and 1.0
    pub smoothness: f32,
}

/// Properties for ripple effect animation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RippleProps {
    /// Center position of the ripple in normalized coordinates [-0.5, 0.5]
    pub center: [f32; 2],
    /// Current radius of the ripple (0.0 to 1.0, where 1.0 covers the entire shape)
    pub radius: f32,
    /// Alpha value for the ripple effect (0.0 to 1.0)
    pub alpha: f32,
    /// Color of the ripple effect (RGB)
    pub color: Color,
}

impl Default for RippleProps {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            radius: 0.0,
            alpha: 0.0,
            color: Color::WHITE,
        }
    }
}

pub struct ShapeCommandComputed {
    pub(crate) vertices: Vec<ShapeVertex>,
    pub(crate) uniforms: ShapeUniforms,
}

impl ShapeCommandComputed {
    pub fn from_command(command: ShapeCommand, size: PxSize, position: PxPosition) -> Self {
        match command {
            ShapeCommand::Rect {
                color,
                corner_radius,
                shadow,
            } => rect_to_computed_draw_command(
                size,
                position,
                color, // RGBA
                corner_radius,
                shadow,
                0.0, // border_width for fill is 0
                0.0, // render_mode for fill is 0.0
            ),
            ShapeCommand::OutlinedRect {
                color,
                corner_radius,
                shadow,
                border_width,
            } => rect_to_computed_draw_command(
                size,
                position,
                color, // RGBA, This color is for the border
                corner_radius,
                shadow,
                border_width,
                1.0, // render_mode for outline is 1.0
            ),
            ShapeCommand::RippleRect {
                color,
                corner_radius,
                shadow,
                ripple,
            } => ripple_rect_to_computed_draw_command(
                size,
                position,
                color,
                corner_radius,
                shadow,
                0.0, // border_width for fill is 0
                0.0, // render_mode for fill is 0.0
                ripple,
            ),
            ShapeCommand::RippleOutlinedRect {
                color,
                corner_radius,
                shadow,
                border_width,
                ripple,
            } => ripple_rect_to_computed_draw_command(
                size,
                position,
                color,
                corner_radius,
                shadow,
                border_width,
                1.0, // render_mode for outline is 1.0
                ripple,
            ),
            ShapeCommand::Ellipse { color, shadow } => rect_to_computed_draw_command(
                size, position, color,
                -1.0, // Use negative corner_radius to signify an ellipse to the shader
                shadow, 0.0, // border_width for fill is 0
                0.0, // render_mode for fill
            ),
            ShapeCommand::OutlinedEllipse {
                color,
                shadow,
                border_width,
            } => rect_to_computed_draw_command(
                size,
                position,
                color,
                -1.0, // Use negative corner_radius to signify an ellipse to the shader
                shadow,
                border_width,
                1.0, // render_mode for outline
            ),
        }
    }
}

/// Helper function to create Shape DrawCommand for both Rect and OutlinedRect
fn rect_to_computed_draw_command(
    size: PxSize,
    position: PxPosition,
    primary_color_rgba: Color,
    corner_radius: f32,
    shadow: Option<ShadowProps>,
    border_width: f32,
    render_mode: f32,
) -> ShapeCommandComputed {
    let width = size.width;
    let height = size.height;

    let rect_local_pos = [
        [-0.5, -0.5], // Top-Left
        [0.5, -0.5],  // Top-Right
        [0.5, 0.5],   // Bottom-Right
        [-0.5, 0.5],  // Bottom-Left
    ];

    let vertex_color_placeholder_rgb = [0.0, 0.0, 0.0];
    let top_left = position.to_f32_arr3();
    let top_right = [top_left[0] + width.to_f32(), top_left[1], top_left[2]];
    let bottom_right = [
        top_left[0] + width.to_f32(),
        top_left[1] + height.to_f32(),
        top_left[2],
    ];
    let bottom_left = [top_left[0], top_left[1] + height.to_f32(), top_left[2]];

    let vertices = vec![
        ShapeVertex {
            position: top_left,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[0],
        },
        ShapeVertex {
            position: top_right,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[1],
        },
        ShapeVertex {
            position: bottom_right,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[2],
        },
        ShapeVertex {
            position: bottom_left,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[3],
        },
    ];

    let (shadow_rgba_color, shadow_offset_vec, shadow_smooth_val) = if let Some(s_props) = shadow {
        (s_props.color, s_props.offset, s_props.smoothness)
    } else {
        (Color::TRANSPARENT, [0.0, 0.0], 0.0)
    };

    let uniforms = ShapeUniforms {
        size_cr_border_width: [width.to_f32(), height.to_f32(), corner_radius, border_width],
        primary_color: primary_color_rgba.into(),
        shadow_color: shadow_rgba_color.into(),
        render_params: [
            shadow_offset_vec[0],
            shadow_offset_vec[1],
            shadow_smooth_val,
            render_mode,
        ],
        ripple_params: [0.0, 0.0, 0.0, 0.0],
        ripple_color: [0.0, 0.0, 0.0, 0.0],
    };

    ShapeCommandComputed { vertices, uniforms }
}

/// Helper function to create Shape DrawCommand for ripple effects
fn ripple_rect_to_computed_draw_command(
    size: PxSize,
    position: PxPosition,
    primary_color_rgba: Color,
    corner_radius: f32,
    shadow: Option<ShadowProps>,
    border_width: f32,
    render_mode: f32,
    ripple: RippleProps,
) -> ShapeCommandComputed {
    let width = size.width;
    let height = size.height;

    let rect_local_pos = [
        [-0.5, -0.5], // Top-Left
        [0.5, -0.5],  // Top-Right
        [0.5, 0.5],   // Bottom-Right
        [-0.5, 0.5],  // Bottom-Left
    ];

    let vertex_color_placeholder_rgb = [0.0, 0.0, 0.0];
    let top_left = position.to_f32_arr3();
    let top_right = [top_left[0] + width.to_f32(), top_left[1], top_left[2]];
    let bottom_right = [
        top_left[0] + width.to_f32(),
        top_left[1] + height.to_f32(),
        top_left[2],
    ];
    let bottom_left = [top_left[0], top_left[1] + height.to_f32(), top_left[2]];

    let vertices = vec![
        ShapeVertex {
            position: top_left,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[0],
        },
        ShapeVertex {
            position: top_right,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[1],
        },
        ShapeVertex {
            position: bottom_right,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[2],
        },
        ShapeVertex {
            position: bottom_left,
            color: vertex_color_placeholder_rgb,
            local_pos: rect_local_pos[3],
        },
    ];

    let (shadow_rgba_color, shadow_offset_vec, shadow_smooth_val) = if let Some(s_props) = shadow {
        (s_props.color.into(), s_props.offset, s_props.smoothness)
    } else {
        ([0.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.0)
    };

    let ripple_render_mode = if render_mode == 0.0 { 3.0 } else { 4.0 };

    let uniforms = ShapeUniforms {
        size_cr_border_width: [width.to_f32(), height.to_f32(), corner_radius, border_width],
        primary_color: primary_color_rgba.into(),
        shadow_color: shadow_rgba_color,
        render_params: [
            shadow_offset_vec[0],
            shadow_offset_vec[1],
            shadow_smooth_val,
            ripple_render_mode,
        ],
        ripple_params: [
            ripple.center[0],
            ripple.center[1],
            ripple.radius,
            ripple.alpha,
        ],
        ripple_color: [ripple.color.r, ripple.color.g, ripple.color.b, 0.0],
    };

    ShapeCommandComputed { vertices, uniforms }
}
