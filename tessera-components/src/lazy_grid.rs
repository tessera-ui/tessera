//! Virtualized grid components for efficient, scrollable tile layouts.
//!
//! ## Usage
//!
//! Use lazy grids to display large, scrollable collections of tiles.
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Range,
    sync::Arc,
};

use parking_lot::Mutex;
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, FocusDirection,
    MeasurementError, Modifier, NodeId, ParentConstraint, Px, PxPosition, RenderSlot, State, key,
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, layout_primitive},
    modifier::FocusModifierExt as _,
    provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    scrollable::{ScrollBarBehavior, ScrollBarLayout, ScrollableController, scrollable},
};

const DEFAULT_VIEWPORT_LINES: usize = 8;

/// Defines how grid slots are computed along the cross axis.
#[derive(Clone, PartialEq, Debug)]
pub enum GridCells {
    /// A fixed number of slots per line.
    Fixed(usize),
    /// As many slots as possible while keeping each slot at least `min_size`.
    Adaptive(Dp),
    /// Slots with a fixed cross-axis size.
    FixedSize(Dp),
}

impl GridCells {
    /// Creates a fixed-count slot configuration.
    pub fn fixed(count: usize) -> Self {
        Self::Fixed(count)
    }

    /// Creates an adaptive slot configuration with a minimum slot size.
    pub fn adaptive(min_size: Dp) -> Self {
        Self::Adaptive(min_size)
    }

    /// Creates a fixed-size slot configuration.
    pub fn fixed_size(size: Dp) -> Self {
        Self::FixedSize(size)
    }

    fn resolve_slots(&self, available: Px, spacing: Px) -> Vec<Px> {
        match self {
            Self::Fixed(count) => {
                let count = (*count).max(1);
                calculate_cells_cross_axis_size(available, count, spacing)
            }
            Self::Adaptive(min_size) => {
                let min_px = ensure_positive_px(Px::from(*min_size));
                let spacing = sanitize_spacing(spacing);
                let available_i32 = available.0.max(0);
                let min_i32 = min_px.0.max(1);
                let spacing_i32 = spacing.0.max(0);
                let count =
                    ((available_i32 + spacing_i32) / (min_i32 + spacing_i32)).max(1) as usize;
                calculate_cells_cross_axis_size(available, count, spacing)
            }
            Self::FixedSize(size) => {
                let cell_size = ensure_positive_px(Px::from(*size));
                let spacing = sanitize_spacing(spacing);
                let available_i32 = available.0.max(0);
                let cell_i32 = cell_size.0.max(1);
                let spacing_i32 = spacing.0.max(0);
                if cell_i32 + spacing_i32 < available_i32 + spacing_i32 {
                    let count =
                        ((available_i32 + spacing_i32) / (cell_i32 + spacing_i32)).max(1) as usize;
                    vec![cell_size; count]
                } else {
                    vec![available.max(Px::ZERO)]
                }
            }
        }
    }
}

impl Default for GridCells {
    fn default() -> Self {
        Self::fixed(2)
    }
}

/// Persistent state shared by lazy grid components.
pub struct LazyGridController {
    cache: LazyGridCache,
}

impl Default for LazyGridController {
    fn default() -> Self {
        Self::new()
    }
}

impl LazyGridController {
    /// Creates a new lazy grid state with default caches.
    pub fn new() -> Self {
        Self {
            cache: LazyGridCache::default(),
        }
    }
}

#[derive(Clone)]
struct LazyGridCollectedSlots(Arc<Mutex<Vec<LazySlot>>>);

fn hash_key<K>(key: K) -> u64
where
    K: Hash,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn collect_lazy_grid_slots(content: RenderSlot) -> Vec<LazySlot> {
    let collected = LazyGridCollectedSlots(Arc::new(Mutex::new(Vec::new())));
    provide_context(
        || collected.clone(),
        move || {
            content.render();
        },
    );
    collected.0.lock().clone()
}

fn push_lazy_grid_slot(slot: LazySlot) {
    let collector = use_context::<LazyGridCollectedSlots>()
        .expect("lazy grid item declarations must be used inside lazy grid content")
        .get();
    collector.0.lock().push(slot);
}

/// Adds a single lazily generated grid item declaration to the current lazy
/// grid content slot.
pub fn lazy_item<F>(builder: F)
where
    F: Fn() + Send + Sync + 'static,
{
    push_lazy_grid_slot(LazySlot::items(
        1,
        move |_| {
            builder();
        },
        None,
    ));
}

/// Adds a single lazily generated grid item declaration with a stable key.
pub fn lazy_item_with_key<K, F>(key: K, builder: F)
where
    K: Hash,
    F: Fn() + Send + Sync + 'static,
{
    let key_hash = hash_key(key);
    push_lazy_grid_slot(LazySlot::items(
        1,
        move |_| {
            builder();
        },
        Some(CallbackWith::new(move |_| key_hash)),
    ));
}

/// Adds a batch of lazily generated grid items to the current lazy grid
/// content slot.
pub fn lazy_items<F>(count: usize, builder: F)
where
    F: Fn(usize) + Send + Sync + 'static,
{
    push_lazy_grid_slot(LazySlot::items(count, builder, None));
}

/// Adds a batch of lazily generated grid items with a stable key provider.
pub fn lazy_items_with_key<K, KF, F>(count: usize, key_provider: KF, builder: F)
where
    K: Hash,
    KF: Fn(usize) -> K + Send + Sync + 'static,
    F: Fn(usize) + Send + Sync + 'static,
{
    let key_provider = CallbackWith::new(move |idx| hash_key(key_provider(idx)));
    push_lazy_grid_slot(LazySlot::items(count, builder, Some(key_provider)));
}

/// Adds lazily generated grid items from an iterator, providing both index and
/// element reference.
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
    push_lazy_grid_slot(LazySlot::items(
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

/// Adds lazily generated grid items from an iterator with a stable key
/// provider.
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

    push_lazy_grid_slot(LazySlot::items(
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct ZeroLayout;

impl LayoutPolicy for ZeroLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData::ZERO)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct VisibleGridLayoutItem {
    line_index: usize,
    slot_index: usize,
}

#[derive(Clone)]
struct LazyGridLayout {
    axis: LazyGridAxis,
    item_alignment: CrossAxisAlignment,
    estimated_line_main: Px,
    main_spacing: Px,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
    viewport_limit: Px,
    line_range: Range<usize>,
    slots: GridSlots,
    visible_items: Vec<VisibleGridLayoutItem>,
    controller: State<LazyGridController>,
    scroll_controller: State<ScrollableController>,
}

impl PartialEq for LazyGridLayout {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis
            && self.item_alignment == other.item_alignment
            && self.estimated_line_main == other.estimated_line_main
            && self.main_spacing == other.main_spacing
            && self.max_viewport_main == other.max_viewport_main
            && self.padding_main == other.padding_main
            && self.padding_cross == other.padding_cross
            && self.viewport_limit == other.viewport_limit
            && self.line_range == other.line_range
            && self.slots == other.slots
            && self.visible_items == other.visible_items
    }
}

impl LayoutPolicy for LazyGridLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        if input.children_ids().len() != self.visible_items.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy grid measured child count mismatch".into(),
            ));
        }

        let mut measured_items = Vec::with_capacity(self.visible_items.len());
        let line_count = self.line_range.end.saturating_sub(self.line_range.start);
        let mut line_max = vec![Px::ZERO; line_count];

        for (visible, child_id) in self.visible_items.iter().zip(input.children_ids().iter()) {
            let cell_cross = self
                .slots
                .sizes
                .get(visible.slot_index)
                .copied()
                .unwrap_or(Px::ZERO);
            let child_constraint = self.axis.child_constraint(cell_cross, self.item_alignment);
            let child_size = input.measure_child(*child_id, &child_constraint)?;
            let line_idx = visible.line_index - self.line_range.start;
            if let Some(line_value) = line_max.get_mut(line_idx) {
                *line_value = (*line_value).max(self.axis.main(&child_size));
            }
            measured_items.push(MeasuredGridItem {
                child_id: *child_id,
                line_index: visible.line_index,
                slot_index: visible.slot_index,
                size: child_size,
            });
        }

        let (placements, total_main) = self.controller.with_mut(|c| {
            for (offset, line_main) in line_max.iter().enumerate() {
                let line_index = self.line_range.start + offset;
                c.cache
                    .record_line_measurement(line_index, *line_main, self.estimated_line_main);
            }

            let mut placements = Vec::with_capacity(measured_items.len());
            for item in &measured_items {
                let line_offset = c.cache.offset_for_line(
                    item.line_index,
                    self.estimated_line_main,
                    self.main_spacing,
                );
                let cell_cross = self
                    .slots
                    .sizes
                    .get(item.slot_index)
                    .copied()
                    .unwrap_or(Px::ZERO);
                let cell_offset = compute_cell_offset(
                    cell_cross,
                    self.axis.cross(&item.size),
                    self.item_alignment,
                );
                let cross_offset = self.padding_cross
                    + self
                        .slots
                        .positions
                        .get(item.slot_index)
                        .copied()
                        .unwrap_or(Px::ZERO)
                    + cell_offset;
                let position = self
                    .axis
                    .position(line_offset + self.padding_main, cross_offset);
                placements.push(GridPlacement {
                    child_id: item.child_id,
                    position,
                });
            }

            let total_main = c
                .cache
                .total_main_size(self.estimated_line_main, self.main_spacing);
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

        for placement in placements {
            output.place_child(placement.child_id, placement.position);
        }

        Ok(self.axis.pack_size(reported_main, cross_with_padding))
    }
}

/// # lazy_vertical_grid
///
/// A vertically scrolling grid that only renders items visible in the
/// viewport.
///
/// ## Usage
///
/// Display tiled content such as photo galleries or dashboards.
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
/// - `columns` - grid cell definition for columns.
/// - `main_axis_spacing` - spacing between rows.
/// - `cross_axis_spacing` - spacing between columns.
/// - `cross_axis_alignment` - how columns are arranged when extra horizontal
///   space is available.
/// - `item_alignment` - alignment of items within each cell.
/// - `overscan` - number of extra rows instantiated before and after the
///   viewport.
/// - `estimated_item_size` - estimated main-axis size for each row.
/// - `content_padding` - symmetric padding applied around the grid content.
/// - `max_viewport_main` - optional maximum viewport length reported back to
///   parents.
/// - `controller` - optional external controller for scroll position and cache.
/// - `content` - optional slot builder for lazy grid content.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_grid::{GridCells, lazy_vertical_grid},
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_vertical_grid()
///         .columns(GridCells::fixed(2))
///         .overscan(0)
///         .content(move || {
///             tessera_components::lazy_grid::lazy_items(4, move |i| {
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
pub fn lazy_vertical_grid(
    modifier: Option<Modifier>,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: ScrollBarLayout,
    columns: GridCells,
    main_axis_spacing: Dp,
    cross_axis_spacing: Dp,
    cross_axis_alignment: MainAxisAlignment,
    item_alignment: CrossAxisAlignment,
    overscan: usize,
    estimated_item_size: Dp,
    content_padding: Dp,
    max_viewport_main: Option<Px>,
    controller: Option<State<LazyGridController>>,
    #[prop(skip_setter)] content: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    let slots = collect_vertical_grid_slots(content);
    let controller = controller.unwrap_or_else(|| remember(LazyGridController::new));
    lazy_vertical_grid_slots(LazyGridSlotsArgs {
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

fn collect_vertical_grid_slots(content: RenderSlot) -> Vec<LazySlot> {
    collect_lazy_grid_slots(content)
}

#[derive(Clone)]
struct LazyGridSlotsArgs {
    modifier: Modifier,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: ScrollBarLayout,
    grid_cells: GridCells,
    main_axis_spacing: Dp,
    cross_axis_spacing: Dp,
    cross_axis_alignment: MainAxisAlignment,
    item_alignment: CrossAxisAlignment,
    overscan: usize,
    estimated_item_size: Dp,
    content_padding: Dp,
    max_viewport_main: Option<Px>,
    controller: State<LazyGridController>,
    slots: Vec<LazySlot>,
}

fn lazy_vertical_grid_slots(args: LazyGridSlotsArgs) {
    let scroll_controller = remember(ScrollableController::default);
    let main_axis_spacing = sanitize_spacing(Px::from(args.main_axis_spacing));
    let cross_axis_spacing = sanitize_spacing(Px::from(args.cross_axis_spacing));
    let estimated_line_main = ensure_positive_px(Px::from(args.estimated_item_size));
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
        let mut builder = lazy_grid_view()
            .axis(LazyGridAxis::Vertical)
            .grid_cells(args.grid_cells.clone())
            .main_axis_spacing(main_axis_spacing)
            .cross_axis_spacing(cross_axis_spacing)
            .cross_axis_alignment(args.cross_axis_alignment)
            .item_alignment(args.item_alignment)
            .estimated_line_main(estimated_line_main)
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
/// # lazy_horizontal_grid
///
/// A horizontally scrolling grid that only renders items visible in the
/// viewport.
///
/// ## Usage
///
/// Display large, horizontally scrolling tile collections.
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
/// - `rows` - grid cell definition for rows.
/// - `main_axis_spacing` - spacing between columns.
/// - `cross_axis_spacing` - spacing between rows.
/// - `cross_axis_alignment` - how rows are arranged when extra vertical space
///   is available.
/// - `item_alignment` - alignment of items within each cell.
/// - `overscan` - number of extra columns instantiated before and after the
///   viewport.
/// - `estimated_item_size` - estimated main-axis size for each column.
/// - `content_padding` - symmetric padding applied around the grid content.
/// - `max_viewport_main` - optional maximum viewport length reported back to
///   parents.
/// - `controller` - optional external controller for scroll position and cache.
/// - `content` - optional slot builder for lazy grid content.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_grid::{GridCells, lazy_horizontal_grid},
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_horizontal_grid()
///         .rows(GridCells::fixed(2))
///         .overscan(0)
///         .content(move || {
///             tessera_components::lazy_grid::lazy_items(3, move |i| {
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
pub fn lazy_horizontal_grid(
    modifier: Option<Modifier>,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: ScrollBarLayout,
    rows: GridCells,
    main_axis_spacing: Dp,
    cross_axis_spacing: Dp,
    cross_axis_alignment: MainAxisAlignment,
    item_alignment: CrossAxisAlignment,
    overscan: usize,
    estimated_item_size: Dp,
    content_padding: Dp,
    max_viewport_main: Option<Px>,
    controller: Option<State<LazyGridController>>,
    #[prop(skip_setter)] content: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    let slots = collect_horizontal_grid_slots(content);
    let controller = controller.unwrap_or_else(|| remember(LazyGridController::new));
    lazy_horizontal_grid_slots(LazyGridSlotsArgs {
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

fn collect_horizontal_grid_slots(content: RenderSlot) -> Vec<LazySlot> {
    collect_lazy_grid_slots(content)
}

fn lazy_horizontal_grid_slots(args: LazyGridSlotsArgs) {
    let scroll_controller = remember(ScrollableController::default);
    let main_axis_spacing = sanitize_spacing(Px::from(args.main_axis_spacing));
    let cross_axis_spacing = sanitize_spacing(Px::from(args.cross_axis_spacing));
    let estimated_line_main = ensure_positive_px(Px::from(args.estimated_item_size));
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
        let mut builder = lazy_grid_view()
            .axis(LazyGridAxis::Horizontal)
            .grid_cells(args.grid_cells.clone())
            .main_axis_spacing(main_axis_spacing)
            .cross_axis_spacing(cross_axis_spacing)
            .cross_axis_alignment(args.cross_axis_alignment)
            .item_alignment(args.item_alignment)
            .estimated_line_main(estimated_line_main)
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
enum LazyGridAxis {
    #[default]
    Vertical,
    Horizontal,
}

#[tessera]
fn lazy_grid_view(
    axis: LazyGridAxis,
    grid_cells: GridCells,
    main_axis_spacing: Px,
    cross_axis_spacing: Px,
    cross_axis_alignment: MainAxisAlignment,
    item_alignment: CrossAxisAlignment,
    estimated_line_main: Px,
    overscan: usize,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
    controller: Option<State<LazyGridController>>,
    slots: Vec<LazySlot>,
    scroll_controller: Option<State<ScrollableController>>,
) {
    let controller = controller.expect("lazy_grid_view requires controller");
    let scroll_controller = scroll_controller.expect("lazy_grid_view requires scroll_controller");

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
    let slots_per_line = grid_slots.len();

    controller.with_mut(|c| c.cache.set_item_count(total_count, slots_per_line));
    let total_main = controller.with(|c| {
        c.cache
            .total_main_size(estimated_line_main, main_axis_spacing)
    });
    let total_main_with_padding = total_main + padding_main + padding_main;
    let cross_with_padding = grid_slots.cross_size + padding_cross + padding_cross;
    scroll_controller.with_mut(|c| {
        c.override_child_size(axis.pack_size(total_main_with_padding, cross_with_padding));
    });

    let scroll_offset = axis.scroll_offset(scroll_controller.with(|s| s.child_position()));
    let viewport_span = resolve_viewport_span(
        axis.visible_span(visible_size),
        estimated_line_main,
        main_axis_spacing,
    );
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);

    let visible_plan = controller.with(|c| {
        compute_visible_items(
            &plan,
            &c.cache,
            total_count,
            slots_per_line,
            scroll_offset,
            viewport_span,
            overscan,
            estimated_line_main,
            main_axis_spacing,
        )
    });

    if visible_plan.items.is_empty() {
        layout_primitive().layout_policy(ZeroLayout);
        return;
    }

    let focus_modifier = lazy_grid_focus_beyond_bounds_modifier(LazyGridFocusArgs {
        axis,
        controller,
        scroll_controller,
        estimated_line_main,
        main_axis_spacing,
        total_main,
        viewport_span,
        visible_line_range: visible_plan.line_range.clone(),
        total_lines: controller.with(|c| c.cache.line_count()),
    });

    let viewport_limit = viewport_span + padding_main + padding_main;
    let visible_layout_items = visible_plan
        .items
        .iter()
        .map(|item| VisibleGridLayoutItem {
            line_index: item.line_index,
            slot_index: item.slot_index,
        })
        .collect();

    let items = visible_plan.items;
    let line_range = visible_plan.line_range.clone();
    layout_primitive()
        .modifier(focus_modifier)
        .layout_policy(LazyGridLayout {
            axis,
            item_alignment,
            estimated_line_main,
            main_spacing: main_axis_spacing,
            max_viewport_main,
            padding_main,
            padding_cross,
            viewport_limit,
            line_range,
            slots: grid_slots.clone(),
            visible_items: visible_layout_items,
            controller,
            scroll_controller,
        })
        .child(move || {
            for child in &items {
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
        let per_line = estimated + spacing;
        px_mul(per_line, DEFAULT_VIEWPORT_LINES.max(1))
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_visible_items(
    plan: &LazySlotPlan,
    cache: &LazyGridCache,
    total_count: usize,
    slots_per_line: usize,
    scroll_offset: Px,
    viewport_span: Px,
    overscan: usize,
    estimated_line_main: Px,
    spacing: Px,
) -> VisibleGridPlan {
    let total_lines = cache.line_count();
    if total_count == 0 || slots_per_line == 0 || total_lines == 0 {
        return VisibleGridPlan::empty();
    }

    let mut start_line = cache.line_index_for_offset(scroll_offset, estimated_line_main, spacing);
    let end_target = scroll_offset + viewport_span;
    let mut end_line = cache.line_index_for_offset(end_target, estimated_line_main, spacing) + 1;

    start_line = start_line.saturating_sub(overscan);
    end_line = (end_line + overscan).min(total_lines);
    if start_line >= end_line {
        end_line = (start_line + 1).min(total_lines);
        start_line = start_line.saturating_sub(1);
    }

    let mut items = Vec::new();
    for line in start_line..end_line {
        let start_index = line * slots_per_line;
        let end_index = (start_index + slots_per_line).min(total_count);
        for index in start_index..end_index {
            if let Some((slot, local_index)) = plan.resolve(index) {
                let key_hash = if let Some(provider) = &slot.key_provider {
                    provider.call(local_index)
                } else {
                    let mut hasher = DefaultHasher::new();
                    index.hash(&mut hasher);
                    hasher.finish()
                };

                items.push(VisibleGridItem {
                    local_index,
                    line_index: line,
                    slot_index: index - start_index,
                    builder: slot.builder,
                    key_hash,
                });
            }
        }
    }

    VisibleGridPlan {
        items,
        line_range: start_line..end_line,
    }
}

fn clamp_reported_main(
    axis: LazyGridAxis,
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

impl LazyGridAxis {
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
    grid_cells: &GridCells,
    spacing: Px,
    alignment: MainAxisAlignment,
) -> GridSlots {
    let spacing = sanitize_spacing(spacing);
    let sizes = grid_cells.resolve_slots(available_cross, spacing);
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
struct GridPlacement {
    child_id: NodeId,
    position: PxPosition,
}

#[derive(Clone, PartialEq)]
struct MeasuredGridItem {
    child_id: NodeId,
    line_index: usize,
    slot_index: usize,
    size: ComputedData,
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
struct VisibleGridItem {
    local_index: usize,
    line_index: usize,
    slot_index: usize,
    builder: CallbackWith<usize, ()>,
    key_hash: u64,
}

#[derive(Clone, PartialEq)]
struct VisibleGridPlan {
    items: Vec<VisibleGridItem>,
    line_range: Range<usize>,
}

impl VisibleGridPlan {
    fn empty() -> Self {
        Self {
            items: Vec::new(),
            line_range: 0..0,
        }
    }
}

struct LazyGridFocusArgs {
    axis: LazyGridAxis,
    controller: State<LazyGridController>,
    scroll_controller: State<ScrollableController>,
    estimated_line_main: Px,
    main_axis_spacing: Px,
    total_main: Px,
    viewport_span: Px,
    visible_line_range: Range<usize>,
    total_lines: usize,
}

fn lazy_grid_focus_beyond_bounds_modifier(args: LazyGridFocusArgs) -> Modifier {
    let current_scroll_offset = args
        .axis
        .scroll_offset(args.scroll_controller.with(|s| s.child_position()));
    let max_scroll = (args.total_main - args.viewport_span).max(Px::ZERO);
    Modifier::new().focus_beyond_bounds_handler(CallbackWith::new(move |direction| {
        let Some(scroll_direction) = args.axis.focus_scroll_direction(direction) else {
            return false;
        };
        if args.total_lines == 0 || args.viewport_span <= Px::ZERO {
            return false;
        }

        let target_line = match scroll_direction {
            FocusScrollDirection::Forward => {
                if args.visible_line_range.end >= args.total_lines {
                    return false;
                }
                args.visible_line_range.end
            }
            FocusScrollDirection::Backward => {
                let Some(line) = args.visible_line_range.start.checked_sub(1) else {
                    return false;
                };
                line
            }
        };

        let (target_offset, target_main) = args.controller.with(|c| {
            (
                c.cache.offset_for_line(
                    target_line,
                    args.estimated_line_main,
                    args.main_axis_spacing,
                ),
                c.cache
                    .measured_line_main
                    .get(target_line)
                    .copied()
                    .flatten()
                    .unwrap_or(args.estimated_line_main),
            )
        });

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
struct LazyGridCache {
    total_items: usize,
    slots_per_line: usize,
    measured_line_main: Vec<Option<Px>>,
    fenwick: FenwickTree,
}

impl LazyGridCache {
    fn set_item_count(&mut self, count: usize, slots_per_line: usize) {
        if self.total_items == count && self.slots_per_line == slots_per_line {
            return;
        }
        self.total_items = count;
        self.slots_per_line = slots_per_line.max(1);
        let lines = line_count(count, self.slots_per_line);
        self.measured_line_main = vec![None; lines];
        self.fenwick.resize(lines);
    }

    fn line_count(&self) -> usize {
        self.measured_line_main.len()
    }

    fn record_line_measurement(&mut self, index: usize, actual: Px, estimated: Px) {
        if index >= self.measured_line_main.len() {
            return;
        }
        let previous = self.measured_line_main[index];
        if previous == Some(actual) {
            return;
        }

        let prev_delta = previous.map(|val| val - estimated).unwrap_or(Px::ZERO);
        let new_delta = actual - estimated;
        let delta_change = new_delta - prev_delta;
        self.measured_line_main[index] = Some(actual);
        self.fenwick.update(index, delta_change);
    }

    fn offset_for_line(&self, index: usize, estimated: Px, spacing: Px) -> Px {
        if self.measured_line_main.is_empty() {
            return Px::ZERO;
        }
        let clamped = index.min(self.measured_line_main.len());
        let spacing_contrib = px_mul(spacing, clamped);
        let estimated_contrib = px_mul(estimated, clamped);
        spacing_contrib + estimated_contrib + self.fenwick.prefix_sum(clamped)
    }

    fn total_main_size(&self, estimated: Px, spacing: Px) -> Px {
        let line_count = self.measured_line_main.len();
        if line_count == 0 {
            return Px::ZERO;
        }
        let spacing_contrib = px_mul(spacing, line_count.saturating_sub(1));
        let estimated_contrib = px_mul(estimated, line_count);
        spacing_contrib + estimated_contrib + self.fenwick.prefix_sum(line_count)
    }

    fn line_index_for_offset(&self, offset: Px, estimated: Px, spacing: Px) -> usize {
        if self.measured_line_main.is_empty() {
            return 0;
        }

        let mut low = 0usize;
        let mut high = self.measured_line_main.len();
        while low < high {
            let mid = (low + high) / 2;
            if self.offset_for_line(mid, estimated, spacing) <= offset {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low.saturating_sub(1)
            .min(self.measured_line_main.len().saturating_sub(1))
    }
}

#[derive(Default, Clone, PartialEq)]
struct FenwickTree {
    data: Vec<i64>,
}

impl FenwickTree {
    fn resize(&mut self, len: usize) {
        self.data.clear();
        self.data.resize(len + 1, 0);
    }

    fn update(&mut self, index: usize, delta: Px) {
        if self.data.is_empty() {
            return;
        }
        let mut i = index + 1;
        let delta_i64 = delta.0 as i64;
        while i < self.data.len() {
            self.data[i] = self.data[i].saturating_add(delta_i64);
            i += i & (!i + 1);
        }
    }

    fn prefix_sum(&self, count: usize) -> Px {
        if self.data.is_empty() {
            return Px::ZERO;
        }
        let mut idx = count;
        let mut sum = 0i64;
        while idx > 0 {
            sum = sum.saturating_add(self.data[idx]);
            idx &= idx - 1;
        }
        px_from_i64(sum)
    }
}

fn line_count(item_count: usize, slots_per_line: usize) -> usize {
    if item_count == 0 || slots_per_line == 0 {
        0
    } else {
        item_count.div_ceil(slots_per_line)
    }
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

#[cfg(test)]
mod tests {
    use tessera_ui::{
        AxisConstraint, ComputedData, LayoutInput, LayoutOutput, LayoutPolicy, MeasurementError,
        Modifier, NoopRenderPolicy, Px, PxPosition, layout::layout_primitive, remember, tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        modifier::{ModifierExt as _, SemanticsArgs},
        scrollable::{ScrollableController, scrollable},
    };

    use super::{GridCells, LazyGridAxis, LazyGridController, lazy_grid_view};

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedTestLayout {
        fn measure(
            &self,
            _input: &LayoutInput<'_>,
            _output: &mut LayoutOutput<'_>,
        ) -> Result<ComputedData, MeasurementError> {
            Ok(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            })
        }
    }

    #[tessera]
    fn fixed_test_box(tag: String, width: i32, height: i32) {
        layout_primitive()
            .layout_policy(FixedTestLayout { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag),
                ..Default::default()
            }));
    }

    #[tessera]
    fn lazy_vertical_grid_layout_case() {
        let controller = remember(LazyGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(60),
                height: Px::new(60),
            });
        });

        lazy_grid_view()
            .axis(LazyGridAxis::Vertical)
            .grid_cells(GridCells::fixed(2))
            .main_axis_spacing(Px::new(3))
            .cross_axis_spacing(Px::new(3))
            .cross_axis_alignment(MainAxisAlignment::Start)
            .item_alignment(CrossAxisAlignment::Start)
            .estimated_line_main(Px::new(10))
            .overscan(0)
            .padding_main(Px::new(4))
            .padding_cross(Px::new(4))
            .slots(vec![super::LazySlot::items(
                4,
                |index| match index {
                    0 => {
                        fixed_test_box()
                            .tag("lazy_grid_v_first".to_string())
                            .width(10)
                            .height(10);
                    }
                    1 => {
                        fixed_test_box()
                            .tag("lazy_grid_v_second".to_string())
                            .width(12)
                            .height(8);
                    }
                    2 => {
                        fixed_test_box()
                            .tag("lazy_grid_v_third".to_string())
                            .width(11)
                            .height(12);
                    }
                    _ => {
                        fixed_test_box()
                            .tag("lazy_grid_v_fourth".to_string())
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
    fn lazy_horizontal_grid_layout_case() {
        let controller = remember(LazyGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(70),
                height: Px::new(40),
            });
        });

        lazy_grid_view()
            .axis(LazyGridAxis::Horizontal)
            .grid_cells(GridCells::fixed(2))
            .main_axis_spacing(Px::new(3))
            .cross_axis_spacing(Px::new(3))
            .cross_axis_alignment(MainAxisAlignment::Start)
            .item_alignment(CrossAxisAlignment::Start)
            .estimated_line_main(Px::new(10))
            .overscan(0)
            .padding_main(Px::new(4))
            .padding_cross(Px::new(4))
            .slots(vec![super::LazySlot::items(
                4,
                |index| match index {
                    0 => {
                        fixed_test_box()
                            .tag("lazy_grid_h_first".to_string())
                            .width(20)
                            .height(10);
                    }
                    1 => {
                        fixed_test_box()
                            .tag("lazy_grid_h_second".to_string())
                            .width(15)
                            .height(12);
                    }
                    2 => {
                        fixed_test_box()
                            .tag("lazy_grid_h_third".to_string())
                            .width(18)
                            .height(8);
                    }
                    _ => {
                        fixed_test_box()
                            .tag("lazy_grid_h_fourth".to_string())
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
    fn lazy_vertical_grid_scrolled_layout_case() {
        let controller = remember(LazyGridController::new);
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
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(60))),
            ))
            .vertical(true)
            .horizontal(false)
            .controller(scroll_controller)
            .child(move || {
                lazy_grid_view()
                    .axis(LazyGridAxis::Vertical)
                    .grid_cells(GridCells::fixed(2))
                    .main_axis_spacing(Px::new(3))
                    .cross_axis_spacing(Px::new(3))
                    .cross_axis_alignment(MainAxisAlignment::Start)
                    .item_alignment(CrossAxisAlignment::Start)
                    .estimated_line_main(Px::new(10))
                    .overscan(0)
                    .padding_main(Px::new(4))
                    .padding_cross(Px::new(4))
                    .slots(vec![super::LazySlot::items(
                        4,
                        |index| match index {
                            0 => {
                                fixed_test_box()
                                    .tag("lazy_grid_scroll_first".to_string())
                                    .width(10)
                                    .height(10);
                            }
                            1 => {
                                fixed_test_box()
                                    .tag("lazy_grid_scroll_second".to_string())
                                    .width(12)
                                    .height(8);
                            }
                            2 => {
                                fixed_test_box()
                                    .tag("lazy_grid_scroll_third".to_string())
                                    .width(11)
                                    .height(12);
                            }
                            _ => {
                                fixed_test_box()
                                    .tag("lazy_grid_scroll_fourth".to_string())
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
    fn lazy_vertical_grid_positions_items_with_padding_and_spacing() {
        tessera_ui::assert_layout! {
            viewport: (80, 80),
            content: {
                lazy_vertical_grid_layout_case();
            },
            expect: {
                node("lazy_grid_v_first").position(4, 4).size(10, 10);
                node("lazy_grid_v_second").position(32, 4).size(12, 8);
                node("lazy_grid_v_third").position(4, 17).size(11, 12);
                node("lazy_grid_v_fourth").position(32, 17).size(9, 9);
            }
        }
    }

    #[test]
    fn lazy_horizontal_grid_positions_items_with_padding_and_spacing() {
        tessera_ui::assert_layout! {
            viewport: (90, 50),
            content: {
                lazy_horizontal_grid_layout_case();
            },
            expect: {
                node("lazy_grid_h_first").position(4, 4).size(20, 10);
                node("lazy_grid_h_second").position(4, 22).size(15, 12);
                node("lazy_grid_h_third").position(27, 4).size(18, 8);
                node("lazy_grid_h_fourth").position(27, 22).size(16, 9);
            }
        }
    }

    #[test]
    fn lazy_vertical_grid_scroll_offset_repositions_visible_items() {
        tessera_ui::assert_layout! {
            viewport: (80, 80),
            content: {
                lazy_vertical_grid_scrolled_layout_case();
            },
            expect: {
                node("lazy_grid_scroll_first").position(4, -1).size(10, 10);
                node("lazy_grid_scroll_second").position(32, -1).size(12, 8);
                node("lazy_grid_scroll_third").position(4, 12).size(11, 12);
                node("lazy_grid_scroll_fourth").position(32, 12).size(9, 9);
            }
        }
    }
}
