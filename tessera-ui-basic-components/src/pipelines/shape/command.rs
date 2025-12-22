use glam::{Vec2, Vec4};
use tessera_ui::{Color, DrawCommand, DrawRegion, PaddingRect, Px, PxPosition, PxSize};

use super::pipeline::ShapeUniforms;

const SHADOW_AA_MARGIN_PX: f32 = 1.0;

pub(crate) fn shadow_padding_xy(shadow: &ShadowLayers) -> (Px, Px) {
    let mut pad_x = 0.0f32;
    let mut pad_y = 0.0f32;

    let update = |pad_x: &mut f32, pad_y: &mut f32, layer: &ShadowLayer| {
        if layer.color.a <= 0.0 {
            return;
        }
        let layer_pad_x = (layer.smoothness + layer.offset[0].abs() + SHADOW_AA_MARGIN_PX).max(0.0);
        let layer_pad_y = (layer.smoothness + layer.offset[1].abs() + SHADOW_AA_MARGIN_PX).max(0.0);
        *pad_x = (*pad_x).max(layer_pad_x);
        *pad_y = (*pad_y).max(layer_pad_y);
    };

    if let Some(layer) = shadow.ambient {
        update(&mut pad_x, &mut pad_y, &layer);
    }
    if let Some(layer) = shadow.spot {
        update(&mut pad_x, &mut pad_y, &layer);
    }

    (
        Px::new(pad_x.ceil() as i32).max(Px::ZERO),
        Px::new(pad_y.ceil() as i32).max(Px::ZERO),
    )
}

/// A single shadow layer (ambient or spot)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLayer {
    /// Color of the shadow (RGBA)
    pub color: Color,
    /// Offset of the shadow in the format [x, y]
    pub offset: [f32; 2],
    /// Smoothness / blur of the shadow
    pub smoothness: f32,
}

/// Collection of shadow layers (ambient + spot)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShadowLayers {
    /// Ambient (diffused) shadow layer
    pub ambient: Option<ShadowLayer>,
    /// Spot (directional / offset) shadow layer
    pub spot: Option<ShadowLayer>,
}

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
        /// Shadow properties of the rectangle (ambient + spot)
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
    },
    /// An outlined ellipse
    OutlinedEllipse {
        /// Color of the border (RGBA)
        color: Color,
        /// Shadow properties of the ellipse (applied to the outline shape)
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
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
        shadow: Option<ShadowLayers>,
        /// Width of the border
        border_width: f32,
    },
}

impl ShapeCommand {
    pub(crate) fn shadow(&self) -> Option<&ShadowLayers> {
        match self {
            ShapeCommand::Rect { shadow, .. }
            | ShapeCommand::OutlinedRect { shadow, .. }
            | ShapeCommand::RippleRect { shadow, .. }
            | ShapeCommand::RippleOutlinedRect { shadow, .. }
            | ShapeCommand::Ellipse { shadow, .. }
            | ShapeCommand::OutlinedEllipse { shadow, .. }
            | ShapeCommand::FilledOutlinedRect { shadow, .. }
            | ShapeCommand::RippleFilledOutlinedRect { shadow, .. }
            | ShapeCommand::FilledOutlinedEllipse { shadow, .. } => shadow.as_ref(),
        }
    }
}

impl DrawCommand for ShapeCommand {
    fn sample_region(&self) -> Option<tessera_ui::SampleRegion> {
        // No specific barrier requirements for shape commands
        None
    }

    fn apply_opacity(&mut self, opacity: f32) {
        fn scale_color(color: &mut Color, factor: f32) {
            *color = color.with_alpha(color.a * factor);
        }

        fn scale_shadow(shadow: &mut Option<ShadowLayers>, factor: f32) {
            if let Some(layers) = shadow {
                if let Some(ref mut ambient) = layers.ambient {
                    scale_color(&mut ambient.color, factor);
                }
                if let Some(ref mut spot) = layers.spot {
                    scale_color(&mut spot.color, factor);
                }
            }
        }

        let factor = opacity.clamp(0.0, 1.0);
        match self {
            ShapeCommand::Rect { color, shadow, .. } => {
                scale_color(color, factor);
                scale_shadow(shadow, factor);
            }
            ShapeCommand::OutlinedRect { color, shadow, .. } => {
                scale_color(color, factor);
                scale_shadow(shadow, factor);
            }
            ShapeCommand::RippleRect {
                color,
                shadow,
                ripple,
                ..
            } => {
                scale_color(color, factor);
                scale_shadow(shadow, factor);
                ripple.alpha *= factor;
            }
            ShapeCommand::RippleOutlinedRect {
                color,
                shadow,
                ripple,
                ..
            } => {
                scale_color(color, factor);
                scale_shadow(shadow, factor);
                ripple.alpha *= factor;
            }
            ShapeCommand::Ellipse { color, shadow } => {
                scale_color(color, factor);
                scale_shadow(shadow, factor);
            }
            ShapeCommand::OutlinedEllipse { color, shadow, .. } => {
                scale_color(color, factor);
                scale_shadow(shadow, factor);
            }
            ShapeCommand::FilledOutlinedRect {
                color,
                border_color,
                shadow,
                ..
            } => {
                scale_color(color, factor);
                scale_color(border_color, factor);
                scale_shadow(shadow, factor);
            }
            ShapeCommand::RippleFilledOutlinedRect {
                color,
                border_color,
                shadow,
                ripple,
                ..
            } => {
                scale_color(color, factor);
                scale_color(border_color, factor);
                scale_shadow(shadow, factor);
                ripple.alpha *= factor;
            }
            ShapeCommand::FilledOutlinedEllipse {
                color,
                border_color,
                shadow,
                ..
            } => {
                scale_color(color, factor);
                scale_color(border_color, factor);
                scale_shadow(shadow, factor);
            }
        }
    }

    fn draw_region(&self) -> DrawRegion {
        let Some(layers) = self.shadow() else {
            return DrawRegion::PaddedLocal(PaddingRect::ZERO);
        };

        let (pad_x, pad_y) = shadow_padding_xy(layers);
        DrawRegion::PaddedLocal(PaddingRect {
            top: pad_y,
            right: pad_x,
            bottom: pad_y,
            left: pad_x,
        })
    }
}

/// Properties for ripple effect animation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RippleProps {
    /// Center position of the ripple in normalized coordinates [-0.5, 0.5]
    pub center: [f32; 2],
    /// If true, the ripple is clipped by the shape bounds.
    ///
    /// If false, the ripple is not clipped by the shape (but is still bounded
    /// by the draw quad).
    pub bounded: bool,
    /// Current radius of the ripple (0.0 to 1.0, where 1.0 covers the entire
    /// shape)
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
            bounded: true,
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

    let (ambient_color, ambient_offset, ambient_smooth) = if let Some(layers) = shadow {
        if let Some(a) = layers.ambient {
            (a.color, a.offset, a.smoothness)
        } else {
            (Color::TRANSPARENT, [0.0, 0.0], 0.0)
        }
    } else {
        (Color::TRANSPARENT, [0.0, 0.0], 0.0)
    };

    let (spot_color, spot_offset, spot_smooth) = if let Some(layers) = shadow {
        if let Some(s) = layers.spot {
            (s.color, s.offset, s.smoothness)
        } else {
            (Color::TRANSPARENT, [0.0, 0.0], 0.0)
        }
    } else {
        (Color::TRANSPARENT, [0.0, 0.0], 0.0)
    };

    let (ripple_params, ripple_color) = if let Some(r_props) = ripple {
        let bounded_flag = if r_props.bounded { 1.0 } else { 0.0 };
        (
            Vec4::new(
                r_props.center[0],
                r_props.center[1],
                r_props.radius,
                r_props.alpha,
            ),
            Vec4::new(
                r_props.color.r,
                r_props.color.g,
                r_props.color.b,
                bounded_flag,
            ),
        )
    } else {
        (Vec4::ZERO, Vec4::ZERO)
    };

    ShapeUniforms {
        corner_radii: corner_radii.into(),
        corner_g2: corner_g2.into(),
        primary_color: primary_color_rgba.to_array().into(),
        border_color: border_color_rgba.to_array().into(),
        shadow_ambient_color: ambient_color.to_array().into(),
        shadow_ambient_params: [ambient_offset[0], ambient_offset[1], ambient_smooth].into(),
        shadow_spot_color: spot_color.to_array().into(),
        shadow_spot_params: [spot_offset[0], spot_offset[1], spot_smooth].into(),
        render_mode,
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
