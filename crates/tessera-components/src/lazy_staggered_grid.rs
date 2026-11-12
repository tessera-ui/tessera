//! Virtualized staggered grid components for masonry layouts.
//!
//! ## Usage
//!
//! Use staggered grids to show variable-size tiles in galleries or feeds.
use std::{hash::Hash, ops::Range, sync::Arc};

use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Dp, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, RenderSlotWith, State, key,
    layout::{MeasureScope, PlacementResult, PlacementScope},
    modifier::FocusModifierExt as _,
    remember, tessera,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    layout_flex::px_mul,
    lazy_grid::{GridCells, GridSlots, compute_cell_offset, resolve_grid_slots},
    lazy_shared::{
        LazyAxis, LazyItemsPlan as LazySlotPlan, LazyItemsSlot as LazySlot, MeasuredMain,
        ZeroLayout, clamp_reported_main, hash_key,
    },
    scrollable::{
        ScrollBarBehavior, ScrollBarLayout, ScrollableController, scrollable,
        synchronize_scroll_density,
    },
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

/// Accumulated lazy staggered-grid item declarations carried by the builder.
#[derive(Clone, Default, PartialEq)]
pub struct LazyStaggeredGridContent {
    slots: Vec<LazySlot>,
}

impl LazyStaggeredGridContent {
    fn push(&mut self, slot: LazySlot) {
        self.slots.push(slot);
    }
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
/// - `scroll_smoothing` - fraction of the remaining distance kept per 60Hz
///   frame while animating a scroll step; defaults to the scrollable default.
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
/// - item declarations are appended through builder methods like `item` and
///   `items`.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_staggered_grid::{StaggeredGridCells, lazy_vertical_staggered_grid},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, tessera};
/// #[tessera]
/// fn demo() {
///     material_theme().child(|| {
///         lazy_vertical_staggered_grid()
///             .columns(StaggeredGridCells::fixed(2))
///             .overscan(0)
///             .items(4, move |i| {
///                 text(format!("Tile {i}")).modifier(
///                     Modifier::new().semantics(SemanticsArgs {
///                         test_tag: Some(format!("tile-{i}")), ..Default::default()
///                     })
///                 );
///             });
///     });
/// }
///
/// let snapshot = tessera_ui::testing::UiTest::new().content(demo).frame();
/// for index in 0..4 {
///     snapshot.node(&format!("tile-{index}")).exists();
/// }
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
    #[prop(skip_setter)] content: Option<LazyStaggeredGridContent>,
) {
    let scroll_smoothing = scroll_smoothing.unwrap_or(crate::scrollable::DEFAULT_SCROLL_SMOOTHING);
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
    let content = content.unwrap_or_default();
    let controller = match controller {
        Some(value) => value,
        None => remember(LazyStaggeredGridController::new),
    };
    let scroll_controller = remember(ScrollableController::default);
    lazy_staggered_grid_slots(
        LazyStaggeredGridSlotsArgs {
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
            scroll_controller,
            slots: content.slots,
        },
        LazyAxis::Vertical,
    );
}

#[derive(Clone, PartialEq)]
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
    scroll_controller: State<ScrollableController>,
    slots: Vec<LazySlot>,
}

#[tessera]
fn lazy_staggered_grid_slots(args: LazyStaggeredGridSlotsArgs, axis: LazyAxis) {
    let scroll_controller = args.scroll_controller;
    let main_axis_spacing = Px::from(args.main_axis_spacing).max(Px::ZERO);
    let cross_axis_spacing = Px::from(args.cross_axis_spacing).max(Px::ZERO);
    let estimated_item_main = ensure_positive_px(Px::from(args.estimated_item_size));
    let padding_main = Px::from(args.content_padding).max(Px::ZERO);
    let padding_cross = Px::from(args.content_padding).max(Px::ZERO);
    scrollable()
        .modifier(args.modifier)
        .vertical(axis == LazyAxis::Vertical)
        .horizontal(axis == LazyAxis::Horizontal)
        .apply_child_offset(false)
        .scroll_smoothing(args.scroll_smoothing)
        .scrollbar_behavior(args.scrollbar_behavior)
        .scrollbar_layout(args.scrollbar_layout)
        .controller(scroll_controller)
        .scrollbar_track_color_optional(args.scrollbar_track_color)
        .scrollbar_thumb_color_optional(args.scrollbar_thumb_color)
        .scrollbar_thumb_hover_color_optional(args.scrollbar_thumb_hover_color)
        .child(move || {
            lazy_staggered_grid_view()
                .axis(axis)
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
                .scroll_controller(scroll_controller)
                .max_viewport_main_optional(args.max_viewport_main);
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
/// - `scroll_smoothing` - fraction of the remaining distance kept per 60Hz
///   frame while animating a scroll step; defaults to the scrollable default.
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
/// - item declarations are appended through builder methods like `item` and
///   `items`.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_staggered_grid::{StaggeredGridCells, lazy_horizontal_staggered_grid},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, tessera};
/// #[tessera]
/// fn demo() {
///     material_theme().child(|| {
///         lazy_horizontal_staggered_grid()
///             .rows(StaggeredGridCells::fixed(2))
///             .overscan(0)
///             .items(3, move |i| {
///                 text(format!("Tile {i}")).modifier(
///                     Modifier::new().semantics(SemanticsArgs {
///                         test_tag: Some(format!("tile-{i}")), ..Default::default()
///                     })
///                 );
///             });
///     });
/// }
///
/// let snapshot = tessera_ui::testing::UiTest::new().content(demo).frame();
/// for index in 0..3 {
///     snapshot.node(&format!("tile-{index}")).exists();
/// }
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
    #[prop(skip_setter)] content: Option<LazyStaggeredGridContent>,
) {
    let scroll_smoothing = scroll_smoothing.unwrap_or(crate::scrollable::DEFAULT_SCROLL_SMOOTHING);
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
    let content = content.unwrap_or_default();
    let controller = match controller {
        Some(value) => value,
        None => remember(LazyStaggeredGridController::new),
    };
    let scroll_controller = remember(ScrollableController::default);
    lazy_staggered_grid_slots(
        LazyStaggeredGridSlotsArgs {
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
            scroll_controller,
            slots: content.slots,
        },
        LazyAxis::Horizontal,
    );
}

crate::lazy_shared::impl_lazy_builder!(LazyVerticalStaggeredGridBuilder, LazyStaggeredGridContent);
crate::lazy_shared::impl_lazy_builder!(
    LazyHorizontalStaggeredGridBuilder,
    LazyStaggeredGridContent
);

#[derive(Clone, PartialEq, Eq)]
struct VisibleStaggeredLayoutItem {
    item_index: usize,
}

#[derive(Clone)]
struct LazyStaggeredGridLayout {
    axis: LazyAxis,
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
    scroll_offset: Px,
    controller: State<LazyStaggeredGridController>,
    scroll_controller: State<ScrollableController>,
}

impl PartialEq for LazyStaggeredGridLayout {
    fn eq(&self, other: &Self) -> bool {
        tessera_ui::LayoutPolicy::placement_eq(self, other)
    }
}

impl tessera_ui::LayoutPolicy for LazyStaggeredGridLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() != self.visible_items.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy staggered grid measured child count mismatch".into(),
            ));
        }

        let (placements, total_main) = self.controller.with_mut(|c| {
            c.cache.prepare(
                self.total_count,
                self.slots.len(),
                self.estimated_item_main,
                self.main_spacing,
            );

            let mut placements = Vec::with_capacity(self.visible_items.len());
            for (visible, child) in self.visible_items.iter().zip(children.iter()) {
                let index = visible.item_index;
                let Some(lane) = c.cache.lane_of(index) else {
                    continue;
                };
                let lane_cross = self.slots.sizes.get(lane).copied().unwrap_or(Px::ZERO);
                let child_constraint = self.axis.child_constraint(
                    AxisConstraint::exact(lane_cross.max(Px::ZERO)),
                    self.item_alignment,
                );
                let child_size = child.measure(input, &child_constraint)?;
                let child_size = child_size.size();
                let item_main = self.axis.main(&child_size);
                c.cache.record_measurement(index, item_main);
                let Some(lane) = c.cache.lane_of(index) else {
                    continue;
                };
                let item_start = c.cache.item_start(index);

                let cell_offset = compute_cell_offset(
                    lane_cross,
                    self.axis.cross(&child_size),
                    self.item_alignment,
                );
                let cross_offset = self.padding_cross
                    + self.slots.positions.get(lane).copied().unwrap_or(Px::ZERO)
                    + cell_offset;
                let position = self.axis.position(
                    item_start + self.padding_main - self.scroll_offset,
                    cross_offset,
                );
                placements.push((*child, position));
            }

            let total_main = c.cache.total_main();
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
            total_main_with_padding,
            self.viewport_limit,
            self.axis.constraint_max(input.parent_constraint()),
            self.max_viewport_main,
        );

        for (child, position) in placements {
            result.place_child(child, position);
        }

        Ok(result.with_size(self.axis.pack_size(reported_main, cross_with_padding)))
    }

    fn measure_eq(&self, other: &Self) -> bool {
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

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other) && self.scroll_offset == other.scroll_offset
    }

    fn place(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let children = input.children();
        if children.len() != self.visible_items.len() {
            return None;
        }

        let mut result = LayoutResult::default();
        let child_sizes: Vec<_> = children.iter().map(|child| child.size()).collect();
        let positions = self.controller.with(|c| {
            compute_staggered_layout_positions(ComputeStaggeredLayoutPositionsArgs {
                axis: self.axis,
                item_alignment: self.item_alignment,
                cache: &c.cache,
                slots: &self.slots,
                padding_main: self.padding_main,
                padding_cross: self.padding_cross,
                visible_items: &self.visible_items,
                child_sizes: &child_sizes,
                scroll_offset: self.scroll_offset,
            })
        });
        if positions.len() != children.len() {
            return None;
        }

        for (child, position) in children.iter().zip(positions) {
            result.place_child(*child, position);
        }

        Some(result.into_placement_result())
    }
}

#[tessera]
fn lazy_staggered_grid_view(
    axis: Option<LazyAxis>,
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
    synchronize_scroll_density(scroll_controller);
    let plan = LazySlotPlan::new(slots.clone());
    let total_count = plan.total_count();

    let visible_size = scroll_controller.with(|s| s.visible_size());
    let available_cross = axis.cross(&visible_size);
    let available_cross = (available_cross - padding_cross * 2).max(Px::ZERO);
    let grid_slots = resolve_grid_slots(
        available_cross,
        &grid_cells,
        cross_axis_spacing,
        cross_axis_alignment,
    );
    let lane_count = grid_slots.len();

    controller.with_mut(|c| {
        c.cache.prepare(
            total_count,
            lane_count,
            estimated_item_main,
            main_axis_spacing,
        );
    });
    let total_main = controller.with(|c| c.cache.total_main());
    let total_main_with_padding = total_main + padding_main + padding_main;
    let cross_with_padding = grid_slots.cross_size + padding_cross + padding_cross;
    scroll_controller.with_mut(|c| {
        c.override_child_size(axis.pack_size(total_main_with_padding, cross_with_padding));
    });

    let scroll_offset = axis.scroll_offset(scroll_controller.with(|s| s.child_position()));
    let viewport_span = resolve_viewport_span(
        axis.main(&visible_size),
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
            main_axis_spacing,
        )
    });
    let visible_items = visible_items(&plan, visible_range.clone());

    if visible_items.is_empty() {
        tessera_ui::layout::layout(ZeroLayout);
        return;
    }

    let focus_modifier =
        lazy_staggered_grid_focus_beyond_bounds_modifier(LazyStaggeredGridFocusArgs {
            axis,
            controller,
            scroll_controller,
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

    tessera_ui::layout::layout(LazyStaggeredGridLayout {
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
        scroll_offset,
        controller,
        scroll_controller,
    })
    .modifier(focus_modifier)
    .child(move || {
        for child in &visible_items {
            let child = child.clone();
            key(child.key_hash, || {
                lazy_staggered_grid_item_host()
                    .builder_handle_shared(child.builder)
                    .local_index(child.local_index);
            });
        }
    });
}

#[tessera]
fn lazy_staggered_grid_item_host(
    builder_handle: Option<RenderSlotWith<usize>>,
    local_index: Option<usize>,
) {
    let local_index = local_index.unwrap_or(0);
    if let Some(builder_handle) = builder_handle {
        builder_handle.render(local_index);
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

fn compute_visible_range(
    cache: &StaggeredGridCache,
    total_count: usize,
    lane_count: usize,
    scroll_offset: Px,
    viewport_span: Px,
    overscan: usize,
    spacing: Px,
) -> Range<usize> {
    if total_count == 0 || lane_count == 0 || cache.built_len() != total_count {
        return 0..0;
    }

    let viewport_start = scroll_offset;
    let viewport_end = scroll_offset + viewport_span;
    let mut first_visible = None;
    let mut last_visible = None;

    for lane in 0..lane_count {
        let ends = cache.lane_ends(lane);
        if ends.is_empty() {
            continue;
        }
        let first = ends.partition_point(|end| *end < viewport_start);
        let last = if viewport_end < Px::ZERO {
            0
        } else {
            (ends.partition_point(|end| *end + spacing <= viewport_end) + 1).min(ends.len())
        };
        if first >= last {
            continue;
        }
        if let Some(start_index) = cache.item_at_lane_slot(lane, first) {
            first_visible =
                Some(first_visible.map_or(start_index, |current: usize| current.min(start_index)));
        }
        if let Some(end_index) = cache.item_at_lane_slot(lane, last - 1) {
            last_visible =
                Some(last_visible.map_or(end_index, |current: usize| current.max(end_index)));
        }
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

fn visible_items(plan: &LazySlotPlan, range: Range<usize>) -> Vec<VisibleStaggeredItem> {
    let mut result = Vec::new();
    for index in range {
        if let Some((slot, local_index)) = plan.resolve(index) {
            let key_hash = if let Some(provider) = &slot.key_provider {
                provider.call(local_index)
            } else {
                hash_key(index)
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

#[derive(Clone, PartialEq)]
struct VisibleStaggeredItem {
    item_index: usize,
    local_index: usize,
    builder: RenderSlotWith<usize>,
    key_hash: u64,
}

struct LazyStaggeredGridFocusArgs {
    axis: LazyAxis,
    controller: State<LazyStaggeredGridController>,
    scroll_controller: State<ScrollableController>,
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
        let Some(forward) = args.axis.focus_forward(direction) else {
            return false;
        };
        if args.total_count == 0 || args.lane_count == 0 || args.viewport_span <= Px::ZERO {
            return false;
        }

        let target_index = if forward {
            if args.visible_range.end >= args.total_count {
                return false;
            }
            args.visible_range.end
        } else {
            let Some(index) = args.visible_range.start.checked_sub(1) else {
                return false;
            };
            index
        };

        let Some((target_offset, target_main)) =
            args.controller.with(|c| c.cache.item_layout(target_index))
        else {
            return false;
        };

        let desired_scroll = if forward {
            (target_offset + target_main - args.viewport_span).max(Px::ZERO)
        } else {
            target_offset
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

#[derive(PartialEq)]
struct StaggeredGridCache {
    /// Measured main-axis sizes. `None` still uses the estimated size.
    item_main: Vec<Option<MeasuredMain>>,
    density: f64,
    /// Lane assignment for each built item, parallel to the item prefix.
    lane_of: Vec<usize>,
    /// Exclusive main-axis end of every item in a lane, in placement order.
    lane_ends: Vec<Vec<Px>>,
    /// Item index occupying each slot of `lane_ends`.
    lane_items: Vec<Vec<usize>>,
    lane_count: usize,
    spacing: Px,
    estimated_item_main: Px,
}

impl Default for StaggeredGridCache {
    fn default() -> Self {
        Self {
            item_main: Vec::new(),
            density: 0.0,
            lane_of: Vec::new(),
            lane_ends: Vec::new(),
            lane_items: Vec::new(),
            lane_count: 0,
            spacing: Px::ZERO,
            estimated_item_main: Px::ZERO,
        }
    }
}

impl StaggeredGridCache {
    /// Keep `item_main` aligned with `count` and refresh lane geometry.
    ///
    /// A longer list only appends. A shorter list truncates. Changing lane
    /// count, spacing, or estimated size drops the lane tables. Inserts and
    /// reorders cannot be told apart from length, so they invalidate the whole
    /// table instead of pretending keys stayed stable.
    fn prepare(&mut self, count: usize, lane_count: usize, estimated_item_main: Px, spacing: Px) {
        let density = tessera_ui::dp::scale_factor();
        if self.density != density {
            for measurement in self.item_main.iter_mut().flatten() {
                measurement.rebase(density);
            }
            self.density = density;
            self.invalidate_geometry();
        }
        if self.lane_count != lane_count
            || self.spacing != spacing
            || self.estimated_item_main != estimated_item_main
        {
            self.invalidate_geometry();
            self.lane_count = lane_count;
            self.spacing = spacing;
            self.estimated_item_main = estimated_item_main;
        }

        if self.item_main.len() > count {
            self.item_main.truncate(count);
        } else if self.item_main.len() < count {
            self.item_main.resize(count, None);
        }

        if self.lane_of.len() > self.item_main.len() {
            self.truncate_geometry(self.item_main.len());
        }

        self.ensure_built(self.item_main.len());
    }

    fn item_main(&self, index: usize) -> Option<Px> {
        self.item_main
            .get(index)
            .copied()
            .flatten()
            .map(MeasuredMain::pixels)
    }

    fn record_measurement(&mut self, index: usize, actual: Px) {
        if index >= self.item_main.len() {
            return;
        }
        if self.item_main(index) == Some(actual) {
            return;
        }
        self.item_main[index] = Some(MeasuredMain::new(actual, self.density));
        self.truncate_geometry(index);
        self.ensure_built(self.item_main.len());
    }

    fn built_len(&self) -> usize {
        self.lane_of.len()
    }

    fn lane_of(&self, index: usize) -> Option<usize> {
        self.lane_of.get(index).copied()
    }

    fn lane_ends(&self, lane: usize) -> &[Px] {
        self.lane_ends.get(lane).map(Vec::as_slice).unwrap_or(&[])
    }

    fn item_start(&self, index: usize) -> Px {
        let Some(lane) = self.lane_of.get(index).copied() else {
            return Px::ZERO;
        };
        let Some(items) = self.lane_items.get(lane) else {
            return Px::ZERO;
        };
        let Some(slot) = items.iter().position(|item| *item == index) else {
            return Px::ZERO;
        };
        if slot == 0 {
            Px::ZERO
        } else {
            self.lane_ends
                .get(lane)
                .and_then(|ends| ends.get(slot - 1))
                .copied()
                .unwrap_or(Px::ZERO)
                + self.spacing
        }
    }

    fn item_layout(&self, index: usize) -> Option<(Px, Px)> {
        if index >= self.lane_of.len() {
            return None;
        }
        Some((self.item_start(index), self.resolved_main(index)))
    }

    fn item_at_lane_slot(&self, lane: usize, slot: usize) -> Option<usize> {
        self.lane_items.get(lane)?.get(slot).copied()
    }

    fn total_main(&self) -> Px {
        if self.lane_count == 0
            || self.item_main.is_empty()
            || self.lane_of.len() != self.item_main.len()
        {
            return Px::ZERO;
        }
        self.lane_ends
            .iter()
            .filter_map(|ends| ends.last().copied())
            .max()
            .unwrap_or(Px::ZERO)
    }

    fn resolved_main(&self, index: usize) -> Px {
        self.item_main(index).unwrap_or(self.estimated_item_main)
    }

    fn invalidate_geometry(&mut self) {
        self.lane_of.clear();
        self.lane_ends.clear();
        self.lane_items.clear();
    }

    /// Drop lane rows at and after `from`. Rows before `from` stay put.
    fn truncate_geometry(&mut self, from: usize) {
        let keep = from.min(self.lane_of.len());
        if keep == self.lane_of.len() && self.lane_ends.len() == self.lane_count {
            return;
        }
        self.lane_of.truncate(keep);
        for items in &mut self.lane_items {
            items.retain(|index| *index < keep);
        }
        let spacing = self.spacing;
        let mains: Vec<Px> = (0..keep).map(|index| self.resolved_main(index)).collect();
        for lane in 0..self.lane_items.len() {
            let Some(items) = self.lane_items.get(lane).cloned() else {
                continue;
            };
            let mut cursor = Px::ZERO;
            let mut rebuilt = Vec::with_capacity(items.len());
            for (slot, index) in items.iter().copied().enumerate() {
                let main = mains.get(index).copied().unwrap_or(Px::ZERO);
                if slot > 0 {
                    cursor += spacing;
                }
                cursor += main;
                rebuilt.push(cursor);
            }
            if let Some(ends) = self.lane_ends.get_mut(lane) {
                *ends = rebuilt;
            }
        }
    }

    fn ensure_built(&mut self, through: usize) {
        let through = through.min(self.item_main.len());
        if self.lane_count == 0 {
            self.lane_of.clear();
            self.lane_ends.clear();
            self.lane_items.clear();
            return;
        }
        if self.lane_ends.len() != self.lane_count || self.lane_items.len() != self.lane_count {
            self.invalidate_geometry();
            self.lane_ends = vec![Vec::new(); self.lane_count];
            self.lane_items = vec![Vec::new(); self.lane_count];
        }
        if self.lane_of.len() > through {
            self.truncate_geometry(through);
        }
        while self.lane_of.len() < through {
            let index = self.lane_of.len();
            let lane = shortest_lane_index(&self.lane_ends);
            let item_main = self.resolved_main(index);
            let start = self
                .lane_ends
                .get(lane)
                .and_then(|ends| ends.last().copied())
                .map(|end| end + self.spacing)
                .unwrap_or(Px::ZERO);
            self.lane_of.push(lane);
            if let Some(items) = self.lane_items.get_mut(lane) {
                items.push(index);
            }
            if let Some(ends) = self.lane_ends.get_mut(lane) {
                ends.push(start + item_main);
            }
        }
    }
}

fn shortest_lane_index(lane_ends: &[Vec<Px>]) -> usize {
    let mut index = 0;
    let mut best = lane_ends
        .first()
        .and_then(|ends| ends.last().copied())
        .unwrap_or(Px::ZERO);
    for (lane, ends) in lane_ends.iter().enumerate().skip(1) {
        let end = ends.last().copied().unwrap_or(Px::ZERO);
        if end < best {
            best = end;
            index = lane;
        }
    }
    index
}

fn ensure_positive_px(px: Px) -> Px {
    if px <= Px::ZERO { Px(1) } else { px }
}

struct ComputeStaggeredLayoutPositionsArgs<'a> {
    axis: LazyAxis,
    item_alignment: CrossAxisAlignment,
    cache: &'a StaggeredGridCache,
    slots: &'a GridSlots,
    padding_main: Px,
    padding_cross: Px,
    visible_items: &'a [VisibleStaggeredLayoutItem],
    child_sizes: &'a [ComputedData],
    scroll_offset: Px,
}

fn compute_staggered_layout_positions(
    args: ComputeStaggeredLayoutPositionsArgs<'_>,
) -> Vec<PxPosition> {
    let ComputeStaggeredLayoutPositionsArgs {
        axis,
        item_alignment,
        cache,
        slots,
        padding_main,
        padding_cross,
        visible_items,
        child_sizes,
        scroll_offset,
    } = args;

    let mut positions = Vec::with_capacity(visible_items.len());
    if slots.len() == 0 {
        return positions;
    }

    for (visible, child_size) in visible_items.iter().zip(child_sizes.iter()) {
        let index = visible.item_index;
        let Some(lane) = cache.lane_of(index) else {
            continue;
        };
        let lane_cross = slots.sizes.get(lane).copied().unwrap_or(Px::ZERO);
        let item_start = cache.item_start(index);
        let cell_offset = compute_cell_offset(lane_cross, axis.cross(child_size), item_alignment);
        let cross_offset =
            padding_cross + slots.positions.get(lane).copied().unwrap_or(Px::ZERO) + cell_offset;
        positions.push(axis.position(item_start + padding_main - scroll_offset, cross_offset));
    }

    positions
}

#[cfg(test)]
mod tests {
    use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
    use tessera_ui::{
        AxisConstraint, ComputedData, LayoutResult, MeasurementError, Modifier, NoopDrawModifier,
        Px, PxPosition, RenderSlotWith, layout::MeasureScope, remember, tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        lazy_shared::LazyAxis,
        scrollable::{ScrollBarBehavior, ScrollableController, scrollable},
        theme::{MaterialTheme, material_theme},
    };

    use super::{GridCells, LazyStaggeredGridController, lazy_staggered_grid_view};

    #[derive(Clone, PartialEq)]
    struct FixedTestLayout {
        width: i32,
        height: i32,
    }

    impl tessera_ui::LayoutPolicy for FixedTestLayout {
        fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
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

        tessera_ui::layout::layout(FixedTestLayout { width, height })
            .draw(NoopDrawModifier)
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
            .axis(LazyAxis::Vertical)
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
                RenderSlotWith::new(|index| match index {
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
                }),
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
            .axis(LazyAxis::Horizontal)
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
                RenderSlotWith::new(|index| match index {
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
                }),
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
                height: Px::new(20),
            });
            c.set_scroll_position(target_position);
        });

        scrollable()
            .scrollbar_behavior(ScrollBarBehavior::Hidden)
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(20))),
            ))
            .vertical(true)
            .horizontal(false)
            .apply_child_offset(false)
            .controller(scroll_controller)
            .child(move || {
                lazy_staggered_grid_view()
                    .axis(LazyAxis::Vertical)
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
                        RenderSlotWith::new(|index| match index {
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
                        }),
                        None,
                    )])
                    .controller(controller)
                    .scroll_controller(scroll_controller);
            });
    }

    #[tessera]
    fn lazy_vertical_staggered_stable_visible_items_case() {
        let controller = remember(LazyStaggeredGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(60),
                height: Px::new(50),
            });
        });

        tessera_ui::launch_once(async move {
            // Frames are pumped every 16.67ms, starting after the launching frame.
            for frame in 1.. {
                tessera_ui::with_frame_time(|_| ()).await;
                let offset = if frame == 1 { 1 } else { 2 };
                scroll_controller.with_mut(|c| {
                    c.set_scroll_position(PxPosition::new(Px::ZERO, Px::new(-offset)));
                });
            }
        });

        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                lazy_staggered_grid_view()
                    .axis(LazyAxis::Vertical)
                    .grid_cells(GridCells::fixed(2))
                    .main_axis_spacing(Px::new(3))
                    .cross_axis_spacing(Px::new(3))
                    .cross_axis_alignment(MainAxisAlignment::Start)
                    .item_alignment(CrossAxisAlignment::Start)
                    .estimated_item_main(Px::new(20))
                    .overscan(0)
                    .padding_main(Px::new(4))
                    .padding_cross(Px::new(4))
                    .slots(vec![super::LazySlot::items(
                        20,
                        RenderSlotWith::new(|index| {
                            fixed_test_box()
                                .tag(format!("lazy_staggered_stable_visible_{index}"))
                                .width(20)
                                .height(20);
                        }),
                        None,
                    )])
                    .controller(controller)
                    .scroll_controller(scroll_controller);
            });
    }

    #[test]
    fn lazy_vertical_staggered_positions_items_in_shortest_lanes() {
        let layout = tessera_ui::testing::UiTest::new()
            .viewport(80, 80)
            .content(lazy_vertical_staggered_layout_case)
            .frame();
        layout.node("lazy_staggered_v_first").rect(4, 4, 10, 10);
        layout.node("lazy_staggered_v_second").rect(32, 4, 12, 8);
        layout.node("lazy_staggered_v_third").rect(32, 15, 11, 12);
        layout.node("lazy_staggered_v_fourth").rect(4, 17, 9, 9);
    }

    #[test]
    fn lazy_horizontal_staggered_positions_items_in_shortest_lanes() {
        let layout = tessera_ui::testing::UiTest::new()
            .viewport(90, 50)
            .content(lazy_horizontal_staggered_layout_case)
            .frame();
        layout.node("lazy_staggered_h_first").rect(4, 4, 20, 10);
        layout.node("lazy_staggered_h_second").rect(4, 22, 15, 12);
        layout.node("lazy_staggered_h_third").rect(22, 22, 18, 8);
        layout.node("lazy_staggered_h_fourth").rect(27, 4, 16, 9);
    }

    #[test]
    fn lazy_vertical_staggered_scroll_repositions_visible_items() {
        let layout = tessera_ui::testing::UiTest::new()
            .viewport(80, 80)
            .content(lazy_vertical_staggered_scrolled_layout_case)
            .frame();
        layout
            .node("lazy_staggered_scroll_first")
            .rect(4, -1, 10, 10);
        layout
            .node("lazy_staggered_scroll_second")
            .rect(32, -1, 12, 8);
        layout
            .node("lazy_staggered_scroll_third")
            .rect(32, 10, 11, 12);
        layout
            .node("lazy_staggered_scroll_fourth")
            .rect(4, 12, 9, 9);
    }

    #[test]
    fn lazy_vertical_staggered_does_not_rebuild_visible_items_when_scroll_stays_in_same_window() {
        let mut session = tessera_ui::testing::UiTest::new()
            .content(lazy_vertical_staggered_stable_visible_items_case)
            .viewport(80, 60);

        session.frame();
        let first_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build);
        assert!(first_builds > 0);

        session.frames(1);
        let second_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build);
        assert_eq!(second_builds, 0);

        session.frames(1);
        let third_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::testing::WorkPhase::Build);
        assert_eq!(third_builds, 0);
    }

    #[test]
    fn density_changes_rebase_lanes_and_keep_the_complete_final_item_extent() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_staggered_grid::tests::density_changes_rebase_lanes_and_keep_the_complete_final_item_extent",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let mut cache = super::StaggeredGridCache::default();
                cache.prepare(6, 2, Px(20), Px(4));
                for (index, size) in [(0, 33), (1, 33), (4, 55), (5, 55)] {
                    cache.record_measurement(index, Px(size));
                }
                for (density, estimated, spacing, last_start, last_main, total) in [
                    (1.25, 25, 5, 76, 68, 144),
                    (1.5, 30, 6, 91, 82, 173),
                    (2.0, 40, 8, 122, 110, 232),
                    (1.0, 20, 4, 61, 55, 116),
                ] {
                    tessera_ui::dp::set_scale_factor(density);
                    cache.prepare(6, 2, Px(estimated), Px(spacing));
                    assert_eq!(cache.item_layout(4), Some((Px(last_start), Px(last_main))));
                    assert_eq!(cache.item_layout(5), Some((Px(last_start), Px(last_main))));
                    assert_eq!(cache.total_main(), Px(total));
                }
            },
        );
    }

    #[test]
    fn staggered_cache_keeps_lanes_before_a_remeasured_item() {
        let mut cache = super::StaggeredGridCache::default();
        let estimated = Px::new(10);
        let spacing = Px::new(2);
        cache.prepare(6, 2, estimated, spacing);
        cache.record_measurement(0, Px::new(30));
        cache.record_measurement(1, Px::new(10));
        cache.record_measurement(2, Px::new(10));
        cache.record_measurement(3, Px::new(40));
        cache.record_measurement(4, Px::new(12));
        cache.record_measurement(5, Px::new(8));

        let lanes_before: Vec<usize> = (0..6)
            .map(|index| {
                cache
                    .lane_of(index)
                    .expect("every measured item must have a lane")
            })
            .collect();
        let starts_before: Vec<Px> = (0..3).map(|index| cache.item_start(index)).collect();

        cache.record_measurement(3, Px::new(4));

        let lanes_after: Vec<usize> = (0..6)
            .map(|index| {
                cache
                    .lane_of(index)
                    .expect("every measured item must have a lane")
            })
            .collect();
        let starts_after: Vec<Px> = (0..3).map(|index| cache.item_start(index)).collect();
        assert_eq!(&lanes_after[..3], &lanes_before[..3]);
        assert_eq!(starts_after, starts_before);
        assert_ne!(&lanes_after[3..], &lanes_before[3..]);
    }
}
