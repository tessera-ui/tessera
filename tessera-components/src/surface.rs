//! A flexible container component with styling and interaction options.
//!
//! ## Usage
//!
//! Use as a base for buttons, cards, or any styled and interactive region.
use std::sync::Arc;

use parking_lot::Mutex;
use tessera_foundation::gesture::{LongPressRecognizer, LongPressSettings, TapRecognizer};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, Color, ComputedData, Constraint, Dp,
    FocusProperties, FocusRequester, LayoutResult, MeasurementError, Modifier, PointerInput,
    PointerInputModifierNode, Px, PxPosition, PxSize, RenderSlot, SemanticsModifierNode, State,
    accesskit::{Role, Toggled},
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    RippleProps,
    alignment::Alignment,
    modifier::{
        ClickableArgs, InteractionState, InteractionVisualState, ModifierExt, PointerEventContext,
        ShadowArgs, apply_combined_clickable_modifier,
    },
    pipelines::{shape::command::ShapeCommand, simple_rect::command::SimpleRectCommand},
    pos_misc::is_position_inside_bounds,
    ripple_state::{RippleSpec, RippleState},
    shape_def::{ResolvedShape, RoundedCorner, Shape},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for},
};

#[cfg(all(test, not(target_family = "wasm")))]
mod ripple_tests;

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

    /// Returns the ambient and directional shadow profiles for an elevation.
    ///
    /// Elevations interpolate between the standard 0–12dp levels. Use the
    /// shadow modifier to draw these profiles with the ambient spread.
    pub fn synthesize_shadow_layers(
        elevation: Dp,
        scheme: &MaterialColorScheme,
    ) -> crate::shadow::ShadowLayers {
        crate::shadow::material_shadow(elevation, scheme).0
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
        SurfaceStyle::Filled {
            color: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl From<Color> for SurfaceStyle {
    fn from(color: Color) -> Self {
        SurfaceStyle::Filled { color }
    }
}

/// A deterministic circular ripple for a pressed visual presentation.
///
/// This sample affects drawing only. It does not dispatch input, activate the
/// surface, or start a ripple animation.
///
/// ```rust
/// use tessera_components::{
///     button::button,
///     modifier::{InteractionState, InteractionVisualState},
///     surface::RipplePresentation,
///     theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn sample() {
///     let live = remember(InteractionState::new);
///     material_theme().child(move || {
///         button()
///             .interaction_state(live)
///             .presentation_state(InteractionVisualState::Pressed)
///             .presentation_ripple(RipplePresentation {
///                 origin: [0.75, 1.0],
///                 radius: 0.7,
///                 opacity: 0.1,
///             });
///         assert_eq!(live.get(), InteractionState::new());
///     });
/// }
/// sample();
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RipplePresentation {
    /// Origin relative to the surface: `[0, 0]` is the top-left corner and
    /// `[1, 1]` is the bottom-right corner. Origins may lie outside the
    /// surface. Non-finite coordinates use the center; coordinates are
    /// bounded to `[-1024, 1024]`.
    pub origin: [f32; 2],
    /// Radius in multiples of the surface's shorter dimension, bounded to
    /// `[0, 1024]`. Non-finite or negative radii produce no ripple.
    pub radius: f32,
    /// Ripple opacity in `[0, 1]`. Non-finite or negative opacity produces no
    /// ripple.
    pub opacity: f32,
}

impl RipplePresentation {
    fn sanitized(self) -> Self {
        let coordinate = |value: f32| {
            if value.is_finite() {
                value.clamp(-1024.0, 1024.0)
            } else {
                0.5
            }
        };
        Self {
            origin: self.origin.map(coordinate),
            radius: if self.radius.is_finite() {
                self.radius.clamp(0.0, 1024.0)
            } else {
                0.0
            },
            opacity: if self.opacity.is_finite() {
                self.opacity.clamp(0.0, 1.0)
            } else {
                0.0
            },
        }
    }

    fn props(self, color: Color, bounded: bool) -> RippleProps {
        let presentation = self.sanitized();
        RippleProps {
            center: [presentation.origin[0] - 0.5, presentation.origin[1] - 0.5],
            bounded,
            radius: presentation.radius,
            alpha: presentation.opacity,
            color: color.with_alpha(1.0),
        }
    }
}

#[derive(Clone, Default, PartialEq)]
struct SurfaceResolvedArgs {
    modifier: Modifier,
    style: SurfaceStyle,
    shape: Shape,
    clip_content: bool,
    elevation: Option<Dp>,
    tonal_elevation: Dp,
    content_color: Option<Color>,
    content_alignment: Alignment,
    enabled: bool,
    on_click: Option<Callback>,
    on_long_click: Option<Callback>,
    ripple_color: Color,
    ripple_bounded: bool,
    ripple_radius: Option<Dp>,
    interaction_state: Option<State<InteractionState>>,
    presentation_state: Option<InteractionVisualState>,
    presentation_ripple: Option<RipplePresentation>,
    show_focus_indicator: bool,
    show_state_layer: bool,
    show_ripple: bool,
    ripple_state: Option<State<RippleState>>,
    block_input: bool,
    accessibility_role: Option<Role>,
    accessibility_toggled: Option<Toggled>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    focus_requester: Option<FocusRequester>,
    focus_properties: Option<FocusProperties>,
    child: Option<RenderSlot>,
}

#[derive(Clone, Copy, PartialEq)]
struct SurfaceInputOwner {
    interaction: Option<State<InteractionState>>,
    ripple: Option<State<RippleState>>,
    tap: Option<State<TapRecognizer>>,
    long_press: Option<State<LongPressRecognizer>>,
}

impl SurfaceInputOwner {
    fn release(self) {
        if let Some(state) = self.interaction
            && state.is_alive()
        {
            state.with_mut(|state| {
                state.release();
                state.set_hovered(false);
                state.set_focused(false);
                state.set_dragged(false);
            });
        }
        if let Some(state) = self.ripple
            && state.is_alive()
        {
            state.with_mut(RippleState::release);
        }
        if let Some(state) = self.tap
            && state.is_alive()
        {
            state.with_mut(TapRecognizer::cancel);
        }
        if let Some(state) = self.long_press
            && state.is_alive()
        {
            state.with_mut(LongPressRecognizer::cancel);
        }
    }
}

struct SurfaceInputLease(Mutex<SurfaceInputOwner>);

impl Drop for SurfaceInputLease {
    fn drop(&mut self) {
        let input = *self.0.get_mut();
        input.release();
    }
}

fn compute_content_offset(
    alignment: Alignment,
    container_w: Px,
    container_h: Px,
    content_w: Px,
    content_h: Px,
) -> (f32, f32) {
    fn center_axis(container: Px, content: Px) -> f32 {
        ((container.0 - content.0).max(0)) as f32 / 2.0
    }

    match alignment {
        Alignment::TopStart => (0.0, 0.0),
        Alignment::TopCenter => (center_axis(container_w, content_w), 0.0),
        Alignment::TopEnd => ((container_w.0 - content_w.0).max(0) as f32, 0.0),
        Alignment::CenterStart => (0.0, center_axis(container_h, content_h)),
        Alignment::Center => (
            center_axis(container_w, content_w),
            center_axis(container_h, content_h),
        ),
        Alignment::CenterEnd => (
            (container_w.0 - content_w.0).max(0) as f32,
            center_axis(container_h, content_h),
        ),
        Alignment::BottomStart => (0.0, (container_h.0 - content_h.0).max(0) as f32),
        Alignment::BottomCenter => (
            center_axis(container_w, content_w),
            (container_h.0 - content_h.0).max(0) as f32,
        ),
        Alignment::BottomEnd => (
            (container_w.0 - content_w.0).max(0) as f32,
            (container_h.0 - content_h.0).max(0) as f32,
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
    let use_ripple = args.show_ripple && ripple_props.alpha > 0.0;

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
    ripple_props: RippleProps,
    size: PxSize,
) -> ShapeCommand {
    build_shape_command(args, style, ripple_props, size)
}

fn try_build_simple_rect_command(
    args: &SurfaceResolvedArgs,
    style: &SurfaceStyle,
    ripple_props: RippleProps,
) -> Option<SimpleRectCommand> {
    if args.show_ripple && ripple_props.alpha > 0.0 {
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
                RoundedCorner::Capsule | RoundedCorner::Relative { .. } => false,
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
    parent_constraint: Constraint,
    child_measurement: ComputedData,
) -> (Px, Px) {
    let width = parent_constraint.width.clamp(child_measurement.width);
    let height = parent_constraint.height.clamp(child_measurement.height);

    (width, height)
}

#[derive(Clone)]
struct SurfaceLayout {
    args: SurfaceResolvedArgs,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
    scheme: MaterialColorScheme,
    absolute_tonal_elevation: Dp,
    ripple_frame_nanos: Option<u64>,
    focus_visible: bool,
}

impl SurfaceLayout {
    fn has_foreground(&self) -> bool {
        !matches!(self.args.style, SurfaceStyle::Filled { .. })
            || (self.args.show_state_layer
                && (self.interaction_state.is_some() || self.args.presentation_state.is_some()))
            || (self.args.show_focus_indicator
                && (self.interaction_state.is_some() || self.args.presentation_state.is_some()))
            || (self.args.show_ripple
                && (self.args.on_click.is_some()
                    || self.ripple_state.is_some()
                    || (self.args.presentation_state == Some(InteractionVisualState::Pressed)
                        && self.args.presentation_ripple.is_some())))
    }
}

impl PartialEq for SurfaceLayout {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args
            && self.interaction_state == other.interaction_state
            && self.ripple_state == other.ripple_state
            && self.ripple_frame_nanos == other.ripple_frame_nanos
            && self.focus_visible == other.focus_visible
            && self.scheme == other.scheme
            && self.absolute_tonal_elevation == other.absolute_tonal_elevation
    }
}

impl tessera_ui::LayoutModifierNode for SurfaceLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.has_foreground() == other.has_foreground()
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other) && self.args.content_alignment == other.args.content_alignment
    }

    fn place_children(
        &self,
        scope: &tessera_ui::layout::PlacementScope<'_>,
        _inner: &tessera_ui::layout::PlacementResult,
    ) -> Option<tessera_ui::layout::PlacementResult> {
        let children = scope.children();
        let (foreground, children) = if self.has_foreground() {
            let (foreground, children) = children.split_last()?;
            (Some(*foreground), children)
        } else {
            (None, children.as_slice())
        };
        let size = scope.size();
        let child_width = children
            .iter()
            .map(|child| child.size().width)
            .max()
            .unwrap_or(Px::ZERO);
        let child_height = children
            .iter()
            .map(|child| child.size().height)
            .max()
            .unwrap_or(Px::ZERO);
        let (x, y) = compute_content_offset(
            self.args.content_alignment,
            size.width,
            size.height,
            child_width,
            child_height,
        );
        let mut result = LayoutResult::new(size);
        if let Some(foreground) = foreground {
            result.place_child(foreground, PxPosition::ZERO);
        }
        for &child in children {
            result.place_child(child, PxPosition::from_f32_xy(x, y));
        }
        Some(result.into_placement_result())
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let parent_constraint = *input.parent_constraint().as_ref();
        let child_constraint = input.parent_constraint().without_min();
        let children = input.children();
        let (foreground, children) = if self.has_foreground() {
            let (foreground, children) = children.split_last().expect("surface foreground");
            (Some(*foreground), children)
        } else {
            (None, children.as_slice())
        };

        let child_measurement = if !children.is_empty() {
            let mut max_width = Px::ZERO;
            let mut max_height = Px::ZERO;
            for &child in children {
                let measurement = child.measure(input, &child_constraint)?;
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

        let (width, height) = compute_surface_size(parent_constraint, child_measurement);
        if let Some(foreground) = foreground {
            foreground.measure(
                input,
                &Constraint::new(
                    tessera_ui::AxisConstraint::exact(width),
                    tessera_ui::AxisConstraint::exact(height),
                ),
            )?;
            result.place_child(foreground, PxPosition::ZERO);
        }

        if !children.is_empty() {
            let (extra_x, extra_y) = compute_content_offset(
                self.args.content_alignment,
                width,
                height,
                child_measurement.width,
                child_measurement.height,
            );
            let origin = PxPosition::from_f32_xy(extra_x, extra_y);
            for &child in children {
                result.place_child(child, origin);
            }
        }

        Ok(result.with_size(ComputedData { width, height }))
    }
}

impl tessera_ui::DrawModifierNode for SurfaceLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        let effective_style = apply_tonal_elevation_to_style(
            &self.args.style,
            &self.scheme,
            self.absolute_tonal_elevation,
        );
        let background = match effective_style {
            SurfaceStyle::Filled { color } => Some(SurfaceStyle::Filled { color }),
            SurfaceStyle::FilledOutlined { fill_color, .. } => {
                Some(SurfaceStyle::Filled { color: fill_color })
            }
            SurfaceStyle::Outlined { .. } => None,
        };
        if let Some(background) = background {
            record_surface_style(input, &self.args, &background, RippleProps::default());
        }
        content.draw(input);
    }
}

fn record_surface_style(
    input: &mut tessera_ui::layout::RenderInput<'_>,
    args: &SurfaceResolvedArgs,
    style: &SurfaceStyle,
    ripple: RippleProps,
) {
    let mut metadata = input.metadata_mut();
    let size = metadata.computed_data().expect("surface draw bounds");
    if let Some(simple) = try_build_simple_rect_command(args, style, ripple) {
        metadata.fragment_mut().push_draw_command(simple);
    } else {
        let drawable =
            make_surface_drawable(args, style, ripple, PxSize::new(size.width, size.height));
        metadata.fragment_mut().push_draw_command(drawable);
    }
}

#[derive(Clone, PartialEq)]
struct SurfaceForeground(SurfaceLayout);

impl tessera_ui::DrawModifierNode for SurfaceForeground {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let policy = &self.0;
        let live = policy
            .interaction_state
            .map(|state| state.get())
            .unwrap_or_default();
        let visual = live.with_presentation(policy.args.presentation_state, policy.args.enabled);
        let alpha = if policy.args.enabled && policy.args.show_state_layer {
            visual.state_layer_alpha()
        } else {
            0.0
        };
        let mut ripple = if policy.args.enabled {
            policy
                .ripple_frame_nanos
                .map(|frame_nanos| {
                    build_ripple_props(&policy.args, policy.ripple_state, frame_nanos)
                })
                .unwrap_or_default()
        } else {
            RippleProps::default()
        };
        if policy.args.enabled
            && policy.args.show_ripple
            && live == InteractionState::new()
            && policy.args.presentation_state == Some(InteractionVisualState::Pressed)
            && let Some(presentation) = policy.args.presentation_ripple
        {
            ripple = presentation.props(policy.args.ripple_color, policy.args.ripple_bounded);
        }
        let style = match policy.args.style {
            SurfaceStyle::Filled { .. } => SurfaceStyle::Filled {
                color: Color::TRANSPARENT,
            },
            SurfaceStyle::Outlined { color, width } => SurfaceStyle::FilledOutlined {
                fill_color: Color::TRANSPARENT,
                border_color: color,
                border_width: width,
            },
            SurfaceStyle::FilledOutlined {
                border_color,
                border_width,
                ..
            } => SurfaceStyle::FilledOutlined {
                fill_color: Color::TRANSPARENT,
                border_color,
                border_width,
            },
        };
        if alpha > 0.0 || ripple.alpha > 0.0 || !matches!(style, SurfaceStyle::Filled { .. }) {
            let style =
                apply_state_layer_to_style(&style, policy.args.ripple_color.with_alpha(1.0), alpha);
            record_surface_style(ctx.render_input, &policy.args, &style, ripple);
        }
        if policy.args.show_focus_indicator
            && visual.is_focused()
            && (!live.is_focused() || policy.focus_visible)
        {
            record_focus_indicator(ctx.render_input, policy.args.shape, policy.scheme.secondary);
        }
        content.draw(ctx.render_input);
    }
}

pub(crate) fn record_focus_indicator(
    input: &mut tessera_ui::layout::RenderInput<'_>,
    shape: Shape,
    color: Color,
) {
    let mut metadata = input.metadata_mut();
    let bounds = metadata
        .computed_data()
        .expect("surface focus indicator bounds");
    let (command, size, position) =
        focus_indicator_geometry(shape, PxSize::new(bounds.width, bounds.height), color);
    let fragment = metadata.fragment_mut();
    let index = fragment.push_draw_command(command);
    let op = &mut fragment.ops_mut()[index as usize];
    op.position_override = Some(position);
    op.size_override = Some(size);
}

fn focus_indicator_geometry(
    shape: Shape,
    body_size: PxSize,
    color: Color,
) -> (ShapeCommand, PxSize, PxPosition) {
    let expansion = Dp(10.0).to_px();
    let outset = expansion.to_f32() / 2.0;
    let size = PxSize::new(body_size.width + expansion, body_size.height + expansion);
    let command = match shape.resolve_for_size(body_size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::OutlinedRect {
            color,
            corner_radii: corner_radii.map(|radius| radius + outset),
            corner_g2,
            border_width: Dp(3.0).to_pixels_f32(),
        },
        ResolvedShape::Ellipse => ShapeCommand::OutlinedEllipse {
            color,
            border_width: Dp(3.0).to_pixels_f32(),
        },
    };
    (command, size, PxPosition::from_f32_xy(-outset, -outset))
}

#[tessera]
fn surface_foreground(paint: Option<State<SurfaceLayout>>) {
    let paint = paint.expect("surface foreground paint").get();
    if paint.has_foreground() {
        layout().draw(SurfaceForeground(paint));
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
            input.consume_pointer_changes();
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
    let focus_visible = surface.show_focus_indicator
        && surface
            .focus_requester
            .is_some_and(|requester| requester.is_focus_visible());
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
    let clickable = surface.on_click.is_some() || surface.on_long_click.is_some();
    let interactive = surface.enabled && clickable;

    let mut ripple_frame_nanos = None;
    if surface.show_ripple
        && let Some(ripple_state) = ripple_state
    {
        let frame_nanos = current_frame_nanos();
        let has_active_ripple = ripple_state.with(|state| {
            state
                .animation_snapshot_at_frame_nanos(frame_nanos)
                .is_some()
        });
        if has_active_ripple {
            ripple_frame_nanos = Some(frame_nanos);
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
    let mut modifier =
        apply_surface_block_input_modifier(Modifier::new(), !interactive && surface.block_input);
    if let Some(elevation) = surface.elevation
        && elevation.0 > 0.0
    {
        modifier = modifier.shadow(
            &scheme,
            &ShadowArgs {
                elevation,
                shape: surface.shape,
                clip: false,
                ..Default::default()
            },
        );
    }
    let policy = SurfaceLayout {
        args: layout_args,
        interaction_state,
        ripple_state,
        scheme,
        absolute_tonal_elevation,
        ripple_frame_nanos,
        focus_visible,
    };
    let current_context = (absolute_tonal_elevation, content_color, policy.args.child);
    let context = remember(|| current_context);
    if context.with(|value| value != &current_context) {
        context.set(current_context);
    }
    let current = policy;
    let live = remember(|| current.clone());
    if live.with(|value| value != &current) {
        live.set(current);
    }
    layout().modifier(modifier).child(move || {
        let policy = live.get();
        let modifier = if policy.args.clip_content {
            Modifier::new().clip_shape(policy.args.shape)
        } else {
            Modifier::new()
        };
        layout()
            .modifier(modifier)
            .policy(policy.clone())
            .draw(policy)
            .child(move || {
                let (absolute_tonal_elevation, content_color, child) = context.get();
                provide_context(
                    || AbsoluteTonalElevation {
                        current: absolute_tonal_elevation,
                    },
                    || {
                        let elevation = use_context::<AbsoluteTonalElevation>().unwrap();
                        if elevation.with(|value| value.current) != absolute_tonal_elevation {
                            elevation.set(AbsoluteTonalElevation {
                                current: absolute_tonal_elevation,
                            });
                        }
                        provide_context(
                            || ContentColor {
                                current: content_color,
                            },
                            || {
                                let color = use_context::<ContentColor>().unwrap();
                                if color.with(|value| value.current) != content_color {
                                    color.set(ContentColor {
                                        current: content_color,
                                    });
                                }
                                if let Some(child) = child.as_ref() {
                                    child.render();
                                }
                            },
                        );
                    },
                );
                surface_foreground().paint(live);
            });
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
/// - `clip_content` — clip the surface and its descendants to the shape while
///   preserving the outer shadow.
/// - `elevation` — optional shadow elevation.
/// - `tonal_elevation` — optional tonal elevation tint for theme surfaces.
/// - `content_color` — optional explicit content color override.
/// - `content_alignment` — optional child alignment within the surface bounds.
/// - `enabled` — optional interactive enabled flag.
/// - `on_long_click` — optional stationary long-press action; also available
///   through the context-menu key, Shift+F10, and accessibility context action.
///   A successful long press suppresses the release click.
/// - `on_click` — optional click callback.
/// - `ripple_color` — optional ripple color.
/// - `ripple_bounded` — optional bounded-ripple flag.
/// - `ripple_radius` — optional explicit ripple radius.
/// - `interaction_state` — optional shared interaction state.
/// - `presentation_state` — optional visual interaction; live input and
///   disabled styling take precedence.
/// - `presentation_ripple` — optional static circular ripple for an idle
///   pressed presentation.
/// - `show_focus_indicator` — show a 3dp focus outline with a 2dp gap; defaults
///   to false.
/// - `show_state_layer` — optional state-layer visibility flag.
/// - `show_ripple` — optional ripple visibility flag.
/// - `ripple_state` — optional shared ripple state.
/// - `block_input` — optional input-blocking flag.
/// - `accessibility_toggled` — optional selection state for toggle controls.
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
/// use tessera_ui::{Dp, LayoutResult, Modifier};
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
    clip_content: Option<bool>,
    elevation: Option<Dp>,
    tonal_elevation: Option<Dp>,
    content_color: Option<Color>,
    content_alignment: Option<Alignment>,
    enabled: Option<bool>,
    on_click: Option<Callback>,
    on_long_click: Option<Callback>,
    ripple_color: Option<Color>,
    ripple_bounded: Option<bool>,
    ripple_radius: Option<Dp>,
    interaction_state: Option<State<InteractionState>>,
    presentation_state: Option<InteractionVisualState>,
    presentation_ripple: Option<RipplePresentation>,
    show_focus_indicator: Option<bool>,
    show_state_layer: Option<bool>,
    show_ripple: Option<bool>,
    ripple_state: Option<State<RippleState>>,
    block_input: Option<bool>,
    accessibility_role: Option<Role>,
    accessibility_toggled: Option<Toggled>,
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
    let mut resolved = SurfaceResolvedArgs {
        modifier: modifier.unwrap_or_default(),
        style: style.unwrap_or(SurfaceStyle::Filled {
            color: scheme.surface,
        }),
        shape: shape.unwrap_or_default(),
        clip_content: clip_content.unwrap_or(false),
        elevation,
        tonal_elevation: tonal_elevation.unwrap_or(Dp(0.0)),
        content_color,
        content_alignment: content_alignment.unwrap_or_default(),
        enabled: enabled.unwrap_or(true),
        on_click,
        on_long_click,
        ripple_color: ripple_color
            .or_else(|| use_context::<ContentColor>().map(|c| c.get().current))
            .unwrap_or(scheme.on_surface),
        ripple_bounded: ripple_bounded.unwrap_or(true),
        ripple_radius,
        interaction_state,
        presentation_state,
        presentation_ripple: presentation_ripple.map(RipplePresentation::sanitized),
        show_focus_indicator: show_focus_indicator.unwrap_or(false),
        show_state_layer: show_state_layer.unwrap_or(true),
        show_ripple: show_ripple.unwrap_or(true),
        ripple_state,
        block_input: block_input.unwrap_or(false),
        accessibility_role,
        accessibility_toggled,
        accessibility_label,
        accessibility_description,
        focus_requester,
        focus_properties,
        child,
    };
    let mut modifier = resolved.modifier.clone();
    let clickable = resolved.on_click.is_some() || resolved.on_long_click.is_some();
    let interactive = resolved.enabled && clickable;
    let internal_focus_requester = remember(FocusRequester::new).get();
    let bound_focus_requester = resolved.focus_requester.unwrap_or(internal_focus_requester);
    resolved.focus_requester = Some(bound_focus_requester);
    let interaction_state = resolved
        .interaction_state
        .or_else(|| interactive.then(|| remember(InteractionState::new)));
    let tap_recognizer = interactive.then(|| remember(TapRecognizer::default));
    let ripple_state = if resolved.show_ripple {
        resolved
            .ripple_state
            .or_else(|| interactive.then(|| remember(RippleState::new)))
    } else {
        None
    };
    let long_press_recognizer = if resolved.on_long_click.is_some() {
        let state = remember(|| {
            LongPressRecognizer::new(LongPressSettings {
                slop_px: Dp(8.0).to_pixels_f32(),
                ..Default::default()
            })
        });
        if !interactive && state.with(LongPressRecognizer::is_active) {
            state.with_mut(LongPressRecognizer::cancel);
        }
        if state.with(LongPressRecognizer::needs_frame) {
            let callback = resolved.on_long_click.expect("long click handler");
            receive_frame_nanos(move |frame| {
                if !state.is_alive() {
                    return tessera_ui::FrameNanosControl::Stop;
                }
                let triggered = state.with_mut(|recognizer| recognizer.advance_frame_nanos(frame));
                if triggered {
                    callback.call();
                }
                if state.with(LongPressRecognizer::needs_frame) {
                    tessera_ui::FrameNanosControl::Continue
                } else {
                    tessera_ui::FrameNanosControl::Stop
                }
            });
        }
        Some(state)
    } else {
        None
    };
    let owner = if interactive {
        let input = SurfaceInputOwner {
            interaction: interaction_state,
            ripple: ripple_state,
            tap: tap_recognizer,
            long_press: long_press_recognizer,
        };
        let owner = remember(|| Arc::new(SurfaceInputLease(Mutex::new(input)))).get();
        let previous = *owner.0.lock();
        if previous != input {
            previous.release();
            *owner.0.lock() = input;
        }
        Some(owner)
    } else {
        None
    };
    let has_semantics = resolved.accessibility_toggled.is_some()
        || resolved.accessibility_role.is_some()
        || resolved.accessibility_label.is_some()
        || resolved.accessibility_description.is_some()
        || accessibility_focusable.is_some();

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
            on_click: resolved.on_click.unwrap_or_else(Callback::noop),
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
            tap_recognizer,
            long_press_recognizer,
        };

        modifier = apply_combined_clickable_modifier(
            modifier,
            clickable_args,
            resolved.on_long_click,
            resolved.on_click.is_some(),
        );
        if resolved.on_long_click.is_some() {
            let owner = owner.expect("interactive surface input owner");
            modifier = modifier.on_focus_changed(move |focus: tessera_ui::FocusState| {
                if !focus.has_focus() {
                    let input = *owner.0.lock();
                    if input.long_press.is_some_and(|state| {
                        state.is_alive() && state.with(LongPressRecognizer::is_active)
                    }) {
                        input.release();
                    }
                }
            });
        }
    } else if resolved.block_input {
        modifier = modifier.block_touch_propagation();
    }

    if !interactive && has_semantics {
        let semantics = SurfaceSemantics {
            role: resolved.accessibility_role,
            label: resolved.accessibility_label.clone(),
            description: resolved.accessibility_description.clone(),
            focusable: accessibility_focusable,
            disabled: !resolved.enabled,
            toggled: resolved.accessibility_toggled,
        };
        modifier = modifier.push_semantics(semantics);
    }

    if interactive && resolved.accessibility_toggled.is_some() {
        modifier = modifier.push_semantics(SurfaceToggleSemantics(resolved.accessibility_toggled));
    }

    layout().modifier(modifier).child(move || {
        surface_content()
            .resolved(resolved.clone())
            .interaction_state_optional(interaction_state)
            .ripple_state_optional(ripple_state);
    });
}

struct SurfaceToggleSemantics(Option<Toggled>);

struct SurfaceSemantics {
    role: Option<Role>,
    label: Option<String>,
    description: Option<String>,
    focusable: Option<bool>,
    disabled: bool,
    toggled: Option<Toggled>,
}

impl SemanticsModifierNode for SurfaceSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        if let Some(role) = self.role {
            node.role = Some(role);
        }
        if let Some(label) = &self.label {
            node.label = Some(label.clone());
        }
        if let Some(description) = &self.description {
            node.description = Some(description.clone());
        }
        if let Some(focusable) = self.focusable {
            node.focusable = focusable;
        }
        if let Some(toggled) = self.toggled {
            node.toggled = Some(toggled);
        }
        node.disabled = self.disabled;
        if self.disabled {
            node.actions.clear();
            *handler = None;
        }
    }
}

impl SemanticsModifierNode for SurfaceToggleSemantics {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        _action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        accessibility.toggled = self.0;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{Color, Dp, Modifier, State, remember, tessera, use_context};

    use crate::{
        boxed::boxed,
        button::button,
        column::column,
        modifier::{InteractionState, ModifierExt as _},
        row::row,
        spacer::spacer,
        theme::{ContentColor, MaterialColorScheme, material_theme},
    };

    use super::{SurfaceLayout, SurfaceResolvedArgs, SurfaceStyle};

    #[test]
    fn focus_indicator_preserves_measured_stroke_gap_and_corners() {
        crate::test_utils::run_isolated_test(
            "surface::tests::focus_indicator_preserves_measured_stroke_gap_and_corners",
            || {
                use tessera_ui::{Px, PxPosition, PxSize};

                use crate::{pipelines::shape::command::ShapeCommand, shape_def::Shape};

                for (shape, body, expected_size, expected_radii) in [
                    (
                        Shape::CAPSULE,
                        PxSize::new(Px(56), Px(56)),
                        PxSize::new(Px(66), Px(66)),
                        [33.0; 4],
                    ),
                    (
                        Shape::rounded_rectangle(Dp(12.0)),
                        PxSize::new(Px(100), Px(40)),
                        PxSize::new(Px(110), Px(50)),
                        [17.0; 4],
                    ),
                ] {
                    let (command, size, position) =
                        super::focus_indicator_geometry(shape, body, Color::RED);
                    assert_eq!(size, expected_size);
                    assert_eq!(position, PxPosition::from_f32_xy(-5.0, -5.0));
                    let ShapeCommand::OutlinedRect {
                        border_width,
                        corner_radii,
                        ..
                    } = command
                    else {
                        panic!("rounded containers require a rounded focus outline");
                    };
                    assert_eq!(border_width, 3.0);
                    assert_eq!(corner_radii, expected_radii);
                }
            },
        );
    }

    thread_local! {
        static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static CONTENT_COLOR: Cell<Color> = const { Cell::new(Color::BLACK) };
    }

    #[tessera]
    fn content_color_probe() {
        CONTENT_COLOR.set(use_context::<ContentColor>().unwrap().get().current);
    }

    #[tessera]
    fn changing_button_fixture() {
        let enabled = remember(|| false);
        ENABLED.set(Some(enabled));
        material_theme().child(move || {
            button().text().enabled(enabled.get()).child(|| {
                content_color_probe();
            });
        });
    }

    #[test]
    fn enabling_a_button_updates_inherited_content_color() {
        use tessera_ui::testing::{
            layout_test, pump_layout_test_session, start_layout_test_session,
        };

        let mut session = start_layout_test_session(layout_test(|| {
            changing_button_fixture();
        }));
        pump_layout_test_session(&mut session, 0);
        let disabled = CONTENT_COLOR.get();
        ENABLED.get().unwrap().set(true);
        for frame in 1..4 {
            pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_ne!(CONTENT_COLOR.get(), disabled);
        assert_eq!(CONTENT_COLOR.get(), MaterialColorScheme::default().primary);
    }

    fn layout_with_style(style: SurfaceStyle) -> SurfaceLayout {
        SurfaceLayout {
            args: SurfaceResolvedArgs {
                style,
                ..SurfaceResolvedArgs::default()
            },
            interaction_state: None,
            ripple_state: None,
            ripple_frame_nanos: None,
            focus_visible: false,
            scheme: MaterialColorScheme::default(),
            absolute_tonal_elevation: Dp(0.0),
        }
    }

    fn outlined_track() -> SurfaceStyle {
        SurfaceStyle::FilledOutlined {
            fill_color: Color::new(0.1, 0.2, 0.3, 1.0),
            border_color: Color::new(0.4, 0.5, 0.6, 1.0),
            border_width: Dp(1.0),
        }
    }

    #[test]
    fn surface_layout_equality_tracks_style_changes() {
        let outlined = layout_with_style(outlined_track());
        let filled = layout_with_style(SurfaceStyle::Filled {
            color: Color::new(0.9, 0.1, 0.1, 1.0),
        });

        assert!(outlined != filled);
        assert!(outlined == layout_with_style(outlined_track()));
    }

    #[tessera]
    fn transparent_state_layer_pair(style: SurfaceStyle, reference: SurfaceStyle) {
        let interaction = remember(|| {
            let mut state = InteractionState::new();
            state.set_hovered(true);
            state
        });
        row().children(move || {
            super::surface()
                .modifier(Modifier::new().required_size(Dp(64.0), Dp(64.0)))
                .style(style.clone())
                .shape(crate::shape_def::Shape::RECTANGLE)
                .ripple_color(Color::BLUE)
                .interaction_state(interaction)
                .show_ripple(false);
            spacer().modifier(Modifier::new().width(Dp(8.0)));
            super::surface()
                .modifier(Modifier::new().required_size(Dp(64.0), Dp(64.0)))
                .style(reference.clone())
                .shape(crate::shape_def::Shape::RECTANGLE)
                .show_state_layer(false)
                .show_ripple(false);
        });
    }

    #[tessera]
    fn transparent_state_layers_fixture() {
        material_theme().child(|| {
            boxed()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .children(|| {
                    column().children(|| {
                        for style in 0..3 {
                            let fill = |color| match style {
                                0 => SurfaceStyle::Filled { color },
                                _ => SurfaceStyle::FilledOutlined {
                                    fill_color: color,
                                    border_color: Color::RED,
                                    border_width: Dp(1.0),
                                },
                            };
                            let base = if style == 1 {
                                SurfaceStyle::Outlined {
                                    color: Color::RED,
                                    width: Dp(1.0),
                                }
                            } else {
                                fill(Color::TRANSPARENT)
                            };
                            transparent_state_layer_pair(base, fill(Color::BLUE.with_alpha(0.08)));
                            spacer().modifier(Modifier::new().height(Dp(8.0)));
                        }
                    });
                });
        });
    }

    fn check_transparent_state_layers(samples: u32) {
        let mut renderer = tessera_ui::HeadlessRenderer::new(
            Box::new(|| {
                transparent_state_layers_fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            tessera_ui::HeadlessConfig {
                width: 200,
                height: 240,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        for frames in [2, 8] {
            renderer.render_frames(frames);
            let pixels = renderer.read_pixels().unwrap();
            for y in [32usize, 104, 176] {
                let actual = (y * 200 + 32) * 4;
                let reference = (y * 200 + 104) * 4;
                assert_eq!(
                    &pixels[actual..actual + 4],
                    &pixels[reference..reference + 4],
                    "transparent state layer, MSAA {samples}, y={y}, frames={frames}"
                );
            }
        }
    }

    #[derive(Clone, PartialEq)]
    struct TransparentOutlineSample {
        fill_color: Color,
        ripple_opacity: f32,
    }

    impl tessera_ui::DrawModifierNode for TransparentOutlineSample {
        fn draw(
            &self,
            ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
            content: &mut dyn tessera_ui::DrawModifierContent,
        ) {
            ctx.render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(
                    crate::pipelines::shape::command::ShapeCommand::RippleFilledOutlinedRect {
                        color: self.fill_color,
                        border_color: Color::new(0.1, 0.4, 0.2, 1.0),
                        corner_radii: [12.0; 4],
                        corner_g2: [2.0; 4],
                        border_width: 3.0,
                        ripple: crate::RippleProps {
                            center: [0.0, 0.0],
                            radius: 0.6,
                            alpha: self.ripple_opacity,
                            bounded: true,
                            color: Color::BLACK,
                        },
                    },
                );
            content.draw(ctx.render_input);
        }
    }

    #[tessera]
    fn transparent_outline_rgb_sample(fill_color: Color, ripple_opacity: f32) {
        tessera_ui::layout::layout()
            .modifier(Modifier::new().required_size(Dp(64.0), Dp(64.0)))
            .draw(TransparentOutlineSample {
                fill_color,
                ripple_opacity,
            });
    }

    #[tessera]
    fn transparent_outline_rgb_fixture() {
        material_theme().child(|| {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(Color::WHITE)
                        .padding_all(Dp(8.0)),
                )
                .children(|| {
                    column().children(|| {
                        for opacity in [0.0, 0.25] {
                            row().children(move || {
                                for color in [Color::RED, Color::BLUE] {
                                    transparent_outline_rgb_sample(color.with_alpha(0.0), opacity);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                }
                            });
                            spacer().modifier(Modifier::new().height(Dp(8.0)));
                        }
                    });
                });
        });
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn transparent_outline_fill_rgb_does_not_affect_border_with_or_without_ripple() {
        crate::test_utils::run_isolated_test(
            "surface::tests::transparent_outline_fill_rgb_does_not_affect_border_with_or_without_ripple",
            || {
                let mut renderer = tessera_ui::HeadlessRenderer::new(
                    Box::new(|| {
                        transparent_outline_rgb_fixture();
                    }),
                    vec![Box::new(crate::TesseraComponents)],
                    tessera_ui::HeadlessConfig {
                        width: 160,
                        height: 160,
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(4);
                let pixels = renderer.read_pixels().unwrap();
                let crop = |left: usize, top: usize| {
                    let mut result = Vec::new();
                    for y in top..top + 64 {
                        let offset = (y * 160 + left) * 4;
                        result.extend_from_slice(&pixels[offset..offset + 64 * 4]);
                    }
                    result
                };
                let zero_ripple = crop(8, 8);
                assert!(
                    zero_ripple
                        .chunks_exact(4)
                        .any(|pixel| pixel[..3] != [255; 3]),
                    "the fixture renders a visible outline"
                );
                for (top, label) in [(8, "zero ripple"), (80, "static ripple")] {
                    let left = crop(8, top);
                    let right = crop(80, top);
                    let difference = left
                        .chunks_exact(4)
                        .zip(right.chunks_exact(4))
                        .position(|(a, b)| a != b);
                    assert_eq!(
                        difference, None,
                        "transparent hidden RGB must not tint border coverage with {label}"
                    );
                }
                assert_ne!(
                    zero_ripple,
                    crop(8, 80),
                    "the second pair renders a visible native ripple"
                );
            },
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn transparent_state_layers_match_direct_alpha_rendering_msaa1() {
        check_transparent_state_layers(1);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn transparent_state_layers_match_direct_alpha_rendering_msaa4() {
        check_transparent_state_layers(4);
    }
}
