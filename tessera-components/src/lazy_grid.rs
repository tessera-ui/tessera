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

use derive_setters::Setters;
use tessera_ui::{
    CallbackWith, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, NodeId,
    ParentConstraint, Px, PxPosition, Slot, State, key,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    remember, tessera,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    scrollable::{ScrollableArgs, ScrollableController, scrollable},
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

/// Arguments shared between lazy vertical grids.
#[derive(PartialEq, Clone, Setters)]
pub struct LazyVerticalGridArgs {
    /// Scroll container arguments. Vertical scrolling is enforced.
    pub scrollable: ScrollableArgs,
    /// Grid cell definition for columns.
    pub columns: GridCells,
    /// Spacing between rows.
    pub main_axis_spacing: Dp,
    /// Spacing between columns.
    pub cross_axis_spacing: Dp,
    /// How columns are arranged when extra horizontal space is available.
    pub cross_axis_alignment: MainAxisAlignment,
    /// Alignment of items within each cell along the cross axis.
    pub item_alignment: CrossAxisAlignment,
    /// Number of extra rows instantiated before/after the viewport.
    pub overscan: usize,
    /// Estimated main-axis size for each row.
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the grid content.
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents.
    pub max_viewport_main: Option<Px>,
    /// Optional external controller for scroll position and cache.
    #[setters(skip)]
    pub controller: Option<State<LazyGridController>>,
    /// Optional slot builder for lazy grid content.
    #[setters(skip)]
    pub content: Option<LazyGridContentSlot>,
}

impl Default for LazyVerticalGridArgs {
    fn default() -> Self {
        Self {
            scrollable: ScrollableArgs::default(),
            columns: GridCells::fixed(2),
            main_axis_spacing: Dp(0.0),
            cross_axis_spacing: Dp(0.0),
            cross_axis_alignment: MainAxisAlignment::Start,
            item_alignment: CrossAxisAlignment::Stretch,
            overscan: 2,
            estimated_item_size: Dp(48.0),
            content_padding: Dp(0.0),
            max_viewport_main: Some(Px(8192)),
            controller: None,
            content: None,
        }
    }
}

impl LazyVerticalGridArgs {
    /// Sets an external lazy grid controller.
    pub fn controller(mut self, controller: State<LazyGridController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Sets the lazy grid content builder.
    pub fn content<F>(mut self, content: F) -> Self
    where
        F: for<'a> Fn(&mut LazyGridScope<'a>) + Send + Sync + 'static,
    {
        self.content = Some(LazyGridContentSlot::new(content));
        self
    }

    /// Sets the lazy grid content builder using a shared slot.
    pub fn content_shared(mut self, content: impl Into<LazyGridContentSlot>) -> Self {
        self.content = Some(content.into());
        self
    }
}

/// Arguments shared between lazy horizontal grids.
#[derive(PartialEq, Clone, Setters)]
pub struct LazyHorizontalGridArgs {
    /// Scroll container arguments. Horizontal scrolling is enforced.
    pub scrollable: ScrollableArgs,
    /// Grid cell definition for rows.
    pub rows: GridCells,
    /// Spacing between columns.
    pub main_axis_spacing: Dp,
    /// Spacing between rows.
    pub cross_axis_spacing: Dp,
    /// How rows are arranged when extra vertical space is available.
    pub cross_axis_alignment: MainAxisAlignment,
    /// Alignment of items within each cell along the cross axis.
    pub item_alignment: CrossAxisAlignment,
    /// Number of extra columns instantiated before/after the viewport.
    pub overscan: usize,
    /// Estimated main-axis size for each column.
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the grid content.
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents.
    pub max_viewport_main: Option<Px>,
    /// Optional external controller for scroll position and cache.
    #[setters(skip)]
    pub controller: Option<State<LazyGridController>>,
    /// Optional slot builder for lazy grid content.
    #[setters(skip)]
    pub content: Option<LazyGridContentSlot>,
}

impl Default for LazyHorizontalGridArgs {
    fn default() -> Self {
        Self {
            scrollable: ScrollableArgs::default(),
            rows: GridCells::fixed(2),
            main_axis_spacing: Dp(0.0),
            cross_axis_spacing: Dp(0.0),
            cross_axis_alignment: MainAxisAlignment::Start,
            item_alignment: CrossAxisAlignment::Stretch,
            overscan: 2,
            estimated_item_size: Dp(48.0),
            content_padding: Dp(0.0),
            max_viewport_main: Some(Px(8192)),
            controller: None,
            content: None,
        }
    }
}

impl LazyHorizontalGridArgs {
    /// Sets an external lazy grid controller.
    pub fn controller(mut self, controller: State<LazyGridController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Sets the lazy grid content builder.
    pub fn content<F>(mut self, content: F) -> Self
    where
        F: for<'a> Fn(&mut LazyGridScope<'a>) + Send + Sync + 'static,
    {
        self.content = Some(LazyGridContentSlot::new(content));
        self
    }

    /// Sets the lazy grid content builder using a shared slot.
    pub fn content_shared(mut self, content: impl Into<LazyGridContentSlot>) -> Self {
        self.content = Some(content.into());
        self
    }
}

/// Scope used to add lazily generated children to a lazy grid.
pub struct LazyGridScope<'a> {
    slots: &'a mut Vec<LazySlot>,
}

type LazyGridRenderFn = dyn for<'a> Fn(&mut LazyGridScope<'a>) + Send + Sync;

/// Shared slot builder for lazy grid content.
#[derive(Clone, PartialEq)]
pub struct LazyGridContentSlot(Slot<LazyGridRenderFn>);

impl LazyGridContentSlot {
    /// Creates a new shared lazy grid content slot.
    pub fn new<F>(content: F) -> Self
    where
        F: for<'a> Fn(&mut LazyGridScope<'a>) + Send + Sync + 'static,
    {
        Self(Slot::from_shared(Arc::new(content)))
    }

    fn render(&self, scope: &mut LazyGridScope<'_>) {
        let render = self.0.shared();
        render(scope);
    }
}

impl<F> From<F> for LazyGridContentSlot
where
    F: for<'a> Fn(&mut LazyGridScope<'a>) + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self::new(value)
    }
}

impl<'a> LazyGridScope<'a> {
    /// Adds a batch of lazily generated items.
    pub fn items<F>(&mut self, count: usize, builder: F)
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.slots.push(LazySlot::items(count, builder, None));
    }

    /// Adds a batch of lazily generated items with a key provider.
    pub fn items_with_key<K, KF, F>(&mut self, count: usize, key_provider: KF, builder: F)
    where
        K: Hash,
        KF: Fn(usize) -> K + Send + Sync + 'static,
        F: Fn(usize) + Send + Sync + 'static,
    {
        let key_provider = CallbackWith::new(move |idx| {
            let key = key_provider(idx);
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            hasher.finish()
        });
        self.slots
            .push(LazySlot::items(count, builder, Some(key_provider)));
    }

    /// Add a single lazily generated item.
    pub fn item<F>(&mut self, builder: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.items(1, move |_| {
            builder();
        });
    }

    /// Adds lazily generated items from an iterator, providing both index and
    /// element reference.
    ///
    /// The iterator is eagerly collected so it can be accessed on demand while
    /// virtualizing.
    pub fn items_from_iter<I, T, F>(&mut self, iter: I, builder: F)
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
        self.slots.push(LazySlot::items(
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

    /// Adds lazily generated items from an iterator with a key provider.
    pub fn items_from_iter_with_key<I, T, K, KF, F>(
        &mut self,
        iter: I,
        key_provider: KF,
        builder: F,
    ) where
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
                if let Some(item) = items.get(idx) {
                    let key = key_provider(idx, item);
                    let mut hasher = DefaultHasher::new();
                    key.hash(&mut hasher);
                    hasher.finish()
                } else {
                    0
                }
            }
        };

        self.slots.push(LazySlot::items(
            count,
            slot_builder,
            Some(CallbackWith::new(slot_key_provider)),
        ));
    }

    /// Convenience helper for iterators when only the element is needed.
    pub fn items_from_iter_values<I, T, F>(&mut self, iter: I, builder: F)
    where
        I: IntoIterator<Item = T>,
        T: Send + Sync + 'static,
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.items_from_iter(iter, move |_, item| builder(item));
    }
}
/// Scope alias for vertical lazy grids.
pub type LazyVerticalGridScope<'a> = LazyGridScope<'a>;
/// Scope alias for horizontal lazy grids.
pub type LazyHorizontalGridScope<'a> = LazyGridScope<'a>;

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
/// - `args` - configures the grid's layout and scrolling behavior; see
///   [`LazyVerticalGridArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_grid::{GridCells, LazyVerticalGridArgs, lazy_vertical_grid},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_vertical_grid(
///         &LazyVerticalGridArgs::default()
///             .columns(GridCells::fixed(2))
///             .overscan(0)
///             .content(move |scope| {
///                 scope.items(4, move |i| {
///                     rendered.with_mut(|count| *count += 1);
///                     text(&TextArgs::default().text(format!("Tile {i}")));
///                 });
///             }),
///     );
///     assert_eq!(rendered.get(), 4);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_vertical_grid(args: &LazyVerticalGridArgs) {
    let args = args.clone();
    let content = args
        .content
        .clone()
        .unwrap_or_else(|| LazyGridContentSlot::new(|_| {}));
    let slots = collect_vertical_grid_slots(content);
    let controller = args
        .controller
        .unwrap_or_else(|| remember(LazyGridController::new));
    lazy_vertical_grid_slots(args, controller, slots);
}

fn collect_vertical_grid_slots(content: LazyGridContentSlot) -> Vec<LazySlot> {
    let mut slots = Vec::new();
    {
        let mut scope = LazyVerticalGridScope { slots: &mut slots };
        content.render(&mut scope);
    }
    slots
}

fn lazy_vertical_grid_slots(
    args: LazyVerticalGridArgs,
    controller: State<LazyGridController>,
    slots: Vec<LazySlot>,
) {
    let mut scrollable_args = args.scrollable.clone();
    scrollable_args.vertical = true;
    scrollable_args.horizontal = false;

    let scroll_controller = remember(ScrollableController::default);
    let view_args = LazyGridViewArgs {
        axis: LazyGridAxis::Vertical,
        grid_cells: args.columns,
        main_axis_spacing: sanitize_spacing(Px::from(args.main_axis_spacing)),
        cross_axis_spacing: sanitize_spacing(Px::from(args.cross_axis_spacing)),
        cross_axis_alignment: args.cross_axis_alignment,
        item_alignment: args.item_alignment,
        estimated_line_main: ensure_positive_px(Px::from(args.estimated_item_size)),
        overscan: args.overscan,
        max_viewport_main: args.max_viewport_main,
        padding_main: sanitize_spacing(Px::from(args.content_padding)),
        padding_cross: sanitize_spacing(Px::from(args.content_padding)),
        controller,
        slots,
        scroll_controller,
    };
    let scrollable_args = scrollable_args
        .controller(scroll_controller)
        .child(move || {
            lazy_grid_view(&view_args);
        });
    scrollable(&scrollable_args);
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
/// - `args` - configures the grid's layout and scrolling behavior; see
///   [`LazyHorizontalGridArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_grid::{GridCells, LazyHorizontalGridArgs, lazy_horizontal_grid},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_horizontal_grid(
///         &LazyHorizontalGridArgs::default()
///             .rows(GridCells::fixed(2))
///             .overscan(0)
///             .content(move |scope| {
///                 scope.items(3, move |i| {
///                     rendered.with_mut(|count| *count += 1);
///                     text(&TextArgs::default().text(format!("Tile {i}")));
///                 });
///             }),
///     );
///     assert_eq!(rendered.get(), 3);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_horizontal_grid(args: &LazyHorizontalGridArgs) {
    let args = args.clone();
    let content = args
        .content
        .clone()
        .unwrap_or_else(|| LazyGridContentSlot::new(|_| {}));
    let slots = collect_horizontal_grid_slots(content);
    let controller = args
        .controller
        .unwrap_or_else(|| remember(LazyGridController::new));
    lazy_horizontal_grid_slots(args, controller, slots);
}

fn collect_horizontal_grid_slots(content: LazyGridContentSlot) -> Vec<LazySlot> {
    let mut slots = Vec::new();
    {
        let mut scope = LazyHorizontalGridScope { slots: &mut slots };
        content.render(&mut scope);
    }
    slots
}

fn lazy_horizontal_grid_slots(
    args: LazyHorizontalGridArgs,
    controller: State<LazyGridController>,
    slots: Vec<LazySlot>,
) {
    let mut scrollable_args = args.scrollable.clone();
    scrollable_args.vertical = false;
    scrollable_args.horizontal = true;

    let scroll_controller = remember(ScrollableController::default);
    let view_args = LazyGridViewArgs {
        axis: LazyGridAxis::Horizontal,
        grid_cells: args.rows,
        main_axis_spacing: sanitize_spacing(Px::from(args.main_axis_spacing)),
        cross_axis_spacing: sanitize_spacing(Px::from(args.cross_axis_spacing)),
        cross_axis_alignment: args.cross_axis_alignment,
        item_alignment: args.item_alignment,
        estimated_line_main: ensure_positive_px(Px::from(args.estimated_item_size)),
        overscan: args.overscan,
        max_viewport_main: args.max_viewport_main,
        padding_main: sanitize_spacing(Px::from(args.content_padding)),
        padding_cross: sanitize_spacing(Px::from(args.content_padding)),
        controller,
        slots,
        scroll_controller,
    };
    let scrollable_args = scrollable_args
        .controller(scroll_controller)
        .child(move || {
            lazy_grid_view(&view_args);
        });
    scrollable(&scrollable_args);
}
#[derive(PartialEq, Clone)]
struct LazyGridViewArgs {
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
    controller: State<LazyGridController>,
    slots: Vec<LazySlot>,
    scroll_controller: State<ScrollableController>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct ZeroLayout;

impl LayoutSpec for ZeroLayout {
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

impl LayoutSpec for LazyGridLayout {
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

#[tessera]
fn lazy_grid_view(args: &LazyGridViewArgs) {
    let args = args.clone();
    let plan = LazySlotPlan::new(args.slots.clone());
    let total_count = plan.total_count();

    let visible_size = args.scroll_controller.with(|s| s.visible_size());
    let available_cross = args.axis.visible_cross(visible_size);
    let available_cross = (available_cross - args.padding_cross * 2).max(Px::ZERO);
    let grid_slots = resolve_grid_slots(
        available_cross,
        &args.grid_cells,
        args.cross_axis_spacing,
        args.cross_axis_alignment,
    );
    let slots_per_line = grid_slots.len();

    args.controller
        .with_mut(|c| c.cache.set_item_count(total_count, slots_per_line));
    let total_main = args.controller.with(|c| {
        c.cache
            .total_main_size(args.estimated_line_main, args.main_axis_spacing)
    });
    let total_main_with_padding = total_main + args.padding_main + args.padding_main;
    let cross_with_padding = grid_slots.cross_size + args.padding_cross + args.padding_cross;
    args.scroll_controller.with_mut(|c| {
        c.override_child_size(
            args.axis
                .pack_size(total_main_with_padding, cross_with_padding),
        );
    });

    let scroll_offset = args
        .axis
        .scroll_offset(args.scroll_controller.with(|s| s.child_position()));
    let padding_main = args.padding_main;
    let viewport_span = resolve_viewport_span(
        args.axis.visible_span(visible_size),
        args.estimated_line_main,
        args.main_axis_spacing,
    );
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);

    let visible_plan = args.controller.with(|c| {
        compute_visible_items(
            &plan,
            &c.cache,
            total_count,
            slots_per_line,
            scroll_offset,
            viewport_span,
            args.overscan,
            args.estimated_line_main,
            args.main_axis_spacing,
        )
    });

    if visible_plan.items.is_empty() {
        layout(ZeroLayout);
        return;
    }

    let viewport_limit = viewport_span + padding_main + padding_main;
    let visible_layout_items = visible_plan
        .items
        .iter()
        .map(|item| VisibleGridLayoutItem {
            line_index: item.line_index,
            slot_index: item.slot_index,
        })
        .collect();

    layout(LazyGridLayout {
        axis: args.axis,
        item_alignment: args.item_alignment,
        estimated_line_main: args.estimated_line_main,
        main_spacing: args.main_axis_spacing,
        max_viewport_main: args.max_viewport_main,
        padding_main,
        padding_cross: args.padding_cross,
        viewport_limit,
        line_range: visible_plan.line_range.clone(),
        slots: grid_slots.clone(),
        visible_items: visible_layout_items,
        controller: args.controller,
        scroll_controller: args.scroll_controller,
    });

    for child in visible_plan.items {
        key(child.key_hash, || {
            child.builder.call(child.local_index);
        });
    }
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
                    builder: slot.builder.clone(),
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum LazyGridAxis {
    Vertical,
    Horizontal,
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

    fn scroll_offset(&self, position: PxPosition) -> Px {
        match self {
            Self::Vertical => (-position.y).max(Px::ZERO),
            Self::Horizontal => (-position.x).max(Px::ZERO),
        }
    }

    fn child_constraint(&self, cross: Px, alignment: CrossAxisAlignment) -> Constraint {
        let cross = cross.max(Px::ZERO);
        let cross_dim = match alignment {
            CrossAxisAlignment::Stretch => DimensionValue::Fixed(cross),
            _ => DimensionValue::Wrap {
                min: None,
                max: Some(cross),
            },
        };

        match self {
            Self::Vertical => Constraint::new(
                cross_dim,
                DimensionValue::Wrap {
                    min: None,
                    max: None,
                },
            ),
            Self::Horizontal => Constraint::new(
                DimensionValue::Wrap {
                    min: None,
                    max: None,
                },
                cross_dim,
            ),
        }
    }

    fn constraint_max(&self, constraint: ParentConstraint<'_>) -> Option<Px> {
        match self {
            Self::Vertical => constraint.height().get_max(),
            Self::Horizontal => constraint.width().get_max(),
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
