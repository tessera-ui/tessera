use derive_builder::Builder;
use tessera::{BasicDrawable, ComponentNode, Constraint, DEFAULT_LAYOUT_DESC, TesseraRuntime};

/// Arguments for the `surface` component.
#[derive(Debug, Default, Builder)]
pub struct SurfaceArgs {
    #[builder(default = "[0.4745, 0.5255, 0.7961]")] // "Soft Blue" in matiral colors
    pub color: [f32; 3],
}

/// Surface component, a basic container
pub fn surface(args: SurfaceArgs, child: impl Fn()) {
    {
        // Add a new node to the component tree
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                constraint: Constraint::NONE,
                drawable: Some(BasicDrawable::Rect { color: args.color }),
            });
    }

    child();

    {
        // Pop the node from the component tree
        TesseraRuntime::write().component_tree.pop_node();
    }
}
