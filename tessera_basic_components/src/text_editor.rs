use std::sync::Arc;

use derive_builder::Builder;
use glyphon::Edit;
use parking_lot::RwLock;

use tessera::{CursorEventContent, DimensionValue, Dp, write_font_system};
use tessera_macros::tessera;

use crate::{
    pos_misc::is_position_in_component,
    surface::{SurfaceArgsBuilder, surface},
    text_edit_core::{map_key_event_to_action, text_edit_core},
};

// Re-export TextEditorState for convenience
pub use crate::text_edit_core::TextEditorState;

/// Arguments for the `text_editor` component.
///
/// # Example
/// ```
/// use tessera_basic_components::text_editor::{TextEditorArgs, TextEditorArgsBuilder, TextEditorState};
/// use tessera::{Dp, DimensionValue};
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// // Create a text editor with a fixed width and height.
/// let editor_args_fixed = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fixed(200))) // pixels
///     .height(Some(DimensionValue::Fixed(100))) // pixels
///     .build()
///     .unwrap();
///
/// // Create a text editor that fills available width up to 500px, with a min width of 50px
/// let editor_args_fill_wrap = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fill { min: Some(50), max: Some(500) })) // pixels
///     .height(Some(DimensionValue::Wrap { min: None, max: None }))
///     .build()
///     .unwrap();
///
/// // Create the editor state
/// let editor_state = Arc::new(RwLock::new(TextEditorState::new(Dp(10.0), Dp(16.0))));
///
/// // text_editor(editor_args_fixed, editor_state.clone());
/// // text_editor(editor_args_fill_wrap, editor_state.clone());
/// ```
#[derive(Debug, Default, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TextEditorArgs {
    /// Optional width constraint for the text editor. Values are in logical pixels.
    #[builder(default = "None")]
    pub width: Option<DimensionValue>,
    /// Optional height constraint for the text editor. Values are in logical pixels.
    #[builder(default = "None")]
    pub height: Option<DimensionValue>,
    /// Minimum width in density-independent pixels. Defaults to 120dp if not specified.
    #[builder(default = "None")]
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels. Defaults to line height + padding if not specified.
    #[builder(default = "None")]
    pub min_height: Option<Dp>,
    /// Background color of the text editor (RGBA). Defaults to light gray.
    #[builder(default = "None")]
    pub background_color: Option<[f32; 4]>,
    /// Border width in pixels. Defaults to 1.0.
    #[builder(default = "1.0")]
    pub border_width: f32,
    /// Border color (RGBA). Defaults to gray.
    #[builder(default = "None")]
    pub border_color: Option<[f32; 4]>,
    /// Corner radius in pixels. Defaults to 4.0.
    #[builder(default = "4.0")]
    pub corner_radius: f32,
    /// Padding inside the text editor. Defaults to 5.0.
    #[builder(default = "Dp(5.0)")]
    pub padding: Dp,
    /// Border color when focused (RGBA). Defaults to blue.
    #[builder(default = "None")]
    pub focus_border_color: Option<[f32; 4]>,
    /// Background color when focused (RGBA). Defaults to white.
    #[builder(default = "None")]
    pub focus_background_color: Option<[f32; 4]>,
}

/// A text editor component with two-layer architecture:
/// - Surface layer: provides visual container, minimum size, and click area
/// - Core layer: handles text rendering and editing logic
///
/// This design solves the issue where empty text editors had zero width and couldn't be clicked.
///
/// # Example
///
/// ```
/// use tessera_basic_components::text_editor::{text_editor, TextEditorArgs, TextEditorArgsBuilder, TextEditorState};
/// use tessera::{Dp, DimensionValue};
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// let args = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fixed(300)))
///     .height(Some(DimensionValue::Fill { min: Some(50), max: Some(500) }))
///     .build()
///     .unwrap();
///
/// let state = Arc::new(RwLock::new(TextEditorState::new(Dp(12.0), Dp(18.0))));
/// // text_editor(args, state);
/// ```
#[tessera]
pub fn text_editor(args: impl Into<TextEditorArgs>, state: Arc<RwLock<TextEditorState>>) {
    let editor_args: TextEditorArgs = args.into();

    // Surface layer - provides visual container and minimum size guarantee
    {
        let state_for_surface = state.clone();
        let args_for_surface = editor_args.clone();
        surface(
            create_surface_args(&args_for_surface, &state_for_surface),
            move || {
                // Core layer - handles text rendering and editing logic
                text_edit_core(state_for_surface.clone());
            },
        );
    }

    // Event handling at the outermost layer - can access full surface area
    {
        let state_for_handler = state.clone();
        state_handler(Box::new(move |input| {
            let size = input.computed_data; // This is the full surface size
            let cursor_position = input.cursor_position;
            let is_cursor_in_editor = cursor_position
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            // Handle click events - now we have a full clickable area from surface
            if is_cursor_in_editor {
                let has_click = input
                    .cursor_events
                    .iter()
                    .any(|event| matches!(event.content, CursorEventContent::Pressed(_)));

                if has_click && !state_for_handler.read().focus_handler().is_focused() {
                    state_for_handler
                        .write()
                        .focus_handler_mut()
                        .request_focus();
                }
            }

            // Handle keyboard events (only when focused)
            if state_for_handler.read().focus_handler().is_focused() {
                let actions = input
                    .keyboard_events
                    .iter()
                    .cloned()
                    .filter_map(map_key_event_to_action)
                    .flatten();
                for action in actions {
                    state_for_handler
                        .write()
                        .editor
                        .action(&mut write_font_system(), action);
                }
            }
        }));
    }
}

/// Create surface arguments based on editor configuration and state
fn create_surface_args(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> crate::surface::SurfaceArgs {
    let mut builder = SurfaceArgsBuilder::default();

    // Set width if available
    if let Some(width) = args.width {
        builder = builder.width(width);
    } else {
        // Use default with minimum
        builder = builder.width(DimensionValue::Wrap {
            min: args.min_width.map(|dp| dp.to_pixels_u32()).or(Some(120)), // Default minimum width 120dp
            max: None,
        });
    }

    // Set height if available
    if let Some(height) = args.height {
        builder = builder.height(height);
    } else {
        // Use line height as basis with some padding
        let line_height = state.read().line_height();
        let padding = args.padding.to_pixels_u32() * 2; // top + bottom padding
        let min_height = args
            .min_height
            .map(|dp| dp.to_pixels_u32())
            .unwrap_or(line_height + padding + 10); // +10 for comfortable spacing
        builder = builder.height(DimensionValue::Wrap {
            min: Some(min_height),
            max: None,
        });
    }

    builder
        .color(determine_background_color(args, state))
        .border_width(determine_border_width(args, state))
        .border_color(determine_border_color(args, state))
        .corner_radius(args.corner_radius)
        .padding(args.padding)
        .build()
        .unwrap()
}

/// Determine background color based on focus state
fn determine_background_color(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> [f32; 4] {
    if state.read().focus_handler().is_focused() {
        args.focus_background_color
            .or(args.background_color)
            .unwrap_or([1.0, 1.0, 1.0, 1.0]) // Default white when focused
    } else {
        args.background_color.unwrap_or([0.95, 0.95, 0.95, 1.0]) // Default light gray when not focused
    }
}

/// Determine border width
fn determine_border_width(args: &TextEditorArgs, _state: &Arc<RwLock<TextEditorState>>) -> f32 {
    args.border_width
}

/// Determine border color based on focus state
fn determine_border_color(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> Option<[f32; 4]> {
    if state.read().focus_handler().is_focused() {
        args.focus_border_color
            .or(args.border_color)
            .or(Some([0.0, 0.5, 1.0, 1.0])) // Default blue focus border
    } else {
        args.border_color.or(Some([0.7, 0.7, 0.7, 1.0])) // Default gray border
    }
}

/// Convenience constructors for common use cases
impl TextEditorArgs {
    /// Create a simple text editor with default styling
    pub fn simple() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some([1.0, 1.0, 1.0, 1.0]))
            .border_width(1.0)
            .border_color(Some([0.7, 0.7, 0.7, 1.0]))
            .corner_radius(4.0)
            .build()
            .unwrap()
    }

    /// Create a text editor with emphasized border for better visibility
    pub fn outlined() -> Self {
        Self::simple()
            .with_border_width(2.0)
            .with_focus_border_color([0.0, 0.5, 1.0, 1.0])
    }

    /// Create a text editor with no border (minimal style)
    pub fn minimal() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some([1.0, 1.0, 1.0, 1.0]))
            .border_width(0.0)
            .corner_radius(0.0)
            .build()
            .unwrap()
    }
}

/// Builder methods for fluent API
impl TextEditorArgs {
    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_min_width(mut self, min_width: Dp) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn with_min_height(mut self, min_height: Dp) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn with_background_color(mut self, color: [f32; 4]) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    pub fn with_border_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_padding(mut self, padding: Dp) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_focus_border_color(mut self, color: [f32; 4]) -> Self {
        self.focus_border_color = Some(color);
        self
    }

    pub fn with_focus_background_color(mut self, color: [f32; 4]) -> Self {
        self.focus_background_color = Some(color);
        self
    }
}
