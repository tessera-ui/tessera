use glam::{Vec2, Vec4};
use tessera_ui::{Color, DrawCommand, PxPosition, PxSize};

use super::pipeline::ShapeUniforms;

/// Represents a shape drawable
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeCommand {
    /// A filled rectangle
    Rect {
        /// Color of the rectangle (RGBA)
        color: Color,
        /// Corner radii of the rectangle (tl, tr, br, bl)
        corner_radii: [f32; 4],
        /// G2 exponent per corner (tl, tr, br, bl).
        /// k=2.0 results in standard G1 circular corners.
        corner_g2: [f32; 4],
        /// Shadow properties of the rectangle
        shadow: Option<ShadowProps>,
    },
    /// An outlined rectangle
    OutlinedRect {
        /// Color of the border (RGBA)
        color: Color,
        /// Corner radii of the rectangle (tl, tr, br, bl)
        corner_radii: [f32; 4],
        /// G2 exponent per corner (tl, tr, br, bl).
        /// k=2.0 results in standard G1 circular corners.
        corner_g2: [f32; 4],
        /// Shadow properties of the rectangle (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
    },
    /// A filled rectangle with ripple effect animation
    RippleRect {
        /// Color of the rectangle (RGBA)
        color: Color,
        /// Corner radii of therectangle (tl, tr, br, bl)
        corner_radii: [f32; 4],
        /// G2 exponent per corner (tl, tr, br, bl).
        /// k=2.0 results in standard G1 circular corners.
        corner_g2: [f32; 4],
        /// Shadow properties of the rectangle
        shadow: Option<ShadowProps>,
        /// Ripple effect properties
        ripple: RippleProps,
    },
    /// An outlined rectangle with ripple effect animation
    RippleOutlinedRect {
        /// Color of the border (RGBA)
        color: Color,
        /// Corner radii of the rectangle (tl, tr, br, bl)
        corner_radii: [f32; 4],
        /// G2 exponent per corner (tl, tr, br, bl).
        /// k=2.0 results in standard G1 circular corners.
        corner_g2: [f32; 4],
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
    /// A filled rectangle with an outline
    FilledOutlinedRect {
        /// Color of the rectangle (RGBA)
        color: Color,
        /// Color of the border (RGBA)
        border_color: Color,
        /// Corner radii of the rectangle (tl, tr, br, bl)
        corner_radii: [f32; 4],
        /// G2 exponent per corner (tl, tr, br, bl).
        /// k=2.0 results in standard G1 circular corners.
        corner_g2: [f32; 4],
        /// Shadow properties of the rectangle (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
    },
    /// A filled rectangle with an outline and ripple effect animation
    RippleFilledOutlinedRect {
        /// Color of the rectangle (RGBA)
        color: Color,
        /// Color of the border (RGBA)
        border_color: Color,
        /// Corner radii of the rectangle (tl, tr, br, bl)
        corner_radii: [f32; 4],
        /// G2 exponent per corner (tl, tr, br, bl).
        /// k=2.0 results in standard G1 circular corners.
        corner_g2: [f32; 4],
        /// Shadow properties of the rectangle (applied to the outline shape)
        shadow: Option<ShadowProps>,
        /// Width of the border
        border_width: f32,
        /// Ripple effect properties
        ripple: RippleProps,
    },
    /// A filled ellipse with an outline
    FilledOutlinedEllipse {
        /// Color of the ellipse (RGBA)
        color: Color,
        /// Color of the border (RGBA)
        border_color: Color,
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

impl Default for ShadowProps {
    fn default() -> Self {
        Self {
            color: Color::BLACK.with_alpha(0.25),
            offset: [0.0, 2.0],
            smoothness: 4.0,
        }
    }
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

pub(crate) fn rect_to_uniforms(
    command: &ShapeCommand,
    size: PxSize,
    position: PxPosition,
) -> ShapeUniforms {
    let (
        primary_color_rgba,
        border_color_rgba,
        corner_radii,
        corner_g2,
        shadow,
        border_width,
        render_mode,
        ripple,
    ) = match command {
        ShapeCommand::Rect {
            color,
            corner_radii,
            corner_g2,
            shadow,
        } => (
            *color,
            Color::TRANSPARENT,
            *corner_radii,
            *corner_g2,
            *shadow,
            0.0,
            0.0,
            None,
        ),
        ShapeCommand::OutlinedRect {
            color,
            corner_radii,
            corner_g2,
            shadow,
            border_width,
        } => (
            *color,
            Color::TRANSPARENT,
            *corner_radii,
            *corner_g2,
            *shadow,
            *border_width,
            1.0,
            None,
        ),
        ShapeCommand::RippleRect {
            color,
            corner_radii,
            corner_g2,
            shadow,
            ripple,
        } => (
            *color,
            Color::TRANSPARENT,
            *corner_radii,
            *corner_g2,
            *shadow,
            0.0,
            3.0,
            Some(*ripple),
        ),
        ShapeCommand::RippleOutlinedRect {
            color,
            corner_radii,
            corner_g2,
            shadow,
            border_width,
            ripple,
        } => (
            *color,
            Color::TRANSPARENT,
            *corner_radii,
            *corner_g2,
            *shadow,
            *border_width,
            4.0,
            Some(*ripple),
        ),
        ShapeCommand::Ellipse { color, shadow } => (
            *color,
            Color::TRANSPARENT,
            [-1.0, -1.0, -1.0, -1.0],
            [0.0; 4],
            *shadow,
            0.0,
            0.0,
            None,
        ),
        ShapeCommand::OutlinedEllipse {
            color,
            shadow,
            border_width,
        } => (
            *color,
            Color::TRANSPARENT,
            [-1.0, -1.0, -1.0, -1.0],
            [0.0; 4],
            *shadow,
            *border_width,
            1.0,
            None,
        ),
        ShapeCommand::FilledOutlinedRect {
            color,
            border_color,
            corner_radii,
            corner_g2,
            shadow,
            border_width,
        } => (
            *color,
            *border_color,
            *corner_radii,
            *corner_g2,
            *shadow,
            *border_width,
            5.0,
            None,
        ),
        ShapeCommand::RippleFilledOutlinedRect {
            color,
            border_color,
            corner_radii,
            corner_g2,
            shadow,
            border_width,
            ripple,
        } => (
            *color,
            *border_color,
            *corner_radii,
            *corner_g2,
            *shadow,
            *border_width,
            5.0,
            Some(*ripple),
        ),
        ShapeCommand::FilledOutlinedEllipse {
            color,
            border_color,
            shadow,
            border_width,
        } => (
            *color,
            *border_color,
            [-1.0, -1.0, -1.0, -1.0],
            [0.0; 4],
            *shadow,
            *border_width,
            5.0,
            None,
        ),
    };

    let width = size.width;
    let height = size.height;

    let (shadow_rgba_color, shadow_offset_vec, shadow_smooth_val) = if let Some(s_props) = shadow {
        (s_props.color, s_props.offset, s_props.smoothness)
    } else {
        (Color::TRANSPARENT, [0.0, 0.0], 0.0)
    };

    let (ripple_params, ripple_color) = if let Some(r_props) = ripple {
        (
            Vec4::new(
                r_props.center[0],
                r_props.center[1],
                r_props.radius,
                r_props.alpha,
            ),
            Vec4::new(r_props.color.r, r_props.color.g, r_props.color.b, 0.0),
        )
    } else {
        (Vec4::ZERO, Vec4::ZERO)
    };

    ShapeUniforms {
        corner_radii: corner_radii.into(),
        corner_g2: corner_g2.into(),
        primary_color: primary_color_rgba.to_array().into(),
        border_color: border_color_rgba.to_array().into(),
        shadow_color: shadow_rgba_color.to_array().into(),
        render_params: [
            shadow_offset_vec[0],
            shadow_offset_vec[1],
            shadow_smooth_val,
            render_mode,
        ]
        .into(),
        ripple_params,
        ripple_color,
        border_width,
        position: [
            position.x.to_f32(),
            position.y.to_f32(),
            width.to_f32(),
            height.to_f32(),
        ]
        .into(),
        screen_size: Vec2::ZERO, // Will be populated in the pipeline
    }
}
