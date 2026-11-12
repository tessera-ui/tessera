//! A component for creating a frosted liquid lens glass visual effect.
//!
//! ## Usage
//!
//! Use as a background for buttons, panels, or other UI elements.
use tessera_foundation::modifier::{
    ClickableArgs, InteractionState, ModifierExt as _, PointerEventContext, SemanticsArgs,
};
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, CursorEventContent, Dp, FocusRequester,
    LayoutResult, MeasurementError, Modifier, PointerEventPass, PointerId, PointerInput,
    PointerInputModifierNode, PressKeyEventType, Px, PxPosition, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{MeasureScope, Stack, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera, use_context,
};

use crate::{
    modifier::ModifierExt as _,
    padding_utils::remove_padding_from_constraint,
    pipelines::{
        blur::command::DualBlurCommand,
        contrast::ContrastCommand,
        fluid_glass::{FluidGlassCommand, FluidGlassRenderArgs},
    },
    pos_misc::is_position_inside_bounds,
    ripple_state::RippleState,
    shape_def::{RoundedCorner, Shape},
    theme::MaterialTheme,
};

/// Selects how the glass filters its backdrop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassMaterial {
    /// Frosted glass for controls and surfaces that need stronger separation.
    #[default]
    Regular,
    /// More transparent glass that retains more of the backdrop's color and contrast.
    Clear,
    /// A refractive lens with no default blur or tone adjustment.
    Custom,
}

/// Selects the glass material's light or dark treatment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlassAppearance {
    /// A brighter glass surface for use with dark content.
    Light,
    /// A darker glass surface for use with light content.
    Dark,
}

impl GlassMaterial {
    fn optics(self, appearance: GlassAppearance) -> Option<(Dp, [f32; 4])> {
        let (radius, tone) = match (self, appearance) {
            (Self::Regular, GlassAppearance::Light) => (4.43, [0.6736, 0.2956, 0.6534, 0.995]),
            (Self::Regular, GlassAppearance::Dark) => (4.43, [0.1316, 0.2600, 0.6488, 1.0183]),
            (Self::Clear, GlassAppearance::Light) => (13.22, [0.3126, 0.6380, 0.8613, 1.0538]),
            (Self::Clear, GlassAppearance::Dark) => (13.22, [0.1325, 0.6730, 0.9097, 1.0543]),
            (Self::Custom, _) => return None,
        };
        Some((Dp(radius), tone))
    }
}

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
            material_tone: None,
            blur_sigma_px: 0.0,
            pixels_per_dp: 1.0,
            opacity: 1.0,
            contrast: None,
            dark: false,
            feedback: 0.0,
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

struct FluidGlassBlockInputPointerModifierNode;

impl PointerInputModifierNode for FluidGlassBlockInputPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let size = input.computed_data;
        let cursor_pos_option = input.cursor_position_rel;
        let is_cursor_in = cursor_pos_option
            .map(|pos| is_position_inside_bounds(size, pos))
            .unwrap_or(false);

        if is_cursor_in {
            // Consume all input events to prevent interaction with underlying
            // components
            input.block_all();
        }
    }
}

fn apply_fluid_glass_block_input_modifier(base: Modifier, block_input: bool) -> Modifier {
    if block_input {
        base.push_pointer_input(FluidGlassBlockInputPointerModifierNode)
    } else {
        base
    }
}

struct GlassFeedbackPointer {
    active: State<Option<PointerId>>,
    ripple: State<RippleState>,
}

impl PointerInputModifierNode for GlassFeedbackPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        for change in input.pointer_changes.iter() {
            let active = self.active.get();
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    if active.is_none() && !change.is_consumed() =>
                {
                    let position = change
                        .position()
                        .map(|point| input.local_to_window.inverse_point(point))
                        .or(input.cursor_position_rel);
                    if let Some(position) = position.filter(|position| {
                        is_position_inside_bounds(input.computed_data, *position)
                    }) {
                        let size = input.computed_data;
                        self.active.set(Some(change.pointer_id));
                        start_glass_feedback(
                            self.ripple,
                            [
                                position.x / size.width.to_f32().max(1.0),
                                position.y / size.height.to_f32().max(1.0),
                            ],
                        );
                    }
                }
                CursorEventContent::Released(PressKeyEventType::Left)
                | CursorEventContent::Cancelled(PressKeyEventType::Left)
                    if active == Some(change.pointer_id) =>
                {
                    self.active.set(None);
                    self.ripple.with_mut(RippleState::release);
                }
                _ => {}
            }
        }
    }
}

fn start_glass_feedback(state: State<RippleState>, center: [f32; 2]) {
    state.with_mut(|ripple| {
        ripple.set_pressed(true);
        ripple.start_animation(center);
    });
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
/// - `shape` — optional glass shape. Regular and Clear default to a capsule.
/// - `material` — backdrop treatment, defaults to [`GlassMaterial::Regular`].
/// - `appearance` — override the inherited theme appearance; defaults to light without a theme.
/// - `blur_radius` — override the material's blur radius. Zero disables backdrop blur.
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
/// - `interactive` — enables optical press feedback. Defaults to true when `on_click` is set.
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
/// use tessera_components::{fluid_glass::{GlassMaterial, fluid_glass}, text::text};
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// fluid_glass().material(GlassMaterial::Clear).child(|| {
///     text("Content on glass");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn fluid_glass(
    tint_color: Option<Color>,
    shape: Option<Shape>,
    material: Option<GlassMaterial>,
    appearance: Option<GlassAppearance>,
    blur_radius: Option<Dp>,
    noise_amount: Option<f32>,
    noise_scale: Option<f32>,
    time: Option<f32>,
    contrast: Option<f32>,
    modifier: Option<Modifier>,
    padding: Option<Dp>,
    ripple_center: Option<[f32; 2]>,
    ripple_radius: Option<f32>,
    ripple_alpha: Option<f32>,
    ripple_strength: Option<f32>,
    on_click: Option<Callback>,
    interactive: Option<bool>,
    border: Option<GlassBorder>,
    block_input: Option<bool>,
    accessibility_role: Option<Role>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    accessibility_focusable: Option<bool>,
    child: Option<RenderSlot>,
) {
    let tint_color = tint_color.unwrap_or(Color::TRANSPARENT);
    let material = material.unwrap_or_default();
    let shape = shape.unwrap_or_else(|| match material {
        GlassMaterial::Regular | GlassMaterial::Clear => Shape::CAPSULE,
        GlassMaterial::Custom => default_glass_shape(),
    });
    let appearance = if let Some(appearance) = appearance {
        appearance
    } else if use_context::<MaterialTheme>().is_some_and(|theme| theme.get().color_scheme.is_dark) {
        GlassAppearance::Dark
    } else {
        GlassAppearance::Light
    };
    let optics = material.optics(appearance);
    let blur_radius = blur_radius.unwrap_or(optics.map_or(Dp(0.0), |(radius, _)| radius));
    let noise_amount = noise_amount.unwrap_or(0.0);
    let noise_scale = noise_scale.unwrap_or(1.0);
    let time = time.unwrap_or(0.0);
    let padding = padding.unwrap_or(Dp(0.0));
    let border = border.or_else(default_glass_border);
    let block_input = block_input.unwrap_or(false);
    let accessibility_focusable = accessibility_focusable.unwrap_or(false);
    let mut modifier = modifier.unwrap_or_default();
    let clickable = on_click.is_some();
    let feedback_enabled = interactive.unwrap_or(clickable);
    let focus_requester = remember(FocusRequester::new).get();
    let interaction_state = if clickable {
        Some(remember(InteractionState::new))
    } else {
        None
    };
    let ripple_state = if feedback_enabled {
        Some(remember(RippleState::new))
    } else {
        None
    };
    let has_semantics = accessibility_role.is_some()
        || accessibility_label.is_some()
        || accessibility_description.is_some()
        || accessibility_focusable;

    if let Some(on_click) = on_click {
        let press_handler = ripple_state.map(|state| {
            move |ctx: PointerEventContext| {
                start_glass_feedback(state, ctx.normalized_pos);
            }
        });
        let release_handler = ripple_state.map(|state| {
            move |_ctx: PointerEventContext| {
                state.with_mut(|s| s.release());
            }
        });
        let clickable_args = ClickableArgs {
            on_click,
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

        modifier = modifier.clickable(clickable_args);
    } else {
        if block_input {
            modifier = modifier.block_touch_propagation();
        }
        if let Some(ripple) = ripple_state {
            let active = remember(|| None::<PointerId>);
            modifier = modifier.push_pointer_input(GlassFeedbackPointer { active, ripple });
        }
    }
    if !clickable && has_semantics {
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
        material_tone: optics.map(|(_, tone)| tone),
        blur_sigma_px: blur_radius.to_pixels_f32().max(0.0),
        pixels_per_dp: Dp(1.0).to_pixels_f32(),
        opacity: 1.0,
        contrast,
        dark: appearance == GlassAppearance::Dark,
        feedback: 0.0,
    };

    layout(Stack).modifier(modifier).child(move || {
        fluid_glass_inner(render.clone())
            .padding(padding)
            .clickable(clickable)
            .block_input(block_input)
            .ripple_state_optional(ripple_state)
            .interaction_state_optional(interaction_state)
            .child_optional(child);
    });
}

#[tessera]
fn fluid_glass_inner(
    render: FluidGlassRenderArgs,
    padding: Option<Dp>,
    clickable: Option<bool>,
    block_input: Option<bool>,
    ripple_state: Option<State<RippleState>>,
    interaction_state: Option<State<InteractionState>>,
    child: Option<RenderSlot>,
) {
    let padding = padding.unwrap_or(Dp(0.0));
    let clickable = clickable.unwrap_or(false);
    let block_input = block_input.unwrap_or(false);
    let mut render = render;
    if let Some(ripple) = ripple_state {
        let (pressed, hovered) = if let Some(interaction) = interaction_state {
            interaction.with(|state| (state.is_pressed(), state.is_hovered()))
        } else {
            (ripple.with(RippleState::is_pressed), false)
        };
        render.feedback = if pressed {
            0.16
        } else if hovered {
            0.025
        } else {
            0.0
        };
    }
    let frame_nanos = current_frame_nanos();
    if let Some((progress, center)) = ripple_state.as_ref().and_then(|state| {
        state.with_mut(|ripple| {
            ripple
                .animation_at_frame_nanos(frame_nanos)
                .map(|animation| (animation.progress, animation.center))
        })
    }) {
        if let Some(ripple_state) = ripple_state {
            // Rebuilds this component on every animated frame.
            let frame_clock = tessera_ui::remember(tessera_ui::current_frame_nanos);
            let _ = frame_clock.get();
            tessera_ui::launch_once(async move {
                let step = move |frame_nanos: u64| {
                    let has_active_ripple = ripple_state
                        .with_mut(|ripple| ripple.animation_at_frame_nanos(frame_nanos).is_some());
                    if has_active_ripple { true } else { false }
                };
                loop {
                    let now = tessera_ui::next_frame().await;
                    frame_clock.set(now);
                    if !step(now) {
                        break;
                    }
                }
            });
        }

        render.ripple_center = Some(center);
        render.ripple_radius = Some(progress);
        render.ripple_alpha = Some((1.0 - progress) * 0.3);
        render.ripple_strength = Some(4.0 * progress * (1.0 - progress));
    }
    let modifier =
        apply_fluid_glass_block_input_modifier(Modifier::new(), !clickable && block_input);
    let policy = FluidGlassLayout {
        render: render.clone(),
        padding,
    };
    tessera_ui::layout::layout(policy.clone())
        .modifier(modifier)
        .draw(policy)
        .child(move || {
            if let Some(child) = child.as_ref() {
                child.render();
            }
        });
}

#[derive(Clone, PartialEq)]
struct FluidGlassLayout {
    render: FluidGlassRenderArgs,
    padding: Dp,
}

impl tessera_ui::LayoutPolicy for FluidGlassLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.padding == other.padding
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let parent_constraint = *input.parent_constraint().as_ref();

        let child_constraint = Constraint::new(
            remove_padding_from_constraint(parent_constraint.width, self.padding.into()),
            remove_padding_from_constraint(parent_constraint.height, self.padding.into()),
        );

        let children = input.children();
        let child_measurement = if let Some(&child) = children.first() {
            let child_measurement = child.measure(input, &child_constraint)?;
            result.place_child(
                child,
                PxPosition::from_f32_xy(self.padding.to_pixels_f32(), self.padding.to_pixels_f32()),
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

impl tessera_ui::DrawModifierNode for FluidGlassLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        let mut render = self.render.clone();
        if render.material_tone.is_none() {
            if render.blur_sigma_px > 0.0 {
                input.metadata_mut().fragment_mut().push_compute_command(
                    DualBlurCommand::horizontal_then_vertical(render.blur_sigma_px),
                );
                render.blur_sigma_px = 0.0;
            }
            if let Some(contrast) = render.contrast.take().filter(|value| *value != 1.0) {
                input
                    .metadata_mut()
                    .fragment_mut()
                    .push_compute_command(ContrastCommand::new(contrast));
            }
        }
        let drawable = FluidGlassCommand { render };

        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(drawable);
        content.draw(input);
    }
}
