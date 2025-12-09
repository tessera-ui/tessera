//! Provides a GPU-accelerated, animated checkmark component for UI elements.
//!
//! This module defines the [`checkmark`] function and related types for rendering a customizable,
//! animated checkmark, typically used to visually indicate a "checked" state in controls such as
//! checkboxes. The checkmark supports smooth animation, color and stroke customization, and is
//! rendered using Tessera's GPU pipeline for high performance and visual fidelity.
//!
//! # Typical Usage
//!
//! The checkmark is most commonly used within checkbox components, but can be integrated into any
//! UI element requiring a checkmark indicator. It is suitable for applications needing responsive,
//! theme-adaptable, and animated visual feedback for selection or confirmation states.
//!
//! # Features
//! - Customizable color, stroke width, size, and padding
//! - Smooth animation progress control
//! - High-performance GPU rendering
//!
//! See [`CheckmarkArgs`] for configuration options and usage examples in the [`checkmark`] function documentation.
use derive_builder::Builder;
use tessera_ui::{Color, ComputedData, Dp, Px, tessera};

use crate::pipelines::checkmark::command::CheckmarkCommand;

/// Arguments for the `checkmark` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct CheckmarkArgs {
    /// Color of the checkmark stroke
    #[builder(default = "Color::new(0.0, 0.6, 0.0, 1.0)")]
    pub color: Color,

    /// Width of the checkmark stroke in pixels
    #[builder(default = "5.0")]
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
        CheckmarkArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

/// Renders a checkmark, a visual indicator that is displayed when in a `checked` state.
///
/// This component is a GPU-rendered checkmark that provides a smooth, animated alternative
/// to traditional emoji or icon-based checkmarks. It supports customization of color,
/// stroke width, and animation progress.
///
/// # Arguments
///
/// The `args` parameter accepts any value that can be converted into `CheckmarkArgs`,
/// including a `CheckmarkArgs` struct or its builder.
///
/// *   `color`: The `Color` of the checkmark stroke. Defaults to a green color.
/// *   `stroke_width`: The width of the checkmark stroke in pixels. Defaults to `5.0`.
/// *   `progress`: The animation progress for drawing the checkmark, from `0.0` (not drawn)
///     to `1.0` (fully drawn). Defaults to `1.0`.
/// *   `padding`: The padding `[horizontal, vertical]` around the checkmark within its bounds.
///     Defaults to `[2.0, 2.0]`.
/// *   `size`: The size of the checkmark area as a `Dp` value. Defaults to `Dp(20.0)`.

#[tessera]
pub fn checkmark(args: impl Into<CheckmarkArgs>) {
    let args: CheckmarkArgs = args.into();

    let size_px = args.size.to_px();

    // Create the checkmark command
    let command = CheckmarkCommand::new()
        .with_color(args.color)
        .with_stroke_width(args.stroke_width)
        .with_progress(args.progress)
        .with_padding(args.padding[0], args.padding[1]);

    // Measure the component and push the draw command within the measure function
    measure(Box::new(move |input| {
        // Push the draw command to the current node's metadata
        input.metadata_mut().push_draw_command(command);

        Ok(ComputedData {
            width: Px::new(size_px.to_f32() as i32),
            height: Px::new(size_px.to_f32() as i32),
        })
    }));
}
