use std::any::Any;
use derive_builder::Builder;
use tessera_macros::tessera;
use tessera::{
    renderer::{DrawCommand, RenderRequirement},
    DimensionValue,
    Px,
};

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct FluidGlassArgs {
    #[builder(default = "[1.0, 1.0, 1.0, 0.0]")]
    pub bleed_color: [f32; 4],
    #[builder(default = "[1.0, 1.0, 1.0, 0.0]")]
    pub highlight_color: [f32; 4],
    #[builder(default = "[0.0, 0.0, 0.0, 0.0]")]
    pub inner_shadow_color: [f32; 4],
    #[builder(default = "30.0")]
    pub corner_radius: f32,
    #[builder(default = "0.0")]
    pub dispersion_height: f32,
    #[builder(default = "1.0")]
    pub chroma_multiplier: f32,
    #[builder(default = "20.0")]
    pub refraction_height: f32,
    #[builder(default = "-60.0")]
    pub refraction_amount: f32,
    #[builder(default = "1.0")]
    pub eccentric_factor: f32,
    #[builder(default = "0.0")]
    pub bleed_amount: f32,
    #[builder(default = "0.0")]
    pub highlight_size: f32,
    #[builder(default = "8.0")]
    pub highlight_smoothing: f32,
    #[builder(default = "0.0")]
    pub inner_shadow_radius: f32,
    #[builder(default = "2.0")]
    pub inner_shadow_smoothing: f32,
    #[builder(default = "0.02")]
    pub noise_amount: f32,
    #[builder(default = "1.5")]
    pub noise_scale: f32,
}

impl Default for FluidGlassArgs {
    fn default() -> Self {
        FluidGlassArgsBuilder::default().build().unwrap()
    }
}

#[derive(Clone)]
pub struct FluidGlassCommand {
    pub args: FluidGlassArgs,
}

impl DrawCommand for FluidGlassCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn requirement(&self) -> RenderRequirement {
        RenderRequirement::SamplesBackground
    }
}

#[tessera(
    measure = |tree, node| {
        let command = FluidGlassCommand {
            args: node.props.clone(),
        };
        tree[node.id].commands.push(Box::new(command));

        let s = &mut tree[node.id].style;
        if s.width.is_auto() {
            s.width = DimensionValue::Px(Px(100.0));
        }
        if s.height.is_auto() {
            s.height = DimensionValue::Px(Px(100.0));
        }
        (s.width.to_px(0.0), s.height.to_px(0.0))
    }
)]
pub fn fluid_glass(_args: FluidGlassArgs) {}