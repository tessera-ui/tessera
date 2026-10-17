//! Shape clipping for rendered content and pointer input.
//!
//! ## Usage
//!
//! Restrict a subtree to rounded corners or an elliptical viewport.

use crate::{PxPosition, PxRect};

/// A shape expressed in physical pixels, relative to a node's bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClipShape {
    /// A rounded rectangle with top-left, top-right, bottom-right, and
    /// bottom-left radii and curvature exponents.
    RoundedRectangle {
        /// Corner radii in physical pixels.
        radii: [f32; 4],
        /// Corner curvature exponents; two produces circular corners.
        exponents: [f32; 4],
    },
    /// An ellipse inscribed in the node's bounds.
    Ellipse,
}

/// A clipping shape in physical coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipRegion {
    /// Origin in physical pixels, including fractional placement.
    pub position: PxPosition,
    /// Width and height in physical pixels, including fractional sizes.
    pub size: [f32; 2],
    /// Shape inscribed in the bounds.
    pub shape: ClipShape,
}

impl ClipRegion {
    /// Returns an enclosing integer rectangle, rounded outwards.
    pub fn bounds(self) -> PxRect {
        PxRect::enclosing_f32(self.position, self.size)
    }

    /// Tests whether an absolute point lies inside the clipping shape.
    pub fn contains(self, position: PxPosition) -> bool {
        let local = position - self.position;
        if !(local.x >= 0.0 && local.y >= 0.0 && local.x < self.size[0] && local.y < self.size[1]) {
            return false;
        }
        let half = [self.size[0] * 0.5, self.size[1] * 0.5];
        let point = [local.x - half[0], local.y - half[1]];
        match self.shape {
            ClipShape::Ellipse => {
                (point[0] / half[0]).powi(2) + (point[1] / half[1]).powi(2) <= 1.0
            }
            ClipShape::RoundedRectangle { radii, exponents } => {
                let corner = match (point[0] < 0.0, point[1] < 0.0) {
                    (true, true) => 0,
                    (false, true) => 1,
                    (false, false) => 2,
                    (true, false) => 3,
                };
                let radius = finite_radius(radii[corner], half[0].min(half[1]));
                if radius == 0.0 {
                    return true;
                }
                let exponent = finite_exponent(exponents[corner]);
                let q = [
                    (point[0].abs() - half[0] + radius).max(0.0) / radius,
                    (point[1].abs() - half[1] + radius).max(0.0) / radius,
                ];
                q[0].powf(exponent) + q[1].powf(exponent) <= 1.0
            }
        }
    }
}

pub(crate) fn finite_radius(radius: f32, max: f32) -> f32 {
    if radius.is_finite() {
        radius.clamp(0.0, max.max(0.0))
    } else {
        0.0
    }
}

pub(crate) fn finite_exponent(exponent: f32) -> f32 {
    if exponent.is_finite() {
        exponent.clamp(1.0, 32.0)
    } else {
        2.0
    }
}

/// Depth and stencil configuration required by UI draw pipelines.
///
/// Set `RenderPipelineDescriptor::depth_stencil` to this value for pipelines
/// submitted through the UI renderer. Independent texture-generation passes
/// may use their own attachment configuration.
pub fn ui_stencil_state() -> Option<wgpu::DepthStencilState> {
    Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Stencil8,
        depth_write_enabled: None,
        depth_compare: None,
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                ..Default::default()
            },
            back: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                ..Default::default()
            },
            read_mask: 0xff,
            write_mask: 0,
        },
        bias: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use crate::Px;

    use super::*;

    #[test]
    fn shape_hit_testing_matches_mask_geometry() {
        let mut region = ClipRegion {
            position: PxPosition::from_f32_xy(10.0, 20.0),
            size: [100.0, 60.0],
            shape: ClipShape::Ellipse,
        };
        assert!(region.contains(PxPosition::from_f32_xy(60.0, 50.0)));
        assert!(!region.contains(PxPosition::from_f32_xy(11.0, 21.0)));
        region.shape = ClipShape::RoundedRectangle {
            radii: [20.0; 4],
            exponents: [2.0; 4],
        };
        assert!(!region.contains(PxPosition::from_f32_xy(11.0, 21.0)));
        assert!(region.contains(PxPosition::from_f32_xy(30.0, 21.0)));
        assert!(!region.contains(PxPosition::from_f32_xy(109.0, 79.0)));
    }

    #[test]
    fn invalid_radii_and_exponents_are_sanitized() {
        assert_eq!(finite_radius(f32::NAN, 10.0), 0.0);
        assert_eq!(finite_radius(100.0, 10.0), 10.0);
        assert_eq!(finite_exponent(f32::INFINITY), 2.0);
        assert_eq!(finite_exponent(-1.0), 1.0);
    }

    #[test]
    fn fractional_masks_keep_geometry_and_round_scissors_outwards() {
        let region = ClipRegion {
            position: PxPosition::from_f32_xy(-0.25, 0.75),
            size: [20.0, 10.0],
            shape: ClipShape::RoundedRectangle {
                radii: [0.0; 4],
                exponents: [2.0; 4],
            },
        };
        assert_eq!(region.bounds(), PxRect::new(Px(-1), Px(0), Px(21), Px(11)));
        assert!(!region.contains(PxPosition::from_f32_xy(0.0, 0.5)));
        assert!(region.contains(PxPosition::from_f32_xy(0.0, 0.8)));
        assert!(!region.contains(PxPosition::from_f32_xy(19.8, 5.0)));
    }
}
