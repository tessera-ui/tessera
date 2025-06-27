use std::any::Any;

/// Defines the rendering requirements for a DrawCommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRequirement {
    /// Standard rendering, does not need to sample the background.
    Standard,
    /// Requires sampling the background texture.
    ///
    /// When a command specifies this requirement, the renderer treats it as a render barrier.
    /// It provides the result of all previous draw calls as a texture bound to the bind group set
    /// defined by `crate::renderer::SCENE_TEXTURE_BIND_GROUP_SET`.
    SamplesBackground,
}

/// A command that can be executed by the renderer's `Drawer`.
///
/// This trait is the primary interface for components to submit their rendering needs to the
/// renderer. Each `DrawCommand` is responsible for declaring its rendering requirements, which
/// allows the renderer to optimize and organize the render passes dynamically.
pub trait DrawCommand: Any + Send + Sync {
    /// Returns this `DrawCommand` as a `&dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Specifies the rendering requirement for this command.
    ///
    /// The renderer uses this information to structure the render loop. For example, if
    /// `RenderRequirement::SamplesBackground` is returned, the renderer will ensure that
    /// the current scene is available as a texture for the command's pipeline to sample from.
    fn requirement(&self) -> RenderRequirement;
}
