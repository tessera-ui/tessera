use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, Px,
    PxPosition, PxSize, modifier::ModifierCapabilityExt as _,
};

use crate::{
    md3::{
        shadow::{ShadowLayer, ShadowLayers, material_shadow},
        theme::MaterialColorScheme,
    },
    pipelines::shape::command::ShapeCommand,
    shape_def::{ResolvedShape, Shape},
};

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
    fragment: &mut tessera_ui::render::RenderFragment,
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
        push_shadow(
            fragment,
            ambient,
            expanded,
            -spread,
            Some(PxSize::new(size.width + growth, size.height + growth)),
        );
    }
    if let Some(spot) = spot {
        push_shadow(fragment, spot, shape, 0.0, None);
    }
}

fn push_shadow(
    fragment: &mut tessera_ui::render::RenderFragment,
    layer: ShadowLayer,
    shape: ResolvedShape,
    inset: f32,
    size: Option<PxSize>,
) {
    let index = fragment.push_draw_command(ShapeCommand::Shadow {
        color: layer.color,
        shape,
        smoothness: layer.smoothness,
    });
    let op = &mut fragment.ops_mut()[index as usize];
    op.size_override = size;
    op.position_override = Some(PxPosition::from_f32_xy(
        inset + layer.offset[0],
        inset + layer.offset[1],
    ));
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        Color, Px, PxPosition, PxSize,
        render::{Command, RenderFragment},
    };

    use crate::{
        md3::shadow::{ShadowLayer, ShadowLayers},
        pipelines::shape::command::ShapeCommand,
        shape_def::ResolvedShape,
    };

    use super::record_md3_shadow;

    fn shadow_shape(command: &Command) -> ResolvedShape {
        let Command::Draw(command) = command else {
            panic!("shadow draw")
        };
        match command.downcast_ref::<ShapeCommand>() {
            Some(ShapeCommand::Shadow { shape, .. }) => *shape,
            _ => panic!("shadow shape"),
        }
    }

    #[test]
    fn ambient_spread_expands_its_shape_and_both_layers_keep_their_offsets() {
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
            Some(PxPosition::from_f32_xy(-4.0, -2.0))
        );
        assert_eq!(ops[1].size_override, None);
        assert_eq!(
            ops[1].position_override,
            Some(PxPosition::from_f32_xy(0.0, 2.0))
        );
        assert_eq!(
            shadow_shape(&ops[0].command),
            ResolvedShape::Rounded {
                corner_radii: [0.0, 3.3125, 8.0, 24.0],
                corner_g2: [2.0; 4]
            }
        );
        assert_eq!(shadow_shape(&ops[1].command), shape);

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
            Some(PxPosition::from_f32_xy(-1.5, 0.5))
        );
    }
}
