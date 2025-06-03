use derive_builder::Builder;
use tessera::{BasicDrawable, ComputedData, ShadowProps, measure_node, place_node};
use tessera_macros::tessera;

/// Arguments for the `surface` component.
#[derive(Debug, Default, Builder)]
pub struct SurfaceArgs {
    /// The color of the surface.
    /// Default: "Soft Blue"(#7986CB) in matiral colors
    #[builder(default = "[0.4745, 0.5255, 0.7961]")]
    pub color: [f32; 3],
    /// The corner radius of the surface.
    /// Default: 0.0, which means no rounded corners.
    #[builder(default = "0.0")]
    pub corner_radius: f32,
    /// The shadow properties of the surface.
    /// Default: None, which means no shadow.
    #[builder(default)]
    pub shadow: Option<ShadowProps>,
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
            let drawable = BasicDrawable::Rect {
                color: args.color,
                corner_radius: args.corner_radius,
                shadow: args.shadow,
            };
            metadatas.get_mut(&node_id).unwrap().basic_drawable = Some(drawable);
            size
        },
    ));

    child();
}
