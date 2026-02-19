//! A flexible container component with styling and interaction options.
//!
//! ## Usage
//!
//! Use as a base for buttons, cards, or any styled and interactive region.
use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px,
    PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    provide_context, remember, tessera, use_context, with_frame_nanos,
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

/// Arguments for the `surface` component.
#[derive(PartialEq, Clone, Setters)]
pub struct SurfaceArgs {
    /// Optional modifier chain applied to the surface subtree.
    pub modifier: Modifier,
    /// Defines the visual style of the surface (fill, outline, or both).
    pub style: SurfaceStyle,
    /// Geometric outline of the surface (rounded rectangle / ellipse / capsule
    /// variants).
    pub shape: Shape,
    /// Elevation of the surface.
    ///
    /// This value determines the shadow cast by the surface and its tonal
    /// elevation (if the color is `surface`).
    #[setters(strip_option)]
    pub elevation: Option<Dp>,
    /// Tonal elevation for surfaces that use the theme `surface` color.
    ///
    /// When the container color equals `MaterialColorScheme.surface`, a tint is
    /// overlaid to simulate Material 3 tonal elevation.
    pub tonal_elevation: Dp,
    /// Optional explicit content color override for descendants.
    ///
    /// When `None`, the surface derives its content color from the theme using
    /// [`content_color_for`].
    #[setters(strip_option)]
    pub content_color: Option<Color>,
    /// Aligns child content within the surface bounds.
    pub content_alignment: Alignment,
    /// Whether this surface is enabled for user interaction.
    ///
    /// When disabled, it will not react to input, will not show hover/ripple
    /// feedback, and will expose a disabled state to accessibility services.
    pub enabled: bool,
    /// Optional click handler. Presence of this value makes the surface
    /// interactive:
    ///
    /// * Cursor changes to pointer when hovered
    /// * Press / release events are captured
    /// * Ripple animation starts on press
    #[setters(skip)]
    pub on_click: Option<Callback>,
    /// Color of the ripple effect (used when interactive).
    pub ripple_color: Color,
    /// Whether ripples are bounded to the surface shape.
    pub ripple_bounded: bool,
    /// Optional explicit ripple radius for this surface.
    #[setters(strip_option)]
    pub ripple_radius: Option<Dp>,
    /// Optional shared interaction state used to render state layers.
    ///
    /// This can be used to render visual feedback in one place while driving
    /// interactions from another.
    #[setters(strip_option)]
    pub interaction_state: Option<State<InteractionState>>,
    /// Whether to render the state-layer overlay for this surface.
    pub show_state_layer: bool,
    /// Whether to render ripple animations for this surface.
    pub show_ripple: bool,
    /// Optional ripple animation state used for rendering ripples.
    #[setters(strip_option)]
    pub ripple_state: Option<State<RippleState>>,
    /// If true, all input events inside the surface bounds are blocked (stop
    /// propagation), after (optionally) handling its own click logic.
    pub block_input: bool,
    /// Optional explicit accessibility role. Defaults to `Role::Button` when
    /// interactive.
    #[setters(strip_option)]
    pub accessibility_role: Option<Role>,
    /// Optional label read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional description read by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
    /// Whether this surface should be focusable even when not interactive.
    pub accessibility_focusable: bool,
    /// Optional child render slot.
    #[setters(skip)]
    pub child: Option<RenderSlot>,
}

impl SurfaceArgs {
    /// Creates props from base args and a child render function.
    pub fn with_child(args: SurfaceArgs, child: impl Fn() + Send + Sync + 'static) -> Self {
        args.child(child)
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Callback::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: impl Into<Callback>) -> Self {
        self.on_click = Some(on_click.into());
        self
    }

    /// Sets the child render slot.
    pub fn child<F>(mut self, child: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.child = Some(RenderSlot::new(child));
        self
    }

    /// Sets the child render slot using a shared callback.
    pub fn child_shared(mut self, child: impl Into<RenderSlot>) -> Self {
        self.child = Some(child.into());
        self
    }
}

impl SurfaceArgs {
    pub(crate) fn set_ripple_state(&mut self, state: Option<State<RippleState>>) {
        self.ripple_state = state;
    }
}

impl Default for SurfaceArgs {
    fn default() -> Self {
        let theme = use_context::<MaterialTheme>();
        Self {
            modifier: Modifier::new(),
            style: SurfaceStyle::default(),
            shape: Shape::default(),
            elevation: None,
            tonal_elevation: Dp(0.0),
            content_color: None,
            content_alignment: Alignment::default(),
            enabled: true,
            on_click: None,
            ripple_color: use_context::<ContentColor>()
                .map(|c| c.get().current)
                .or_else(|| theme.map(|t| t.get().color_scheme.on_surface))
                .unwrap_or_else(|| ContentColor::default().current),
            ripple_bounded: true,
            ripple_radius: None,
            interaction_state: None,
            show_state_layer: true,
            show_ripple: true,
            ripple_state: None,
            block_input: false,
            accessibility_role: None,
            accessibility_label: None,
            accessibility_description: None,
            accessibility_focusable: false,
            child: None,
        }
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

fn build_ripple_props(args: &SurfaceArgs, ripple_state: Option<State<RippleState>>) -> RippleProps {
    if !args.show_ripple {
        return RippleProps::default();
    }
    let Some(ripple_state) = ripple_state else {
        return RippleProps::default();
    };

    if let Some(animation) = ripple_state.with(|s| s.animation_snapshot()) {
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
    _args: &SurfaceArgs,
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
    _args: &SurfaceArgs,
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
    if args.show_ripple && args.on_click.is_some() {
        return None;
    }
    if args.show_ripple
        && ripple_state
            .and_then(|state| state.with(|s| s.animation_snapshot()))
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

#[derive(Clone)]
struct SurfaceLayout {
    args: SurfaceArgs,
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

impl LayoutSpec for SurfaceLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let effective_surface_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

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

        let mut metadata = input.metadata_mut();
        let size = metadata
            .computed_data
            .expect("Surface node must have computed size before record");

        if let Some(simple) =
            try_build_simple_rect_command(&self.args, &effective_style, ripple_state_for_draw)
        {
            metadata.fragment_mut().push_draw_command(simple);
        } else {
            let drawable = make_surface_drawable(
                &self.args,
                &effective_style,
                ripple_state_for_draw,
                PxSize::new(size.width, size.height),
            );

            metadata.fragment_mut().push_draw_command(drawable);
        }
    }
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
/// - `args` — props for this component; see [`SurfaceArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     modifier::{ModifierExt, Padding},
///     surface::{SurfaceArgs, surface},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{Dp, Modifier};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(|| MaterialTheme::default(), || {
/// let args = SurfaceArgs::default()
///     .modifier(Modifier::new().padding(Padding::all(Dp(16.0))))
///     .on_click(|| println!("Surface was clicked!"))
///     .child(|| {
///         text(&TextArgs::default().text("Click me"));
///     });
/// surface(&args);
/// # });
/// # material_theme(&args);
/// # }
/// # component();
/// ```
/// Renders a styled surface container.
#[tessera]
pub fn surface(args: &SurfaceArgs) {
    let args = args.clone();
    let child = args.child.clone();
    let mut modifier = args.modifier.clone();
    let clickable = args.on_click.is_some();
    let interactive = args.enabled && clickable;
    let interaction_state = args
        .interaction_state
        .or_else(|| interactive.then(|| remember(InteractionState::new)));
    let ripple_state = if args.show_ripple {
        args.ripple_state
            .or_else(|| interactive.then(|| remember(RippleState::new)))
    } else {
        None
    };
    let has_semantics = args.accessibility_role.is_some()
        || args.accessibility_label.is_some()
        || args.accessibility_description.is_some()
        || args.accessibility_focusable;

    if clickable {
        modifier = modifier.minimum_interactive_component_size();
    }

    if interactive {
        let ripple_spec = RippleSpec {
            bounded: args.ripple_bounded,
            radius: args.ripple_radius,
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
        let mut clickable_args = ClickableArgs::new(
            args.on_click
                .clone()
                .expect("interactive implies on_click is set"),
        )
        .enabled(args.enabled)
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
        if let Some(description) = args.accessibility_description.clone() {
            semantics = semantics.description(description);
        }
        if args.accessibility_focusable {
            semantics = semantics.focusable(true);
        }
        if !args.enabled {
            semantics = semantics.disabled(true);
        }
        modifier = modifier.semantics(semantics);
    }

    if let Some(elevation) = args.elevation
        && elevation.0 > 0.0
    {
        modifier = modifier.shadow(&ShadowArgs::new(elevation).shape(args.shape).clip(false));
    }

    let inner_args = SurfaceInnerArgs {
        surface: args.clone(),
        interaction_state,
        ripple_state,
        child,
    };

    modifier.run(move || {
        let inner_args = inner_args.clone();
        surface_inner(&inner_args);
    });
}

#[tessera]
fn surface_inner(args: &SurfaceInnerArgs) {
    let surface = &args.surface;
    let interaction_state = args.interaction_state;
    let ripple_state = args.ripple_state;

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
        let has_active_ripple = ripple_state.with(|s| s.animation_snapshot().is_some());
        if has_active_ripple {
            with_frame_nanos(move |_| {
                ripple_state.with_mut(|_| {});
            });
        }
    }

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
                    if let Some(child) = args.child.as_ref() {
                        child.render();
                    }
                },
            );
        },
    );

    let layout_args = surface.clone();
    layout(SurfaceLayout {
        args: layout_args,
        interaction_state,
        ripple_state,
        scheme,
        absolute_tonal_elevation,
    });

    if !interactive && surface.block_input {
        input_handler(move |mut input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_inside_bounds(size, pos))
                .unwrap_or(false);
            if is_cursor_in_surface {
                input.block_all();
            }
        });
    }
}

#[derive(Clone, PartialEq)]
struct SurfaceInnerArgs {
    surface: SurfaceArgs,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
    child: Option<RenderSlot>,
}
