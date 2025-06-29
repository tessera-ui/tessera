use derive_builder::Builder;
use std::sync::{Arc, atomic};
use tessera::{
    ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType, Px,
    PxPosition, measure_node, place_node,
};
use tessera_macros::tessera;

use crate::{
    pipelines::{RippleProps, ShadowProps, ShapeCommand},
    pos_misc::is_position_in_component,
};

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
    (child)();
    let ripple_state_for_measure = ripple_state.clone();
    let args_measure_clone = args.clone();

    measure(Box::new(move |input| {
        // Determine Surface's intrinsic constraint based on args
        let surface_intrinsic_width = args_measure_clone.width.unwrap_or(DimensionValue::Wrap {
            min: None,
            max: None,
        });
        let surface_intrinsic_height = args_measure_clone.height.unwrap_or(DimensionValue::Wrap {
            min: None,
            max: None,
        });
        let surface_intrinsic_constraint =
            Constraint::new(surface_intrinsic_width, surface_intrinsic_height);
        // Merge with parent_constraint to get effective_surface_constraint
        let effective_surface_constraint =
            surface_intrinsic_constraint.merge(input.parent_constraint);
        // Determine constraint for the child
        let child_constraint = Constraint::new(
            remove_padding_from_dimension(
                effective_surface_constraint.width,
                args_measure_clone.padding.into(),
            ),
            remove_padding_from_dimension(
                effective_surface_constraint.height,
                args_measure_clone.padding.into(),
            ),
        );
        // Measure the child with the computed constraint
        let child_measurement = if !input.children_ids.is_empty() {
            let child_measurement = measure_node(
                input.children_ids[0],
                &child_constraint,
                input.tree,
                input.metadatas,
            )?;
            // place the child
            place_node(
                input.children_ids[0],
                PxPosition {
                    x: args.padding.into(),
                    y: args.padding.into(),
                },
                input.metadatas,
            );
            child_measurement
        } else {
            ComputedData {
                width: Px(0),
                height: Px(0),
            }
        };
        // Add drawable for the surface
        let is_hovered = ripple_state_for_measure
            .as_ref()
            .map(|state| state.is_hovered())
            .unwrap_or(false);

        let effective_color = if is_hovered && args_measure_clone.hover_color.is_some() {
            args_measure_clone.hover_color.unwrap()
        } else {
            args_measure_clone.color
        };

        let drawable = if args_measure_clone.on_click.is_some() {
            // Interactive surface with ripple effect
            let ripple_props = if let Some(ref state) = ripple_state_for_measure {
                if let Some((progress, click_pos)) = state.get_animation_progress() {
                    let radius = progress; // Expand from 0 to 1
                    let alpha = (1.0 - progress) * 0.3; // Fade out

                    RippleProps {
                        center: click_pos,
                        radius,
                        alpha,
                        color: args_measure_clone.ripple_color,
                    }
                } else {
                    RippleProps::default()
                }
            } else {
                RippleProps::default()
            };

            if args_measure_clone.border_width > 0.0 {
                ShapeCommand::RippleOutlinedRect {
                    color: args_measure_clone.border_color.unwrap_or(effective_color),
                    corner_radius: args_measure_clone.corner_radius,
                    shadow: args_measure_clone.shadow,
                    border_width: args_measure_clone.border_width,
                    ripple: ripple_props,
                    segments_per_corner: 32,
                }
            } else {
                ShapeCommand::RippleRect {
                    color: effective_color,
                    corner_radius: args_measure_clone.corner_radius,
                    shadow: args_measure_clone.shadow,
                    ripple: ripple_props,
                    segments_per_corner: 32,
                }
            }
        } else {
            // Non-interactive surface
            if args_measure_clone.border_width > 0.0 {
                ShapeCommand::OutlinedRect {
                    color: args_measure_clone.border_color.unwrap_or(effective_color),
                    corner_radius: args_measure_clone.corner_radius,
                    shadow: args_measure_clone.shadow,
                    border_width: args_measure_clone.border_width,
                    segments_per_corner: 32,
                }
            } else {
                ShapeCommand::Rect {
                    color: effective_color,
                    corner_radius: args_measure_clone.corner_radius,
                    shadow: args_measure_clone.shadow,
                    segments_per_corner: 32,
                }
            }
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(Box::new(drawable));
        }

        // Calculate the final size of the surface
        let padding_px: Px = args_measure_clone.padding.into();
        let min_width = child_measurement.width + padding_px * 2;
        let min_height = child_measurement.height + padding_px * 2;
        let width = match effective_surface_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect("Seems that you are trying to fill an infinite width, which is not allowed")
                .max(min_height)
                .max(min.unwrap_or(Px(0))),
        };
        let height = match effective_surface_constraint.height {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_height)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect(
                    "Seems that you are trying to fill an infinite height, which is not allowed",
                )
                .max(min_height)
                .max(min.unwrap_or(Px(0))),
        };
        Ok(ComputedData { width, height })
    }));

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

            // Set cursor to pointer if hovered and clickable
            if is_cursor_in_surface && args_for_handler.on_click.is_some() {
                input
                    .requests
                    .cursor_icon
                    .set(Some(tessera::winit::window::CursorIcon::Pointer));
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

fn remove_padding_from_dimension(dimension: DimensionValue, padding: Px) -> DimensionValue {
    match dimension {
        DimensionValue::Fixed(value) => DimensionValue::Fixed((value - padding * 2).max(Px(0))),
        DimensionValue::Wrap { min, max } => DimensionValue::Wrap {
            min: min.map(|m| (m - padding * 2).max(Px(0))),
            max: max.map(|m| (m - padding * 2).max(Px(0))),
        },
        DimensionValue::Fill { min, max } => DimensionValue::Fill {
            min,
            max: max.map(|m| (m - padding * 2).max(Px(0))),
        },
    }
}
