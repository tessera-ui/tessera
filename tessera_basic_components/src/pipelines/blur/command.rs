use tessera::wgpu;

/// A synchronous command to execute a gaussian blur.
pub struct BlurCommand<'a> {
    /// The texture view to be used as the source for the blur.
    pub source_view: &'a wgpu::TextureView,
    /// The texture view to write the blur result into.
    pub dest_view: &'a wgpu::TextureView,
    /// The radius of the blur.
    pub radius: f32,
    /// The direction of the blur: (1.0, 0.0) for horizontal, (0.0, 1.0) for vertical.
    pub direction: (f32, f32),
    /// The size of the texture to be blurred (width, height).
    pub size: (u32, u32),
}
