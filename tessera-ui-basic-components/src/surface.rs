//! A flexible container component with styling and interaction options.
//!
//! ## Usage
//!
//! Use as a base for buttons, cards, or any styled and interactive region.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, GestureState,
    InputHandlerInput, Modifier, PressKeyEventType, Px, PxPosition, PxSize, State,
    accesskit::{Action, Role},
    provide_context, remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    RippleProps, ShadowProps,
    alignment::Alignment,
    modifier::ModifierExt,
    pipelines::{shape::command::ShapeCommand, simple_rect::command::SimpleRectCommand},
    pos_misc::is_position_in_component,
    ripple_state::{RippleSpec, RippleState},
    shape_def::{ResolvedShape, RoundedCorner, Shape},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for},
};

#[derive(Clone, Copy, Debug)]
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

    /// Synthesizes a shadow style for the provided elevation.
    pub fn synthesize_shadow(elevation: Dp, scheme: &MaterialColorScheme) -> ShadowProps {
        let elevation_px = elevation.to_pixels_f32();
        let offset_y = (elevation_px * 0.5).clamp(1.0, 12.0);
        let smoothness = (elevation_px * 0.75).clamp(2.0, 24.0);
        ShadowProps {
            color: scheme.shadow.with_alpha(0.25),
            offset: [0.0, offset_y],
            smoothness,
        }
    }
}

/// Defines the visual style of the surface (fill, outline, or both).
#[derive(Clone)]
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
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
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

/// Arguments for the `surface` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SurfaceArgs {
    /// Optional modifier chain applied to the surface subtree.
    #[builder(default = "Modifier::new()")]
    pub modifier: Modifier,
    /// Defines the visual style of the surface (fill, outline, or both).
    #[builder(default)]
    pub style: SurfaceStyle,
    /// Geometric outline of the surface (rounded rectangle / ellipse / capsule
    /// variants).
    #[builder(default)]
    pub shape: Shape,
    /// Optional shadow/elevation style. When present it is passed through to
    /// the shape pipeline.
    #[builder(default, setter(strip_option))]
    pub shadow: Option<ShadowProps>,
    /// Optional elevation hint used to synthesize a shadow when `shadow` is not
    /// provided.
    ///
    /// This is a lightweight approximation intended to ease gradual migration
    /// towards Material 3 style APIs.
    #[builder(default, setter(strip_option))]
    pub shadow_elevation: Option<Dp>,
    /// Tonal elevation for surfaces that use the theme `surface` color.
    ///
    /// When the container color equals `MaterialColorScheme.surface`, a tint is
    /// overlaid to simulate Material 3 tonal elevation.
    #[builder(default = "Dp(0.0)")]
    pub tonal_elevation: Dp,
    /// Optional explicit content color override for descendants.
    ///
    /// When `None`, the surface derives its content color from the theme using
    /// [`content_color_for`].
    #[builder(default, setter(strip_option))]
    pub content_color: Option<Color>,
    /// Aligns child content within the surface bounds.
    #[builder(default)]
    pub content_alignment: Alignment,
    /// Whether this surface is enabled for user interaction.
    ///
    /// When disabled, it will not react to input, will not show hover/ripple
    /// feedback, and will expose a disabled state to accessibility services.
    #[builder(default = "true")]
    pub enabled: bool,
    /// Optional click handler. Presence of this value makes the surface
    /// interactive:
    ///
    /// * Cursor changes to pointer when hovered
    /// * Press / release events are captured
    /// * Ripple animation starts on press
    #[builder(default, setter(custom, strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Color of the ripple effect (used when interactive).
    #[builder(default = "use_context::<ContentColor>().get().current")]
    pub ripple_color: Color,
    /// Whether ripples are bounded to the surface shape.
    #[builder(default = "true")]
    pub ripple_bounded: bool,
    /// Optional explicit ripple radius for this surface.
    #[builder(default, setter(strip_option))]
    pub ripple_radius: Option<Dp>,
    /// Optional shared interaction state used to render state layers and
    /// ripples.
    ///
    /// This can be used to render visual feedback in one place while driving
    /// interactions from another.
    #[builder(default, setter(strip_option))]
    pub interaction_state: Option<State<RippleState>>,
    /// Whether to render the state-layer overlay for this surface.
    #[builder(default = "true")]
    pub show_state_layer: bool,
    /// Whether to render ripple animations for this surface.
    #[builder(default = "true")]
    pub show_ripple: bool,
    /// If true, all input events inside the surface bounds are blocked (stop
    /// propagation), after (optionally) handling its own click logic.
    #[builder(default = "false")]
    pub block_input: bool,
    /// Optional explicit accessibility role. Defaults to `Role::Button` when
    /// interactive.
    #[builder(default, setter(strip_option))]
    pub accessibility_role: Option<Role>,
    /// Optional label read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional description read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
    /// Whether this surface should be focusable even when not interactive.
    #[builder(default)]
    pub accessibility_focusable: bool,
}

impl SurfaceArgsBuilder {
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

impl Default for SurfaceArgs {
    fn default() -> Self {
        SurfaceArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
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

fn synthesize_shadow_for_elevation(elevation: Dp, scheme: &MaterialColorScheme) -> ShadowProps {
    SurfaceDefaults::synthesize_shadow(elevation, scheme)
}

fn build_ripple_props(args: &SurfaceArgs, ripple_state: Option<State<RippleState>>) -> RippleProps {
    if !args.show_ripple {
        return RippleProps::default();
    }
    let Some(ripple_state) = ripple_state else {
        return RippleProps::default();
    };

    if let Some(animation) = ripple_state.with_mut(|s| s.animation()) {
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
    args: &SurfaceArgs,
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
                    shadow: args.shadow,
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::Rect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                    shadow: args.shadow,
                }
            }
        }
        SurfaceStyle::Outlined { color, width } => {
            if use_ripple {
                ShapeCommand::RippleOutlinedRect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                    shadow: args.shadow,
                    border_width: width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::OutlinedRect {
                    color: *color,
                    corner_radii,
                    corner_g2,
                    shadow: args.shadow,
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
                    shadow: args.shadow,
                    border_width: border_width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::FilledOutlinedRect {
                    color: *fill_color,
                    border_color: *border_color,
                    corner_radii,
                    corner_g2,
                    shadow: args.shadow,
                    border_width: border_width.to_pixels_f32(),
                }
            }
        }
    }
}

fn build_ellipse_command(
    args: &SurfaceArgs,
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
                    shadow: args.shadow,
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::Ellipse {
                    color: *color,
                    shadow: args.shadow,
                }
            }
        }
        SurfaceStyle::Outlined { color, width } => {
            if use_ripple {
                ShapeCommand::RippleOutlinedRect {
                    color: *color,
                    corner_radii: corner_marker,
                    corner_g2: [0.0; 4],
                    shadow: args.shadow,
                    border_width: width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::OutlinedEllipse {
                    color: *color,
                    shadow: args.shadow,
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
                shadow: args.shadow,
                border_width: border_width.to_pixels_f32(),
            }
        }
    }
}

fn build_shape_command(
    args: &SurfaceArgs,
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
            args,
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
    args: &SurfaceArgs,
    style: &SurfaceStyle,
    ripple_state: Option<State<RippleState>>,
    size: PxSize,
) -> ShapeCommand {
    let ripple_props = build_ripple_props(args, ripple_state);
    build_shape_command(args, style, ripple_props, size)
}

fn try_build_simple_rect_command(
    args: &SurfaceArgs,
    style: &SurfaceStyle,
    ripple_state: Option<State<RippleState>>,
) -> Option<SimpleRectCommand> {
    if args.shadow.is_some() {
        return None;
    }
    if args.show_ripple && args.on_click.is_some() {
        return None;
    }
    if args.show_ripple
        && ripple_state
            .and_then(|state| state.with_mut(|s| s.animation()))
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
    fn clamp_wrap(min: Option<Px>, max: Option<Px>, min_measure: Px) -> Px {
        min.unwrap_or(Px(0))
            .max(min_measure)
            .min(max.unwrap_or(Px::MAX))
    }

    fn fill_value(min: Option<Px>, max: Px, min_measure: Px) -> Px {
        max.max(min_measure).max(min.unwrap_or(Px(0)))
    }

    let width = match effective_surface_constraint.width {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, child_measurement.width),
        DimensionValue::Fill {
            min,
            max: Some(max),
        } => fill_value(min, max, child_measurement.width),
        DimensionValue::Fill { .. } => {
            panic!(
                "Seems that you are trying to fill an infinite dimension, which is not allowed\nsurface width = Fill without max\nconstraint = {effective_surface_constraint:?}\nchild_measurement = {child_measurement:?}"
            )
        }
    };

    let height = match effective_surface_constraint.height {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, child_measurement.height),
        DimensionValue::Fill {
            min,
            max: Some(max),
        } => fill_value(min, max, child_measurement.height),
        DimensionValue::Fill { .. } => {
            panic!(
                "Seems that you are trying to fill an infinite dimension, which is not allowed\nsurface height = Fill without max\nconstraint = {effective_surface_constraint:?}\nchild_measurement = {child_measurement:?}"
            )
        }
    };

    (width, height)
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
/// - `args` — configures the surface's appearance, layout, and interaction; see
///   [`SurfaceArgs`].
/// - `child` — a closure that renders the content inside the surface.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::{Dp, Modifier};
/// use tessera_ui_basic_components::{
///     modifier::{ModifierExt, Padding},
///     surface::{SurfaceArgsBuilder, surface},
///     text::{TextArgsBuilder, text},
/// };
///
/// surface(
///     SurfaceArgsBuilder::default()
///         .modifier(Modifier::new().padding(Padding::all(Dp(16.0))))
///         .on_click(|| println!("Surface was clicked!"))
///         .build()
///         .unwrap(),
///     || {
///         text(
///             TextArgsBuilder::default()
///                 .text("Click me".to_string())
///                 .build()
///                 .expect("builder construction failed"),
///         );
///     },
/// );
/// # }
/// # component();
/// ```
pub fn surface(args: SurfaceArgs, child: impl FnOnce() + Send + Sync + 'static) {
    let modifier = if args.on_click.is_some() {
        args.modifier.minimum_interactive_component_size()
    } else {
        args.modifier
    };
    let mut args = args;
    args.modifier = Modifier::new();
    modifier.run(move || surface_inner(args, child));
}

#[tessera]
fn surface_inner(args: SurfaceArgs, child: impl FnOnce() + Send + Sync + 'static) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let parent_absolute_elevation = use_context::<AbsoluteTonalElevation>().get().current;
    let absolute_tonal_elevation = Dp(parent_absolute_elevation.0 + args.tonal_elevation.0);
    let inherited_content_color = use_context::<ContentColor>().get().current;
    let content_color = args.content_color.unwrap_or_else(|| match &args.style {
        SurfaceStyle::Filled { color } => {
            content_color_for(*color, &scheme).unwrap_or(inherited_content_color)
        }
        SurfaceStyle::FilledOutlined { fill_color, .. } => {
            content_color_for(*fill_color, &scheme).unwrap_or(inherited_content_color)
        }
        SurfaceStyle::Outlined { .. } => inherited_content_color,
    });
    let clickable = args.on_click.is_some();
    let interactive = args.enabled && clickable;
    let interaction_state = args
        .interaction_state
        .or_else(|| interactive.then(|| remember(RippleState::new)));

    provide_context(
        AbsoluteTonalElevation {
            current: absolute_tonal_elevation,
        },
        || {
            provide_context(
                ContentColor {
                    current: content_color,
                },
                || {
                    (child)();
                },
            );
        },
    );
    let args_measure = args.clone();
    let absolute_tonal_elevation_for_draw = absolute_tonal_elevation;

    measure(Box::new(move |input| {
        let mut args_for_draw = args_measure.clone();
        if args_for_draw.shadow.is_none()
            && let Some(elevation) = args_for_draw.shadow_elevation
            && elevation.0 > 0.0
        {
            args_for_draw.shadow = Some(synthesize_shadow_for_elevation(elevation, &scheme));
        }

        let effective_surface_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );

        let child_measurement = if !input.children_ids.is_empty() {
            let child_measurements = input.measure_children(
                input
                    .children_ids
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

        let state_layer_alpha = if args_measure.show_state_layer {
            interaction_state
                .as_ref()
                .map(|state| state.with(|s| s.state_layer_alpha()))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let effective_style = &args_measure.style;
        let effective_style = apply_tonal_elevation_to_style(
            effective_style,
            &scheme,
            absolute_tonal_elevation_for_draw,
        );
        let effective_style = if args_measure.show_state_layer {
            apply_state_layer_to_style(
                &effective_style,
                args_for_draw.ripple_color.with_alpha(1.0),
                state_layer_alpha,
            )
        } else {
            effective_style
        };

        let (width, height) = compute_surface_size(effective_surface_constraint, child_measurement);

        if !input.children_ids.is_empty() {
            let (extra_x, extra_y) = compute_content_offset(
                args_measure.content_alignment,
                width,
                height,
                child_measurement.width,
                child_measurement.height,
            );
            let origin = PxPosition {
                x: extra_x,
                y: extra_y,
            };
            for &child_id in input.children_ids.iter() {
                input.place_child(child_id, origin);
            }
        }

        let ripple_state_for_draw = if args_measure.show_ripple {
            interaction_state
        } else {
            None
        };

        if let Some(simple) =
            try_build_simple_rect_command(&args_for_draw, &effective_style, ripple_state_for_draw)
        {
            input.metadata_mut().push_draw_command(simple);
        } else {
            let drawable = make_surface_drawable(
                &args_for_draw,
                &effective_style,
                ripple_state_for_draw,
                PxSize::new(width, height),
            );

            input.metadata_mut().push_draw_command(drawable);
        }

        Ok(ComputedData { width, height })
    }));

    if clickable {
        let args = args;
        input_handler(Box::new(move |mut input| {
            // Apply accessibility metadata first.
            apply_surface_accessibility(
                &mut input,
                &args,
                true,
                args.enabled,
                args.on_click.clone(),
            );

            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            if interactive {
                if let Some(ref state) = interaction_state {
                    state.with_mut(|s| s.set_hovered(is_cursor_in_surface));
                }

                if input.cursor_events.iter().any(|event| {
                    matches!(
                        event.content,
                        CursorEventContent::Released(PressKeyEventType::Left)
                    )
                }) {
                    if let Some(ref state) = interaction_state {
                        state.with_mut(|s| s.release());
                    }
                }

                if is_cursor_in_surface {
                    input.requests.cursor_icon = CursorIcon::Pointer;
                }

                if is_cursor_in_surface {
                    let press_events: Vec<_> = input
                        .cursor_events
                        .iter()
                        .filter(|event| {
                            matches!(
                                event.content,
                                CursorEventContent::Pressed(PressKeyEventType::Left)
                            )
                        })
                        .collect();

                    let release_events: Vec<_> = input
                        .cursor_events
                        .iter()
                        .filter(|event| event.gesture_state == GestureState::TapCandidate)
                        .filter(|event| {
                            matches!(
                                event.content,
                                CursorEventContent::Released(PressKeyEventType::Left)
                            )
                        })
                        .collect();

                    if !press_events.is_empty()
                        && let Some(cursor_pos) = cursor_pos_option
                        && let Some(state) = interaction_state.as_ref()
                    {
                        let denom_w = size.width.to_f32().max(1.0);
                        let denom_h = size.height.to_f32().max(1.0);
                        let normalized_x = (cursor_pos.x.to_f32() / denom_w).clamp(0.0, 1.0);
                        let normalized_y = (cursor_pos.y.to_f32() / denom_h).clamp(0.0, 1.0);
                        let spec = RippleSpec {
                            bounded: args.ripple_bounded,
                            radius: args.ripple_radius,
                        };

                        state.with_mut(|s| {
                            s.start_animation_with_spec(
                                [normalized_x, normalized_y],
                                PxSize::new(size.width, size.height),
                                spec,
                            );
                            s.set_pressed(true);
                        });
                    }

                    if !release_events.is_empty()
                        && let Some(ref on_click) = args.on_click
                    {
                        on_click();
                    }

                    if args.block_input {
                        input.block_all();
                    }
                }
            } else if args.block_input && is_cursor_in_surface {
                input.block_all();
            }
        }));
    } else {
        let args = args;
        input_handler(Box::new(move |mut input| {
            // Apply accessibility metadata first
            apply_surface_accessibility(&mut input, &args, false, args.enabled, None);

            // Then handle input blocking if needed
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);
            if args.block_input && is_cursor_in_surface {
                input.block_all();
            }
        }));
    }
}

fn apply_surface_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &SurfaceArgs,
    interactive: bool,
    enabled: bool,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
) {
    let has_metadata = args.accessibility_role.is_some()
        || args.accessibility_label.is_some()
        || args.accessibility_description.is_some()
        || args.accessibility_focusable
        || interactive;

    if !has_metadata {
        return;
    }

    let mut builder = input.accessibility();

    let role = args
        .accessibility_role
        .or_else(|| interactive.then_some(Role::Button));
    if let Some(role) = role {
        builder = builder.role(role);
    }
    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }
    if let Some(description) = args.accessibility_description.as_ref() {
        builder = builder.description(description.clone());
    }
    if !enabled {
        builder = builder.disabled();
    } else {
        if args.accessibility_focusable || interactive {
            builder = builder.focusable();
        }
        if interactive {
            builder = builder.action(Action::Click);
        }
    }
    builder.commit();

    if enabled
        && interactive
        && let Some(on_click) = on_click
    {
        input.set_accessibility_action_handler(move |action| {
            if action == Action::Click {
                on_click();
            }
        });
    }
}
