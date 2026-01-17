//! Shadow layer definitions for Material surfaces.
//!
//! ## Usage
//!
//! Define ambient and spot shadow layers for surfaces.

use tessera_ui::Color;

/// A single shadow layer (ambient or spot).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLayer {
    /// Color of the shadow (RGBA).
    pub color: Color,
    /// Offset of the shadow in the format [x, y].
    pub offset: [f32; 2],
    /// Smoothness / blur radius in pixels.
    pub smoothness: f32,
}

/// Collection of shadow layers (ambient + spot).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShadowLayers {
    /// Ambient (diffused) shadow layer.
    pub ambient: Option<ShadowLayer>,
    /// Spot (directional / offset) shadow layer.
    pub spot: Option<ShadowLayer>,
}
