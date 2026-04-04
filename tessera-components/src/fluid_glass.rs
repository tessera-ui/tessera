//! A component for creating a frosted/distorted glass visual effect.
//!
//! ## Usage
//!
//! Use as a background for buttons, panels, or other UI elements.
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, Dp, FocusRequester, MeasurementError, Modifier,
    PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot, SampleRegion, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{
        LayoutInput, LayoutOutput, LayoutPolicy, RenderInput, RenderPolicy, layout_primitive,
    },
    modifier::ModifierCapabilityExt as _,
    receive_frame_nanos, remember,
    renderer::DrawCommand,
    tessera,
};

use crate::{
    modifier::{ClickableArgs, InteractionState, ModifierExt, PointerEventContext, SemanticsArgs},
    padding_utils::remove_padding_from_constraint,
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

/// Fully resolved fluid glass configuration passed to rendering pipelines.
#[derive(Clone, PartialEq)]
pub(crate) struct FluidGlassResolvedArgs {
    /// The tint color of the glass.
    /// The alpha channel uniquely and directly controls the tint strength.
    /// `A=0.0` means no tint (100% background visibility).
    /// `A=1.0` means full tint (100% color visibility).
    pub(crate) tint_color: Color,
    /// The shape of the component, an enum that can be `RoundedRectangle` or
    /// `Ellipse`.
    pub(crate) shape: Shape,
    /// The radius for the background blur effect. A value of `0.0` disables the
    /// blur.
    pub(crate) blur_radius: Dp,
    /// The height of the chromatic dispersion effect.
    pub(crate) dispersion_height: Dp,
    /// Multiplier for the chromatic aberration, enhancing the color separation
    /// effect.
    pub(crate) chroma_multiplier: f32,
    /// The height of the refraction effect, simulating light bending through
    /// the glass.
    pub(crate) refraction_height: Dp,
    /// The amount of refraction to apply.
    pub(crate) refraction_amount: f32,
    /// Controls the shape and eccentricity of the highlight.
    pub(crate) eccentric_factor: f32,
    /// The amount of noise to apply over the surface, adding texture.
    pub(crate) noise_amount: f32,
    /// The scale of the noise pattern.
    pub(crate) noise_scale: f32,
    /// A time value, typically used to animate the noise or other effects.
    pub(crate) time: f32,
    /// The contrast adjustment factor.
    pub(crate) contrast: Option<f32>,
    /// Optional modifier chain applied to the glass node.
    pub(crate) modifier: Modifier,
    /// Padding inside the glass component.
    pub(crate) padding: Dp,
    /// Optional normalized center (x, y) for the ripple animation on click.
    pub(crate) ripple_center: Option<[f32; 2]>,
    /// Optional ripple radius, expressed in normalized coordinates relative to
    /// the surface.
    pub(crate) ripple_radius: Option<f32>,
    /// Optional ripple tint alpha (0.0 = transparent, 1.0 = opaque).
    pub(crate) ripple_alpha: Option<f32>,
    /// Strength multiplier for the ripple distortion.
    pub(crate) ripple_strength: Option<f32>,

    /// Optional click callback for interactive glass surfaces.
    pub(crate) on_click: Option<Callback>,

    /// Optional border defining the outline thickness for the glass.
    pub(crate) border: Option<GlassBorder>,

    /// Whether to block input events on the glass surface.
    /// When `true`, the surface will consume all input events, preventing
    /// interaction with underlying components.
    pub(crate) block_input: bool,
    /// Optional accessibility role override; defaults to `Role::Button` when
    /// interactive.
    pub(crate) accessibility_role: Option<Role>,
    /// Optional label announced by assistive technologies.
    pub(crate) accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    pub(crate) accessibility_description: Option<String>,
    /// Whether the surface should be focusable even when not interactive.
    pub(crate) accessibility_focusable: bool,
}

impl Default for FluidGlassResolvedArgs {
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
        }
    }
}

impl FluidGlassBuilder {
    /// Creates props from base args and a child render function.
    pub fn with_child(self, child: impl Fn() + Send + Sync + 'static) -> Self {
        self.child(child)
    }
}

/// Draw command wrapping the arguments for the fluid glass surface.
#[derive(Clone, PartialEq)]
pub(crate) struct FluidGlassCommand {
    /// Full configuration used by the draw pipeline.
    pub(crate) args: FluidGlassResolvedArgs,
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

struct FluidGlassBlockInputPointerModifierNode {
    block_input: bool,
}

impl PointerInputModifierNode for FluidGlassBlockInputPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        if self.block_input {
            handle_block_input(&mut input);
        }
    }
}

fn apply_fluid_glass_block_input_modifier(base: Modifier, block_input: bool) -> Modifier {
    if block_input {
        base.push_pointer_input(FluidGlassBlockInputPointerModifierNode { block_input: true })
    } else {
        base
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
/// - `tint_color` — optional glass tint color.
/// - `shape` — optional glass shape.
/// - `blur_radius` — optional background blur radius.
/// - `dispersion_height` — optional chromatic dispersion height.
/// - `chroma_multiplier` — optional chromatic aberration multiplier.
/// - `refraction_height` — optional refraction height.
/// - `refraction_amount` — optional refraction strength.
/// - `eccentric_factor` — optional highlight eccentricity.
/// - `noise_amount` — optional noise amount.
/// - `noise_scale` — optional noise scale.
/// - `time` — optional animated time input.
/// - `contrast` — optional contrast adjustment.
/// - `modifier` — modifier chain applied to the glass subtree.
/// - `padding` — optional inner padding.
/// - `ripple_center` — optional normalized ripple center.
/// - `ripple_radius` — optional normalized ripple radius.
/// - `ripple_alpha` — optional ripple alpha.
/// - `ripple_strength` — optional ripple strength.
/// - `on_click` — optional click callback.
/// - `border` — optional glass outline.
/// - `block_input` — optional input blocking flag.
/// - `accessibility_role` — optional accessibility role.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `accessibility_focusable` — optional accessibility focusable flag.
/// - `child` — optional child render slot.
///
/// ## Examples
///
/// ```
/// use tessera_components::{fluid_glass::fluid_glass, text::text};
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// fluid_glass().child(|| {
///     text().content("Content on glass");
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn fluid_glass(
    tint_color: Option<Color>,
    shape: Option<Shape>,
    blur_radius: Option<Dp>,
    dispersion_height: Option<Dp>,
    chroma_multiplier: Option<f32>,
    refraction_height: Option<Dp>,
    refraction_amount: Option<f32>,
    eccentric_factor: Option<f32>,
    noise_amount: Option<f32>,
    noise_scale: Option<f32>,
    time: Option<f32>,
    contrast: Option<f32>,
    modifier: Modifier,
    padding: Option<Dp>,
    ripple_center: Option<[f32; 2]>,
    ripple_radius: Option<f32>,
    ripple_alpha: Option<f32>,
    ripple_strength: Option<f32>,
    on_click: Option<Callback>,
    border: Option<GlassBorder>,
    block_input: Option<bool>,
    accessibility_role: Option<Role>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    accessibility_focusable: Option<bool>,
    child: Option<RenderSlot>,
) {
    let fluid_args = FluidGlassResolvedArgs {
        tint_color: tint_color.unwrap_or(Color::TRANSPARENT),
        shape: shape.unwrap_or(Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(25.0), 3.0),
            top_right: RoundedCorner::manual(Dp(25.0), 3.0),
            bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
            bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
        }),
        blur_radius: blur_radius.unwrap_or(Dp(0.0)),
        dispersion_height: dispersion_height.unwrap_or(Dp(25.0)),
        chroma_multiplier: chroma_multiplier.unwrap_or(1.1),
        refraction_height: refraction_height.unwrap_or(Dp(24.0)),
        refraction_amount: refraction_amount.unwrap_or(32.0),
        eccentric_factor: eccentric_factor.unwrap_or(0.2),
        noise_amount: noise_amount.unwrap_or(0.0),
        noise_scale: noise_scale.unwrap_or(1.0),
        time: time.unwrap_or(0.0),
        contrast,
        modifier,
        padding: padding.unwrap_or(Dp(0.0)),
        ripple_center,
        ripple_radius,
        ripple_alpha,
        ripple_strength,
        on_click,
        border: border.or(Some(GlassBorder {
            width: Dp(1.35).into(),
        })),
        block_input: block_input.unwrap_or(false),
        accessibility_role,
        accessibility_label,
        accessibility_description,
        accessibility_focusable: accessibility_focusable.unwrap_or(false),
    };

    let mut modifier = fluid_args.modifier.clone();
    let interactive = fluid_args.on_click.is_some();
    let focus_requester = remember(FocusRequester::new).get();
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
        let clickable_args = ClickableArgs {
            on_click: fluid_args
                .on_click
                .expect("interactive implies on_click is set"),
            block_input: fluid_args.block_input,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            role: fluid_args
                .accessibility_role
                .or_else(|| fluid_args.accessibility_focusable.then_some(Role::Button)),
            label: fluid_args.accessibility_label.clone(),
            description: fluid_args.accessibility_description.clone(),
            interaction_state,
            focus_requester: Some(focus_requester),
            ..Default::default()
        };

        modifier = modifier.clickable_with(clickable_args);
    } else if fluid_args.block_input {
        modifier = modifier.block_touch_propagation();
    }
    if !interactive && has_semantics {
        let semantics = SemanticsArgs {
            role: fluid_args
                .accessibility_role
                .or_else(|| fluid_args.accessibility_focusable.then_some(Role::Button)),
            label: fluid_args.accessibility_label.clone(),
            description: fluid_args.accessibility_description.clone(),
            focusable: fluid_args.accessibility_focusable,
            ..Default::default()
        };
        modifier = modifier.semantics(semantics);
    }

    layout_primitive().modifier(modifier).child(move || {
        let mut builder = fluid_glass_inner().fluid(fluid_args.clone());
        if let Some(ripple_state) = ripple_state {
            builder = builder.ripple_state(ripple_state);
        }
        if let Some(child) = child {
            builder = builder.child_shared(child);
        }
        drop(builder);
    });
}

#[tessera]
fn fluid_glass_inner(
    fluid: FluidGlassResolvedArgs,
    ripple_state: Option<State<RippleState>>,
    child: Option<RenderSlot>,
) {
    let mut fluid_args = fluid.clone();
    let frame_nanos = current_frame_nanos();
    if let Some((progress, center)) = ripple_state.as_ref().and_then(|state| {
        state.with_mut(|ripple| {
            ripple
                .animation_at_frame_nanos(frame_nanos)
                .map(|animation| (animation.progress, animation.center))
        })
    }) {
        if let Some(ripple_state) = ripple_state {
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
    let modifier = apply_fluid_glass_block_input_modifier(
        Modifier::new(),
        fluid_args.on_click.is_none() && fluid_args.block_input,
    );
    let policy = FluidGlassLayout {
        args: fluid_args.clone(),
    };
    layout_primitive()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            if let Some(child) = child.as_ref() {
                child.render();
            }
        });
}

#[derive(Clone, PartialEq)]
struct FluidGlassLayout {
    args: FluidGlassResolvedArgs,
}

impl LayoutPolicy for FluidGlassLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let effective_glass_constraint = *input.parent_constraint().as_ref();

        let child_constraint = Constraint::new(
            remove_padding_from_constraint(
                effective_glass_constraint.width,
                self.args.padding.into(),
            ),
            remove_padding_from_constraint(
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
        let width = effective_glass_constraint.width.clamp(min_width);
        let height = effective_glass_constraint.height.clamp(min_height);

        Ok(ComputedData { width, height })
    }
}

impl RenderPolicy for FluidGlassLayout {
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
