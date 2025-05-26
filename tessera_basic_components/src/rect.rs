use derive_builder::Builder;
use tessera::{BasicDrawable, ComponentNode, Constraint, DEFAULT_LAYOUT_DESC, TesseraRuntime};

/// Arguments for the `rect` component.
///
/// # Example
/// ```
/// use tessera_basic_components::rect::{RectArgs, RectArgsBuilder};
/// // a simple rectangle, in black
/// let args = RectArgsBuilder::default()
///     .color([0.0, 0.0, 0.0]) // Black
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default, Builder)]
pub struct RectArgs {
    #[builder(default = "[0.0, 0.0, 0.0]")] // Default color is black
    pub color: [f32; 3],
}

/// Basic rectangle component.
///
/// # Example
/// ```no_run
/// use tessera_basic_components::rect::{rect, RectArgs, RectArgsBuilder};
/// // a simple rectangle, in black
/// let args = RectArgsBuilder::default()
///     .color([0.0, 0.0, 0.0]) // Black
///     .build()
///     .unwrap();
/// rect(args);
/// ```
pub fn rect(args: RectArgs) {
    {
        // Add a rectangle node
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                constraint: Constraint::NONE,
                drawable: Some(BasicDrawable::Rect { color: args.color }),
            });
    }

    {
        // Pop the rectangle node from the component tree
        TesseraRuntime::write().component_tree.pop_node();
    }
}
