use derive_builder::Builder;
use tessera_ui::{Color, ComputedData, Dp, Px};
use tessera_ui_macros::tessera;

use crate::pipelines::CheckmarkCommand;

/// Arguments for the `checkmark` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct CheckmarkArgs {
    /// Color of the checkmark stroke
    #[builder(default = "Color::new(0.0, 0.6, 0.0, 1.0)")]
    pub color: Color,

    /// Width of the checkmark stroke in pixels
    #[builder(default = "10.0")]
    pub stroke_width: f32,

    /// Animation progress from 0.0 (not drawn) to 1.0 (fully drawn)
    #[builder(default = "1.0")]
    pub progress: f32,

    /// Padding around the checkmark within its bounds
    #[builder(default = "[2.0, 2.0]")]
    pub padding: [f32; 2], // [horizontal, vertical]

    /// Size of the checkmark area
    #[builder(default = "Dp(20.0)")]
    pub size: Dp,
}

impl std::fmt::Debug for CheckmarkArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckmarkArgs")
            .field("color", &self.color)
            .field("stroke_width", &self.stroke_width)
            .field("progress", &self.progress)
            .field("padding", &self.padding)
            .field("size", &self.size)
            .finish()
    }
}

impl Default for CheckmarkArgs {
    fn default() -> Self {
        CheckmarkArgsBuilder::default().build().unwrap()
    }
}

/// A checkmark component that renders an animated checkmark using a custom GPU pipeline.
///
/// This component provides a robust alternative to emoji-based checkmarks, with support
/// for linear drawing animation and customizable appearance.
#[tessera]
pub fn checkmark(args: impl Into<CheckmarkArgs>) {
    let args: CheckmarkArgs = args.into();

    let size_px = args.size.to_px();

    // Create the checkmark command
    let command = CheckmarkCommand {
        color: args.color,
        stroke_width: args.stroke_width,
        progress: args.progress,
        padding: args.padding,
    };

    // Measure the component and push the draw command within the measure function
    measure(Box::new(move |input| {
        // Push the draw command to the current node's metadata
        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.push_draw_command(command.clone());
        }

        Ok(ComputedData {
            width: Px::new(size_px.to_f32() as i32),
            height: Px::new(size_px.to_f32() as i32),
        })
    }));
}
