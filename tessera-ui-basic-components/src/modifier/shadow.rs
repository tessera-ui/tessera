use tessera_ui::{Color, Dp, Modifier, PxPosition, PxSize, tessera, use_context};

use crate::{
    pipelines::shape::command::{ShadowLayers, ShapeCommand},
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
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
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
        move || {
            modifier_shadow_layers(layers, shape, || {
                child();
            });
        }
    });

    if args.clip {
        // Very basic clipping support (rect only for now as per existing modifier)
        modifier = modifier.clip_to_bounds();
    }

    modifier
}

#[tessera]
pub(super) fn modifier_shadow_layers<F>(shadow: ShadowLayers, shape: Shape, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_shadow expects exactly one child");
        let child_measurement = input.measure_child_in_parent_constraint(child_id)?;
        input
            .metadata_mut()
            .push_draw_command(shape_shadow_command_layers(
                shadow,
                shape,
                child_measurement.into(),
            ));

        input.place_child(child_id, PxPosition::ZERO);

        Ok(child_measurement)
    }));

    child();
}

pub(super) fn shape_shadow_command_layers(
    shadow: ShadowLayers,
    shape: Shape,
    size: PxSize,
) -> ShapeCommand {
    let color = Color::TRANSPARENT;
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::Rect {
            color,
            corner_radii,
            corner_g2,
            shadow: Some(shadow),
        },
        ResolvedShape::Ellipse => ShapeCommand::Ellipse {
            color,
            shadow: Some(shadow),
        },
    }
}
