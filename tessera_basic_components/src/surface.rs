use derive_builder::Builder;
use tessera::{BasicDrawable, ComputedData, measure_node, place_node};
use tessera_macros::tessera;

/// Arguments for the `surface` component.
#[derive(Debug, Default, Builder)]
pub struct SurfaceArgs {
    #[builder(default = "[0.4745, 0.5255, 0.7961]")] // "Soft Blue" in matiral colors
    pub color: [f32; 3],
}

/// Surface component, a basic container
#[tessera]
pub fn surface(args: SurfaceArgs, child: impl Fn()) {
    measure(Box::new(
        move |node_id, tree, constraint, children, metadatas| {
            let mut size = ComputedData::ZERO;
            for child in children {
                let child_size = measure_node(*child, constraint, tree, metadatas);
                size = size.max(child_size);
                place_node(*child, [0, 0], metadatas);
            }
            // Add rect drawable
            let drawable = BasicDrawable::Rect { color: args.color };
            metadatas.get_mut(&node_id).unwrap().basic_drawable = Some(drawable);
            size
        },
    ));

    child();
}
