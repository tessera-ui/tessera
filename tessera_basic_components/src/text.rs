use derive_builder::Builder;
use tessera::{
    BasicDrawable, ComponentNode, Constraint, DEFAULT_LAYOUT_DESC, TesseraRuntime, TextConstraint,
    TextData,
};

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
pub fn text(args: TextArgs) {
    {
        // Add a text node
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                constraint: Constraint::NONE,
                drawable: Some(BasicDrawable::Text {
                    data: TextData::new(
                        args.text,
                        args.color,
                        args.size,
                        args.line_height,
                        TextConstraint {
                            max_width: None,
                            max_height: None,
                        },
                    ),
                }),
            });
    }

    {
        TesseraRuntime::write().component_tree.pop_node();
    }
}
