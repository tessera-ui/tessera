//! A component that displays content sliding up from the bottom of the screen.
//!
//! ## Usage
//!
//! Used to show contextual information or actions in a modal sheet.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use tessera_ui::{
    Color, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType, Px, PxPosition,
    State, remember, tessera, use_context, winit,
};

use crate::{
    alignment::CrossAxisAlignment,
    animation,
    column::{ColumnArgsBuilder, column},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    shape_def::{RoundedCorner, Shape},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    theme::MaterialColorScheme,
};

const ANIM_TIME: Duration = Duration::from_millis(300);

/// Defines the visual style of the bottom sheet's scrim.
///
/// The scrim is the overlay that appears behind the bottom sheet, covering the
/// main content.
#[derive(Default, Clone, Copy)]
pub enum BottomSheetStyle {
    /// A translucent glass effect that blurs the content behind it.
    /// This style is more resource-intensive and may not be suitable for all
    /// targets.
    Glass,
    /// A simple, semi-transparent dark overlay. This is the default style.
    #[default]
    Material,
}

/// Configuration arguments for the [`bottom_sheet_provider`].
#[derive(Builder)]
pub struct BottomSheetProviderArgs {
    /// A callback that is invoked when the user requests to close the sheet.
    ///
    /// This can be triggered by clicking the scrim or pressing the `Escape`
    /// key. The callback is responsible for closing the sheet.
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
    /// The visual style of the scrim. See [`BottomSheetStyle`].
    #[builder(default)]
    pub style: BottomSheetStyle,
    /// Whether the sheet is initially open (for declarative usage).
    #[builder(default = "false")]
    pub is_open: bool,
}

/// Controller for [`bottom_sheet_provider`], managing open/closed state.
///
/// This controller can be created by the application and passed to the
/// [`bottom_sheet_provider_with_controller`]. It is used to control the
/// visibility of the sheet programmatically.
#[derive(Clone)]
pub struct BottomSheetController {
    is_open: bool,
    timer: Option<Instant>,
    is_dragging: bool,
    drag_offset: f32,
    drag_start_y: f32,
}

impl BottomSheetController {
    /// Creates a new controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            timer: None,
            is_dragging: false,
            drag_offset: 0.0,
            drag_start_y: 0.0,
        }
    }

    /// Initiates the animation to open the bottom sheet.
    ///
    /// If the sheet is already open, this has no effect. If the sheet is
    /// currently closing, it will reverse direction and start opening from
    /// its current position.
    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            self.drag_offset = 0.0;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }

    /// Initiates the animation to close the bottom sheet.
    ///
    /// If the sheet is already closed, this has no effect. If the sheet is
    /// currently opening, it will reverse direction and start closing from
    /// its current position.
    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }

    /// Returns whether the sheet is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns whether the sheet is currently animating in either direction.
    pub fn is_animating(&self) -> bool {
        self.timer.is_some_and(|t| t.elapsed() < ANIM_TIME)
    }

    fn snapshot(&self) -> (bool, Option<Instant>, f32) {
        (self.is_open, self.timer, self.drag_offset)
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    fn update_drag_offset(&mut self, offset: f32) {
        self.drag_offset = offset;
    }

    fn get_drag_offset(&self) -> f32 {
        self.drag_offset
    }

    fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    fn set_drag_start_y(&mut self, y: f32) {
        self.drag_start_y = y;
    }

    fn get_drag_start_y(&self) -> f32 {
        self.drag_start_y
    }
}

impl Default for BottomSheetController {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Compute eased progress from an optional timer reference.
fn calc_progress_from_timer(timer: Option<&Instant>) -> f32 {
    let raw = match timer {
        None => 1.0,
        Some(t) => {
            let elapsed = t.elapsed();
            if elapsed >= ANIM_TIME {
                1.0
            } else {
                elapsed.as_secs_f32() / ANIM_TIME.as_secs_f32()
            }
        }
    };
    animation::easing(raw)
}

/// Compute blur radius for glass style.
fn blur_radius_for(progress: f32, is_open: bool, max_blur_radius: f32) -> f32 {
    if is_open {
        progress * max_blur_radius
    } else {
        max_blur_radius * (1.0 - progress)
    }
}

/// Compute scrim alpha for material style.
fn scrim_alpha_for(progress: f32, is_open: bool) -> f32 {
    if is_open {
        progress * 0.32
    } else {
        0.32 * (1.0 - progress)
    }
}

/// Compute Y position for bottom sheet placement.
fn compute_bottom_sheet_y(
    parent_height: Px,
    child_height: Px,
    progress: f32,
    is_open: bool,
    drag_offset: f32,
) -> i32 {
    let parent = parent_height.0 as f32;
    let child = child_height.0 as f32;
    let y = if is_open {
        parent - child * progress
    } else {
        parent - child * (1.0 - progress)
    };
    (y + drag_offset) as i32
}

fn render_glass_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
    // Glass scrim: compute blur radius and render using fluid_glass.
    let max_blur_radius = 5.0;
    let blur_radius = blur_radius_for(progress, is_open, max_blur_radius);
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .on_click(args.on_close_request.clone())
            .tint_color(Color::TRANSPARENT)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .dispersion_height(Dp(0.0))
            .refraction_height(Dp(0.0))
            .block_input(true)
            .blur_radius(Dp(blur_radius as f64))
            .border(None)
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
            .noise_amount(0.0)
            .build()
            .expect("FluidGlassArgsBuilder failed with required fields set"),
        || {},
    );
}

fn render_material_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
    // Material scrim: compute alpha and render a simple dark surface.
    let scrim_alpha = scrim_alpha_for(progress, is_open);
    let scrim_color = use_context::<MaterialColorScheme>().get().scrim;
    surface(
        SurfaceArgsBuilder::default()
            .style(scrim_color.with_alpha(scrim_alpha).into())
            .on_click(args.on_close_request.clone())
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .block_input(true)
            .build()
            .expect("SurfaceArgsBuilder failed with required fields set"),
        || {},
    );
}

/// Render scrim according to configured style.
/// Delegates actual rendering to small, focused helpers to keep the
/// main API surface concise and improve readability.
fn render_scrim(args: &BottomSheetProviderArgs, progress: f32, is_open: bool) {
    match args.style {
        BottomSheetStyle::Glass => render_glass_scrim(args, progress, is_open),
        BottomSheetStyle::Material => render_material_scrim(args, progress, is_open),
    }
}

/// Create the keyboard handler closure used to close the sheet on Escape.
fn make_keyboard_closure(
    on_close: Arc<dyn Fn() + Send + Sync>,
) -> Box<dyn Fn(tessera_ui::InputHandlerInput<'_>) + Send + Sync> {
    Box::new(move |input: tessera_ui::InputHandlerInput<'_>| {
        for event in input.keyboard_events.drain(..) {
            if event.state == winit::event::ElementState::Pressed
                && let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                    event.physical_key
            {
                (on_close)();
            }
        }
    })
}

/// Handle drag gestures on the bottom sheet.
fn handle_drag_gestures(
    controller: State<BottomSheetController>,
    input: &mut tessera_ui::InputHandlerInput<'_>,
    on_close: &Arc<dyn Fn() + Send + Sync>,
) {
    let mut is_dragging = controller.with(|c| c.is_dragging());
    let drag_offset = controller.with(|c| c.get_drag_offset());

    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(PressKeyEventType::Left) => {
                if let Some(pos) = input.cursor_position_rel {
                    is_dragging = true;
                    controller.with_mut(|c| {
                        c.set_dragging(true);
                        c.set_drag_start_y(pos.y.0 as f32);
                    });
                }
            }
            CursorEventContent::Released(PressKeyEventType::Left) => {
                if is_dragging {
                    is_dragging = false;
                    controller.with_mut(|c| c.set_dragging(false));

                    if drag_offset > 100.0 {
                        (on_close)();
                    } else {
                        controller.with_mut(|c| c.update_drag_offset(0.0));
                    }
                }
            }
            _ => {}
        }
    }

    if is_dragging && let Some(pos) = input.cursor_position_rel {
        let current_y = pos.y.0 as f32;
        let start_y = controller.with(|c| c.get_drag_start_y());
        let delta = current_y - start_y;

        // Accumulate delta since component moves with drag.
        let new_offset = (drag_offset + delta).max(0.0);
        if (new_offset - drag_offset).abs() > 0.001 {
            controller.with_mut(|c| c.update_drag_offset(new_offset));
        }
    }
}

/// Place bottom sheet if present. Extracted to reduce complexity of the parent
/// function.
fn place_bottom_sheet_if_present(
    input: &tessera_ui::MeasureInput<'_>,
    controller_for_measure: State<BottomSheetController>,
    progress: f32,
) {
    if input.children_ids.len() <= 2 {
        return;
    }

    let bottom_sheet_id = input.children_ids[2];

    let parent_width = input.parent_constraint.width.get_max().unwrap_or(Px(0));
    let parent_height = input.parent_constraint.height.get_max().unwrap_or(Px(0));

    // M3 Spec: Max width 640dp.
    let max_width_px = Dp(640.0).to_px();
    let is_large_screen = parent_width >= max_width_px;

    let sheet_width = if is_large_screen {
        max_width_px
    } else {
        parent_width
    };

    // M3 Spec: Top margin 56dp or 72dp.
    let top_margin = if is_large_screen {
        Dp(56.0).to_px()
    } else {
        Dp(72.0).to_px()
    };
    let max_height = (parent_height - top_margin).max(Px(0));

    let constraint = Constraint {
        width: DimensionValue::Fixed(sheet_width),
        height: DimensionValue::Wrap {
            min: None,
            max: Some(max_height),
        },
    };

    let child_size = match input.measure_child(bottom_sheet_id, &constraint) {
        Ok(s) => s,
        Err(_) => return,
    };

    let (current_is_open, _, drag_offset) = controller_for_measure.with(|c| c.snapshot());
    let y = compute_bottom_sheet_y(
        parent_height,
        child_size.height,
        progress,
        current_is_open,
        drag_offset,
    );

    let x = if is_large_screen {
        (parent_width - child_size.width) / 2
    } else {
        Px(0)
    };

    input.place_child(bottom_sheet_id, PxPosition::new(x, Px(y)));
}

#[derive(Clone)]
struct DragHandlerArgs {
    controller: State<BottomSheetController>,
    on_close: Arc<dyn Fn() + Send + Sync>,
}

#[tessera]
fn drag_handler(args: DragHandlerArgs, child: impl FnOnce() + Send + Sync + 'static) {
    let controller = args.controller;
    let on_close = args.on_close;

    input_handler(Box::new(move |mut input| {
        handle_drag_gestures(controller, &mut input, &on_close);
    }));

    child();
}

fn render_content(
    style: BottomSheetStyle,
    bottom_sheet_content: impl FnOnce() + Send + Sync + 'static,
    controller: State<BottomSheetController>,
    on_close: Arc<dyn Fn() + Send + Sync>,
) {
    let content_wrapper = move || {
        drag_handler(
            DragHandlerArgs {
                controller,
                on_close: on_close.clone(),
            },
            || {
                column(
                    ColumnArgsBuilder::default()
                        .width(DimensionValue::Fill {
                            min: None,
                            max: None,
                        })
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .build()
                        .expect("ColumnArgsBuilder failed"),
                    |scope| {
                        scope.child(|| {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .height(Dp(22.0))
                                    .build()
                                    .expect("SpacerArgsBuilder failed"),
                            );
                        });
                        scope.child(|| {
                            surface(
                                SurfaceArgsBuilder::default()
                                    .style(
                                        use_context::<MaterialColorScheme>()
                                            .get()
                                            .on_surface_variant
                                            .with_alpha(0.4)
                                            .into(),
                                    )
                                    .shape(Shape::capsule())
                                    .width(DimensionValue::Fixed(Dp(32.0).to_px()))
                                    .height(DimensionValue::Fixed(Dp(4.0).to_px()))
                                    .build()
                                    .expect("SurfaceArgsBuilder failed"),
                                || {},
                            );
                        });
                        scope.child(|| {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .height(Dp(22.0))
                                    .build()
                                    .expect("SpacerArgsBuilder failed"),
                            );
                        });

                        scope.child(bottom_sheet_content);
                    },
                );
            },
        );
    };
    match style {
        BottomSheetStyle::Glass => {
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .tint_color(Color::WHITE.with_alpha(0.4))
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .refraction_amount(32.0)
                    .blur_radius(Dp(5.0))
                    .block_input(true)
                    .build()
                    .expect("FluidGlassArgsBuilder failed with required fields set"),
                content_wrapper,
            );
        }
        BottomSheetStyle::Material => {
            surface(
                SurfaceArgsBuilder::default()
                    .style(
                        use_context::<MaterialColorScheme>()
                            .get()
                            .surface_container_low
                            .into(),
                    )
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .block_input(true)
                    .build()
                    .expect("SurfaceArgsBuilder failed with required fields set"),
                content_wrapper,
            );
        }
    }
}

/// # bottom_sheet_provider
///
/// Provides a modal bottom sheet for contextual actions or information.
///
/// # Usage
///
/// Show contextual menus, supplemental information, or simple forms without
/// navigating away from the main screen.
///
/// ## Parameters
///
/// - `args` — configuration for the sheet's appearance and behavior; see
///   [`BottomSheetProviderArgs`].
/// - `main_content` — closure that renders the always-visible base UI.
/// - `bottom_sheet_content` — closure that renders the content of the sheet
///   itself.
///
/// # Examples
///
/// ```
/// use tessera_ui_basic_components::bottom_sheet::{
///     BottomSheetProviderArgsBuilder, bottom_sheet_provider,
/// };
///
/// bottom_sheet_provider(
///     BottomSheetProviderArgsBuilder::default()
///         .is_open(true)
///         .on_close_request(std::sync::Arc::new(|| {}))
///         .build()
///         .unwrap(),
///     || { /* main content */ },
///     || { /* bottom sheet content */ },
/// );
/// ```
#[tessera]
pub fn bottom_sheet_provider(
    args: impl Into<BottomSheetProviderArgs>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    bottom_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: BottomSheetProviderArgs = args.into();
    let controller = remember(|| BottomSheetController::new(args.is_open));

    let current_open = controller.with(|c| c.is_open());
    if args.is_open != current_open {
        if args.is_open {
            controller.with_mut(|c| c.open());
        } else {
            controller.with_mut(|c| c.close());
        }
    }

    bottom_sheet_provider_with_controller(args, controller, main_content, bottom_sheet_content);
}

/// # bottom_sheet_provider_with_controller
///
/// Controlled version of [`bottom_sheet_provider`] that accepts an external
/// controller.
///
/// # Usage
///
/// Show contextual menus, supplemental information, or simple forms without
/// navigating away from the main screen. And also need to control the sheet's
/// open/closed state programmatically via a controller.
///
/// # Parameters
///
/// - `args` — configuration for the sheet's appearance and behavior; see
///   [`BottomSheetProviderArgs`].
/// - `controller` — a [`BottomSheetController`] used to open and close the
///   sheet.
/// - `main_content` — closure that renders the always-visible base UI.
/// - `bottom_sheet_content` — closure that renders the content of the sheet
///   itself.
#[tessera]
pub fn bottom_sheet_provider_with_controller(
    args: impl Into<BottomSheetProviderArgs>,
    controller: State<BottomSheetController>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    bottom_sheet_content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: BottomSheetProviderArgs = args.into();

    main_content();

    // Snapshot state to minimize locking overhead.
    let (is_open, timer_opt, _) = controller.with(|c| c.snapshot());

    if !(is_open || timer_opt.is_some_and(|t| t.elapsed() < ANIM_TIME)) {
        return;
    }

    let on_close_for_keyboard = args.on_close_request.clone();
    let progress = calc_progress_from_timer(timer_opt.as_ref());

    render_scrim(&args, progress, is_open);

    let keyboard_closure = make_keyboard_closure(on_close_for_keyboard);
    input_handler(keyboard_closure);

    render_content(
        args.style,
        bottom_sheet_content,
        controller,
        args.on_close_request.clone(),
    );

    let controller_for_measure = controller;
    let measure_closure = Box::new(move |input: &tessera_ui::MeasureInput<'_>| {
        let main_content_id = input.children_ids[0];
        let main_content_size = input.measure_child(main_content_id, input.parent_constraint)?;
        input.place_child(main_content_id, PxPosition::new(Px(0), Px(0)));

        if input.children_ids.len() > 1 {
            let scrim_id = input.children_ids[1];
            input.measure_child(scrim_id, input.parent_constraint)?;
            input.place_child(scrim_id, PxPosition::new(Px(0), Px(0)));
        }

        place_bottom_sheet_if_present(input, controller_for_measure, progress);

        // Return main content size to satisfy closure type.
        Ok(main_content_size)
    });
    measure(measure_closure);
}
