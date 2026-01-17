//! Shadow mask and composite commands for MD3-style shadows.
//!
//! ## Usage
//!
//! Build shadow mask and composite commands for layered surface shadows.

use tessera_ui::{
    Color, DrawCommand, DrawRegion, PaddingRect, PxPosition, PxRect, PxSize, SampleRegion,
};

use crate::shape_def::ResolvedShape;

/// Draw command for rendering a shape mask into an offscreen texture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowMaskCommand {
    /// Resolved shape geometry for the mask.
    pub shape: ResolvedShape,
    /// Mask color (alpha controls opacity).
    pub color: Color,
}

impl ShadowMaskCommand {
    /// Creates a white mask command for the provided shape.
    pub fn new(shape: ResolvedShape) -> Self {
        Self {
            shape,
            color: Color::WHITE,
        }
    }
}

impl DrawCommand for ShadowMaskCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        None
    }

    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::ZERO)
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha((self.color.a * opacity).clamp(0.0, 1.0));
    }
}

/// Draw command for compositing a blurred mask into the scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowCompositeCommand {
    /// Shadow color including alpha.
    pub color: Color,
    /// UV origin in the mask texture (normalized).
    pub uv_origin: [f32; 2],
    /// UV size in the mask texture (normalized).
    pub uv_size: [f32; 2],
    /// Ordering offset applied to the op position.
    pub ordering_offset: PxPosition,
    /// Ordering size used for dependency checks.
    pub ordering_size: PxSize,
}

impl DrawCommand for ShadowCompositeCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        None
    }

    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::ZERO)
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha((self.color.a * opacity).clamp(0.0, 1.0));
    }

    fn ordering_rect(&self, position: PxPosition, _size: PxSize) -> Option<PxRect> {
        if self.ordering_size.width.0 <= 0 || self.ordering_size.height.0 <= 0 {
            return None;
        }
        Some(PxRect::from_position_size(
            position + self.ordering_offset,
            self.ordering_size,
        ))
    }
}

impl ShadowCompositeCommand {
    /// Creates a composite command with full-texture UVs.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            uv_origin: [0.0, 0.0],
            uv_size: [1.0, 1.0],
            ordering_offset: PxPosition::ZERO,
            ordering_size: PxSize::ZERO,
        }
    }

    /// Sets the ordering bounds for render graph scheduling.
    pub fn with_ordering(mut self, ordering_offset: PxPosition, ordering_size: PxSize) -> Self {
        self.ordering_offset = ordering_offset;
        self.ordering_size = ordering_size;
        self
    }
}
