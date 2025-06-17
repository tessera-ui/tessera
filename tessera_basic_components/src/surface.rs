use derive_builder::Builder;
use std::sync::{Arc, atomic};
use tessera::{
    BasicDrawable, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp,
    MeasurementError, PressKeyEventType, Px, PxPosition, RippleProps, ShadowProps, measure_nodes,
    place_node,
};
use tessera_macros::tessera;

use crate::pos_misc::is_position_in_component;

/// State for managing ripple animation and hover effects
pub struct RippleState {
    pub is_animating: atomic::AtomicBool,
    pub start_time: atomic::AtomicU64, // Store as u64 millis since epoch
    pub click_pos_x: atomic::AtomicI32, // Store as fixed-point * 1000
    pub click_pos_y: atomic::AtomicI32, // Store as fixed-point * 1000
    pub is_hovered: atomic::AtomicBool, // Track hover state
}

impl Default for RippleState {
    fn default() -> Self {
        Self::new()
    }
}

impl RippleState {
    pub fn new() -> Self {
        Self {
            is_animating: atomic::AtomicBool::new(false),
            start_time: atomic::AtomicU64::new(0),
            click_pos_x: atomic::AtomicI32::new(0),
            click_pos_y: atomic::AtomicI32::new(0),
            is_hovered: atomic::AtomicBool::new(false),
        }
    }

    pub fn start_animation(&self, click_pos: [f32; 2]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.start_time.store(now, atomic::Ordering::SeqCst);
        self.click_pos_x
            .store((click_pos[0] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.click_pos_y
            .store((click_pos[1] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.is_animating.store(true, atomic::Ordering::SeqCst);
    }

    pub fn get_animation_progress(&self) -> Option<(f32, [f32; 2])> {
        let is_animating = self.is_animating.load(atomic::Ordering::SeqCst);

        if !is_animating {
            return None;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let start = self.start_time.load(atomic::Ordering::SeqCst);
        let elapsed_ms = now.saturating_sub(start);
        let progress = (elapsed_ms as f32) / 600.0; // 600ms animation

        if progress >= 1.0 {
            self.is_animating.store(false, atomic::Ordering::SeqCst);
            return None;
        }

        let click_pos = [
            self.click_pos_x.load(atomic::Ordering::SeqCst) as f32 / 1000.0,
            self.click_pos_y.load(atomic::Ordering::SeqCst) as f32 / 1000.0,
        ];

        Some((progress, click_pos))
    }

    /// Set hover state
    pub fn set_hovered(&self, hovered: bool) {
        self.is_hovered.store(hovered, atomic::Ordering::SeqCst);
    }

    /// Get hover state
    pub fn is_hovered(&self) -> bool {
        self.is_hovered.load(atomic::Ordering::SeqCst)
    }
}

/// Arguments for the `surface` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SurfaceArgs {
    /// The fill color of the surface (RGBA).
    #[builder(default = "[0.4745, 0.5255, 0.7961, 1.0]")]
    pub color: [f32; 4],
    /// The hover color of the surface (RGBA). If None, no hover effect is applied.
    #[builder(default)]
    pub hover_color: Option<[f32; 4]>,
    /// The corner radius of the surface.
    #[builder(default = "0.0")]
    pub corner_radius: f32,
    /// The shadow properties of the surface.
    #[builder(default)]
    pub shadow: Option<ShadowProps>,
    /// The padding of the surface.
    #[builder(default = "Dp(0.0)")]
    pub padding: Dp,
    /// Optional explicit width behavior for the surface. Defaults to Wrap {min: None, max: None} if None.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the surface. Defaults to Wrap {min: None, max: None} if None.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
    /// Width of the border. If > 0, an outline will be drawn.
    #[builder(default = "0.0")]
    pub border_width: f32,
    /// Optional color for the border (RGBA). If None and border_width > 0, `color` will be used.
    #[builder(default)]
    pub border_color: Option<[f32; 4]>,
    /// Optional click callback function. If provided, surface becomes interactive with ripple effect.
    #[builder(default)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// The ripple color (RGB) for interactive surfaces.
    #[builder(default = "[1.0, 1.0, 1.0]")]
    pub ripple_color: [f32; 3],
}

impl std::fmt::Debug for SurfaceArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceArgs")
            .field("color", &self.color)
            .field("hover_color", &self.hover_color)
            .field("corner_radius", &self.corner_radius)
            .field("shadow", &self.shadow)
            .field("padding", &self.padding)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("border_width", &self.border_width)
            .field("border_color", &self.border_color)
            .field(
                "on_click",
                &if self.on_click.is_some() {
                    "<callback>"
                } else {
                    "None"
                },
            )
            .field("ripple_color", &self.ripple_color)
            .finish()
    }
}

// Manual implementation of Default because derive_builder's default conflicts with our specific defaults
impl Default for SurfaceArgs {
    fn default() -> Self {
        SurfaceArgsBuilder::default().build().unwrap()
    }
}

/// Surface component, a basic container that can have its own size constraints.
/// If args contains an on_click callback, a ripple_state must be provided for interactive behavior.
#[tessera]
pub fn surface(args: SurfaceArgs, ripple_state: Option<Arc<RippleState>>, child: impl FnOnce()) {
    let measure_args = args.clone();
    let ripple_state_for_measure = ripple_state.clone();

    measure(Box::new(move |input| {
        let padding_px: Px = measure_args.padding.into();
        let padding_2_px = padding_px * 2;

        // 1. Determine Surface's intrinsic constraint based on args
        let surface_intrinsic_width = measure_args.width.unwrap_or(DimensionValue::Wrap {
            min: None,
            max: None,
        });
        let surface_intrinsic_height = measure_args.height.unwrap_or(DimensionValue::Wrap {
            min: None,
            max: None,
        });
        let surface_intrinsic_constraint =
            Constraint::new(surface_intrinsic_width, surface_intrinsic_height);

        // 2. Merge with parent_constraint to get effective_surface_constraint
        let effective_surface_constraint =
            surface_intrinsic_constraint.merge(input.effective_constraint);

        // 3. Determine constraint for the child
        // For Fill constraint, Surface should determine its own final size first, then give child a Fixed constraint
        let child_constraint_width = match effective_surface_constraint.width {
            DimensionValue::Fixed(sw) => DimensionValue::Fixed((sw - padding_2_px).max(Px(0))),
            DimensionValue::Wrap {
                min: s_min_w,
                max: s_max_w,
            } => DimensionValue::Wrap {
                min: s_min_w.map(|m| (m - padding_2_px).max(Px(0))),
                max: s_max_w.map(|m| (m - padding_2_px).max(Px(0))),
            },
            DimensionValue::Fill {
                min: _s_min_w,
                max: s_max_w,
            } => {
                // For Fill, Surface should use parent's provided width and give child a Fixed constraint
                let parent_provided_width = match input.effective_constraint.width {
                    DimensionValue::Fixed(pw) => Some(pw),
                    DimensionValue::Fill {
                        max: p_max_fill, ..
                    } => p_max_fill,
                    _ => None,
                };

                if let Some(ppw) = parent_provided_width {
                    // Surface takes the full parent-provided width, child gets fixed constraint
                    DimensionValue::Fixed((ppw - padding_2_px).max(Px(0)))
                } else {
                    // No parent width available, fallback to wrap-like behavior
                    DimensionValue::Wrap {
                        min: None,
                        max: s_max_w.map(|m| (m - padding_2_px).max(Px(0))),
                    }
                }
            }
        };
        let child_constraint_height = match effective_surface_constraint.height {
            DimensionValue::Fixed(sh) => DimensionValue::Fixed((sh - padding_2_px).max(Px(0))),
            DimensionValue::Wrap {
                min: s_min_h,
                max: s_max_h,
            } => DimensionValue::Wrap {
                min: s_min_h.map(|m| (m - padding_2_px).max(Px(0))),
                max: s_max_h.map(|m| (m - padding_2_px).max(Px(0))),
            },
            DimensionValue::Fill {
                min: _s_min_h,
                max: s_max_h,
            } => {
                // For Fill, Surface should use parent's provided height and give child a Fixed constraint
                let parent_provided_height = match input.effective_constraint.height {
                    DimensionValue::Fixed(ph) => Some(ph),
                    DimensionValue::Fill {
                        max: p_max_fill, ..
                    } => p_max_fill,
                    _ => None,
                };

                if let Some(pph) = parent_provided_height {
                    // Surface takes the full parent-provided height, child gets fixed constraint
                    DimensionValue::Fixed((pph - padding_2_px).max(Px(0)))
                } else {
                    // No parent height available, fallback to wrap-like behavior
                    DimensionValue::Wrap {
                        min: None,
                        max: s_max_h.map(|m| (m - padding_2_px).max(Px(0))),
                    }
                }
            }
        };
        let child_actual_constraint =
            Constraint::new(child_constraint_width, child_constraint_height);

        // 4. Measure the child
        let mut child_measured_size = ComputedData::ZERO;
        if let Some(&child_node_id) = input.children_ids.first() {
            let child_intrinsic_constraint = input
                .metadatas
                .get(&child_node_id)
                .ok_or(MeasurementError::ChildMeasurementFailed(child_node_id))?
                .constraint;
            let final_child_constraint_for_measure =
                child_intrinsic_constraint.merge(&child_actual_constraint);

            let nodes_to_measure = vec![(child_node_id, final_child_constraint_for_measure)];
            let results_map = measure_nodes(nodes_to_measure, input.tree, input.metadatas);

            child_measured_size = results_map
                .get(&child_node_id)
                .ok_or_else(|| {
                    MeasurementError::MeasureFnFailed(format!(
                        "Child {child_node_id:?} result missing in map"
                    ))
                })?
                .clone()?;

            place_node(
                child_node_id,
                PxPosition::new(padding_px, padding_px),
                input.metadatas,
            );
        }

        // 5. Calculate final Surface dimensions
        let content_width_with_padding = child_measured_size.width + padding_2_px;
        let content_height_with_padding = child_measured_size.height + padding_2_px;

        let mut final_surface_width = content_width_with_padding;
        match effective_surface_constraint.width {
            DimensionValue::Fixed(sw) => final_surface_width = sw,
            DimensionValue::Wrap { min, max } => {
                if let Some(min_w) = min {
                    final_surface_width = final_surface_width.max(min_w);
                }
                if let Some(max_w) = max {
                    final_surface_width = final_surface_width.min(max_w);
                }
            }
            DimensionValue::Fill { min, max } => {
                // For Fill constraint, use the max value from Surface's constraint (which comes from parent)
                if let Some(max_w) = max {
                    final_surface_width = max_w; // Fill should use the provided max constraint
                } else {
                    // When no max constraint provided, wrap content (like a Wrap behavior)
                    final_surface_width = content_width_with_padding;
                }
                if let Some(min_w) = min {
                    final_surface_width = final_surface_width.max(min_w);
                }
            }
        };

        let mut final_surface_height = content_height_with_padding;
        match effective_surface_constraint.height {
            DimensionValue::Fixed(sh) => final_surface_height = sh,
            DimensionValue::Wrap { min, max } => {
                if let Some(min_h) = min {
                    final_surface_height = final_surface_height.max(min_h);
                }
                if let Some(max_h) = max {
                    final_surface_height = final_surface_height.min(max_h);
                }
            }
            DimensionValue::Fill { min, max } => {
                // For Fill constraint, use the max value from Surface's constraint (which comes from parent)
                if let Some(max_h) = max {
                    final_surface_height = max_h; // Fill should use the provided max constraint
                } else {
                    // When no max constraint provided, wrap content (like a Wrap behavior)
                    final_surface_height = content_height_with_padding;
                }
                if let Some(min_h) = min {
                    final_surface_height = final_surface_height.max(min_h);
                }
            }
        };

        // 6. Determine the color to use based on hover state
        let is_hovered = ripple_state_for_measure
            .as_ref()
            .map(|state| state.is_hovered())
            .unwrap_or(false);

        let effective_color = if is_hovered && measure_args.hover_color.is_some() {
            measure_args.hover_color.unwrap()
        } else {
            measure_args.color
        };

        let drawable = if measure_args.on_click.is_some() {
            // Interactive surface with ripple effect
            let ripple_props = if let Some(ref state) = ripple_state_for_measure {
                if let Some((progress, click_pos)) = state.get_animation_progress() {
                    let radius = progress; // Expand from 0 to 1
                    let alpha = (1.0 - progress) * 0.3; // Fade out

                    RippleProps {
                        center: click_pos,
                        radius,
                        alpha,
                        color: measure_args.ripple_color,
                    }
                } else {
                    RippleProps::default()
                }
            } else {
                RippleProps::default()
            };

            if measure_args.border_width > 0.0 {
                BasicDrawable::RippleOutlinedRect {
                    color: measure_args.border_color.unwrap_or(effective_color),
                    corner_radius: measure_args.corner_radius,
                    shadow: measure_args.shadow,
                    border_width: measure_args.border_width,
                    ripple: ripple_props,
                }
            } else {
                BasicDrawable::RippleRect {
                    color: effective_color,
                    corner_radius: measure_args.corner_radius,
                    shadow: measure_args.shadow,
                    ripple: ripple_props,
                }
            }
        } else {
            // Non-interactive surface
            if measure_args.border_width > 0.0 {
                BasicDrawable::OutlinedRect {
                    color: measure_args.border_color.unwrap_or(effective_color),
                    corner_radius: measure_args.corner_radius,
                    shadow: measure_args.shadow,
                    border_width: measure_args.border_width,
                }
            } else {
                BasicDrawable::Rect {
                    color: effective_color,
                    corner_radius: measure_args.corner_radius,
                    shadow: measure_args.shadow,
                }
            }
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(drawable);
        }

        Ok(ComputedData {
            width: final_surface_width.max(Px(0)), // Ensure final dimensions are not negative
            height: final_surface_height.max(Px(0)), // Ensure final dimensions are not negative
        })
    }));

    child();

    // Event handling for interactive surfaces
    if args.on_click.is_some() {
        let args_for_handler = args.clone();
        let state_for_handler = ripple_state;
        state_handler(Box::new(move |input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position;
            let is_cursor_in_surface = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            // Update hover state
            if let Some(ref state) = state_for_handler {
                state.set_hovered(is_cursor_in_surface);
            }

            // Handle mouse events
            if is_cursor_in_surface {
                // Check for mouse press events to start ripple
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

                // Check for mouse release events (click)
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
                    // Convert cursor position to normalized coordinates [-0.5, 0.5]
                    let normalized_x = (cursor_pos.x.to_f32() / size.width.to_f32()) - 0.5;
                    let normalized_y = (cursor_pos.y.to_f32() / size.height.to_f32()) - 0.5;

                    // Start ripple animation
                    state.start_animation([normalized_x, normalized_y]);
                }

                if !release_events.is_empty() {
                    // Trigger click callback
                    if let Some(ref on_click) = args_for_handler.on_click {
                        on_click();
                    }
                }

                // Consume cursor events if we're handling relevant mouse events
                if !press_events.is_empty() || !release_events.is_empty() {
                    input.cursor_events.clear();
                }
            }
        }));
    }
}
