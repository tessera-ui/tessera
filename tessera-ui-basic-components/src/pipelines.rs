//! Render and compute pipelines backing the basic components.
//!
//! Register these pipelines once during renderer initialization before rendering components.

/// Blur pipeline utilities for glass effects.
pub mod blur;
/// Animated checkmark pipeline.
pub mod checkmark;
/// Contrast compute pipeline.
pub mod contrast;
pub(crate) mod fluid_glass;
/// Mean (box) blur compute pipeline.
pub mod mean;
mod pos_misc;
/// Shape pipeline for filled, outlined, and ripple-animated primitives.
pub mod shape;
/// Simple rectangle pipeline used for solid fills.
pub mod simple_rect;
/// Text rendering pipeline.
pub mod text;

/// Image pipeline for raster assets.
pub mod image;
/// Image pipeline for vector (SVG) assets.
pub mod image_vector;

mod compute;
mod draw;

pub use checkmark::{CheckmarkCommand, CheckmarkPipeline};
pub use image_vector::{ImageVectorCommand, ImageVectorPipeline};
pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use simple_rect::{SimpleRectCommand, SimpleRectPipeline};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

/// Register all draw and compute pipelines required by this crate.
pub fn register_pipelines(app: &mut tessera_ui::renderer::WgpuApp) {
    draw::register(app);
    compute::register(app);
}
