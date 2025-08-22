//! Miscellaneous position utilities for rendering pipelines.
//!
//! This module provides utility functions for coordinate conversion and position calculations,
//! primarily focused on converting pixel-based UI positions to normalized device coordinates (NDC).
//! It is intended for use in shape, text, and effect pipelines where consistent coordinate normalization
//! is required for rendering. The conversion assumes a top-left origin, matching typical UI conventions,
//! and ensures compatibility with graphics APIs that expect NDC input.
//!
//! Typical scenarios include transforming UI element positions for GPU-based rendering, shader pipelines,
//! and any context where pixel-to-NDC mapping is necessary for visual correctness.

use tessera_ui::PxPosition;

/// Converts a pixel position to normalized device coordinates (NDC).
///
/// The origin is at the top-left corner, matching UI coordinate conventions.
///
/// # Parameters
/// - `pos`: The pixel position to convert.
/// - `screen_size`: The size of the screen as [width, height].
///
/// # Returns
/// An array `[x, y]` representing the NDC coordinates.
///
/// # Example
///
/// ```
/// use tessera_ui::PxPosition;
/// use tessera_ui_basic_components::pipelines::pos_misc::pixel_to_ndc;
///
/// let ndc = pixel_to_ndc(PxPosition::new(100, 50), [800, 600]);
/// ```
pub fn pixel_to_ndc(pos: PxPosition, screen_size: [u32; 2]) -> [f32; 2] {
    // Guard against zero dimensions to avoid division by zero.
    let width = (screen_size[0].max(1)) as f32;
    let height = (screen_size[1].max(1)) as f32;

    // Convert pixel coordinates to floats once for clarity and to avoid repeated casts.
    let px_x = pos.x.to_f32();
    let px_y = pos.y.to_f32();

    // Normalize to [0, 1], then map to NDC [-1, 1].
    let ndc_x = (px_x / width) * 2.0 - 1.0;
    let ndc_y = (px_y / height) * 2.0 - 1.0;

    // Convert UI top-left origin to NDC bottom-left origin by inverting Y.
    // Keep the existing x sign convention for compatibility with caller code.
    [-ndc_x + 0.0 /* symmetry clarity */, -ndc_y]
}
