//! Provides a GPU-accelerated, animated checkmark component for UI elements.
//!
//! This module defines the [`checkmark`] function and related types for
//! rendering a customizable, animated checkmark, typically used to visually
//! indicate a "checked" state in controls such as checkboxes. The checkmark
//! supports smooth animation, color and stroke customization, and is
//! rendered using Tessera's GPU pipeline for high performance and visual
//! fidelity.
//!
//! # Typical Usage
//!
//! The checkmark is most commonly used within checkbox components, but can be
//! integrated into any UI element requiring a checkmark indicator. It is
//! suitable for applications needing responsive, theme-adaptable, and animated
//! visual feedback for selection or confirmation states.
//!
//! # Features
//! - Customizable color, stroke width, size, and padding
//! - Smooth animation progress control
//! - High-performance GPU rendering
//!
//! See [`CheckmarkArgs`] for configuration options and usage examples in the
//! [`checkmark`] function documentation.
use tessera_ui::{
    Color, ComputedData, Dp, LayoutPolicy, LayoutResult, MeasurementError, Px, RenderInput,
    RenderPolicy,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::pipelines::checkmark::command::CheckmarkCommand;

#[derive(Clone, PartialEq)]
struct CheckmarkLayout {
    size: Px,
    color: Color,
    stroke_width: f32,
    progress: f32,
    padding: [f32; 2],
}

impl LayoutPolicy for CheckmarkLayout {
    fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: self.size,
            height: self.size,
        }))
    }
}

impl RenderPolicy for CheckmarkLayout {
    fn record(&self, input: &RenderInput<'_>) {
        let command = CheckmarkCommand::new()
            .with_color(self.color)
            .with_stroke_width(self.stroke_width)
            .with_progress(self.progress)
            .with_padding(self.padding[0], self.padding[1]);
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(command);
    }
}

/// Renders a checkmark, a visual indicator that is displayed when in a
/// `checked` state.
///
/// This component is a GPU-rendered checkmark that provides a smooth, animated
/// alternative to traditional emoji or icon-based checkmarks. It supports
/// customization of color, stroke width, and animation progress.
///
/// # Arguments
///
/// The `args` parameter accepts any value that can be converted into
/// `CheckmarkArgs`, including a `CheckmarkArgs` struct.
///
/// * `color`: The `Color` of the checkmark stroke. Defaults to a green color.
/// * `stroke_width`: The width of the checkmark stroke in pixels. Defaults to
///   `5.0`.
/// * `progress`: The animation progress for drawing the checkmark, from `0.0`
///   (not drawn) to `1.0` (fully drawn). Defaults to `1.0`.
/// * `padding`: The padding `[horizontal, vertical]` around the checkmark
///   within its bounds. Defaults to `[2.0, 2.0]`.
/// * `size`: The size of the checkmark area as a `Dp` value. Defaults to
///   `Dp(20.0)`.
#[tessera]
pub fn checkmark(
    color: Option<Color>,
    stroke_width: Option<f32>,
    progress: Option<f32>,
    padding: Option<[f32; 2]>,
    size: Option<Dp>,
) {
    let color = color.unwrap_or(Color::new(0.0, 0.6, 0.0, 1.0));
    let stroke_width = stroke_width.unwrap_or(5.0);
    let progress = progress.unwrap_or(1.0);
    let padding = padding.unwrap_or([2.0, 2.0]);
    let size = size.unwrap_or(Dp(20.0));
    let size_px = size.to_px();
    let policy = CheckmarkLayout {
        size: Px::new(size_px.to_f32() as i32),
        color,
        stroke_width,
        progress,
        padding,
    };
    layout().layout_policy(policy.clone()).render_policy(policy);
}
