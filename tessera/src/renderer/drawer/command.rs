use std::any::Any;

/// Defines the rendering requirements for a DrawCommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRequirement {
    /// Standard rendering, does not need to sample the background.
    Standard,
    /// Requires sampling the background texture.
    SamplesBackground,
}

/// Every draw command is a command that can be executed by the drawer.
pub trait DrawCommand: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    /// Specifies the rendering requirement for this command.
    /// The renderer will use this to dynamically organize render passes.
    fn requirement(&self) -> RenderRequirement;
}
