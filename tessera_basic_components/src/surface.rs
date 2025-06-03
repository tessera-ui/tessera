use derive_builder::Builder;
use tessera::{
    BasicDrawable,
    ComputedData,
    Constraint,
    DimensionValue,
    ShadowProps, // Removed NodeId from here
    measure_node,
    place_node, // Removed ComponentNodeMetaDatas
};
use tessera_macros::tessera;

/// Arguments for the `surface` component.
#[derive(Debug, Default, Builder, Clone)] // Added Clone
#[builder(pattern = "owned")]
pub struct SurfaceArgs {
    /// The color of the surface.
    #[builder(default = "[0.4745, 0.5255, 0.7961]")]
    pub color: [f32; 3],
    /// The corner radius of the surface.
    #[builder(default = "0.0")]
    pub corner_radius: f32,
    /// The shadow properties of the surface.
    #[builder(default)]
    pub shadow: Option<ShadowProps>,
    /// The padding of the surface.
    #[builder(default = "0.0")]
    pub padding: f32,
    /// Optional explicit width behavior for the surface. Defaults to Wrap if None.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the surface. Defaults to Wrap if None.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
}

/// Surface component, a basic container that can have its own size constraints.
#[tessera]
pub fn surface(args: SurfaceArgs, child: impl Fn()) {
    // Clone args for the measure closure, as it's FnBox and might be called multiple times or sent across threads.
    let measure_args = args.clone();

    measure(Box::new(
        move |node_id, tree, parent_constraint, children_node_ids, metadatas| {
            let padding_2_f32 = measure_args.padding * 2.0;
            let padding_2_u32 = padding_2_f32 as u32;

            // 1. Determine Surface's intrinsic constraint based on args
            let surface_intrinsic_width = measure_args.width.unwrap_or(DimensionValue::Wrap);
            let surface_intrinsic_height = measure_args.height.unwrap_or(DimensionValue::Wrap);
            let surface_intrinsic_constraint =
                Constraint::new(surface_intrinsic_width, surface_intrinsic_height);

            // 2. Merge with parent_constraint to get effective_surface_constraint
            let effective_surface_constraint =
                surface_intrinsic_constraint.merge(&parent_constraint);

            // 3. Determine constraint for the child
            let child_constraint_width = match effective_surface_constraint.width {
                DimensionValue::Fixed(sw) => {
                    DimensionValue::Fixed(sw.saturating_sub(padding_2_u32))
                }
                DimensionValue::Wrap => DimensionValue::Wrap, // Child wraps, padding added later
                DimensionValue::Fill { max: s_max_w } => DimensionValue::Fill {
                    max: s_max_w.map(|m| m.saturating_sub(padding_2_u32)),
                },
            };
            let child_constraint_height = match effective_surface_constraint.height {
                DimensionValue::Fixed(sh) => {
                    DimensionValue::Fixed(sh.saturating_sub(padding_2_u32))
                }
                DimensionValue::Wrap => DimensionValue::Wrap, // Child wraps, padding added later
                DimensionValue::Fill { max: s_max_h } => DimensionValue::Fill {
                    max: s_max_h.map(|m| m.saturating_sub(padding_2_u32)),
                },
            };
            let child_actual_constraint =
                Constraint::new(child_constraint_width, child_constraint_height);

            // 4. Measure the child (assuming single child from `child: impl Fn()`)
            let mut child_measured_size = ComputedData::ZERO;
            if let Some(&child_node_id) = children_node_ids.first() {
                // The child's own defined constraints also need to be merged.
                let final_child_constraint_for_measure = metadatas[&child_node_id]
                    .constraint
                    .merge(&child_actual_constraint);
                child_measured_size = measure_node(
                    child_node_id,
                    &final_child_constraint_for_measure,
                    tree,
                    metadatas,
                );

                // Place the child
                place_node(
                    child_node_id,
                    [measure_args.padding as u32, measure_args.padding as u32],
                    metadatas,
                );
            }

            // 5. Calculate final Surface dimensions based on effective_surface_constraint and child's size
            let content_width_with_padding =
                child_measured_size.width.saturating_add(padding_2_u32);
            let content_height_with_padding =
                child_measured_size.height.saturating_add(padding_2_u32);

            let final_surface_width = match effective_surface_constraint.width {
                DimensionValue::Fixed(sw) => sw,
                DimensionValue::Wrap => content_width_with_padding,
                DimensionValue::Fill { max: Some(s_max_w) } => s_max_w, // Surface fills up to this max
                DimensionValue::Fill { max: None } => content_width_with_padding, // Behaves like Wrap if parent didn't give fixed size
            };

            let final_surface_height = match effective_surface_constraint.height {
                DimensionValue::Fixed(sh) => sh,
                DimensionValue::Wrap => content_height_with_padding,
                DimensionValue::Fill { max: Some(s_max_h) } => s_max_h, // Surface fills up to this max
                DimensionValue::Fill { max: None } => content_height_with_padding, // Behaves like Wrap if parent didn't give fixed size
            };

            // Add rect drawable
            let drawable = BasicDrawable::Rect {
                color: measure_args.color,
                corner_radius: measure_args.corner_radius,
                shadow: measure_args.shadow,
            };
            if let Some(metadata) = metadatas.get_mut(&node_id) {
                metadata.basic_drawable = Some(drawable);
            }

            ComputedData {
                width: final_surface_width,
                height: final_surface_height,
            }
        },
    ));

    child();
}
