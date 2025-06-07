use derive_builder::Builder;
use tessera::{
    BasicDrawable, ComponentNodeMetaData, ComputedData, DimensionValue, TextConstraint, TextData,
}; // Re-added Constraint
use tessera_macros::tessera;

/// Arguments for the `text` component.
///
/// # Example
/// ```
/// use tessera_basic_components::text::{TextArgs, TextArgsBuilder};
/// // a simple hello world text, in black
/// let args = TextArgsBuilder::default()
///     .text("Hello, World!".to_string())
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TextArgs {
    pub text: String,
    #[builder(default = "[0, 0, 0]")] // Default color is black
    pub color: [u8; 3],
    #[builder(default = "50.0")]
    pub size: f32,
    #[builder(default = "50.0")]
    pub line_height: f32,
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
/// // a simple hello world text, in black
/// let args = TextArgsBuilder::default()
///     .text("Hello, World!".to_string())
///     .build()
///     .unwrap();
/// text(args);
/// ```
#[tessera]
pub fn text(args: impl Into<TextArgs>) {
    let text_args = args.into();
    measure(Box::new(
        move |node_id, _, parent_constraint, _, metadatas| {
            let max_width_for_resize: Option<f32> = match parent_constraint.width {
                DimensionValue::Fixed(w) => Some(w as f32),
                DimensionValue::Wrap => None,
                DimensionValue::Fill { max } => max.map(|m| m as f32),
            };

            let max_height_for_resize: Option<f32> = match parent_constraint.height {
                DimensionValue::Fixed(h) => Some(h as f32),
                DimensionValue::Wrap => None,
                DimensionValue::Fill { max } => max.map(|m| m as f32),
            };

            let mut text_data = TextData::new(
                text_args.text.clone(),
                text_args.color,
                text_args.size,
                text_args.line_height,
                TextConstraint {
                    max_width: None,
                    max_height: None,
                },
            );
            text_data.resize(max_width_for_resize, max_height_for_resize);

            let size = text_data.size;
            let drawable = BasicDrawable::Text { data: text_data };

            if let Some(mut metadata) = metadatas.get_mut(&node_id) {
                metadata.basic_drawable = Some(drawable);
            } else {
                let mut default_meta = ComponentNodeMetaData::default();
                default_meta.basic_drawable = Some(drawable);
                metadatas.insert(node_id, default_meta);
            }

            ComputedData {
                width: size[0],
                height: size[1],
            }
        },
    ));
}
