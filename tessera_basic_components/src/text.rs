use derive_builder::Builder;
use tessera::{
    BasicDrawable,
    ComponentNodeMetaData,
    ComputedData,
    DimensionValue,
    Dp, // Re-added Constraint
    TextConstraint,
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
    #[builder(default = "Dp(50.0)")]
    pub size: Dp,
    #[builder(default = "Dp(50.0)")]
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
    let text_args = args.into();
    measure(Box::new(
        move |node_id, _, parent_constraint, _, metadatas| {
            let max_width: Option<f32> = match parent_constraint.width {
                DimensionValue::Fixed(w) => Some(w as f32),
                DimensionValue::Wrap => None,
                DimensionValue::Fill { max } => max.map(|m| m as f32),
            };

            let max_height: Option<f32> = match parent_constraint.height {
                DimensionValue::Fixed(h) => Some(h as f32),
                DimensionValue::Wrap => None,
                DimensionValue::Fill { max } => max.map(|m| m as f32),
            };

            let text_data = TextData::new(
                text_args.text.clone(),
                text_args.color,
                text_args.size.to_pixels_f32(),
                text_args.line_height.to_pixels_f32(),
                TextConstraint {
                    max_width,
                    max_height,
                },
            );

            let size = text_data.size;
            let drawable = BasicDrawable::Text { data: text_data };

            if let Some(mut metadata) = metadatas.get_mut(&node_id) {
                metadata.basic_drawable = Some(drawable);
            } else {
                let mut default_meta = ComponentNodeMetaData::default();
                default_meta.basic_drawable = Some(drawable);
                metadatas.insert(node_id, default_meta);
            }

            Ok(ComputedData {
                width: size[0],
                height: size[1],
            })
        },
    ));
}
