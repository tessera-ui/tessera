use tessera_ui::{
    Color, ComputedData, Dp, MeasurementError, Modifier, PxPosition, PxSize, RenderSlot,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera, use_context,
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
#[derive(PartialEq, Clone, Debug)]
pub struct ShadowArgs {
    /// The elevation of the shadow.
    pub elevation: Dp,
    /// The shape of the shadow.
    pub shape: Shape,
    /// Whether to clip the content to the shape.
    pub clip: bool,
    /// Color of the ambient shadow. If None, uses the theme default.
    pub ambient_color: Option<Color>,
    /// Color of the spot shadow. If None, uses the theme default.
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

impl ShadowArgs {
    /// Creates a default `ShadowArgs` with the given elevation.
    pub fn new(elevation: Dp) -> Self {
        Self {
            elevation,
            ..Default::default()
        }
    }

    /// Sets the shape.
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets whether to clip the content.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the ambient shadow color.
    pub fn ambient_color(mut self, color: Color) -> Self {
        self.ambient_color = Some(color);
        self
    }

    /// Sets the spot shadow color.
    pub fn spot_color(mut self, color: Color) -> Self {
        self.spot_color = Some(color);
        self
    }
}

impl From<Dp> for ShadowArgs {
    fn from(elevation: Dp) -> Self {
        Self::new(elevation)
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

    let mut modifier = base.push_wrapper(move |child| {
        let shape = args.shape;
        let child = RenderSlot::new(child);
        move || {
            modifier_shadow_layers(layers, shape, child.clone());
        }
    });

    if args.clip {
        // Very basic clipping support (rect only for now as per existing modifier)
        modifier = modifier.clip_to_bounds();
    }

    modifier
}

#[derive(Clone, PartialEq)]
struct ShadowLayout {
    shadow: ShadowLayers,
    shape: Shape,
}

impl LayoutSpec for ShadowLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("modifier_shadow expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        output.place_child(child_id, PxPosition::ZERO);
        Ok(child_measurement)
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        let Some(size) = metadata.computed_data else {
            return;
        };
        let size = PxSize::from(size);
        if size.width.0 <= 0 || size.height.0 <= 0 {
            return;
        }
        record_md3_shadow(
            metadata.fragment_mut(),
            self.shadow,
            self.shape.resolve_for_size(size),
        );
    }
}

#[derive(Clone, PartialEq)]
struct ModifierShadowLayersArgs {
    shadow: ShadowLayers,
    shape: Shape,
    child: RenderSlot,
}

pub(super) fn modifier_shadow_layers(shadow: ShadowLayers, shape: Shape, child: RenderSlot) {
    let args = ModifierShadowLayersArgs {
        shadow,
        shape,
        child,
    };
    modifier_shadow_layers_node(&args);
}

#[tessera]
fn modifier_shadow_layers_node(args: &ModifierShadowLayersArgs) {
    layout(ShadowLayout {
        shadow: args.shadow,
        shape: args.shape,
    });
    args.child.render();
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
