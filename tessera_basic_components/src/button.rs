use std::sync::Arc;
use derive_builder::Builder;
use tessera::{
    CursorEventContent, DimensionValue, Dp, PressKeyEventType
};
use tessera_macros::tessera;

use crate::{
    pos_misc::is_position_in_component,
    surface::{SurfaceArgsBuilder, surface},
    text::{text, TextArgsBuilder},
};

/// Arguments for the `button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonArgs {
    /// The text to display on the button.
    pub text: String,
    /// The fill color of the button (RGBA).
    #[builder(default = "[0.2, 0.5, 0.8, 1.0]")]
    pub color: [f32; 4],
    /// The text color (RGB).
    #[builder(default = "[255, 255, 255]")]
    pub text_color: [u8; 3],
    /// The corner radius of the button.
    #[builder(default = "8.0")]
    pub corner_radius: f32,
    /// The padding of the button.
    #[builder(default = "Dp(12.0)")]
    pub padding: Dp,
    /// The text size.
    #[builder(default = "Dp(16.0)")]
    pub text_size: Dp,
    /// The text line height.
    #[builder(default = "Dp(20.0)")]
    pub text_line_height: Dp,
    /// Optional explicit width behavior for the button.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the button.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
    /// The click callback function
    pub on_click: Arc<dyn Fn() + Send + Sync>,
}

impl std::fmt::Debug for ButtonArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonArgs")
            .field("text", &self.text)
            .field("color", &self.color)
            .field("text_color", &self.text_color)
            .field("corner_radius", &self.corner_radius)
            .field("padding", &self.padding)
            .field("text_size", &self.text_size)
            .field("text_line_height", &self.text_line_height)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("on_click", &"<callback>")
            .finish()
    }
}

impl Default for ButtonArgs {
    fn default() -> Self {
        ButtonArgsBuilder::default()
            .text("Button".to_string())
            .on_click(Arc::new(|| {}))
            .build()
            .unwrap()
    }
}

/// Interactive button component that can display text and handle click events.
/// 
/// # Example
/// ```no_run
/// use tessera_basic_components::button::{button, ButtonArgsBuilder};
/// use tessera::Dp;
/// use std::sync::Arc;
/// 
/// let args = ButtonArgsBuilder::default()
///     .text("Click me!".to_string())
///     .color([0.1, 0.7, 0.3, 1.0]) // Green button
///     .padding(Dp(16.0))
///     .on_click(Arc::new(|| println!("Button clicked!")))
///     .build()
///     .unwrap();
/// 
/// button(args);
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>) {
    let button_args: ButtonArgs = args.into();
    
    // Create surface for button background and container
    {
        let args_for_surface = button_args.clone();
        surface(
            create_surface_args(&args_for_surface),
            move || {
                // Text content inside the button
                text(create_text_args(&args_for_surface));
            },
        );
    }

    // Event handling for button interactions
    {
        let args_for_handler = button_args.clone();
        state_handler(Box::new(move |input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position;
            let is_cursor_in_button = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            // Handle mouse events
            if is_cursor_in_button {
                // Check for mouse release events (click)
                let release_events: Vec<_> = input
                    .cursor_events
                    .iter()
                    .filter(|event| matches!(event.content, CursorEventContent::Released(PressKeyEventType::Left)))
                    .collect();

                if !release_events.is_empty() {
                    // Trigger click callback
                    (args_for_handler.on_click)();
                }

                // Consume cursor events to prevent propagation
                input.cursor_events.clear();
            }
        }));
    }
}

/// Create surface arguments based on button configuration
fn create_surface_args(args: &ButtonArgs) -> crate::surface::SurfaceArgs {
    let mut builder = SurfaceArgsBuilder::default();

    // Set width if available
    if let Some(width) = args.width {
        builder = builder.width(width);
    }

    // Set height if available  
    if let Some(height) = args.height {
        builder = builder.height(height);
    }

    builder
        .color(args.color)
        .corner_radius(args.corner_radius)
        .padding(args.padding)
        .build()
        .unwrap()
}

/// Create text arguments for the button label
fn create_text_args(args: &ButtonArgs) -> crate::text::TextArgs {
    TextArgsBuilder::default()
        .text(args.text.clone())
        .color(args.text_color)
        .size(args.text_size)
        .line_height(args.text_line_height)
        .build()
        .unwrap()
}

/// Convenience constructors for common button styles
impl ButtonArgs {
    /// Create a primary button with default blue styling
    pub fn primary(text: String, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .text(text)
            .color([0.2, 0.5, 0.8, 1.0]) // Blue
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a secondary button with gray styling
    pub fn secondary(text: String, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .text(text)
            .color([0.6, 0.6, 0.6, 1.0]) // Gray
            .text_color([0, 0, 0]) // Black text
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a success button with green styling
    pub fn success(text: String, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .text(text)
            .color([0.1, 0.7, 0.3, 1.0]) // Green
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a danger button with red styling
    pub fn danger(text: String, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .text(text)
            .color([0.8, 0.2, 0.2, 1.0]) // Red
            .on_click(on_click)
            .build()
            .unwrap()
    }
}

/// Builder methods for fluent API
impl ButtonArgs {
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_text_color(mut self, text_color: [u8; 3]) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn with_padding(mut self, padding: Dp) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_corner_radius(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn with_text_size(mut self, text_size: Dp) -> Self {
        self.text_size = text_size;
        self
    }

    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = Some(height);
        self
    }
}