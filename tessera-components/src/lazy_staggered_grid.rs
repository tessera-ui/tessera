//! Virtualized staggered grid components for masonry layouts.
//!
//! ## Usage
//!
//! Use staggered grids to show variable-size tiles in galleries or feeds.
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Range,
    sync::Arc,
};

use parking_lot::Mutex;
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, FocusDirection,
    LayoutResult, MeasurementError, Modifier, ParentConstraint, Px, PxPosition, RenderSlot, State,
    key,
    layout::{LayoutPolicy, MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    lazy_grid::GridCells,
    scrollable::{ScrollBarBehavior, ScrollBarLayout, ScrollableController, scrollable},
};

const DEFAULT_VIEWPORT_ITEMS: usize = 8;

/// Cell configuration for staggered grids.
pub type StaggeredGridCells = GridCells;

/// Persistent state shared by staggered grid components.
pub struct LazyStaggeredGridController {
    cache: StaggeredGridCache,
}

impl Default for LazyStaggeredGridController {
    fn default() -> Self {
        Self::new()
    }
}

impl LazyStaggeredGridController {
    /// Creates a new staggered grid state with default caches.
    pub fn new() -> Self {
        Self {
            cache: StaggeredGridCache::default(),
        }
    }
}

#[derive(Clone)]
struct LazyStaggeredGridCollectedSlots(Arc<Mutex<Vec<LazySlot>>>);

fn hash_key<K>(key: K) -> u64
where
    K: Hash,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn collect_lazy_staggered_grid_slots(content: RenderSlot) -> Vec<LazySlot> {
    let collected = LazyStaggeredGridCollectedSlots(Arc::new(Mutex::new(Vec::new())));
    provide_context(
        || collected.clone(),
        move || {
            content.render();
        },
    );
    collected.0.lock().clone()
}

fn push_lazy_staggered_grid_slot(slot: LazySlot) {
    let collector = use_context::<LazyStaggeredGridCollectedSlots>()
        .expect(
            "lazy staggered grid item declarations must be used inside lazy staggered grid content",
        )
        .get();
    collector.0.lock().push(slot);
}

/// Adds a single lazily generated staggered-grid item declaration to the
/// current content slot.
pub fn lazy_item<F>(builder: F)
where
    F: Fn() + Send + Sync + 'static,
{
    push_lazy_staggered_grid_slot(LazySlot::items(
        1,
        move |_| {
            builder();
        },
        None,
    ));
}

/// Adds a single lazily generated staggered-grid item declaration with a
/// stable key.
pub fn lazy_item_with_key<K, F>(key: K, builder: F)
where
    K: Hash,
    F: Fn() + Send + Sync + 'static,
{
    let key_hash = hash_key(key);
    push_lazy_staggered_grid_slot(LazySlot::items(
        1,
        move |_| {
            builder();
        },
        Some(CallbackWith::new(move |_| key_hash)),
    ));
}

/// Adds a batch of lazily generated staggered-grid items to the current
/// content slot.
pub fn lazy_items<F>(count: usize, builder: F)
where
    F: Fn(usize) + Send + Sync + 'static,
{
    push_lazy_staggered_grid_slot(LazySlot::items(count, builder, None));
}

/// Adds a batch of lazily generated staggered-grid items with a stable key
/// provider.
pub fn lazy_items_with_key<K, KF, F>(count: usize, key_provider: KF, builder: F)
where
    K: Hash,
    KF: Fn(usize) -> K + Send + Sync + 'static,
    F: Fn(usize) + Send + Sync + 'static,
{
    let key_provider = CallbackWith::new(move |idx| hash_key(key_provider(idx)));
    push_lazy_staggered_grid_slot(LazySlot::items(count, builder, Some(key_provider)));
}

/// Adds lazily generated staggered-grid items from an iterator, providing both
/// index and element reference.
pub fn lazy_items_from_iter<I, T, F>(iter: I, builder: F)
where
    I: IntoIterator<Item = T>,
    T: Send + Sync + 'static,
    F: Fn(usize, &T) + Send + Sync + 'static,
{
    let items: Arc<Vec<T>> = Arc::new(iter.into_iter().collect());
    if items.is_empty() {
        return;
    }
    let builder = Arc::new(builder);
    let count = items.len();
    push_lazy_staggered_grid_slot(LazySlot::items(
        count,
        {
            let items = items.clone();
            let builder = builder.clone();
            move |idx| {
                if let Some(item) = items.get(idx) {
                    builder(idx, item);
                }
            }
        },
        None,
    ));
}

/// Adds lazily generated staggered-grid items from an iterator with a stable
/// key provider.
pub fn lazy_items_from_iter_with_key<I, T, K, KF, F>(iter: I, key_provider: KF, builder: F)
where
    I: IntoIterator<Item = T>,
    T: Send + Sync + 'static,
    K: Hash,
    KF: Fn(usize, &T) -> K + Send + Sync + 'static,
    F: Fn(usize, &T) + Send + Sync + 'static,
{
    let items: Arc<Vec<T>> = Arc::new(iter.into_iter().collect());
    if items.is_empty() {
        return;
    }
    let builder = Arc::new(builder);
    let key_provider = Arc::new(key_provider);
    let count = items.len();

    let slot_builder = {
        let items = items.clone();
        let builder = builder.clone();
        move |idx: usize| {
            if let Some(item) = items.get(idx) {
                builder(idx, item);
            }
        }
    };

    let slot_key_provider = {
        let items = items.clone();
        let key_provider = key_provider.clone();
        move |idx: usize| -> u64 {
            items
                .get(idx)
                .map(|item| hash_key(key_provider(idx, item)))
                .unwrap_or(0)
        }
    };

    push_lazy_staggered_grid_slot(LazySlot::items(
        count,
        slot_builder,
        Some(CallbackWith::new(slot_key_provider)),
    ));
}

/// Convenience helper for iterators when only the element is needed.
pub fn lazy_items_from_iter_values<I, T, F>(iter: I, builder: F)
where
    I: IntoIterator<Item = T>,
    T: Send + Sync + 'static,
    F: Fn(&T) + Send + Sync + 'static,
{
    lazy_items_from_iter(iter, move |_, item| builder(item));
}

/// # lazy_vertical_staggered_grid
///
/// A vertically scrolling staggered grid for masonry galleries and feeds.
///
/// ## Usage
///
/// Display mixed-height tiles in photo galleries or content feeds.
///
/// ## Parameters
///
/// - `modifier` - optional modifier for the scroll container.
/// - `scroll_smoothing` - interpolation factor used when animating scroll
///   position.
/// - `scrollbar_behavior` - visibility behavior of the scrollbars.
/// - `scrollbar_track_color` - optional scrollbar track color override.
/// - `scrollbar_thumb_color` - optional scrollbar thumb color override.
/// - `scrollbar_thumb_hover_color` - optional scrollbar thumb hover color
///   override.
/// - `scrollbar_layout` - whether scrollbars are overlaid or laid out alongside
///   content.
/// - `columns` - lane definition for columns.
/// - `main_axis_spacing` - spacing between items within a lane.
/// - `cross_axis_spacing` - spacing between lanes.
/// - `cross_axis_alignment` - how lanes are arranged when extra cross-axis
///   space is available.
/// - `item_alignment` - alignment of items within each lane.
/// - `overscan` - number of extra items instantiated before and after the
///   viewport.
/// - `estimated_item_size` - estimated main-axis size for each item.
/// - `content_padding` - symmetric padding applied around the grid content.
/// - `max_viewport_main` - optional maximum viewport length reported back to
///   parents.
/// - `controller` - optional external controller for scroll position and cache.
/// - `content` - optional slot builder for lazy staggered grid content.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_staggered_grid::{StaggeredGridCells, lazy_vertical_staggered_grid},
///     text::text,
/// };
/// use tessera_ui::{LayoutResult, remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_vertical_staggered_grid()
///         .columns(StaggeredGridCells::fixed(2))
///         .overscan(0)
///         .content(move || {
///             tessera_components::lazy_staggered_grid::lazy_items(4, move |i| {
///                 rendered.with_mut(|count| *count += 1);
///                 text().content(format!("Tile {i}"));
///             });
///         });
///     assert_eq!(rendered.get(), 4);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_vertical_staggered_grid(
    modifier: Option<Modifier>,
    scroll_smoothing: Option<f32>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: Option<ScrollBarLayout>,
    columns: Option<StaggeredGridCells>,
    main_axis_spacing: Option<Dp>,
    cross_axis_spacing: Option<Dp>,
    cross_axis_alignment: Option<MainAxisAlignment>,
    item_alignment: Option<CrossAxisAlignment>,
    overscan: Option<usize>,
    estimated_item_size: Option<Dp>,
    content_padding: Option<Dp>,
    max_viewport_main: Option<Px>,
    controller: Option<State<LazyStaggeredGridController>>,
    #[prop(skip_setter)] content: Option<RenderSlot>,
) {
    let scroll_smoothing = scroll_smoothing.unwrap_or(0.12);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let scrollbar_layout = scrollbar_layout.unwrap_or_default();
    let columns = columns.unwrap_or_default();
    let main_axis_spacing = main_axis_spacing.unwrap_or(Dp(0.0));
    let cross_axis_spacing = cross_axis_spacing.unwrap_or(Dp(0.0));
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let item_alignment = item_alignment.unwrap_or_default();
    let overscan = overscan.unwrap_or(0);
    let estimated_item_size = estimated_item_size.unwrap_or(Dp(0.0));
    let content_padding = content_padding.unwrap_or(Dp(0.0));
    let content = content.unwrap_or_else(RenderSlot::empty);
    let slots = collect_vertical_staggered_grid_slots(content);
    let controller = controller.unwrap_or_else(|| remember(LazyStaggeredGridController::new));
    lazy_vertical_staggered_grid_slots(LazyStaggeredGridSlotsArgs {
        modifier: modifier.unwrap_or_default(),
        scroll_smoothing,
        scrollbar_behavior,
        scrollbar_track_color,
        scrollbar_thumb_color,
        scrollbar_thumb_hover_color,
        scrollbar_layout,
        grid_cells: columns,
        main_axis_spacing,
        cross_axis_spacing,
        cross_axis_alignment,
        item_alignment,
        overscan,
        estimated_item_size,
        content_padding,
        max_viewport_main,
        controller,
        slots,
    });
}

fn collect_vertical_staggered_grid_slots(content: RenderSlot) -> Vec<LazySlot> {
    collect_lazy_staggered_grid_slots(content)
}

#[derive(Clone)]
struct LazyStaggeredGridSlotsArgs {
    modifier: Modifier,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: ScrollBarLayout,
    grid_cells: StaggeredGridCells,
    main_axis_spacing: Dp,
    cross_axis_spacing: Dp,
    cross_axis_alignment: MainAxisAlignment,
    item_alignment: CrossAxisAlignment,
    overscan: usize,
    estimated_item_size: Dp,
    content_padding: Dp,
    max_viewport_main: Option<Px>,
    controller: State<LazyStaggeredGridController>,
    slots: Vec<LazySlot>,
}

fn lazy_vertical_staggered_grid_slots(args: LazyStaggeredGridSlotsArgs) {
    let scroll_controller = remember(ScrollableController::default);
    let main_axis_spacing = sanitize_spacing(Px::from(args.main_axis_spacing));
    let cross_axis_spacing = sanitize_spacing(Px::from(args.cross_axis_spacing));
    let estimated_item_main = ensure_positive_px(Px::from(args.estimated_item_size));
    let padding_main = sanitize_spacing(Px::from(args.content_padding));
    let padding_cross = sanitize_spacing(Px::from(args.content_padding));
    let mut scrollable_builder = scrollable()
        .modifier(args.modifier)
        .vertical(true)
        .horizontal(false)
        .scroll_smoothing(args.scroll_smoothing)
        .scrollbar_behavior(args.scrollbar_behavior)
        .scrollbar_layout(args.scrollbar_layout)
        .controller(scroll_controller);
    if let Some(color) = args.scrollbar_track_color {
        scrollable_builder = scrollable_builder.scrollbar_track_color(color);
    }
    if let Some(color) = args.scrollbar_thumb_color {
        scrollable_builder = scrollable_builder.scrollbar_thumb_color(color);
    }
    if let Some(color) = args.scrollbar_thumb_hover_color {
        scrollable_builder = scrollable_builder.scrollbar_thumb_hover_color(color);
    }
    scrollable_builder.child(move || {
        let mut builder = lazy_staggered_grid_view()
            .axis(StaggeredGridAxis::Vertical)
            .grid_cells(args.grid_cells.clone())
            .main_axis_spacing(main_axis_spacing)
            .cross_axis_spacing(cross_axis_spacing)
            .cross_axis_alignment(args.cross_axis_alignment)
            .item_alignment(args.item_alignment)
            .estimated_item_main(estimated_item_main)
            .overscan(args.overscan)
            .padding_main(padding_main)
            .padding_cross(padding_cross)
            .slots(args.slots.clone())
            .controller(args.controller)
            .scroll_controller(scroll_controller);
        if let Some(max_viewport_main) = args.max_viewport_main {
            builder = builder.max_viewport_main(max_viewport_main);
        }
        drop(builder);
    });
}

/// # lazy_horizontal_staggered_grid
///
/// A horizontally scrolling staggered grid for masonry carousels.
///
/// ## Usage
///
/// Display mixed-width tiles in horizontally scrolling galleries.
///
/// ## Parameters
///
/// - `modifier` - optional modifier for the scroll container.
/// - `scroll_smoothing` - interpolation factor used when animating scroll
///   position.
/// - `scrollbar_behavior` - visibility behavior of the scrollbars.
/// - `scrollbar_track_color` - optional scrollbar track color override.
/// - `scrollbar_thumb_color` - optional scrollbar thumb color override.
/// - `scrollbar_thumb_hover_color` - optional scrollbar thumb hover color
///   override.
/// - `scrollbar_layout` - whether scrollbars are overlaid or laid out alongside
///   content.
/// - `rows` - lane definition for rows.
/// - `main_axis_spacing` - spacing between items within a lane.
/// - `cross_axis_spacing` - spacing between lanes.
/// - `cross_axis_alignment` - how lanes are arranged when extra cross-axis
///   space is available.
/// - `item_alignment` - alignment of items within each lane.
/// - `overscan` - number of extra items instantiated before and after the
///   viewport.
/// - `estimated_item_size` - estimated main-axis size for each item.
/// - `content_padding` - symmetric padding applied around the grid content.
/// - `max_viewport_main` - optional maximum viewport length reported back to
///   parents.
/// - `controller` - optional external controller for scroll position and cache.
/// - `content` - optional slot builder for lazy staggered grid content.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_staggered_grid::{StaggeredGridCells, lazy_horizontal_staggered_grid},
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_horizontal_staggered_grid()
///         .rows(StaggeredGridCells::fixed(2))
///         .overscan(0)
///         .content(move || {
///             tessera_components::lazy_staggered_grid::lazy_items(3, move |i| {
///                 rendered.with_mut(|count| *count += 1);
///                 text().content(format!("Tile {i}"));
///             });
///         });
///     assert_eq!(rendered.get(), 3);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_horizontal_staggered_grid(
    modifier: Option<Modifier>,
    scroll_smoothing: Option<f32>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: Option<ScrollBarLayout>,
    rows: Option<StaggeredGridCells>,
    main_axis_spacing: Option<Dp>,
    cross_axis_spacing: Option<Dp>,
    cross_axis_alignment: Option<MainAxisAlignment>,
    item_alignment: Option<CrossAxisAlignment>,
    overscan: Option<usize>,
    estimated_item_size: Option<Dp>,
    content_padding: Option<Dp>,
    max_viewport_main: Option<Px>,
    controller: Option<State<LazyStaggeredGridController>>,
    #[prop(skip_setter)] content: Option<RenderSlot>,
) {
    let scroll_smoothing = scroll_smoothing.unwrap_or(0.12);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let scrollbar_layout = scrollbar_layout.unwrap_or_default();
    let rows = rows.unwrap_or_default();
    let main_axis_spacing = main_axis_spacing.unwrap_or(Dp(0.0));
    let cross_axis_spacing = cross_axis_spacing.unwrap_or(Dp(0.0));
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let item_alignment = item_alignment.unwrap_or_default();
    let overscan = overscan.unwrap_or(0);
    let estimated_item_size = estimated_item_size.unwrap_or(Dp(0.0));
    let content_padding = content_padding.unwrap_or(Dp(0.0));
    let content = content.unwrap_or_else(RenderSlot::empty);
    let slots = collect_horizontal_staggered_grid_slots(content);
    let controller = controller.unwrap_or_else(|| remember(LazyStaggeredGridController::new));
    lazy_horizontal_staggered_grid_slots(LazyStaggeredGridSlotsArgs {
        modifier: modifier.unwrap_or_default(),
        scroll_smoothing,
        scrollbar_behavior,
        scrollbar_track_color,
        scrollbar_thumb_color,
        scrollbar_thumb_hover_color,
        scrollbar_layout,
        grid_cells: rows,
        main_axis_spacing,
        cross_axis_spacing,
        cross_axis_alignment,
        item_alignment,
        overscan,
        estimated_item_size,
        content_padding,
        max_viewport_main,
        controller,
        slots,
    });
}

fn collect_horizontal_staggered_grid_slots(content: RenderSlot) -> Vec<LazySlot> {
    collect_lazy_staggered_grid_slots(content)
}

fn lazy_horizontal_staggered_grid_slots(args: LazyStaggeredGridSlotsArgs) {
    let scroll_controller = remember(ScrollableController::default);
    let main_axis_spacing = sanitize_spacing(Px::from(args.main_axis_spacing));
    let cross_axis_spacing = sanitize_spacing(Px::from(args.cross_axis_spacing));
    let estimated_item_main = ensure_positive_px(Px::from(args.estimated_item_size));
    let padding_main = sanitize_spacing(Px::from(args.content_padding));
    let padding_cross = sanitize_spacing(Px::from(args.content_padding));
    let mut scrollable_builder = scrollable()
        .modifier(args.modifier)
        .vertical(false)
        .horizontal(true)
        .scroll_smoothing(args.scroll_smoothing)
        .scrollbar_behavior(args.scrollbar_behavior)
        .scrollbar_layout(args.scrollbar_layout)
        .controller(scroll_controller);
    if let Some(color) = args.scrollbar_track_color {
        scrollable_builder = scrollable_builder.scrollbar_track_color(color);
    }
    if let Some(color) = args.scrollbar_thumb_color {
        scrollable_builder = scrollable_builder.scrollbar_thumb_color(color);
    }
    if let Some(color) = args.scrollbar_thumb_hover_color {
        scrollable_builder = scrollable_builder.scrollbar_thumb_hover_color(color);
    }
    scrollable_builder.child(move || {
        let mut builder = lazy_staggered_grid_view()
            .axis(StaggeredGridAxis::Horizontal)
            .grid_cells(args.grid_cells.clone())
            .main_axis_spacing(main_axis_spacing)
            .cross_axis_spacing(cross_axis_spacing)
            .cross_axis_alignment(args.cross_axis_alignment)
            .item_alignment(args.item_alignment)
            .estimated_item_main(estimated_item_main)
            .overscan(args.overscan)
            .padding_main(padding_main)
            .padding_cross(padding_cross)
            .slots(args.slots.clone())
            .controller(args.controller)
            .scroll_controller(scroll_controller);
        if let Some(max_viewport_main) = args.max_viewport_main {
            builder = builder.max_viewport_main(max_viewport_main);
        }
        drop(builder);
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum StaggeredGridAxis {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct ZeroLayout;

impl LayoutPolicy for ZeroLayout {
    fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData::ZERO))
    }
}

#[derive(Clone, PartialEq, Eq)]
struct VisibleStaggeredLayoutItem {
    item_index: usize,
}

#[derive(Clone)]
struct LazyStaggeredGridLayout {
    axis: StaggeredGridAxis,
    item_alignment: CrossAxisAlignment,
    estimated_item_main: Px,
    main_spacing: Px,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
    viewport_limit: Px,
    total_count: usize,
    slots: GridSlots,
    visible_items: Vec<VisibleStaggeredLayoutItem>,
    controller: State<LazyStaggeredGridController>,
    scroll_controller: State<ScrollableController>,
}

impl PartialEq for LazyStaggeredGridLayout {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.item_alignment == other.item_alignment
            && self.estimated_item_main == other.estimated_item_main
            && self.main_spacing == other.main_spacing
            && self.max_viewport_main == other.max_viewport_main
            && self.padding_main == other.padding_main
            && self.padding_cross == other.padding_cross
            && self.viewport_limit == other.viewport_limit
            && self.total_count == other.total_count
            && self.slots == other.slots
            && self.visible_items == other.visible_items
    }
}

impl LayoutPolicy for LazyStaggeredGridLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() != self.visible_items.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy staggered grid measured child count mismatch".into(),
            ));
        }

        let (placements, total_main) = self.controller.with_mut(|c| {
            let mut placements = Vec::with_capacity(self.visible_items.len());
            let mut visible_iter = self.visible_items.iter().zip(children.iter()).peekable();

            let mut lane_offsets = vec![Px::ZERO; self.slots.len()];

            for index in 0..self.total_count {
                if self.slots.len() == 0 {
                    break;
                }

                let lane = find_shortest_lane(&lane_offsets);
                let lane_cross = self.slots.sizes.get(lane).copied().unwrap_or(Px::ZERO);
                let item_start = lane_offsets[lane];
                let mut item_main = c.cache.item_main(index).unwrap_or(self.estimated_item_main);

                if let Some((visible, child)) = visible_iter.peek()
                    && visible.item_index == index
                {
                    let child_constraint =
                        self.axis.child_constraint(lane_cross, self.item_alignment);
                    let child_size = child.measure(&child_constraint)?;
                    let child_size = child_size.size();
                    item_main = self.axis.main(&child_size);
                    c.cache.record_measurement(index, item_main);

                    let cell_offset = compute_cell_offset(
                        lane_cross,
                        self.axis.cross(&child_size),
                        self.item_alignment,
                    );
                    let cross_offset = self.padding_cross
                        + self.slots.positions.get(lane).copied().unwrap_or(Px::ZERO)
                        + cell_offset;
                    let position = self
                        .axis
                        .position(item_start + self.padding_main, cross_offset);
                    placements.push((**child, position));

                    visible_iter.next();
                }

                lane_offsets[lane] = item_start + item_main + self.main_spacing;
            }

            let total_main = finalize_lane_offsets(&lane_offsets, self.main_spacing);
            Ok::<_, MeasurementError>((placements, total_main))
        })?;

        let total_main_with_padding = total_main + self.padding_main + self.padding_main;
        let cross_with_padding = self.slots.cross_size + self.padding_cross + self.padding_cross;
        let size = self
            .axis
            .pack_size(total_main_with_padding, cross_with_padding);
        self.scroll_controller
            .with_mut(|c| c.override_child_size(size));

        let reported_main = clamp_reported_main(
            self.axis,
            input.parent_constraint(),
            total_main_with_padding,
            self.viewport_limit,
            self.max_viewport_main,
        );

        for (child, position) in placements {
            result.place_child(child, position);
        }

        Ok(result.with_size(self.axis.pack_size(reported_main, cross_with_padding)))
    }
}

#[tessera]
fn lazy_staggered_grid_view(
    axis: Option<StaggeredGridAxis>,
    grid_cells: Option<StaggeredGridCells>,
    main_axis_spacing: Option<Px>,
    cross_axis_spacing: Option<Px>,
    cross_axis_alignment: Option<MainAxisAlignment>,
    item_alignment: Option<CrossAxisAlignment>,
    estimated_item_main: Option<Px>,
    overscan: Option<usize>,
    max_viewport_main: Option<Px>,
    padding_main: Option<Px>,
    padding_cross: Option<Px>,
    controller: Option<State<LazyStaggeredGridController>>,
    slots: Option<Vec<LazySlot>>,
    scroll_controller: Option<State<ScrollableController>>,
) {
    let axis = axis.unwrap_or_default();
    let grid_cells = grid_cells.unwrap_or_default();
    let main_axis_spacing = main_axis_spacing.unwrap_or(Px::ZERO);
    let cross_axis_spacing = cross_axis_spacing.unwrap_or(Px::ZERO);
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let item_alignment = item_alignment.unwrap_or_default();
    let estimated_item_main = estimated_item_main.unwrap_or(Px::ZERO);
    let overscan = overscan.unwrap_or(0);
    let padding_main = padding_main.unwrap_or(Px::ZERO);
    let padding_cross = padding_cross.unwrap_or(Px::ZERO);
    let slots = slots.unwrap_or_default();
    let controller = controller.expect("lazy_staggered_grid_view requires controller");
    let scroll_controller =
        scroll_controller.expect("lazy_staggered_grid_view requires scroll_controller");
    let plan = LazySlotPlan::new(slots.clone());
    let total_count = plan.total_count();

    let visible_size = scroll_controller.with(|s| s.visible_size());
    let available_cross = axis.visible_cross(visible_size);
    let available_cross = (available_cross - padding_cross * 2).max(Px::ZERO);
    let grid_slots = resolve_grid_slots(
        available_cross,
        &grid_cells,
        cross_axis_spacing,
        cross_axis_alignment,
    );
    let lane_count = grid_slots.len();

    controller.with_mut(|c| c.cache.set_item_count(total_count));
    let total_main = controller.with(|c| {
        if lane_count == 0 || total_count == 0 {
            return Px::ZERO;
        }
        let mut lane_offsets = vec![Px::ZERO; lane_count];
        for index in 0..total_count {
            let lane = find_shortest_lane(&lane_offsets);
            let item_main = c.cache.item_main(index).unwrap_or(estimated_item_main);
            lane_offsets[lane] = lane_offsets[lane] + item_main + main_axis_spacing;
        }
        finalize_lane_offsets(&lane_offsets, main_axis_spacing)
    });
    let total_main_with_padding = total_main + padding_main + padding_main;
    let cross_with_padding = grid_slots.cross_size + padding_cross + padding_cross;
    scroll_controller.with_mut(|c| {
        c.override_child_size(axis.pack_size(total_main_with_padding, cross_with_padding));
    });

    let scroll_offset = axis.scroll_offset(scroll_controller.with(|s| s.child_position()));
    let viewport_span = resolve_viewport_span(
        axis.visible_span(visible_size),
        estimated_item_main,
        main_axis_spacing,
    );
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);

    let visible_range = controller.with(|c| {
        compute_visible_range(
            &c.cache,
            total_count,
            lane_count,
            scroll_offset,
            viewport_span,
            overscan,
            estimated_item_main,
            main_axis_spacing,
        )
    });
    let visible_items = plan.visible_items(visible_range.clone());

    if visible_items.is_empty() {
        layout().layout_policy(ZeroLayout);
        return;
    }

    let focus_modifier =
        lazy_staggered_grid_focus_beyond_bounds_modifier(LazyStaggeredGridFocusArgs {
            axis,
            controller,
            scroll_controller,
            estimated_item_main,
            main_axis_spacing,
            total_count,
            total_main,
            viewport_span,
            visible_range,
            lane_count,
        });

    let viewport_limit = viewport_span + padding_main + padding_main;
    let visible_layout_items = visible_items
        .iter()
        .map(|item| VisibleStaggeredLayoutItem {
            item_index: item.item_index,
        })
        .collect();

    layout()
        .modifier(focus_modifier)
        .layout_policy(LazyStaggeredGridLayout {
            axis,
            item_alignment,
            estimated_item_main,
            main_spacing: main_axis_spacing,
            max_viewport_main,
            padding_main,
            padding_cross,
            viewport_limit,
            total_count,
            slots: grid_slots.clone(),
            visible_items: visible_layout_items,
            controller,
            scroll_controller,
        })
        .child(move || {
            for child in &visible_items {
                let child = child.clone();
                key(child.key_hash, || {
                    child.builder.call(child.local_index);
                });
            }
        });
}

fn resolve_viewport_span(current: Px, estimated: Px, spacing: Px) -> Px {
    if current > Px::ZERO {
        current
    } else {
        let per_item = estimated + spacing;
        px_mul(per_item, DEFAULT_VIEWPORT_ITEMS.max(1))
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_visible_range(
    cache: &StaggeredGridCache,
    total_count: usize,
    lane_count: usize,
    scroll_offset: Px,
    viewport_span: Px,
    overscan: usize,
    estimated_item_main: Px,
    spacing: Px,
) -> Range<usize> {
    if total_count == 0 || lane_count == 0 {
        return 0..0;
    }

    let viewport_start = scroll_offset;
    let viewport_end = scroll_offset + viewport_span;
    let mut lane_offsets = vec![Px::ZERO; lane_count];
    let mut first_visible = None;
    let mut last_visible = None;

    for index in 0..total_count {
        let item_main = cache.item_main(index).unwrap_or(estimated_item_main);
        let lane = find_shortest_lane(&lane_offsets);
        let item_start = lane_offsets[lane];
        let item_end = item_start + item_main;

        if item_end >= viewport_start && item_start <= viewport_end {
            if first_visible.is_none() {
                first_visible = Some(index);
            }
            last_visible = Some(index);
        }

        lane_offsets[lane] = item_end + spacing;
    }

    let (mut start, mut end) = match (first_visible, last_visible) {
        (Some(start), Some(end)) => (start, end + 1),
        _ => {
            let end = total_count;
            let start = total_count.saturating_sub(1);
            (start, end)
        }
    };

    start = start.saturating_sub(overscan);
    end = end.saturating_add(overscan).min(total_count);
    if start >= end {
        end = (start + 1).min(total_count);
        start = start.saturating_sub(1);
    }

    start..end
}

fn clamp_reported_main(
    axis: StaggeredGridAxis,
    parent_constraint: ParentConstraint<'_>,
    total_main: Px,
    viewport_span: Px,
    fallback_limit: Option<Px>,
) -> Px {
    let viewport = viewport_span.max(Px::ZERO);
    let mut report = total_main.min(viewport);
    if let Some(max_value) = axis.constraint_max(parent_constraint).or(fallback_limit) {
        report = report.min(max_value.max(Px::ZERO));
    }
    report
}

fn compute_cell_offset(cell_cross: Px, child_cross: Px, alignment: CrossAxisAlignment) -> Px {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Px::ZERO,
        CrossAxisAlignment::Center => (cell_cross - child_cross).max(Px::ZERO) / 2,
        CrossAxisAlignment::End => (cell_cross - child_cross).max(Px::ZERO),
    }
}

#[derive(Clone, Copy)]
enum FocusScrollDirection {
    Forward,
    Backward,
}

impl StaggeredGridAxis {
    fn main(&self, size: &ComputedData) -> Px {
        match self {
            Self::Vertical => size.height,
            Self::Horizontal => size.width,
        }
    }

    fn cross(&self, size: &ComputedData) -> Px {
        match self {
            Self::Vertical => size.width,
            Self::Horizontal => size.height,
        }
    }

    fn pack_size(&self, main: Px, cross: Px) -> ComputedData {
        match self {
            Self::Vertical => ComputedData {
                width: cross,
                height: main,
            },
            Self::Horizontal => ComputedData {
                width: main,
                height: cross,
            },
        }
    }

    fn position(&self, main: Px, cross: Px) -> PxPosition {
        match self {
            Self::Vertical => PxPosition { x: cross, y: main },
            Self::Horizontal => PxPosition { x: main, y: cross },
        }
    }

    fn visible_span(&self, size: ComputedData) -> Px {
        match self {
            Self::Vertical => size.height,
            Self::Horizontal => size.width,
        }
    }

    fn visible_cross(&self, size: ComputedData) -> Px {
        match self {
            Self::Vertical => size.width,
            Self::Horizontal => size.height,
        }
    }

    fn scroll_position(&self, offset: Px) -> PxPosition {
        match self {
            Self::Vertical => PxPosition::new(Px::ZERO, -offset),
            Self::Horizontal => PxPosition::new(-offset, Px::ZERO),
        }
    }

    fn scroll_offset(&self, position: PxPosition) -> Px {
        match self {
            Self::Vertical => (-position.y).max(Px::ZERO),
            Self::Horizontal => (-position.x).max(Px::ZERO),
        }
    }

    fn focus_scroll_direction(&self, direction: FocusDirection) -> Option<FocusScrollDirection> {
        match (self, direction) {
            (_, FocusDirection::Next | FocusDirection::Enter) => {
                Some(FocusScrollDirection::Forward)
            }
            (_, FocusDirection::Previous | FocusDirection::Exit) => {
                Some(FocusScrollDirection::Backward)
            }
            (Self::Vertical, FocusDirection::Down) => Some(FocusScrollDirection::Forward),
            (Self::Vertical, FocusDirection::Up) => Some(FocusScrollDirection::Backward),
            (Self::Horizontal, FocusDirection::Right) => Some(FocusScrollDirection::Forward),
            (Self::Horizontal, FocusDirection::Left) => Some(FocusScrollDirection::Backward),
            _ => None,
        }
    }

    fn child_constraint(&self, cross: Px, alignment: CrossAxisAlignment) -> Constraint {
        let cross = cross.max(Px::ZERO);
        let cross_constraint = match alignment {
            CrossAxisAlignment::Stretch => AxisConstraint::exact(cross),
            _ => AxisConstraint::new(Px::ZERO, Some(cross)),
        };

        match self {
            Self::Vertical => Constraint::new(cross_constraint, AxisConstraint::NONE),
            Self::Horizontal => Constraint::new(AxisConstraint::NONE, cross_constraint),
        }
    }

    fn constraint_max(&self, constraint: ParentConstraint<'_>) -> Option<Px> {
        match self {
            Self::Vertical => constraint.height().resolve_max(),
            Self::Horizontal => constraint.width().resolve_max(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct GridSlots {
    sizes: Vec<Px>,
    positions: Vec<Px>,
    cross_size: Px,
}

impl GridSlots {
    fn len(&self) -> usize {
        self.sizes.len()
    }
}

fn resolve_grid_slots(
    available_cross: Px,
    grid_cells: &StaggeredGridCells,
    spacing: Px,
    alignment: MainAxisAlignment,
) -> GridSlots {
    let spacing = sanitize_spacing(spacing);
    let sizes = match grid_cells {
        GridCells::Fixed(count) => {
            let count = (*count).max(1);
            calculate_cells_cross_axis_size(available_cross, count, spacing)
        }
        GridCells::Adaptive(min_size) => {
            let min_px = ensure_positive_px(Px::from(*min_size));
            let spacing = sanitize_spacing(spacing);
            let available_i32 = available_cross.0.max(0);
            let min_i32 = min_px.0.max(1);
            let spacing_i32 = spacing.0.max(0);
            let count = ((available_i32 + spacing_i32) / (min_i32 + spacing_i32)).max(1) as usize;
            calculate_cells_cross_axis_size(available_cross, count, spacing)
        }
        GridCells::FixedSize(size) => {
            let cell_size = ensure_positive_px(Px::from(*size));
            let spacing = sanitize_spacing(spacing);
            let available_i32 = available_cross.0.max(0);
            let cell_i32 = cell_size.0.max(1);
            let spacing_i32 = spacing.0.max(0);
            if cell_i32 + spacing_i32 < available_i32 + spacing_i32 {
                let count =
                    ((available_i32 + spacing_i32) / (cell_i32 + spacing_i32)).max(1) as usize;
                vec![cell_size; count]
            } else {
                vec![available_cross.max(Px::ZERO)]
            }
        }
    };

    if sizes.is_empty() {
        return GridSlots {
            sizes: Vec::new(),
            positions: Vec::new(),
            cross_size: Px::ZERO,
        };
    }

    let total_sizes = sizes.iter().copied().fold(Px::ZERO, |acc, size| acc + size);
    let base_spacing = if sizes.len() > 1 {
        px_mul(spacing, sizes.len().saturating_sub(1))
    } else {
        Px::ZERO
    };
    let content_size = total_sizes + base_spacing;
    let available_space = (available_cross - content_size).max(Px::ZERO);
    let (start_cross, extra_spacing) =
        calculate_alignment_offsets(available_space, sizes.len(), alignment);
    let gap = spacing + extra_spacing;

    let mut positions = Vec::with_capacity(sizes.len());
    let mut cursor = start_cross;
    for (pos, size) in sizes.iter().enumerate() {
        positions.push(cursor);
        cursor += *size;
        if pos + 1 < sizes.len() {
            cursor += gap;
        }
    }

    let cross_size =
        positions.last().copied().unwrap_or(Px::ZERO) + sizes.last().copied().unwrap_or(Px::ZERO);

    GridSlots {
        sizes,
        positions,
        cross_size,
    }
}

fn calculate_alignment_offsets(
    available_space: Px,
    count: usize,
    alignment: MainAxisAlignment,
) -> (Px, Px) {
    match alignment {
        MainAxisAlignment::Start => (Px::ZERO, Px::ZERO),
        MainAxisAlignment::Center => (available_space / 2, Px::ZERO),
        MainAxisAlignment::End => (available_space, Px::ZERO),
        MainAxisAlignment::SpaceEvenly => {
            if count > 0 {
                let s = available_space / (count as i32 + 1);
                (s, s)
            } else {
                (Px::ZERO, Px::ZERO)
            }
        }
        MainAxisAlignment::SpaceBetween => {
            if count > 1 {
                (Px::ZERO, available_space / (count as i32 - 1))
            } else if count == 1 {
                (available_space / 2, Px::ZERO)
            } else {
                (Px::ZERO, Px::ZERO)
            }
        }
        MainAxisAlignment::SpaceAround => {
            if count > 0 {
                let s = available_space / (count as i32);
                (s / 2, s)
            } else {
                (Px::ZERO, Px::ZERO)
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum LazySlot {
    Items(LazyItemsSlot),
}

impl LazySlot {
    fn items<F>(count: usize, builder: F, key_provider: Option<CallbackWith<usize, u64>>) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        Self::Items(LazyItemsSlot {
            count,
            builder: CallbackWith::new(builder),
            key_provider,
        })
    }

    fn len(&self) -> usize {
        match self {
            Self::Items(slot) => slot.count,
        }
    }
}

#[derive(Clone, PartialEq)]
struct LazyItemsSlot {
    count: usize,
    builder: CallbackWith<usize, ()>,
    key_provider: Option<CallbackWith<usize, u64>>,
}

#[derive(Clone, PartialEq)]
struct LazySlotPlan {
    entries: Vec<LazySlotEntry>,
    total_count: usize,
}

impl LazySlotPlan {
    fn new(slots: Vec<LazySlot>) -> Self {
        let mut entries = Vec::with_capacity(slots.len());
        let mut cursor = 0;
        for slot in slots {
            let len = slot.len();
            entries.push(LazySlotEntry {
                start: cursor,
                len,
                slot,
            });
            cursor += len;
        }
        Self {
            entries,
            total_count: cursor,
        }
    }

    fn total_count(&self) -> usize {
        self.total_count
    }

    fn visible_items(&self, range: Range<usize>) -> Vec<VisibleStaggeredItem> {
        let mut result = Vec::new();
        for index in range {
            if let Some((slot, local_index)) = self.resolve(index) {
                let key_hash = if let Some(provider) = &slot.key_provider {
                    provider.call(local_index)
                } else {
                    let mut hasher = DefaultHasher::new();
                    index.hash(&mut hasher);
                    hasher.finish()
                };

                result.push(VisibleStaggeredItem {
                    item_index: index,
                    local_index,
                    builder: slot.builder,
                    key_hash,
                });
            }
        }
        result
    }

    fn resolve(&self, index: usize) -> Option<(&LazyItemsSlot, usize)> {
        self.entries.iter().find_map(|entry| {
            if index >= entry.start && index < entry.start + entry.len {
                let local_index = index - entry.start;
                match &entry.slot {
                    LazySlot::Items(slot) => Some((slot, local_index)),
                }
            } else {
                None
            }
        })
    }
}

#[derive(Clone, PartialEq)]
struct LazySlotEntry {
    start: usize,
    len: usize,
    slot: LazySlot,
}

#[derive(Clone, PartialEq)]
struct VisibleStaggeredItem {
    item_index: usize,
    local_index: usize,
    builder: CallbackWith<usize, ()>,
    key_hash: u64,
}

struct LazyStaggeredGridFocusArgs {
    axis: StaggeredGridAxis,
    controller: State<LazyStaggeredGridController>,
    scroll_controller: State<ScrollableController>,
    estimated_item_main: Px,
    main_axis_spacing: Px,
    total_count: usize,
    total_main: Px,
    viewport_span: Px,
    visible_range: Range<usize>,
    lane_count: usize,
}

fn lazy_staggered_grid_focus_beyond_bounds_modifier(args: LazyStaggeredGridFocusArgs) -> Modifier {
    let current_scroll_offset = args
        .axis
        .scroll_offset(args.scroll_controller.with(|s| s.child_position()));
    let max_scroll = (args.total_main - args.viewport_span).max(Px::ZERO);
    Modifier::new().focus_beyond_bounds_handler(CallbackWith::new(move |direction| {
        let Some(scroll_direction) = args.axis.focus_scroll_direction(direction) else {
            return false;
        };
        if args.total_count == 0 || args.lane_count == 0 || args.viewport_span <= Px::ZERO {
            return false;
        }

        let target_index = match scroll_direction {
            FocusScrollDirection::Forward => {
                if args.visible_range.end >= args.total_count {
                    return false;
                }
                args.visible_range.end
            }
            FocusScrollDirection::Backward => {
                let Some(index) = args.visible_range.start.checked_sub(1) else {
                    return false;
                };
                index
            }
        };

        let Some((target_offset, target_main)) = args.controller.with(|c| {
            staggered_item_layout_info(
                &c.cache,
                target_index,
                args.lane_count,
                args.estimated_item_main,
                args.main_axis_spacing,
            )
        }) else {
            return false;
        };

        let desired_scroll = match scroll_direction {
            FocusScrollDirection::Forward => {
                (target_offset + target_main - args.viewport_span).max(Px::ZERO)
            }
            FocusScrollDirection::Backward => target_offset,
        }
        .min(max_scroll);

        if desired_scroll == current_scroll_offset {
            return false;
        }

        let position = args.axis.scroll_position(desired_scroll);
        args.scroll_controller
            .with_mut(|c| c.set_scroll_position(position));
        true
    }))
}

#[derive(PartialEq, Default)]
struct StaggeredGridCache {
    item_main: Vec<Option<Px>>,
}

impl StaggeredGridCache {
    fn set_item_count(&mut self, count: usize) {
        if self.item_main.len() == count {
            return;
        }
        self.item_main.resize(count, None);
    }

    fn item_main(&self, index: usize) -> Option<Px> {
        self.item_main.get(index).copied().flatten()
    }

    fn record_measurement(&mut self, index: usize, actual: Px) {
        if index >= self.item_main.len() {
            return;
        }
        self.item_main[index] = Some(actual);
    }
}

fn staggered_item_layout_info(
    cache: &StaggeredGridCache,
    target_index: usize,
    lane_count: usize,
    estimated_item_main: Px,
    spacing: Px,
) -> Option<(Px, Px)> {
    if lane_count == 0 || target_index >= cache.item_main.len() {
        return None;
    }

    let mut lane_offsets = vec![Px::ZERO; lane_count];
    for index in 0..=target_index {
        let lane = find_shortest_lane(&lane_offsets);
        let item_main = cache.item_main(index).unwrap_or(estimated_item_main);
        let item_offset = lane_offsets[lane];
        if index == target_index {
            return Some((item_offset, item_main));
        }
        lane_offsets[lane] = item_offset + item_main + spacing;
    }
    None
}

fn calculate_cells_cross_axis_size(available: Px, slot_count: usize, spacing: Px) -> Vec<Px> {
    let slot_count = slot_count.max(1);
    let spacing_total = px_mul(spacing, slot_count.saturating_sub(1));
    let available_without_spacing = (available - spacing_total).max(Px::ZERO);
    let base = available_without_spacing.0 / slot_count as i32;
    let remainder = available_without_spacing.0 % slot_count as i32;
    (0..slot_count)
        .map(|index| {
            let extra = if (index as i32) < remainder { 1 } else { 0 };
            Px(base + extra)
        })
        .collect()
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

fn ensure_positive_px(px: Px) -> Px {
    if px <= Px::ZERO { Px(1) } else { px }
}

fn sanitize_spacing(px: Px) -> Px {
    if px < Px::ZERO { Px::ZERO } else { px }
}

fn find_shortest_lane(lane_offsets: &[Px]) -> usize {
    let mut index = 0;
    let mut best = lane_offsets.first().copied().unwrap_or(Px::ZERO);
    for (i, offset) in lane_offsets.iter().enumerate().skip(1) {
        if *offset < best {
            best = *offset;
            index = i;
        }
    }
    index
}

fn finalize_lane_offsets(lane_offsets: &[Px], spacing: Px) -> Px {
    let max_offset = lane_offsets.iter().copied().max().unwrap_or(Px::ZERO);
    if max_offset == Px::ZERO {
        Px::ZERO
    } else {
        (max_offset - spacing).max(Px::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Modifier, NoopRenderPolicy, Px,
        PxPosition,
        layout::{MeasureScope, layout},
        remember, tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        modifier::{ModifierExt as _, SemanticsArgs},
        scrollable::{ScrollableController, scrollable},
    };

    use super::{
        GridCells, LazyStaggeredGridController, StaggeredGridAxis, lazy_staggered_grid_view,
    };

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    #[tessera]
    fn fixed_test_box(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or(0);
        let height = height.unwrap_or(0);

        layout()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn lazy_vertical_staggered_layout_case() {
        let controller = remember(LazyStaggeredGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(60),
                height: Px::new(60),
            });
        });

        lazy_staggered_grid_view()
            .axis(StaggeredGridAxis::Vertical)
            .grid_cells(GridCells::fixed(2))
            .main_axis_spacing(Px::new(3))
            .cross_axis_spacing(Px::new(3))
            .cross_axis_alignment(MainAxisAlignment::Start)
            .item_alignment(CrossAxisAlignment::Start)
            .estimated_item_main(Px::new(10))
            .overscan(0)
            .padding_main(Px::new(4))
            .padding_cross(Px::new(4))
            .slots(vec![super::LazySlot::items(
                4,
                |index| match index {
                    0 => {
                        fixed_test_box()
                            .tag("lazy_staggered_v_first".to_string())
                            .width(10)
                            .height(10);
                    }
                    1 => {
                        fixed_test_box()
                            .tag("lazy_staggered_v_second".to_string())
                            .width(12)
                            .height(8);
                    }
                    2 => {
                        fixed_test_box()
                            .tag("lazy_staggered_v_third".to_string())
                            .width(11)
                            .height(12);
                    }
                    _ => {
                        fixed_test_box()
                            .tag("lazy_staggered_v_fourth".to_string())
                            .width(9)
                            .height(9);
                    }
                },
                None,
            )])
            .controller(controller)
            .scroll_controller(scroll_controller);
    }

    #[tessera]
    fn lazy_horizontal_staggered_layout_case() {
        let controller = remember(LazyStaggeredGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(70),
                height: Px::new(40),
            });
        });

        lazy_staggered_grid_view()
            .axis(StaggeredGridAxis::Horizontal)
            .grid_cells(GridCells::fixed(2))
            .main_axis_spacing(Px::new(3))
            .cross_axis_spacing(Px::new(3))
            .cross_axis_alignment(MainAxisAlignment::Start)
            .item_alignment(CrossAxisAlignment::Start)
            .estimated_item_main(Px::new(10))
            .overscan(0)
            .padding_main(Px::new(4))
            .padding_cross(Px::new(4))
            .slots(vec![super::LazySlot::items(
                4,
                |index| match index {
                    0 => {
                        fixed_test_box()
                            .tag("lazy_staggered_h_first".to_string())
                            .width(20)
                            .height(10);
                    }
                    1 => {
                        fixed_test_box()
                            .tag("lazy_staggered_h_second".to_string())
                            .width(15)
                            .height(12);
                    }
                    2 => {
                        fixed_test_box()
                            .tag("lazy_staggered_h_third".to_string())
                            .width(18)
                            .height(8);
                    }
                    _ => {
                        fixed_test_box()
                            .tag("lazy_staggered_h_fourth".to_string())
                            .width(16)
                            .height(9);
                    }
                },
                None,
            )])
            .controller(controller)
            .scroll_controller(scroll_controller);
    }

    #[tessera]
    fn lazy_vertical_staggered_scrolled_layout_case() {
        let controller = remember(LazyStaggeredGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        let target_position = PxPosition::new(Px::ZERO, Px::new(-5));
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(60),
                height: Px::new(60),
            });
            c.set_scroll_position(target_position);
        });

        scrollable()
            .vertical(true)
            .horizontal(false)
            .controller(scroll_controller)
            .child(move || {
                lazy_staggered_grid_view()
                    .axis(StaggeredGridAxis::Vertical)
                    .grid_cells(GridCells::fixed(2))
                    .main_axis_spacing(Px::new(3))
                    .cross_axis_spacing(Px::new(3))
                    .cross_axis_alignment(MainAxisAlignment::Start)
                    .item_alignment(CrossAxisAlignment::Start)
                    .estimated_item_main(Px::new(10))
                    .overscan(0)
                    .padding_main(Px::new(4))
                    .padding_cross(Px::new(4))
                    .slots(vec![super::LazySlot::items(
                        4,
                        |index| match index {
                            0 => {
                                fixed_test_box()
                                    .tag("lazy_staggered_scroll_first".to_string())
                                    .width(10)
                                    .height(10);
                            }
                            1 => {
                                fixed_test_box()
                                    .tag("lazy_staggered_scroll_second".to_string())
                                    .width(12)
                                    .height(8);
                            }
                            2 => {
                                fixed_test_box()
                                    .tag("lazy_staggered_scroll_third".to_string())
                                    .width(11)
                                    .height(12);
                            }
                            _ => {
                                fixed_test_box()
                                    .tag("lazy_staggered_scroll_fourth".to_string())
                                    .width(9)
                                    .height(9);
                            }
                        },
                        None,
                    )])
                    .controller(controller)
                    .scroll_controller(scroll_controller);
            });
    }

    #[test]
    fn lazy_vertical_staggered_positions_items_in_shortest_lanes() {
        tessera_ui::assert_layout! {
            viewport: (80, 80),
            content: {
                lazy_vertical_staggered_layout_case();
            },
            expect: {
                node("lazy_staggered_v_first").position(4, 4).size(10, 10);
                node("lazy_staggered_v_second").position(32, 4).size(12, 8);
                node("lazy_staggered_v_third").position(32, 15).size(11, 12);
                node("lazy_staggered_v_fourth").position(4, 17).size(9, 9);
            }
        }
    }

    #[test]
    fn lazy_horizontal_staggered_positions_items_in_shortest_lanes() {
        tessera_ui::assert_layout! {
            viewport: (90, 50),
            content: {
                lazy_horizontal_staggered_layout_case();
            },
            expect: {
                node("lazy_staggered_h_first").position(4, 4).size(20, 10);
                node("lazy_staggered_h_second").position(4, 22).size(15, 12);
                node("lazy_staggered_h_third").position(22, 22).size(18, 8);
                node("lazy_staggered_h_fourth").position(27, 4).size(16, 9);
            }
        }
    }

    #[test]
    fn lazy_vertical_staggered_scroll_repositions_visible_items() {
        tessera_ui::assert_layout! {
            viewport: (80, 80),
            content: {
                lazy_vertical_staggered_scrolled_layout_case();
            },
            expect: {
                node("lazy_staggered_scroll_first").position(4, -1).size(10, 10);
                node("lazy_staggered_scroll_second").position(32, -1).size(12, 8);
                node("lazy_staggered_scroll_third").position(32, 10).size(11, 12);
                node("lazy_staggered_scroll_fourth").position(4, 12).size(9, 9);
            }
        }
    }
}
