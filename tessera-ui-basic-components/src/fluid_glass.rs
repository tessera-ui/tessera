//! A component for creating a frosted/distorted glass visual effect.
//!
//! ## Usage
//!
//! Use as a background for buttons, panels, or other UI elements.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, Modifier, Px, PxPosition, SampleRegion,
    State, accesskit::Role, remember, renderer::DrawCommand, tessera,
};

use crate::{
    modifier::{ClickableArgs, InteractionState, ModifierExt, PointerEventContext, SemanticsArgs},
    padding_utils::remove_padding_from_dimension,
    pipelines::{
        blur::command::DualBlurCommand, contrast::ContrastCommand, mean::command::MeanCommand,
    },
    pos_misc::is_position_in_component,
    ripple_state::RippleState,
    shape_def::{RoundedCorner, Shape},
};

/// Border properties applied to the glass surface.
///
/// # Example
///
/// ```
/// use tessera_ui::Px;
/// use tessera_ui_basic_components::fluid_glass::GlassBorder;
///
/// let border = GlassBorder::new(Px(2)); // Creates a border with 2 physical pixels width
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GlassBorder {
    /// Border width in physical pixels.
    pub width: Px,
}

impl GlassBorder {
    /// Creates a new border with the given width.
    pub fn new(width: Px) -> Self {
        Self { width }
    }
}

/// Arguments for the `fluid_glass` component, providing extensive control over
/// its appearance.
///
/// This struct uses the builder pattern for easy construction.
#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"), pattern = "owned", setter(into))]
pub struct FluidGlassArgs {
    /// The tint color of the glass.
    /// The alpha channel uniquely and directly controls the tint strength.
    /// `A=0.0` means no tint (100% background visibility).
    /// `A=1.0` means full tint (100% color visibility).
    #[builder(default = "Color::TRANSPARENT")]
    pub tint_color: Color,
    /// The shape of the component, an enum that can be `RoundedRectangle` or
    /// `Ellipse`.
    #[builder(default = "Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(25.0), 3.0),
            top_right: RoundedCorner::manual(Dp(25.0), 3.0),
            bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
            bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
        }")]
    pub shape: Shape,
    /// The radius for the background blur effect. A value of `0.0` disables the
    /// blur.
    #[builder(default = "Dp(0.0)")]
    pub blur_radius: Dp,
    /// The height of the chromatic dispersion effect.
    #[builder(default = "Dp(25.0)")]
    pub dispersion_height: Dp,
    /// Multiplier for the chromatic aberration, enhancing the color separation
    /// effect.
    #[builder(default = "1.1")]
    pub chroma_multiplier: f32,
    /// The height of the refraction effect, simulating light bending through
    /// the glass.
    #[builder(default = "Dp(24.0)")]
    pub refraction_height: Dp,
    /// The amount of refraction to apply.
    #[builder(default = "32.0")]
    pub refraction_amount: f32,
    /// Controls the shape and eccentricity of the highlight.
    #[builder(default = "0.2")]
    pub eccentric_factor: f32,
    /// The amount of noise to apply over the surface, adding texture.
    #[builder(default = "0.0")]
    pub noise_amount: f32,
    /// The scale of the noise pattern.
    #[builder(default = "1.0")]
    pub noise_scale: f32,
    /// A time value, typically used to animate the noise or other effects.
    #[builder(default = "0.0")]
    pub time: f32,
    /// The contrast adjustment factor.
    #[builder(default, setter(strip_option))]
    pub contrast: Option<f32>,
    /// Optional modifier chain applied to the glass node.
    #[builder(default = "Modifier::new()")]
    pub modifier: Modifier,
    /// Padding inside the glass component.
    #[builder(default = "Dp(0.0)")]
    pub padding: Dp,
    /// Optional normalized center (x, y) for the ripple animation on click.
    #[builder(default, setter(strip_option))]
    pub ripple_center: Option<[f32; 2]>,
    /// Optional ripple radius, expressed in normalized coordinates relative to
    /// the surface.
    #[builder(default, setter(strip_option))]
    pub ripple_radius: Option<f32>,
    /// Optional ripple tint alpha (0.0 = transparent, 1.0 = opaque).
    #[builder(default, setter(strip_option))]
    pub ripple_alpha: Option<f32>,
    /// Strength multiplier for the ripple distortion.
    #[builder(default, setter(strip_option))]
    pub ripple_strength: Option<f32>,

    /// Optional click callback for interactive glass surfaces.
    #[builder(default, setter(custom, strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,

    /// Optional border defining the outline thickness for the glass.
    #[builder(default = "Some(GlassBorder { width: Dp(1.35).into() })")]
    pub border: Option<GlassBorder>,

    /// Whether to block input events on the glass surface.
    /// When `true`, the surface will consume all input events, preventing
    /// interaction with underlying components.
    #[builder(default = "false")]
    pub block_input: bool,
    /// Optional accessibility role override; defaults to `Role::Button` when
    /// interactive.
    #[builder(default, setter(strip_option))]
    pub accessibility_role: Option<Role>,
    /// Optional label announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
    /// Whether the surface should be focusable even when not interactive.
    #[builder(default)]
    pub accessibility_focusable: bool,
}

impl PartialEq for FluidGlassArgs {
    fn eq(&self, other: &Self) -> bool {
        self.tint_color == other.tint_color
            && self.shape == other.shape
            && self.blur_radius == other.blur_radius
            && self.dispersion_height == other.dispersion_height
            && self.chroma_multiplier == other.chroma_multiplier
            && self.refraction_height == other.refraction_height
            && self.refraction_amount == other.refraction_amount
            && self.eccentric_factor == other.eccentric_factor
            && self.noise_amount == other.noise_amount
            && self.noise_scale == other.noise_scale
            && self.time == other.time
            && self.contrast == other.contrast
            && self.padding == other.padding
            && self.ripple_center == other.ripple_center
            && self.ripple_radius == other.ripple_radius
            && self.ripple_alpha == other.ripple_alpha
            && self.ripple_strength == other.ripple_strength
            && self.border == other.border
            && self.block_input == other.block_input
    }
}

impl FluidGlassArgsBuilder {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl FluidGlassArgsBuilder {
    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Some(Arc::new(on_click)));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(Some(on_click));
        self
    }
}

// Manual implementation of Default because derive_builder's default conflicts
// with our specific defaults
impl Default for FluidGlassArgs {
    fn default() -> Self {
        FluidGlassArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

/// Draw command wrapping the arguments for the fluid glass surface.
#[derive(Clone, PartialEq)]
pub struct FluidGlassCommand {
    /// Full configuration used by the draw pipeline.
    pub args: FluidGlassArgs,
}

impl DrawCommand for FluidGlassCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        Some(SampleRegion::uniform_padding_local(Px(10)))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        let factor = opacity.clamp(0.0, 1.0);
        self.args.tint_color = self
            .args
            .tint_color
            .with_alpha(self.args.tint_color.a * factor);
        if let Some(ripple_alpha) = self.args.ripple_alpha.as_mut() {
            *ripple_alpha *= factor;
        }
    }
}

// Helper: input handler logic extracted to reduce complexity of `fluid_glass`.
fn handle_block_input(input: &mut tessera_ui::InputHandlerInput) {
    let size = input.computed_data;
    let cursor_pos_option = input.cursor_position_rel;
    let is_cursor_in = cursor_pos_option
        .map(|pos| is_position_in_component(size, pos))
        .unwrap_or(false);

    if is_cursor_in {
        // Consume all input events to prevent interaction with underlying components
        input.block_all();
    }
}

/// # fluid_glass
///
/// Renders a highly customizable surface with blur, tint, and other glass-like
/// effects.
///
/// ## Usage
///
/// Use to create a dynamic, layered background for other components.
///
/// ## Parameters
///
/// - `args` — configures the glass effect's appearance; see [`FluidGlassArgs`].
/// - `child` — a closure that renders content on top of the glass surface.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::{
///     fluid_glass::{FluidGlassArgs, fluid_glass},
///     text::{TextArgsBuilder, text},
/// };
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// fluid_glass(FluidGlassArgs::default(), || {
///     text(
///         TextArgsBuilder::default()
///             .text("Content on glass".to_string())
///             .build()
///             .expect("builder construction failed"),
///     );
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn fluid_glass(args: FluidGlassArgs, child: impl FnOnce() + Send + Sync + 'static) {
    let mut modifier = args.modifier;
    let interactive = args.on_click.is_some();
    let interaction_state = interactive.then(|| remember(InteractionState::new));
    let ripple_state = interactive.then(|| remember(RippleState::new));
    let has_semantics = args.accessibility_role.is_some()
        || args.accessibility_label.is_some()
        || args.accessibility_description.is_some();

    if interactive {
        let press_handler = ripple_state.map(|state| {
            Arc::new(move |ctx: PointerEventContext| {
                state.with_mut(|s| {
                    s.start_animation(ctx.normalized_pos);
                });
            })
        });
        let release_handler = ripple_state.map(|state| {
            Arc::new(move |_ctx: PointerEventContext| {
                state.with_mut(|s| s.release());
            })
        });
        let mut clickable_args = ClickableArgs::new(
            args.on_click
                .clone()
                .expect("interactive implies on_click is set"),
        )
        .block_input(args.block_input);

        if let Some(role) = args.accessibility_role {
            clickable_args = clickable_args.role(role);
        }
        if let Some(label) = args.accessibility_label.clone() {
            clickable_args = clickable_args.label(label);
        }
        if let Some(description) = args.accessibility_description.clone() {
            clickable_args = clickable_args.description(description);
        }
        if let Some(state) = interaction_state {
            clickable_args = clickable_args.interaction_state(state);
        }
        if let Some(handler) = press_handler {
            clickable_args = clickable_args.on_press(handler);
        }
        if let Some(handler) = release_handler {
            clickable_args = clickable_args.on_release(handler);
        }

        modifier = modifier.clickable(clickable_args);
    } else if args.block_input {
        modifier = modifier.block_touch_propagation();
    }
    if !interactive && has_semantics {
        let mut semantics = SemanticsArgs::new();
        if let Some(role) = args.accessibility_role {
            semantics = semantics.role(role);
        }
        if let Some(label) = args.accessibility_label.clone() {
            semantics = semantics.label(label);
        }
        if let Some(desc) = args.accessibility_description.clone() {
            semantics = semantics.description(desc);
        }
        modifier = modifier.semantics(semantics);
    }

    modifier.run(move || fluid_glass_inner(args, ripple_state, child));
}

#[tessera]
fn fluid_glass_inner(
    mut args: FluidGlassArgs,
    ripple_state: Option<State<RippleState>>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    if let Some((progress, center)) = ripple_state
        .as_ref()
        .and_then(|state| state.with_mut(|s| s.get_animation_progress()))
    {
        args.ripple_center = Some(center);
        args.ripple_radius = Some(progress);
        args.ripple_alpha = Some((1.0 - progress) * 0.3);
        args.ripple_strength = Some(progress);
    }
    (child)();
    let args_measure_clone = args.clone();
    measure(Box::new(move |input| {
        let effective_glass_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );

        let child_constraint = Constraint::new(
            remove_padding_from_dimension(
                effective_glass_constraint.width,
                args_measure_clone.padding.into(),
            ),
            remove_padding_from_dimension(
                effective_glass_constraint.height,
                args_measure_clone.padding.into(),
            ),
        );

        let child_measurement = if !input.children_ids.is_empty() {
            let child_measurement =
                input.measure_child(input.children_ids[0], &child_constraint)?;
            input.place_child(
                input.children_ids[0],
                PxPosition {
                    x: args.padding.into(),
                    y: args.padding.into(),
                },
            );
            child_measurement
        } else {
            ComputedData {
                width: Px(0),
                height: Px(0),
            }
        };

        if args.blur_radius > Dp(0.0) {
            let blur_command =
                DualBlurCommand::horizontal_then_vertical(args.blur_radius.to_pixels_f32());
            let mut metadata = input.metadata_mut();
            metadata.push_compute_command(blur_command);
        }

        if let Some(contrast_value) = args.contrast
            && contrast_value != 1.0
        {
            let mean_command =
                MeanCommand::new(input.gpu, &mut input.compute_resource_manager.write());
            let contrast_command =
                ContrastCommand::new(contrast_value, mean_command.result_buffer_ref());
            let mut metadata = input.metadata_mut();
            metadata.push_compute_command(mean_command);
            metadata.push_compute_command(contrast_command);
        }

        let drawable = FluidGlassCommand {
            args: args_measure_clone.clone(),
        };

        input.metadata_mut().push_draw_command(drawable);

        let padding_px: Px = args_measure_clone.padding.into();
        let min_width = child_measurement.width + padding_px * 2;
        let min_height = child_measurement.height + padding_px * 2;
        let width = match effective_glass_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect("Seems that you are trying to fill an infinite width, which is not allowed")
                .max(min_width)
                .max(min.unwrap_or(Px(0))),
        };
        let height = match effective_glass_constraint.height {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_height)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect(
                    "Seems that you are trying to fill an infinite height, which is not allowed",
                )
                .max(min_height)
                .max(min.unwrap_or(Px(0))),
        };
        Ok(ComputedData { width, height })
    }));

    if args.on_click.is_none() && args.block_input {
        let args_for_handler = args.clone();
        input_handler(Box::new(move |mut input: tessera_ui::InputHandlerInput| {
            if args_for_handler.block_input {
                handle_block_input(&mut input);
            }
        }));
    }
}
