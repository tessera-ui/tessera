//! Virtualized grid components for efficient, scrollable tile layouts.
//!
//! ## Usage
//!
//! Use lazy grids to display large, scrollable collections of tiles.
use std::{hash::Hash, ops::Range, sync::Arc};

use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Dp, LayoutResult, MeasurementError,
    Modifier, Px, RenderSlotWith, State, key,
    layout::{LayoutChild, MeasureScope, PlacementResult, PlacementScope},
    modifier::FocusModifierExt as _,
    remember, tessera,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    layout_flex::px_mul,
    lazy_shared::{
        FenwickTree, LazyAxis, LazyItemsPlan as LazySlotPlan, LazyItemsSlot as LazySlot,
        MeasuredMain, ZeroLayout, clamp_reported_main, hash_key,
    },
    scrollable::{
        ScrollBarBehavior, ScrollBarLayout, ScrollableController, scrollable,
        synchronize_scroll_density,
    },
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
                let spacing = spacing.max(Px::ZERO);
                let available_i32 = available.0.max(0);
                let min_i32 = min_px.0.max(1);
                let spacing_i32 = spacing.0.max(0);
                let count =
                    ((available_i32 + spacing_i32) / (min_i32 + spacing_i32)).max(1) as usize;
                calculate_cells_cross_axis_size(available, count, spacing)
            }
            Self::FixedSize(size) => {
                let cell_size = ensure_positive_px(Px::from(*size));
                let spacing = spacing.max(Px::ZERO);
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

/// Accumulated lazy-grid item declarations carried by the builder.
#[derive(Clone, Default, PartialEq)]
pub struct LazyGridContent {
    slots: Vec<LazySlot>,
}

impl LazyGridContent {
    fn push(&mut self, slot: LazySlot) {
        self.slots.push(slot);
    }
}

#[derive(Clone, PartialEq, Eq)]
struct VisibleGridLayoutItem {
    line_index: usize,
    slot_index: usize,
}

#[derive(Clone)]
struct LazyGridLayout {
    axis: LazyAxis,
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
    scroll_offset: Px,
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
            && self.scroll_offset == other.scroll_offset
    }
}

impl tessera_ui::LayoutPolicy for LazyGridLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() != self.visible_items.len() {
            return Err(MeasurementError::MeasureFnFailed(
                "Lazy grid measured child count mismatch".into(),
            ));
        }

        let mut measured_items = Vec::with_capacity(self.visible_items.len());
        let line_count = self.line_range.end.saturating_sub(self.line_range.start);
        let mut line_max = vec![Px::ZERO; line_count];

        for (visible, child) in self.visible_items.iter().zip(children.iter()) {
            let cell_cross = self
                .slots
                .sizes
                .get(visible.slot_index)
                .copied()
                .unwrap_or(Px::ZERO);
            let child_constraint = self.axis.child_constraint(
                AxisConstraint::exact(cell_cross.max(Px::ZERO)),
                self.item_alignment,
            );
            let child_size = child.measure(input, &child_constraint)?;
            let line_idx = visible.line_index - self.line_range.start;
            if let Some(line_value) = line_max.get_mut(line_idx) {
                *line_value = (*line_value).max(self.axis.main(&child_size.size()));
            }
            measured_items.push(MeasuredGridItem {
                child: *child,
                line_index: visible.line_index,
                slot_index: visible.slot_index,
                size: child_size.size(),
            });
        }

        // Preserve usable line measurements while the cross axis is collapsed.
        let measurements_changed = self.slots.sizes.iter().all(|size| *size > Px::ZERO)
            && self.controller.with(|c| {
                line_max.iter().enumerate().any(|(offset, size)| {
                    c.cache
                        .measured_line_main
                        .get(self.line_range.start + offset)
                        .copied()
                        .flatten()
                        .map(MeasuredMain::pixels)
                        != Some(*size)
                })
            });
        if measurements_changed {
            self.controller.with_mut(|c| {
                for (offset, line_main) in line_max.iter().enumerate() {
                    let line_index = self.line_range.start + offset;
                    c.cache.record_line_measurement(
                        line_index,
                        *line_main,
                        self.estimated_line_main,
                    );
                }
            });
        }

        let (placements, total_main) = self.controller.with(|c| {
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
                let position = self.axis.position(
                    line_offset + self.padding_main - self.scroll_offset,
                    cross_offset,
                );
                placements.push((item.child, position));
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
        if self.scroll_controller.with(|c| c.child_size() != size) {
            self.scroll_controller
                .with_mut(|c| c.override_child_size(size));
        }

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
        for (visible, child) in self.visible_items.iter().zip(children.iter()) {
            let line_offset = self.controller.with(|c| {
                c.cache.offset_for_line(
                    visible.line_index,
                    self.estimated_line_main,
                    self.main_spacing,
                )
            });
            let cell_cross = self
                .slots
                .sizes
                .get(visible.slot_index)
                .copied()
                .unwrap_or(Px::ZERO);
            let child_size = child.size();
            let cell_offset = compute_cell_offset(
                cell_cross,
                self.axis.cross(&child_size),
                self.item_alignment,
            );
            let cross_offset = self.padding_cross
                + self
                    .slots
                    .positions
                    .get(visible.slot_index)
                    .copied()
                    .unwrap_or(Px::ZERO)
                + cell_offset;
            let position = self.axis.position(
                line_offset + self.padding_main - self.scroll_offset,
                cross_offset,
            );
            result.place_child(*child, position);
        }

        Some(result.into_placement_result())
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
/// - `scroll_smoothing` - fraction of the remaining distance kept per 60Hz
///   frame while animating a scroll step; defaults to the scrollable default.
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
/// - item declarations are appended through builder methods like `item` and
///   `items`.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_grid::{GridCells, lazy_vertical_grid},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_components::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, tessera};
/// #[tessera]
/// fn demo() {
///     material_theme().child(|| {
///         lazy_vertical_grid()
///             .columns(GridCells::fixed(2))
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
/// let snapshot = tessera_ui::testing::layout_test(demo).run();
/// for index in 0..4 {
///     snapshot.node(&format!("tile-{index}")).exists();
/// }
/// ```
#[tessera]
pub fn lazy_vertical_grid(
    modifier: Option<Modifier>,
    scroll_smoothing: Option<f32>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: Option<ScrollBarLayout>,
    columns: Option<GridCells>,
    main_axis_spacing: Option<Dp>,
    cross_axis_spacing: Option<Dp>,
    cross_axis_alignment: Option<MainAxisAlignment>,
    item_alignment: Option<CrossAxisAlignment>,
    overscan: Option<usize>,
    estimated_item_size: Option<Dp>,
    content_padding: Option<Dp>,
    max_viewport_main: Option<Px>,
    controller: Option<State<LazyGridController>>,
    #[prop(skip_setter)] content: Option<LazyGridContent>,
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
        None => remember(LazyGridController::new),
    };
    let scroll_controller = remember(ScrollableController::default);
    lazy_grid_slots(
        LazyGridSlotsArgs {
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
    scroll_controller: State<ScrollableController>,
    slots: Vec<LazySlot>,
}

#[tessera]
fn lazy_grid_slots(args: LazyGridSlotsArgs, axis: LazyAxis) {
    let scroll_controller = args.scroll_controller;
    let main_axis_spacing = Px::from(args.main_axis_spacing).max(Px::ZERO);
    let cross_axis_spacing = Px::from(args.cross_axis_spacing).max(Px::ZERO);
    let estimated_line_main = ensure_positive_px(Px::from(args.estimated_item_size));
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
            lazy_grid_view()
                .axis(axis)
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
                .scroll_controller(scroll_controller)
                .max_viewport_main_optional(args.max_viewport_main);
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
/// - `scroll_smoothing` - fraction of the remaining distance kept per 60Hz
///   frame while animating a scroll step; defaults to the scrollable default.
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
/// - item declarations are appended through builder methods like `item` and
///   `items`.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     lazy_grid::{GridCells, lazy_horizontal_grid},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_components::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, tessera};
/// #[tessera]
/// fn demo() {
///     material_theme().child(|| {
///         lazy_horizontal_grid()
///             .rows(GridCells::fixed(2))
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
/// let snapshot = tessera_ui::testing::layout_test(demo).run();
/// for index in 0..3 {
///     snapshot.node(&format!("tile-{index}")).exists();
/// }
/// ```
#[tessera]
pub fn lazy_horizontal_grid(
    modifier: Option<Modifier>,
    scroll_smoothing: Option<f32>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: Option<ScrollBarLayout>,
    rows: Option<GridCells>,
    main_axis_spacing: Option<Dp>,
    cross_axis_spacing: Option<Dp>,
    cross_axis_alignment: Option<MainAxisAlignment>,
    item_alignment: Option<CrossAxisAlignment>,
    overscan: Option<usize>,
    estimated_item_size: Option<Dp>,
    content_padding: Option<Dp>,
    max_viewport_main: Option<Px>,
    controller: Option<State<LazyGridController>>,
    #[prop(skip_setter)] content: Option<LazyGridContent>,
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
        None => remember(LazyGridController::new),
    };
    let scroll_controller = remember(ScrollableController::default);
    lazy_grid_slots(
        LazyGridSlotsArgs {
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

crate::lazy_shared::impl_lazy_builder!(LazyVerticalGridBuilder, LazyGridContent);
crate::lazy_shared::impl_lazy_builder!(LazyHorizontalGridBuilder, LazyGridContent);

#[tessera]
fn lazy_grid_view(
    axis: Option<LazyAxis>,
    grid_cells: Option<GridCells>,
    main_axis_spacing: Option<Px>,
    cross_axis_spacing: Option<Px>,
    cross_axis_alignment: Option<MainAxisAlignment>,
    item_alignment: Option<CrossAxisAlignment>,
    estimated_line_main: Option<Px>,
    overscan: Option<usize>,
    max_viewport_main: Option<Px>,
    padding_main: Option<Px>,
    padding_cross: Option<Px>,
    controller: Option<State<LazyGridController>>,
    slots: Option<Vec<LazySlot>>,
    scroll_controller: Option<State<ScrollableController>>,
) {
    let axis = axis.unwrap_or_default();
    let grid_cells = grid_cells.unwrap_or_default();
    let main_axis_spacing = main_axis_spacing.unwrap_or(Px::ZERO);
    let cross_axis_spacing = cross_axis_spacing.unwrap_or(Px::ZERO);
    let cross_axis_alignment = cross_axis_alignment.unwrap_or_default();
    let item_alignment = item_alignment.unwrap_or_default();
    let estimated_line_main = estimated_line_main.unwrap_or(Px::ZERO);
    let overscan = overscan.unwrap_or(0);
    let padding_main = padding_main.unwrap_or(Px::ZERO);
    let padding_cross = padding_cross.unwrap_or(Px::ZERO);
    let slots = slots.unwrap_or_default();
    let controller = controller.expect("lazy_grid_view requires controller");
    let scroll_controller = scroll_controller.expect("lazy_grid_view requires scroll_controller");
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
    let slots_per_line = grid_slots.len();

    let cache_config_changed = controller.with(|c| {
        c.cache.total_items != total_count
            || c.cache.slots_per_line != slots_per_line.max(1)
            || c.cache.estimated_main != estimated_line_main
            || c.cache.density != tessera_ui::dp::scale_factor()
    });
    if cache_config_changed {
        controller.with_mut(|c| {
            c.cache
                .configure(total_count, slots_per_line, estimated_line_main)
        });
    }
    let total_main = controller.with(|c| {
        c.cache
            .total_main_size(estimated_line_main, main_axis_spacing)
    });
    let total_main_with_padding = total_main + padding_main + padding_main;
    let cross_with_padding = grid_slots.cross_size + padding_cross + padding_cross;
    let child_size = axis.pack_size(total_main_with_padding, cross_with_padding);
    if scroll_controller.with(|c| c.child_size() != child_size) {
        scroll_controller.with_mut(|c| c.override_child_size(child_size));
    }

    let scroll_offset = axis.scroll_offset(scroll_controller.with(|s| s.child_position()));
    let viewport_span = resolve_viewport_span(
        axis.main(&visible_size),
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
        tessera_ui::layout::layout(ZeroLayout);
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
    tessera_ui::layout::layout(LazyGridLayout {
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
        scroll_offset,
        controller,
        scroll_controller,
    })
    .modifier(focus_modifier)
    .child(move || {
        for child in &items {
            let child = child.clone();
            key(child.key_hash, || {
                lazy_grid_item_host()
                    .builder_handle_shared(child.builder)
                    .local_index(child.local_index);
            });
        }
    });
}

#[tessera]
fn lazy_grid_item_host(builder_handle: Option<RenderSlotWith<usize>>, local_index: Option<usize>) {
    let local_index = local_index.unwrap_or(0);
    if let Some(builder_handle) = builder_handle {
        builder_handle.render(local_index);
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
                    hash_key(index)
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

pub(crate) fn compute_cell_offset(
    cell_cross: Px,
    child_cross: Px,
    alignment: CrossAxisAlignment,
) -> Px {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Px::ZERO,
        CrossAxisAlignment::Center => (cell_cross - child_cross).max(Px::ZERO) / 2,
        CrossAxisAlignment::End => (cell_cross - child_cross).max(Px::ZERO),
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct GridSlots {
    pub(crate) sizes: Vec<Px>,
    pub(crate) positions: Vec<Px>,
    pub(crate) cross_size: Px,
}

impl GridSlots {
    pub(crate) fn len(&self) -> usize {
        self.sizes.len()
    }
}

pub(crate) fn resolve_grid_slots(
    available_cross: Px,
    grid_cells: &GridCells,
    spacing: Px,
    alignment: MainAxisAlignment,
) -> GridSlots {
    let spacing = spacing.max(Px::ZERO);
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
struct MeasuredGridItem {
    child: LayoutChild,
    line_index: usize,
    slot_index: usize,
    size: ComputedData,
}

#[derive(Clone, PartialEq)]
struct VisibleGridItem {
    local_index: usize,
    line_index: usize,
    slot_index: usize,
    builder: RenderSlotWith<usize>,
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
    axis: LazyAxis,
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
        let Some(forward) = args.axis.focus_forward(direction) else {
            return false;
        };
        if args.total_lines == 0 || args.viewport_span <= Px::ZERO {
            return false;
        }

        let target_line = if forward {
            if args.visible_line_range.end >= args.total_lines {
                return false;
            }
            args.visible_line_range.end
        } else {
            let Some(line) = args.visible_line_range.start.checked_sub(1) else {
                return false;
            };
            line
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
                    .map(MeasuredMain::pixels)
                    .unwrap_or(args.estimated_line_main),
            )
        });

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
#[derive(PartialEq, Default)]
struct LazyGridCache {
    total_items: usize,
    slots_per_line: usize,
    estimated_main: Px,
    density: f64,
    measured_line_main: Vec<Option<MeasuredMain>>,
    fenwick: FenwickTree,
}

impl LazyGridCache {
    fn configure(&mut self, count: usize, slots_per_line: usize, estimated: Px) {
        let density = tessera_ui::dp::scale_factor();
        let density_changed = self.density != density;
        let slots_per_line = slots_per_line.max(1);
        if self.total_items != count || self.slots_per_line != slots_per_line {
            self.total_items = count;
            self.slots_per_line = slots_per_line;
            let lines = line_count(count, self.slots_per_line);
            self.measured_line_main = vec![None; lines];
            self.fenwick.resize(lines);
        }
        if self.estimated_main != estimated || density_changed {
            self.estimated_main = estimated;
            self.density = density;
            self.fenwick.resize(self.measured_line_main.len());
            for (index, actual) in self.measured_line_main.iter_mut().enumerate() {
                if let Some(actual) = actual {
                    if density_changed {
                        actual.rebase(density);
                    }
                    self.fenwick.update(index, actual.pixels() - estimated);
                }
            }
        }
    }

    fn line_count(&self) -> usize {
        self.measured_line_main.len()
    }

    fn record_line_measurement(&mut self, index: usize, actual: Px, estimated: Px) {
        if index >= self.measured_line_main.len() {
            return;
        }
        let previous = self.measured_line_main[index].map(MeasuredMain::pixels);
        if previous == Some(actual) {
            return;
        }

        let prev_delta = previous.map(|val| val - estimated).unwrap_or(Px::ZERO);
        let new_delta = actual - estimated;
        let delta_change = new_delta - prev_delta;
        self.measured_line_main[index] = Some(MeasuredMain::new(actual, self.density));
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

fn ensure_positive_px(px: Px) -> Px {
    if px <= Px::ZERO { Px(1) } else { px }
}

#[cfg(test)]
mod tests {
    use tessera_ui::RenderSlotWith;

    use tessera_ui::{
        AxisConstraint, ComputedData, Dp, LayoutResult, MeasurementError, Modifier,
        NoopDrawModifier, Px, PxPosition, State, layout::MeasureScope, remember, tessera,
    };

    use crate::{
        alignment::{CrossAxisAlignment, MainAxisAlignment},
        lazy_shared::LazyAxis,
        modifier::{ModifierExt as _, SemanticsArgs},
        scrollable::{ScrollBarBehavior, ScrollableController, scrollable},
        text::text,
        theme::{MaterialTheme, material_theme},
    };

    use super::{GridCells, LazyGridCache, LazyGridController, lazy_grid_view, lazy_vertical_grid};

    #[tessera]
    fn collapsed_viewport_fixture(
        observed: tessera_ui::testing::TestProbe<Option<(State<Dp>, State<LazyGridController>)>>,
    ) {
        let width = remember(|| Dp(300.0));
        let controller = remember(LazyGridController::new);
        observed.set(Some((width, controller)));
        material_theme().child(move || {
            lazy_vertical_grid()
                .columns(GridCells::fixed(2))
                .modifier(Modifier::new().width(width.get()).height(Dp(240.0)))
                .controller(controller)
                .cross_axis_spacing(Dp(8.0))
                .estimated_item_size(Dp(60.0))
                .items(4, |_| {
                    text("Keep measured grid geometry when the viewport temporarily collapses.");
                });
        });
    }

    #[test]
    fn collapsed_cells_preserve_measurements_even_with_nonzero_spacing() {
        let observed = tessera_ui::testing::TestProbe::default();
        let fixture_observed = observed.clone();
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(move || {
                collapsed_viewport_fixture(fixture_observed.clone());
            })
            .viewport_px(300, 240),
        );
        for frame in 0..8 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        let controller = observed.get().expect("grid handles").1;
        let before = controller.with(|state| state.cache.measured_line_main.clone());
        assert!(before.iter().all(Option::is_some));
        let width = observed.get().expect("grid handles").0;
        width.set(Dp(0.0));
        for frame in 8..12 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(
            controller.with(|state| state.cache.measured_line_main.clone()),
            before
        );
        width.set(Dp(300.0));
        for frame in 12..16 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(
            controller.with(|state| state.cache.measured_line_main.clone()),
            before
        );
    }

    #[test]
    fn density_changes_rebase_unvisited_lines_without_round_trip_drift() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        crate::test_utils::run_isolated_test(
            "lazy_grid::tests::density_changes_rebase_unvisited_lines_without_round_trip_drift",
            || {
                tessera_ui::dp::set_scale_factor(1.0);
                let mut cache = LazyGridCache::default();
                cache.configure(6, 2, Px(20));
                cache.record_line_measurement(0, Px(33), Px(20));
                cache.record_line_measurement(2, Px(55), Px(20));
                for (density, estimated, spacing, offset, total) in [
                    (1.25, 25, 5, 46, 144),
                    (1.5, 30, 6, 55, 173),
                    (2.0, 40, 8, 74, 232),
                    (1.0, 20, 4, 37, 116),
                ] {
                    tessera_ui::dp::set_scale_factor(density);
                    cache.configure(6, 2, Px(estimated));
                    assert_eq!(
                        cache.offset_for_line(1, Px(estimated), Px(spacing)),
                        Px(offset)
                    );
                    assert_eq!(cache.total_main_size(Px(estimated), Px(spacing)), Px(total));
                }
            },
        );
    }

    #[test]
    fn estimate_changes_rebase_line_offsets_and_column_changes_clear_measurements() {
        let mut cache = LazyGridCache::default();
        cache.configure(10, 2, Px(100));
        for (index, size) in [(0, 40), (1, 60), (3, 80)] {
            cache.record_line_measurement(index, Px(size), Px(100));
        }
        assert_eq!(cache.total_main_size(Px(100), Px(8)), Px(412));
        cache.configure(10, 2, Px(20));
        assert_eq!(cache.total_main_size(Px(20), Px(8)), Px(252));
        assert_eq!(cache.offset_for_line(2, Px(20), Px(8)), Px(116));
        assert_eq!(cache.line_index_for_offset(Px(116), Px(20), Px(8)), 2);
        cache.configure(10, 3, Px(20));
        assert_eq!(cache.total_main_size(Px(20), Px(8)), Px(104));
    }

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
            .axis(LazyAxis::Vertical)
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
                RenderSlotWith::new(|index| match index {
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
                }),
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
            .axis(LazyAxis::Horizontal)
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
                RenderSlotWith::new(|index| match index {
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
                }),
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
                height: Px::new(28),
            });
            c.set_scroll_position(target_position);
        });

        scrollable()
            .scrollbar_behavior(ScrollBarBehavior::Hidden)
            .modifier(Modifier::new().constrain(
                Some(AxisConstraint::exact(Px::new(60))),
                Some(AxisConstraint::exact(Px::new(28))),
            ))
            .vertical(true)
            .horizontal(false)
            .apply_child_offset(false)
            .controller(scroll_controller)
            .child(move || {
                lazy_grid_view()
                    .axis(LazyAxis::Vertical)
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
                        RenderSlotWith::new(|index| match index {
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
                        }),
                        None,
                    )])
                    .controller(controller)
                    .scroll_controller(scroll_controller);
            });
    }

    #[tessera]
    fn lazy_vertical_grid_stable_visible_items_case() {
        let controller = remember(LazyGridController::new);
        let scroll_controller = remember(ScrollableController::default);
        scroll_controller.with_mut(|c| {
            c.set_visible_size_for_test(ComputedData {
                width: Px::new(60),
                height: Px::new(50),
            });
        });

        // Rebuilds this component on every animated frame.
        let frame_clock = tessera_ui::remember(tessera_ui::current_frame_nanos);
        let _ = frame_clock.get();
        tessera_ui::launch_once(async move {
            let step = move |frame_nanos: u64| {
                let offset = match frame_nanos {
                    0 => 0,
                    16_666_667 => 1,
                    33_333_334 => 2,
                    _ => 2,
                };
                scroll_controller.with_mut(|c| {
                    c.set_scroll_position(PxPosition::new(Px::ZERO, Px::new(-offset)));
                });
                true
            };
            loop {
                let now = tessera_ui::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });

        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                lazy_grid_view()
                    .axis(LazyAxis::Vertical)
                    .grid_cells(GridCells::fixed(2))
                    .main_axis_spacing(Px::new(3))
                    .cross_axis_spacing(Px::new(3))
                    .cross_axis_alignment(MainAxisAlignment::Start)
                    .item_alignment(CrossAxisAlignment::Start)
                    .estimated_line_main(Px::new(20))
                    .overscan(0)
                    .padding_main(Px::new(4))
                    .padding_cross(Px::new(4))
                    .slots(vec![super::LazySlot::items(
                        20,
                        RenderSlotWith::new(|index| {
                            fixed_test_box()
                                .tag(format!("lazy_grid_stable_visible_{index}"))
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

    #[test]
    fn lazy_vertical_grid_does_not_rebuild_visible_items_when_scroll_stays_in_same_window() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                lazy_vertical_grid_stable_visible_items_case();
            })
            .viewport_px(80, 60),
        );

        let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 0);
        let first_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::WorkPhase::Build);
        assert!(first_builds > 0);

        let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 16_666_667);
        let second_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::WorkPhase::Build);
        assert_eq!(second_builds, 0);

        let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 33_333_334);
        let third_builds = session
            .work()
            .count("fixed_test_box", tessera_ui::WorkPhase::Build);
        assert_eq!(third_builds, 0);
    }
}
