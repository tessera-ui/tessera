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

use derive_setters::Setters;
use tessera_ui::{
    ComputedData, Constraint, DimensionValue, Dp, MeasurementError, NodeId, ParentConstraint, Px,
    PxPosition, State, key,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    remember, tessera,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    lazy_grid::GridCells,
    scrollable::{ScrollableArgs, ScrollableController, scrollable_with_controller},
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

/// Arguments shared between lazy vertical staggered grids.
#[derive(Clone, Setters)]
pub struct LazyVerticalStaggeredGridArgs {
    /// Scroll container arguments. Vertical scrolling is enforced.
    pub scrollable: ScrollableArgs,
    /// Lane definition for columns.
    pub columns: StaggeredGridCells,
    /// Spacing between items within a lane.
    pub main_axis_spacing: Dp,
    /// Spacing between lanes.
    pub cross_axis_spacing: Dp,
    /// How lanes are arranged when extra cross-axis space is available.
    pub cross_axis_alignment: MainAxisAlignment,
    /// Alignment of items within each lane.
    pub item_alignment: CrossAxisAlignment,
    /// Number of extra items instantiated before/after the viewport.
    pub overscan: usize,
    /// Estimated main-axis size for each item.
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the grid content.
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents.
    pub max_viewport_main: Option<Px>,
}

impl Default for LazyVerticalStaggeredGridArgs {
    fn default() -> Self {
        Self {
            scrollable: ScrollableArgs::default(),
            columns: StaggeredGridCells::fixed(2),
            main_axis_spacing: Dp(0.0),
            cross_axis_spacing: Dp(0.0),
            cross_axis_alignment: MainAxisAlignment::Start,
            item_alignment: CrossAxisAlignment::Stretch,
            overscan: 2,
            estimated_item_size: Dp(72.0),
            content_padding: Dp(0.0),
            max_viewport_main: Some(Px(8192)),
        }
    }
}

/// Arguments shared between lazy horizontal staggered grids.
#[derive(Clone, Setters)]
pub struct LazyHorizontalStaggeredGridArgs {
    /// Scroll container arguments. Horizontal scrolling is enforced.
    pub scrollable: ScrollableArgs,
    /// Lane definition for rows.
    pub rows: StaggeredGridCells,
    /// Spacing between items within a lane.
    pub main_axis_spacing: Dp,
    /// Spacing between lanes.
    pub cross_axis_spacing: Dp,
    /// How lanes are arranged when extra cross-axis space is available.
    pub cross_axis_alignment: MainAxisAlignment,
    /// Alignment of items within each lane.
    pub item_alignment: CrossAxisAlignment,
    /// Number of extra items instantiated before/after the viewport.
    pub overscan: usize,
    /// Estimated main-axis size for each item.
    pub estimated_item_size: Dp,
    /// Symmetric padding applied around the grid content.
    pub content_padding: Dp,
    /// Maximum viewport length reported back to parents.
    pub max_viewport_main: Option<Px>,
}

impl Default for LazyHorizontalStaggeredGridArgs {
    fn default() -> Self {
        Self {
            scrollable: ScrollableArgs::default(),
            rows: StaggeredGridCells::fixed(2),
            main_axis_spacing: Dp(0.0),
            cross_axis_spacing: Dp(0.0),
            cross_axis_alignment: MainAxisAlignment::Start,
            item_alignment: CrossAxisAlignment::Stretch,
            overscan: 2,
            estimated_item_size: Dp(72.0),
            content_padding: Dp(0.0),
            max_viewport_main: Some(Px(8192)),
        }
    }
}

/// Scope used to add lazily generated children to a staggered grid.
pub struct LazyStaggeredGridScope<'a> {
    slots: &'a mut Vec<LazySlot>,
}

impl<'a> LazyStaggeredGridScope<'a> {
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
        let key_provider = Arc::new(move |idx| {
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
            Some(Arc::new(slot_key_provider)),
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
/// Scope alias for vertical lazy staggered grids.
pub type LazyVerticalStaggeredGridScope<'a> = LazyStaggeredGridScope<'a>;
/// Scope alias for horizontal lazy staggered grids.
pub type LazyHorizontalStaggeredGridScope<'a> = LazyStaggeredGridScope<'a>;

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
/// - `args` - configures layout, spacing, and scrolling; see
///   [`LazyVerticalStaggeredGridArgs`].
/// - `configure` - a closure that receives a [`LazyVerticalStaggeredGridScope`]
///   for adding items to the grid.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::{
///     lazy_staggered_grid::{
///         LazyVerticalStaggeredGridArgs, StaggeredGridCells, lazy_vertical_staggered_grid,
///     },
///     text::{TextArgs, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_vertical_staggered_grid(
///         LazyVerticalStaggeredGridArgs::default()
///             .columns(StaggeredGridCells::fixed(2))
///             .overscan(0),
///         |scope| {
///             scope.items(4, move |i| {
///                 rendered.with_mut(|count| *count += 1);
///                 text(TextArgs::default().text(format!("Tile {i}")));
///             });
///         },
///     );
///     assert_eq!(rendered.get(), 4);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_vertical_staggered_grid<F>(args: LazyVerticalStaggeredGridArgs, configure: F)
where
    F: FnOnce(&mut LazyVerticalStaggeredGridScope),
{
    let controller = remember(LazyStaggeredGridController::new);
    lazy_vertical_staggered_grid_with_controller(args, controller, configure);
}

/// # lazy_vertical_staggered_grid_with_controller
///
/// Controlled vertical staggered grid variant for persistent scroll state.
///
/// ## Usage
///
/// Use when you want to preserve scroll state across remounts.
///
/// ## Parameters
///
/// - `args` - configures layout, spacing, and scrolling; see
///   [`LazyVerticalStaggeredGridArgs`].
/// - `controller` - a [`LazyStaggeredGridController`] that holds scroll offsets
///   and size cache.
/// - `configure` - a closure that receives a [`LazyVerticalStaggeredGridScope`]
///   for adding items to the grid.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::{
///     lazy_staggered_grid::{
///         LazyStaggeredGridController, LazyVerticalStaggeredGridArgs, StaggeredGridCells,
///         lazy_vertical_staggered_grid_with_controller,
///     },
///     text::{TextArgs, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(LazyStaggeredGridController::new);
///     let rendered = remember(|| 0usize);
///     lazy_vertical_staggered_grid_with_controller(
///         LazyVerticalStaggeredGridArgs::default()
///             .columns(StaggeredGridCells::fixed(2))
///             .overscan(0),
///         controller,
///         |scope| {
///             scope.items(2, move |i| {
///                 rendered.with_mut(|count| *count += 1);
///                 text(TextArgs::default().text(format!("Cell {i}")));
///             });
///         },
///     );
///     assert_eq!(rendered.get(), 2);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_vertical_staggered_grid_with_controller<F>(
    args: LazyVerticalStaggeredGridArgs,
    controller: State<LazyStaggeredGridController>,
    configure: F,
) where
    F: FnOnce(&mut LazyVerticalStaggeredGridScope),
{
    let mut slots = Vec::new();
    {
        let mut scope = LazyVerticalStaggeredGridScope { slots: &mut slots };
        configure(&mut scope);
    }

    let mut scrollable_args = args.scrollable.clone();
    scrollable_args.vertical = true;
    scrollable_args.horizontal = false;

    let view_args = LazyStaggeredGridViewArgs {
        axis: StaggeredGridAxis::Vertical,
        grid_cells: args.columns,
        main_axis_spacing: sanitize_spacing(Px::from(args.main_axis_spacing)),
        cross_axis_spacing: sanitize_spacing(Px::from(args.cross_axis_spacing)),
        cross_axis_alignment: args.cross_axis_alignment,
        item_alignment: args.item_alignment,
        estimated_item_main: ensure_positive_px(Px::from(args.estimated_item_size)),
        overscan: args.overscan,
        max_viewport_main: args.max_viewport_main,
        padding_main: sanitize_spacing(Px::from(args.content_padding)),
        padding_cross: sanitize_spacing(Px::from(args.content_padding)),
    };

    let scroll_controller = remember(ScrollableController::default);
    scrollable_with_controller(scrollable_args, scroll_controller, move || {
        lazy_staggered_grid_view(view_args, controller, slots.clone(), scroll_controller);
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
/// - `args` - configures layout, spacing, and scrolling; see
///   [`LazyHorizontalStaggeredGridArgs`].
/// - `configure` - a closure that receives a
///   [`LazyHorizontalStaggeredGridScope`] for adding items to the grid.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::{
///     lazy_staggered_grid::{
///         LazyHorizontalStaggeredGridArgs, StaggeredGridCells, lazy_horizontal_staggered_grid,
///     },
///     text::{TextArgs, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let rendered = remember(|| 0usize);
///     lazy_horizontal_staggered_grid(
///         LazyHorizontalStaggeredGridArgs::default()
///             .rows(StaggeredGridCells::fixed(2))
///             .overscan(0),
///         |scope| {
///             scope.items(3, move |i| {
///                 rendered.with_mut(|count| *count += 1);
///                 text(TextArgs::default().text(format!("Tile {i}")));
///             });
///         },
///     );
///     assert_eq!(rendered.get(), 3);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_horizontal_staggered_grid<F>(args: LazyHorizontalStaggeredGridArgs, configure: F)
where
    F: FnOnce(&mut LazyHorizontalStaggeredGridScope),
{
    let controller = remember(LazyStaggeredGridController::new);
    lazy_horizontal_staggered_grid_with_controller(args, controller, configure);
}

/// # lazy_horizontal_staggered_grid_with_controller
///
/// Controlled horizontal staggered grid variant for synchronized scroll state.
///
/// ## Usage
///
/// Use when you need to sync scroll position with other UI.
///
/// ## Parameters
///
/// - `args` - configures layout, spacing, and scrolling; see
///   [`LazyHorizontalStaggeredGridArgs`].
/// - `controller` - a [`LazyStaggeredGridController`] that holds scroll offsets
///   and size cache.
/// - `configure` - a closure that receives a
///   [`LazyHorizontalStaggeredGridScope`] for adding items to the grid.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::{
///     lazy_staggered_grid::{
///         LazyHorizontalStaggeredGridArgs, LazyStaggeredGridController, StaggeredGridCells,
///         lazy_horizontal_staggered_grid_with_controller,
///     },
///     text::{TextArgs, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(LazyStaggeredGridController::new);
///     let rendered = remember(|| 0usize);
///     lazy_horizontal_staggered_grid_with_controller(
///         LazyHorizontalStaggeredGridArgs::default()
///             .rows(StaggeredGridCells::fixed(2))
///             .overscan(0),
///         controller,
///         |scope| {
///             scope.items(2, move |i| {
///                 rendered.with_mut(|count| *count += 1);
///                 text(TextArgs::default().text(format!("Cell {i}")));
///             });
///         },
///     );
///     assert_eq!(rendered.get(), 2);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn lazy_horizontal_staggered_grid_with_controller<F>(
    args: LazyHorizontalStaggeredGridArgs,
    controller: State<LazyStaggeredGridController>,
    configure: F,
) where
    F: FnOnce(&mut LazyHorizontalStaggeredGridScope),
{
    let mut slots = Vec::new();
    {
        let mut scope = LazyHorizontalStaggeredGridScope { slots: &mut slots };
        configure(&mut scope);
    }

    let mut scrollable_args = args.scrollable.clone();
    scrollable_args.vertical = false;
    scrollable_args.horizontal = true;

    let view_args = LazyStaggeredGridViewArgs {
        axis: StaggeredGridAxis::Horizontal,
        grid_cells: args.rows,
        main_axis_spacing: sanitize_spacing(Px::from(args.main_axis_spacing)),
        cross_axis_spacing: sanitize_spacing(Px::from(args.cross_axis_spacing)),
        cross_axis_alignment: args.cross_axis_alignment,
        item_alignment: args.item_alignment,
        estimated_item_main: ensure_positive_px(Px::from(args.estimated_item_size)),
        overscan: args.overscan,
        max_viewport_main: args.max_viewport_main,
        padding_main: sanitize_spacing(Px::from(args.content_padding)),
        padding_cross: sanitize_spacing(Px::from(args.content_padding)),
    };

    let scroll_controller = remember(ScrollableController::default);
    scrollable_with_controller(scrollable_args, scroll_controller, move || {
        lazy_staggered_grid_view(view_args, controller, slots.clone(), scroll_controller);
    });
}

#[derive(Clone)]
struct LazyStaggeredGridViewArgs {
    axis: StaggeredGridAxis,
    grid_cells: StaggeredGridCells,
    main_axis_spacing: Px,
    cross_axis_spacing: Px,
    cross_axis_alignment: MainAxisAlignment,
    item_alignment: CrossAxisAlignment,
    estimated_item_main: Px,
    overscan: usize,
    max_viewport_main: Option<Px>,
    padding_main: Px,
    padding_cross: Px,
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

impl LayoutSpec for LazyStaggeredGridLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        if input.children_ids().len() != self.visible_items.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy staggered grid measured child count mismatch".into(),
            ));
        }

        let (placements, total_main) = self.controller.with_mut(|c| {
            let mut placements = Vec::with_capacity(self.visible_items.len());
            let mut visible_iter = self
                .visible_items
                .iter()
                .zip(input.children_ids().iter())
                .peekable();

            let mut lane_offsets = vec![Px::ZERO; self.slots.len()];

            for index in 0..self.total_count {
                if self.slots.len() == 0 {
                    break;
                }

                let lane = find_shortest_lane(&lane_offsets);
                let lane_cross = self.slots.sizes.get(lane).copied().unwrap_or(Px::ZERO);
                let item_start = lane_offsets[lane];
                let mut item_main = c.cache.item_main(index).unwrap_or(self.estimated_item_main);

                if let Some((visible, child_id)) = visible_iter.peek()
                    && visible.item_index == index
                {
                    let child_constraint =
                        self.axis.child_constraint(lane_cross, self.item_alignment);
                    let child_size = input.measure_child(**child_id, &child_constraint)?;
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
                    placements.push(StaggeredPlacement {
                        child_id: **child_id,
                        position,
                    });

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

        for placement in placements {
            output.place_child(placement.child_id, placement.position);
        }

        Ok(self.axis.pack_size(reported_main, cross_with_padding))
    }
}

#[tessera]
fn lazy_staggered_grid_view(
    view_args: LazyStaggeredGridViewArgs,
    controller: State<LazyStaggeredGridController>,
    slots: Vec<LazySlot>,
    scroll_controller: State<ScrollableController>,
) {
    let plan = LazySlotPlan::new(slots);
    let total_count = plan.total_count();

    let visible_size = scroll_controller.with(|s| s.visible_size());
    let available_cross = view_args.axis.visible_cross(visible_size);
    let available_cross = (available_cross - view_args.padding_cross * 2).max(Px::ZERO);
    let grid_slots = resolve_grid_slots(
        available_cross,
        &view_args.grid_cells,
        view_args.cross_axis_spacing,
        view_args.cross_axis_alignment,
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
            let item_main = c
                .cache
                .item_main(index)
                .unwrap_or(view_args.estimated_item_main);
            lane_offsets[lane] = lane_offsets[lane] + item_main + view_args.main_axis_spacing;
        }
        finalize_lane_offsets(&lane_offsets, view_args.main_axis_spacing)
    });
    let total_main_with_padding = total_main + view_args.padding_main + view_args.padding_main;
    let cross_with_padding =
        grid_slots.cross_size + view_args.padding_cross + view_args.padding_cross;
    scroll_controller.with_mut(|c| {
        c.override_child_size(
            view_args
                .axis
                .pack_size(total_main_with_padding, cross_with_padding),
        );
    });

    let scroll_offset = view_args
        .axis
        .scroll_offset(scroll_controller.with(|s| s.child_position()));
    let padding_main = view_args.padding_main;
    let viewport_span = resolve_viewport_span(
        view_args.axis.visible_span(visible_size),
        view_args.estimated_item_main,
        view_args.main_axis_spacing,
    );
    let viewport_span = (viewport_span - (padding_main * 2)).max(Px::ZERO);

    let visible_range = controller.with(|c| {
        compute_visible_range(
            &c.cache,
            total_count,
            lane_count,
            scroll_offset,
            viewport_span,
            view_args.overscan,
            view_args.estimated_item_main,
            view_args.main_axis_spacing,
        )
    });
    let visible_items = plan.visible_items(visible_range);

    if visible_items.is_empty() {
        layout(ZeroLayout);
        return;
    }

    let viewport_limit = viewport_span + padding_main + padding_main;
    let visible_layout_items = visible_items
        .iter()
        .map(|item| VisibleStaggeredLayoutItem {
            item_index: item.item_index,
        })
        .collect();

    layout(LazyStaggeredGridLayout {
        axis: view_args.axis,
        item_alignment: view_args.item_alignment,
        estimated_item_main: view_args.estimated_item_main,
        main_spacing: view_args.main_axis_spacing,
        max_viewport_main: view_args.max_viewport_main,
        padding_main,
        padding_cross: view_args.padding_cross,
        viewport_limit,
        total_count,
        slots: grid_slots.clone(),
        visible_items: visible_layout_items,
        controller,
        scroll_controller,
    });

    for child in visible_items {
        key(child.key_hash, || {
            (child.builder)(child.local_index);
        });
    }
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum StaggeredGridAxis {
    Vertical,
    Horizontal,
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

#[derive(Clone)]
struct StaggeredPlacement {
    child_id: NodeId,
    position: PxPosition,
}

#[derive(Clone)]
enum LazySlot {
    Items(LazyItemsSlot),
}

impl LazySlot {
    fn items<F>(
        count: usize,
        builder: F,
        key_provider: Option<Arc<dyn Fn(usize) -> u64 + Send + Sync>>,
    ) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        Self::Items(LazyItemsSlot {
            count,
            builder: Arc::new(builder),
            key_provider,
        })
    }

    fn len(&self) -> usize {
        match self {
            Self::Items(slot) => slot.count,
        }
    }
}

#[derive(Clone)]
struct LazyItemsSlot {
    count: usize,
    builder: Arc<dyn Fn(usize) + Send + Sync>,
    key_provider: Option<Arc<dyn Fn(usize) -> u64 + Send + Sync>>,
}

#[derive(Clone)]
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
                    provider(local_index)
                } else {
                    let mut hasher = DefaultHasher::new();
                    index.hash(&mut hasher);
                    hasher.finish()
                };

                result.push(VisibleStaggeredItem {
                    item_index: index,
                    local_index,
                    builder: slot.builder.clone(),
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

#[derive(Clone)]
struct LazySlotEntry {
    start: usize,
    len: usize,
    slot: LazySlot,
}

#[derive(Clone)]
struct VisibleStaggeredItem {
    item_index: usize,
    local_index: usize,
    builder: Arc<dyn Fn(usize) + Send + Sync>,
    key_hash: u64,
}

#[derive(Default)]
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
