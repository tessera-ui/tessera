use derive_builder::Builder;
use tessera::{
    BasicDrawable, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, PxPosition,
    ShadowProps, measure_nodes, place_node,
};
use tessera_macros::tessera;

/// Arguments for the `surface` component.
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SurfaceArgs {
    /// The fill color of the surface (RGBA).
    #[builder(default = "[0.4745, 0.5255, 0.7961, 1.0]")]
    pub color: [f32; 4],
    /// The corner radius of the surface.
    #[builder(default = "0.0")]
    pub corner_radius: f32,
    /// The shadow properties of the surface.
    #[builder(default)]
    pub shadow: Option<ShadowProps>,
    /// The padding of the surface.
    #[builder(default = "Dp(0.0)")]
    pub padding: Dp,
    /// Optional explicit width behavior for the surface. Defaults to Wrap {min: None, max: None} if None.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the surface. Defaults to Wrap {min: None, max: None} if None.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
    /// Width of the border. If > 0, an outline will be drawn.
    #[builder(default = "0.0")]
    pub border_width: f32,
    /// Optional color for the border (RGBA). If None and border_width > 0, `color` will be used.
    #[builder(default)]
    pub border_color: Option<[f32; 4]>,
}

// Manual implementation of Default because derive_builder's default conflicts with our specific defaults
impl Default for SurfaceArgs {
    fn default() -> Self {
        SurfaceArgsBuilder::default().build().unwrap()
    }
}

/// Surface component, a basic container that can have its own size constraints.
#[tessera]
pub fn surface(args: SurfaceArgs, child: impl FnOnce()) {
    let measure_args = args.clone();

    measure(Box::new(
        move |node_id, tree, parent_constraint, children_node_ids, metadatas| {
            let padding_val = measure_args.padding.to_pixels_f32();
            let padding_2_f32 = padding_val * 2.0;
            let padding_2_u32 = padding_2_f32 as u32;

            // 1. Determine Surface's intrinsic constraint based on args
            let surface_intrinsic_width = measure_args.width.unwrap_or(DimensionValue::Wrap {
                min: None,
                max: None,
            });
            let surface_intrinsic_height = measure_args.height.unwrap_or(DimensionValue::Wrap {
                min: None,
                max: None,
            });
            let surface_intrinsic_constraint =
                Constraint::new(surface_intrinsic_width, surface_intrinsic_height);

            // 2. Merge with parent_constraint to get effective_surface_constraint
            let effective_surface_constraint =
                surface_intrinsic_constraint.merge(parent_constraint);

            // 3. Determine constraint for the child
            // For Fill constraint, Surface should determine its own final size first, then give child a Fixed constraint
            let child_constraint_width = match effective_surface_constraint.width {
                DimensionValue::Fixed(sw) => {
                    DimensionValue::Fixed(sw.saturating_sub(padding_2_u32))
                }
                DimensionValue::Wrap {
                    min: s_min_w,
                    max: s_max_w,
                } => DimensionValue::Wrap {
                    min: s_min_w.map(|m| m.saturating_sub(padding_2_u32)),
                    max: s_max_w.map(|m| m.saturating_sub(padding_2_u32)),
                },
                DimensionValue::Fill {
                    min: _s_min_w,
                    max: s_max_w,
                } => {
                    // For Fill, Surface should use parent's provided width and give child a Fixed constraint
                    let parent_provided_width = match parent_constraint.width {
                        DimensionValue::Fixed(pw) => Some(pw),
                        DimensionValue::Fill {
                            max: p_max_fill, ..
                        } => p_max_fill,
                        _ => None,
                    };

                    if let Some(ppw) = parent_provided_width {
                        // Surface takes the full parent-provided width, child gets fixed constraint
                        DimensionValue::Fixed(ppw.saturating_sub(padding_2_u32))
                    } else {
                        // No parent width available, fallback to wrap-like behavior
                        DimensionValue::Wrap {
                            min: None,
                            max: s_max_w.map(|m| m.saturating_sub(padding_2_u32)),
                        }
                    }
                }
            };
            let child_constraint_height = match effective_surface_constraint.height {
                DimensionValue::Fixed(sh) => {
                    DimensionValue::Fixed(sh.saturating_sub(padding_2_u32))
                }
                DimensionValue::Wrap {
                    min: s_min_h,
                    max: s_max_h,
                } => DimensionValue::Wrap {
                    min: s_min_h.map(|m| m.saturating_sub(padding_2_u32)),
                    max: s_max_h.map(|m| m.saturating_sub(padding_2_u32)),
                },
                DimensionValue::Fill {
                    min: _s_min_h,
                    max: s_max_h,
                } => {
                    // For Fill, Surface should use parent's provided height and give child a Fixed constraint
                    let parent_provided_height = match parent_constraint.height {
                        DimensionValue::Fixed(ph) => Some(ph),
                        DimensionValue::Fill {
                            max: p_max_fill, ..
                        } => p_max_fill,
                        _ => None,
                    };

                    if let Some(pph) = parent_provided_height {
                        // Surface takes the full parent-provided height, child gets fixed constraint
                        DimensionValue::Fixed(pph.saturating_sub(padding_2_u32))
                    } else {
                        // No parent height available, fallback to wrap-like behavior
                        DimensionValue::Wrap {
                            min: None,
                            max: s_max_h.map(|m| m.saturating_sub(padding_2_u32)),
                        }
                    }
                }
            };
            let child_actual_constraint =
                Constraint::new(child_constraint_width, child_constraint_height);

            // 4. Measure the child
            let mut child_measured_size = ComputedData::ZERO;
            if let Some(&child_node_id) = children_node_ids.first() {
                let child_intrinsic_constraint = metadatas
                    .get(&child_node_id)
                    .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                    .constraint;
                let final_child_constraint_for_measure =
                    child_intrinsic_constraint.merge(&child_actual_constraint);

                let nodes_to_measure = vec![(child_node_id, final_child_constraint_for_measure)];
                let results_map = measure_nodes(nodes_to_measure, tree, metadatas);

                child_measured_size = results_map
                    .get(&child_node_id)
                    .ok_or_else(|| {
                        MeasurementError::MeasureFnFailed(format!(
                            "Child {child_node_id:?} result missing in map"
                        ))
                    })?
                    .clone()?;

                place_node(
                    child_node_id,
                    PxPosition::new(measure_args.padding.into(), measure_args.padding.into()),
                    metadatas,
                );
            }

            // 5. Calculate final Surface dimensions
            let content_width_with_padding =
                child_measured_size.width.saturating_add(padding_2_u32);
            let content_height_with_padding =
                child_measured_size.height.saturating_add(padding_2_u32);

            let mut final_surface_width = content_width_with_padding;
            match effective_surface_constraint.width {
                DimensionValue::Fixed(sw) => final_surface_width = sw,
                DimensionValue::Wrap { min, max } => {
                    if let Some(min_w) = min {
                        final_surface_width = final_surface_width.max(min_w);
                    }
                    if let Some(max_w) = max {
                        final_surface_width = final_surface_width.min(max_w);
                    }
                }
                DimensionValue::Fill { min, max } => {
                    // For Fill constraint, use the max value from Surface's constraint (which comes from parent)
                    if let Some(max_w) = max {
                        final_surface_width = max_w; // Fill should use the provided max constraint
                    } else {
                        // When no max constraint provided, wrap content (like a Wrap behavior)
                        final_surface_width = content_width_with_padding;
                    }
                    if let Some(min_w) = min {
                        final_surface_width = final_surface_width.max(min_w);
                    }
                }
            };

            let mut final_surface_height = content_height_with_padding;
            match effective_surface_constraint.height {
                DimensionValue::Fixed(sh) => final_surface_height = sh,
                DimensionValue::Wrap { min, max } => {
                    if let Some(min_h) = min {
                        final_surface_height = final_surface_height.max(min_h);
                    }
                    if let Some(max_h) = max {
                        final_surface_height = final_surface_height.min(max_h);
                    }
                }
                DimensionValue::Fill { min, max } => {
                    // For Fill constraint, use the max value from Surface's constraint (which comes from parent)
                    if let Some(max_h) = max {
                        final_surface_height = max_h; // Fill should use the provided max constraint
                    } else {
                        // When no max constraint provided, wrap content (like a Wrap behavior)
                        final_surface_height = content_height_with_padding;
                    }
                    if let Some(min_h) = min {
                        final_surface_height = final_surface_height.max(min_h);
                    }
                }
            };

            let drawable = if measure_args.border_width > 0.0 {
                BasicDrawable::OutlinedRect {
                    color: measure_args.border_color.unwrap_or(measure_args.color),
                    corner_radius: measure_args.corner_radius,
                    shadow: measure_args.shadow,
                    border_width: measure_args.border_width,
                }
            } else {
                BasicDrawable::Rect {
                    color: measure_args.color,
                    corner_radius: measure_args.corner_radius,
                    shadow: measure_args.shadow,
                }
            };

            if let Some(mut metadata) = metadatas.get_mut(&node_id) {
                metadata.basic_drawable = Some(drawable);
            }

            Ok(ComputedData {
                width: final_surface_width,
                height: final_surface_height,
            })
        },
    ));

    child();
}
