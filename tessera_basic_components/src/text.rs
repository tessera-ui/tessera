use derive_builder::Builder;
use tessera::{BasicDrawable, ComputedData, TextConstraint, TextData};
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
#[derive(Debug, Default, Builder)]
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
    let args = args.into();
    measure(Box::new(move |node_id, _, constraint, _, metadatas| {
        // Create a new text node with the given arguments
        let mut text_data = TextData::new(
            args.text.clone(),
            args.color,
            args.size,
            args.line_height,
            TextConstraint {
                max_width: None,
                max_height: None,
            },
        );
        // resize text data based on the constraint
        text_data.resize(
            constraint.max_width.map(|width| width as f32),
            constraint.max_height.map(|height| height as f32),
        );
        // save it's actual size
        let size = text_data.size;
        // Add to drawable
        let drawable = BasicDrawable::Text { data: text_data };
        metadatas.get_mut(&node_id).unwrap().basic_drawable = Some(drawable);
        ComputedData {
            width: size[0],
            height: size[1],
        }
    }));
}
