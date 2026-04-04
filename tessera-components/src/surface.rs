//! A flexible container component with styling and interaction options.
//!
//! ## Usage
//!
//! Use as a base for buttons, cards, or any styled and interactive region.
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, Dp, FocusProperties, FocusRequester,
    MeasurementError, Modifier, PointerInput, PointerInputModifierNode, Px, PxPosition, PxSize,
    RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{
        LayoutInput, LayoutOutput, LayoutPolicy, RenderInput, RenderPolicy, layout_primitive,
    },
    modifier::ModifierCapabilityExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    RippleProps,
    alignment::Alignment,
    modifier::{
        ClickableArgs, InteractionState, ModifierExt, PointerEventContext, SemanticsArgs,
        ShadowArgs,
    },
    pipelines::{shape::command::ShapeCommand, simple_rect::command::SimpleRectCommand},
    pos_misc::is_position_inside_bounds,
    ripple_state::{RippleSpec, RippleState},
    shape_def::{ResolvedShape, RoundedCorner, Shape},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for},
};

#[derive(Clone, PartialEq, Copy, Debug)]
struct AbsoluteTonalElevation {
    current: Dp,
}

impl Default for AbsoluteTonalElevation {
    fn default() -> Self {
        Self { current: Dp(0.0) }
    }
}

/// Material Design 3 defaults for [`surface`].
pub struct SurfaceDefaults;

impl SurfaceDefaults {
    /// Default pressed ripple alpha used by surfaces.
    pub const RIPPLE_ALPHA: f32 = MaterialAlpha::PRESSED;

    /// Returns the standard ripple color for a surface.
    pub fn ripple_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface
    }

    /// Synthesize ambient and spot shadow layers for the given elevation.
    pub fn synthesize_shadow_layers(
        elevation: Dp,
        scheme: &MaterialColorScheme,
    ) -> crate::shadow::ShadowLayers {
        use crate::shadow::{ShadowLayer, ShadowLayers};
        let elevation_px = elevation.to_pixels_f32();
        let spot_offset_y = (elevation_px * 0.5).clamp(1.0, 12.0);
        let spot_smoothness = (elevation_px * 0.75).clamp(2.0, 24.0);
        let ambient_smoothness = (elevation_px * 1.0).clamp(4.0, 36.0);

        let spot = ShadowLayer {
            color: scheme.shadow.with_alpha(0.25),
            offset: [0.0, spot_offset_y],
            smoothness: spot_smoothness,
        };

        let ambient = ShadowLayer {
            color: scheme.shadow.with_alpha(0.14),
            offset: [0.0, 0.0],
            smoothness: ambient_smoothness,
        };

        ShadowLayers {
            ambient: Some(ambient),
            spot: Some(spot),
        }
    }
}

/// Defines the visual style of the surface (fill, outline, or both).
#[derive(Clone, PartialEq)]
pub enum SurfaceStyle {
    /// A solid color fill.
    Filled {
        /// Fill color used for the surface.
        color: Color,
    },
    /// A solid color outline with a transparent fill.
    Outlined {
        /// Outline color for the surface border.
        color: Color,
        /// Width of the outline stroke.
        width: Dp,
    },
    /// A solid color fill with a solid color outline.
    FilledOutlined {
        /// Fill color used for the surface.
        fill_color: Color,
        /// Outline color used to draw the border.
        border_color: Color,
        /// Width of the outline stroke.
        border_width: Dp,
    },
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        SurfaceStyle::Filled {
            color: scheme.surface,
        }
    }
}

impl From<Color> for SurfaceStyle {
    fn from(color: Color) -> Self {
        SurfaceStyle::Filled { color }
    }
}

impl SurfaceBuilder {
    /// Creates props from base args and a child render function.
    pub fn with_child(mut self, child: impl Fn() + Send + Sync + 'static) -> Self {
        self.props.child = Some(RenderSlot::new(child));
        self
    }

    pub(crate) fn set_ripple_state(&mut self, state: Option<State<RippleState>>) {
        self.props.ripple_state = state;
    }
}

#[derive(Clone, Default, PartialEq)]
struct SurfaceResolvedArgs {
    modifier: Modifier,
    style: SurfaceStyle,
    shape: Shape,
    elevation: Option<Dp>,
    tonal_elevation: Dp,
    content_color: Option<Color>,
    content_alignment: Alignment,
    enabled: bool,
    on_click: Option<Callback>,
    ripple_color: Color,
    ripple_bounded: bool,
    ripple_radius: Option<Dp>,
    interaction_state: Option<State<InteractionState>>,
    show_state_layer: bool,
    show_ripple: bool,
    ripple_state: Option<State<RippleState>>,
    block_input: bool,
    accessibility_role: Option<Role>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    accessibility_focusable: bool,
    focus_requester: Option<FocusRequester>,
    focus_properties: Option<FocusProperties>,
    child: Option<RenderSlot>,
}

fn compute_content_offset(
    alignment: Alignment,
    container_w: Px,
    container_h: Px,
    content_w: Px,
    content_h: Px,
) -> (Px, Px) {
    fn center_axis(container: Px, content: Px) -> Px {
        Px(((container.0 - content.0).max(0)) / 2)
    }

    match alignment {
        Alignment::TopStart => (Px(0), Px(0)),
        Alignment::TopCenter => (center_axis(container_w, content_w), Px(0)),
        Alignment::TopEnd => (Px((container_w.0 - content_w.0).max(0)), Px(0)),
        Alignment::CenterStart => (Px(0), center_axis(container_h, content_h)),
        Alignment::Center => (
            center_axis(container_w, content_w),
            center_axis(container_h, content_h),
        ),
        Alignment::CenterEnd => (
            Px((container_w.0 - content_w.0).max(0)),
            center_axis(container_h, content_h),
        ),
        Alignment::BottomStart => (Px(0), Px((container_h.0 - content_h.0).max(0))),
        Alignment::BottomCenter => (
            center_axis(container_w, content_w),
            Px((container_h.0 - content_h.0).max(0)),
        ),
        Alignment::BottomEnd => (
            Px((container_w.0 - content_w.0).max(0)),
            Px((container_h.0 - content_h.0).max(0)),
        ),
    }
}

fn apply_tonal_elevation_to_style(
    style: &SurfaceStyle,
    scheme: &MaterialColorScheme,
    absolute_tonal_elevation: Dp,
) -> SurfaceStyle {
    match style {
        SurfaceStyle::Filled { color } => SurfaceStyle::Filled {
            color: scheme.surface_color_at_elevation_for(*color, absolute_tonal_elevation),
        },
        SurfaceStyle::FilledOutlined {
            fill_color,
            border_color,
            border_width,
        } => SurfaceStyle::FilledOutlined {
            fill_color: scheme
                .surface_color_at_elevation_for(*fill_color, absolute_tonal_elevation),
            border_color: *border_color,
            border_width: *border_width,
        },
        SurfaceStyle::Outlined { .. } => style.clone(),
    }
}

fn build_ripple_props(
    args: &SurfaceResolvedArgs,
    ripple_state: Option<State<RippleState>>,
    frame_nanos: u64,
) -> RippleProps {
    if !args.show_ripple {
        return RippleProps::default();
    }
    let Some(ripple_state) = ripple_state else {
        return RippleProps::default();
    };

    if let Some(animation) =
        ripple_state.with(|state| state.animation_snapshot_at_frame_nanos(frame_nanos))
    {
        return RippleProps {
            center: [animation.center[0] - 0.5, animation.center[1] - 0.5],
            bounded: args.ripple_bounded,
            radius: animation.radius,
            alpha: animation.alpha,
            color: args.ripple_color.with_alpha(1.0),
        };
    }
    RippleProps::default()
}

fn apply_state_layer_to_style(style: &SurfaceStyle, color: Color, alpha: f32) -> SurfaceStyle {
    if alpha <= 0.0 {
        return style.clone();
    }

    match style {
        SurfaceStyle::Filled { color: fill_color } => SurfaceStyle::Filled {
            color: fill_color.blend_over(color, alpha),
        },
        SurfaceStyle::Outlined {
            color: border_color,
            width,
        } => SurfaceStyle::FilledOutlined {
            fill_color: Color::TRANSPARENT.blend_over(color, alpha),
            border_color: *border_color,
            border_width: *width,
        },
        SurfaceStyle::FilledOutlined {
            fill_color,
            border_color,
            border_width,
        } => SurfaceStyle::FilledOutlined {
            fill_color: fill_color.blend_over(color, alpha),
            border_color: *border_color,
            border_width: *border_width,
        },
    }
}

fn build_rounded_rectangle_command(
    style: &SurfaceStyle,
    ripple_props: RippleProps,
    corner_radii: [f32; 4],
    corner_g2: [f32; 4],
    use_ripple: bool,
) -> ShapeCommand {
    match style {
        SurfaceStyle::Filled { color } => {
            if use_ripple {
                ShapeCommand::RippleRect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::Rect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                }
            }
        }
        SurfaceStyle::Outlined { color, width } => {
            if use_ripple {
                ShapeCommand::RippleOutlinedRect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                    border_width: width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::OutlinedRect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                    border_width: width.to_pixels_f32(),
                }
            }
        }
        SurfaceStyle::FilledOutlined {
            fill_color,
            border_color,
            border_width,
        } => {
            if use_ripple {
                ShapeCommand::RippleFilledOutlinedRect {
                    color: *fill_color,
                    border_color: *border_color,
                    corner_radii,
                    corner_g2,
                    border_width: border_width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::FilledOutlinedRect {
                    color: *fill_color,
                    border_color: *border_color,
                    corner_radii,
                    corner_g2,
                    border_width: border_width.to_pixels_f32(),
                }
            }
        }
    }
}

fn build_ellipse_command(
    _args: &SurfaceResolvedArgs,
    style: &SurfaceStyle,
    ripple_props: RippleProps,
    use_ripple: bool,
) -> ShapeCommand {
    let corner_marker = [-1.0, -1.0, -1.0, -1.0];
    match style {
        SurfaceStyle::Filled { color } => {
            if use_ripple {
                ShapeCommand::RippleRect {
                    color: *color,
                    corner_radii: corner_marker,
                    corner_g2: [0.0; 4],
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::Ellipse { color: *color }
            }
        }
        SurfaceStyle::Outlined { color, width } => {
            if use_ripple {
                ShapeCommand::RippleOutlinedRect {
                    color: *color,
                    corner_radii: corner_marker,
                    corner_g2: [0.0; 4],
                    border_width: width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::OutlinedEllipse {
                    color: *color,
                    border_width: width.to_pixels_f32(),
                }
            }
        }
        SurfaceStyle::FilledOutlined {
            fill_color,
            border_color,
            border_width,
        } => {
            // NOTE: No ripple variant for FilledOutlinedEllipse yet.
            ShapeCommand::FilledOutlinedEllipse {
                color: *fill_color,
                border_color: *border_color,
                border_width: border_width.to_pixels_f32(),
            }
        }
    }
}

fn build_shape_command(
    args: &SurfaceResolvedArgs,
    style: &SurfaceStyle,
    ripple_props: RippleProps,
    size: PxSize,
) -> ShapeCommand {
    let use_ripple = args.show_ripple && (args.on_click.is_some() || ripple_props.alpha > 0.0);

    match args.shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => build_rounded_rectangle_command(
            style,
            ripple_props,
            corner_radii,
            corner_g2,
            use_ripple,
        ),
        ResolvedShape::Ellipse => build_ellipse_command(args, style, ripple_props, use_ripple),
    }
}

fn make_surface_drawable(
    args: &SurfaceResolvedArgs,
    style: &SurfaceStyle,
    ripple_state: Option<State<RippleState>>,
    frame_nanos: u64,
    size: PxSize,
) -> ShapeCommand {
    let ripple_props = build_ripple_props(args, ripple_state, frame_nanos);
    build_shape_command(args, style, ripple_props, size)
}

fn try_build_simple_rect_command(
    args: &SurfaceResolvedArgs,
    style: &SurfaceStyle,
    ripple_state: Option<State<RippleState>>,
    frame_nanos: u64,
) -> Option<SimpleRectCommand> {
    if args.show_ripple && args.on_click.is_some() {
        return None;
    }
    if args.show_ripple
        && ripple_state
            .and_then(|state| {
                state.with(|ripple| ripple.animation_snapshot_at_frame_nanos(frame_nanos))
            })
            .is_some()
    {
        return None;
    }

    let color = match style {
        SurfaceStyle::Filled { color } => *color,
        _ => return None,
    };

    match args.shape {
        Shape::RoundedRectangle {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
            ..
        } => {
            let corners = [top_left, top_right, bottom_right, bottom_left];
            if corners
                .iter()
                .any(|corner| matches!(corner, RoundedCorner::Capsule))
            {
                return None;
            }

            let zero_eps = 0.0001;
            if corners.iter().all(|corner| match corner {
                RoundedCorner::Manual { radius, .. } => radius.to_pixels_f32().abs() <= zero_eps,
                RoundedCorner::Capsule => false,
            }) {
                Some(SimpleRectCommand { color })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn compute_surface_size(
    effective_surface_constraint: Constraint,
    child_measurement: ComputedData,
) -> (Px, Px) {
    let width = effective_surface_constraint
        .width
        .clamp(child_measurement.width);
    let height = effective_surface_constraint
        .height
        .clamp(child_measurement.height);

    (width, height)
}

#[derive(Clone)]
struct SurfaceLayout {
    args: SurfaceResolvedArgs,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
    scheme: MaterialColorScheme,
    absolute_tonal_elevation: Dp,
}

impl PartialEq for SurfaceLayout {
    fn eq(&self, other: &Self) -> bool {
        self.args.content_alignment == other.args.content_alignment
    }
}

impl LayoutPolicy for SurfaceLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let effective_surface_constraint = *input.parent_constraint().as_ref();

        let child_measurement = if !input.children_ids().is_empty() {
            let child_measurements = input.measure_children(
                input
                    .children_ids()
                    .iter()
                    .copied()
                    .map(|node_id| (node_id, effective_surface_constraint))
                    .collect(),
            )?;
            let mut max_width = Px::ZERO;
            let mut max_height = Px::ZERO;
            for measurement in child_measurements.values() {
                max_width = max_width.max(measurement.width);
                max_height = max_height.max(measurement.height);
            }
            ComputedData {
                width: max_width,
                height: max_height,
            }
        } else {
            ComputedData {
                width: Px(0),
                height: Px(0),
            }
        };

        let (width, height) = compute_surface_size(effective_surface_constraint, child_measurement);

        if !input.children_ids().is_empty() {
            let (extra_x, extra_y) = compute_content_offset(
                self.args.content_alignment,
                width,
                height,
                child_measurement.width,
                child_measurement.height,
            );
            let origin = PxPosition {
                x: extra_x,
                y: extra_y,
            };
            for &child_id in input.children_ids().iter() {
                output.place_child(child_id, origin);
            }
        }

        Ok(ComputedData { width, height })
    }
}

impl RenderPolicy for SurfaceLayout {
    fn record(&self, input: &RenderInput<'_>) {
        let state_layer_alpha = if self.args.show_state_layer {
            self.interaction_state
                .as_ref()
                .map(|state| state.with(|s| s.state_layer_alpha()))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let mut effective_style = apply_tonal_elevation_to_style(
            &self.args.style,
            &self.scheme,
            self.absolute_tonal_elevation,
        );
        if self.args.show_state_layer {
            effective_style = apply_state_layer_to_style(
                &effective_style,
                self.args.ripple_color.with_alpha(1.0),
                state_layer_alpha,
            );
        }

        let ripple_state_for_draw = if self.args.show_ripple {
            self.ripple_state
        } else {
            None
        };
        let frame_nanos = current_frame_nanos();

        let mut metadata = input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("Surface node must have computed size before record");

        if let Some(simple) = try_build_simple_rect_command(
            &self.args,
            &effective_style,
            ripple_state_for_draw,
            frame_nanos,
        ) {
            metadata.fragment_mut().push_draw_command(simple);
        } else {
            let drawable = make_surface_drawable(
                &self.args,
                &effective_style,
                ripple_state_for_draw,
                frame_nanos,
                PxSize::new(size.width, size.height),
            );

            metadata.fragment_mut().push_draw_command(drawable);
        }
    }
}

struct SurfaceBlockInputPointerModifierNode;

impl PointerInputModifierNode for SurfaceBlockInputPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        let is_cursor_in_surface = input
            .cursor_position_rel
            .map(|pos| is_position_inside_bounds(input.computed_data, pos))
            .unwrap_or(false);
        if is_cursor_in_surface {
            input.block_all();
        }
    }
}

fn apply_surface_block_input_modifier(base: Modifier, block_input: bool) -> Modifier {
    if block_input {
        base.push_pointer_input(SurfaceBlockInputPointerModifierNode)
    } else {
        base
    }
}

#[tessera]
fn surface_content(
    resolved: Option<SurfaceResolvedArgs>,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
) {
    let surface = resolved.expect("surface_content requires resolved args");
    let child = surface.child;
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let parent_absolute_elevation = use_context::<AbsoluteTonalElevation>()
        .map(|e| e.get().current)
        .unwrap_or_else(|| AbsoluteTonalElevation::default().current);
    let absolute_tonal_elevation = Dp(parent_absolute_elevation.0 + surface.tonal_elevation.0);
    let inherited_content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or_else(|| ContentColor::default().current);
    let content_color = surface
        .content_color
        .unwrap_or_else(|| match &surface.style {
            SurfaceStyle::Filled { color } => {
                content_color_for(*color, &scheme).unwrap_or(inherited_content_color)
            }
            SurfaceStyle::FilledOutlined { fill_color, .. } => {
                content_color_for(*fill_color, &scheme).unwrap_or(inherited_content_color)
            }
            SurfaceStyle::Outlined { .. } => inherited_content_color,
        });
    let clickable = surface.on_click.is_some();
    let interactive = surface.enabled && clickable;

    if surface.show_ripple
        && let Some(ripple_state) = ripple_state
    {
        let has_active_ripple = ripple_state.with(|state| {
            state
                .animation_snapshot_at_frame_nanos(current_frame_nanos())
                .is_some()
        });
        if has_active_ripple {
            receive_frame_nanos(move |frame_nanos| {
                let has_active_ripple = ripple_state.with(|state| {
                    state
                        .animation_snapshot_at_frame_nanos(frame_nanos)
                        .is_some()
                });
                if has_active_ripple {
                    tessera_ui::FrameNanosControl::Continue
                } else {
                    tessera_ui::FrameNanosControl::Stop
                }
            });
        }
    }

    let layout_args = surface.clone();
    let modifier =
        apply_surface_block_input_modifier(Modifier::new(), !interactive && surface.block_input);
    let policy = SurfaceLayout {
        args: layout_args,
        interaction_state,
        ripple_state,
        scheme,
        absolute_tonal_elevation,
    };
    layout_primitive()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            provide_context(
                || AbsoluteTonalElevation {
                    current: absolute_tonal_elevation,
                },
                || {
                    provide_context(
                        || ContentColor {
                            current: content_color,
                        },
                        || {
                            if let Some(child) = child.as_ref() {
                                child.render();
                            }
                        },
                    );
                },
            );
        });
}

/// # surface
///
/// Renders a styled container for content with optional interaction.
///
/// ## Usage
///
/// Wrap content to provide a visual background, shape, and optional click
/// handling with a ripple effect.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the surface subtree.
/// - `style` — optional visual style of the surface.
/// - `shape` — optional geometric outline of the surface.
/// - `elevation` — optional shadow elevation.
/// - `tonal_elevation` — optional tonal elevation tint for theme surfaces.
/// - `content_color` — optional explicit content color override.
/// - `content_alignment` — optional child alignment within the surface bounds.
/// - `enabled` — optional interactive enabled flag.
/// - `on_click` — optional click callback.
/// - `ripple_color` — optional ripple color.
/// - `ripple_bounded` — optional bounded-ripple flag.
/// - `ripple_radius` — optional explicit ripple radius.
/// - `interaction_state` — optional shared interaction state.
/// - `show_state_layer` — optional state-layer visibility flag.
/// - `show_ripple` — optional ripple visibility flag.
/// - `ripple_state` — optional shared ripple state.
/// - `block_input` — optional input-blocking flag.
/// - `accessibility_role` — optional accessibility role.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `accessibility_focusable` — optional accessibility focusable flag.
/// - `focus_requester` — optional externally managed focus requester.
/// - `focus_properties` — optional focus properties.
/// - `child` — optional child render slot.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     modifier::{ModifierExt, Padding},
///     surface::surface,
///     text::text,
/// };
/// use tessera_ui::{Dp, Modifier};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// surface()
///     .modifier(Modifier::new().padding(Padding::all(Dp(16.0))))
///     .on_click(|| println!("Surface was clicked!"))
///     .child(|| {
///         text().content("Click me");
///     });
/// # });
/// # }
/// # component();
/// ```
/// Renders a styled surface container.
#[tessera]
pub fn surface(
    modifier: Option<Modifier>,
    style: Option<SurfaceStyle>,
    shape: Option<Shape>,
    elevation: Option<Dp>,
    tonal_elevation: Option<Dp>,
    content_color: Option<Color>,
    content_alignment: Option<Alignment>,
    enabled: Option<bool>,
    on_click: Option<Callback>,
    ripple_color: Option<Color>,
    ripple_bounded: Option<bool>,
    ripple_radius: Option<Dp>,
    interaction_state: Option<State<InteractionState>>,
    show_state_layer: Option<bool>,
    show_ripple: Option<bool>,
    ripple_state: Option<State<RippleState>>,
    block_input: Option<bool>,
    accessibility_role: Option<Role>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    accessibility_focusable: Option<bool>,
    focus_requester: Option<FocusRequester>,
    focus_properties: Option<FocusProperties>,
    child: Option<RenderSlot>,
) {
    let theme = use_context::<MaterialTheme>();
    let scheme = theme
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let resolved = SurfaceResolvedArgs {
        modifier: modifier.unwrap_or_default(),
        style: style.unwrap_or_default(),
        shape: shape.unwrap_or_default(),
        elevation,
        tonal_elevation: tonal_elevation.unwrap_or(Dp(0.0)),
        content_color,
        content_alignment: content_alignment.unwrap_or_default(),
        enabled: enabled.unwrap_or(true),
        on_click,
        ripple_color: ripple_color
            .or_else(|| use_context::<ContentColor>().map(|c| c.get().current))
            .unwrap_or(scheme.on_surface),
        ripple_bounded: ripple_bounded.unwrap_or(true),
        ripple_radius,
        interaction_state,
        show_state_layer: show_state_layer.unwrap_or(true),
        show_ripple: show_ripple.unwrap_or(true),
        ripple_state,
        block_input: block_input.unwrap_or(false),
        accessibility_role,
        accessibility_label,
        accessibility_description,
        accessibility_focusable: accessibility_focusable.unwrap_or(false),
        focus_requester,
        focus_properties,
        child,
    };
    let mut modifier = resolved.modifier.clone();
    let clickable = resolved.on_click.is_some();
    let interactive = resolved.enabled && clickable;
    let internal_focus_requester = remember(FocusRequester::new).get();
    let bound_focus_requester = resolved.focus_requester.unwrap_or(internal_focus_requester);
    let interaction_state = resolved
        .interaction_state
        .or_else(|| interactive.then(|| remember(InteractionState::new)));
    let ripple_state = if resolved.show_ripple {
        resolved
            .ripple_state
            .or_else(|| interactive.then(|| remember(RippleState::new)))
    } else {
        None
    };
    let has_semantics = resolved.accessibility_role.is_some()
        || resolved.accessibility_label.is_some()
        || resolved.accessibility_description.is_some()
        || resolved.accessibility_focusable;

    if clickable {
        modifier = modifier.minimum_interactive_component_size();
    }

    if interactive {
        let ripple_spec = RippleSpec {
            bounded: resolved.ripple_bounded,
            radius: resolved.ripple_radius,
        };
        let press_handler = ripple_state.map(|state| {
            let spec = ripple_spec;
            move |ctx: PointerEventContext| {
                state.with_mut(|s| {
                    s.start_animation_with_spec(ctx.normalized_pos, ctx.size, spec);
                });
            }
        });
        let release_handler = ripple_state
            .map(|state| move |_ctx: PointerEventContext| state.with_mut(|s| s.release()));
        let clickable_args = ClickableArgs {
            on_click: resolved
                .on_click
                .expect("interactive implies on_click is set"),
            enabled: resolved.enabled,
            block_input: resolved.block_input,
            on_press: press_handler.map(Into::into),
            on_release: release_handler.map(Into::into),
            role: resolved.accessibility_role,
            label: resolved.accessibility_label.clone(),
            description: resolved.accessibility_description.clone(),
            interaction_state,
            focus_requester: Some(bound_focus_requester),
            focus_properties: resolved.focus_properties,
        };

        modifier = modifier.clickable_with(clickable_args);
    } else if resolved.block_input {
        modifier = modifier.block_touch_propagation();
    }

    if !interactive && has_semantics {
        let semantics = SemanticsArgs {
            role: resolved.accessibility_role,
            label: resolved.accessibility_label.clone(),
            description: resolved.accessibility_description.clone(),
            focusable: resolved.accessibility_focusable,
            disabled: !resolved.enabled,
            ..Default::default()
        };
        modifier = modifier.semantics(semantics);
    }

    if let Some(elevation) = resolved.elevation
        && elevation.0 > 0.0
    {
        modifier = modifier.shadow(&ShadowArgs {
            elevation,
            shape: resolved.shape,
            clip: false,
            ..Default::default()
        });
    }

    layout_primitive().modifier(modifier).child(move || {
        let mut builder = surface_content().resolved(resolved.clone());
        if let Some(interaction_state) = interaction_state {
            builder = builder.interaction_state(interaction_state);
        }
        if let Some(ripple_state) = ripple_state {
            builder = builder.ripple_state(ripple_state);
        }
        drop(builder);
    });
}
