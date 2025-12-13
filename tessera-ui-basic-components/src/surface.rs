//! A flexible container component with styling and interaction options.
//!
//! ## Usage
//!
//! Use as a base for buttons, cards, or any styled and interactive region.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, GestureState,
    InputHandlerInput, PressKeyEventType, Px, PxPosition, PxSize, State,
    accesskit::{Action, Role},
    provide_context, remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    RippleProps, ShadowProps,
    padding_utils::remove_padding_from_dimension,
    pipelines::{shape::command::ShapeCommand, simple_rect::command::SimpleRectCommand},
    pos_misc::is_position_in_component,
    ripple_state::RippleState,
    shape_def::{ResolvedShape, RoundedCorner, Shape},
    theme::{ContentColor, MaterialColorScheme, content_color_for},
};

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
        let scheme = use_context::<MaterialColorScheme>().get();
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
    /// Defines the visual style of the surface (fill, outline, or both).
    #[builder(default)]
    pub style: SurfaceStyle,
    /// Optional style to apply when the cursor is hovering over the surface.
    /// This is only active when `on_click` is also provided.
    #[builder(default)]
    pub hover_style: Option<SurfaceStyle>,
    /// Geometric outline of the surface (rounded rectangle / ellipse / capsule
    /// variants).
    #[builder(default)]
    pub shape: Shape,
    /// Optional shadow/elevation style. When present it is passed through to
    /// the shape pipeline.
    #[builder(default, setter(strip_option))]
    pub shadow: Option<ShadowProps>,
    /// Internal padding applied symmetrically (left/right & top/bottom). Child
    /// content is positioned at (padding, padding). Also influences
    /// measured minimum size.
    #[builder(default = "Dp(0.0)")]
    pub padding: Dp,
    /// Explicit width constraint (Fixed / Wrap / Fill). Defaults to `Wrap`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,
    /// Explicit height constraint (Fixed / Wrap / Fill). Defaults to `Wrap`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
    /// Optional click handler. Presence of this value makes the surface
    /// interactive:
    ///
    /// * Cursor changes to pointer when hovered
    /// * Press / release events are captured
    /// * Ripple animation starts on press
    #[builder(default, setter(custom, strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Color of the ripple effect (used when interactive).
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface.with_alpha(0.12)")]
    pub ripple_color: Color,
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

fn build_ripple_props(args: &SurfaceArgs, ripple_state: Option<State<RippleState>>) -> RippleProps {
    let Some(ripple_state) = ripple_state else {
        return RippleProps::default();
    };

    if let Some((progress, click_pos)) = ripple_state.with_mut(|s| s.get_animation_progress()) {
        let radius = progress;
        let alpha = (1.0 - progress) * 0.3;
        return RippleProps {
            center: click_pos,
            radius,
            alpha,
            color: args.ripple_color,
        };
    }
    RippleProps::default()
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
    let use_ripple = args.on_click.is_some();

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
    if args.on_click.is_some() {
        return None;
    }
    if ripple_state
        .and_then(|state| state.with_mut(|s| s.get_animation_progress()))
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
    padding_px: Px,
) -> (Px, Px) {
    let min_width = child_measurement.width + padding_px * 2;
    let min_height = child_measurement.height + padding_px * 2;

    fn clamp_wrap(min: Option<Px>, max: Option<Px>, min_measure: Px) -> Px {
        min.unwrap_or(Px(0))
            .max(min_measure)
            .min(max.unwrap_or(Px::MAX))
    }

    fn fill_value(min: Option<Px>, max: Option<Px>, min_measure: Px) -> Px {
        max.expect("Seems that you are trying to fill an infinite dimension, which is not allowed")
            .max(min_measure)
            .max(min.unwrap_or(Px(0)))
    }

    let width = match effective_surface_constraint.width {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, min_width),
        DimensionValue::Fill { min, max } => fill_value(min, max, min_width),
    };

    let height = match effective_surface_constraint.height {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, min_height),
        DimensionValue::Fill { min, max } => fill_value(min, max, min_height),
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
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::{
///     surface::{SurfaceArgsBuilder, surface},
///     text::{TextArgsBuilder, text},
/// };
///
/// surface(
///     SurfaceArgsBuilder::default()
///         .padding(Dp(16.0))
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
#[tessera]
pub fn surface(args: SurfaceArgs, child: impl FnOnce()) {
    let scheme = use_context::<MaterialColorScheme>().get();
    let inherited_content_color = use_context::<ContentColor>().get().current;
    let content_color = match &args.style {
        SurfaceStyle::Filled { color } => content_color_for(*color, &scheme),
        SurfaceStyle::FilledOutlined { fill_color, .. } => content_color_for(*fill_color, &scheme),
        SurfaceStyle::Outlined { .. } => inherited_content_color,
    };

    provide_context(
        ContentColor {
            current: content_color,
        },
        || {
            (child)();
        },
    );
    let ripple_state = args.on_click.as_ref().map(|_| remember(RippleState::new));
    let args_measure_clone = args.clone();
    let args_for_handler = args.clone();

    measure(Box::new(move |input| {
        let surface_intrinsic_width = args_measure_clone.width;
        let surface_intrinsic_height = args_measure_clone.height;
        let surface_intrinsic_constraint =
            Constraint::new(surface_intrinsic_width, surface_intrinsic_height);
        let effective_surface_constraint =
            surface_intrinsic_constraint.merge(input.parent_constraint);
        let padding_px: Px = args_measure_clone.padding.into();
        let child_constraint = Constraint::new(
            remove_padding_from_dimension(effective_surface_constraint.width, padding_px),
            remove_padding_from_dimension(effective_surface_constraint.height, padding_px),
        );

        let child_measurement = if !input.children_ids.is_empty() {
            let child_measurements = input.measure_children(
                input
                    .children_ids
                    .iter()
                    .copied()
                    .map(|node_id| (node_id, child_constraint))
                    .collect(),
            )?;
            input.place_child(
                input.children_ids[0],
                PxPosition {
                    x: args.padding.into(),
                    y: args.padding.into(),
                },
            );
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

        let is_hovered = ripple_state
            .as_ref()
            .map(|state| state.with(|s| s.is_hovered()))
            .unwrap_or(false);

        let effective_style = args_measure_clone
            .hover_style
            .as_ref()
            .filter(|_| is_hovered)
            .unwrap_or(&args_measure_clone.style);

        let padding_px: Px = args_measure_clone.padding.into();
        let (width, height) =
            compute_surface_size(effective_surface_constraint, child_measurement, padding_px);

        if let Some(simple) =
            try_build_simple_rect_command(&args_measure_clone, effective_style, ripple_state)
        {
            input.metadata_mut().push_draw_command(simple);
        } else {
            let drawable = make_surface_drawable(
                &args_measure_clone,
                effective_style,
                ripple_state,
                PxSize::new(width, height),
            );

            input.metadata_mut().push_draw_command(drawable);
        }

        Ok(ComputedData { width, height })
    }));

    if args.on_click.is_some() {
        let args_for_handler = args.clone();
        input_handler(Box::new(move |mut input| {
            // Apply accessibility metadata first
            apply_surface_accessibility(
                &mut input,
                &args_for_handler,
                true,
                args_for_handler.on_click.clone(),
            );

            // Then handle interactive behavior
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            if let Some(ref state) = ripple_state {
                state.with_mut(|s| s.set_hovered(is_cursor_in_surface));
            }

            if is_cursor_in_surface && args_for_handler.on_click.is_some() {
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
                    && let Some(state) = ripple_state.as_ref()
                {
                    let normalized_x = (cursor_pos.x.to_f32() / size.width.to_f32()) - 0.5;
                    let normalized_y = (cursor_pos.y.to_f32() / size.height.to_f32()) - 0.5;

                    state.with_mut(|s| s.start_animation([normalized_x, normalized_y]));
                }

                if !release_events.is_empty()
                    && let Some(ref on_click) = args_for_handler.on_click
                {
                    on_click();
                }

                if args_for_handler.block_input {
                    input.block_all();
                }
            }
        }));
    } else {
        input_handler(Box::new(move |mut input| {
            // Apply accessibility metadata first
            apply_surface_accessibility(&mut input, &args_for_handler, false, None);

            // Then handle input blocking if needed
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);
            if args_for_handler.block_input && is_cursor_in_surface {
                input.block_all();
            }
        }));
    }
}

fn apply_surface_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &SurfaceArgs,
    interactive: bool,
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
    if args.accessibility_focusable || interactive {
        builder = builder.focusable();
    }
    if interactive {
        builder = builder.action(Action::Click);
    }
    builder.commit();

    if interactive && let Some(on_click) = on_click {
        input.set_accessibility_action_handler(move |action| {
            if action == Action::Click {
                on_click();
            }
        });
    }
}
