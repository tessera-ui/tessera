use tessera_ui::{
    Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, Px,
    PxPosition, PxSize, modifier::ModifierCapabilityExt as _,
};

use crate::{
    pipelines::shadow::atlas::ShadowAtlasCommand,
    shadow::{ShadowLayers, material_shadow},
    shape_def::{ResolvedShape, Shape},
    theme::MaterialColorScheme,
};

use super::ModifierExt;

/// Arguments for the `shadow` modifier.
#[derive(Clone, Debug)]
pub struct ShadowArgs {
    /// Material elevation, interpolated between standard levels and clamped
    /// to 0–12dp.
    pub elevation: Dp,
    /// The shape of the shadow.
    pub shape: Shape,
    /// Whether to clip the content to the shape.
    pub clip: bool,
    /// Color of the ambient shadow. If `None`, uses the theme default.
    pub ambient_color: Option<Color>,
    /// Color of the spot shadow. If `None`, uses the theme default.
    pub spot_color: Option<Color>,
}

impl Default for ShadowArgs {
    fn default() -> Self {
        Self {
            elevation: Dp(0.0),
            shape: Shape::RECTANGLE,
            clip: false,
            ambient_color: None,
            spot_color: None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ShadowModifierNode {
    pub shadow: ShadowLayers,
    pub shape: Shape,
    pub ambient_spread: f32,
}

impl DrawModifierNode for ShadowModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        {
            let mut metadata = ctx.render_input.metadata_mut();
            let Some(size) = metadata.computed_data() else {
                return;
            };
            let size = PxSize::from(size);
            if size.width.0 > 0 && size.height.0 > 0 {
                record_md3_shadow(
                    metadata.fragment_mut(),
                    self.shadow,
                    self.shape.resolve_for_size(size),
                    size,
                    self.ambient_spread,
                );
            }
        }
        content.draw(ctx.render_input);
    }
}

pub(super) fn apply_shadow_modifier(
    base: Modifier,
    scheme: &MaterialColorScheme,
    args: ShadowArgs,
) -> Modifier {
    let (mut layers, ambient_spread) = material_shadow(args.elevation, scheme);

    if let Some(ambient) = args.ambient_color
        && let Some(ref mut layer) = layers.ambient
    {
        layer.color = ambient;
    }
    if let Some(spot) = args.spot_color
        && let Some(ref mut layer) = layers.spot
    {
        layer.color = spot;
    }

    let mut modifier = base.push_draw(ShadowModifierNode {
        shadow: layers,
        shape: args.shape,
        ambient_spread,
    });

    if args.clip {
        // Very basic clipping support (rect only for now as per existing
        // modifier)
        modifier = modifier.clip_to_bounds();
    }

    modifier
}

fn record_md3_shadow(
    fragment: &mut tessera_ui::RenderFragment,
    shadow: ShadowLayers,
    shape: ResolvedShape,
    size: PxSize,
    ambient_spread: f32,
) {
    let ambient = shadow
        .ambient
        .filter(|layer| layer.color.a > 0.0 && layer.smoothness > 0.0);
    let spot = shadow
        .spot
        .filter(|layer| layer.color.a > 0.0 && layer.smoothness > 0.0);

    if let Some(ambient) = ambient {
        let growth = Px((ambient_spread * 2.0).round() as i32);
        let spread = growth.to_f32() / 2.0;
        let expanded = match shape {
            ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } => ResolvedShape::Rounded {
                corner_radii: corner_radii.map(|radius| {
                    if radius < spread && spread > 0.0 {
                        radius + spread * (1.0 - (1.0 - radius / spread).powi(3))
                    } else {
                        radius + spread
                    }
                }),
                corner_g2,
            },
            ResolvedShape::Ellipse => ResolvedShape::Ellipse,
        };
        let index = fragment.push_composite_command(ShadowAtlasCommand::new(
            expanded,
            ShadowLayers {
                ambient: Some(ambient),
                spot: None,
            },
        ));
        let op = &mut fragment.ops_mut()[index as usize];
        op.size_override = Some(PxSize::new(size.width + growth, size.height + growth));
        op.position_override = Some(PxPosition::from_f32_xy(-spread, -spread));
    }
    if let Some(spot) = spot {
        fragment.push_composite_command(ShadowAtlasCommand::new(
            shape,
            ShadowLayers {
                ambient: None,
                spot: Some(spot),
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{Color, Command, Px, PxPosition, PxSize, RenderFragment};

    use crate::{
        pipelines::shadow::atlas::ShadowAtlasCommand,
        shadow::{ShadowLayer, ShadowLayers},
        shape_def::ResolvedShape,
    };

    use super::record_md3_shadow;

    #[test]
    fn ambient_spread_expands_its_mask_without_moving_the_key_shadow() {
        let mut fragment = RenderFragment::default();
        let shape = ResolvedShape::Rounded {
            corner_radii: [0.0, 1.0, 4.0, 20.0],
            corner_g2: [2.0; 4],
        };
        let layer = ShadowLayer {
            color: Color::BLACK,
            offset: [0.0, 2.0],
            smoothness: 9.0,
        };
        record_md3_shadow(
            &mut fragment,
            ShadowLayers {
                ambient: Some(layer),
                spot: Some(layer),
            },
            shape,
            PxSize::new(Px(100), Px(40)),
            4.0,
        );
        let ops = fragment.ops();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].size_override, Some(PxSize::new(Px(108), Px(48))));
        assert_eq!(
            ops[0].position_override,
            Some(PxPosition::from_f32_xy(-4.0, -4.0))
        );
        assert_eq!(ops[1].size_override, None);
        assert_eq!(ops[1].position_override, None);
        let Command::Composite(ambient) = &ops[0].command else {
            panic!("ambient mask")
        };
        assert_eq!(
            ambient
                .downcast_ref::<ShadowAtlasCommand>()
                .expect("shadow")
                .shape,
            ResolvedShape::Rounded {
                corner_radii: [0.0, 3.3125, 8.0, 24.0],
                corner_g2: [2.0; 4]
            }
        );
        let Command::Composite(key) = &ops[1].command else {
            panic!("key mask")
        };
        assert_eq!(
            key.downcast_ref::<ShadowAtlasCommand>()
                .expect("shadow")
                .shape,
            shape
        );

        fragment.clear();
        record_md3_shadow(
            &mut fragment,
            ShadowLayers {
                ambient: Some(layer),
                spot: None,
            },
            ResolvedShape::Ellipse,
            PxSize::new(Px(40), Px(40)),
            1.5,
        );
        assert_eq!(
            fragment.ops()[0].size_override,
            Some(PxSize::new(Px(43), Px(43)))
        );
        assert_eq!(
            fragment.ops()[0].position_override,
            Some(PxPosition::from_f32_xy(-1.5, -1.5))
        );
    }
}
