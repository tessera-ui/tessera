use std::any::TypeId;

use smallvec::SmallVec;
use tessera_ui::{
    Color, Command, ComputedData, Dp, MeasurementError, Modifier, Px, PxPosition, PxSize,
    RenderFragmentOp, RenderResourceId, RenderTextureDesc,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera, use_context, wgpu,
};

use crate::{
    pipelines::{
        blur::command::{DualBlurCommand, downscale_factor_for_radius},
        shadow::command::{ShadowCompositeCommand, ShadowMaskCommand},
    },
    shadow::{ShadowLayer, ShadowLayers},
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
            size,
        );
    }
}

const SHADOW_AA_MARGIN_PX: f32 = 1.0;
// The blur shader tops out around ~30px effective radius at MAX_SAMPLES=16,
// so split larger blurs into multiple passes to preserve width.
const SHADOW_MAX_SINGLE_BLUR_RADIUS: f32 = 30.0;

fn blur_pass_radii(radius: f32) -> SmallVec<[f32; 4]> {
    if radius <= 0.0 {
        return SmallVec::new();
    }

    let max_radius = SHADOW_MAX_SINGLE_BLUR_RADIUS.max(1.0);
    let ratio = radius / max_radius;
    let pass_count = (ratio * ratio).ceil().max(1.0) as u32;
    let step_radius = radius / (pass_count as f32).sqrt();

    let mut radii = SmallVec::with_capacity(pass_count as usize);
    for _ in 0..pass_count {
        radii.push(step_radius);
    }
    radii
}

fn shadow_padding_xy(shadow: &ShadowLayers) -> (Px, Px) {
    let mut pad_x = 0.0f32;
    let mut pad_y = 0.0f32;

    let update = |pad_x: &mut f32, pad_y: &mut f32, layer: &ShadowLayer| {
        if layer.color.a <= 0.0 {
            return;
        }
        let scale = downscale_factor_for_radius(layer.smoothness) as f32;
        let blur_pad = (layer.smoothness * scale).max(0.0);
        let layer_pad_x = (blur_pad + layer.offset[0].abs() + SHADOW_AA_MARGIN_PX).max(0.0);
        let layer_pad_y = (blur_pad + layer.offset[1].abs() + SHADOW_AA_MARGIN_PX).max(0.0);
        *pad_x = (*pad_x).max(layer_pad_x);
        *pad_y = (*pad_y).max(layer_pad_y);
    };

    if let Some(layer) = shadow.ambient {
        update(&mut pad_x, &mut pad_y, &layer);
    }
    if let Some(layer) = shadow.spot {
        update(&mut pad_x, &mut pad_y, &layer);
    }

    (
        Px::new(pad_x.ceil() as i32).max(Px::ZERO),
        Px::new(pad_y.ceil() as i32).max(Px::ZERO),
    )
}

#[tessera]
pub(super) fn modifier_shadow_layers<F>(shadow: ShadowLayers, shape: Shape, child: F)
where
    F: FnOnce(),
{
    layout(ShadowLayout { shadow, shape });

    child();
}

fn record_md3_shadow(
    fragment: &mut tessera_ui::RenderFragment,
    shadow: ShadowLayers,
    shape: ResolvedShape,
    size: PxSize,
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

    let (pad_x, pad_y) = shadow_padding_xy(&active_layers);
    let pad_pos = PxPosition::new(pad_x, pad_y);
    let mask_size = PxSize::new(size.width + pad_x + pad_x, size.height + pad_y + pad_y);
    if mask_size.width.0 <= 0 || mask_size.height.0 <= 0 {
        return;
    }

    let mask_id = fragment.add_local_texture(RenderTextureDesc {
        size: mask_size,
        format: wgpu::TextureFormat::Rgba8Unorm,
    });
    let blur_id = fragment.add_local_texture(RenderTextureDesc {
        size: mask_size,
        format: wgpu::TextureFormat::Rgba8Unorm,
    });

    let mask_op = fragment.push_op(RenderFragmentOp {
        command: Command::Draw(Box::new(ShadowMaskCommand::new(shape))),
        type_id: TypeId::of::<ShadowMaskCommand>(),
        read: None,
        write: Some(mask_id),
        deps: SmallVec::new(),
        size_override: Some(size),
        position_override: Some(pad_pos),
    });

    for layer in [active_layers.ambient, active_layers.spot]
        .into_iter()
        .flatten()
    {
        let mut last_blur_op = None;
        for (index, radius) in blur_pass_radii(layer.smoothness)
            .iter()
            .copied()
            .enumerate()
        {
            let blur_command = DualBlurCommand::horizontal_then_vertical(radius);
            let blur_write = Some(blur_id);
            let mut blur_deps = SmallVec::new();

            let blur_read = if index == 0 {
                blur_deps.push(mask_op);
                Some(mask_id)
            } else {
                if let Some(prev) = last_blur_op {
                    blur_deps.push(prev);
                }
                Some(blur_id)
            };

            let blur_op = fragment.push_op(RenderFragmentOp {
                command: Command::Compute(Box::new(blur_command)),
                type_id: TypeId::of::<DualBlurCommand>(),
                read: blur_read,
                write: blur_write,
                deps: blur_deps,
                size_override: Some(mask_size),
                position_override: Some(PxPosition::ZERO),
            });
            last_blur_op = Some(blur_op);
        }

        let Some(blur_op) = last_blur_op else {
            continue;
        };

        let offset = PxPosition::new(Px::from_f32(layer.offset[0]), Px::from_f32(layer.offset[1]));
        let composite_offset = offset - pad_pos;
        let ordering_offset = pad_pos - offset;

        let mut composite_deps = SmallVec::new();
        composite_deps.push(blur_op);

        fragment.push_op(RenderFragmentOp {
            command: Command::Draw(Box::new(
                ShadowCompositeCommand::new(layer.color).with_ordering(ordering_offset, size),
            )),
            type_id: TypeId::of::<ShadowCompositeCommand>(),
            read: Some(blur_id),
            write: Some(RenderResourceId::SceneColor),
            deps: composite_deps,
            size_override: Some(mask_size),
            position_override: Some(composite_offset),
        });
    }
}
