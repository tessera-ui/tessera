//! Provides a flexible, customizable surface container component for UI elements.
//!
//! This module defines the [`surface`] component and its configuration via [`SurfaceArgs`].
//! The surface acts as a visual and interactive container, supporting background color,
//! shape, shadow, border, padding, and optional ripple effects for user interaction.
//!
//! Typical use cases include wrapping content to visually separate it from the background,
//! providing elevation or emphasis, and enabling interactive feedback (e.g., ripple on click).
//! It is commonly used as the foundational layer for buttons, dialogs, editors, and other
//! interactive or visually distinct UI elements.
//!
//! The surface can be configured for both static and interactive scenarios, with support for
//! hover and click callbacks, making it suitable for a wide range of UI composition needs.

use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType, Px,
    PxPosition, PxSize, tessera, winit::window::CursorIcon,
};

use crate::{
    padding_utils::remove_padding_from_dimension,
    pipelines::{RippleProps, ShadowProps, ShapeCommand},
    pos_misc::is_position_in_component,
    ripple_state::RippleState,
    shape_def::Shape,
};

/// Defines the visual style of the surface (fill, outline, or both).
#[derive(Clone)]
pub enum SurfaceStyle {
    /// A solid color fill.
    Filled { color: Color },
    /// A solid color outline with a transparent fill.
    Outlined { color: Color, width: Dp },
    /// A solid color fill with a solid color outline.
    FilledOutlined {
        fill_color: Color,
        border_color: Color,
        border_width: Dp,
    },
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        SurfaceStyle::Filled {
            color: Color::new(0.4745, 0.5255, 0.7961, 1.0),
        }
    }
}

impl From<Color> for SurfaceStyle {
    fn from(color: Color) -> Self {
        SurfaceStyle::Filled { color }
    }
}

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

    /// Geometric outline of the surface (rounded rectangle / ellipse / capsule variants).
    #[builder(default)]
    pub shape: Shape,

    /// Optional shadow/elevation style. When present it is passed through to the shape pipeline.
    #[builder(default, setter(strip_option))]
    pub shadow: Option<ShadowProps>,

    /// Internal padding applied symmetrically (left/right & top/bottom). Child content is
    /// positioned at (padding, padding). Also influences measured minimum size.
    #[builder(default = "Dp(0.0)")]
    pub padding: Dp,

    /// Explicit width constraint (Fixed / Wrap / Fill). Defaults to `Wrap`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,

    /// Explicit height constraint (Fixed / Wrap / Fill). Defaults to `Wrap`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,

    /// Optional click handler. Presence of this value makes the surface interactive:
    /// * Cursor changes to pointer when hovered
    /// * Press / release events are captured
    /// * Ripple animation starts on press if a `RippleState` is provided
    #[builder(default, setter(strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,

    /// Color of the ripple effect (if interactive & ripple state provided).
    #[builder(default = "Color::from_rgb(1.0, 1.0, 1.0)")]
    pub ripple_color: Color,

    /// If true, all input events inside the surface bounds are blocked (stop propagation),
    /// after (optionally) handling its own click logic.
    #[builder(default = "false")]
    pub block_input: bool,
}

impl Default for SurfaceArgs {
    fn default() -> Self {
        SurfaceArgsBuilder::default().build().unwrap()
    }
}

fn build_ripple_props(args: &SurfaceArgs, ripple_state: Option<&Arc<RippleState>>) -> RippleProps {
    if let Some(state) = ripple_state
        && let Some((progress, click_pos)) = state.get_animation_progress()
    {
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
    g2_k_value: f32,
    interactive: bool,
) -> ShapeCommand {
    match style {
        SurfaceStyle::Filled { color } => {
            if interactive {
                ShapeCommand::RippleRect {
                    color: *color,
                    corner_radii,
                    g2_k_value,
                    shadow: args.shadow,
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::Rect {
                    color: *color,
                    corner_radii,
                    g2_k_value,
                    shadow: args.shadow,
                }
            }
        }
        SurfaceStyle::Outlined { color, width } => {
            if interactive {
                ShapeCommand::RippleOutlinedRect {
                    color: *color,
                    corner_radii,
                    g2_k_value,
                    shadow: args.shadow,
                    border_width: width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::OutlinedRect {
                    color: *color,
                    corner_radii,
                    g2_k_value,
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
            if interactive {
                ShapeCommand::RippleFilledOutlinedRect {
                    color: *fill_color,
                    border_color: *border_color,
                    corner_radii,
                    g2_k_value,
                    shadow: args.shadow,
                    border_width: border_width.to_pixels_f32(),
                    ripple: ripple_props,
                }
            } else {
                ShapeCommand::FilledOutlinedRect {
                    color: *fill_color,
                    border_color: *border_color,
                    corner_radii,
                    g2_k_value,
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
    interactive: bool,
) -> ShapeCommand {
    let corner_marker = [-1.0, -1.0, -1.0, -1.0];
    match style {
        SurfaceStyle::Filled { color } => {
            if interactive {
                ShapeCommand::RippleRect {
                    color: *color,
                    corner_radii: corner_marker,
                    g2_k_value: 0.0,
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
            if interactive {
                ShapeCommand::RippleOutlinedRect {
                    color: *color,
                    corner_radii: corner_marker,
                    g2_k_value: 0.0,
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
    let interactive = args.on_click.is_some();

    match args.shape {
        Shape::RoundedRectangle {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
            g2_k_value,
        } => {
            let corner_radii = [
                top_left.to_pixels_f32(),
                top_right.to_pixels_f32(),
                bottom_right.to_pixels_f32(),
                bottom_left.to_pixels_f32(),
            ];
            build_rounded_rectangle_command(
                args,
                style,
                ripple_props,
                corner_radii,
                g2_k_value,
                interactive,
            )
        }
        Shape::Ellipse => build_ellipse_command(args, style, ripple_props, interactive),
        Shape::HorizontalCapsule => {
            let radius = size.height.to_f32() / 2.0;
            let corner_radii = [radius, radius, radius, radius];
            build_rounded_rectangle_command(
                args,
                style,
                ripple_props,
                corner_radii,
                2.0, // Use G1 curve for perfect circle
                interactive,
            )
        }
        Shape::VerticalCapsule => {
            let radius = size.width.to_f32() / 2.0;
            let corner_radii = [radius, radius, radius, radius];
            build_rounded_rectangle_command(
                args,
                style,
                ripple_props,
                corner_radii,
                2.0, // Use G1 curve for perfect circle
                interactive,
            )
        }
    }
}

fn make_surface_drawable(
    args: &SurfaceArgs,
    style: &SurfaceStyle,
    ripple_state: Option<&Arc<RippleState>>,
    size: PxSize,
) -> ShapeCommand {
    let ripple_props = build_ripple_props(args, ripple_state);
    build_shape_command(args, style, ripple_props, size)
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

/// Renders a styled rectangular (or elliptic / capsule) container and optionally
/// provides interactive click + ripple feedback.
///
/// # Behavior
/// * Child closure is executed first so that nested components are registered.
/// * Layout (`measure`) phase:
///   - Measures (optional) single child (if present) with padding removed from constraints
///   - Computes final size using `width` / `height` (Wrap / Fill / Fixed) merging parent constraints
///   - Pushes a shape draw command sized to computed width/height
/// * Interaction (`input_handler`) phase (only when `on_click` is `Some`):
///   - Tracks cursor containment
///   - Sets hover state on provided `RippleState`
///   - Starts ripple animation on mouse press
///   - Invokes `on_click` on mouse release inside bounds
///   - Optionally blocks further event propagation if `block_input` is true
/// * Non‑interactive variant only blocks events if `block_input` and cursor inside.
///
/// # Ripple
/// Ripple requires a `RippleState` (pass in `Some(Arc<RippleState>)`). Without it, the surface
/// still detects clicks but no animation is shown.
///
/// # Sizing
/// Effective minimum size = child size + `padding * 2` in each axis (if child exists).
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::{Dp, tessera, Color};
/// use tessera_ui_basic_components::{
///     surface::{surface, SurfaceArgsBuilder},
///     ripple_state::RippleState,
/// };
///
/// #[tessera]
/// fn example_box() {
///     let ripple = Arc::new(RippleState::new());
///     surface(
///         SurfaceArgsBuilder::default()
///             .padding(Dp(8.0))
///             .on_click(Arc::new(|| println!("Surface clicked")))
///             .build()
///             .unwrap(),
///         Some(ripple),
///         || {
///             // child content here
///         },
///     );
/// }
/// ```
#[tessera]
pub fn surface(args: SurfaceArgs, ripple_state: Option<Arc<RippleState>>, child: impl FnOnce()) {
    (child)();
    let ripple_state_for_measure = ripple_state.clone();
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

        let is_hovered = ripple_state_for_measure
            .as_ref()
            .map(|state| state.is_hovered())
            .unwrap_or(false);

        let effective_style = if is_hovered && args_measure_clone.hover_style.is_some() {
            args_measure_clone.hover_style.as_ref().unwrap()
        } else {
            &args_measure_clone.style
        };

        let padding_px: Px = args_measure_clone.padding.into();
        let (width, height) =
            compute_surface_size(effective_surface_constraint, child_measurement, padding_px);

        let drawable = make_surface_drawable(
            &args_measure_clone,
            effective_style,
            ripple_state_for_measure.as_ref(),
            PxSize::new(width, height),
        );

        input.metadata_mut().push_draw_command(drawable);

        Ok(ComputedData { width, height })
    }));

    if args.on_click.is_some() {
        let args_for_handler = args.clone();
        let state_for_handler = ripple_state;
        input_handler(Box::new(move |mut input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            if let Some(ref state) = state_for_handler {
                state.set_hovered(is_cursor_in_surface);
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
                    .filter(|event| {
                        matches!(
                            event.content,
                            CursorEventContent::Released(PressKeyEventType::Left)
                        )
                    })
                    .collect();

                if !press_events.is_empty()
                    && let (Some(cursor_pos), Some(state)) =
                        (cursor_pos_option, state_for_handler.as_ref())
                {
                    let normalized_x = (cursor_pos.x.to_f32() / size.width.to_f32()) - 0.5;
                    let normalized_y = (cursor_pos.y.to_f32() / size.height.to_f32()) - 0.5;

                    state.start_animation([normalized_x, normalized_y]);
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
