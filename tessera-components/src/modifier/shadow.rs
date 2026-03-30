use tessera_ui::{
    Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, PxSize,
    modifier::ModifierCapabilityExt as _, use_context,
};

use crate::{
    pipelines::shadow::atlas::ShadowAtlasCommand,
    shadow::ShadowLayers,
    shape_def::{ResolvedShape, Shape},
    surface::SurfaceDefaults,
    theme::MaterialTheme,
};

use super::ModifierExt;

/// Arguments for the `shadow` modifier.
#[derive(Clone, Debug)]
pub struct ShadowArgs {
    /// The elevation of the shadow.
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
}

impl DrawModifierNode for ShadowModifierNode {
    fn draw(&self, ctx: &mut DrawModifierContext<'_>, content: &mut dyn DrawModifierContent) {
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
            );
        }
        drop(metadata);
        content.draw(ctx.render_input);
    }
}

pub(super) fn apply_shadow_modifier(base: Modifier, args: ShadowArgs) -> Modifier {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let mut layers = SurfaceDefaults::synthesize_shadow_layers(args.elevation, &scheme);

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
    });

    if args.clip {
        // Very basic clipping support (rect only for now as per existing modifier)
        modifier = modifier.clip_to_bounds();
    }

    modifier
}

fn record_md3_shadow(
    fragment: &mut tessera_ui::RenderFragment,
    shadow: ShadowLayers,
    shape: ResolvedShape,
) {
    let ambient = shadow
        .ambient
        .filter(|layer| layer.color.a > 0.0 && layer.smoothness > 0.0);
    let spot = shadow
        .spot
        .filter(|layer| layer.color.a > 0.0 && layer.smoothness > 0.0);

    let active_layers = ShadowLayers { ambient, spot };
    if active_layers.ambient.is_none() && active_layers.spot.is_none() {
        return;
    }

    fragment.push_composite_command(ShadowAtlasCommand::new(shape, active_layers));
}
