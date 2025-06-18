use derive_builder::Builder;
use tessera::{
    BasicDrawable, ComponentNodeMetaData, ComputedData, DimensionValue, Dp, Px, TextConstraint,
    TextData,
};
use tessera_macros::tessera;

/// Arguments for the `text` component.
///
/// # Example
/// ```
/// use tessera_basic_components::text::{TextArgs, TextArgsBuilder};
/// use tessera::Dp;
/// // a simple hello world text, in black
/// let args = TextArgsBuilder::default()
///     .text("Hello, World!".to_string())
///     .size(Dp(50.0)) // Example using Dp
///     .line_height(Dp(50.0)) // Example using Dp
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TextArgs {
    pub text: String,
    #[builder(default = "[0, 0, 0]")] // Default color is black
    pub color: [u8; 3],
    #[builder(default = "Dp(25.0)")]
    pub size: Dp,
    #[builder(default = "Dp(25.0)")]
    pub line_height: Dp,
}

impl From<String> for TextArgs {
    fn from(val: String) -> Self {
        TextArgsBuilder::default().text(val).build().unwrap()
    }
}

impl From<&str> for TextArgs {
    fn from(val: &str) -> Self {
        TextArgsBuilder::default()
            .text(val.to_string())
            .build()
            .unwrap()
    }
}

/// Basic text component.
///
/// # Example
/// ```no_run
/// use tessera_basic_components::text::{text, TextArgs, TextArgsBuilder};
/// use tessera::Dp;
/// // a simple hello world text, in black
/// let args = TextArgsBuilder::default()
///     .text("Hello, World!".to_string())
///     .size(Dp(50.0)) // Example using Dp
///     .line_height(Dp(50.0)) // Example using Dp
///     .build()
///     .unwrap();
/// text(args);
/// ```
#[tessera]
pub fn text(args: impl Into<TextArgs>) {
    let text_args: TextArgs = args.into();
    measure(Box::new(move |input| {
        let max_width: Option<Px> = match input.parent_constraint.width {
            DimensionValue::Fixed(w) => Some(w),
            DimensionValue::Wrap { max, .. } => max, // Use max from Wrap
            DimensionValue::Fill { max, .. } => max, // Use max from Fill
        };

        let max_height: Option<Px> = match input.parent_constraint.height {
            DimensionValue::Fixed(h) => Some(h),
            DimensionValue::Wrap { max, .. } => max, // Use max from Wrap
            DimensionValue::Fill { max, .. } => max, // Use max from Fill
        };

        let text_data = TextData::new(
            text_args.text.clone(),
            text_args.color,
            text_args.size.to_pixels_f32(),
            text_args.line_height.to_pixels_f32(),
            TextConstraint {
                max_width: max_width.map(|px| px.to_f32()),
                max_height: max_height.map(|px| px.to_f32()),
            },
        );

        let size = text_data.size;
        let drawable = BasicDrawable::Text { data: text_data };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(drawable);
        } else {
            // This branch might be less common if metadatas are pre-populated or entry().or_default() is used.
            // However, keeping it for safety if a node_id somehow exists without prior metadata entry.
            let default_meta = ComponentNodeMetaData {
                basic_drawable: Some(drawable),
                ..Default::default()
            };
            input.metadatas.insert(input.current_node_id, default_meta);
        }

        Ok(ComputedData {
            width: size[0].into(),
            height: size[1].into(),
        })
    }));
}
