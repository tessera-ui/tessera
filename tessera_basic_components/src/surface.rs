use derive_builder::Builder;
use tessera::{BasicDrawable, ComputedData, Constraint, ShadowProps, measure_node, place_node};
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
    /// The padding of the surface.
    /// Default: 0.0
    #[builder(default = "0.0")]
    pub padding: f32,
}

/// Surface component, a basic container
#[tessera]
pub fn surface(args: SurfaceArgs, child: impl Fn()) {
    measure(Box::new(
        move |node_id, tree, constraint, children, metadatas| {
            let mut size = ComputedData::ZERO;
            let padding_2_f32 = args.padding * 2.0;
            let padding_2_u32 = padding_2_f32 as u32;

            let child_constraint = Constraint {
                min_width: constraint.min_width,
                min_height: constraint.min_height,
                max_width: constraint.max_width.map(|mw| (mw.saturating_sub(padding_2_u32)).max(constraint.min_width.unwrap_or(0))).or(constraint.min_width),
                max_height: constraint.max_height.map(|mh| (mh.saturating_sub(padding_2_u32)).max(constraint.min_height.unwrap_or(0))).or(constraint.min_height),
            };

            for child_node_id in children {
                let child_size = measure_node(*child_node_id, &child_constraint, tree, metadatas);
                size = size.max(child_size);
                place_node(*child_node_id, [args.padding as u32, args.padding as u32], metadatas);
            }

            // Add rect drawable
            let drawable = BasicDrawable::Rect {
                color: args.color,
                corner_radius: args.corner_radius,
                shadow: args.shadow,
            };
            metadatas.get_mut(&node_id).unwrap().basic_drawable = Some(drawable);

            ComputedData {
                width: size.width + padding_2_u32,
                height: size.height + padding_2_u32,
            }
        },
    ));

    child();
}
