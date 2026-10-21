//! Adaptive carousels with smoothly masked, full-size content.
//!
//! ## Usage
//!
//! Browse photos, media, and featured content in a scrollable collection.

use std::{fmt, sync::Arc};

use tessera_foundation::gesture::{DragAxis, DragRecognizer, DragSettings};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, ClipRegion, ClipShape,
    ComputedData, Constraint, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    FocusProperties, FocusRequester, KeyboardInput, KeyboardInputModifierNode, LayoutResult,
    MeasurementError, Modifier, PointerEventPass, PointerInput, PointerInputModifierNode, Px,
    PxPosition, RenderSlotWith, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    dp::scale_factor,
    key,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    normalize_platform_scroll_delta, receive_frame_nanos, remember, tessera, winit,
};

use crate::{
    animation::Animatable,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::{ResolvedShape, RoundedCorner, Shape},
};

use strategy::{Strategy, StrategyConfig};
use variable::normalized_ratio;

mod strategy;
mod variable;

/// Arrangement of large and partially masked carousel items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselVariant {
    /// Fit a mixture of large, medium, and small items to the viewport.
    #[default]
    MultiBrowse,
    /// Preserve a fixed item size with a partially visible trailing item.
    Uncontained,
    /// Preserve independently sized portrait, landscape, and square items.
    UncontainedMultiAspectRatio,
    /// Emphasize large items with a small trailing item.
    Hero,
    /// Center large items between small neighbors, shifting at collection ends.
    CenteredHero,
    /// Show one edge-to-edge item per page in a vertical feed.
    FullScreen,
}

/// Behavior after a drag ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarouselFlingBehavior {
    /// Snap with at most one additional item of momentum after release.
    SingleAdvance,
    /// Allow momentum to cross several items before snapping.
    MultiAdvance,
    /// Decelerate without snapping.
    Free,
}

/// Information supplied to an item's content slot.
///
/// Read widths inside item content to respond to scrolling and viewport
/// changes.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct CarouselItemInfo {
    /// Zero-based index of the item.
    pub index: usize,
    controller: Option<State<CarouselController>>,
}

impl fmt::Debug for CarouselItemInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CarouselItemInfo")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl CarouselItemInfo {
    /// Returns the full content width, independent of the current mask.
    pub fn full_width(&self) -> Dp {
        self.widths().0
    }

    /// Returns the current visible width, updating content as the mask changes.
    pub fn masked_width(&self) -> Dp {
        self.widths().1
    }

    fn widths(&self) -> (Dp, Dp) {
        self.controller
            .map(|controller| {
                controller.with(|state| {
                    let density = state.config.map(|config| config.density).unwrap_or(1.0);
                    let full = state
                        .strategy
                        .as_ref()
                        .map(|strategy| strategy.item_width(self.index))
                        .unwrap_or(180.0 * density);
                    let width = state
                        .masks
                        .binary_search_by_key(&self.index, |mask| mask.0)
                        .ok()
                        .map(|position| {
                            let (_, center, width) = state.masks[position];
                            visible_mask_width(center, width, state.viewport_width)
                        })
                        .unwrap_or_else(|| if state.strategy.is_some() { 0.0 } else { full });
                    let (full, width) = if state
                        .config
                        .is_some_and(|config| config.variant == CarouselVariant::FullScreen)
                    {
                        (state.viewport_width, state.viewport_width)
                    } else {
                        (full, width)
                    };
                    (Dp((full / density) as f64), Dp((width / density) as f64))
                })
            })
            .unwrap_or_default()
    }
}

fn visible_mask_width(center: f32, width: f32, viewport: f32) -> f32 {
    ((center + width * 0.5).min(viewport.max(0.0)) - (center - width * 0.5).max(0.0)).max(0.0)
}

/// Persistent carousel scroll position and animation state.
///
/// Share through `State<CarouselController>` to navigate programmatically and
/// observe the current item. Requests made before layout are applied on mount.
#[derive(Clone)]
pub struct CarouselController {
    current_item: usize,
    viewport_width: f32,
    aspect_ratios: Arc<[f32]>,
    aspect_revision: u64,
    behavior: CarouselFlingBehavior,
    motion: Animatable,
    free_velocity: f32,
    strategy: Option<Arc<Strategy>>,
    config: Option<StrategyConfig>,
    masks: Vec<(usize, f32, f32)>,
    pending: Option<(usize, bool)>,
    dragging: bool,
    drag_velocity: f32,
    last_input: Option<u64>,
    last_frame: Option<u64>,
    wheel_idle: Option<u64>,
}

impl Default for CarouselController {
    fn default() -> Self {
        Self::new(0)
    }
}

impl CarouselController {
    /// Creates a controller positioned at `initial_item`.
    pub fn new(initial_item: usize) -> Self {
        Self {
            current_item: initial_item,
            viewport_width: 0.0,
            aspect_ratios: Arc::from([]),
            aspect_revision: 0,
            behavior: CarouselFlingBehavior::SingleAdvance,
            motion: resting_motion(0.0),
            free_velocity: 0.0,
            strategy: None,
            config: None,
            masks: Vec::new(),
            pending: Some((initial_item, false)),
            dragging: false,
            drag_velocity: 0.0,
            last_input: None,
            last_frame: None,
            wheel_idle: None,
        }
    }

    /// Returns the current or programmatically requested item index.
    pub fn current_item(&self) -> usize {
        self.current_item
    }

    /// Returns the current scroll offset in density-independent pixels.
    pub fn scroll_offset(&self) -> Dp {
        Dp((self.motion.value() / self.config.map(|config| config.density).unwrap_or(1.0)) as f64)
    }

    /// Returns whether scrolling or a snap animation is active.
    pub fn is_scrolling(&self) -> bool {
        self.dragging
            || self.free_velocity.abs() > 0.1
            || !self.motion.is_idle()
            || self.wheel_idle.is_some()
    }

    /// Moves immediately to an item, clamping its index when layout is known.
    pub fn jump_to_item(&mut self, item: usize) {
        self.request(item, false);
    }

    /// Animates to an item, clamping its index when layout is known.
    pub fn animate_to_item(&mut self, item: usize) {
        self.request(item, true);
    }

    fn navigation_required(&self, item: usize) -> bool {
        if item != self.current_item || self.dragging || self.needs_frame() {
            return true;
        }
        match (&self.strategy, self.config) {
            (Some(strategy), Some(config)) => {
                (self.motion.value() - strategy.snap(item, config.count)).abs() > 0.01
            }
            _ => false,
        }
    }

    fn request(&mut self, item: usize, animate: bool) {
        let now = if animate {
            tessera_ui::current_frame_nanos()
        } else {
            0
        };
        self.request_at(item, animate, now);
    }

    fn request_at(&mut self, item: usize, animate: bool, now: u64) {
        self.dragging = false;
        self.free_velocity = 0.0;
        self.wheel_idle = None;
        self.last_frame = animate.then_some(now);
        self.current_item = self
            .config
            .map(|config| item.min(config.count.saturating_sub(1)))
            .unwrap_or(item);
        if let (Some(strategy), Some(config)) = (&self.strategy, self.config) {
            let offset = strategy.snap(self.current_item, config.count);
            if animate {
                self.motion.set_target(offset);
            } else {
                self.motion = resting_motion(offset);
            }
            self.pending = None;
            self.refresh_masks();
        } else {
            self.pending = Some((item, animate));
        }
    }

    fn configure(&mut self, config: StrategyConfig) {
        if self.config == Some(config) {
            return;
        }
        self.config = Some(config);
        if config.viewport <= 0.0
            || config.count == 0
            || (config.variant == CarouselVariant::UncontainedMultiAspectRatio
                && config.item_height <= 0.0)
        {
            self.strategy = None;
            self.motion = resting_motion(0.0);
            self.masks.clear();
            if config.count == 0 {
                self.current_item = 0;
            } else {
                self.current_item = self.current_item.min(config.count - 1);
                self.pending.get_or_insert((self.current_item, false));
            }
            self.stop_input();
            return;
        }
        self.strategy = Some(Arc::new(Strategy::new(config, &self.aspect_ratios)));
        let requested = self
            .pending
            .take()
            .map(|request| request.0)
            .unwrap_or(self.current_item);
        self.request(requested, false);
    }

    fn refresh_masks(&mut self) {
        self.masks = match (&self.strategy, self.config) {
            (Some(strategy), Some(config)) => strategy.masks(self.motion.value(), config.count),
            _ => Vec::new(),
        };
    }

    fn nearest(&self, offset: f32) -> usize {
        let (Some(strategy), Some(config)) = (&self.strategy, self.config) else {
            return 0;
        };
        if config.count == 0 {
            return 0;
        }
        if let Some(index) = strategy.variable_nearest(offset, config.count) {
            return index;
        }
        let approximate = (offset / (strategy.item_size + strategy.spacing))
            .round()
            .max(0.0) as usize;
        let radius = (strategy.viewport / strategy.item_size).ceil().max(1.0) as usize + 4;
        let start = approximate.saturating_sub(radius).min(config.count - 1);
        let end = approximate
            .saturating_add(radius)
            .saturating_add(1)
            .min(config.count);
        (start..end)
            .min_by(|left, right| {
                (strategy.snap(*left, config.count) - offset)
                    .abs()
                    .total_cmp(&(strategy.snap(*right, config.count) - offset).abs())
            })
            .unwrap_or(start)
    }

    fn move_by(&mut self, delta: f32) -> bool {
        let (Some(strategy), Some(config)) = (&self.strategy, self.config) else {
            return false;
        };
        let next = (self.motion.value() + delta).clamp(0.0, strategy.max_scroll(config.count));
        if next == self.motion.value() {
            return false;
        }
        self.motion = resting_motion(next);
        self.current_item = self.nearest(next);
        self.refresh_masks();
        true
    }

    fn stop_input(&mut self) {
        self.dragging = false;
        self.drag_velocity = 0.0;
        self.free_velocity = 0.0;
        self.wheel_idle = None;
        self.last_input = None;
    }

    fn end_drag(&mut self, behavior: CarouselFlingBehavior, now: u64, cancelled: bool) {
        self.dragging = false;
        let velocity = if cancelled
            || self
                .last_input
                .is_none_or(|last| now.saturating_sub(last) > 100_000_000)
        {
            0.0
        } else {
            self.drag_velocity
        };
        self.drag_velocity = 0.0;
        match behavior {
            CarouselFlingBehavior::Free => {
                self.free_velocity = velocity;
                self.last_frame = Some(now);
            }
            _ => {
                let current = self.nearest(self.motion.value());
                let mut target = self.nearest(self.motion.value() + velocity / 6.0);
                if behavior == CarouselFlingBehavior::SingleAdvance {
                    target = target.clamp(current.saturating_sub(1), current.saturating_add(1));
                }
                let threshold = 50.0 * self.config.map(|config| config.density).unwrap_or(1.0);
                if target == current && velocity.abs() > threshold {
                    target = if velocity > 0.0 {
                        current.saturating_add(1)
                    } else {
                        current.saturating_sub(1)
                    };
                }
                self.motion = self.motion.with_velocity(velocity);
                self.request_at(target, true, now);
            }
        }
    }

    fn needs_frame(&self) -> bool {
        !self.dragging
            && (self.free_velocity.abs() > 0.1
                || !self.motion.is_idle()
                || self.wheel_idle.is_some())
    }

    fn tick(&mut self, now: u64) {
        let dt = self
            .last_frame
            .map(|last| now.saturating_sub(last) as f32 * 1.0e-9)
            .unwrap_or(1.0 / 60.0)
            .clamp(0.0, 0.05);
        self.last_frame = Some(now);
        if self.dragging {
            return;
        }
        if dt <= 0.0 {
            return;
        }
        if self
            .wheel_idle
            .is_some_and(|last| now.saturating_sub(last) >= 120_000_000)
        {
            self.wheel_idle = None;
            if self.behavior != CarouselFlingBehavior::Free {
                self.request_at(self.nearest(self.motion.value()), true, now);
            }
        }
        if self.free_velocity.abs() > 0.1 {
            let decay = (-6.0 * dt).exp();
            let delta = self.free_velocity * (1.0 - decay) / 6.0;
            if !self.move_by(delta) {
                self.free_velocity = 0.0;
            } else {
                self.free_velocity *= decay;
            }
            if self.free_velocity.abs() <= 0.1 {
                self.free_velocity = 0.0;
            }
        } else if !self.motion.is_idle() {
            self.motion.tick(dt);
            if let (Some(strategy), Some(config)) = (&self.strategy, self.config) {
                let position = self
                    .motion
                    .value()
                    .clamp(0.0, strategy.max_scroll(config.count));
                if position != self.motion.value() {
                    let target = self.motion.target();
                    self.motion = resting_motion(position);
                    self.motion.set_target(target);
                }
            }
            if (self.motion.value() - self.motion.target()).abs() < 0.02
                && self.motion.velocity().abs() < 0.2
            {
                self.motion = resting_motion(self.motion.target());
            }
            self.refresh_masks();
        }
    }
}

fn resting_motion(position: f32) -> Animatable {
    Animatable::spring(position, 200.0f32.sqrt(), 1.0)
}

#[derive(Clone, PartialEq)]
struct CarouselLayout {
    controller: State<CarouselController>,
    config: StrategyConfig,
    height: Px,
    indices: Vec<usize>,
    offset: f32,
    reverse: bool,
}

impl tessera_ui::LayoutModifierNode for CarouselLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = input.parent_constraint();
        let width = parent.width().resolve_max().ok_or_else(|| {
            MeasurementError::MeasureFnFailed("Carousel requires a bounded width".into())
        })?;
        let height = parent.height().clamp(self.height);
        let vertical = self.config.variant == CarouselVariant::FullScreen;
        let config = StrategyConfig {
            viewport: if vertical {
                height.to_f32()
            } else {
                width.to_f32()
            },
            padding: self
                .config
                .padding
                .map(|padding| padding.min(width.to_f32().max(0.0))),
            item_height: if self.config.variant == CarouselVariant::UncontainedMultiAspectRatio {
                height.to_f32()
            } else {
                0.0
            },
            ..self.config
        };
        if self
            .controller
            .with(|state| state.config != Some(config) || state.viewport_width != width.to_f32())
        {
            self.controller.with_mut(|state| {
                state.viewport_width = width.to_f32();
                state.configure(config);
            });
        }
        let (strategy, masks) = self
            .controller
            .with(|state| (state.strategy.clone(), state.masks.clone()));
        let children = input.children();
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (&child, &index) in children.iter().zip(&self.indices) {
            let item_size = strategy
                .as_ref()
                .map(|strategy| strategy.item_width(index))
                .unwrap_or(0.0);
            let constraint = Constraint::new(
                AxisConstraint::exact(if vertical {
                    width
                } else {
                    Px(item_size.round() as i32)
                }),
                AxisConstraint::exact(height),
            );
            child.measure(input, &constraint)?;
            let center = masks
                .binary_search_by_key(&index, |mask| mask.0)
                .ok()
                .map(|position| masks[position].1)
                .unwrap_or(-item_size * 2.0);
            let center = if self.reverse {
                (if vertical {
                    height.to_f32()
                } else {
                    width.to_f32()
                }) - center
            } else {
                center
            };
            result.place_child(
                child,
                if vertical {
                    PxPosition::from_f32_xy(0.0, center - item_size.round() * 0.5)
                } else {
                    PxPosition::from_f32_xy(center - item_size.round() * 0.5, 0.0)
                },
            );
        }
        Ok(result)
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.height == other.height
            && self.indices == other.indices
            && self.reverse == other.reverse
    }

    fn place_children(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let (strategy, masks) = self
            .controller
            .with(|state| (state.strategy.clone(), state.masks.clone()));
        let children = input.children();
        if children.len() != self.indices.len() {
            return None;
        }
        let mut result = LayoutResult::new(input.size());
        let vertical = self.config.variant == CarouselVariant::FullScreen;
        for (&child, &index) in children.iter().zip(&self.indices) {
            let item_size = strategy
                .as_ref()
                .map(|strategy| strategy.item_width(index))
                .unwrap_or(0.0);
            let center = masks
                .binary_search_by_key(&index, |mask| mask.0)
                .ok()
                .map(|position| masks[position].1)
                .unwrap_or(-item_size * 2.0);
            let center = if self.reverse {
                (if vertical {
                    input.size().height.to_f32()
                } else {
                    input.size().width.to_f32()
                }) - center
            } else {
                center
            };
            result.place_child(
                child,
                if vertical {
                    PxPosition::from_f32_xy(0.0, center - item_size.round() * 0.5)
                } else {
                    PxPosition::from_f32_xy(center - item_size.round() * 0.5, 0.0)
                },
            );
        }
        Some(result.into_placement_result())
    }
}

#[derive(Clone, Copy)]
struct ItemMask {
    controller: State<CarouselController>,
    index: usize,
    shape: Shape,
}

impl DrawModifierNode for ItemMask {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        let width = self.controller.with(|state| {
            state
                .masks
                .binary_search_by_key(&self.index, |mask| mask.0)
                .ok()
                .map(|position| state.masks[position].2)
                .unwrap_or(0.0)
        });
        let vertical = self.controller.with(|state| {
            state
                .config
                .is_some_and(|config| config.variant == CarouselVariant::FullScreen)
        });
        let mut metadata = ctx.render_input.metadata_mut();
        let size = metadata.computed_data().expect("measured carousel item");
        let shape = match self.shape.resolve_for_size(size.into()) {
            ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } => ClipShape::RoundedRectangle {
                radii: corner_radii,
                exponents: corner_g2,
            },
            ResolvedShape::Ellipse => ClipShape::Ellipse,
        };
        metadata.set_clip_region(ClipRegion {
            position: if vertical {
                PxPosition::from_f32_xy(0.0, (size.height.to_f32() - width) * 0.5)
            } else {
                PxPosition::from_f32_xy((size.width.to_f32() - width) * 0.5, 0.0)
            },
            size: if vertical {
                [size.width.to_f32(), width]
            } else {
                [width, size.height.to_f32()]
            },
            shape,
        });
        content.draw(ctx.render_input);
    }
}

struct CarouselInput {
    controller: State<CarouselController>,
    recognizer: State<DragRecognizer>,
    focus: FocusRequester,
    enabled: bool,
    reverse: bool,
    vertical: bool,
    behavior: CarouselFlingBehavior,
}

impl PointerInputModifierNode for CarouselInput {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if !self.enabled || input.pass != PointerEventPass::Main {
            return;
        }
        let inside = input.cursor_position_rel.is_some_and(|point| {
            point.x >= 0.0
                && point.y >= 0.0
                && point.x < input.computed_data.width.to_f32()
                && point.y < input.computed_data.height.to_f32()
        });
        let now = tessera_ui::current_frame_nanos();
        if inside {
            for change in input.pointer_changes.iter_mut() {
                if change.is_consumed() {
                    continue;
                }
                if matches!(
                    change.content,
                    tessera_ui::CursorEventContent::Pressed(tessera_ui::PressKeyEventType::Left)
                ) {
                    self.controller.with_mut(|state| {
                        state.free_velocity = 0.0;
                        state.wheel_idle = None;
                        state.motion = resting_motion(state.motion.value());
                        state.current_item = state.nearest(state.motion.value());
                    });
                }
                if let tessera_ui::CursorEventContent::Scroll(scroll) = &mut change.content {
                    let (delta_x, delta_y) = normalize_platform_scroll_delta(
                        scroll.delta_x,
                        scroll.delta_y,
                        scroll.unit,
                        scroll.source,
                    );
                    let delta = if self.vertical { delta_y } else { delta_x };
                    if delta.abs() > 0.001 {
                        let moved = self.controller.with_mut(|state| {
                            state.free_velocity = 0.0;
                            let moved = state.move_by(if self.reverse { delta } else { -delta });
                            if moved && self.behavior != CarouselFlingBehavior::Free {
                                state.wheel_idle = Some(now);
                            }
                            moved
                        });
                        if moved {
                            let remaining = if self.vertical {
                                scroll.delta_y = 0.0;
                                scroll.delta_x
                            } else {
                                scroll.delta_x = 0.0;
                                scroll.delta_y
                            };
                            if remaining.abs() < 0.001 {
                                change.consume();
                            }
                        }
                    }
                }
            }
        }
        let drag = self.recognizer.with_mut(|recognizer| {
            recognizer.update(input.pass, input.pointer_changes.as_mut_slice(), inside)
        });
        if drag.started {
            self.focus.request_focus();
            self.controller.with_mut(|state| {
                state.stop_input();
                state.dragging = true;
                state.motion = resting_motion(state.motion.value());
            });
        }
        if drag.updated {
            self.controller.with_mut(|state| {
                let delta = if self.vertical {
                    drag.delta_y
                } else {
                    drag.delta_x
                }
                .to_f32()
                    * if self.reverse { 1.0 } else { -1.0 };
                let dt = state
                    .last_input
                    .map(|last| now.saturating_sub(last) as f32 * 1.0e-9)
                    .unwrap_or(1.0 / 60.0)
                    .clamp(0.004, 0.1);
                state.drag_velocity = state.drag_velocity * 0.6 + delta / dt * 0.4;
                state.last_input = Some(now);
                state.move_by(delta);
            });
        }
        let cancelled = !self.recognizer.with(DragRecognizer::is_dragging)
            && self.controller.with(|state| state.dragging)
            && !drag.ended;
        if drag.ended || cancelled {
            self.controller
                .with_mut(|state| state.end_drag(self.behavior, now, cancelled));
        }
    }
}

impl KeyboardInputModifierNode for CarouselInput {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        use winit::{
            event::ElementState,
            keyboard::{Key, NamedKey},
        };

        if !self.enabled
            || input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
        {
            return;
        }
        for event in input.keyboard_events.iter() {
            if event.state != ElementState::Pressed {
                continue;
            }
            let Some(count) = self
                .controller
                .with(|state| state.config.map(|config| config.count))
            else {
                continue;
            };
            let current = self.controller.with(CarouselController::current_item);
            let target = match &event.logical_key {
                Key::Named(NamedKey::Home) => 0,
                Key::Named(NamedKey::End) => count.saturating_sub(1),
                Key::Named(key)
                    if *key
                        == if self.vertical {
                            NamedKey::ArrowDown
                        } else {
                            NamedKey::ArrowRight
                        } =>
                {
                    if self.reverse {
                        current.saturating_sub(1)
                    } else {
                        current.saturating_add(1).min(count.saturating_sub(1))
                    }
                }
                Key::Named(key)
                    if *key
                        == if self.vertical {
                            NamedKey::ArrowUp
                        } else {
                            NamedKey::ArrowLeft
                        } =>
                {
                    if self.reverse {
                        current.saturating_add(1).min(count.saturating_sub(1))
                    } else {
                        current.saturating_sub(1)
                    }
                }
                _ => continue,
            };
            if self
                .controller
                .with(|state| state.navigation_required(target))
            {
                self.controller
                    .with_mut(|state| state.animate_to_item(target));
                self.focus.request_focus();
                input.block_keyboard();
            }
            break;
        }
    }
}

struct CarouselSemantics {
    controller: State<CarouselController>,
    enabled: bool,
    reverse: bool,
    vertical: bool,
}

impl SemanticsModifierNode for CarouselSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        let (item, count, extent) = self.controller.with(|state| {
            (
                state.current_item,
                state.config.map(|config| config.count).unwrap_or(0),
                state
                    .strategy
                    .as_ref()
                    .map(|strategy| {
                        strategy.max_scroll(state.config.map(|config| config.count).unwrap_or(0))
                    })
                    .unwrap_or(0.0),
            )
        });
        node.value = (count > 0).then(|| format!("{} of {count}", item + 1));
        if !self.enabled || extent <= 0.0 {
            *handler = None;
            return;
        }
        let (mut backward, mut forward) = if self.vertical {
            (Action::ScrollUp, Action::ScrollDown)
        } else {
            (Action::ScrollLeft, Action::ScrollRight)
        };
        if self.reverse {
            std::mem::swap(&mut backward, &mut forward);
        }
        if item > 0 {
            node.actions.push(backward);
        }
        if item + 1 < count {
            node.actions.push(forward);
        }
        let controller = self.controller;
        *handler = Some(Box::new(move |action| {
            let advance = if action == forward {
                true
            } else if action == backward {
                false
            } else {
                return;
            };
            controller.with_mut(|state| {
                let next = if advance {
                    state.current_item.saturating_add(1)
                } else {
                    state.current_item.saturating_sub(1)
                };
                state.animate_to_item(next);
            });
        }));
    }
}

#[derive(Clone, PartialEq)]
struct CarouselRenderState {
    count: usize,
    shape: Shape,
    content: Option<RenderSlotWith<CarouselItemInfo>>,
}

fn visible_indices(state: &CarouselController, count: usize) -> Vec<usize> {
    if state.strategy.is_some() {
        state
            .masks
            .iter()
            .map(|mask| mask.0)
            .filter(|index| *index < count)
            .collect()
    } else {
        (state.current_item.saturating_sub(2)..state.current_item.saturating_add(6).min(count))
            .collect()
    }
}

#[tessera]
fn carousel_item(
    index: usize,
    controller: State<CarouselController>,
    render_state: State<CarouselRenderState>,
) {
    let shape = render_state.with(|state| state.shape);
    let vertical = controller.with(|state| {
        state
            .config
            .is_some_and(|config| config.variant == CarouselVariant::FullScreen)
    });
    layout()
        .modifier(
            Modifier::new()
                .semantics(SemanticsArgs {
                    role: Some(Role::ListItem),
                    merge_descendants: true,
                    collection_item_info: Some(if vertical {
                        (index, 1, 0, 1, false)
                    } else {
                        (0, 1, index, 1, false)
                    }),
                    ..Default::default()
                })
                .push_draw(ItemMask {
                    controller,
                    index,
                    shape,
                }),
        )
        .child(move || {
            if let Some(content) = render_state.with(|state| state.content) {
                let info = CarouselItemInfo {
                    index,
                    controller: Some(controller),
                };
                content.render(info);
            }
        });
}

/// # carousel
///
/// Browse media collections with adaptive masks and scroll snapping.
///
/// ## Usage
///
/// Browse photo albums, recommendations, and featured media collections.
///
/// ## Parameters
///
/// - `item_count` — number of items in the collection.
/// - `modifier`, `height` — viewport placement, bounded width, and height
///   (180dp by default).
/// - `variant` — multi-browse, uncontained, mixed-aspect uncontained, hero,
///   centered hero, or full-screen arrangement.
/// - `preferred_item_width` — target width; defaults to 180dp for
///   multi-browse/uncontained and the viewport for hero layouts; ignored in
///   full-screen and mixed-aspect modes.
/// - `item_aspect_ratios` — width/height ratios for mixed-aspect items, bounded
///   to 9:16–16:9. Missing or invalid ratios use square items.
/// - `item_spacing` — spacing between items: 16dp for full-screen, 8dp for
///   mixed-aspect uncontained, and zero otherwise.
/// - `min_small_item_width`, `max_small_item_width` — small-item bounds, 40dp
///   and 56dp by default.
/// - `content_padding_start`, `content_padding_end` — padding at collection
///   ends; mixed-aspect uncontained defaults to 16dp at the start and zero at
///   the end. Ignored in full-screen mode.
/// - `shape` — mask shape; defaults to rectangular full-screen pages and
///   circular 28dp corners otherwise.
/// - `controller` — optional external scroll controller.
/// - `focus_requester` — optional handle for focusing viewport keyboard
///   navigation.
/// - `user_scroll_enabled` — enables gestures, keyboard, and accessibility
///   navigation.
/// - `reverse_layout` — reverses the layout and its navigation direction.
/// - `fling_behavior` — single-advance snapping by default, or free scrolling
///   for uncontained layouts. Full-screen pages always snap.
/// - `accessibility_label` — localized collection name.
/// - `content` — item slot, measured at its full width regardless of its mask.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     carousel::{CarouselController, carousel},
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
///
/// let mut controller = CarouselController::new(2);
/// controller.jump_to_item(4);
/// assert_eq!(controller.current_item(), 4);
///
/// #[tessera]
/// fn photos() {
///     let controller = remember(|| CarouselController::new(2));
///     assert_eq!(controller.with(CarouselController::current_item), 2);
///     carousel(12).controller(controller).content(|item| {
///         text().content(format!("Photo {}", item.index + 1));
///     });
/// }
/// ```
#[tessera]
pub fn carousel(
    item_count: usize,
    modifier: Option<Modifier>,
    height: Option<Dp>,
    variant: Option<CarouselVariant>,
    preferred_item_width: Option<Dp>,
    #[prop(into)] item_aspect_ratios: Option<Arc<[f32]>>,
    item_spacing: Option<Dp>,
    min_small_item_width: Option<Dp>,
    max_small_item_width: Option<Dp>,
    content_padding_start: Option<Dp>,
    content_padding_end: Option<Dp>,
    shape: Option<Shape>,
    controller: Option<State<CarouselController>>,
    focus_requester: Option<FocusRequester>,
    user_scroll_enabled: Option<bool>,
    reverse_layout: Option<bool>,
    fling_behavior: Option<CarouselFlingBehavior>,
    #[prop(into)] accessibility_label: Option<String>,
    content: Option<RenderSlotWith<CarouselItemInfo>>,
) {
    let variant = variant.unwrap_or_default();
    let vertical = variant == CarouselVariant::FullScreen;
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(CarouselController::default)
    };
    let recognizer = remember(|| {
        DragRecognizer::new(DragSettings {
            axis: Some(if vertical {
                DragAxis::Vertical
            } else {
                DragAxis::Horizontal
            }),
            ..Default::default()
        })
    });
    let previous_vertical = remember(|| vertical);
    if previous_vertical.get() != vertical {
        previous_vertical.set(vertical);
        recognizer.set(DragRecognizer::new(DragSettings {
            axis: Some(if vertical {
                DragAxis::Vertical
            } else {
                DragAxis::Horizontal
            }),
            ..Default::default()
        }));
    }
    let focus = if let Some(focus) = focus_requester {
        focus
    } else {
        remember(FocusRequester::new).get()
    };
    if variant == CarouselVariant::UncontainedMultiAspectRatio {
        let ratios = item_aspect_ratios.as_deref().unwrap_or(&[]);
        let count = ratios.len().min(item_count);
        let changed = controller.with(|state| {
            state.aspect_ratios.len() != count
                || state
                    .aspect_ratios
                    .iter()
                    .zip(ratios)
                    .any(|(old, ratio)| *old != normalized_ratio(*ratio))
        });
        if changed {
            controller.with_mut(|state| {
                state.aspect_ratios = ratios
                    .iter()
                    .take(count)
                    .map(|ratio| normalized_ratio(*ratio))
                    .collect::<Vec<_>>()
                    .into();
                state.aspect_revision = state.aspect_revision.wrapping_add(1);
            });
        }
    }
    let mut behavior = fling_behavior.unwrap_or(
        if matches!(
            variant,
            CarouselVariant::Uncontained | CarouselVariant::UncontainedMultiAspectRatio
        ) {
            CarouselFlingBehavior::Free
        } else {
            CarouselFlingBehavior::SingleAdvance
        },
    );
    if vertical && behavior == CarouselFlingBehavior::Free {
        behavior = CarouselFlingBehavior::SingleAdvance;
    }
    if controller.with(|state| state.behavior != behavior) {
        controller.with_mut(|state| state.behavior = behavior);
    }
    let enabled = user_scroll_enabled.unwrap_or(true);
    let reverse = reverse_layout.unwrap_or(false);
    if !enabled
        && controller.with(|state| {
            state.dragging || state.wheel_idle.is_some() || state.free_velocity != 0.0
        })
    {
        controller.with_mut(CarouselController::stop_input);
        recognizer.set(DragRecognizer::new(DragSettings {
            axis: Some(if vertical {
                DragAxis::Vertical
            } else {
                DragAxis::Horizontal
            }),
            ..Default::default()
        }));
    }
    if controller.with(CarouselController::needs_frame) {
        receive_frame_nanos(move |now| {
            controller.with_mut(|state| state.tick(now));
            if controller.with(CarouselController::needs_frame) {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let density = scale_factor() as f32;
    let min_small = finite_dp(min_small_item_width.unwrap_or(Dp(40.0)), 40.0).max(1.0);
    let max_small = finite_dp(max_small_item_width.unwrap_or(Dp(56.0)), 56.0).max(min_small);
    let preferred = preferred_item_width
        .map(|width| finite_dp(width, 180.0).max(1.0))
        .or_else(|| {
            matches!(
                variant,
                CarouselVariant::MultiBrowse | CarouselVariant::Uncontained
            )
            .then_some(180.0 * density)
        });
    let height = Px(finite_dp(height.unwrap_or(Dp(180.0)), 180.0).round() as i32);
    let variable = variant == CarouselVariant::UncontainedMultiAspectRatio;
    let config = StrategyConfig {
        variant,
        viewport: 0.0,
        preferred,
        spacing: finite_dp(
            item_spacing.unwrap_or(Dp(if variant == CarouselVariant::FullScreen {
                16.0
            } else if variable {
                8.0
            } else {
                0.0
            })),
            0.0,
        ),
        density,
        count: item_count,
        item_height: if variable {
            controller.with(|state| {
                state
                    .config
                    .filter(|config| config.variant == variant)
                    .map(|config| config.item_height)
                    .unwrap_or(height.to_f32())
            })
        } else {
            0.0
        },
        aspect_revision: if variable {
            controller.with(|state| state.aspect_revision)
        } else {
            0
        },
        small_range: [min_small, max_small],
        padding: [
            finite_dp(
                content_padding_start.unwrap_or(Dp(if variable { 16.0 } else { 0.0 })),
                0.0,
            ),
            finite_dp(content_padding_end.unwrap_or(Dp(0.0)), 0.0),
        ],
    };
    if let Some(viewport) = controller.with(|state| {
        state
            .config
            .map(|config| config.viewport)
            .filter(|viewport| *viewport > 0.0)
    }) {
        let next = StrategyConfig {
            viewport,
            padding: config.padding.map(|padding| padding.min(viewport)),
            ..config
        };
        if controller.with(|state| state.config != Some(next)) {
            controller.with_mut(|state| state.configure(next));
        }
    }
    let (indices, offset) =
        controller.with(|state| (visible_indices(state, item_count), state.motion.value()));
    let policy = CarouselLayout {
        controller,
        config,
        height,
        indices,
        offset,
        reverse,
    };
    let scroll_range = (
        offset as f64,
        0.0,
        controller.with(|state| {
            state
                .strategy
                .as_ref()
                .map(|strategy| strategy.max_scroll(item_count))
                .unwrap_or(0.0)
        }) as f64,
    );
    let modifier = modifier
        .unwrap_or_else(|| Modifier::new().fill_max_width())
        .clip_to_bounds()
        .focusable()
        .focus_requester(focus)
        .focus_properties(
            FocusProperties::new()
                .can_focus(enabled && item_count > 1)
                .can_request_focus(enabled && item_count > 1),
        )
        .semantics(SemanticsArgs {
            role: Some(Role::List),
            merge_descendants: true,
            role_description: Some("carousel".into()),
            label: accessibility_label,
            collection_info: Some(if vertical {
                (item_count, 1, false)
            } else {
                (1, item_count, false)
            }),
            focusable: enabled && item_count > 1,
            scroll_x: (!vertical).then_some(scroll_range),
            scroll_y: vertical.then_some(scroll_range),
            ..Default::default()
        })
        .push_semantics(CarouselSemantics {
            controller,
            enabled,
            reverse,
            vertical,
        })
        .push_pointer_input(CarouselInput {
            controller,
            recognizer,
            focus,
            enabled,
            reverse,
            vertical,
            behavior,
        })
        .push_keyboard_input(CarouselInput {
            controller,
            recognizer,
            focus,
            enabled,
            reverse,
            vertical,
            behavior,
        });
    let corner = RoundedCorner::manual(Dp(28.0), 2.0);
    let shape = shape.unwrap_or(if variant == CarouselVariant::FullScreen {
        Shape::RECTANGLE
    } else {
        Shape::RoundedRectangle {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
        }
    });
    let value = CarouselRenderState {
        count: item_count,
        shape,
        content,
    };
    let render_state = remember(|| value.clone());
    if render_state.with(|state| *state != value) {
        render_state.set(value);
    }
    layout().modifier(modifier).policy(policy).child(move || {
        let count = render_state.with(|state| state.count);
        let indices = controller.with(|state| visible_indices(state, count));
        for index in indices {
            key(index, || {
                carousel_item(index, controller, render_state);
            });
        }
    });
}

fn finite_dp(value: Dp, fallback: f32) -> f32 {
    let pixels = value.to_pixels_f32();
    if pixels.is_finite() {
        pixels.max(0.0)
    } else {
        fallback * scale_factor() as f32
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        Color, MOUSE_POINTER_ID, PressKeyEventType,
        headless::{HeadlessConfig, HeadlessRenderer},
    };

    use crate::{
        boxed::boxed,
        button::button,
        column::column,
        flow_row::flow_row,
        image::{ImageData, image},
        lazy_list::lazy_column,
        scaffold::scaffold,
        spacer::spacer,
        text::text,
        theme::{MaterialTheme, material_theme},
    };

    use super::*;

    #[test]
    fn visible_item_width_intersects_the_viewport_at_both_ends() {
        for density in [1.0, 1.25, 1.5, 2.0] {
            for (center, width, viewport, expected) in [
                (176.0, 320.0, 380.0, 320.0),
                (394.625, 101.25, 380.0, 36.0),
                (83.0, 320.0, 380.0, 243.0),
                (-200.0, 320.0, 380.0, 0.0),
                (600.0, 320.0, 380.0, 0.0),
                (0.0, 320.0, 0.0, 0.0),
            ] {
                assert_eq!(
                    visible_mask_width(center * density, width * density, viewport * density),
                    expected * density
                );
            }
        }
    }

    fn configuration(variant: CarouselVariant, count: usize) -> StrategyConfig {
        StrategyConfig {
            variant,
            viewport: 380.0,
            preferred: Some(186.0),
            spacing: 8.0,
            density: 1.0,
            count,
            small_range: [40.0, 56.0],
            padding: [0.0; 2],
            item_height: 180.0,
            aspect_revision: 0,
        }
    }

    fn settle(state: &mut CarouselController) {
        let start = state.last_frame.unwrap_or(0);
        for frame in 1..=240 {
            state.tick(start + frame * 16_666_667);
        }
        assert!(
            !state.needs_frame(),
            "a settled carousel must stop requesting frames"
        );
    }

    #[test]
    fn prelayout_navigation_clamps_and_resizes_without_losing_the_item() {
        let mut state = CarouselController::new(2);
        state.jump_to_item(7);
        state.configure(configuration(CarouselVariant::MultiBrowse, 10));
        assert_eq!(state.current_item(), 7);
        assert_eq!(
            state.motion.value(),
            state.strategy.as_ref().expect("strategy").snap(7, 10)
        );
        state.configure(StrategyConfig {
            viewport: 0.0,
            ..configuration(CarouselVariant::MultiBrowse, 10)
        });
        assert_eq!(state.current_item(), 7);
        assert!(state.masks.is_empty());
        state.configure(configuration(CarouselVariant::MultiBrowse, 10));
        assert_eq!(state.current_item(), 7);
        state.configure(StrategyConfig {
            viewport: 620.0,
            ..configuration(CarouselVariant::MultiBrowse, 10)
        });
        assert_eq!(state.current_item(), 7);
        state.configure(configuration(CarouselVariant::CenteredHero, 3));
        assert_eq!(state.current_item(), 2);
        state.configure(configuration(CarouselVariant::CenteredHero, 0));
        assert!(state.masks.is_empty());
        assert_eq!(state.current_item(), 0);
        assert!(!state.is_scrolling());
        state.configure(configuration(CarouselVariant::MultiBrowse, 10));
        assert!(!state.masks.is_empty());
    }

    #[test]
    fn navigation_realigns_the_current_item_after_free_scrolling() {
        for variant in [
            CarouselVariant::Uncontained,
            CarouselVariant::UncontainedMultiAspectRatio,
        ] {
            let mut state = CarouselController::default();
            state.configure(configuration(variant, 10));
            state.behavior = CarouselFlingBehavior::Free;
            assert!(!state.navigation_required(0));
            state.move_by(40.0);
            assert_eq!(state.current_item(), 0);
            assert!(state.navigation_required(0));
            state.request_at(0, true, 0);
            settle(&mut state);
            assert_eq!(state.motion.value(), 0.0);
            assert!(!state.navigation_required(0));
            state.dragging = true;
            assert!(state.navigation_required(0));
        }
        let mut state = CarouselController::default();
        state.configure(configuration(CarouselVariant::FullScreen, 3));
        state.jump_to_item(2);
        let end = state.motion.value();
        state.move_by(-40.0);
        assert_eq!(state.current_item(), 2);
        assert!(state.navigation_required(2));
        state.request_at(2, true, 0);
        settle(&mut state);
        assert_eq!(state.motion.value(), end);
        assert!(!state.navigation_required(2));
    }

    #[test]
    fn retargeted_snaps_settle_inside_the_scroll_range() {
        let mut state = CarouselController::default();
        state.configure(configuration(CarouselVariant::MultiBrowse, 100));
        state.animate_to_item(90);
        state.tick(16_666_667);
        let partial = state.motion.value();
        assert!(partial > 0.0);
        state.animate_to_item(2);
        settle(&mut state);
        assert_eq!(state.current_item(), 2);
        assert_eq!(
            state.motion.value(),
            state.strategy.as_ref().expect("strategy").snap(2, 100)
        );
        state.motion = state.motion.with_velocity(-10_000.0);
        state.animate_to_item(0);
        settle(&mut state);
        assert_eq!(state.motion.value(), 0.0);
        state.jump_to_item(usize::MAX);
        assert_eq!(state.current_item(), 99);
        assert_eq!(
            state.motion.value(),
            state.strategy.as_ref().expect("strategy").max_scroll(100)
        );
    }

    #[test]
    fn free_scrolling_keeps_fractional_offsets_and_snap_modes_limit_fling_distance() {
        let mut free = CarouselController::default();
        free.behavior = CarouselFlingBehavior::Free;
        free.configure(configuration(CarouselVariant::Uncontained, 100));
        free.move_by(53.25);
        free.drag_velocity = 400.0;
        free.last_input = Some(1);
        free.end_drag(CarouselFlingBehavior::Free, 1, false);
        settle(&mut free);
        let snap = free
            .strategy
            .as_ref()
            .expect("strategy")
            .snap(free.nearest(free.motion.value()), 100);
        assert!((free.motion.value() - snap).abs() > 1.0);

        let mut single = CarouselController::default();
        single.configure(configuration(CarouselVariant::MultiBrowse, 100));
        single.move_by(30.0);
        single.drag_velocity = 4000.0;
        single.last_input = Some(1);
        single.end_drag(CarouselFlingBehavior::SingleAdvance, 1, false);
        assert_eq!(single.current_item(), 1);
        settle(&mut single);
        assert_eq!(single.current_item(), 1);
    }

    #[test]
    fn wheel_idle_reads_the_latest_fling_setting() {
        let mut state = CarouselController::default();
        state.configure(configuration(CarouselVariant::MultiBrowse, 10));
        state.move_by(53.25);
        state.wheel_idle = Some(0);
        state.behavior = CarouselFlingBehavior::Free;
        settle(&mut state);
        assert_eq!(state.motion.value(), 53.25);
    }

    #[derive(Clone, Copy)]
    struct Settings {
        variant: CarouselVariant,
        count: usize,
        enabled: bool,
        reverse: bool,
        dimmed: bool,
        stress: bool,
        ratios: [f32; 4],
        height: f64,
    }

    thread_local! {
        static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
        static CONTROLLER: Cell<Option<State<CarouselController>>> = const { Cell::new(None) };
        static ACTIVATED: Cell<Option<State<Vec<usize>>>> = const { Cell::new(None) };
        static RASTER: Cell<Option<State<ImageData>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture() {
        let settings = remember(|| Settings {
            variant: CarouselVariant::MultiBrowse,
            count: 10,
            enabled: true,
            reverse: false,
            dimmed: false,
            stress: false,
            ratios: [16.0 / 9.0, 9.0 / 16.0, 1.0, 3.0 / 4.0],
            height: 180.0,
        });
        let controller = remember(CarouselController::default);
        let activated = remember(Vec::<usize>::new);
        let raster = remember(|| {
            let mut pixels = Vec::new();
            for _ in 0..8 {
                for x in 0..16 {
                    pixels.extend(if x == 3 || x == 4 {
                        [0, 255, 0, 255]
                    } else if x < 8 {
                        [255, 0, 0, 255]
                    } else {
                        [0, 0, 255, 255]
                    });
                }
            }
            ImageData::from_rgba(pixels, 16, 8)
        });
        SETTINGS.set(Some(settings));
        CONTROLLER.set(Some(controller));
        ACTIVATED.set(Some(activated));
        RASTER.set(Some(raster));
        material_theme().child(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .children(move || {
                    if settings.with(|settings| settings.stress) {
                        for index in 0..40usize {
                            key(index, || {
                                image().raster(raster.get()).modifier(
                                    Modifier::new().size(Dp(24.0), Dp(24.0)).offset(
                                        Dp((10 + (index % 8) * 28) as f64),
                                        Dp((20 + (index / 8) * 28) as f64),
                                    ),
                                );
                            });
                        }
                    } else {
                        lazy_column()
                            .modifier(Modifier::new().fill_max_size())
                            .estimated_item_size(Dp(280.0))
                            .item(move || {
                                boxed()
                                    .modifier(Modifier::new().size(Dp(400.0), Dp(200.0)))
                                    .children(move || {
                                        let value = settings.get();
                                        carousel(value.count)
                                            .modifier(
                                                Modifier::new()
                                                    .width(Dp(380.0))
                                                    .offset(Dp(10.0), Dp(20.0)),
                                            )
                                            .height(Dp(value.height))
                                            .item_aspect_ratios(Vec::from(value.ratios))
                                            .preferred_item_width(Dp(186.0))
                                            .item_spacing(Dp(8.0))
                                            .variant(value.variant)
                                            .controller(controller)
                                            .user_scroll_enabled(value.enabled)
                                            .reverse_layout(value.reverse)
                                            .accessibility_label("Gallery")
                                            .content(move |info| {
                                                button()
                                                    .modifier(
                                                        Modifier::new().fill_max_size().alpha(
                                                            if settings
                                                                .with(|settings| settings.dimmed)
                                                            {
                                                                0.5
                                                            } else {
                                                                1.0
                                                            },
                                                        ),
                                                    )
                                                    .padding(Dp(0.0))
                                                    .color(Color::TRANSPARENT)
                                                    .shape(Shape::RECTANGLE)
                                                    .accessibility_label(format!(
                                                        "Item {}",
                                                        info.index
                                                    ))
                                                    .on_click(move || {
                                                        activated.with_mut(|items| {
                                                            items.push(info.index)
                                                        })
                                                    })
                                                    .child(move || {
                                                        image().raster(raster.get()).modifier(
                                                            Modifier::new().fill_max_size(),
                                                        );
                                                    });
                                            });
                                    });
                            })
                            .item(|| {
                                text().content("Following carousel");
                            });
                    }
                });
        });
    }

    fn click(renderer: &mut HeadlessRenderer, point: [f32; 2]) {
        let point = PxPosition::from_f32_xy(point[0], point[1]);
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(3);
    }

    fn check_native_carousel(samples: u32) {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 500,
                height: 300,
                sample_count: samples,
                ..Default::default()
            },
        )
        .expect("carousel renderer");
        renderer.render_frames(8);
        let controller = CONTROLLER.get().expect("controller");
        let settings = SETTINGS.get().expect("settings");
        let activated = ACTIVATED.get().expect("activations");
        let initial = renderer.read_pixels().expect("pixels");
        if let Some(directory) = std::env::var_os("TESSERA_CAROUSEL_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).expect("carousel QA directory");
            tessera_ui::headless::write_png_file(
                directory.join(format!("carousel-content-msaa{samples}.png")),
                500,
                300,
                &initial,
            )
            .expect("carousel QA image");
            std::fs::write(
                directory.join(format!("carousel-snapshot-msaa{samples}.json")),
                renderer.snapshot().to_string(),
            )
            .expect("snapshot file");
        }
        let pixel: fn(&[u8], usize, usize) -> &[u8] =
            |pixels, x, y| &pixels[(y * 500 + x) * 4..(y * 500 + x) * 4 + 4];
        for center in [103usize, 265, 362] {
            assert_eq!(
                pixel(&initial, center - 10, 100),
                [255, 0, 0, 255],
                "full-size content must remain centered inside every mask at {center}; configuration: {:?}; masks: {:?}",
                controller.with(|state| state.config),
                controller.with(|state| state.masks.clone())
            );
            assert_eq!(pixel(&initial, center + 10, 100), [0, 0, 255, 255]);
        }
        assert_eq!(pixel(&initial, 56, 100), [0, 255, 0, 255]);
        assert_eq!(
            pixel(&initial, 218, 100),
            [0, 255, 0, 255],
            "masked images must preserve the full-width landmark position"
        );
        settings.with_mut(|settings| settings.dimmed = true);
        renderer.render_frames(3);
        let half = renderer.read_pixels().expect("half opacity");
        let color = pixel(&half, 93, 100);
        assert_eq!(color[0], 255);
        assert!(
            (i16::from(color[1]) - 188).abs() <= 1 && (i16::from(color[2]) - 188).abs() <= 1,
            "image opacity must be applied exactly once: {color:?}"
        );
        let raster = RASTER.get().expect("raster state");
        let original = raster.get();
        settings.with_mut(|settings| settings.dimmed = false);
        raster.set(ImageData::from_rgba(Arc::clone(&original.data), 8, 16));
        renderer.render_frames(3);
        let reshaped = renderer.read_pixels().expect("reshaped raster pixels");
        assert_eq!(
            pixel(&reshaped, 93, 100),
            pixel(&reshaped, 113, 100),
            "reshaping shared pixels must use a texture with the new row dimensions"
        );
        raster.set(ImageData::from_rgba(vec![255, 0, 0, 128], 1, 1));
        renderer.render_frames(3);
        let translucent = renderer.read_pixels().expect("translucent image");
        let color = pixel(&translucent, 93, 100);
        assert_eq!(color[0], 255);
        assert!(
            (i16::from(color[1]) - 187).abs() <= 1 && (i16::from(color[2]) - 187).abs() <= 1,
            "transparent images must blend in linear light: {color:?}"
        );
        raster.set(original);
        renderer.render_frames(3);
        let tree = renderer
            .accessibility_snapshot()
            .expect("accessibility tree");
        let gallery = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Gallery"))
            .expect("gallery");
        assert!(gallery.1.supports_action(Action::ScrollRight));
        let medium = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Item 1"))
            .unwrap_or_else(|| {
                panic!(
                    "medium item missing; published nodes: {:?}",
                    tree.nodes
                        .iter()
                        .map(|(_, node)| (node.role(), node.label()))
                        .collect::<Vec<_>>()
                )
            });
        assert_eq!(
            medium.1.bounds(),
            Some(tessera_ui::accesskit::Rect {
                x0: 204.0,
                y0: 20.0,
                x1: 326.0,
                y1: 200.0
            })
        );
        click(&mut renderer, [180.0, 100.0]);
        assert_eq!(
            activated.get(),
            [0],
            "overlapping unmasked bounds must not steal the preceding item's click"
        );
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(490.0, 290.0));
        renderer.render_frames(100);
        let focused = renderer.accessibility_snapshot().expect("focused tree");
        assert!(
            focused
                .nodes
                .iter()
                .any(|(id, node)| *id == focused.focus && node.label() == Some("Item 0"))
        );
        assert!(
            renderer.read_pixels().expect("focused image pixels") != initial,
            "focus feedback must remain visible over opaque carousel images"
        );
        tessera_ui::FocusManager::current().clear_focus(true);
        renderer.render_frames(12);
        assert!(renderer.read_pixels().expect("restored pixels") == initial);

        let mut node = AccessibilityNode::new();
        let mut handler = None;
        CarouselSemantics {
            controller,
            enabled: true,
            reverse: false,
            vertical: false,
        }
        .apply(&mut node, &mut handler);
        handler.expect("scroll action")(Action::ScrollRight);
        renderer.render_frames(120);
        assert_eq!(controller.with(CarouselController::current_item), 1);
        assert!(!controller.with(CarouselController::is_scrolling));
        controller.with_mut(|state| state.jump_to_item(0));
        renderer.render_frames(3);
        let point = PxPosition::from_f32_xy(150.0, 100.0);
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.step_frame();
        for x in [150.0, 120.0, 80.0, 40.0] {
            renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(x, 100.0));
            renderer.step_frame();
        }
        renderer.pointer_release(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(40.0, 100.0),
            PressKeyEventType::Left,
        );
        renderer.render_frames(160);
        assert_eq!(
            activated.get(),
            [0],
            "dragging media must cancel the original tap"
        );
        assert!(controller.with(CarouselController::current_item) > 0);
        assert!(!controller.with(CarouselController::is_scrolling));
        let focused = renderer.accessibility_snapshot().expect("drag focus");
        let (gallery, _) = focused
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Gallery"))
            .expect("gallery focus target");
        assert_eq!(
            focused.focus, *gallery,
            "dragging focuses viewport keyboard navigation"
        );

        settings.with_mut(|settings| settings.variant = CarouselVariant::Uncontained);
        controller.with_mut(|state| state.jump_to_item(2));
        renderer.render_frames(5);
        let before = controller.with(|state| state.motion.value());
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(200.0, 100.0),
            -37.0,
            0.0,
        );
        renderer.render_frames(100);
        let free = controller.with(|state| state.motion.value());
        assert!((free - before).abs() > 1.0);
        assert!(!controller.with(CarouselController::is_scrolling));
        let nearest = controller.with(|state| {
            state
                .strategy
                .as_ref()
                .expect("strategy")
                .snap(state.nearest(free), 10)
        });
        assert!(
            (free - nearest).abs() > 1.0,
            "uncontained wheel scrolling must not snap"
        );
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(200.0, 100.0),
            0.0,
            -30.0,
        );
        renderer.render_frames(3);
        assert_eq!(
            controller.with(|state| state.motion.value()),
            free,
            "vertical wheel movement belongs to the parent scroll container"
        );

        for variant in [
            CarouselVariant::UncontainedMultiAspectRatio,
            CarouselVariant::Hero,
            CarouselVariant::CenteredHero,
            CarouselVariant::FullScreen,
            CarouselVariant::MultiBrowse,
        ] {
            settings.with_mut(|settings| settings.variant = variant);
            controller.with_mut(|state| state.jump_to_item(9));
            renderer.render_frames(8);
            let pixels = renderer.read_pixels().expect("variant pixels");
            assert!(pixels != initial);
            assert_eq!(controller.with(CarouselController::current_item), 9);
            if variant == CarouselVariant::UncontainedMultiAspectRatio {
                controller.with_mut(|state| state.jump_to_item(0));
                renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(490.0, 290.0));
                renderer.render_frames(120);
                assert_eq!(controller.with(|state| state.masks[0]), (0, 176.0, 320.0));
                assert_eq!(controller.with(|state| state.masks[1].2), 101.25);
                let first = CarouselItemInfo {
                    index: 0,
                    controller: Some(controller),
                };
                let second = CarouselItemInfo {
                    index: 1,
                    controller: Some(controller),
                };
                assert_eq!(first.masked_width(), Dp(320.0));
                assert_eq!(second.full_width(), Dp(101.25));
                assert_eq!(second.masked_width(), Dp(36.0));
                controller.with_mut(|state| state.move_by(93.0));
                renderer.render_frames(8);
                assert_eq!(first.full_width(), Dp(320.0));
                assert_eq!(first.masked_width(), Dp(243.0));
                controller.with_mut(|state| state.jump_to_item(0));
                renderer.render_frames(8);
                let before = renderer.read_pixels().expect("mixed aspect pixels");
                assert_eq!(pixel(&before, 240, 110), [0, 0, 255, 255]);
                settings.with_mut(|settings| {
                    settings.ratios = [1.0, 16.0 / 9.0, 9.0 / 16.0, 3.0 / 4.0]
                });
                renderer.render_frames(8);
                assert_eq!(controller.with(|state| state.masks[0].2), 180.0);
                assert_eq!(controller.with(|state| state.masks[1].2), 320.0);
                let changed = renderer.read_pixels().expect("changed aspect pixels");
                assert_eq!(pixel(&changed, 240, 110), [255, 0, 0, 255]);
                settings.with_mut(|settings| settings.height = 120.0);
                renderer.render_frames(8);
                assert_eq!(controller.with(|state| state.masks[0].2), 120.0);
                let tree = renderer
                    .accessibility_snapshot()
                    .expect("mixed aspect accessibility");
                let bounds = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some("Item 0"))
                    .expect("first item")
                    .1
                    .bounds()
                    .expect("first item bounds");
                assert_eq!(bounds.width(), 120.0);
                assert_eq!(bounds.height(), 120.0);
                let settled = renderer.read_pixels().expect("settled mixed aspect pixels");
                renderer.render_frames(20);
                assert_eq!(controller.with(|state| state.masks[0].2), 120.0);
                assert_eq!(
                    renderer.read_pixels().expect("stable mixed aspect pixels"),
                    settled
                );
                settings.with_mut(|settings| {
                    settings.height = 180.0;
                    settings.ratios = [16.0 / 9.0, 9.0 / 16.0, 1.0, 3.0 / 4.0];
                });
            }
            if variant == CarouselVariant::FullScreen {
                assert_eq!(
                    controller.with(|state| state.masks.clone()),
                    vec![(9, 90.0, 180.0)]
                );
                controller.with_mut(|state| state.jump_to_item(3));
                renderer.render_frames(3);
                let mut node = AccessibilityNode::new();
                let mut handler = None;
                CarouselSemantics {
                    controller,
                    enabled: true,
                    reverse: false,
                    vertical: true,
                }
                .apply(&mut node, &mut handler);
                assert!(node.actions.contains(&Action::ScrollUp));
                assert!(node.actions.contains(&Action::ScrollDown));
                assert!(!node.actions.contains(&Action::ScrollRight));
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(200.0, 100.0),
                    0.0,
                    -140.0,
                );
                renderer.render_frames(120);
                assert_eq!(controller.with(CarouselController::current_item), 4);
                let offset = controller.with(|state| state.motion.value());
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(200.0, 100.0),
                    -140.0,
                    0.0,
                );
                renderer.render_frames(3);
                assert_eq!(controller.with(|state| state.motion.value()), offset);
                handler.expect("vertical scroll action")(Action::ScrollUp);
                renderer.render_frames(120);
                assert_eq!(controller.with(CarouselController::current_item), 3);
                settings.with_mut(|settings| settings.reverse = true);
                renderer.render_frames(3);
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(200.0, 100.0),
                    0.0,
                    140.0,
                );
                renderer.render_frames(120);
                assert_eq!(controller.with(CarouselController::current_item), 4);
                settings.with_mut(|settings| settings.reverse = false);
            }
        }
        settings.with_mut(|settings| {
            settings.count = 2;
            settings.reverse = true;
        });
        renderer.render_frames(8);
        assert_eq!(controller.with(CarouselController::current_item), 1);
        settings.with_mut(|settings| settings.enabled = false);
        renderer.render_frames(3);
        let before = controller.with(|state| state.motion.value());
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(200.0, 100.0),
            -50.0,
            0.0,
        );
        renderer.render_frames(8);
        assert_eq!(controller.with(|state| state.motion.value()), before);
        let tree = renderer.accessibility_snapshot().expect("disabled tree");
        let gallery = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Gallery"))
            .expect("gallery");
        assert!(!gallery.1.supports_action(Action::ScrollLeft));
        assert!(!gallery.1.supports_action(Action::ScrollRight));
        settings.with_mut(|settings| settings.count = 0);
        renderer.render_frames(8);
        assert!(controller.with(|state| state.masks.is_empty()));
        let empty = renderer.read_pixels().expect("empty pixels");
        for y in 0..300 {
            for x in 0..500 {
                let expected = if (10..390).contains(&x) && (20..200).contains(&y) {
                    &[255, 255, 255, 255]
                } else {
                    pixel(&initial, x, y)
                };
                assert_eq!(pixel(&empty, x, y), expected, "empty carousel at {x}, {y}");
            }
        }
        settings.with_mut(|settings| {
            settings.count = 10;
            settings.enabled = true;
            settings.reverse = false;
            settings.variant = CarouselVariant::MultiBrowse;
        });
        renderer.render_frames(8);
        assert!(renderer.read_pixels().expect("regrown pixels") == initial);
        renderer.resize(640, 360);
        renderer.render_frames(8);
        let resized = renderer.read_pixels().expect("resized pixels");
        for center in [103usize, 265, 362] {
            let offset = (100 * 640 + center - 10) * 4;
            assert_eq!(&resized[offset..offset + 4], [255, 0, 0, 255]);
        }
        renderer.resize(500, 300);
        renderer.render_frames(8);
        assert!(renderer.read_pixels().expect("restored viewport pixels") == initial);
        controller.with_mut(|state| state.jump_to_item(3));
        renderer.render_frames(8);
        tessera_ui::dp::set_scale_factor(1.5);
        renderer.resize(750, 450);
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(740.0, 440.0));
        renderer.render_frames(8);
        assert_eq!(renderer.snapshot()["scale_factor"], 1.5);
        assert!(CONTROLLER.get() == Some(controller));
        assert_eq!(controller.with(CarouselController::current_item), 3);
        let scaled = renderer.read_pixels().expect("density change pixels");
        for (_, center, width) in controller.with(|state| state.masks.clone()) {
            if width < 60.0 || center < 15.0 || center > 720.0 {
                continue;
            }
            let x = center.round() as usize;
            let offset = (150 * 750 + x) * 4;
            assert_eq!(&scaled[offset..offset + 4], [255, 0, 0, 255]);
        }
        tessera_ui::dp::set_scale_factor(1.0);
        renderer.resize(390, 780);
        renderer.render_frames(120);
        assert_eq!(controller.with(CarouselController::current_item), 3);
        let tree = renderer
            .accessibility_snapshot()
            .expect("portrait accessibility");
        let following = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Following carousel"))
            .expect("following item must remain visible");
        let bounds = following.1.bounds().expect("following bounds");
        assert!(bounds.y0 >= 200.0 && bounds.y1 < 780.0);
        let portrait = renderer.read_pixels().expect("portrait pixels");
        assert!((205..235).any(|y| {
            (5..180).any(|x| portrait[(y * 390 + x) * 4..(y * 390 + x) * 4 + 3] != [255, 255, 255])
        }));
        renderer.resize(500, 300);
        renderer.render_frames(8);
        assert_eq!(controller.with(CarouselController::current_item), 3);
        controller.with_mut(|state| state.jump_to_item(0));
        renderer.render_frames(8);
        assert!(renderer.read_pixels().expect("restored density pixels") == initial);
        let uploads = crate::pipelines::image::pipeline::TEXTURE_UPLOADS
            .load(std::sync::atomic::Ordering::Relaxed);
        let original = raster.get();
        raster.set(ImageData::from_rgba(
            original.data.as_ref().clone(),
            original.width,
            original.height,
        ));
        settings.with_mut(|settings| settings.stress = true);
        renderer.render_frames(3);
        assert_eq!(
            crate::pipelines::image::pipeline::TEXTURE_UPLOADS
                .load(std::sync::atomic::Ordering::Relaxed),
            uploads,
            "independently decoded identical pixels reuse the existing texture"
        );
        let repeated = renderer.read_pixels().expect("repeated image instances");
        for index in 0..40usize {
            let x = 10 + (index % 8) * 28;
            let y = 20 + (index / 8) * 28;
            assert_eq!(
                pixel(&repeated, x + 9, y + 12),
                [255, 0, 0, 255],
                "instance {index} after arena growth"
            );
            assert_eq!(pixel(&repeated, x + 18, y + 12), [0, 0, 255, 255]);
        }
        let allocations = crate::pipelines::image::pipeline::INSTANCE_ALLOCATIONS
            .load(std::sync::atomic::Ordering::Relaxed);
        let uploads = crate::pipelines::image::pipeline::TEXTURE_UPLOADS
            .load(std::sync::atomic::Ordering::Relaxed);
        let comparisons = crate::pipelines::image::command::PIXEL_BUFFER_COMPARISONS.get();
        renderer.render_frames(5);
        assert_eq!(
            crate::pipelines::image::command::PIXEL_BUFFER_COMPARISONS.get(),
            comparisons,
            "rendering shared raster instances must not scan their pixel buffers"
        );
        assert_eq!(
            crate::pipelines::image::pipeline::INSTANCE_ALLOCATIONS
                .load(std::sync::atomic::Ordering::Relaxed),
            allocations
        );
        assert_eq!(
            crate::pipelines::image::pipeline::TEXTURE_UPLOADS
                .load(std::sync::atomic::Ordering::Relaxed),
            uploads
        );
        settings.with_mut(|settings| {
            settings.stress = false;
            settings.count = 0;
        });
        renderer.render_frames(130);
        settings.with_mut(|settings| settings.stress = true);
        renderer.render_frames(3);
        assert_eq!(
            crate::pipelines::image::pipeline::TEXTURE_UPLOADS
                .load(std::sync::atomic::Ordering::Relaxed),
            uploads,
            "unused textures stay cached while they fit the byte budget"
        );
        assert!(renderer.read_pixels().expect("restored cached image") == repeated);
        let duplicate = raster.get();
        for shade in 1..=5_u8 {
            let mut pixels = vec![0; 2048 * 2048 * 4];
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[shade * 31, 0, 0, 255]);
            }
            raster.set(ImageData::from_rgba(pixels, 2048, 2048));
            renderer.render_frames(2);
        }
        let pressure_uploads = crate::pipelines::image::pipeline::TEXTURE_UPLOADS
            .load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(pressure_uploads, uploads + 5);
        raster.set(duplicate);
        renderer.render_frames(3);
        assert_eq!(
            crate::pipelines::image::pipeline::TEXTURE_UPLOADS
                .load(std::sync::atomic::Ordering::Relaxed),
            pressure_uploads + 1,
            "exceeding the byte budget evicts the oldest unused texture"
        );
        assert!(renderer.read_pixels().expect("image after cache eviction") == repeated);
        if let Some(directory) = std::env::var_os("TESSERA_CAROUSEL_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).expect("carousel QA directory");
            tessera_ui::headless::write_png_file(
                directory.join(format!("carousel-content-msaa{samples}.png")),
                500,
                300,
                &initial,
            )
            .expect("carousel QA image");
        }
    }

    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn native_carousel_masks_input_and_dynamic_props_at_one_sample() {
        check_native_carousel(1);
    }

    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn native_carousel_masks_input_and_dynamic_props_at_four_samples() {
        check_native_carousel(4);
    }

    #[tessera]
    fn responsive_section(controller: State<CarouselController>) {
        column().modifier(Modifier::new().semantics(SemanticsArgs {
            label: Some("Responsive carousel section".into()), merge_descendants: true, ..Default::default()
        })).children(move || {
            text().content("Carousel");
            text().content("Items retain their full size as masks expand and collapse. Drag horizontally or use arrow keys, Home, and End.");
            flow_row().item_spacing(Dp(8.0)).line_spacing(Dp(8.0)).children(move || {
                for label in ["Layout: Multi-browse", "Previous item", "Next item", "Use reverse layout", "Disable scrolling"] {
                    button().tonal().child(move || { text().content(label); });
                }
            });
            spacer().modifier(Modifier::new().height(Dp(12.0)));
            carousel(12).controller(controller).height(Dp(180.0)).preferred_item_width(Dp(186.0)).item_spacing(Dp(8.0)).content(|item| {
                button().modifier(Modifier::new().fill_max_size()).child(move || { text().content(format!("{} {}", if item.masked_width() >= Dp(120.0) { "Collection" } else { "Preview" }, item.index + 1)); });
            });
            text().content(format!("Current item: {}", controller.with(CarouselController::current_item)));
            text().content("No item activated");
        });
    }

    #[tessera]
    fn responsive_feed() {
        let controller = remember(CarouselController::default);
        CONTROLLER.set(Some(controller));
        material_theme().child(move || {
            let theme = tessera_ui::use_context::<MaterialTheme>().expect("material theme");
            scaffold()
                .top_bar(|| {
                    spacer().modifier(Modifier::new().fill_max_width().height(Dp(64.0)));
                })
                .content(move || {
                    lazy_column()
                        .modifier(Modifier::new().fill_max_size())
                        .estimated_item_size(Dp(280.0))
                        .content_padding(Dp(16.0))
                        .item_spacing(Dp(24.0))
                        .item(move || {
                            column().children(move || {
                                text()
                                    .content("Material Components")
                                    .style(theme.get().typography.headline_large);
                                spacer().modifier(Modifier::new().height(Dp(8.0)));
                                text().content(
                                    "Additional Material 3 components beyond the basic set.",
                                );
                            });
                        })
                        .item(move || {
                            responsive_section(controller);
                        })
                        .item(|| {
                            text().content("Following responsive section");
                        });
                });
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter"]
    fn native_responsive_feed_keeps_following_items_visible_after_density_changes() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                responsive_feed();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 1316,
                height: 1307,
                scale_factor: 1.5,
                sample_count: 4,
                ..Default::default()
            },
        )
        .expect("responsive renderer");
        renderer.render_frames(20);
        let controller = CONTROLLER.get().expect("responsive controller");
        controller.with_mut(|state| state.animate_to_item(4));
        renderer.render_frames(120);
        tessera_ui::dp::set_scale_factor(1.0);
        renderer.resize(1, 780);
        renderer.render_frames(2);
        renderer.resize(390, 780);
        for frame in 0..120 {
            renderer.step_frame();
            assert_eq!(
                controller.with(CarouselController::current_item),
                4,
                "portrait frame {frame}"
            );
        }
        let tree = renderer.accessibility_snapshot().expect("responsive tree");
        let masks = controller.with(|state| state.masks.clone());
        let snapshot = renderer.snapshot().to_string();
        assert!(
            snapshot.contains("\"label\":\"Preview 6\""),
            "the next item's nested content must observe its smaller mask; masks: {masks:?}"
        );
        assert!(
            !snapshot.contains("\"label\":\"Collection 6\""),
            "a previously large item's label must update after resize"
        );
        let section = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Responsive carousel section"))
            .expect("carousel section")
            .1
            .bounds()
            .expect("section bounds");
        let following = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Following responsive section"))
            .expect("following section")
            .1
            .bounds()
            .expect("following bounds");
        assert!(section.y1 < 740.0, "carousel section: {section:?}");
        assert!(
            following.y0 >= section.y1 + 24.0 && following.y1 <= 780.0,
            "section: {section:?}; following: {following:?}"
        );
        let pixels = renderer.read_pixels().expect("responsive pixels");
        assert!(
            (following.y0.floor() as usize..following.y1.ceil() as usize).any(|y| {
                (following.x0.floor() as usize..following.x1.ceil().min(390.0) as usize).any(|x| {
                    pixels[(y * 390 + x) * 4..(y * 390 + x) * 4 + 3]
                        .iter()
                        .all(|value| *value < 180)
                })
            }),
            "following text must remain rendered after zero-width measurement"
        );
        tessera_ui::dp::set_scale_factor(1.5);
        renderer.resize(1316, 1307);
        renderer.render_frames(120);
        assert_eq!(controller.with(CarouselController::current_item), 4);
        tessera_ui::dp::set_scale_factor(1.0);
    }
}
