//! A component for creating a frosted liquid lens glass visual effect.
//!
//! ## Usage
//!
//! Use as a background for buttons, panels, or other UI elements.
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, Dp, FocusRequester, LayoutResult, MeasurementError,
    Modifier, PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutPolicy, MeasureScope, RenderInput, RenderPolicy, layout},
    modifier::ModifierCapabilityExt as _,
    receive_frame_nanos, remember, tessera,
};

use crate::{
    modifier::{ClickableArgs, InteractionState, ModifierExt, PointerEventContext, SemanticsArgs},
    padding_utils::remove_padding_from_constraint,
    pipelines::{
        blur::command::DualBlurCommand,
        contrast::ContrastCommand,
        fluid_glass::{FluidGlassCommand, FluidGlassRenderArgs},
        mean::command::MeanCommand,
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

impl Default for FluidGlassRenderArgs {
    fn default() -> Self {
        Self {
            tint_color: Color::TRANSPARENT,
            shape: default_glass_shape(),
            noise_amount: 0.0,
            noise_scale: 1.0,
            time: 0.0,
            ripple_center: None,
            ripple_radius: None,
            ripple_alpha: None,
            ripple_strength: None,
            border: default_glass_border(),
        }
    }
}

fn default_glass_shape() -> Shape {
    Shape::RoundedRectangle {
        top_left: RoundedCorner::manual(Dp(25.0), 3.0),
        top_right: RoundedCorner::manual(Dp(25.0), 3.0),
        bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
        bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
    }
}

fn default_glass_border() -> Option<GlassBorder> {
    Some(GlassBorder {
        width: Dp(1.35).into(),
    })
}

impl FluidGlassBuilder {
    /// Creates props from base args and a child render function.
    pub fn with_child(self, child: impl Fn() + Send + Sync + 'static) -> Self {
        self.child(child)
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
/// Renders a highly customizable surface with blur, tint, and a liquid lens
/// deformation effect.
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
    let tint_color = tint_color.unwrap_or(Color::TRANSPARENT);
    let shape = shape.unwrap_or_else(default_glass_shape);
    let blur_radius = blur_radius.unwrap_or(Dp(0.0));
    let noise_amount = noise_amount.unwrap_or(0.0);
    let noise_scale = noise_scale.unwrap_or(1.0);
    let time = time.unwrap_or(0.0);
    let padding = padding.unwrap_or(Dp(0.0));
    let border = border.or_else(default_glass_border);
    let block_input = block_input.unwrap_or(false);
    let accessibility_focusable = accessibility_focusable.unwrap_or(false);
    let mut modifier = modifier;
    let interactive = on_click.is_some();
    let focus_requester = remember(FocusRequester::new).get();
    let interaction_state = interactive.then(|| remember(InteractionState::new));
    let ripple_state = interactive.then(|| remember(RippleState::new));
    let has_semantics = accessibility_role.is_some()
        || accessibility_label.is_some()
        || accessibility_description.is_some()
        || accessibility_focusable;

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
            on_click: on_click.expect("interactive implies on_click is set"),
            block_input,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            role: accessibility_role.or_else(|| accessibility_focusable.then_some(Role::Button)),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            interaction_state,
            focus_requester: Some(focus_requester),
            ..Default::default()
        };

        modifier = modifier.clickable_with(clickable_args);
    } else if block_input {
        modifier = modifier.block_touch_propagation();
    }
    if !interactive && has_semantics {
        let semantics = SemanticsArgs {
            role: accessibility_role.or_else(|| accessibility_focusable.then_some(Role::Button)),
            label: accessibility_label.clone(),
            description: accessibility_description.clone(),
            focusable: accessibility_focusable,
            ..Default::default()
        };
        modifier = modifier.semantics(semantics);
    }

    let render = FluidGlassRenderArgs {
        tint_color,
        shape,
        noise_amount,
        noise_scale,
        time,
        ripple_center,
        ripple_radius,
        ripple_alpha,
        ripple_strength,
        border,
    };

    layout().modifier(modifier).child(move || {
        let mut builder = fluid_glass_inner()
            .render(render.clone())
            .blur_radius(blur_radius)
            .padding(padding)
            .interactive(interactive)
            .block_input(block_input);
        if let Some(contrast) = contrast {
            builder = builder.contrast(contrast);
        }
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
    render: FluidGlassRenderArgs,
    blur_radius: Dp,
    contrast: Option<f32>,
    padding: Dp,
    interactive: bool,
    block_input: bool,
    ripple_state: Option<State<RippleState>>,
    child: Option<RenderSlot>,
) {
    let mut render = render.clone();
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

        render.ripple_center = Some(center);
        render.ripple_radius = Some(progress);
        render.ripple_alpha = Some((1.0 - progress) * 0.3);
        render.ripple_strength = Some(progress);
    }
    let modifier =
        apply_fluid_glass_block_input_modifier(Modifier::new(), !interactive && block_input);
    let policy = FluidGlassLayout {
        render: render.clone(),
        blur_radius,
        contrast,
        padding,
    };
    layout()
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
    render: FluidGlassRenderArgs,
    blur_radius: Dp,
    contrast: Option<f32>,
    padding: Dp,
}

impl LayoutPolicy for FluidGlassLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let parent_constraint = *input.parent_constraint().as_ref();

        let child_constraint = Constraint::new(
            remove_padding_from_constraint(parent_constraint.width, self.padding.into()),
            remove_padding_from_constraint(parent_constraint.height, self.padding.into()),
        );

        let children = input.children();
        let child_measurement = if let Some(&child) = children.first() {
            let child_measurement = child.measure(&child_constraint)?;
            result.place_child(
                child,
                PxPosition {
                    x: self.padding.into(),
                    y: self.padding.into(),
                },
            );
            child_measurement.size()
        } else {
            ComputedData {
                width: Px(0),
                height: Px(0),
            }
        };

        let padding_px: Px = self.padding.into();
        let min_width = child_measurement.width + padding_px * 2;
        let min_height = child_measurement.height + padding_px * 2;
        let width = parent_constraint.width.clamp(min_width);
        let height = parent_constraint.height.clamp(min_height);

        Ok(result.with_size(ComputedData { width, height }))
    }
}

impl RenderPolicy for FluidGlassLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
        if self.blur_radius > Dp(0.0) {
            let blur_command =
                DualBlurCommand::horizontal_then_vertical(self.blur_radius.to_pixels_f32());
            let mut metadata = input.metadata_mut();
            metadata.fragment_mut().push_compute_command(blur_command);
        }

        if let Some(contrast_value) = self.contrast
            && contrast_value != 1.0
        {
            let mean_command = MeanCommand::new(input.gpu, input.compute_resource_manager);
            let contrast_command =
                ContrastCommand::new(contrast_value, mean_command.result_buffer_ref());
            let mut metadata = input.metadata_mut();
            metadata.fragment_mut().push_compute_command(mean_command);
            metadata
                .fragment_mut()
                .push_compute_command(contrast_command);
        }

        let drawable = FluidGlassCommand {
            render: self.render.clone(),
        };

        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(drawable);
    }
}
