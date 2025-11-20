pub mod blur;
pub mod checkmark;
pub mod contrast;
pub(crate) mod fluid_glass;
pub mod mean;
mod pos_misc;
pub mod shape;
pub mod simple_rect;
pub mod text;

pub mod image;
pub mod image_vector;

mod compute;
mod draw;

pub use checkmark::{CheckmarkCommand, CheckmarkPipeline};
pub use image_vector::{ImageVectorCommand, ImageVectorPipeline};
pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use simple_rect::{SimpleRectCommand, SimpleRectPipeline};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

pub fn register_pipelines(app: &mut tessera_ui::renderer::WgpuApp) {
    draw::register(app);
    compute::register(app);
}

