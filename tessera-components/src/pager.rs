//! Pager container for swipeable pages.
//!
//! ## Usage
//!
//! Show onboarding steps or media carousels that snap between pages.
use tessera_foundation::gesture::{
    DragAxis, DragRecognizer, DragSettings, ScrollRecognizer, ScrollSettings,
};
use tessera_ui::{
    AxisConstraint, CallbackWith, ComputedData, Constraint, Dp, FocusProperties, KeyboardInput,
    KeyboardInputModifierNode, LayoutResult, MeasurementError, Modifier, PointerInput,
    PointerInputModifierNode, Px, PxPosition, State, key,
    layout::{LayoutPolicy, MeasureScope, PlacementScope, RenderInput, RenderPolicy, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    receive_frame_nanos, remember, tessera, winit,
};

use crate::{
    alignment::CrossAxisAlignment, modifier::ModifierExt as _, pos_misc::is_position_inside_bounds,
};

const DEFAULT_SNAP_THRESHOLD: f32 = 0.5;
const DEFAULT_SCROLL_SMOOTHING: f32 = 0.12;
const SNAP_IDLE_TIME_NANOS: u64 = 120_000_000;

/// Describes how a pager page is sized along the scroll axis.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PagerPageSize {
    /// Page size fills the available main-axis space.
    #[default]
    Fill,
    /// Page size is a fixed density-independent value.
    Fixed(Dp),
}

/// Configuration arguments shared by pager variants.
#[derive(Clone)]
struct PagerConfig {
    /// Modifier chain applied to the pager subtree.
    pub modifier: Modifier,
    /// Total number of pages available in the pager.
    pub page_count: usize,
    /// Initial page index when the pager is first created.
    pub initial_page: usize,
    /// Size of each page along the scroll axis.
    pub page_size: PagerPageSize,
    /// Spacing between pages.
    pub page_spacing: Dp,
    /// Symmetric padding applied before the first and after the last page.
    pub content_padding: Dp,
    /// Number of extra pages kept alive on either side of the visible pages.
    pub beyond_viewport_page_count: usize,
    /// Alignment for pages along the cross axis.
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Whether user scrolling is enabled.
    pub user_scroll_enabled: bool,
    /// Fraction of a page that must be crossed to snap to the next page.
    pub snap_threshold: f32,
    /// Smoothing factor for snapping animations.
    pub scroll_smoothing: f32,
    /// Optional page-rendering callback.
    pub page_content: CallbackWith<usize>,
    /// Optional external pager controller.
    ///
    /// When this is `None`, the pager creates and owns an internal controller.
    pub controller: Option<State<PagerController>>,
}

impl Default for PagerConfig {
    fn default() -> Self {
        Self {
            modifier: Modifier::new().fill_max_size(),
            page_count: 0,
            initial_page: 0,
            page_size: PagerPageSize::default(),
            page_spacing: Dp(0.0),
            content_padding: Dp(0.0),
            beyond_viewport_page_count: 0,
            cross_axis_alignment: CrossAxisAlignment::Center,
            user_scroll_enabled: true,
            snap_threshold: DEFAULT_SNAP_THRESHOLD,
            scroll_smoothing: DEFAULT_SCROLL_SMOOTHING,
            page_content: CallbackWith::default_value(),
            controller: None,
        }
    }
}

struct PagerParams {
    modifier: Option<Modifier>,
    page_count: usize,
    initial_page: usize,
    page_size: PagerPageSize,
    page_spacing: Dp,
    content_padding: Dp,
    beyond_viewport_page_count: usize,
    cross_axis_alignment: CrossAxisAlignment,
    user_scroll_enabled: bool,
    snap_threshold: Option<f32>,
    scroll_smoothing: Option<f32>,
    page_content: Option<CallbackWith<usize>>,
    controller: Option<State<PagerController>>,
}

fn pager_config_from_params(params: PagerParams) -> PagerConfig {
    let defaults = PagerConfig::default();
    PagerConfig {
        modifier: params.modifier.unwrap_or(defaults.modifier),
        page_count: params.page_count,
        initial_page: params.initial_page,
        page_size: params.page_size,
        page_spacing: params.page_spacing,
        content_padding: params.content_padding,
        beyond_viewport_page_count: params.beyond_viewport_page_count,
        cross_axis_alignment: params.cross_axis_alignment,
        user_scroll_enabled: params.user_scroll_enabled,
        snap_threshold: params.snap_threshold.unwrap_or(defaults.snap_threshold),
        scroll_smoothing: params.scroll_smoothing.unwrap_or(defaults.scroll_smoothing),
        page_content: params
            .page_content
            .unwrap_or_else(CallbackWith::default_value),
        controller: params.controller,
    }
}

/// Controller for pager components.
#[derive(Clone, PartialEq)]
pub struct PagerController {
    current_page: usize,
    current_page_offset_fraction: f32,
    page_count: usize,
    page_size: Px,
    page_spacing: Px,
    scroll_offset: f32,
    target_offset: f32,
    last_frame_nanos: Option<u64>,
    last_scroll_frame_nanos: Option<u64>,
    is_dragging: bool,
    initialized: bool,
}

impl PagerController {
    /// Creates a new controller with the requested initial page.
    pub fn new(initial_page: usize) -> Self {
        Self {
            current_page: initial_page,
            current_page_offset_fraction: 0.0,
            page_count: 0,
            page_size: Px::ZERO,
            page_spacing: Px::ZERO,
            scroll_offset: 0.0,
            target_offset: 0.0,
            last_frame_nanos: None,
            last_scroll_frame_nanos: None,
            is_dragging: false,
            initialized: false,
        }
    }

    /// Returns the currently selected page.
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Returns the current page offset fraction in the range -0.5..0.5.
    pub fn current_page_offset_fraction(&self) -> f32 {
        self.current_page_offset_fraction
    }

    /// Jumps immediately to the requested page.
    pub fn jump_to_page(&mut self, page: usize) {
        let page = self.clamp_page(page);
        self.current_page = page;
        let offset = self.offset_for_page(page);
        self.scroll_offset = offset;
        self.target_offset = offset;
        self.last_scroll_frame_nanos = None;
        self.update_current_page_from_offset();
    }

    /// Scrolls toward the requested page using snap smoothing.
    pub fn scroll_to_page(&mut self, page: usize) {
        let page = self.clamp_page(page);
        self.target_offset = self.offset_for_page(page);
        self.last_scroll_frame_nanos = None;
    }

    fn set_page_count(&mut self, page_count: usize) {
        self.page_count = page_count;
        if page_count == 0 {
            self.current_page = 0;
            self.scroll_offset = 0.0;
            self.target_offset = 0.0;
            self.current_page_offset_fraction = 0.0;
            return;
        }
        if self.current_page >= page_count {
            self.current_page = page_count - 1;
        }
    }

    fn update_layout(&mut self, page_size: Px, page_spacing: Px, page_count: usize) {
        let size_changed = page_size != self.page_size || page_spacing != self.page_spacing;
        self.page_size = page_size;
        self.page_spacing = page_spacing;
        self.page_count = page_count;
        self.current_page = self.clamp_page(self.current_page);

        if page_count == 0 {
            self.scroll_offset = 0.0;
            self.target_offset = 0.0;
            self.current_page_offset_fraction = 0.0;
            return;
        }

        if !self.initialized && page_size > Px::ZERO {
            let offset = self.offset_for_page(self.current_page);
            self.scroll_offset = offset;
            self.target_offset = offset;
            self.initialized = true;
        } else if size_changed && page_size > Px::ZERO {
            let offset = self.offset_for_page(self.current_page);
            self.scroll_offset = offset;
            self.target_offset = offset;
        }

        self.scroll_offset = self.clamp_offset(self.scroll_offset);
        self.target_offset = self.clamp_offset(self.target_offset);
        self.update_current_page_from_offset();
    }

    fn tick(&mut self, frame_nanos: u64, snap_threshold: f32, scroll_smoothing: f32) {
        if self.page_count == 0 {
            return;
        }
        if self.page_distance() <= f32::EPSILON {
            return;
        }

        let snap_threshold = snap_threshold.clamp(0.0, 1.0);
        let scroll_smoothing = scroll_smoothing.clamp(0.0, 1.0);
        let idle = self
            .last_scroll_frame_nanos
            .map(|last_scroll_frame_nanos| {
                frame_nanos.saturating_sub(last_scroll_frame_nanos) > SNAP_IDLE_TIME_NANOS
            })
            .unwrap_or(true);

        if idle && !self.is_dragging {
            let target_page = self.snap_target_page(snap_threshold);
            self.target_offset = self.offset_for_page(target_page);
        }

        self.update_scroll_offset(frame_nanos, scroll_smoothing);
        self.scroll_offset = self.clamp_offset(self.scroll_offset);
        self.update_current_page_from_offset();
    }

    fn has_pending_animation_frame(&self, frame_nanos: u64) -> bool {
        if self.page_count == 0 || self.page_distance() <= f32::EPSILON {
            return false;
        }

        if self.is_dragging {
            return true;
        }

        if (self.target_offset - self.scroll_offset).abs() > f32::EPSILON {
            return true;
        }

        self.last_scroll_frame_nanos
            .map(|last_scroll_frame_nanos| {
                frame_nanos.saturating_sub(last_scroll_frame_nanos) <= SNAP_IDLE_TIME_NANOS
            })
            .unwrap_or(false)
    }

    fn apply_scroll_delta(&mut self, delta: f32, frame_nanos: u64) {
        if self.page_distance() <= f32::EPSILON || self.page_count == 0 {
            return;
        }
        self.scroll_offset = self.clamp_offset(self.scroll_offset + delta);
        self.target_offset = self.scroll_offset;
        self.last_scroll_frame_nanos = Some(frame_nanos);
        self.update_current_page_from_offset();
    }

    fn start_drag(&mut self, frame_nanos: u64) {
        self.is_dragging = true;
        self.last_scroll_frame_nanos = Some(frame_nanos);
    }

    fn end_drag(&mut self) {
        self.is_dragging = false;
    }

    fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    fn scroll_offset_px(&self) -> Px {
        Px::saturating_from_f32(self.scroll_offset)
    }

    fn clamp_page(&self, page: usize) -> usize {
        if self.page_count == 0 {
            0
        } else {
            page.min(self.page_count.saturating_sub(1))
        }
    }

    fn page_distance(&self) -> f32 {
        (self.page_size + self.page_spacing).to_f32()
    }

    fn offset_for_page(&self, page: usize) -> f32 {
        -self.page_distance() * page as f32
    }

    fn clamp_offset(&self, offset: f32) -> f32 {
        if self.page_count <= 1 || self.page_distance() <= f32::EPSILON {
            return 0.0;
        }
        let max_offset = 0.0;
        let min_offset = -self.page_distance() * self.page_count.saturating_sub(1) as f32;
        offset.clamp(min_offset, max_offset)
    }

    fn snap_target_page(&self, threshold: f32) -> usize {
        let distance = self.page_distance();
        if distance <= f32::EPSILON || self.page_count == 0 {
            return 0;
        }
        let page_float = -self.scroll_offset / distance;
        let base_page = page_float.floor();
        let fraction = page_float - base_page;
        let target = if fraction >= threshold {
            base_page + 1.0
        } else {
            base_page
        };
        let max_page = self.page_count.saturating_sub(1) as f32;
        if target.is_finite() {
            target.clamp(0.0, max_page) as usize
        } else {
            0
        }
    }

    fn update_current_page_from_offset(&mut self) {
        let distance = self.page_distance();
        if distance <= f32::EPSILON || self.page_count == 0 {
            self.current_page = 0;
            self.current_page_offset_fraction = 0.0;
            return;
        }

        let page_float = -self.scroll_offset / distance;
        let mut nearest = page_float.round();
        if !nearest.is_finite() {
            nearest = 0.0;
        }
        let max_page = self.page_count.saturating_sub(1) as f32;
        let current_page = nearest.clamp(0.0, max_page) as usize;
        let snapped_offset = -distance * current_page as f32;
        let offset_fraction = ((self.scroll_offset - snapped_offset) / distance).clamp(-0.5, 0.5);
        self.current_page = current_page;
        self.current_page_offset_fraction = offset_fraction;
    }

    fn update_scroll_offset(&mut self, frame_nanos: u64, smoothing: f32) {
        let delta_time = if let Some(last_frame_nanos) = self.last_frame_nanos {
            frame_nanos.saturating_sub(last_frame_nanos) as f32 / 1_000_000_000.0
        } else {
            1.0 / 60.0
        };
        self.last_frame_nanos = Some(frame_nanos);

        let diff = self.target_offset - self.scroll_offset;
        if diff.abs() < 0.5 {
            self.scroll_offset = self.target_offset;
            return;
        }

        let mut movement_factor = (1.0 - smoothing) * delta_time * 60.0;
        if movement_factor > 1.0 {
            movement_factor = 1.0;
        }

        self.scroll_offset += diff * movement_factor;
    }
}

impl Default for PagerController {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PagerAxis {
    Horizontal,
    Vertical,
}

impl PagerAxis {
    fn cross(self, size: ComputedData) -> Px {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    fn pack_size(self, main: Px, cross: Px) -> ComputedData {
        match self {
            Self::Horizontal => ComputedData {
                width: main,
                height: cross,
            },
            Self::Vertical => ComputedData {
                width: cross,
                height: main,
            },
        }
    }

    fn position(self, main: Px, cross: Px) -> PxPosition {
        match self {
            Self::Horizontal => PxPosition::new(main, cross),
            Self::Vertical => PxPosition::new(cross, main),
        }
    }

    fn scroll_delta(self, delta_x: f32, delta_y: f32) -> f32 {
        match self {
            Self::Horizontal => {
                if delta_x.abs() >= 0.01 {
                    delta_x
                } else {
                    delta_y
                }
            }
            Self::Vertical => {
                if delta_y.abs() >= 0.01 {
                    delta_y
                } else {
                    delta_x
                }
            }
        }
    }
}

#[derive(Clone)]
struct PagerLayout {
    axis: PagerAxis,
    cross_axis_alignment: CrossAxisAlignment,
    page_size: PagerPageSize,
    page_spacing: Px,
    content_padding: Px,
    page_count: usize,
    visible_pages: Vec<usize>,
    scroll_offset: Px,
    controller: State<PagerController>,
}

impl PartialEq for PagerLayout {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.cross_axis_alignment == other.cross_axis_alignment
            && self.page_size == other.page_size
            && self.page_spacing == other.page_spacing
            && self.content_padding == other.content_padding
            && self.page_count == other.page_count
            && self.visible_pages == other.visible_pages
            && self.scroll_offset == other.scroll_offset
    }
}

impl LayoutPolicy for PagerLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if self.page_count == 0 {
            return Ok(result.with_size(ComputedData::min_from_constraint(
                input.parent_constraint().as_ref(),
            )));
        }

        if children.len() != self.visible_pages.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Pager measured child count mismatch".into(),
            ));
        }

        let parent = input.parent_constraint();
        let main_dimension = match self.axis {
            PagerAxis::Horizontal => parent.width(),
            PagerAxis::Vertical => parent.height(),
        };
        let cross_dimension = match self.axis {
            PagerAxis::Horizontal => parent.height(),
            PagerAxis::Vertical => parent.width(),
        };

        let page_main =
            resolve_page_main_size(self.page_size, main_dimension, self.content_padding);
        let page_spacing = self.page_spacing;
        let padding = self.content_padding;
        let container_main = main_dimension.clamp(page_main + padding + padding);

        let cross_constraint =
            cross_dimension_for_alignment(cross_dimension, self.cross_axis_alignment);
        let child_constraint = match self.axis {
            PagerAxis::Horizontal => {
                Constraint::new(AxisConstraint::exact(page_main), cross_constraint)
            }
            PagerAxis::Vertical => {
                Constraint::new(cross_constraint, AxisConstraint::exact(page_main))
            }
        };

        let children_to_measure: Vec<_> = children
            .iter()
            .copied()
            .map(|child| (child, child_constraint))
            .collect();
        let measurements = input.measure_children(children_to_measure)?;

        let mut max_cross = Px::ZERO;
        for size in measurements.values() {
            max_cross = max_cross.max(self.axis.cross(size.size()));
        }
        let container_cross = cross_dimension.clamp(max_cross);

        let should_update_layout = self.controller.with(|controller| {
            let mut next = controller.clone();
            next.update_layout(page_main, page_spacing, self.page_count);
            next != *controller
        });
        if should_update_layout {
            self.controller.with_mut(|controller| {
                controller.update_layout(page_main, page_spacing, self.page_count);
            });
        }

        let scroll_offset = self.controller.with(|c| c.scroll_offset_px());
        let page_step = page_main + page_spacing;

        for (&child, &page_index) in children.iter().zip(self.visible_pages.iter()) {
            let measured = measurements
                .get(&child)
                .copied()
                .map(|size| size.size())
                .unwrap_or(ComputedData::ZERO);
            let cross_offset = compute_cross_offset(
                container_cross,
                self.axis.cross(measured),
                self.cross_axis_alignment,
            );
            let page_offset = padding + px_mul(page_step, page_index) + scroll_offset;
            let position = self.axis.position(page_offset, cross_offset);
            result.place_child(child, position);
        }

        Ok(result.with_size(self.axis.pack_size(container_main, container_cross)))
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.cross_axis_alignment == other.cross_axis_alignment
            && self.page_size == other.page_size
            && self.page_spacing == other.page_spacing
            && self.content_padding == other.content_padding
            && self.page_count == other.page_count
            && self.visible_pages == other.visible_pages
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.cross_axis_alignment == other.cross_axis_alignment
            && self.page_size == other.page_size
            && self.page_spacing == other.page_spacing
            && self.content_padding == other.content_padding
            && self.page_count == other.page_count
            && self.visible_pages == other.visible_pages
            && self.scroll_offset == other.scroll_offset
    }

    fn place_children(&self, input: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>> {
        let mut result = LayoutResult::default();
        if self.page_count == 0 {
            return Some(result.into_placements());
        }

        let children = input.children();
        if children.len() != self.visible_pages.len() {
            return None;
        }

        let container_cross = self.axis.cross(input.size());
        let page_step = self
            .controller
            .with(|controller| controller.page_size + controller.page_spacing);
        let padding = self.content_padding;

        for (&child, &page_index) in children.iter().zip(self.visible_pages.iter()) {
            let measured = child.size();
            let cross_offset = compute_cross_offset(
                container_cross,
                self.axis.cross(measured),
                self.cross_axis_alignment,
            );
            let page_offset = padding + px_mul(page_step, page_index) + self.scroll_offset;
            let position = self.axis.position(page_offset, cross_offset);
            result.place_child(child, position);
        }

        Some(result.into_placements())
    }
}

impl RenderPolicy for PagerLayout {
    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().set_clips_children(true);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ZeroLayout;

impl LayoutPolicy for ZeroLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData::min_from_constraint(
            input.parent_constraint().as_ref(),
        )))
    }
}

fn compute_visible_pages(current_page: usize, page_count: usize, beyond: usize) -> Vec<usize> {
    if page_count == 0 {
        return Vec::new();
    }
    let extra = beyond.saturating_add(1);
    let start = current_page.saturating_sub(extra);
    let end = (current_page + extra + 1).min(page_count);
    (start..end).collect()
}

fn resolve_page_main_size(
    page_size: PagerPageSize,
    main_dimension: AxisConstraint,
    padding: Px,
) -> Px {
    match page_size {
        PagerPageSize::Fill => {
            let max = main_dimension
                .resolve_max()
                .expect("Pager page size Fill requires a bounded main-axis constraint");
            (max - padding - padding).max(Px::ZERO)
        }
        PagerPageSize::Fixed(dp) => dp.into(),
    }
}

fn cross_dimension_for_alignment(
    cross_dimension: AxisConstraint,
    alignment: CrossAxisAlignment,
) -> AxisConstraint {
    match alignment {
        CrossAxisAlignment::Stretch => match cross_dimension.resolve_max() {
            Some(max) => AxisConstraint::exact(max),
            None => cross_dimension,
        },
        _ => AxisConstraint::new(Px::ZERO, cross_dimension.resolve_max()),
    }
}

fn compute_cross_offset(container: Px, child: Px, alignment: CrossAxisAlignment) -> Px {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Px::ZERO,
        CrossAxisAlignment::Center => (container - child).max(Px::ZERO) / 2,
        CrossAxisAlignment::End => (container - child).max(Px::ZERO),
    }
}

fn sanitize_spacing(px: Px) -> Px {
    if px < Px::ZERO { Px::ZERO } else { px }
}

fn px_mul(px: Px, times: usize) -> Px {
    if times == 0 {
        return Px::ZERO;
    }
    px_from_i64(px.0 as i64 * times as i64)
}

fn px_from_i64(value: i64) -> Px {
    if value > i64::from(i32::MAX) {
        Px(i32::MAX)
    } else if value < i64::from(i32::MIN) {
        Px(i32::MIN)
    } else {
        Px(value as i32)
    }
}

struct PagerKeyboardModifierNode {
    controller: State<PagerController>,
    axis: PagerAxis,
    user_scroll_enabled: bool,
}

impl KeyboardInputModifierNode for PagerKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if !self.user_scroll_enabled
            || input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
        {
            return;
        }

        let mut handled = false;
        for event in input.keyboard_events.iter() {
            if event.state != winit::event::ElementState::Pressed {
                continue;
            }

            let Some(command) = pager_keyboard_command(self.axis, &event.logical_key) else {
                continue;
            };

            handled = run_pager_keyboard_command(self.controller, command);
            if handled {
                break;
            }
        }

        if handled {
            input.block_keyboard();
        }
    }
}

struct PagerPointerModifierNode {
    controller: State<PagerController>,
    axis: PagerAxis,
    user_scroll_enabled: bool,
    drag_recognizer: State<DragRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
}

impl PointerInputModifierNode for PagerPointerModifierNode {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if !self.user_scroll_enabled {
            return;
        }

        let is_cursor_in_component = input
            .cursor_position_rel
            .map(|pos| is_position_inside_bounds(input.computed_data, pos))
            .unwrap_or(false);
        let is_dragging = self.controller.with(|controller| controller.is_dragging());
        if !is_cursor_in_component && !is_dragging {
            return;
        }

        let frame_nanos = tessera_ui::current_frame_nanos();
        let scroll_result = self.scroll_recognizer.with_mut(|recognizer| {
            recognizer.update(input.pass, input.pointer_changes.as_mut_slice())
        });
        let scroll_delta = self
            .axis
            .scroll_delta(scroll_result.delta_x, scroll_result.delta_y);

        if scroll_delta.abs() >= 0.01 {
            self.controller.with_mut(|controller| {
                controller.apply_scroll_delta(scroll_delta, frame_nanos);
                controller.end_drag();
            });
            return;
        }

        let drag_result = self.drag_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                is_cursor_in_component,
            )
        });
        if drag_result.started {
            self.controller
                .with_mut(|controller| controller.start_drag(frame_nanos));
        }

        let drag_delta = self
            .axis
            .scroll_delta(drag_result.delta_x.to_f32(), drag_result.delta_y.to_f32());
        if drag_result.updated && drag_delta.abs() >= 0.01 {
            self.controller
                .with_mut(|controller| controller.apply_scroll_delta(drag_delta, frame_nanos));
        }

        if drag_result.ended {
            self.controller.with_mut(|controller| controller.end_drag());
        }
    }
}

fn apply_pager_input_modifiers(
    base: Modifier,
    controller: State<PagerController>,
    axis: PagerAxis,
    user_scroll_enabled: bool,
    drag_recognizer: State<DragRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
) -> Modifier {
    base.push_keyboard_input(PagerKeyboardModifierNode {
        controller,
        axis,
        user_scroll_enabled,
    })
    .push_pointer_input(PagerPointerModifierNode {
        controller,
        axis,
        user_scroll_enabled,
        drag_recognizer,
        scroll_recognizer,
    })
}

/// # horizontal_pager
///
/// Renders a horizontally swipeable pager that snaps between full-width pages.
///
/// ## Usage
///
/// Build onboarding flows, image carousels, or horizontally paged dashboards.
///
/// ## Parameters
///
/// - `args` — configures paging, spacing, and layout behavior; see
///   [`PagerConfig`].
/// - `page_content` — closure that renders each page by index.
///
/// ## Examples
///
/// ```
/// use tessera_components::pager::horizontal_pager;
/// use tessera_components::text::text;
/// use tessera_ui::{LayoutResult, remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
///     let start_page = remember(|| 0usize);
///     start_page.with_mut(|value| *value = 2);
///     assert_eq!(start_page.get(), 2);
///
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(move || {
///             horizontal_pager()
///                 .page_count(4)
///                 .initial_page(start_page.get())
///                 .page_content(|page| {
///                     text().content(format!("Page {page}"));
///                 });
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn horizontal_pager(
    modifier: Option<Modifier>,
    page_count: usize,
    initial_page: usize,
    page_size: PagerPageSize,
    page_spacing: Dp,
    content_padding: Dp,
    beyond_viewport_page_count: usize,
    cross_axis_alignment: CrossAxisAlignment,
    user_scroll_enabled: bool,
    snap_threshold: Option<f32>,
    scroll_smoothing: Option<f32>,
    page_content: Option<CallbackWith<usize>>,
    controller: Option<State<PagerController>>,
) {
    let pager_args = pager_config_from_params(PagerParams {
        modifier,
        page_count,
        initial_page,
        page_size,
        page_spacing,
        content_padding,
        beyond_viewport_page_count,
        cross_axis_alignment,
        user_scroll_enabled,
        snap_threshold,
        scroll_smoothing,
        page_content,
        controller,
    });
    let controller = pager_args
        .controller
        .unwrap_or_else(|| remember(|| PagerController::new(pager_args.initial_page)));
    pager_render(pager_args, controller, PagerAxis::Horizontal);
}

/// # vertical_pager
///
/// Renders a vertically swipeable pager that snaps between full-height pages.
///
/// ## Usage
///
/// Build vertically paged dashboards, step-by-step forms, or stacked galleries.
///
/// ## Parameters
///
/// - `args` — configures paging, spacing, and layout behavior; see
///   [`PagerConfig`].
/// - `page_content` — closure that renders each page by index.
///
/// ## Examples
///
/// ```
/// use tessera_components::pager::vertical_pager;
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
///     let start_page = remember(|| 1usize);
///     start_page.with_mut(|value| *value = 0);
///     assert_eq!(start_page.get(), 0);
///
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(move || {
///             vertical_pager().page_count(2).page_content(|page| {
///                 text().content(format!("Page {page}"));
///             });
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn vertical_pager(
    modifier: Option<Modifier>,
    page_count: usize,
    initial_page: usize,
    page_size: PagerPageSize,
    page_spacing: Dp,
    content_padding: Dp,
    beyond_viewport_page_count: usize,
    cross_axis_alignment: CrossAxisAlignment,
    user_scroll_enabled: bool,
    snap_threshold: Option<f32>,
    scroll_smoothing: Option<f32>,
    page_content: Option<CallbackWith<usize>>,
    controller: Option<State<PagerController>>,
) {
    let pager_args = pager_config_from_params(PagerParams {
        modifier,
        page_count,
        initial_page,
        page_size,
        page_spacing,
        content_padding,
        beyond_viewport_page_count,
        cross_axis_alignment,
        user_scroll_enabled,
        snap_threshold,
        scroll_smoothing,
        page_content,
        controller,
    });
    let controller = pager_args
        .controller
        .unwrap_or_else(|| remember(|| PagerController::new(pager_args.initial_page)));
    pager_render(pager_args, controller, PagerAxis::Vertical);
}

fn pager_render(args: PagerConfig, controller: State<PagerController>, axis: PagerAxis) {
    let page_content = args.page_content;
    let should_set_page_count = controller.with(|current| {
        let mut next = current.clone();
        next.set_page_count(args.page_count);
        next != *current
    });
    if should_set_page_count {
        controller.with_mut(|current| current.set_page_count(args.page_count));
    }
    let frame_nanos = tessera_ui::current_frame_nanos();
    if controller.with(|current| current.has_pending_animation_frame(frame_nanos)) {
        receive_frame_nanos(move |frame_nanos| {
            let has_pending_animation_frame = controller.with_mut(|current| {
                current.tick(frame_nanos, args.snap_threshold, args.scroll_smoothing);
                current.has_pending_animation_frame(frame_nanos)
            });
            if has_pending_animation_frame {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let current_page = controller.with(|current| current.current_page());
    let visible_pages = compute_visible_pages(
        current_page,
        args.page_count,
        args.beyond_viewport_page_count,
    );

    if visible_pages.is_empty() {
        layout()
            .modifier(args.modifier.clone())
            .layout_policy(ZeroLayout);
        return;
    }

    let drag_recognizer = remember(move || {
        DragRecognizer::new(DragSettings {
            axis: Some(match axis {
                PagerAxis::Horizontal => DragAxis::Horizontal,
                PagerAxis::Vertical => DragAxis::Vertical,
            }),
            ..DragSettings::default()
        })
    });
    let scroll_recognizer = remember(|| ScrollRecognizer::new(ScrollSettings { consume: true }));
    let modifier = apply_pager_input_modifiers(
        args.modifier.clone().focusable().focus_properties(
            FocusProperties::new()
                .can_focus(args.user_scroll_enabled && args.page_count > 1)
                .can_request_focus(args.user_scroll_enabled && args.page_count > 1),
        ),
        controller,
        axis,
        args.user_scroll_enabled,
        drag_recognizer,
        scroll_recognizer,
    );

    let policy = PagerLayout {
        axis,
        cross_axis_alignment: args.cross_axis_alignment,
        page_size: args.page_size,
        page_spacing: sanitize_spacing(Px::from(args.page_spacing)),
        content_padding: sanitize_spacing(Px::from(args.content_padding)),
        page_count: args.page_count,
        visible_pages: visible_pages.clone(),
        scroll_offset: controller.with(|current| current.scroll_offset_px()),
        controller,
    };
    layout()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            for page_index in visible_pages.iter().copied() {
                key(page_index, || {
                    page_content.call(page_index);
                });
            }
        });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PagerKeyboardCommand {
    Previous,
    Next,
    First,
    Last,
}

fn pager_keyboard_command(
    axis: PagerAxis,
    logical_key: &winit::keyboard::Key,
) -> Option<PagerKeyboardCommand> {
    use winit::keyboard::{Key, NamedKey};

    match logical_key {
        Key::Named(NamedKey::Home) => Some(PagerKeyboardCommand::First),
        Key::Named(NamedKey::End) => Some(PagerKeyboardCommand::Last),
        Key::Named(NamedKey::PageUp) => Some(PagerKeyboardCommand::Previous),
        Key::Named(NamedKey::PageDown) => Some(PagerKeyboardCommand::Next),
        Key::Named(NamedKey::ArrowLeft) if axis == PagerAxis::Horizontal => {
            Some(PagerKeyboardCommand::Previous)
        }
        Key::Named(NamedKey::ArrowRight) if axis == PagerAxis::Horizontal => {
            Some(PagerKeyboardCommand::Next)
        }
        Key::Named(NamedKey::ArrowUp) if axis == PagerAxis::Vertical => {
            Some(PagerKeyboardCommand::Previous)
        }
        Key::Named(NamedKey::ArrowDown) if axis == PagerAxis::Vertical => {
            Some(PagerKeyboardCommand::Next)
        }
        _ => None,
    }
}

fn run_pager_keyboard_command(
    controller: State<PagerController>,
    command: PagerKeyboardCommand,
) -> bool {
    controller.with_mut(|controller| {
        if controller.page_count <= 1 {
            return false;
        }

        let target = match command {
            PagerKeyboardCommand::Previous => {
                if controller.current_page == 0 {
                    return false;
                }
                controller.current_page - 1
            }
            PagerKeyboardCommand::Next => {
                if controller.current_page + 1 >= controller.page_count {
                    return false;
                }
                controller.current_page + 1
            }
            PagerKeyboardCommand::First => {
                if controller.current_page == 0 {
                    return false;
                }
                0
            }
            PagerKeyboardCommand::Last => {
                let last_page = controller.page_count - 1;
                if controller.current_page == last_page {
                    return false;
                }
                last_page
            }
        };

        controller.scroll_to_page(target);
        true
    })
}
