//! Pager container for swipeable pages.
//!
//! ## Usage
//!
//! Show onboarding steps or media carousels that snap between pages.
use std::time::{Duration, Instant};

use derive_setters::Setters;
use tessera_ui::{
    ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, MeasurementError, Modifier,
    PressKeyEventType, Px, PxPosition, State, key,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    remember, tessera,
};

use crate::{
    alignment::CrossAxisAlignment, modifier::ModifierExt as _, pos_misc::is_position_in_component,
};

const DEFAULT_SNAP_THRESHOLD: f32 = 0.5;
const DEFAULT_SCROLL_SMOOTHING: f32 = 0.12;
const SNAP_IDLE_TIME: Duration = Duration::from_millis(120);

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
#[derive(Clone, Setters)]
pub struct PagerArgs {
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
}

impl Default for PagerArgs {
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
        }
    }
}

/// Controller for pager components.
#[derive(Clone)]
pub struct PagerController {
    current_page: usize,
    current_page_offset_fraction: f32,
    page_count: usize,
    page_size: Px,
    page_spacing: Px,
    scroll_offset: f32,
    target_offset: f32,
    last_frame_time: Option<Instant>,
    last_scroll_time: Option<Instant>,
    is_dragging: bool,
    last_drag_position: Option<PxPosition>,
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
            last_frame_time: None,
            last_scroll_time: None,
            is_dragging: false,
            last_drag_position: None,
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
        self.last_scroll_time = None;
        self.update_current_page_from_offset();
    }

    /// Scrolls toward the requested page using snap smoothing.
    pub fn scroll_to_page(&mut self, page: usize) {
        let page = self.clamp_page(page);
        self.target_offset = self.offset_for_page(page);
        self.last_scroll_time = None;
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

    fn tick(&mut self, now: Instant, snap_threshold: f32, scroll_smoothing: f32) {
        if self.page_count == 0 {
            return;
        }
        if self.page_distance() <= f32::EPSILON {
            return;
        }

        let snap_threshold = snap_threshold.clamp(0.0, 1.0);
        let scroll_smoothing = scroll_smoothing.clamp(0.0, 1.0);
        let idle = self
            .last_scroll_time
            .map(|t| now.duration_since(t) > SNAP_IDLE_TIME)
            .unwrap_or(true);

        if idle && !self.is_dragging {
            let target_page = self.snap_target_page(snap_threshold);
            self.target_offset = self.offset_for_page(target_page);
        }

        self.update_scroll_offset(now, scroll_smoothing);
        self.scroll_offset = self.clamp_offset(self.scroll_offset);
        self.update_current_page_from_offset();
    }

    fn apply_scroll_delta(&mut self, delta: f32, now: Instant) {
        if self.page_distance() <= f32::EPSILON || self.page_count == 0 {
            return;
        }
        self.scroll_offset = self.clamp_offset(self.scroll_offset + delta);
        self.target_offset = self.scroll_offset;
        self.last_scroll_time = Some(now);
        self.update_current_page_from_offset();
    }

    fn start_drag(&mut self, pos: PxPosition, now: Instant) {
        self.is_dragging = true;
        self.last_drag_position = Some(pos);
        self.last_scroll_time = Some(now);
    }

    fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_drag_position = None;
    }

    fn drag_delta(&mut self, pos: PxPosition, axis: PagerAxis) -> Option<f32> {
        let last = self.last_drag_position?;
        self.last_drag_position = Some(pos);
        Some(axis.drag_delta(last, pos))
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

    fn update_scroll_offset(&mut self, now: Instant, smoothing: f32) {
        let delta_time = if let Some(last) = self.last_frame_time {
            now.duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.last_frame_time = Some(now);

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

#[derive(Clone, Copy, PartialEq, Eq)]
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

    fn drag_delta(self, from: PxPosition, to: PxPosition) -> f32 {
        match self {
            Self::Horizontal => (to.x - from.x).to_f32(),
            Self::Vertical => (to.y - from.y).to_f32(),
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

impl LayoutSpec for PagerLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        if self.page_count == 0 {
            return Ok(ComputedData::min_from_constraint(
                input.parent_constraint().as_ref(),
            ));
        }

        if input.children_ids().len() != self.visible_pages.len() {
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
        let container_main = resolve_dimension(
            main_dimension,
            page_main + padding + padding,
            "pager main axis",
        );

        let cross_constraint =
            cross_dimension_for_alignment(cross_dimension, self.cross_axis_alignment);
        let child_constraint = match self.axis {
            PagerAxis::Horizontal => {
                Constraint::new(DimensionValue::Fixed(page_main), cross_constraint)
            }
            PagerAxis::Vertical => {
                Constraint::new(cross_constraint, DimensionValue::Fixed(page_main))
            }
        };

        let children_to_measure: Vec<_> = input
            .children_ids()
            .iter()
            .map(|&child_id| (child_id, child_constraint))
            .collect();
        let measurements = input.measure_children(children_to_measure)?;

        let mut max_cross = Px::ZERO;
        for size in measurements.values() {
            max_cross = max_cross.max(self.axis.cross(*size));
        }
        let container_cross = resolve_dimension(cross_dimension, max_cross, "pager cross axis");

        self.controller
            .with_mut(|c| c.update_layout(page_main, page_spacing, self.page_count));

        let scroll_offset = self.controller.with(|c| c.scroll_offset_px());
        let page_step = page_main + page_spacing;

        for (&child_id, &page_index) in input.children_ids().iter().zip(self.visible_pages.iter()) {
            let measured = measurements
                .get(&child_id)
                .copied()
                .unwrap_or(ComputedData::ZERO);
            let cross_offset = compute_cross_offset(
                container_cross,
                self.axis.cross(measured),
                self.cross_axis_alignment,
            );
            let page_offset = padding + px_mul(page_step, page_index) + scroll_offset;
            let position = self.axis.position(page_offset, cross_offset);
            output.place_child(child_id, position);
        }

        Ok(self.axis.pack_size(container_main, container_cross))
    }

    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().clips_children = true;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ZeroLayout;

impl LayoutSpec for ZeroLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData::min_from_constraint(
            input.parent_constraint().as_ref(),
        ))
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

fn clamp_wrap(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    min.unwrap_or(Px(0))
        .max(measure)
        .min(max.unwrap_or(Px::MAX))
}

fn fill_value(min: Option<Px>, max: Option<Px>, measure: Px, context: &str) -> Px {
    let Some(max) = max else {
        panic!("Pager cannot fill an unbounded {context}");
    };
    let mut value = max.max(measure);
    if let Some(min) = min {
        value = value.max(min);
    }
    value
}

fn resolve_dimension(dim: DimensionValue, measure: Px, context: &str) -> Px {
    match dim {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, measure),
        DimensionValue::Fill { min, max } => fill_value(min, max, measure, context),
    }
}

fn resolve_page_main_size(
    page_size: PagerPageSize,
    main_dimension: DimensionValue,
    padding: Px,
) -> Px {
    match page_size {
        PagerPageSize::Fill => {
            let max = main_dimension
                .get_max()
                .expect("Pager page size Fill requires a bounded main-axis constraint");
            (max - padding - padding).max(Px::ZERO)
        }
        PagerPageSize::Fixed(dp) => dp.into(),
    }
}

fn cross_dimension_for_alignment(
    cross_dimension: DimensionValue,
    alignment: CrossAxisAlignment,
) -> DimensionValue {
    let max = cross_dimension.get_max();
    match alignment {
        CrossAxisAlignment::Stretch => match cross_dimension {
            DimensionValue::Fixed(value) => DimensionValue::Fixed(value),
            _ => DimensionValue::Fill {
                min: cross_dimension.get_min(),
                max,
            },
        },
        _ => DimensionValue::Wrap { min: None, max },
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
///   [`PagerArgs`].
/// - `page_content` — closure that renders each page by index.
///
/// ## Examples
///
/// ```
/// use tessera_components::pager::{PagerArgs, horizontal_pager};
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let start_page = remember(|| 0usize);
///     start_page.with_mut(|value| *value = 2);
///     assert_eq!(start_page.get(), 2);
///
///     horizontal_pager(
///         PagerArgs::default()
///             .page_count(4)
///             .initial_page(start_page.get()),
///         |page| {
///             text(format!("Page {page}"));
///         },
///     );
/// }
///
/// demo();
/// ```
#[tessera]
pub fn horizontal_pager(args: PagerArgs, page_content: impl Fn(usize) + Send + Sync + 'static) {
    let controller = remember(|| PagerController::new(args.initial_page));
    horizontal_pager_with_controller(args, controller, page_content);
}

/// # horizontal_pager_with_controller
///
/// Horizontal pager variant that is driven by an explicit controller.
///
/// ## Usage
///
/// Use when you need to drive paging programmatically or read the current
/// page.
///
/// ## Parameters
///
/// - `args` — configures paging, spacing, and layout behavior; see
///   [`PagerArgs`].
/// - `controller` — a [`PagerController`] that tracks the current page and
///   scroll offset.
/// - `page_content` — closure that renders each page by index.
///
/// ## Examples
///
/// ```
/// use tessera_components::pager::{PagerArgs, PagerController, horizontal_pager_with_controller};
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| PagerController::new(1));
///     horizontal_pager_with_controller(PagerArgs::default().page_count(3), controller, |page| {
///         text(format!("Page {page}"));
///     });
///
///     assert_eq!(controller.with(|pager| pager.current_page()), 1);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn horizontal_pager_with_controller(
    args: PagerArgs,
    controller: State<PagerController>,
    page_content: impl Fn(usize) + Send + Sync + 'static,
) {
    let modifier = args.modifier;
    modifier.run(move || pager_inner(args, controller, PagerAxis::Horizontal, page_content));
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
///   [`PagerArgs`].
/// - `page_content` — closure that renders each page by index.
///
/// ## Examples
///
/// ```
/// use tessera_components::pager::{PagerArgs, vertical_pager};
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let start_page = remember(|| 1usize);
///     start_page.with_mut(|value| *value = 0);
///     assert_eq!(start_page.get(), 0);
///
///     vertical_pager(PagerArgs::default().page_count(2), |page| {
///         text(format!("Page {page}"));
///     });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn vertical_pager(args: PagerArgs, page_content: impl Fn(usize) + Send + Sync + 'static) {
    let controller = remember(|| PagerController::new(args.initial_page));
    vertical_pager_with_controller(args, controller, page_content);
}

/// # vertical_pager_with_controller
///
/// Vertical pager variant that is driven by an explicit controller.
///
/// ## Usage
///
/// Use when you need to drive paging programmatically or read the current
/// page.
///
/// ## Parameters
///
/// - `args` — configures paging, spacing, and layout behavior; see
///   [`PagerArgs`].
/// - `controller` — a [`PagerController`] that tracks the current page and
///   scroll offset.
/// - `page_content` — closure that renders each page by index.
///
/// ## Examples
///
/// ```
/// use tessera_components::pager::{PagerArgs, PagerController, vertical_pager_with_controller};
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| PagerController::new(0));
///     controller.with_mut(|pager| pager.scroll_to_page(1));
///     assert_eq!(controller.with(|pager| pager.current_page()), 0);
///
///     vertical_pager_with_controller(PagerArgs::default().page_count(2), controller, |page| {
///         text(format!("Page {page}"));
///     });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn vertical_pager_with_controller(
    args: PagerArgs,
    controller: State<PagerController>,
    page_content: impl Fn(usize) + Send + Sync + 'static,
) {
    let modifier = args.modifier;
    modifier.run(move || pager_inner(args, controller, PagerAxis::Vertical, page_content));
}

#[tessera]
fn pager_inner(
    args: PagerArgs,
    controller: State<PagerController>,
    axis: PagerAxis,
    page_content: impl Fn(usize) + Send + Sync + 'static,
) {
    controller.with_mut(|c| c.set_page_count(args.page_count));
    controller.with_mut(|c| {
        c.tick(Instant::now(), args.snap_threshold, args.scroll_smoothing);
    });

    let current_page = controller.with(|c| c.current_page());
    let visible_pages = compute_visible_pages(
        current_page,
        args.page_count,
        args.beyond_viewport_page_count,
    );

    if visible_pages.is_empty() {
        layout(ZeroLayout);
        return;
    }

    let page_spacing = sanitize_spacing(Px::from(args.page_spacing));
    let content_padding = sanitize_spacing(Px::from(args.content_padding));
    let scroll_offset = controller.with(|c| c.scroll_offset_px());
    layout(PagerLayout {
        axis,
        cross_axis_alignment: args.cross_axis_alignment,
        page_size: args.page_size,
        page_spacing,
        content_padding,
        page_count: args.page_count,
        visible_pages: visible_pages.clone(),
        scroll_offset,
        controller,
    });

    let user_scroll_enabled = args.user_scroll_enabled;
    input_handler(move |input| {
        if !user_scroll_enabled {
            return;
        }

        let is_cursor_in_component = input
            .cursor_position_rel
            .map(|pos| is_position_in_component(input.computed_data, pos))
            .unwrap_or(false);
        let is_dragging = controller.with(|c| c.is_dragging());
        if !is_cursor_in_component && !is_dragging {
            return;
        }

        let now = Instant::now();
        let mut scroll_delta = 0.0;
        for event in input.cursor_events.iter() {
            if let CursorEventContent::Scroll(scroll_event) = &event.content {
                let delta = axis.scroll_delta(scroll_event.delta_x, scroll_event.delta_y);
                if delta.abs() >= 0.01 {
                    scroll_delta += delta;
                }
            }
        }

        if scroll_delta.abs() >= 0.01 {
            controller.with_mut(|c| {
                c.apply_scroll_delta(scroll_delta, now);
                c.end_drag();
            });
            input
                .cursor_events
                .retain(|event| !matches!(event.content, CursorEventContent::Scroll(_)));
            return;
        }

        let mut drag_start_pos = None;
        let mut should_end_drag = false;
        for event in input.cursor_events.iter() {
            match &event.content {
                CursorEventContent::Pressed(PressKeyEventType::Left) => {
                    if is_cursor_in_component {
                        drag_start_pos = input.cursor_position_rel;
                    }
                }
                CursorEventContent::Released(PressKeyEventType::Left) => {
                    should_end_drag = true;
                }
                _ => {}
            }
        }

        controller.with_mut(|c| {
            if let Some(pos) = drag_start_pos {
                c.start_drag(pos, now);
            }
            if should_end_drag {
                c.end_drag();
            }
            if c.is_dragging()
                && let Some(pos) = input.cursor_position_rel
                && let Some(delta) = c.drag_delta(pos, axis)
            {
                c.apply_scroll_delta(delta, now);
            }
        });
    });

    let page_content = &page_content;
    for page_index in visible_pages {
        key(page_index, || {
            page_content(page_index);
        });
    }
}
