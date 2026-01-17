//! Render and compute pipelines backing the basic components.
//!
//! Register these pipelines once during renderer initialization before
//! rendering components.

pub(crate) mod blur;
pub(crate) mod checkmark;
pub(crate) mod contrast;
pub(crate) mod fluid_glass;
pub(crate) mod image;
pub(crate) mod image_vector;
pub(crate) mod mean;
pub(crate) mod pos_misc;
pub(crate) mod progress_arc;
pub(crate) mod shadow;
pub(crate) mod shape;
pub(crate) mod simple_rect;
pub(crate) mod text;

mod compute;
mod draw;

/// Register all draw and compute pipelines required by this crate.
pub fn register_pipelines(context: &mut tessera_ui::PipelineContext<'_>) {
    draw::register(context);
    compute::register(context);
}
