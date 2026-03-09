//! A component for creating a frosted/distorted glass visual effect.
//!
//! ## Usage
//!
//! Use as a background for buttons, panels, or other UI elements.
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier,
    Prop, Px, PxPosition, RenderSlot, SampleRegion, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    receive_frame_nanos, remember,
    renderer::DrawCommand,
    tessera,
};

use crate::{
    modifier::{ClickableArgs, InteractionState, ModifierExt, PointerEventContext, SemanticsArgs},
    padding_utils::remove_padding_from_dimension,
    pipelines::{
        blur::command::DualBlurCommand, contrast::ContrastCommand, mean::command::MeanCommand,
    },
    pos_misc::is_position_inside_bounds,
    ripple_state::RippleState,
    shape_def::{RoundedCorner, Shape},
};

/// Border properties applied to the glass surface.
///
/// # Example
///
/// ```
/// use tessera_components::fluid_glass::GlassBorder;
/// use tessera_ui::Px;
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
/// This struct uses fluent Prop for easy construction.
#[derive(Clone, Prop)]
pub struct FluidGlassArgs {
    /// The tint color of the glass.
    /// The alpha channel uniquely and directly controls the tint strength.
    /// `A=0.0` means no tint (100% background visibility).
    /// `A=1.0` means full tint (100% color visibility).
    pub tint_color: Color,
    /// The shape of the component, an enum that can be `RoundedRectangle` or
    /// `Ellipse`.
    pub shape: Shape,
    /// The radius for the background blur effect. A value of `0.0` disables the
    /// blur.
    pub blur_radius: Dp,
    /// The height of the chromatic dispersion effect.
    pub dispersion_height: Dp,
    /// Multiplier for the chromatic aberration, enhancing the color separation
    /// effect.
    pub chroma_multiplier: f32,
    /// The height of the refraction effect, simulating light bending through
    /// the glass.
    pub refraction_height: Dp,
    /// The amount of refraction to apply.
    pub refraction_amount: f32,
    /// Controls the shape and eccentricity of the highlight.
    pub eccentric_factor: f32,
    /// The amount of noise to apply over the surface, adding texture.
    pub noise_amount: f32,
    /// The scale of the noise pattern.
    pub noise_scale: f32,
    /// A time value, typically used to animate the noise or other effects.
    pub time: f32,
    /// The contrast adjustment factor.
    pub contrast: Option<f32>,
    /// Optional modifier chain applied to the glass node.
    pub modifier: Modifier,
    /// Padding inside the glass component.
    pub padding: Dp,
    /// Optional normalized center (x, y) for the ripple animation on click.
    pub ripple_center: Option<[f32; 2]>,
    /// Optional ripple radius, expressed in normalized coordinates relative to
    /// the surface.
    pub ripple_radius: Option<f32>,
    /// Optional ripple tint alpha (0.0 = transparent, 1.0 = opaque).
    pub ripple_alpha: Option<f32>,
    /// Strength multiplier for the ripple distortion.
    pub ripple_strength: Option<f32>,

    /// Optional click callback for interactive glass surfaces.
    pub on_click: Option<Callback>,

    /// Optional border defining the outline thickness for the glass.
    pub border: Option<GlassBorder>,

    /// Whether to block input events on the glass surface.
    /// When `true`, the surface will consume all input events, preventing
    /// interaction with underlying components.
    pub block_input: bool,
    /// Optional accessibility role override; defaults to `Role::Button` when
    /// interactive.
    pub accessibility_role: Option<Role>,
    /// Optional label announced by assistive technologies.
    #[prop(into)]
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    #[prop(into)]
    pub accessibility_description: Option<String>,
    /// Whether the surface should be focusable even when not interactive.
    pub accessibility_focusable: bool,
    /// Optional child render slot.
    pub child: Option<RenderSlot>,
}

impl FluidGlassArgs {
    /// Creates props from base args and a child render function.
    pub fn with_child(
        args: impl Into<FluidGlassArgs>,
        child: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        args.into().child(child)
    }
}

impl Default for FluidGlassArgs {
    fn default() -> Self {
        Self {
            tint_color: Color::TRANSPARENT,
            shape: Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(25.0), 3.0),
                top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
            },
            blur_radius: Dp(0.0),
            dispersion_height: Dp(25.0),
            chroma_multiplier: 1.1,
            refraction_height: Dp(24.0),
            refraction_amount: 32.0,
            eccentric_factor: 0.2,
            noise_amount: 0.0,
            noise_scale: 1.0,
            time: 0.0,
            contrast: None,
            modifier: Modifier::new(),
            padding: Dp(0.0),
            ripple_center: None,
            ripple_radius: None,
            ripple_alpha: None,
            ripple_strength: None,
            on_click: None,
            border: Some(GlassBorder {
                width: Dp(1.35).into(),
            }),
            block_input: false,
            accessibility_role: None,
            accessibility_label: None,
            accessibility_description: None,
            accessibility_focusable: false,
            child: None,
        }
    }
}

impl From<&FluidGlassArgs> for FluidGlassArgs {
    fn from(value: &FluidGlassArgs) -> Self {
        value.clone()
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

// Helper: pointer blocking logic extracted to reduce complexity of
// `fluid_glass`.
fn handle_block_input(input: &mut tessera_ui::PointerInput) {
    let size = input.computed_data;
    let cursor_pos_option = input.cursor_position_rel;
    let is_cursor_in = cursor_pos_option
        .map(|pos| is_position_inside_bounds(size, pos))
        .unwrap_or(false);

    if is_cursor_in {
        // Consume all input events to prevent interaction with underlying components
        input.block_all();
    }
}

#[derive(Clone, Prop)]
struct FluidGlassInnerArgs {
    fluid: FluidGlassArgs,
    ripple_state: Option<State<RippleState>>,
    child: Option<RenderSlot>,
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
/// - `args` — props for this component; see [`FluidGlassArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     fluid_glass::{FluidGlassArgs, fluid_glass},
///     text::{TextArgs, text},
/// };
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// let args = FluidGlassArgs::default().child(|| {
///     text(&TextArgs::default().text("Content on glass"));
/// });
/// fluid_glass(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn fluid_glass(args: &FluidGlassArgs) {
    let fluid_args = args.clone();
    let mut modifier = fluid_args.modifier.clone();
    let interactive = fluid_args.on_click.is_some();
    let focus_requester = remember_focus_requester();
    let interaction_state = interactive.then(|| remember(InteractionState::new));
    let ripple_state = interactive.then(|| remember(RippleState::new));
    let has_semantics = fluid_args.accessibility_role.is_some()
        || fluid_args.accessibility_label.is_some()
        || fluid_args.accessibility_description.is_some()
        || fluid_args.accessibility_focusable;

    if interactive {
        let press_handler = ripple_state.map(|state| {
            move |ctx: PointerEventContext| {
                state.with_mut(|s| {
                    s.start_animation(ctx.normalized_pos);
                });
            }
        });
        let release_handler = ripple_state.map(|state| {
            move |_ctx: PointerEventContext| {
                state.with_mut(|s| s.release());
            }
        });
        let mut clickable_args = ClickableArgs::new(
            fluid_args
                .on_click
                .expect("interactive implies on_click is set"),
        )
        .block_input(fluid_args.block_input);

        if let Some(role) = fluid_args.accessibility_role {
            clickable_args = clickable_args.role(role);
        } else if fluid_args.accessibility_focusable {
            clickable_args = clickable_args.role(Role::Button);
        }
        if let Some(label) = fluid_args.accessibility_label.clone() {
            clickable_args = clickable_args.label(label);
        }
        if let Some(description) = fluid_args.accessibility_description.clone() {
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
        clickable_args = clickable_args.focus_requester(focus_requester);

        modifier = modifier.clickable(clickable_args);
    } else if fluid_args.block_input {
        modifier = modifier.block_touch_propagation();
    }
    if !interactive && has_semantics {
        let mut semantics = SemanticsArgs::new();
        if let Some(role) = fluid_args.accessibility_role {
            semantics = semantics.role(role);
        } else if fluid_args.accessibility_focusable {
            semantics = semantics.role(Role::Button);
        }
        if let Some(label) = fluid_args.accessibility_label.clone() {
            semantics = semantics.label(label);
        }
        if let Some(desc) = fluid_args.accessibility_description.clone() {
            semantics = semantics.description(desc);
        }
        if fluid_args.accessibility_focusable {
            semantics = semantics.focusable(true);
        }
        modifier = modifier.semantics(semantics);
    }

    let inner_args = FluidGlassInnerArgs {
        fluid: fluid_args,
        ripple_state,
        child: args.child.clone(),
    };

    modifier.run(move || fluid_glass_inner(&inner_args));
}

#[tessera]
fn fluid_glass_inner(args: &FluidGlassInnerArgs) {
    let mut fluid_args = args.fluid.clone();
    let frame_nanos = current_frame_nanos();
    if let Some((progress, center)) = args.ripple_state.as_ref().and_then(|state| {
        state.with_mut(|ripple| {
            ripple
                .animation_at_frame_nanos(frame_nanos)
                .map(|animation| (animation.progress, animation.center))
        })
    }) {
        if let Some(ripple_state) = args.ripple_state {
            receive_frame_nanos(move |frame_nanos| {
                let has_active_ripple = ripple_state
                    .with_mut(|ripple| ripple.animation_at_frame_nanos(frame_nanos).is_some());
                if has_active_ripple {
                    tessera_ui::FrameNanosControl::Continue
                } else {
                    tessera_ui::FrameNanosControl::Stop
                }
            });
        }

        fluid_args.ripple_center = Some(center);
        fluid_args.ripple_radius = Some(progress);
        fluid_args.ripple_alpha = Some((1.0 - progress) * 0.3);
        fluid_args.ripple_strength = Some(progress);
    }
    if let Some(child) = args.child.as_ref() {
        child.render();
    }
    layout(FluidGlassLayout {
        args: fluid_args.clone(),
    });

    if fluid_args.on_click.is_none() && fluid_args.block_input {
        pointer_input_handler(move |mut input: tessera_ui::PointerInput| {
            if fluid_args.block_input {
                handle_block_input(&mut input);
            }
        });
    }
}

#[derive(Clone, PartialEq)]
struct FluidGlassLayout {
    args: FluidGlassArgs,
}

impl LayoutSpec for FluidGlassLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let effective_glass_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

        let child_constraint = Constraint::new(
            remove_padding_from_dimension(
                effective_glass_constraint.width,
                self.args.padding.into(),
            ),
            remove_padding_from_dimension(
                effective_glass_constraint.height,
                self.args.padding.into(),
            ),
        );

        let child_measurement = if !input.children_ids().is_empty() {
            let child_measurement =
                input.measure_child(input.children_ids()[0], &child_constraint)?;
            output.place_child(
                input.children_ids()[0],
                PxPosition {
                    x: self.args.padding.into(),
                    y: self.args.padding.into(),
                },
            );
            child_measurement
        } else {
            ComputedData {
                width: Px(0),
                height: Px(0),
            }
        };

        let padding_px: Px = self.args.padding.into();
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
    }

    fn record(&self, input: &RenderInput<'_>) {
        if self.args.blur_radius > Dp(0.0) {
            let blur_command =
                DualBlurCommand::horizontal_then_vertical(self.args.blur_radius.to_pixels_f32());
            let mut metadata = input.metadata_mut();
            metadata.fragment_mut().push_compute_command(blur_command);
        }

        if let Some(contrast_value) = self.args.contrast
            && contrast_value != 1.0
        {
            let mean_command =
                MeanCommand::new(input.gpu, &mut input.compute_resource_manager.write());
            let contrast_command =
                ContrastCommand::new(contrast_value, mean_command.result_buffer_ref());
            let mut metadata = input.metadata_mut();
            metadata.fragment_mut().push_compute_command(mean_command);
            metadata
                .fragment_mut()
                .push_compute_command(contrast_command);
        }

        let drawable = FluidGlassCommand {
            args: self.args.clone(),
        };

        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(drawable);
    }
}
